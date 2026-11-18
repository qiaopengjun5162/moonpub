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

/// 模拟公众号后台：删掉「只含空白的文本节点」，保留文本节点内部的空格。
///
/// 线上实测（2026-11-18）：我们输出 `<span>npx</span> <span>tsx</span>`，
/// 成品里那个空格整段消失；同一个文本节点内部的空格（正文 `20 条签名`）
/// 完好无损。所以这个删法必须被模拟进契约，不能只靠肉眼看。
fn drop_blank_text_nodes(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    let mut pending_ws = String::new();
    for ch in html.chars() {
        if in_tag {
            out.push(ch);
            if ch == '>' {
                in_tag = false;
            }
        } else if ch == '<' {
            pending_ws.clear();
            out.push(ch);
            in_tag = true;
        } else if ch.is_whitespace() {
            pending_ws.push(ch);
        } else {
            out.push_str(&pending_ws);
            pending_ws.clear();
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

/// 列表容器的 `<section>` 片段。列表项在 `<p>` 里、不嵌套 `<section>`，
/// 所以第一个 `</section>` 就是容器的结束。
fn list_block(html: &str) -> &str {
    let start = html
        .find("<section style=\"margin:18px 0 22px;\">")
        .expect("列表容器缺失");
    let rest = &html[start..];
    let end = rest.find("</section>").expect("列表容器没有收尾");
    &rest[..end]
}

/// `- 第一项` 下面缩进的一行是这一项的继续，不是新段落。此前它掉出列表变成顶格
/// 段落，读者看到「列表只有一行，剩下是另一段话」——用户拿到的是成品，只能在
/// 成品上验。
#[test]
fn list_continuation_line_renders_inside_the_list() {
    let cwd = workspace("list-continuation");
    let html = render_article(
        &cwd,
        "list.md",
        "行为说明：\n\n- 输出都以 next: 收尾\n  给出推进这条链的命令，外加一行理由。\n  包还没渲染时建议 render。\n\n之后是新段落。\n",
    );

    let list = list_block(&html);
    assert!(
        list.contains("给出推进这条链的命令"),
        "续行掉出了列表：{html}"
    );
    assert!(
        list.contains("包还没渲染时建议"),
        "第二行续行也掉了：{html}"
    );

    // 段首残留的缩进不该出现在可见正文里。
    assert!(
        !visible_text(&html).contains("\n  给出"),
        "续行的原始缩进漏进了正文：{html}"
    );
    assert!(html.contains("之后是新段落。"));

    fs::remove_dir_all(cwd).ok();
}

/// 空行是用户明确的分段信号。跨过它继续拼进列表项，就是把两段话粘成一段。
#[test]
fn blank_line_still_ends_the_list_item() {
    let cwd = workspace("list-blank");
    let html = render_article(&cwd, "blank.md", "- 第一项\n\n  另一段话。\n");

    let list = list_block(&html);
    assert!(
        !list.contains("另一段话"),
        "空行之后的缩进段落被吞进了列表项：{html}"
    );
    assert!(html.contains("另一段话。"));

    fs::remove_dir_all(cwd).ok();
}

/// 一段写三行是中文写作最正常的样子。此前每行各印一个 `<p>`，一段变三段、每段之间
/// 1.5em 段距——用户写的是一段话，读者看到的是三个短句。
#[test]
fn soft_line_breaks_render_as_one_paragraph() {
    let cwd = workspace("soft-break");
    let html = render_article(
        &cwd,
        "soft.md",
        "这是第一段的第一行，\n这是第一段的第二行，\n这是第一段的第三行。\n\n这是第二段。\n",
    );

    assert_eq!(
        html.matches("margin: 0 0 1.5em").count(),
        2,
        "两个空行分隔的段落应各自一个 <p>：{html}"
    );
    assert_eq!(html.matches("<br>").count(), 2, "段内换行丢了：{html}");
    let text = visible_text(&html);
    for line in [
        "第一段的第一行",
        "第一段的第二行",
        "第一段的第三行",
        "这是第二段",
    ] {
        assert!(text.contains(line), "「{line}」丢了：{html}");
    }

    fs::remove_dir_all(cwd).ok();
}

/// 段落缓冲一旦漏 flush，内容顺序就会错——读者能看见，但很难归因到渲染器。
#[test]
fn blocks_keep_their_document_order() {
    let cwd = workspace("block-order");
    let html = render_article(
        &cwd,
        "order.md",
        "段前\n\n## 标题\n\n- 项\n\n| a | b |\n| --- | --- |\n| 1 | 2 |\n\n段后\n",
    );

    let at = |needle: &str| {
        html.find(needle)
            .unwrap_or_else(|| panic!("「{needle}」丢了：{html}"))
    };
    assert!(
        at("段前") < at("标题") && at("标题") < at("项") && at("项") < at("段后"),
        "块级元素顺序错乱：{html}"
    );

    fs::remove_dir_all(cwd).ok();
}

/// 波浪线围栏是 CommonMark 的第二种代码围栏。此前只有反引号被认，`~~~bash`
/// 整块漏成正文：围栏标记自己印成一段，里面的代码被行内语法重新解析。
#[test]
fn tilde_fenced_code_is_not_reparsed_as_prose() {
    let cwd = workspace("tilde-fence");
    let html = render_article(
        &cwd,
        "tilde.md",
        "下面是一段 shell 示例：\n\n~~~bash\n# 这是注释\necho \"**不该加粗**\"\n- 这不是列表\n~~~\n\n结束。\n",
    );

    assert!(!html.contains("<h2"), "代码里的井号成了标题：{html}");
    assert!(!html.contains("<strong"), "代码里的星号成了粗体：{html}");
    assert!(!html.contains('•'), "代码里的短横成了列表项：{html}");
    assert!(
        visible_text(&html).contains("# 这是注释"),
        "代码原文丢了：{html}"
    );
    assert!(
        !visible_text(&html).contains("~~~"),
        "围栏标记漏进读者可见的正文：{html}"
    );

    fs::remove_dir_all(cwd).ok();
}

/// 围栏里的链接定义是教学示例。此前它被当成真定义从正文里抽走：代码块少一行，
/// 围栏外的同名引用还会指向「代码里的」地址——读的人完全看不出发生了什么。
#[test]
fn link_definition_inside_a_tilde_fence_stays_in_the_code() {
    let cwd = workspace("tilde-link-def");
    let html = render_article(
        &cwd,
        "tilde-link.md",
        "~~~markdown\n[docs]: https://example.com/x\n~~~\n\n正文用 [文档][docs]。\n",
    );

    assert!(
        visible_text(&html).contains("[docs]:"),
        "定义行被抽走了：{html}"
    );
    assert!(
        !html.contains("href=\"https://example.com/x\""),
        "围栏里的定义被当成真定义：{html}"
    );

    fs::remove_dir_all(cwd).ok();
}

/// 代码行的排版不许依赖 `white-space`。
///
/// 线上实测：代码块在公众号后台变成「一个 token 一行」。代码行当时靠
/// `white-space:pre-wrap` 保住空格，而后台编辑器会把我们给的 HTML 重新
/// 序列化（在元素之间插换行），那些换行落进 `pre-wrap` 就成了真实断行。
/// 换行本来就由「一行一个 `<p>`」保证，缩进与多空格由 `&nbsp;` 保证，
/// 所以这条样式必须没有。
#[test]
fn code_block_does_not_lean_on_white_space() {
    let cwd = workspace("code-nowrap");
    let html = render_article(
        &cwd,
        "code.md",
        "```ts\nconst opts: any = {limit: 10};\n\tif (x) {\n        deep();\n    }\n```\n",
    );

    assert!(
        !html.contains("white-space"),
        "代码行又依赖 white-space 了：{html}"
    );
    assert!(html.contains("font-family:Menlo"), "代码块丢了：{html}");
    assert!(!html.contains('\t'), "制表符漏进成品：{html}");
    assert!(
        html.contains("&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;"),
        "8 空格缩进没落成 &nbsp;：{html}"
    );

    fs::remove_dir_all(cwd).ok();
}

/// 词与词之间的空格不能是「只含空白的文本节点」。
///
/// 线上实测：代码块每个 token 都是一个独立 `<span>`，词间那个裸空格被后台
/// 整段删掉，`npx tsx simple-indexer.ts` 变成 `npxtsxsimple-indexer.ts`，
/// 整行命令连成一片。空格必须是 `&nbsp;` 这种字符，删不掉。
#[test]
fn code_block_words_survive_blank_node_stripping() {
    let cwd = workspace("code-spaces");
    let html = render_article(
        &cwd,
        "code-spaces.md",
        "```sh\nnpx tsx simple-indexer.ts \\\n  --limit 40 --reset\n```\n",
    );

    assert!(html.contains("font-family:Menlo"), "代码块丢了：{html}");
    assert!(
        !html.contains("</span> <span"),
        "词间又留裸空格了，后台会删掉：{html}"
    );

    let text = visible_text(&drop_blank_text_nodes(&html)).replace("&nbsp;", " ");
    assert!(
        text.contains("npx tsx simple-indexer.ts \\"),
        "空格被删后命令粘成一片：{text}"
    );
    assert!(
        text.contains("  --limit 40 --reset"),
        "缩进或参数粘住了：{text}"
    );

    fs::remove_dir_all(cwd).ok();
}
