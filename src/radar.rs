//! Radar command — trend sample management, analysis, and scraping.

use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::{
    AppError, escape_json, extract_json_optional_string, extract_json_optional_u64,
    extract_json_string, parse_frontmatter, resolve_article_path, strip_frontmatter,
};

// ── radar ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RadarCommand {
    Add(TrendSample),
    List {
        platform: Option<String>,
        keyword: Option<String>,
    },
    Import {
        path: PathBuf,
        platform: Option<String>,
    },
    Analyze {
        article: PathBuf,
        platform: String,
        top: usize,
    },
    Suggest {
        article: PathBuf,
        platform: String,
        top: usize,
    },
    Scrape {
        platform: String,
        keyword: String,
        count: usize,
        url: Option<String>,
    },
}

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
}

pub(crate) fn parse_radar_command(args: &[String]) -> Result<RadarCommand, AppError> {
    let Some(command) = args.first() else {
        return Err(AppError::MissingValue("radar <add|list|import|analyze>"));
    };

    match command.as_str() {
        "add" => parse_radar_add(&args[1..]),
        "list" => parse_radar_list(&args[1..]),
        "import" => parse_radar_import(&args[1..]),
        "analyze" => parse_radar_analyze(&args[1..]),
        "suggest" => parse_radar_suggest(&args[1..]),
        "scrape" => parse_radar_scrape(&args[1..]),
        value => Err(AppError::UnknownCommand(format!("radar {value}"))),
    }
}

pub(crate) fn parse_radar_add(args: &[String]) -> Result<RadarCommand, AppError> {
    let mut platform = None;
    let mut keyword = None;
    let mut title = None;
    let mut url = None;
    let mut author = None;
    let mut likes = None;
    let mut collects = None;
    let mut comments = None;
    let mut source = String::from("manual");

    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--platform" => platform = Some(next_arg(&mut args, "--platform")?),
            "--keyword" => keyword = Some(next_arg(&mut args, "--keyword")?),
            "--title" => title = Some(next_arg(&mut args, "--title")?),
            "--url" => url = Some(next_arg(&mut args, "--url")?),
            "--author" => author = Some(next_arg(&mut args, "--author")?),
            "--likes" => likes = Some(parse_u64("--likes", next_arg(&mut args, "--likes")?)?),
            "--collects" => {
                collects = Some(parse_u64("--collects", next_arg(&mut args, "--collects")?)?);
            }
            "--comments" => {
                comments = Some(parse_u64("--comments", next_arg(&mut args, "--comments")?)?);
            }
            "--source" => source = next_arg(&mut args, "--source")?,
            value if value.starts_with('-') => {
                return Err(AppError::UnknownOption(value.to_owned()));
            }
            value => return Err(AppError::UnknownCommand(value.to_owned())),
        }
    }

    Ok(RadarCommand::Add(TrendSample {
        platform: platform.ok_or(AppError::MissingValue("--platform"))?,
        keyword: keyword.ok_or(AppError::MissingValue("--keyword"))?,
        title: title.ok_or(AppError::MissingValue("--title"))?,
        url,
        author,
        likes,
        collects,
        comments,
        source,
    }))
}

pub(crate) fn parse_radar_list(args: &[String]) -> Result<RadarCommand, AppError> {
    let mut platform = None;
    let mut keyword = None;
    let mut args = args.iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--platform" => platform = Some(next_arg(&mut args, "--platform")?),
            "--keyword" => keyword = Some(next_arg(&mut args, "--keyword")?),
            value if value.starts_with('-') => {
                return Err(AppError::UnknownOption(value.to_owned()));
            }
            value => return Err(AppError::UnknownCommand(value.to_owned())),
        }
    }

    Ok(RadarCommand::List { platform, keyword })
}

