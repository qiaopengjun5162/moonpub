//! Fetch WeChat article content via Chrome headless.
//! WeChat articles require a real browser to bypass anti-scraping checks.

/// Result of fetching a WeChat article.
pub struct ArticleContent {
    pub title: String,
    pub body: String,
    pub author: String,
}

/// Fetch a WeChat article via Chrome headless, returning title + plain-text body.
/// Requires Chrome/Chromium installed.
pub fn fetch_article(url: &str) -> Result<ArticleContent, String> {
    let chrome = crate::find_chrome().ok_or("Chrome/Chromium not found")?;

    // Use Chrome headless with --dump-dom to get the rendered HTML
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

    // Parse article content from the rendered HTML
    let title = extract_between(&html, r#"<title>"#, "</title>")
        .map(|t| t.trim().to_owned())
        .unwrap_or_default();

    // The article body is in #js_content
    let body_html = extract_between(&html, r#"id="js_content""#, "</div>")
        .or_else(|| extract_between(&html, r#"class="rich_media_content"#, "</div>"))
        .unwrap_or_default();

    // Strip HTML tags for plain text output
    let body = strip_tags(body_html);

    let author = extract_between(&html, r#"id="js_name""#, "</span>")
        .or_else(|| extract_between(&html, r#"class="rich_media_meta_text""#, "</span>"))
        .map(|a| strip_tags(a).trim().to_owned())
        .unwrap_or_default();

    Ok(ArticleContent {
        title,
        body,
        author,
    })
}

/// Extract text between two markers in a string.
fn extract_between<'a>(haystack: &'a str, prefix: &str, suffix: &str) -> Option<&'a str> {
    let start = haystack.find(prefix)? + prefix.len();
    let content = &haystack[start..];
    // If the prefix ends inside an HTML tag, skip to after the next '>'
    let real_start = if prefix.ends_with('>') {
        0
    } else {
        content.find('>').map(|i| i + 1).unwrap_or(0)
    };
    let end = content[real_start..].find(suffix)?;
    Some(&content[real_start..][..end])
}

/// Remove HTML tags and decode common entities.
fn strip_tags(html: &str) -> String {
    let mut result = String::new();
    let mut in_tag = false;
    let chars = html.chars().peekable();

    for ch in chars {
        if ch == '<' {
            in_tag = true;
        } else if ch == '>' {
            in_tag = false;
        } else if !in_tag {
            result.push(ch);
        }
    }

    // Decode common entities
    result = result.replace("&nbsp;", " ");
    result = result.replace("&lt;", "<");
    result = result.replace("&gt;", ">");
    result = result.replace("&amp;", "&");
    result = result.replace("&quot;", "\"");

    // Collapse whitespace
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
}
