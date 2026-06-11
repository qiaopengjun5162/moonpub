//! Fetch article/tweet content via Chrome headless.
//! WeChat and Twitter/X require a real browser to bypass anti-scraping.

/// Result of fetching web content.
pub struct ArticleContent {
    pub title: String,
    pub body: String,
    pub author: String,
}

/// Fetch content from a URL. Automatically detects WeChat vs Twitter/X.
pub fn fetch_article(url: &str) -> Result<ArticleContent, String> {
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

    if url.contains("x.com") || url.contains("twitter.com") {
        parse_tweet(&html)
    } else {
        parse_wechat(&html)
    }
}

/// Extract tweet content from Twitter/X page HTML.
fn parse_tweet(html: &str) -> Result<ArticleContent, String> {
    let title = extract_meta(html, "og:title")
        .or_else(|| extract_between(html, "<title>", "</title>"))
        .unwrap_or_default()
        .to_owned();

    // Twitter embeds the tweet text in og:description meta tag
    let body = extract_meta(html, "og:description")
        .or_else(|| {
            // Fallback: look for tweet text in data-text or article elements
            extract_between(html, r#"data-testid="tweetText""#, "</div>")
        })
        .unwrap_or_default()
        .to_owned();

    // Author from og:title (format: "Name on X: ...")
    let author = title.split(" on X").next().unwrap_or("").to_owned();

    Ok(ArticleContent {
        title: title.trim().to_owned(),
        body: strip_tags(&body).trim().to_owned(),
        author,
    })
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

    Ok(ArticleContent {
        title,
        body,
        author,
    })
}

/// Extract the content attribute of a <meta name="..." content="..."> tag.
fn extract_meta<'a>(html: &'a str, name: &str) -> Option<&'a str> {
    let pattern = format!("property=\"{name}\"");
    let start = html.find(&pattern)? + pattern.len();
    let rest = &html[start..];
    let content_start = rest.find("content=\"")? + "content=\"".len();
    let content = &rest[content_start..];
    let end = content.find('"')?;
    Some(&content[..end])
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
    let chars = html.chars();

    for ch in chars {
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
    fn extract_meta_basic() {
        let html = r#"<meta property="og:title" content="Hello World">"#;
        assert_eq!(extract_meta(html, "og:title"), Some("Hello World"));
    }

    #[test]
    fn parse_tweet_from_meta() {
        let html = r#"<html><head>
<meta property="og:title" content="Alice on X: Rust is great">
<meta property="og:description" content="I&apos;ve been using Rust for 3 years and here&apos;s why...">
</head></html>"#;
        let article = parse_tweet(html).unwrap();
        assert_eq!(article.author, "Alice");
        assert!(article.body.contains("Rust"));
    }
}