pub(crate) fn parse_radar_import(args: &[String]) -> Result<RadarCommand, AppError> {
    let path = args
        .first()
        .map(PathBuf::from)
        .ok_or(AppError::MissingValue("radar import <file.csv>"))?;

    let mut platform = None;
    let mut args = args[1..].iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--platform" => platform = Some(next_arg(&mut args, "--platform")?),
            value if value.starts_with('-') => {
                return Err(AppError::UnknownOption(value.to_owned()));
            }
            value => return Err(AppError::UnknownCommand(value.to_owned())),
        }
    }

    Ok(RadarCommand::Import { path, platform })
}

pub(crate) fn parse_radar_suggest(args: &[String]) -> Result<RadarCommand, AppError> {
    let mut article = None;
    let mut platform = None;
    let mut top = 10usize;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if let Some(a) = parse_radar_article_arg(arg, &mut args) {
            article = a;
        } else {
            match arg.as_str() {
                "--platform" => platform = Some(next_arg(&mut args, "--platform")?),
                "--top" => {
                    let v = next_arg(&mut args, "--top")?;
                    top = v.parse().map_err(|_| AppError::InvalidNumber {
                        flag: "--top",
                        value: v,
                    })?;
                }
                _ => {}
            }
        }
    }
    Ok(RadarCommand::Suggest {
        article: PathBuf::from(article.ok_or(AppError::MissingValue("suggest <article.md>"))?),
        platform: platform.ok_or(AppError::MissingValue("--platform"))?,
        top,
    })
}

pub(crate) fn parse_radar_article_arg(
    arg: &str,
    _args: &mut std::slice::Iter<String>,
) -> Option<Option<String>> {
    if !arg.starts_with('-') {
        Some(Some(arg.to_owned()))
    } else {
        None
    }
}

pub(crate) fn parse_radar_analyze(args: &[String]) -> Result<RadarCommand, AppError> {
    let article = args
        .first()
        .map(PathBuf::from)
        .ok_or(AppError::MissingValue("radar analyze <article.md>"))?;

    let mut platform = None;
    let mut top = 10usize;
    let mut args = args[1..].iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--platform" => platform = Some(next_arg(&mut args, "--platform")?),
            "--top" => {
                let v = next_arg(&mut args, "--top")?;
                top = v.parse().map_err(|_| AppError::InvalidNumber {
                    flag: "--top",
                    value: v,
                })?;
            }
            value if value.starts_with('-') => {
                return Err(AppError::UnknownOption(value.to_owned()));
            }
            value => return Err(AppError::UnknownCommand(value.to_owned())),
        }
    }

    Ok(RadarCommand::Analyze {
        article,
        platform: platform.ok_or(AppError::MissingValue("--platform"))?,
        top,
    })
}

pub(crate) fn parse_radar_scrape(args: &[String]) -> Result<RadarCommand, AppError> {
    let mut platform = None;
    let mut keyword = None;
    let mut count = 10usize;
    let mut url = None;
    let mut args = args.iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--platform" => platform = Some(next_arg(&mut args, "--platform")?),
            "--keyword" => keyword = Some(next_arg(&mut args, "--keyword")?),
            "--count" => {
                let v = next_arg(&mut args, "--count")?;
                count = v.parse().map_err(|_| AppError::InvalidNumber {
                    flag: "--count",
                    value: v,
                })?;
            }
            "--url" => url = Some(next_arg(&mut args, "--url")?),
            value if value.starts_with('-') => {
                return Err(AppError::UnknownOption(value.to_owned()));
            }
            value => return Err(AppError::UnknownCommand(value.to_owned())),
        }
    }

    Ok(RadarCommand::Scrape {
        platform: platform.ok_or(AppError::MissingValue("--platform"))?,
        keyword: keyword.ok_or(AppError::MissingValue("--keyword"))?,
        count,
        url,
    })
}

pub(crate) fn next_arg<'a>(
    args: &mut impl Iterator<Item = &'a String>,
    flag: &'static str,
) -> Result<String, AppError> {
    args.next().cloned().ok_or(AppError::MissingValue(flag))
}

