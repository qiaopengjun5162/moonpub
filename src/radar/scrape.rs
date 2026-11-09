use std::path::{Path, PathBuf};

use crate::error::AppError;

use super::{TrendSample, add_trend_sample, trend_store_path};

/// Scrape trending articles for a platform keyword and store them in trends.jsonl.
///
/// Uses playwright-cli if found in PATH, otherwise falls back to curl.
/// Default search URL for wechat: Sogou WeChat search (public, no auth).
pub fn scrape_radar(
    articles_dir: &Path,
    platform: &str,
    keyword: &str,
    count: usize,
    custom_url: Option<&str>,
) -> Result<String, AppError> {
    let url = custom_url
        .map(str::to_owned)
        .unwrap_or_else(|| default_search_url(platform, keyword));

    let raw = fetch_page(&url)?;
    let samples = extract_samples(&raw, platform, keyword, count);

    if samples.is_empty() {
        return Ok(format!(
            "scraped 0 samples from {url}\n  tip: try --url to specify a direct search page"
        ));
    }

    for s in &samples {
        add_trend_sample(articles_dir, s)?;
    }

    let store = trend_store_path(articles_dir);
    Ok(format!(
        "scraped {} samples → {}\n{}",
        samples.len(),
        store.display(),
        samples
            .iter()
            .map(|s| format!("  · {}", s.title))
            .collect::<Vec<_>>()
            .join("\n")
    ))
}

pub(crate) fn default_search_url(platform: &str, keyword: &str) -> String {
    let encoded = url_encode(keyword);
    match platform {
        "wechat" | "微信" => {
            format!("https://weixin.sogou.com/weixin?type=2&query={encoded}")
        }
        "xiaohongshu" | "xhs" | "小红书" => {
            format!("https://www.xiaohongshu.com/search_result/?keyword={encoded}")
        }
        _ => format!("https://weixin.sogou.com/weixin?type=2&query={encoded}"),
    }
}

pub(crate) fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// Fetch a page using playwright-cli (if available) or curl.
pub(crate) fn fetch_page(url: &str) -> Result<String, AppError> {
    if let Some(content) = try_playwright_cli(url) {
        return Ok(content);
    }
    fetch_with_curl(url)
}

