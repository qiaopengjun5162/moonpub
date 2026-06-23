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
