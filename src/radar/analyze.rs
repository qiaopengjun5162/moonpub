use std::fs;
use std::path::Path;

use crate::{article::resolve_article_path, error::AppError};

use super::{TrendSample, load_all_samples, trend_store_path};

pub fn analyze_article(
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

    let article_tokens = tokenize(&content);

    let store_path = trend_store_path(articles_dir);
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

fn count_overlap(a: &[String], b: &[String]) -> usize {
    b.iter().filter(|t| a.contains(t)).count()
}

fn format_analyze_results(platform: &str, scored: &[(u64, &TrendSample)]) -> String {
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_with(title: &str, keyword: &str, likes: Option<u64>) -> TrendSample {
        TrendSample {
            platform: "xhs".into(),
            keyword: keyword.into(),
            title: title.into(),
            url: None,
            author: None,
            likes,
            collects: None,
            comments: None,
            source: "s".into(),
        }
    }

    // ── tokenize ───────────────────────────────────────────────────
    #[test]
    fn tokenize_empty_is_empty() {
        assert!(tokenize("").is_empty());
    }

    #[test]
    fn tokenize_lowercases_ascii_words() {
        assert_eq!(tokenize("Hello WORLD"), vec!["hello", "world"]);
    }

    #[test]
    fn tokenize_drops_single_char_tokens() {
        // 长度 < 2 的片段被丢弃
        assert!(tokenize("a b c").is_empty());
    }

    #[test]
    fn tokenize_splits_on_punctuation() {
        assert_eq!(tokenize("hello,world.test"), vec!["hello", "world", "test"]);
    }

    #[test]
    fn tokenize_keeps_cjk_phrase_as_one_token() {
        // 中文无空格时整串视为一个 token（每个汉字 is_alphabetic）
        assert_eq!(tokenize("渲染管线"), vec!["渲染管线"]);
    }

    #[test]
    fn tokenize_mixed_ascii_cjk_number() {
        assert_eq!(
            tokenize("Rust 渲染管线 2024"),
            vec!["rust", "渲染管线", "2024"]
        );
    }

    #[test]
    fn tokenize_truncates_trailing_single_char() {
        // "v2.0" → "v2" 保留，"0" 单字丢弃
        assert_eq!(tokenize("v2.0"), vec!["v2"]);
    }

    // ── count_overlap ──────────────────────────────────────────────
    #[test]
    fn count_overlap_counts_shared_tokens() {
        let a = vec!["rust".to_string(), "go".to_string()];
        let b = vec!["rust".to_string(), "python".to_string()];
        assert_eq!(count_overlap(&a, &b), 1);
    }

    #[test]
    fn count_overlap_empty_b_is_zero() {
        let a = vec!["rust".to_string()];
        assert_eq!(count_overlap(&a, &[]), 0);
    }

    #[test]
    fn count_overlap_empty_a_is_zero() {
        let b = vec!["rust".to_string()];
        assert_eq!(count_overlap(&[], &b), 0);
    }

    // ── format_analyze_results ─────────────────────────────────────
    #[test]
    fn format_empty_shows_no_samples() {
        let out = format_analyze_results("xhs", &[]);
        assert!(out.contains("title suggestions for [xhs]"));
        assert!(out.contains("no trend samples for this platform"));
    }

    #[test]
    fn format_ranks_scores_keyword_and_optional_likes() {
        let a = sample_with("标题一", "kw1", Some(10));
        let b = sample_with("标题二", "kw2", None);
        let scored = vec![(150u64, &a), (80u64, &b)];
        let out = format_analyze_results("xhs", &scored);
        assert!(out.contains("1. 标题一 (score=150, likes=10, keyword=kw1)"));
        assert!(out.contains("2. 标题二 (score=80, keyword=kw2)"));
    }
}
