//! HTML escaping shared by every renderer.
//!
//! Article text, brand copy from user config and cover metadata all end up in
//! HTML, so they must be escaped before interpolation. This used to exist as
//! four near-identical private copies (markdown inline/blocks, footer,
//! illustrate) plus a wider one in cover, which meant a fix to the escaping
//! rules had to be applied in five places. Keeping one implementation removes
//! that divergence risk.

/// Escapes `&`, `<`, `>`, `"` and `'` so untrusted text can be embedded both in
/// HTML text nodes and in attribute values.
///
/// Single pass on purpose: chaining `.replace()` would allocate a new `String`
/// per escaped character class.
pub(crate) fn escape(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    for character in input.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

/// 把会执行脚本或读取本地文件的 URL 目标换成无害的 `#`。
///
/// 公众号后台大概率会过滤这些协议，但 `moonpub preview` 生成的 HTML 是在本地
/// 浏览器里打开的：从 RSS / 抓取内容里来的一条 `[点我](javascript:...)`，在
/// 预览里点一下就能执行。转义只管得住「引号截断属性」，管不住浏览器去解析
/// 一个合法的 `javascript:` 协议头，所以协议本身也要过一遍。
///
/// 只放行 `http` / `https` / `mailto` / `tel` 与不带协议的相对地址（`/x.png`、
/// `./a.md`、`a?b=1`）。判断协议时按浏览器的口径忽略前面的空白与控制字符、
/// 大小写不敏感。实体编码的协议头（`jav&#x61;script:`）里的 `&` 会被
/// [`escape`] 转成 `&amp;`，解码后仍是普通文本，不构成入口。
pub(crate) fn safe_destination(url: &str) -> &str {
    let trimmed = url.trim_start_matches(|c: char| c.is_ascii_whitespace() || c.is_ascii_control());
    let mut scheme = String::new();
    let mut has_colon = false;
    for ch in trimmed.chars() {
        match ch {
            ':' => {
                has_colon = true;
                break;
            }
            // '/'、'?'、'#' 出现在冒号前就说明这不是协议头，是相对地址
            '/' | '?' | '#' => return url,
            c if c.is_ascii_alphanumeric() || matches!(c, '+' | '-' | '.') => {
                scheme.push(c.to_ascii_lowercase());
            }
            // 浏览器解析协议时会忽略空白与控制字符
            c if c.is_ascii_whitespace() || c.is_ascii_control() => {}
            // 其他字符说明这不是协议头
            _ => return url,
        }
    }
    if !has_colon || matches!(scheme.as_str(), "http" | "https" | "mailto" | "tel") {
        url
    } else {
        "#"
    }
}

#[cfg(test)]
mod tests {
    use super::{escape, safe_destination};

    #[test]
    fn escapes_markup_and_quotes() {
        assert_eq!(escape("a&b<c>d\"e'f"), "a&amp;b&lt;c&gt;d&quot;e&#39;f");
    }

    #[test]
    fn leaves_plain_text_untouched() {
        assert_eq!(escape("MoonPub 自动发布"), "MoonPub 自动发布");
    }

    #[test]
    fn dangerous_url_schemes_become_inert() {
        for url in [
            "javascript:alert(1)",
            "JavaScript:alert(1)",
            "  javascript:alert(1)",
            "vbscript:msgbox(1)",
            "data:text/html;base64,PHNjcmlwdD4=",
            "file:///etc/passwd",
            "about:blank",
        ] {
            assert_eq!(safe_destination(url), "#", "{url} 不该直接进 href/src");
        }
    }

    #[test]
    fn ordinary_destinations_are_kept() {
        for url in [
            "https://example.com/a?x=1&y=2",
            "http://example.com",
            "mailto:a@b.com",
            "tel:12345",
            "//cdn.example.com/x.png",
            "./a.md",
            "/images/logo.png",
            "a b.png",
        ] {
            assert_eq!(safe_destination(url), url, "{url} 是正常目标");
        }
    }
}
