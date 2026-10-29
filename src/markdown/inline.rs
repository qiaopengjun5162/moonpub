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
                s.push_str(&format!(
                    "<code style=\"background:{};{}padding:2px 4px;border-radius:3px;font-size:14px;font-family:monospace;\">{}</code>",
                    theme.code_bg,
                    color_style,
                    html_escape(&code)
                ));
                i += rel + 2;
                continue;
            }
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
            s.push_str(&format!(
                "<p style=\"margin: 1.5em 0; text-align: center;\"><img src=\"{url}\" alt=\"{alt}\" style=\"max-width: 100%; display: block; margin: 0 auto;\" /></p>"
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
        if ch.is_whitespace() || matches!(ch, '，' | '。' | '；' | '、' | '）' | ')' | ']' | '》')
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
    if rest.first() != Some(&'(') {
        return None;
    }
    let url_end = rest[1..].iter().position(|&c| c == ')')?;
    let url: String = rest[1..1 + url_end].iter().collect();
    Some((text, url, 1 + text_end + 1 + 1 + url_end + 1))
}

pub(super) fn parse_image(chars: &[char]) -> Option<(String, String, usize)> {
    if chars.len() < 5 || chars[0] != '!' || chars[1] != '[' {
        return None;
    }
    let alt_end = chars[2..].iter().position(|&c| c == ']')?;
    let alt: String = chars[2..2 + alt_end].iter().collect();
    let rest = &chars[2 + alt_end + 1..];
    if rest.first() != Some(&'(') {
        return None;
    }
    let url_end = rest[1..].iter().position(|&c| c == ')')?;
    let url: String = rest[1..1 + url_end].iter().collect();
    Some((alt, url, 2 + alt_end + 1 + 1 + url_end + 1))
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
        assert!(html.contains("<code"));
        assert!(html.contains("let x = 1;"));
        assert!(html.contains("</code>"));
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
    fn mix_of_bold_and_italic() {
        let html = inline_md("**粗体** 和 *斜体* 混合", &test_theme());
        assert!(html.contains("<strong style=\"color: #142333;\">粗体</strong>"));
        assert!(html.contains("<em>斜体</em>"));
    }
}
