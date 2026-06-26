use std::fs;
use std::path::Path;

use crate::error::AppError;

use super::{TrendSample, add_trend_sample};

/// CSV column header names we recognise (case-insensitive).
const COL_PLATFORM: &[&str] = &["platform", "平台"];
const COL_KEYWORD: &[&str] = &["keyword", "关键词", "keywords"];
const COL_TITLE: &[&str] = &["title", "标题"];
const COL_URL: &[&str] = &["url", "链接"];
const COL_AUTHOR: &[&str] = &["author", "作者"];
const COL_LIKES: &[&str] = &["likes", "点赞", "like_count"];
const COL_COLLECTS: &[&str] = &["collects", "收藏", "collect_count", "favorites"];
const COL_COMMENTS: &[&str] = &["comments", "评论", "comment_count"];
const COL_SOURCE: &[&str] = &["source", "来源"];

pub fn import_csv(
    articles_dir: &Path,
    csv_path: &Path,
    default_platform: Option<&str>,
) -> Result<String, AppError> {
    let content = fs::read_to_string(csv_path).map_err(|source| AppError::Io {
        path: csv_path.to_path_buf(),
        source,
    })?;

    let mut lines = content.lines();
    let header_line = lines
        .next()
        .ok_or_else(|| AppError::InvalidCsv("empty file".to_owned()))?;
    let headers: Vec<String> = parse_csv_row(header_line);

    fn col_index(headers: &[String], names: &[&str]) -> Option<usize> {
        headers.iter().position(|h| {
            let lower = h.to_lowercase();
            names.iter().any(|n| lower == *n)
        })
    }

    let idx_platform = col_index(&headers, COL_PLATFORM);
    let idx_keyword = col_index(&headers, COL_KEYWORD)
        .ok_or_else(|| AppError::InvalidCsv("missing 'keyword' column".to_owned()))?;
    let idx_title = col_index(&headers, COL_TITLE)
        .ok_or_else(|| AppError::InvalidCsv("missing 'title' column".to_owned()))?;
    let idx_url = col_index(&headers, COL_URL);
    let idx_author = col_index(&headers, COL_AUTHOR);
    let idx_likes = col_index(&headers, COL_LIKES);
    let idx_collects = col_index(&headers, COL_COLLECTS);
    let idx_comments = col_index(&headers, COL_COMMENTS);
    let idx_source = col_index(&headers, COL_SOURCE);

    let mut count = 0u32;
    for line in lines.filter(|l| !l.trim().is_empty()) {
        let cols = parse_csv_row(line);
        let get = |idx: Option<usize>| -> Option<String> {
            idx.and_then(|i| cols.get(i))
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
        };
        let get_u64 = |idx: Option<usize>| -> Option<u64> { get(idx).and_then(|v| v.parse().ok()) };

        let platform = get(idx_platform)
            .or_else(|| default_platform.map(str::to_owned))
            .unwrap_or_else(|| "unknown".to_owned());
        let keyword = match get(Some(idx_keyword)) {
            Some(v) => v,
            None => continue,
        };
        let title = match get(Some(idx_title)) {
            Some(v) => v,
            None => continue,
        };

        let sample = TrendSample {
            platform,
            keyword,
            title,
            url: get(idx_url),
            author: get(idx_author),
            likes: get_u64(idx_likes),
            collects: get_u64(idx_collects),
            comments: get_u64(idx_comments),
            source: get(idx_source).unwrap_or_else(|| "csv".to_owned()),
        };

        add_trend_sample(articles_dir, &sample)?;
        count += 1;
    }

    Ok(format!(
        "imported {count} samples from {}",
        csv_path.display()
    ))
}

/// Parse a single CSV row, respecting double-quoted fields.
pub(crate) fn parse_csv_row(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' if in_quotes => {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    current.push('"');
                } else {
                    in_quotes = false;
                }
            }
            '"' => in_quotes = true,
            ',' if !in_quotes => {
                fields.push(current.clone());
                current.clear();
            }
            other => current.push(other),
        }
    }
    fields.push(current);
    fields
}
