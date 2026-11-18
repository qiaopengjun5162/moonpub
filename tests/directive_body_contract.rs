//! `:::name` 块正文的进程边界契约。
//!
//! 2026-11-18：`:::` 块的正文一直被当成「一串行内文本」渲染（`inline_md(body)`），
//! 于是正文里的块级语法整类降级：``` 围栏变成三个空的反引号 span + 一整段塞进行内
//! `<span>` 的代码，语言标记 `ts` 印进正文，换行和缩进全靠浏览器猜；`:::` 里放一段
//! 教 `:::callout` 的示例代码时，`parse_blocks` 还会在那个 `:::` 处把文档切开，
//! 代码块被截成两半。
//!
//! 缺陷都在渲染层内部，但**用户拿到的是 html 文件**，所以契约必须落在真实二进制上。

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn workspace(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "moonpub-directive-body-{name}-{}",
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

/// 只留可见文本：标签里的 `style="…"` 不是正文。
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
fn code_fence_inside_a_card_survives_as_a_code_block() {
    let cwd = workspace("card-code");
    let html = render_article(
        &cwd,
        "card-code.md",
        "---\ntitle: 卡片里的代码\n---\n\n:::card\n```ts\nconst opts: any = {limit: 10};\nconst tail = 1;\n```\n:::\n",
    );

    // 每行代码一个等宽段落，而不是一整坨行内 span。
    assert_eq!(
        html.matches("font-family:Menlo").count(),
        2,
        "两份代码没有各自的等宽段落：{html}"
    );
    let text = visible_text(&html);
    assert!(
        !text.split_whitespace().any(|word| word == "ts"),
        "语言标记漏进正文：{html}"
    );
    assert!(text.contains("const opts: any = {limit: 10};"), "{html}");
    assert!(text.contains("const tail = 1;"), "{html}");
}

#[test]
fn directive_line_inside_a_code_fence_does_not_truncate_the_block() {
    let cwd = workspace("fence-directive");
    let html = render_article(
        &cwd,
        "fence-directive.md",
        "---\ntitle: 教学示例\n---\n\n```markdown\n:::callout\n重点\n:::\n```\n\n正文一段。\n",
    );

    let text = visible_text(&html);
    assert!(text.contains(":::callout"), "示例代码被截断：{html}");
    assert!(text.contains("重点"), "示例代码被截断：{html}");
    assert!(text.contains("正文一段"), "围栏后面的正文丢了：{html}");
    // 围栏里的 `:::` 不该开出一个真的 callout 卡片。
    assert!(
        !html.contains("letter-spacing:0.12em;\">重点</p>"),
        "代码里的 `:::` 被当成了指令：{html}"
    );
}

#[test]
fn block_body_inside_a_summary_is_not_nested_in_a_paragraph() {
    let cwd = workspace("summary-block");
    let html = render_article(
        &cwd,
        "summary-block.md",
        "---\ntitle: 总结里的代码\n---\n\n:::summary\n```bash\nmoonpub render a.md\n```\n:::\n",
    );

    // 块级元素嵌在 <p> 里会被浏览器提前闭合成空段落，微信端会再多拆一层。
    assert!(
        !html.contains("<p style=\"margin:0;font-size:14px"),
        "块级正文仍被塞进 <p>：{html}"
    );
    assert!(html.contains("font-family:Menlo"), "代码块丢了：{html}");
}

#[test]
fn plain_summary_body_keeps_the_compact_paragraph() {
    let cwd = workspace("summary-plain");
    let html = render_article(
        &cwd,
        "summary-plain.md",
        "---\ntitle: 纯文字总结\n---\n\n:::summary\n三句话讲完这件事。\n:::\n",
    );

    // 反向保险：修代码块不能把所有 `:::summary` 都改成块级排版。
    assert!(
        html.contains("<p style=\"margin:0;font-size:14px"),
        "纯文字正文被改了排版：{html}"
    );
    assert!(!html.contains("font-family:Menlo"), "{html}");
}
