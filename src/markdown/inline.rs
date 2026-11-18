use crate::theme;

pub(super) fn inline_md(text: &str, theme: &theme::Theme) -> String {
    let mut s = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '`' {
            let end = chars[i + 1..].iter().position(|&c| c == '`');
            if let Some(rel) = end {
                let code: String = chars[i + 1..i + 1 + rel].iter().collect();
                let color_style = if theme.code_color.is_empty() {
                    String::new()
                } else {
                    format!("color:{};", theme.code_color)
                };
                // 微信正文不支持 <code>：编辑器会剥掉标签但把内容当独立段落输出，
                // 一段带 3 个行内代码的句子会额外多出 3 行重复文本。用 span + 内联样式。
                s.push_str(&format!(
                    "<span style=\"background:{};{}padding:2px 4px;border-radius:3px;font-size:14px;font-family:monospace;\">{}</span>",
                    theme.code_bg,
                    color_style,
                    html_escape(&code)
                ));
                i += rel + 2;
                continue;
            }
        }
        // `***粗斜***`：必须排在 `**` 之前，否则先命中粗体分支，
        // 中间的 `*` 被当正文，收尾还剩一个裸星号。
        if chars[i] == '*'
            && chars.get(i + 1) == Some(&'*')
            && chars.get(i + 2) == Some(&'*')
            && let Some(rel) = chars[i + 3..].windows(3).position(|w| w == ['*', '*', '*'])
        {
            let inner: String = chars[i + 3..i + 3 + rel].iter().collect();
            s.push_str(&format!(
                "<strong style=\"color: {};\"><em>{}</em></strong>",
                theme.heading_color,
                inline_md(&inner, theme)
            ));
            i += rel + 6;
            continue;
        }
        if chars[i] == '*' && i + 1 < chars.len() && chars[i + 1] == '*' {
            let end = chars[i + 2..].windows(2).position(|w| w == ['*', '*']);
            if let Some(rel) = end {
                let inner: String = chars[i + 2..i + 2 + rel].iter().collect();
                s.push_str(&format!(
                    "<strong style=\"color: {};\">{}</strong>",
                    theme.heading_color,
                    inline_md(&inner, theme)
                ));
                i += rel + 4;
                continue;
            }
        }
        if chars[i] == '=' && i + 1 < chars.len() && chars[i + 1] == '=' {
            let end = chars[i + 2..].windows(2).position(|w| w == ['=', '=']);
            if let Some(rel) = end {
                let inner: String = chars[i + 2..i + 2 + rel].iter().collect();
                s.push_str(&format!(
                    "<mark style=\"background:{};color:{};padding:1px 4px;border-radius:3px;\">{}</mark>",
                    theme.accent_soft,
                    theme.heading_color,
                    inline_md(&inner, theme)
                ));
                i += rel + 4;
                continue;
            }
        }
        if chars[i] == '~' && i + 1 < chars.len() && chars[i + 1] == '~' {
            let end = chars[i + 2..].windows(2).position(|w| w == ['~', '~']);
            if let Some(rel) = end {
                let inner: String = chars[i + 2..i + 2 + rel].iter().collect();
                s.push_str(&format!(
                    "<del style=\"color:{};text-decoration-color:{};\">{}</del>",
                    theme.text_muted,
                    theme.accent,
                    inline_md(&inner, theme)
                ));
                i += rel + 4;
                continue;
            }
        }
        if chars[i] == '*' {
            let end = chars[i + 1..].iter().position(|&c| c == '*');
            if let Some(rel) = end {
                let inner: String = chars[i + 1..i + 1 + rel].iter().collect();
                s.push_str(&format!("<em>{}</em>", inline_md(&inner, theme)));
                i += rel + 2;
                continue;
            }
        }
        if chars[i] == '!'
            && i + 1 < chars.len()
            && chars[i + 1] == '['
            && let Some((alt, url, consumed)) = parse_image(&chars[i..])
        {
            // 块级图片（整行只有一张图）由 plain.rs 的 render_image_figure 负责，
            // 走不到这里。这里只会是**段落中间**的行内图片，所以不能吐 <p>：
            // 外层已经是 <p>，套一层会让微信编辑器把段落拆坏。用块级样式的
            // 行内 <img>，语义仍然是流内元素。
            s.push_str(&format!(
                "<img src=\"{}\" alt=\"{}\" style=\"max-width: 100%; display: block; margin: 1.5em auto; border-radius: 5px;\" />",
                html_escape(&url),
                html_escape(&alt)
            ));
            i += consumed;
            continue;
        }
        if chars[i] == '['
            && let Some((text, url, consumed)) = parse_link(&chars[i..])
        {
            s.push_str(&format!(
                "<a href=\"{}\" style=\"color:{};font-weight:bold;text-decoration:none;border-bottom:1px solid {};\">{}</a>",
                html_escape(&url),
                theme.heading_border,
                theme.accent_soft,
                inline_md(&text, theme)
            ));
            i += consumed;
            continue;
        }
        if chars[i] == '<'
            && let Some((url, consumed)) = parse_autolink(&chars[i..])
        {
            // `<https://example.com>` 是 GFM 自动链接。尖括号是语法标记，
            // 不能进 href，也不能当正文输出。
            let escaped = html_escape(&url);
            s.push_str(&format!(
                "<a href=\"{escaped}\" style=\"color:{};font-weight:bold;text-decoration:none;border-bottom:1px solid {};word-break:break-all;\">{escaped}</a>",
                theme.heading_border,
                theme.accent_soft
            ));
            i += consumed;
            continue;
        }
        if starts_url(&chars, i) {
            let (url, consumed) = take_url(&chars[i..]);
            let escaped_url = html_escape(&url);
            s.push_str(&format!(
                "<a href=\"{escaped_url}\" style=\"color:{};font-weight:bold;text-decoration:none;border-bottom:1px solid {};word-break:break-all;\">{escaped_url}</a>",
                theme.heading_border,
                theme.accent_soft
            ));
            i += consumed;
            continue;
        }
        s.push(chars[i]);
        i += 1;
    }
    s
}