pub(crate) fn try_playwright_cli(url: &str) -> Option<String> {
    // playwright-cli needs a persistent session; open then snapshot in sequence.
    // Requires playwright-cli binary in PATH.
    let open_ok = std::process::Command::new("playwright-cli")
        .args(["open", url, "--headless"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .is_some();

    if !open_ok {
        return None;
    }

    let snap = std::process::Command::new("playwright-cli")
        .arg("snapshot")
        .output()
        .ok()?;

    let _ = std::process::Command::new("playwright-cli")
        .arg("close")
        .output();

    let content = String::from_utf8_lossy(&snap.stdout).into_owned();
    if content.trim().is_empty() {
        None
    } else {
        Some(content)
    }
}

pub(crate) fn fetch_with_curl(url: &str) -> Result<String, AppError> {
    let out = std::process::Command::new("curl")
        .args([
            "-sL",
            "--max-time",
            "15",
            "-H",
            "User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
            "-H",
            "Accept-Language: zh-CN,zh;q=0.9",
            "--compressed",
            url,
        ])
        .output()
        .map_err(|source| AppError::Io {
            path: PathBuf::from("curl"),
            source,
        })?;

    if !out.status.success() {
        return Err(AppError::PushFailed {
            message: format!("curl failed for {url}"),
            ip_hint: None,
        });
    }

    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Extract TrendSample candidates from raw HTML or a playwright snapshot.
pub(crate) fn extract_samples(
    raw: &str,
    platform: &str,
    keyword: &str,
    limit: usize,
) -> Vec<TrendSample> {
    let mut samples = Vec::new();

    // Heuristic 1: playwright-cli snapshot lines look like:
    //   - link "文章标题" [ref=e5]
    //   - heading "文章标题"
    // Heuristic 2: HTML <title> / <h3> / <a> text content
    let is_snapshot = raw.contains("[ref=");

    let titles: Vec<String> = if is_snapshot {
        extract_from_snapshot(raw)
    } else {
        extract_from_html(raw)
    };

    for title in titles.into_iter().take(limit) {
        samples.push(TrendSample {
            platform: platform.to_owned(),
            keyword: keyword.to_owned(),
            title,
            url: None,
            author: None,
            likes: None,
            collects: None,
            comments: None,
            source: "scrape".to_owned(),
        });
    }
    samples
}

pub(crate) fn extract_from_snapshot(snapshot: &str) -> Vec<String> {
    let mut titles = Vec::new();
    for line in snapshot.lines() {
        let trimmed = line.trim();
        for prefix in &["- link \"", "- heading \"", "  - link \"", "  - heading \""] {
            if let Some(rest) = trimmed.strip_prefix(prefix)
                && let Some(end) = rest.find('"')
            {
                let text = rest[..end].trim();
                if is_good_title(text) {
                    titles.push(text.to_owned());
                }
            }
        }
    }
    titles
}

pub(crate) fn extract_from_html(html: &str) -> Vec<String> {
    let mut titles = Vec::new();

    let text = {
        let mut s = html.to_owned();
        while let (Some(a), Some(b)) = (s.find("<script"), s.find("</script>")) {
            if a < b {
                s = format!("{}{}", &s[..a], &s[b + 9..]);
            } else {
                break;
            }
        }
        s
    };

    let tag_re_patterns = ["<h2", "<h3", "<a "];
    for &tag in &tag_re_patterns {
        let mut pos = 0;
        while let Some(start) = text[pos..].find(tag) {
            let abs_start = pos + start;
            let Some(gt) = text[abs_start..].find('>') else {
                break;
            };
            let content_start = abs_start + gt + 1;
            let close_tag = format!("</{}", tag[1..].trim_end_matches(' '));
            let content_end = text[content_start..]
                .find(close_tag.as_str())
                .map(|i| content_start + i)
                .unwrap_or_else(|| {
                    // No closing tag anywhere after the opening one (truncated
                    // page, uppercase `</H2>`, sloppy markup). Fall back to a
                    // bounded window, but snap it down to a char boundary: the
                    // raw `+200` used to land inside a CJK glyph (3 bytes) and
                    // panic on slicing.
                    floor_char_boundary(
                        &text,
                        content_start,
                        content_start + 200.min(text.len() - content_start),
                    )
                });

            if content_end > content_start {
                let inner = &text[content_start..content_end];
                let plain = strip_html_tags(inner);
                let plain = plain.trim();
                if is_good_title(plain) {
                    titles.push(plain.to_owned());
                }
            }

            pos = abs_start + 1;
            if pos >= text.len() {
                break;
            }
        }
    }

    let mut seen = std::collections::HashSet::new();
    titles.retain(|t| seen.insert(t.clone()));
    titles
}

/// Largest char boundary of `text` within `start..=end`.
///
/// Scraped pages are arbitrary UTF-8, so a byte offset computed as
/// `content_start + N` is not guaranteed to be a char boundary: CJK glyphs are
/// 3 bytes and emoji are 4. `str::floor_char_boundary` would express this
/// directly, but it only stabilized in Rust 1.91 while this crate's MSRV is
/// 1.88, so the boundary is walked back explicitly (at most 3 steps).
fn floor_char_boundary(text: &str, start: usize, end: usize) -> usize {
    let mut end = end.min(text.len());
    while end > start && !text.is_char_boundary(end) {
        end -= 1;
    }
    end
}

pub(crate) fn strip_html_tags(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    out.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&#34;", "\"")
        .replace("&#39;", "'")
}

/// Valid title char count range — shorter is likely nav text, longer is likely body copy.
const TITLE_MIN_CHARS: usize = 6;
const TITLE_MAX_CHARS: usize = 80;
const TITLE_MIN_VISIBLE_CHARS: usize = 4;

pub(crate) fn is_good_title(text: &str) -> bool {
    let char_count = text.chars().count();
    if !(TITLE_MIN_CHARS..=TITLE_MAX_CHARS).contains(&char_count) {
        return false;
    }
    if text.chars().filter(|c| !c.is_whitespace()).count() < TITLE_MIN_VISIBLE_CHARS {
        return false;
    }
    let lower = text.to_lowercase();
    let nav_words = [
        "javascript",
        "function",
        "var ",
        "onclick",
        "copyright",
        "©",
    ];
    !nav_words.iter().any(|w| lower.contains(w))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Baseline: well-formed Chinese headings keep being extracted.
    #[test]
    fn extract_from_html_parses_closed_chinese_headings() {
        let html = "<html><body><h2>中文标题测试样例</h2><h3>第二个中文标题样例</h3></body></html>";

        let titles = extract_from_html(html);

        assert!(
            titles.contains(&"中文标题测试样例".to_owned()),
            "{titles:?}"
        );
        assert!(
            titles.contains(&"第二个中文标题样例".to_owned()),
            "{titles:?}"
        );
    }

    /// Regression: a heading with no closing tag used to slice the page at a
    /// fixed `content_start + 200` byte offset and panic inside a CJK glyph
    /// (`end byte index 204 is not a char boundary`).
    #[test]
    fn extract_from_html_does_not_panic_when_close_tag_is_missing() {
        // 4 + 12 * 18 = 220 bytes, so the 200-byte window lands at byte 204,
        // in the middle of the 3-byte glyph starting at 202.
        let html = format!("<h2>{}", "中文标题内容".repeat(12));

        let titles = extract_from_html(&html);

        assert_eq!(titles.len(), 1, "{titles:?}");
        assert!(titles[0].chars().count() > 6, "{titles:?}");
    }

    /// Same fallback, different trigger: the closing tag exists but is
    /// uppercase while the search is case-sensitive.
    #[test]
    fn extract_from_html_handles_uppercase_close_tag_without_panicking() {
        let html = format!("<h2>{}</H2>", "中文标题内容".repeat(12));

        let titles = extract_from_html(&html);

        assert_eq!(titles.len(), 1, "{titles:?}");
    }

    /// 4-byte emoji must not break the same window.
    #[test]
    fn extract_from_html_does_not_panic_with_emoji_fallback() {
        let html = format!("<h2>{}", "🎉表情测试".repeat(20));

        let titles = extract_from_html(&html);

        assert_eq!(titles.len(), 1, "{titles:?}");
        assert!(titles[0].contains('🎉'), "{titles:?}");
    }

    /// Text inside `<script>` is removed before tag scanning, so fake headings
    /// there must never reach the sample list.
    #[test]
    fn extract_from_html_skips_script_contents() {
        let html = concat!(
            r#"<script>var s = "<a href='/x'>脚本里的假中文标题内容</a>";</script>"#,
            "<h3>这是真正的中文标题内容</h3>"
        );

        let titles = extract_from_html(html);

        assert_eq!(titles, vec!["这是真正的中文标题内容".to_owned()]);
    }

    /// The same title matched twice is stored once.
    #[test]
    fn extract_from_html_dedupes_repeated_titles() {
        let html = "<h3>重复出现的中文标题</h3><h3>重复出现的中文标题</h3>";

        let titles = extract_from_html(html);

        assert_eq!(titles, vec!["重复出现的中文标题".to_owned()]);
    }

    /// Plain text has no headings and must not panic.
    #[test]
    fn extract_from_html_returns_empty_without_tags() {
        assert!(extract_from_html("纯文本没有标签也没有标题").is_empty());
    }
}