pub(crate) fn parse_u64(flag: &'static str, value: String) -> Result<u64, AppError> {
    value
        .parse()
        .map_err(|_| AppError::InvalidNumber { flag, value })
}

pub fn run_radar(vault: &Path, command: &RadarCommand) -> Result<String, AppError> {
    match command {
        RadarCommand::Add(sample) => add_trend_sample(vault, sample),
        RadarCommand::List { platform, keyword } => list_trend_samples(vault, platform, keyword),
        RadarCommand::Import { path, platform } => import_csv(vault, path, platform.as_deref()),
        RadarCommand::Analyze {
            article,
            platform,
            top,
        } => analyze_article(vault, article, platform, *top),
        RadarCommand::Suggest {
            article,
            platform,
            top,
        } => suggest_titles(vault, article, platform, *top),
        RadarCommand::Scrape {
            platform,
            keyword,
            count,
            url,
        } => scrape_radar(vault, platform, keyword, *count, url.as_deref()),
    }
}

pub fn add_trend_sample(vault: &Path, sample: &TrendSample) -> Result<String, AppError> {
    let path = trend_store_path(vault);
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
    vault: &Path,
    platform: &Option<String>,
    keyword: &Option<String>,
) -> Result<String, AppError> {
    let path = trend_store_path(vault);
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

// ── radar import ──────────────────────────────────────────────────────────────

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
    vault: &Path,
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
    let store_path = trend_store_path(vault);
    if let Some(parent) = store_path.parent() {
        fs::create_dir_all(parent).map_err(|source| AppError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&store_path)
        .map_err(|source| AppError::Io {
            path: store_path.clone(),
            source,
        })?;

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

        writeln!(file, "{}", sample.to_json_line()).map_err(|source| AppError::Io {
            path: store_path.clone(),
            source,
        })?;
        count += 1;
    }

    Ok(format!(
        "imported {count} samples from {}",
        csv_path.display()
    ))
}

