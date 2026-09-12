use std::fs;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::{
    error::AppError,
    json_util::{
        escape_json, extract_json_optional_string, extract_json_optional_u64, extract_json_string,
    },
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrendSample {
    pub platform: String,
    pub keyword: String,
    pub title: String,
    pub url: Option<String>,
    pub author: Option<String>,
    pub likes: Option<u64>,
    pub collects: Option<u64>,
    pub comments: Option<u64>,
    pub source: String,
}

/// Weights for engagement scoring: likes count 1x, collects 2x, comments 3x.
/// Comments weighted highest because they reflect deeper engagement than passive likes.
const COLLECT_WEIGHT: u64 = 2;
const COMMENT_WEIGHT: u64 = 3;

impl TrendSample {
    pub(crate) fn engagement_score(&self) -> u64 {
        self.likes.unwrap_or(0)
            + self.collects.unwrap_or(0) * COLLECT_WEIGHT
            + self.comments.unwrap_or(0) * COMMENT_WEIGHT
    }

    pub(crate) fn to_json_line(&self) -> String {
        let fields = [
            json_string_field("platform", &self.platform),
            json_string_field("keyword", &self.keyword),
            json_string_field("title", &self.title),
            json_optional_string_field("url", self.url.as_deref()),
            json_optional_string_field("author", self.author.as_deref()),
            json_optional_u64_field("likes", self.likes),
            json_optional_u64_field("collects", self.collects),
            json_optional_u64_field("comments", self.comments),
            json_string_field("source", &self.source),
        ];
        format!("{{{}}}", fields.join(","))
    }

    pub(crate) fn from_json_line(line: &str) -> Option<Self> {
        Some(Self {
            platform: extract_json_string(line, "platform")?,
            keyword: extract_json_string(line, "keyword")?,
            title: extract_json_string(line, "title")?,
            url: extract_json_optional_string(line, "url"),
            author: extract_json_optional_string(line, "author"),
            likes: extract_json_optional_u64(line, "likes"),
            collects: extract_json_optional_u64(line, "collects"),
            comments: extract_json_optional_u64(line, "comments"),
            source: extract_json_string(line, "source")?,
        })
    }
}

pub fn add_trend_sample(articles_dir: &Path, sample: &TrendSample) -> Result<String, AppError> {
    let path = trend_store_path(articles_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| AppError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|source| AppError::Io {
            path: path.clone(),
            source,
        })?;
    writeln!(file, "{}", sample.to_json_line()).map_err(|source| AppError::Io {
        path: path.clone(),
        source,
    })?;

    Ok(format!("added trend sample to {}", path.display()))
}

pub fn list_trend_samples(
    articles_dir: &Path,
    platform: &Option<String>,
    keyword: &Option<String>,
) -> Result<String, AppError> {
    let path = trend_store_path(articles_dir);
    if !path.exists() {
        return Ok("trend samples\n  (empty)".to_owned());
    }

    let content = fs::read_to_string(&path).map_err(|source| AppError::Io {
        path: path.clone(),
        source,
    })?;

    let mut rows = Vec::new();
    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        if let Some(sample) = TrendSample::from_json_line(line) {
            if let Some(expected) = platform
                && &sample.platform != expected
            {
                continue;
            }
            if let Some(expected) = keyword
                && &sample.keyword != expected
            {
                continue;
            }
            rows.push(sample);
        }
    }

    Ok(format_trend_samples(&rows))
}

pub(crate) fn load_all_samples(path: &Path) -> Result<Vec<TrendSample>, AppError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(TrendSample::from_json_line)
        .collect())
}

pub(crate) fn trend_store_path(articles_dir: &Path) -> PathBuf {
    articles_dir.join(".moonpub").join("trends.jsonl")
}

fn json_string_field(name: &str, value: &str) -> String {
    format!("\"{name}\":\"{}\"", escape_json(value))
}

fn json_optional_string_field(name: &str, value: Option<&str>) -> String {
    value.map_or_else(
        || format!("\"{name}\":null"),
        |value| json_string_field(name, value),
    )
}

fn json_optional_u64_field(name: &str, value: Option<u64>) -> String {
    value.map_or_else(
        || format!("\"{name}\":null"),
        |value| format!("\"{name}\":{value}"),
    )
}