fn starts_url(chars: &[char], index: usize) -> bool {
    chars
        .get(index..index.saturating_add(8))
        .is_some_and(|slice| slice.iter().collect::<String>().starts_with("https://"))
        || chars
            .get(index..index.saturating_add(7))
            .is_some_and(|slice| slice.iter().collect::<String>().starts_with("http://"))
}

fn take_url(chars: &[char]) -> (String, usize) {
    let mut consumed = 0;
    let mut url = String::new();
    for ch in chars {
        if ch.is_whitespace()
            || matches!(
                ch,
                '，' | '。' | '；' | '、' | '）' | ')' | ']' | '》' | '<' | '>'
            )
        {
            break;
        }
        url.push(*ch);
        consumed += 1;
    }
    (url, consumed)
}

fn parse_link(chars: &[char]) -> Option<(String, String, usize)> {
    if chars.first() != Some(&'[') {
        return None;
    }
    let text_end = chars[1..].iter().position(|&c| c == ']')?;
    let text: String = chars[1..1 + text_end].iter().collect();
    let rest = &chars[1 + text_end + 1..];
    let url_end = destination_end(rest)?;
    let url = split_destination(&unescape_parens(
        &rest[1..url_end].iter().collect::<String>(),
    ));
    Some((text, url, 1 + text_end + 1 + url_end + 1))
}

pub(super) fn parse_image(chars: &[char]) -> Option<(String, String, usize)> {
    if chars.len() < 5 || chars[0] != '!' || chars[1] != '[' {
        return None;
    }
    let alt_end = chars[2..].iter().position(|&c| c == ']')?;
    let alt: String = chars[2..2 + alt_end].iter().collect();
    let rest = &chars[2 + alt_end + 1..];
    let url_end = destination_end(rest)?;
    let url = split_destination(&unescape_parens(
        &rest[1..url_end].iter().collect::<String>(),
    ));
    Some((alt, url, 2 + alt_end + 1 + url_end + 1))
}

/// 识别 `<https://example.com>` 这种自动链接，返回目标与消费的字符数。
/// 尖括号里必须是 `http(s)://` 开头且不含空白，否则当普通文本处理。
fn parse_autolink(chars: &[char]) -> Option<(String, usize)> {
    if chars.first() != Some(&'<') {
        return None;
    }
    let close = chars[1..].iter().position(|&c| c == '>')?;
    let inner: String = chars[1..1 + close].iter().collect();
    if !(inner.starts_with("https://") || inner.starts_with("http://")) {
        return None;
    }
    if inner.chars().any(char::is_whitespace) {
        return None;
    }
    Some((inner, close + 2))
}