/// Parse a single CSV row, respecting double-quoted fields.
pub fn parse_csv_row(line: &str) -> Vec<String> {
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

// ── radar analyze ─────────────────────────────────────────────────────────────

pub fn analyze_article(
    vault: &Path,
    article: &Path,
    platform: &str,
    top: usize,
) -> Result<String, AppError> {
    let article = resolve_article_path(vault, article);
    let content = fs::read_to_string(&article).map_err(|source| AppError::Io {
        path: article.clone(),
        source,
    })?;

    let article_tokens = tokenize(&content);

    let store_path = trend_store_path(vault);
    let samples = load_all_samples(&store_path)?;

    let mut scored: Vec<(u64, &TrendSample)> = samples
        .iter()
        .filter(|s| s.platform == platform)
        .map(|s| {
            let title_tokens = tokenize(&s.title);
            let keyword_tokens = tokenize(&s.keyword);
            let overlap = count_overlap(&article_tokens, &title_tokens)
                + count_overlap(&article_tokens, &keyword_tokens) * 2;
            let engagement = s.engagement_score();
            let score = engagement.saturating_add(overlap as u64 * 100);
            (score, s)
        })
        .collect();

    scored.sort_by_key(|b| std::cmp::Reverse(b.0));

    let top_n: Vec<_> = scored.into_iter().take(top).collect();

    Ok(format_analyze_results(platform, &top_n))
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

/// Min token length to filter out single-char fragments and punctuation noise.
const MIN_TOKEN_LEN: usize = 2;

pub(crate) fn tokenize(text: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || ch.is_alphabetic() {
            current.push(ch);
        } else {
            if current.chars().count() >= MIN_TOKEN_LEN {
                tokens.push(current.to_lowercase());
            }
            current.clear();
        }
    }
    if current.chars().count() >= MIN_TOKEN_LEN {
        tokens.push(current.to_lowercase());
    }
    tokens
}

pub(crate) fn count_overlap(a: &[String], b: &[String]) -> usize {
    b.iter().filter(|t| a.contains(t)).count()
}

pub(crate) fn format_analyze_results(platform: &str, scored: &[(u64, &TrendSample)]) -> String {
    let mut output = format!("title suggestions for [{platform}]\n");
    if scored.is_empty() {
        output.push_str("  (no trend samples for this platform)");
        return output;
    }
    for (rank, (score, sample)) in scored.iter().enumerate() {
        output.push_str(&format!("  {}. {} (score={score}", rank + 1, sample.title));
        if let Some(likes) = sample.likes {
            output.push_str(&format!(", likes={likes}"));
        }
        output.push_str(&format!(", keyword={})\n", sample.keyword));
    }
    output.trim_end().to_owned()
}

// ── radar suggest ─────────────────────────────────────────────────────────────

/// Apply 4 golden title formulas to suggest titles based on article content
/// and trending data. Reference: "如何写出好标题" (green planet PPT).
pub fn suggest_titles(
    vault: &Path,
    article: &Path,
    platform: &str,
    top: usize,
) -> Result<String, AppError> {
    let article = resolve_article_path(vault, article);
    let content = fs::read_to_string(&article).map_err(|source| AppError::Io {
        path: article.clone(),
        source,
    })?;

    let front = parse_frontmatter(&content);
    let body = strip_frontmatter(&content);
    let orig_title = front.title.as_deref().unwrap_or("");
    let digest = front.digest.as_deref().unwrap_or("");

    let store_path = trend_store_path(vault);
    let samples = load_all_samples(&store_path).unwrap_or_default();
    let platform_samples: Vec<&TrendSample> =
        samples.iter().filter(|s| s.platform == platform).collect();

    let article_tokens = tokenize(body);

    let mut scored: Vec<(u64, &TrendSample)> = platform_samples
        .iter()
        .map(|s| (s.engagement_score(), *s))
        .collect();
    scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    let top_trends: Vec<&TrendSample> = scored.iter().take(top.min(10)).map(|(_, s)| *s).collect();

    let mut phrases: Vec<&str> = article_tokens.iter().map(|s| s.as_str()).collect();
    phrases.sort_by_key(|p| std::cmp::Reverse(p.chars().count()));
    let key_phrase = phrases.first().copied().unwrap_or("");

    let mut output = format!("title suggestions for [{platform}]");
    if !orig_title.is_empty() {
        output.push_str(&format!(" (current: {orig_title})"));
    }
    output.push('\n');
    output.push_str("────────────────────────────────────────\n\n");

    // ── Formula 1: 痛点 + 解决方案 ──
    output.push_str("▎痛点 + 解决方案\n");
    let pain_raw = extract_pain_point(body).unwrap_or("努力却没有成果");
    let pain_short = short_phrase(pain_raw, PAIN_LEN);
    let solution = first_paragraph_hook(body).unwrap_or("这里有答案");
    let solution_short = short_phrase(solution, SOLUTION_LEN);
    output.push_str(&format!("  {}总是{}？{}\n", "", pain_short, solution_short));
    push_trend_ref(&mut output, top_trends.first().copied());

    // ── Formula 2: 数字 + 利益结果 ──
    output.push_str("▎数字 + 利益结果\n");
    let real_sections: Vec<&str> = body
        .lines()
        .filter(|l| l.trim().starts_with("## "))
        .collect();
    let h2_count = real_sections.len().clamp(2, 8);
    let theme = real_sections
        .first()
        .map(|l| l.trim().trim_start_matches("## ").trim())
        .unwrap_or("改变认知");
    output.push_str(&format!(
        "  这本书我读了{}遍，总结出{}条关于{}的真相\n",
        h2_count,
        h2_count,
        short_phrase(theme, THEME_LEN),
    ));
    push_trend_ref(&mut output, top_trends.get(1).copied());

    // ── Formula 3: 故事悬念/冲突 ──
    output.push_str("▎故事悬念 / 冲突\n");
    let hook = first_paragraph_hook(body).unwrap_or(digest);
    let hook_short = short_phrase(hook, HOOK_LEN);
    let contrast = extract_contrast(body).unwrap_or("完全不同的答案");
    let contrast_short = short_phrase(contrast, CONTRAST_LEN);
    let f3 = if !hook.is_empty() {
        format!(
            "{}……这不是{}，而是{}",
            hook_short, key_phrase, contrast_short
        )
    } else {
        format!("我原本以为{}，没想到却是{}", key_phrase, contrast_short)
    };
    output.push_str(&format!("  {f3}\n"));
    push_trend_ref(&mut output, top_trends.get(2).copied());

    // ── Formula 4: 用户标签 + 情感共鸣 ──
    output.push_str("▎用户标签 + 情感共鸣\n");
    let label_raw = extract_reader_label(body).unwrap_or("每一个还在坚持的人");
    output.push_str(&format!(
        "  致所有热爱{}的人：{}\n",
        short_phrase(label_raw, LABEL_LEN),
        orig_title,
    ));
    push_trend_ref(&mut output, top_trends.get(3).copied());

    // ── trending references ──
    if !top_trends.is_empty() {
        output.push_str("────────────────────────────────────────\n");
        output.push_str("trending on this platform (for reference):\n");
        for (i, t) in top_trends.iter().take(top).enumerate() {
            output.push_str(&format!(
                "  {}. {} (score={})\n",
                i + 1,
                t.title,
                t.engagement_score()
            ));
        }
    }

    Ok(output.trim_end().to_owned())
}

/// Truncation lengths for title formula short phrases.
const PAIN_LEN: usize = 10;
const SOLUTION_LEN: usize = 12;
const THEME_LEN: usize = 6;
const HOOK_LEN: usize = 15;
const CONTRAST_LEN: usize = 15;
const LABEL_LEN: usize = 6;

fn push_trend_ref(output: &mut String, trend: Option<&TrendSample>) {
    if let Some(t) = trend {
        output.push_str(&format!(
            "  ↳ 参考: {} (likes={})\n\n",
            t.title,
            t.likes.unwrap_or(0)
        ));
    } else {
        output.push('\n');
    }
}

/// Strip block syntax and headings, return only plain paragraph text lines.
/// Truncate a string at the nearest Chinese char boundary, adding "…" if cut.
pub(crate) fn truncate_cn(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_owned();
    }
    let truncated: String = s.chars().take(max_chars).collect();
    format!("{truncated}…")
}

