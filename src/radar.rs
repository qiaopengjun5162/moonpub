//! Radar command — trend sample management, analysis, and scraping.

use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    article::{parse_frontmatter, resolve_article_path, strip_frontmatter},
    error::AppError,
};

mod analyze;
mod cli;
mod scrape;
mod store;
pub use analyze::analyze_article;
pub(crate) use analyze::tokenize;
pub(crate) use cli::parse_radar_command;
pub use scrape::scrape_radar;
#[cfg(test)]
pub(crate) use scrape::{extract_from_snapshot, extract_samples, is_good_title, url_encode};
pub use store::{TrendSample, add_trend_sample, list_trend_samples};
pub(crate) use store::{load_all_samples, trend_store_path};

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

pub fn run_radar(articles_dir: &Path, command: &RadarCommand) -> Result<String, AppError> {
    match command {
        RadarCommand::Add(sample) => add_trend_sample(articles_dir, sample),
        RadarCommand::List { platform, keyword } => {
            list_trend_samples(articles_dir, platform, keyword)
        }
        RadarCommand::Import { path, platform } => {
            import_csv(articles_dir, path, platform.as_deref())
        }
        RadarCommand::Analyze {
            article,
            platform,
            top,
        } => analyze_article(articles_dir, article, platform, *top),
        RadarCommand::Suggest {
            article,
            platform,
            top,
        } => suggest_titles(articles_dir, article, platform, *top),
        RadarCommand::Scrape {
            platform,
            keyword,
            count,
            url,
        } => scrape_radar(articles_dir, platform, keyword, *count, url.as_deref()),
    }
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

// ── radar suggest ─────────────────────────────────────────────────────────────

/// Apply 4 golden title formulas to suggest titles based on article content
/// and trending data. Reference: "如何写出好标题" (green planet PPT).
pub fn suggest_titles(
    articles_dir: &Path,
    article: &Path,
    platform: &str,
    top: usize,
) -> Result<String, AppError> {
    let article = resolve_article_path(articles_dir, article);
    let content = fs::read_to_string(&article).map_err(|source| AppError::Io {
        path: article.clone(),
        source,
    })?;

    let front = parse_frontmatter(&content);
    let body = strip_frontmatter(&content);
    let orig_title = front.title.as_deref().unwrap_or("");
    let digest = front.digest.as_deref().unwrap_or("");

    let store_path = trend_store_path(articles_dir);
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
    output.push_str(&format!("  总是{}？{}\n", pain_short, solution_short));
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

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::cli::{Command, Options};
    use crate::radar::{
        RadarCommand, TrendSample, add_trend_sample, analyze_article, extract_from_snapshot,
        extract_samples, import_csv, is_good_title, list_trend_samples, parse_csv_row, url_encode,
    };
    use crate::test_helpers::{create_file, temp_root};

    #[test]
    fn parses_radar_add_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "radar".to_owned(),
            "add".to_owned(),
            "--platform".to_owned(),
            "xiaohongshu".to_owned(),
            "--keyword".to_owned(),
            "AI写作".to_owned(),
            "--title".to_owned(),
            "我的标题".to_owned(),
            "--likes".to_owned(),
            "42".to_owned(),
        ])?;

        let Command::Radar(RadarCommand::Add(sample)) = options.command else {
            panic!("expected radar add");
        };
        assert_eq!(sample.platform, "xiaohongshu");
        assert_eq!(sample.keyword, "AI写作");
        assert_eq!(sample.title, "我的标题");
        assert_eq!(sample.likes, Some(42));
        Ok(())
    }

    #[test]
    fn radar_add_and_list_samples() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("radar")?;
        add_trend_sample(
            &root,
            &TrendSample {
                platform: "wechat".to_owned(),
                keyword: "AI写作".to_owned(),
                title: "一个值得参考的标题".to_owned(),
                url: Some("https://example.com/post".to_owned()),
                author: Some("demo".to_owned()),
                likes: Some(100),
                collects: Some(50),
                comments: Some(8),
                source: "manual".to_owned(),
            },
        )?;

        let output = list_trend_samples(&root, &Some("wechat".to_owned()), &None)?;

        assert!(output.contains("[wechat] AI写作 | 一个值得参考的标题"));
        assert!(output.contains("likes=100"));
        assert!(output.contains("collects=50"));
        assert!(output.contains("comments=8"));
        assert!(output.contains("https://example.com/post"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn radar_list_filters_by_platform() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("radar-filter")?;
        add_sample(&root, "wechat", "公众号标题")?;
        add_sample(&root, "xiaohongshu", "小红书标题")?;

        let output = list_trend_samples(&root, &Some("xiaohongshu".to_owned()), &None)?;

        assert!(output.contains("小红书标题"));
        assert!(!output.contains("公众号标题"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn trend_sample_json_roundtrip_escapes_text() {
        let sample = TrendSample {
            platform: "wechat".to_owned(),
            keyword: "AI\"写作".to_owned(),
            title: "第一行\n第二行".to_owned(),
            url: None,
            author: None,
            likes: None,
            collects: None,
            comments: None,
            source: "manual".to_owned(),
        };

        let line = sample.to_json_line();
        let parsed = TrendSample::from_json_line(&line).expect("valid json line");

        assert_eq!(parsed.keyword, sample.keyword);
        assert_eq!(parsed.title, sample.title);
    }

    #[test]
    fn csv_import_basic() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("csv-import")?;
        let csv = root.join("trends.csv");
        create_file(
            &csv,
            "platform,keyword,title,likes,source\nwechat,AI写作,标题一,100,csv\nwechat,AI写作,标题二,200,csv\n",
        )?;

        let msg = import_csv(&root, &csv, None)?;
        assert!(msg.contains("imported 2 samples"));

        let output = list_trend_samples(&root, &Some("wechat".to_owned()), &None)?;
        assert!(output.contains("标题一"));
        assert!(output.contains("标题二"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn csv_import_uses_default_platform() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("csv-default-platform")?;
        let csv = root.join("trends.csv");
        create_file(&csv, "keyword,title\nAI写作,一篇好文章\n")?;

        import_csv(&root, &csv, Some("xiaohongshu"))?;

        let output = list_trend_samples(&root, &Some("xiaohongshu".to_owned()), &None)?;
        assert!(output.contains("一篇好文章"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn csv_import_quoted_fields() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("csv-quoted")?;
        let csv = root.join("trends.csv");
        create_file(&csv, "platform,keyword,title\nwechat,AI,\"标题含,逗号\"\n")?;

        import_csv(&root, &csv, None)?;

        let output = list_trend_samples(&root, &Some("wechat".to_owned()), &None)?;
        assert!(output.contains("标题含,逗号"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn csv_parse_row_handles_quoted_commas() {
        let row = r#""hello,world",foo,"bar""#;
        let fields = parse_csv_row(row);
        assert_eq!(fields, vec!["hello,world", "foo", "bar"]);
    }

    #[test]
    fn analyze_ranks_by_engagement() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("analyze")?;
        add_trend_sample(
            &root,
            &TrendSample {
                platform: "wechat".to_owned(),
                keyword: "AI写作".to_owned(),
                title: "高互动标题".to_owned(),
                url: None,
                author: None,
                likes: Some(500),
                collects: Some(200),
                comments: Some(50),
                source: "manual".to_owned(),
            },
        )?;
        add_trend_sample(
            &root,
            &TrendSample {
                platform: "wechat".to_owned(),
                keyword: "AI写作".to_owned(),
                title: "低互动标题".to_owned(),
                url: None,
                author: None,
                likes: Some(5),
                collects: None,
                comments: None,
                source: "manual".to_owned(),
            },
        )?;

        let article = root.join("demo.md");
        create_file(&article, "# AI写作技巧\n这篇文章讨论AI写作。")?;

        let output = analyze_article(&root, &article, "wechat", 10)?;

        let high_pos = output.find("高互动标题").expect("高互动标题 not found");
        let low_pos = output.find("低互动标题").expect("低互动标题 not found");
        assert!(high_pos < low_pos, "高互动应排在低互动之前");

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn analyze_filters_by_platform() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("analyze-platform")?;
        add_sample(&root, "wechat", "公众号专属标题")?;
        add_sample(&root, "xiaohongshu", "小红书专属标题")?;

        let article = root.join("demo.md");
        create_file(&article, "# AI写作")?;

        let output = analyze_article(&root, &article, "wechat", 10)?;

        assert!(output.contains("公众号专属标题"));
        assert!(!output.contains("小红书专属标题"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn parses_radar_import_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "radar".to_owned(),
            "import".to_owned(),
            "trends.csv".to_owned(),
            "--platform".to_owned(),
            "wechat".to_owned(),
        ])?;

        let Command::Radar(RadarCommand::Import { path, platform }) = options.command else {
            panic!("expected radar import");
        };
        assert_eq!(path, PathBuf::from("trends.csv"));
        assert_eq!(platform.as_deref(), Some("wechat"));
        Ok(())
    }

    #[test]
    fn parses_radar_analyze_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "radar".to_owned(),
            "analyze".to_owned(),
            "demo.md".to_owned(),
            "--platform".to_owned(),
            "wechat".to_owned(),
            "--top".to_owned(),
            "5".to_owned(),
        ])?;

        let Command::Radar(RadarCommand::Analyze {
            article,
            platform,
            top,
        }) = options.command
        else {
            panic!("expected radar analyze");
        };
        assert_eq!(article, PathBuf::from("demo.md"));
        assert_eq!(platform, "wechat");
        assert_eq!(top, 5);
        Ok(())
    }

    #[test]
    fn parses_radar_scrape_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "radar".to_owned(),
            "scrape".to_owned(),
            "--platform".to_owned(),
            "wechat".to_owned(),
            "--keyword".to_owned(),
            "AI写作".to_owned(),
            "--count".to_owned(),
            "5".to_owned(),
        ])?;
        let Command::Radar(RadarCommand::Scrape {
            platform,
            keyword,
            count,
            url,
        }) = options.command
        else {
            panic!("expected Scrape");
        };
        assert_eq!(platform, "wechat");
        assert_eq!(keyword, "AI写作");
        assert_eq!(count, 5);
        assert!(url.is_none());
        Ok(())
    }

    #[test]
    fn url_encode_handles_chinese() {
        let encoded = url_encode("AI写作");
        assert!(encoded.starts_with("AI"));
        assert!(encoded.contains('%'), "汉字应被百分比编码");
        assert!(!encoded.contains(' '));
    }

    #[test]
    fn extract_from_snapshot_parses_titles() {
        let snapshot = r#"
- document
  - main
    - heading "AI时代的写作技巧：10个让你效率翻倍的方法" [ref=e1]
    - link "普通人如何用AI写出爆款文章" [ref=e2]
    - link "关注" [ref=e3]
    - link "更多" [ref=e4]
"#;
        let titles = extract_from_snapshot(snapshot);
        assert!(titles.contains(&"AI时代的写作技巧：10个让你效率翻倍的方法".to_owned()));
        assert!(titles.contains(&"普通人如何用AI写出爆款文章".to_owned()));
        assert!(!titles.contains(&"关注".to_owned()), "太短的导航文字应过滤");
    }

    #[test]
    fn is_good_title_filters_short_and_nav() {
        assert!(is_good_title("AI时代的写作技巧让你效率翻倍"));
        assert!(!is_good_title("关注"), "太短");
        assert!(!is_good_title("var x = function(){}"), "JS代码");
        assert!(!is_good_title("ab"), "太短ASCII");
    }

    #[test]
    fn scrape_stores_samples_in_jsonl() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("scrape-store")?;

        let html = r#"<html><body>
            <h3><a href="/a1">坚持每天写作：我用AI辅助的30天实验</a></h3>
            <h3><a href="/a2">公众号涨粉秘诀：内容为王还是运营为王</a></h3>
            <h3><a href="/a3">短</a></h3>
        </body></html>"#;

        let samples = extract_samples(html, "wechat", "AI写作", 10);
        assert!(!samples.is_empty(), "应提取到文章标题");
        assert!(samples.iter().all(|s| s.platform == "wechat"));
        assert!(samples.iter().all(|s| s.keyword == "AI写作"));
        assert!(samples.iter().all(|s| s.source == "scrape"));

        for s in &samples {
            assert!(s.title.chars().count() >= 6, "标题太短: {}", s.title);
        }

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    fn add_sample(root: &Path, platform: &str, title: &str) -> Result<(), crate::error::AppError> {
        add_trend_sample(
            root,
            &TrendSample {
                platform: platform.to_owned(),
                keyword: "AI".to_owned(),
                title: title.to_owned(),
                url: None,
                author: None,
                likes: None,
                collects: None,
                comments: None,
                source: "manual".to_owned(),
            },
        )?;
        Ok(())
    }
}
