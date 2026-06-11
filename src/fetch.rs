//! Fetch article content from WeChat and other platforms.
//! Uses Chrome headless for WeChat (JS-rendered pages).

/// Result of fetching web content.
#[derive(Debug)]
pub struct ArticleContent {
    pub title: String,
    pub body: String,
    pub author: String,
}

/// Fetch content from a URL. Supports WeChat articles; Twitter/X is limited
/// to meta-tag extraction (og:title/og:description) which is often truncated.
pub fn fetch_article(url: &str) -> Result<ArticleContent, String> {
    if url.contains("x.com") || url.contains("twitter.com") {
        return Err("Twitter/X 需要浏览器执行 JavaScript 才能获取完整推文。\n\
             请手动复制推文内容，或使用以下替代方案：\n\
             - 截图后用 OCR 提取\n\
             - 使用 Twitter API (付费)\n\
             - 用浏览器打开后复制"
            .to_owned());
    }

    if url.contains("mp.weixin.qq.com") {
        return fetch_wechat(url);
    }

    Err(format!("unsupported URL: {url}"))
}

/// Fetch WeChat article via Chrome headless.
fn fetch_wechat(url: &str) -> Result<ArticleContent, String> {
    let chrome = crate::find_chrome().ok_or("Chrome/Chromium not found")?;

    let output = std::process::Command::new(&chrome)
        .args([
            "--headless",
            "--disable-gpu",
            "--no-sandbox",
            "--dump-dom",
            "--virtual-time-budget=10000",
            url,
        ])
        .output()
        .map_err(|e| format!("failed to run Chrome: {e}"))?;

    if !output.status.success() {
        return Err(format!(
            "Chrome exited with {}",
            output.status.code().unwrap_or(-1)
        ));
    }

    let html = String::from_utf8_lossy(&output.stdout);
    parse_wechat(&html)
}

/// Extract WeChat article content from page HTML.
fn parse_wechat(html: &str) -> Result<ArticleContent, String> {
    let title = extract_between(html, "<title>", "</title>")
        .map(|t| t.trim().to_owned())
        .unwrap_or_default();

    let body_html = extract_between(html, r#"id="js_content""#, "</div>")
        .or_else(|| extract_between(html, r#"class="rich_media_content"#, "</div>"))
        .unwrap_or_default();

    let body = strip_tags(body_html);

    let author = extract_between(html, r#"id="js_name""#, "</span>")
        .or_else(|| extract_between(html, r#"class="rich_media_meta_text""#, "</span>"))
        .map(|a| strip_tags(a).trim().to_owned())
        .unwrap_or_default();

    if body.is_empty() {
        return Err("未找到文章正文 — 页面可能需要验证或登录".to_owned());
    }

    Ok(ArticleContent {
        title,
        body,
        author,
    })
}

/// Extract text between two markers.
fn extract_between<'a>(haystack: &'a str, prefix: &str, suffix: &str) -> Option<&'a str> {
    let start = haystack.find(prefix)? + prefix.len();
    let content = &haystack[start..];
    let real_start = if prefix.ends_with('>') {
        0
    } else {
        content.find('>').map(|i| i + 1).unwrap_or(0)
    };
    let end = content[real_start..].find(suffix)?;
    Some(&content[real_start..][..end])
}

/// Remove HTML tags, decode entities, collapse whitespace.
fn strip_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;

    for ch in html.chars() {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }

    result = result.replace("&nbsp;", " ");
    result = result.replace("&lt;", "<");
    result = result.replace("&gt;", ">");
    result = result.replace("&amp;", "&");
    result = result.replace("&quot;", "\"");
    result = result.replace("&#39;", "'");

    let mut out = String::new();
    let mut last_was_ws = false;
    for ch in result.chars() {
        if ch.is_whitespace() {
            if !last_was_ws {
                out.push(' ');
                last_was_ws = true;
            }
        } else {
            out.push(ch);
            last_was_ws = false;
        }
    }
    out.trim().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_tags_basic() {
        assert_eq!(strip_tags("<p>hello</p>"), "hello");
    }

    #[test]
    fn strip_tags_with_entities() {
        assert_eq!(strip_tags("<p>a&nbsp;b&nbsp;c</p>"), "a b c");
    }

    #[test]
    fn extract_between_basic() {
        assert_eq!(
            extract_between("aa<prefix>hello</suffix>bb", "<prefix>", "</suffix>"),
            Some("hello")
        );
    }

    #[test]
    fn twitter_returns_clear_error() {
        let result = fetch_article("https://x.com/user/status/123");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Twitter/X"));
    }

    #[test]
    fn parse_wechat_empty_body_is_error() {
        let result = parse_wechat("<html></html>");
        assert!(result.is_err());
    }
}
