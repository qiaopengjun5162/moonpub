use std::fs;
use std::path::Path;

use crate::{
    article::{parse_frontmatter, resolve_article_path, strip_frontmatter},
    error::AppError,
};

use super::{TrendSample, load_all_samples, tokenize, trend_store_path};

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

/// Truncate a string at the nearest Chinese char boundary, adding "…" if cut.
fn truncate_cn(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        return s.to_owned();
    }
    let truncated: String = s.chars().take(max_chars).collect();
    format!("{truncated}…")
}

/// Extract first meaningful short phrase from text (not just a letter/number fragment).
fn short_phrase(s: &str, max_chars: usize) -> String {
    let clean: String = s
        .chars()
        .take_while(|c| *c != '.' && *c != ',' && *c != ';' && *c != '\n')
        .collect();
    if clean.chars().count() <= max_chars {
        return clean;
    }
    truncate_cn(&clean, max_chars)
}

/// Strip block syntax and headings, return only plain paragraph text lines.
fn body_text_only(body: &str) -> Vec<&str> {
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

fn extract_pain_point(body: &str) -> Option<&str> {
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

fn extract_contrast(body: &str) -> Option<&str> {
    let paragraphs = body_text_only(body);
    for line in &paragraphs {
        if line.contains("不是") && line.contains("而是") {
            return Some(line);
        }
    }
    paragraphs
        .iter()
        .filter(|l| l.chars().count() > 10)
        .nth(2)
        .copied()
}

fn extract_reader_label(body: &str) -> Option<&str> {
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

fn first_paragraph_hook(body: &str) -> Option<&str> {
    let paragraphs = body_text_only(body);
    paragraphs.first().copied()
}
