use std::collections::HashMap;

use crate::theme;

/// 引用式链接的定义表：`[label]: url` 收集后的结果。
///
/// 键是**归一化后的标签**（大小写不敏感、内部空白折叠成一个空格），
/// 值是原始 URL。见 [`normalize_label`]。
pub(super) type LinkRefs = HashMap<String, String>;

pub(super) fn inline_md(text: &str, theme: &theme::Theme) -> String {
    inline_md_with_refs(text, theme, &LinkRefs::new())
}

pub(super) fn inline_md_with_refs(text: &str, theme: &theme::Theme, refs: &LinkRefs) -> String {
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
                inline_md_with_refs(&inner, theme, refs)
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
                    inline_md_with_refs(&inner, theme, refs)
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
                    inline_md_with_refs(&inner, theme, refs)
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
                    inline_md_with_refs(&inner, theme, refs)
                ));
                i += rel + 4;
                continue;
            }
        }
        if chars[i] == '*' {
            let end = chars[i + 1..].iter().position(|&c| c == '*');
            if let Some(rel) = end {
                let inner: String = chars[i + 1..i + 1 + rel].iter().collect();
                s.push_str(&format!(
                    "<em>{}</em>",
                    inline_md_with_refs(&inner, theme, refs)
                ));
                i += rel + 2;
                continue;
            }
        }
        if chars[i] == '!'
            && i + 1 < chars.len()
            && chars[i + 1] == '['
            && let Some((alt, url, consumed)) = parse_image(&chars[i..], refs)
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
            && let Some((text, url, consumed)) = parse_link(&chars[i..], refs)
        {
            s.push_str(&format!(
                "<a href=\"{}\" style=\"color:{};font-weight:bold;text-decoration:none;border-bottom:1px solid {};\">{}</a>",
                html_escape(&url),
                theme.heading_border,
                theme.accent_soft,
                inline_md_with_refs(&text, theme, refs)
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
        // 正文文本必须转义，一个裸 `<` 就能毁掉整篇文章。
        //
        // 公众号侧会把成品 HTML 重新解析一遍：标题里写 `<div>` 会开出一个
        // 真的 div，把后面的段落全吞进这个 div 里（读者看到的是「文章少了
        // 一半」）；写 `<script>` 更狠，整段不显示。CommonMark 默认把 HTML
        // 原样透传，但公众号不支持自定义标签，透传等于丢内容，所以这里一律
        // 落成字符实体。`>` 与 `&` 同理：`&` 后面跟字母时会被当成实体起始，
        // 用户想写的 `&lt;` 与让人看到的 `<` 会错位。
        // 夹在两个行内元素之间的空白必须是**字符**，不能是排版。
        //
        // 公众号后台的真实规则（2026-11-18 对着线上成品逐段核对过）：
        // 文本节点里只要还有正文，里面的空格会被它转成 `&nbsp;` 保留
        // （`没有 <code>before</code>` 线上是 `没有&nbsp;</span>`）；
        // 但**整段只有空白**的节点会被直接删掉。于是 `\`a\` **b**` 这种
        // 紧挨着写的两个元素，中间那个空格在成品里消失，读者看到 `ab`
        // 粘在一起——全语料扫出 116 篇共 1753 处。
        //
        // 判据落在「上一字符是 `>`（刚从元素里出来）且下一段又是个元素」：
        // 只堵这一种，正文与元素之间的空格原样保留——那是正常的断行点。
        // 正文里的 `>` 已被转义成 `&gt;`，所以 `ends_with('>')` 只可能来自标签。
        if chars[i].is_whitespace() && s.ends_with('>') && starts_inline_element(&chars, i) {
            let mut j = i;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            s.push_str("&nbsp;");
            i = j;
            continue;
        }
        match chars[i] {
            // 合法的字符引用原样透传：作者写 `&nbsp;` 是想让读者看到一个空格，
            // 转义成 `&amp;nbsp;` 等于让读者看到七个字符。这不构成注入面——
            // HTML 解析器不会把解码出来的 `<` 再当成标签起始。
            '&' if starts_char_ref(&chars[i..]) => s.push('&'),
            '&' => s.push_str("&amp;"),
            '<' => s.push_str("&lt;"),
            '>' => s.push_str("&gt;"),
            ch => s.push(ch),
        }
        i += 1;
    }
    s
}

/// `chars[i]` 起跳过空白之后，是不是紧接着一个行内元素的开头。
///
/// 只用来判断「这处空白要不要写成 `&nbsp;`」，所以宁可放宽：猜错的结果是
/// 多一个 `&nbsp;`（少一个断行点），猜漏的结果是成品里文字粘在一起。
fn starts_inline_element(chars: &[char], i: usize) -> bool {
    let mut j = i;
    while j < chars.len() && chars[j].is_whitespace() {
        j += 1;
    }
    match chars.get(j) {
        Some('`') | Some('*') | Some('=') | Some('~') | Some('[') | Some('!') => true,
        // 自动链接 `<https://…>`；普通的 `<` 会被转义成 `&lt;`，成不了元素
        Some('<') => chars.get(j + 1) == Some(&'h'),
        _ => false,
    }
}

/// `chars` 以 `&` 开头，且后面是不是一个合法的字符引用。
///
/// 认三种写法：`&nbsp;`、`&#123;`、`&#x1F600;`。命名引用至少两个字符
/// （HTML 里最短的是 `&lt;`），首字符必须是字母。
fn starts_char_ref(chars: &[char]) -> bool {
    if chars.first() != Some(&'&') {
        return false;
    }
    let mut i = 1;
    if chars.get(i) == Some(&'#') {
        i += 1;
        let hex = matches!(chars.get(i), Some('x') | Some('X'));
        if hex {
            i += 1;
        }
        let start = i;
        while i < chars.len()
            && if hex {
                chars[i].is_ascii_hexdigit()
            } else {
                chars[i].is_ascii_digit()
            }
        {
            i += 1;
        }
        return i > start && chars.get(i) == Some(&';');
    }
    let start = i;
    while i < chars.len() && chars[i].is_ascii_alphanumeric() {
        i += 1;
    }
    i - start >= 2 && chars[start].is_ascii_alphabetic() && chars.get(i) == Some(&';')
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

/// 从 `chars[open]` 这个 `[` 出发，找与它配对的 `]` 的下标。
///
/// 链接文字里可以再套一层方括号：`[![alt](x.png)](url)` 是可点击的徽章，
/// shields.io 的标准写法，几乎每个开源 README 都有。老实现取「第一个 `]`」，
/// 链接文字被截成 `![alt`，剩下的 `](x.png)` 漏进正文变成一串乱码。
/// 这里按嵌套层数配对；`\]` 视为字面量，不参与计数。
fn matching_bracket(chars: &[char], open: usize) -> Option<usize> {
    if chars.get(open) != Some(&'[') {
        return None;
    }
    let mut depth = 0usize;
    let mut i = open;
    while i < chars.len() {
        match chars[i] {
            '\\' => {
                i += 2;
                continue;
            }
            '[' => depth += 1,
            ']' => {
                depth -= 1;
                if depth == 0 {
                    return Some(i);
                }
            }
            _ => {}
        }
        i += 1;
    }
    None
}

/// 解析行内链接，三种写法都归到这里：
///
/// - `[文字](url)`：行内链接，目标就在原地；
/// - `[文字][标签]` / `[文字][]`：引用式链接，目标要去定义表里查；
/// - `[标签]`：简写引用，文字自己就是标签。
///
/// 后两种**只有标签真的被定义过**才返回 `Some`，否则一律当字面文本。
/// 这是这条规则的关键：`[0]`、`[x]` 这类正文里的方括号不能因为「长得像」
/// 就被吃掉，只有作者确实写了 `[标签]: url` 才说明他想要链接。
fn parse_link(chars: &[char], refs: &LinkRefs) -> Option<(String, String, usize)> {
    if chars.first() != Some(&'[') {
        return None;
    }
    let text_end = matching_bracket(chars, 0)?;
    let text: String = chars[1..text_end].iter().collect();
    let head = text_end + 1;
    let rest = &chars[head..];
    if rest.first() == Some(&'(') {
        let url_end = destination_end(rest)?;
        let url = split_destination(&unescape_parens(
            &rest[1..url_end].iter().collect::<String>(),
        ));
        return Some((text, url, head + url_end + 1));
    }
    if rest.first() == Some(&'[') {
        // `[文字][标签]` 与 `[文字][]`：空标签表示「用文字自己当标签」。
        let label_end = rest[1..].iter().position(|&c| c == ']')?;
        let label: String = rest[1..1 + label_end].iter().collect();
        let key = if label.trim().is_empty() {
            &text
        } else {
            &label
        };
        let url = refs.get(&normalize_label(key))?.clone();
        return Some((text, url, head + label_end + 2));
    }
    // `[标签]` 简写：没定义就返回 None，调用方会把方括号原样输出。
    if text.trim().is_empty() {
        return None;
    }
    let url = refs.get(&normalize_label(&text))?.clone();
    Some((text, url, head))
}

pub(super) fn parse_image(chars: &[char], refs: &LinkRefs) -> Option<(String, String, usize)> {
    if chars.len() < 5 || chars[0] != '!' || chars[1] != '[' {
        return None;
    }
    let alt_end = matching_bracket(chars, 1)?;
    let alt: String = chars[2..alt_end].iter().collect();
    let head = alt_end + 1;
    let rest = &chars[head..];
    if rest.first() == Some(&'(') {
        let url_end = destination_end(rest)?;
        let url = split_destination(&unescape_parens(
            &rest[1..url_end].iter().collect::<String>(),
        ));
        return Some((alt, url, head + url_end + 1));
    }
    if rest.first() == Some(&'[') {
        let label_end = rest[1..].iter().position(|&c| c == ']')?;
        let label: String = rest[1..1 + label_end].iter().collect();
        let key = if label.trim().is_empty() {
            &alt
        } else {
            &label
        };
        let url = refs.get(&normalize_label(key))?.clone();
        return Some((alt, url, head + label_end + 2));
    }
    None
}

/// 标签归一化：CommonMark 规定标签比对大小写不敏感，内部连续空白折叠成一个空格。
///
/// 中文标签 `to_lowercase()` 是恒等变换，不受影响；英文标签 `[Homepage]` 与
/// `[homepage]` 必须能互相对上，否则同一份文档换个大小写就变成死链。
pub(super) fn normalize_label(raw: &str) -> String {
    raw.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_lowercase()
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
    use super::{LinkRefs, parse_image};
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
        // 粗体里的文字同样是正文：`<` 不转义就会开出一个真标签，
        // 后面的段落会被吞进这个标签里（公众号会重新解析我们给的 HTML）。
        assert!(
            html.contains("<strong style=\"color: #142333;\">a &amp; b &lt; c &gt; d</strong>"),
            "{html}"
        );
    }

    /// 行内元素之间的空白必须落成 `&nbsp;`。
    ///
    /// 公众号后台会删掉「只含空白字符的文本节点」：`</span> <strong>` 存下来是
    /// `</span><strong>`，读者看到 `step_yulan调用它` 粘在一起。紧跟元素的空白
    /// 因此不能是裸空格——但紧跟正文的空白仍是裸空格（那个文本节点有内容，
    /// 后台不删，还能当断行点）。
    #[test]
    fn whitespace_after_an_inline_element_is_escaped() {
        let html = inline_md("`step_yulan` **调用它**", &test_theme());
        assert!(!html.contains("</span> <strong"), "又留裸空格了：{html}");
        assert!(html.contains("</span>&nbsp;<strong"), "{html}");

        // 正文尾巴上的空格不能被顺手改掉：那是正常的断行点
        let prose = inline_md("普通 **粗体** 后面跟文字", &test_theme());
        assert!(prose.contains("</strong> 后面跟文字"), "{prose}");
    }

    /// 合法的字符引用要原样透传：作者写 `&nbsp;` 是想让读者看到空格。
    #[test]
    fn valid_character_references_pass_through() {
        let html = inline_md("宽&nbsp;100&#160;px 与 &#x1F600;", &test_theme());
        assert!(html.contains("&nbsp;100&#160;px"), "{html}");
        assert!(html.contains("&#x1F600;"), "{html}");
        assert!(!html.contains("&amp;nbsp;"), "实体被二次转义了：{html}");
    }

    /// 野生的 `&` 必须转义，否则 `&` 后面跟着字母时会被当成实体起始。
    #[test]
    fn stray_ampersand_is_escaped() {
        let html = inline_md("A & B，结尾 &", &test_theme());
        assert!(html.contains("A &amp; B"), "{html}");
        assert!(html.ends_with("&amp;"), "{html}");
        // 没有分号的“伪实体”不是引用，照样要转义
        let html = inline_md("&notaref 与 &x", &test_theme());
        assert!(html.contains("&amp;notaref"), "{html}");
        assert!(html.contains("&amp;x"), "{html}");
    }

    #[test]
    fn inline_code_escapes_html() {
        let html = inline_md("`a < b > c & d`", &test_theme());
        assert!(html.contains("a &lt; b &gt; c &amp; d"));
    }

    #[test]
    fn parse_image_returns_none_for_non_image() {
        assert!(parse_image(&['[', 't', 'e', 'x', 't', ']', '(', ')'], &LinkRefs::new()).is_none());
    }

    #[test]
    fn parse_image_returns_none_for_short_input() {
        assert!(parse_image(&['!'], &LinkRefs::new()).is_none());
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
        let (alt, url, consumed) = parse_image(&chars, &LinkRefs::new()).expect("应识别为图片");
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
        // 不是自动链接（没有 `http`），但要落成文本：裸 `<` 会让浏览器
        // 从 `<` 开始找标签名，`< b 且 b >` 这段就可能被当成标签吞掉。
        assert_eq!(html, "如果 a &lt; b 且 b &gt; c");
    }
}
