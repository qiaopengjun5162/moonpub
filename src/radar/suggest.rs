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

// ── tests ─────────────────────────────────────────────────────────────────────
#[cfg(test)]
mod tests {
    use super::*;

    // truncate_cn ── 中文边界截断 + 省略号
    #[test]
    fn truncate_cn_under_limit_returns_unchanged() {
        assert_eq!(truncate_cn("短", 10), "短");
        assert_eq!(truncate_cn("正好十个字啊啊啊啊", 10), "正好十个字啊啊啊啊");
        assert_eq!(truncate_cn("", 5), "");
    }

    #[test]
    fn truncate_cn_over_limit_adds_ellipsis() {
        assert_eq!(
            truncate_cn("一二三四五六七八九十一二", 10),
            "一二三四五六七八九十…"
        );
    }

    // short_phrase ── 英文标点前截断，超长走 truncate_cn
    #[test]
    fn short_phrase_stops_at_english_punctuation() {
        assert_eq!(short_phrase("Hello, world", 20), "Hello");
        assert_eq!(short_phrase("first; second", 20), "first");
        assert_eq!(short_phrase("line1. line2", 20), "line1");
    }

    #[test]
    fn short_phrase_no_punct_returns_full_when_short() {
        // 中文标点不触发截断；整串 ≤ max 时原样返回
        assert_eq!(short_phrase("坚持写作", 6), "坚持写作");
    }

    #[test]
    fn short_phrase_over_limit_truncates_at_char_boundary() {
        // 无英文标点、超 max=6 → 走 truncate_cn，取前 6 字加 …
        assert_eq!(short_phrase("坚持写作每天进步一点点", 6), "坚持写作每天…");
    }

    // body_text_only ── 过滤标题/引用/块/分割线/空行
    #[test]
    fn body_text_only_filters_non_paragraph_lines() {
        let body = "# 标题\n> 引用\n普通段落\n\n:::\nnote\n块内文字\n:::\n另一段";
        let out = body_text_only(body);
        assert_eq!(out, vec!["普通段落", "另一段"]);
    }

    #[test]
    fn body_text_only_toggles_fence_state() {
        // 块结束后恢复正常段落
        let body = "前段\n:::info\n隐藏\n:::\n后段";
        let out = body_text_only(body);
        assert_eq!(out, vec!["前段", "后段"]);
    }

    // extract_pain_point ── 关键词命中优先，否则兜底
    #[test]
    fn extract_pain_point_hits_keyword() {
        assert_eq!(
            extract_pain_point("今天学习很难坚持下来"),
            Some("今天学习很难坚持下来")
        );
        assert_eq!(
            extract_pain_point("我真的没有动力了"),
            Some("我真的没有动力了")
        );
    }

    #[test]
    fn extract_pain_point_falls_back_to_long_line() {
        let body = "短\n阳光明媚的午后我们结伴去公园散步享受微风";
        assert_eq!(
            extract_pain_point(body),
            Some("阳光明媚的午后我们结伴去公园散步享受微风")
        );
    }

    #[test]
    fn extract_pain_point_none_when_empty() {
        assert_eq!(extract_pain_point("\n\n"), None);
    }

    // extract_contrast ── "不是…而是" 命中，否则兜底第 3 段
    #[test]
    fn extract_contrast_hits_pattern() {
        assert_eq!(
            extract_contrast("这根本不是结束而是新开始"),
            Some("这根本不是结束而是新开始")
        );
    }

    #[test]
    fn extract_contrast_falls_back_to_third_paragraph() {
        // 无 "不是…而是" 时，取 chars>10 过滤后的第 3 个段落
        let body = "短\n短\n第一段足够长的文字内容啊\n第二段足够长的文字内容啊\n第三段足够长的文字内容啊\n短";
        assert_eq!(extract_contrast(body), Some("第三段足够长的文字内容啊"));
    }

    #[test]
    fn extract_contrast_none_when_too_few_paragraphs() {
        assert_eq!(extract_contrast("只有一段"), None);
    }

    // extract_reader_label ── 标签命中优先，否则兜底首段
    #[test]
    fn extract_reader_label_hits_known_label() {
        assert_eq!(extract_reader_label("我热爱读书和写作"), Some("读书"));
        assert_eq!(extract_reader_label("坚持是一种力量"), Some("坚持"));
    }

    #[test]
    fn extract_reader_label_falls_back_to_first_paragraph() {
        let body = "今天天气真好我们去爬山了\n第二段";
        assert_eq!(extract_reader_label(body), Some("今天天气真好我们去爬山了"));
    }

    // first_paragraph_hook
    #[test]
    fn first_paragraph_hook_returns_first() {
        assert_eq!(first_paragraph_hook("首段\n次段"), Some("首段"));
    }

    #[test]
    fn first_paragraph_hook_none_when_empty() {
        assert_eq!(first_paragraph_hook("\n\n# 标题\n"), None);
    }
}