/// Extract first meaningful short phrase from text (not just a letter/number fragment).
pub(crate) fn short_phrase(s: &str, max_chars: usize) -> String {
    let clean: String = s
        .chars()
        .take_while(|c| *c != '.' && *c != ',' && *c != ';' && *c != '\n')
        .collect();
    if clean.chars().count() <= max_chars {
        return clean;
    }
    truncate_cn(&clean, max_chars)
}

pub(crate) fn body_text_only(body: &str) -> Vec<&str> {
    let mut in_block = false;
    body.lines()
        .filter(|l| {
            let t = l.trim();
            if t.starts_with(":::") {
                in_block = !in_block;
                return false;
            }
            if in_block {
                return false;
            }
            if t.starts_with('#') || t.starts_with('>') || t.is_empty() {
                return false;
            }
            if t.starts_with("---") || t.starts_with("***") {
                return false;
            }
            true
        })
        .collect()
}

pub(crate) fn extract_pain_point(body: &str) -> Option<&str> {
    let keywords = [
        "很难",
        "不容易",
        "崩溃",
        "放弃",
        "痛苦",
        "没有",
        "不知道",
        "怎么办",
    ];
    for line in body.lines() {
        let t = line.trim();
        if t.starts_with(':') || t.starts_with('#') || t.starts_with('>') || t.is_empty() {
            continue;
        }
        for kw in &keywords {
            if t.contains(kw) {
                return Some(t);
            }
        }
    }
    // Fallback: first real paragraph
    body.lines()
        .find(|l| {
            let t = l.trim();
            !t.is_empty()
                && !t.starts_with(':')
                && !t.starts_with('#')
                && !t.starts_with('>')
                && t.chars().count() > 10
        })
        .map(|l| l.trim())
}

