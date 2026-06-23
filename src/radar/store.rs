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