/// 从 `(...)` 里的原文切出真正的 URL。
///
/// GFM 允许在目标后面跟标题：`[文字](https://example.com "标题")`，单引号、双引号
/// 都合法。老实现把整段当 URL，`href` 变成 `https://example.com &quot;标题&quot;`，
/// 点开就是死链。标题在微信里没有可见价值（`title` 属性会被编辑器剥掉），直接丢掉。
///
/// 目标写成 `<https://example.com/a b>` 时，尖括号里允许空格——此时剥掉尖括号、
/// 保留空格，不能按空格切标题。
fn split_destination(raw: &str) -> String {
    let raw = raw.trim();
    if let Some(inner) = raw
        .strip_prefix('<')
        .and_then(|rest| rest.strip_suffix('>'))
    {
        return inner.to_owned();
    }
    match raw.find(char::is_whitespace) {
        Some(pos) => raw[..pos].to_owned(),
        None => raw.to_owned(),
    }
}

/// 找出 `(...)` 里真正收尾的那个 `)` 的下标（`rest[0]` 必须已经是 `(`）。
///
/// 链接目标里允许出现成对括号——`https://zh.wikipedia.org/wiki/Rust_(编程语言)` 是
/// 维基百科最常见的形态。老实现取「第一个 `)`」，会把这类 URL 截断成死链，还把
/// 收尾括号漏进正文。这里按括号深度配对找结尾；`\(` / `\)` 视为字面量，不参与配对。
///
/// 深度始终没归零（例如 `/a(b` 只有左括号）时退回「第一个 `)`」的老口径，
/// 保证原本能用的链接不会退化成纯文本。
fn destination_end(rest: &[char]) -> Option<usize> {
    if rest.first() != Some(&'(') {
        return None;
    }
    let mut depth = 0usize;
    let mut first_close: Option<usize> = None;
    let mut i = 1;
    while i < rest.len() {
        match rest[i] {
            '\\' if matches!(rest.get(i + 1), Some('(') | Some(')')) => {
                i += 2;
                continue;
            }
            '(' => depth += 1,
            ')' => {
                if depth == 0 {
                    return Some(i);
                }
                depth -= 1;
                if first_close.is_none() {
                    first_close = Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    first_close
}

/// 把目标里的 `\(` / `\)` 还原成括号本身；其余反斜杠原样保留。
fn unescape_parens(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch == '\\' && matches!(chars.peek(), Some('(') | Some(')')) {
            out.push(chars.next().unwrap_or_default());
            continue;
        }
        out.push(ch);
    }
    out
}

fn html_escape(s: &str) -> String {
    crate::html::escape(s)
}

#[cfg(test)]
mod tests {
    use super::html_escape;
    use super::inline_md;
    use super::parse_image;
    use crate::theme::Theme;

    fn test_theme() -> Theme {
        Theme::from_name("porcelain")
    }

    #[test]
    fn plain_text_passes_through() {
        let html = inline_md("Hello 世界", &test_theme());
        assert_eq!(html, "Hello 世界");
    }

    #[test]
    fn backtick_inline_code() {
        let html = inline_md("使用 `let x = 1;` 语句", &test_theme());
        assert!(
            html.contains("<span style=\"background"),
            "行内代码应渲染为 span"
        );
        assert!(!html.contains("<code"), "微信不支持 code 标签");
        assert!(html.contains("let x = 1;"));
        assert!(html.contains("</span>"));
    }

    #[test]
    fn double_star_bold() {
        let html = inline_md("这是 **重要** 内容", &test_theme());
        assert!(html.contains("<strong"));
        assert!(html.contains("重要"));
        assert!(html.contains("</strong>"));
    }

    #[test]
    fn double_equal_highlight() {
        let html = inline_md("请注意 ==这个重点==", &test_theme());
        assert!(html.contains("<mark"));
        assert!(html.contains("这个重点"));
        assert!(html.contains("</mark>"));
    }

    #[test]
    fn double_tilde_strikethrough() {
        let html = inline_md("~~已废弃~~ 不再使用", &test_theme());
        assert!(html.contains("<del"));
        assert!(html.contains("已废弃"));
        assert!(html.contains("</del>"));
    }

    #[test]
    fn single_star_italic() {
        let html = inline_md("这是 *斜体* 文字", &test_theme());
        assert!(html.contains("<em>"));
        assert!(html.contains("斜体"));
        assert!(html.contains("</em>"));
    }

    #[test]
    fn links_are_highlighted_and_bold() {
        let theme = test_theme();
        let html = inline_md(
            "项目主页：https://github.com/qiaopengjun5162/moonpub 和 [文档](https://example.com/docs)",
            &theme,
        );
        assert!(html.contains("font-weight:bold"));
        assert!(html.contains("border-bottom:1px solid"));
        assert!(html.contains("https://github.com/qiaopengjun5162/moonpub"));
        assert!(html.contains("href=\"https://example.com/docs\""));
    }

    #[test]
    fn bare_url_is_linked() {
        let html = inline_md("访问 https://example.com 了解更多", &test_theme());
        assert!(html.contains("<a href=\"https://example.com\""));
        assert!(html.contains("example.com"));
        assert!(html.contains("</a>"));
    }

    #[test]
    fn markdown_image_renders_img_tag() {
        let html = inline_md("看图 ![logo](/images/logo.png)", &test_theme());
        assert!(html.contains("<img src=\"/images/logo.png\""));
        assert!(html.contains("alt=\"logo\""));
    }

    #[test]
    fn html_special_chars_are_escaped() {
        assert_eq!(html_escape("a&b<c>d\"e"), "a&amp;b&lt;c&gt;d&quot;e");
    }

    #[test]
    fn empty_text_returns_empty() {
        assert_eq!(inline_md("", &test_theme()), "");
    }

    #[test]
    fn bold_with_escape_characters() {
        let html = inline_md("**a & b < c > d**", &test_theme());
        assert!(html.contains("<strong style=\"color: #142333;\">a & b < c > d</strong>"));
    }

    #[test]
    fn inline_code_escapes_html() {
        let html = inline_md("`a < b > c & d`", &test_theme());
        assert!(html.contains("a &lt; b &gt; c &amp; d"));
    }

    #[test]
    fn parse_image_returns_none_for_non_image() {
        assert!(parse_image(&['[', 't', 'e', 'x', 't', ']', '(', ')']).is_none());
    }

    #[test]
    fn parse_image_returns_none_for_short_input() {
        assert!(parse_image(&['!']).is_none());
    }

    #[test]
    fn link_destination_keeps_balanced_parentheses() {
        let html = inline_md(
            "[维基](https://zh.wikipedia.org/wiki/Rust_(编程语言))",
            &test_theme(),
        );
        assert!(
            html.contains("href=\"https://zh.wikipedia.org/wiki/Rust_(编程语言)\""),
            "成对括号必须留在链接里，不能在第一处 `)` 截断: {html}"
        );
        assert!(
            !html.contains("</a>)"),
            "收尾括号不能漏进正文，否则链接是死的: {html}"
        );
    }

    #[test]
    fn escaped_parenthesis_in_destination_is_unescaped() {
        let html = inline_md("[文档](/guide/a\\(1\\))", &test_theme());
        assert!(
            html.contains("href=\"/guide/a(1)\""),
            "转义括号应还原成字面量: {html}"
        );
    }

    #[test]
    fn unbalanced_open_paren_falls_back_to_the_first_closing_paren() {
        // 目标里只有左括号（没有配平）：沿用「第一个 `)` 收尾」的老口径，
        // 不让原本能用的链接退化成纯文本。
        let html = inline_md("[x](/a(b)", &test_theme());
        assert!(html.contains("href=\"/a(b\""), "{html}");
    }

    #[test]
    fn inline_image_never_nests_a_paragraph_inside_a_paragraph() {
        let html = inline_md("前缀 ![图](/x.png) 后缀", &test_theme());
        assert!(
            !html.contains("<p"),
            "行内图片不能吐 <p>：外层段落会套段落，微信编辑器会拆坏: {html}"
        );
        assert!(html.contains("<img src=\"/x.png\""), "{html}");
        assert!(html.contains("前缀 ") && html.contains(" 后缀"), "{html}");
    }

    #[test]
    fn image_src_and_alt_are_html_escaped() {
        let html = inline_md("![a\"b](/x.png?a=1&b=2)", &test_theme());
        assert!(
            html.contains("src=\"/x.png?a=1&amp;b=2\""),
            "src 里的 & 必须转义: {html}"
        );
        assert!(
            html.contains("alt=\"a&quot;b\""),
            "alt 引号必须转义: {html}"
        );
    }

    #[test]
    fn mix_of_bold_and_italic() {
        let html = inline_md("**粗体** 和 *斜体* 混合", &test_theme());
        assert!(html.contains("<strong style=\"color: #142333;\">粗体</strong>"));
        assert!(html.contains("<em>斜体</em>"));
    }

    #[test]
    fn double_quoted_link_title_is_not_part_of_href() {
        // GFM 允许 `[文字](url "标题")`。老实现把括号里整段当 URL，
        // href 变成 `https://example.com &quot;标题文字&quot;` —— 点开是死链。
        let html = inline_md("[示例](https://example.com \"标题文字\")", &test_theme());
        assert!(
            html.contains("href=\"https://example.com\""),
            "标题必须从 href 里切掉: {html}"
        );
        assert!(
            !html.contains("&quot;标题文字&quot;"),
            "标题不该进 href，也不该漏进正文: {html}"
        );
    }

    #[test]
    fn single_quoted_link_title_is_not_part_of_href() {
        // CommonMark 同样接受单引号标题。
        let html = inline_md("[示例](https://example.com '标题文字')", &test_theme());
        assert!(
            html.contains("href=\"https://example.com\""),
            "单引号标题也要切掉: {html}"
        );
        assert!(!html.contains("'标题文字'"), "标题不该漏进正文: {html}");
    }

    #[test]
    fn image_title_is_not_part_of_src() {
        let chars: Vec<char> = "![替代](/a.png \"图片标题\")".chars().collect();
        let (alt, url, consumed) = parse_image(&chars).expect("应识别为图片");
        assert_eq!(alt, "替代");
        assert_eq!(url, "/a.png", "标题不能进 src，否则图片加载不出来");
        assert_eq!(consumed, chars.len(), "标题也要一起消费掉，不能漏进正文");
    }

    #[test]
    fn autolink_angle_brackets_are_not_rendered_as_text() {
        // `<https://example.com>` 是 GFM 自动链接。老实现只在 `h` 处认出 URL，
        // 于是输出 `<<a href="https://example.com&gt;">…` —— 正文多一个 `<`，
        // href 尾巴还挂着 `&gt;`。
        let html = inline_md("见 <https://example.com> 结束", &test_theme());
        assert!(
            html.contains("href=\"https://example.com\""),
            "href 不能带 &gt;: {html}"
        );
        assert!(
            !html.contains("&gt;"),
            "尖括号不该出现在 href 或链接文字里: {html}"
        );
        assert!(!html.contains("<<"), "正文多出一个裸 `<`: {html}");
        assert!(html.contains("https://example.com</a>"), "{html}");
    }

    #[test]
    fn angle_bracket_destination_allows_spaces() {
        // `<...>` 形式的链接目标里允许出现空格，此时不能按空格切标题。
        let html = inline_md("[文档](<https://example.com/a b>)", &test_theme());
        assert!(
            html.contains("href=\"https://example.com/a b\""),
            "尖括号目标里的空格要保留、尖括号要剥掉: {html}"
        );
    }

    #[test]
    fn link_without_title_is_unchanged() {
        let html = inline_md("[示例](https://example.com/a?b=1)", &test_theme());
        assert!(
            html.contains("href=\"https://example.com/a?b=1\""),
            "{html}"
        );
    }

    #[test]
    fn triple_emphasis_does_not_leak_asterisks() {
        // `***粗斜***` 是「加粗 + 斜体」。老实现先命中 `**` 分支，把中间的
        // `*粗斜` 当正文，收尾还剩一个裸 `*`，正文里出现 `<strong>*粗斜</strong>*`。
        let html = inline_md("***粗斜***", &test_theme());
        assert!(!html.contains('*'), "三级强调不该漏出裸星号: {html}");
        assert!(html.contains("<em>粗斜</em>"), "斜体没有套上: {html}");
        assert!(html.contains("<strong"), "粗体没有套上: {html}");
    }

    #[test]
    fn triple_emphasis_inside_a_sentence() {
        let html = inline_md("这是 ***重点*** 内容", &test_theme());
        assert!(!html.contains('*'), "句中的三级强调也漏星号: {html}");
        assert!(html.contains("重点") && html.contains("内容"), "{html}");
    }

    #[test]
    fn less_than_sign_in_prose_is_not_an_autolink() {
        let html = inline_md("如果 a < b 且 b > c", &test_theme());
        assert_eq!(html, "如果 a < b 且 b > c");
    }
}