pub(crate) fn extract_contrast(body: &str) -> Option<&str> {
    let paragraphs = body_text_only(body);
    for line in &paragraphs {
        if line.contains("不是") && line.contains("而是") {
            return Some(line);
        }
    }
    // Fallback: find characteristic phrase
    paragraphs
        .iter()
        .filter(|l| l.chars().count() > 10)
        .nth(2)
        .copied()
}

pub(crate) fn extract_reader_label(body: &str) -> Option<&str> {
    let labels = [
        "读书", "写作", "坚持", "努力", "成长", "挣扎", "孤独", "选择", "热爱", "艺术",
    ];
    for label in &labels {
        if body.contains(label) {
            return Some(label);
        }
    }
    let paragraphs = body_text_only(body);
    paragraphs.first().copied()
}

pub(crate) fn first_paragraph_hook(body: &str) -> Option<&str> {
    let paragraphs = body_text_only(body);
    paragraphs.first().copied()
}

pub(crate) fn trend_store_path(vault: &Path) -> PathBuf {
    vault.join(".moonpub").join("trends.jsonl")
}

// ── radar scrape ─────────────────────────────────────────────────────────────

/// Scrape trending articles for a platform keyword and store them in trends.jsonl.
///
/// Uses playwright-cli if found in PATH, otherwise falls back to curl.
/// Default search URL for wechat: Sogou WeChat search (public, no auth).
pub fn scrape_radar(
    vault: &Path,
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

    let store = trend_store_path(vault);
    if let Some(p) = store.parent() {
        fs::create_dir_all(p).map_err(|source| AppError::Io {
            path: p.to_path_buf(),
            source,
        })?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&store)
        .map_err(|source| AppError::Io {
            path: store.clone(),
            source,
        })?;
    for s in &samples {
        writeln!(file, "{}", s.to_json_line()).map_err(|source| AppError::Io {
            path: store.clone(),
            source,
        })?;
    }

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

    // Remove script/style blocks first
    let text = {
        let mut s = html.to_owned();
        // Remove <script> blocks
        while let (Some(a), Some(b)) = (s.find("<script"), s.find("</script>")) {
            if a < b {
                s = format!("{}{}", &s[..a], &s[b + 9..]);
            } else {
                break;
            }
        }
        s
    };

    // Extract text from <a>, <h3>, <h2> tags
    let tag_re_patterns = ["<h2", "<h3", "<a "];
    for &tag in &tag_re_patterns {
        let mut pos = 0;
        while let Some(start) = text[pos..].find(tag) {
            let abs_start = pos + start;
            // Find end of opening tag
            let Some(gt) = text[abs_start..].find('>') else {
                break;
            };
            let content_start = abs_start + gt + 1;
            // Find closing tag
            let close_tag = format!("</{}", &tag[1..].trim_end_matches(' '));
            let content_end = text[content_start..]
                .find(close_tag.as_str())
                .map(|i| content_start + i)
                .unwrap_or(content_start + 200.min(text.len() - content_start));

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

    // Deduplicate preserving order
    let mut seen = std::collections::HashSet::new();
    titles.retain(|t| seen.insert(t.clone()));
    titles
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
    // Decode common entities
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

// ── JSONL serialisation ───────────────────────────────────────────────────────

impl TrendSample {
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