pub(crate) fn format_trend_samples(samples: &[TrendSample]) -> String {
    let mut output = String::from("trend samples\n");
    if samples.is_empty() {
        output.push_str("  (empty)");
        return output;
    }

    for sample in samples {
        output.push_str(&format!(
            "  [{}] {} | {}",
            sample.platform, sample.keyword, sample.title
        ));
        if let Some(likes) = sample.likes {
            output.push_str(&format!(" | likes={likes}"));
        }
        if let Some(collects) = sample.collects {
            output.push_str(&format!(" | collects={collects}"));
        }
        if let Some(comments) = sample.comments {
            output.push_str(&format!(" | comments={comments}"));
        }
        if let Some(url) = &sample.url {
            output.push_str(&format!(" | {url}"));
        }
        output.push('\n');
    }
    output.trim_end().to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> TrendSample {
        TrendSample {
            platform: "xhs".into(),
            keyword: "rust".into(),
            title: "用 Rust 重写渲染管线".into(),
            url: None,
            author: None,
            likes: None,
            collects: None,
            comments: None,
            source: "scrape".into(),
        }
    }

    // ── engagement_score：加权求和契约 ──────────────────────────────
    #[test]
    fn engagement_score_all_none_is_zero() {
        assert_eq!(sample().engagement_score(), 0);
    }

    #[test]
    fn engagement_score_applies_weights() {
        let s = TrendSample {
            likes: Some(10),
            collects: Some(5),
            comments: Some(2),
            ..sample()
        };
        // likes 1x + collects 2x + comments 3x
        assert_eq!(s.engagement_score(), 10 + 5 * 2 + 2 * 3);
    }

    #[test]
    fn engagement_score_comments_weighted_highest() {
        let s = TrendSample {
            likes: Some(1),
            collects: Some(1),
            comments: Some(1),
            ..sample()
        };
        assert_eq!(s.engagement_score(), 1 + 2 + 3);
    }

    // ── to_json_line ↔ from_json_line 往返 ─────────────────────────
    #[test]
    fn round_trip_without_optionals() {
        let s = sample();
        let back = TrendSample::from_json_line(&s.to_json_line()).expect("round-trip");
        assert_eq!(s, back);
    }

    #[test]
    fn round_trip_with_all_optionals() {
        let s = TrendSample {
            url: Some("https://example.com/a".into()),
            author: Some("月梁".into()),
            likes: Some(42),
            collects: Some(7),
            comments: Some(3),
            ..sample()
        };
        let back = TrendSample::from_json_line(&s.to_json_line()).expect("round-trip");
        assert_eq!(s, back);
    }

    #[test]
    fn round_trip_preserves_escaped_quotes() {
        let s = TrendSample {
            platform: "a\"b".into(),
            keyword: "k\"w".into(),
            title: "他说 \"hi\" 然后离开".into(),
            source: "s\"rc".into(),
            ..sample()
        };
        let back = TrendSample::from_json_line(&s.to_json_line()).expect("round-trip");
        assert_eq!(s, back);
    }

    // ── from_json_line 故障注入 ─────────────────────────────────────
    #[test]
    fn from_json_line_missing_required_field_is_none() {
        // 缺 source（必填）→ 应回退 None
        let partial = "{\"platform\":\"xhs\",\"keyword\":\"rust\",\"title\":\"t\"}";
        assert!(TrendSample::from_json_line(partial).is_none());
    }

    #[test]
    fn from_json_line_empty_string_is_none() {
        assert!(TrendSample::from_json_line("").is_none());
    }

    #[test]
    fn from_json_line_garbage_is_none() {
        assert!(TrendSample::from_json_line("not json at all").is_none());
    }

    // ── 字段构造器 ─────────────────────────────────────────────────
    #[test]
    fn json_string_field_escapes_quotes() {
        assert_eq!(
            json_string_field("name", "he\"llo"),
            "\"name\":\"he\\\"llo\""
        );
    }

    #[test]
    fn json_string_field_escapes_backslash_and_newline() {
        assert_eq!(json_string_field("k", "a\\b\nc"), "\"k\":\"a\\\\b\\nc\"");
    }

    #[test]
    fn json_optional_string_field_none_is_null() {
        assert_eq!(json_optional_string_field("url", None), "\"url\":null");
    }

    #[test]
    fn json_optional_string_field_some_reuses_string_field() {
        assert_eq!(
            json_optional_string_field("author", Some("me")),
            "\"author\":\"me\""
        );
    }

    #[test]
    fn json_optional_u64_field_none_is_null() {
        assert_eq!(json_optional_u64_field("likes", None), "\"likes\":null");
    }

    #[test]
    fn json_optional_u64_field_some_renders_number() {
        assert_eq!(json_optional_u64_field("likes", Some(99)), "\"likes\":99");
    }

    // ── format_trend_samples 边界 ──────────────────────────────────
    #[test]
    fn format_empty_is_labeled_empty() {
        assert_eq!(format_trend_samples(&[]), "trend samples\n  (empty)");
    }

    #[test]
    fn format_one_row_without_optionals() {
        let out = format_trend_samples(&[sample()]);
        assert_eq!(out, "trend samples\n  [xhs] rust | 用 Rust 重写渲染管线");
    }

    #[test]
    fn format_one_row_with_all_optionals() {
        let s = TrendSample {
            url: Some("https://x.com".into()),
            author: Some("月梁".into()),
            likes: Some(10),
            collects: Some(4),
            comments: Some(2),
            ..sample()
        };
        let out = format_trend_samples(&[s]);
        assert!(out.contains("[xhs] rust | 用 Rust 重写渲染管线"));
        assert!(out.contains("likes=10"));
        assert!(out.contains("collects=4"));
        assert!(out.contains("comments=2"));
        assert!(out.contains("https://x.com"));
    }
}
