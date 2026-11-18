//! 渲染语法的进程边界契约。
//!
//! 2026-11-18：`标题` + 下一行 `====`（setext 标题的合法写法）会掉进行内渲染器，
//! 被 `==高亮==` 语法吃成两个空 `<mark>` 加半个 `==`——标题降级成正文，读者看到
//! 一串标签残渣。缺陷在 `render_markdown_segment` 里，但**用户拿到的是 html 文件**，
//! 所以这层契约必须在真实二进制上验：调用方只看得到成品，看不到我们内部的段落结构。
//!
//! 同时锁住这次的刻意取舍：`---` 不收成标题（它在真实语料里更常是分隔线 /
//! frontmatter 边界），正文下面的 `---` 仍要渲染成分隔线。

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn workspace(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "moonpub-render-syntax-{name}-{}",
        std::process::id()
    ));
    fs::create_dir_all(root.join("Articles/drafts")).expect("create temp workspace");
    root
}

fn run(cwd: &PathBuf, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_moonpub"))
        .args(args)
        .current_dir(cwd)
        .env_remove("WECHAT_APPID")
        .env_remove("WECHAT_SECRET")
        .output()
        .expect("run moonpub binary")
}

/// 只留可见文本：标签内的 `style="…=…"` 不是正文，不能算进「等号漏出来」的判定里。
fn visible_text(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        if in_tag {
            if ch == '>' {
                in_tag = false;
            }
        } else if ch == '<' {
            in_tag = true;
        } else {
            out.push(ch);
        }
    }
    out
}

fn render_article(cwd: &PathBuf, name: &str, markdown: &str) -> String {
    let article = cwd.join("Articles/drafts").join(name);
    fs::write(&article, markdown).expect("write article");
    let arg = format!("Articles/drafts/{name}");
    let output = run(cwd, &["render", &arg]);
    assert!(
        output.status.success(),
        "render must succeed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    fs::read_to_string(article.with_extension("html")).expect("read rendered html")
}

#[test]
fn setext_heading_reaches_the_html_as_a_heading() {
    let cwd = workspace("setext");
    let html = render_article(&cwd, "setext.md", "社区营造\n====\n\n正文一段。\n");

    assert!(html.contains("<h2"), "标题标签缺失：{html}");
    assert!(html.contains("社区营造"), "标题文字缺失：{html}");
    assert!(!html.contains("<mark"), "下划线被当成高亮标记：{html}");
    assert!(
        !visible_text(&html).contains('='),
        "等号漏进读者可见的正文：{html}"
    );

    fs::remove_dir_all(cwd).ok();
}

#[test]
fn dashes_after_a_paragraph_stay_a_divider() {
    let cwd = workspace("divider");
    let html = render_article(&cwd, "divider.md", "正文一段。\n\n---\n\n后一段。\n");

    assert!(!html.contains("<h2"), "`---` 不该被升级成标题：{html}");
    assert!(html.contains("正文一段。"), "正文丢了：{html}");
    assert!(html.contains("后一段。"), "后一段丢了：{html}");
    assert!(
        !visible_text(&html).contains("---"),
        "分隔线印成了正文：{html}"
    );

    fs::remove_dir_all(cwd).ok();
}
