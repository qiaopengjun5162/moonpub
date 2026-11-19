//! 用户文本进 HTML 之前必须转义。
//!
//! 2026-11-18：正文里一句 `<div>` 在成品里就是**真标签**。公众号后台会把我们
//! 给的 HTML 重新解析一遍，于是标题里的 `<div>` 开出一个真 div，把后面的段落
//! 全吞进这个 div（读者看到的是「文章少了一半」）；写 `<script>` 更狠，整段不
//! 显示。CommonMark 默认把 HTML 原样透传，但公众号不支持自定义标签，透传等于
//! 丢内容。
//!
//! 同一轮还查出三处**属性**注入：`:::figure` 的 `image:` 里带一个引号就能截断
//! `src`（`/a.png" onerror="alert(1)`），`:::book-info` 的 `cover:`、`:::photo-grid`
//! 的图片地址同理。属性值和文本节点都必须转义，不能只修看得见的那半边。
//!
//! 契约放在真实二进制上：调用方拿到的是 html 文件，看不到我们内部的渲染路径。

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn workspace(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("moonpub-escape-{name}-{}", std::process::id()));
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
    fs::read_to_string(
        cwd.join("Articles/drafts")
            .join(name.replace(".md", ".html")),
    )
    .expect("read rendered html")
}

/// 去掉标签、再把字符实体还原成读者看到的样子。
///
/// 顺序有讲究：先剥标签再反转义，否则 `&lt;div&gt;` 反转义出来的 `<div>`
/// 会被当成标签一起剥掉，断言就永远不可能失败。
fn readable(html: &str) -> String {
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
    out.replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
        .replace("&amp;", "&")
}

/// 正文、标题、粗体、行内代码、链接、列表、引用、表格里出现 `<` 都不许漏成标签。
#[test]
fn markup_in_every_block_position_stays_text() {
    let cwd = workspace("blocks");
    let html = render_article(
        &cwd,
        "escape.md",
        "---\ntitle: 转义探测\n---\n\n\
         # 标题里的 <div> & \"引号\"\n\n\
         正文里的 <div> 和 & 符号 和 \"引号\"。\n\n\
         **粗体 <b> & 符号**\n\n\
         `code <div> & \"q\"`\n\n\
         [链接 <i> & x](https://example.com/a?x=1&y=2)\n\n\
         - 列表里的 <div> & \"q\"\n\n\
         > 引用里的 <div> & \"q\"\n\n\
         | 表头 <div> | 第二列 |\n| --- | --- |\n| 单元格 <div> | 值 |\n",
    );

    for tag in ["<div", "<b>", "<i>", "<script"] {
        assert!(
            !html.contains(tag),
            "用户文本漏成真标签 {tag}（后台会把后面内容吞掉）：{html}"
        );
    }

    let text = readable(&html);
    for needle in [
        "标题里的 <div> & \"引号\"",
        "正文里的 <div> 和 & 符号",
        "粗体 <b> & 符号",
        "code <div> & \"q\"",
        "链接 <i> & x",
        "列表里的 <div> & \"q\"",
        "引用里的 <div> & \"q\"",
        // 表格按「术语卡」渲染：首列是标签、其余列拼成值，表头本身不显示。
        "单元格 <div>",
    ] {
        assert!(text.contains(needle), "读者看不到 {needle}：{text}");
    }

    // 链接目标里的 `&` 必须落成 `&amp;`，否则属性会被截断成 `x=1`。
    assert!(html.contains("href=\"https://example.com/a?x=1&amp;y=2\""));
    assert!(!html.contains("x=1&y=2\""), "href 里的 & 没转义：{html}");

    fs::remove_dir_all(cwd).ok();
}

/// 卡片属性同样是作者写的正文，也要转义——包括进 `src` 的那一半。
#[test]
fn card_props_cannot_break_out_of_the_markup() {
    let cwd = workspace("cards");
    let html = render_article(
        &cwd,
        "cards.md",
        "---\ntitle: 卡片转义\n---\n\n\
         :::callout\n\
         label: 标签 <div> & 方块\n\
         正文 <div> & \"q\"\n\
         :::\n\n\
         :::figure\n\
         image: /a.png\" onerror=\"alert(1)\n\
         caption: 图注 <div> & 方块\n\
         :::\n\n\
         :::book-info\n\
         title: 书 <div>\n\
         author: 作者 <div>\n\
         cover: /c.png\" onerror=\"alert(2)\n\
         :::\n\n\
         :::photo-grid\n\
         - /p.png\" onerror=\"alert(3) | 图 <div>\n\
         :::\n\n\
         :::quote-card\n\
         引用 <div> & \"q\"\n\
         :::\n",
    );

    assert!(!html.contains("<div"), "卡片属性漏成真标签：{html}");
    // 属性截断的判据：转义之后 `onerror="` 这种「真引号 + 属性名」不该存在。
    assert!(
        !html.contains("onerror=\""),
        "属性被引号截断了，能注入事件处理器：{html}"
    );

    let text = readable(&html);
    // 属性值里的引号会被属性解析器吃掉（那是另一条契约），这里只验标记字符。
    for needle in ["标签 <div> & 方块", "图注 <div>", "书 <div>", "作者 <div>"] {
        assert!(text.contains(needle), "读者看不到 {needle}：{text}");
    }

    fs::remove_dir_all(cwd).ok();
}

/// 危险协议不许进入 `href` / `src`：转义只防得住「引号截断属性」，防不住
/// 浏览器真的去解析一个合法的 `javascript:`（本地预览是在浏览器里打开的）。
#[test]
fn dangerous_url_schemes_never_reach_an_attribute() {
    let cwd = workspace("url-schemes");
    let html = render_article(
        &cwd,
        "url.md",
        "正文 [js](javascript:alert(1)) [大写](JavaScript:alert(2)) \
         [空格式](  vbscript:msgbox(1)) [数据](data:text/html;base64,PHNjcmlwdD4=) \
         [本地文件](file:///etc/passwd) [正常](https://example.com) \
         [引用式][x] 与 ![图片](javascript:alert(3))\n\n\
         [x]: javascript:alert(4)\n\n\
         :::figure\n\
         image: javascript:alert(5)\n\
         caption: 危险图注\n\
         :::\n\n\
         :::compact-links\n\
         - 01 | 标题 | 来源 | javascript:alert(6)\n\
         :::\n",
    );

    let lower = html.to_ascii_lowercase();
    for prefix in [
        "href=\"javascript:",
        "href=\"vbscript:",
        "href=\"data:",
        "href=\"file:",
        "src=\"javascript:",
        "src=\"vbscript:",
        "src=\"data:",
        "src=\"file:",
    ] {
        assert!(
            !lower.contains(prefix),
            "危险协议漏进属性（{prefix}）：{html}"
        );
    }

    // 正常目标不能顺手一起干掉
    assert!(
        html.contains("href=\"https://example.com\""),
        "正常链接被误伤：{html}"
    );

    fs::remove_dir_all(cwd).ok();
}
