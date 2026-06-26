use std::fs;
use std::path::Path;

use crate::article::resolve_article_path;
use crate::config::Config;
use crate::draft::write_article_file;
use crate::error::AppError;
use crate::ship::ship_article;

fn resolve_ai_config(cfg: &Config) -> Result<(crate::ai::AiProvider, String, String), AppError> {
    let provider = cfg
        .ai_provider
        .as_deref()
        .unwrap_or("deepseek")
        .parse::<crate::ai::AiProvider>()?;
    let model = cfg
        .ai_model
        .clone()
        .unwrap_or_else(|| provider.default_model().to_owned());
    let api_key = cfg
        .ai_api_key
        .clone()
        .map(Ok)
        .unwrap_or_else(|| crate::ai::api_key(provider))?;
    Ok((provider, model, api_key))
}

pub fn write_article(articles_dir: &Path, cfg: &Config, idea: &str) -> Result<String, AppError> {
    let (provider, model, api_key) = resolve_ai_config(cfg)?;
    let user_prompt = format!(
        "请根据以下想法，写一篇微信公众号文章。\n\n想法：{idea}\n\n要求：800-2000字，有明确的标题和结构。"
    );
    let article = crate::ai::call_ai(
        provider,
        Some(&model),
        crate::ai::ARTICLE_SYSTEM_PROMPT,
        &user_prompt,
        &api_key,
    )?;
    let path = write_article_file(articles_dir, idea, &article)?;
    Ok(format!("generated\n  {}", path.display()))
}

pub fn polish_article(
    articles_dir: &Path,
    cfg: &Config,
    article: &Path,
) -> Result<String, AppError> {
    let (provider, model, api_key) = resolve_ai_config(cfg)?;
    let art_path = resolve_article_path(articles_dir, article);
    let content = read_article(&art_path)?;
    let user_prompt = format!("请润色以下文章：\n\n{content}");
    let polished = crate::ai::call_ai(
        provider,
        Some(&model),
        crate::ai::POLISH_SYSTEM_PROMPT,
        &user_prompt,
        &api_key,
    )?;
    write_article_content(&art_path, &polished)?;
    Ok(format!("polished\n  {}", art_path.display()))
}

pub fn expand_article(
    articles_dir: &Path,
    cfg: &Config,
    article: &Path,
) -> Result<String, AppError> {
    let (provider, model, api_key) = resolve_ai_config(cfg)?;
    let art_path = resolve_article_path(articles_dir, article);
    let content = read_article(&art_path)?;
    let user_prompt =
        format!("请将以下读书笔记展开为一篇完整的微信公众号文章。\n\n笔记内容：\n\n{content}");
    let expanded = crate::ai::call_ai(
        provider,
        Some(&model),
        crate::ai::EXPAND_SYSTEM_PROMPT,
        &user_prompt,
        &api_key,
    )?;
    let output = expanded_article_output(&content, &expanded);
    write_article_content(&art_path, &output)?;
    Ok(format!("expanded\n  {}", art_path.display()))
}

pub fn ship_ai_article(
    articles_dir: &Path,
    config_path: Option<&Path>,
    cfg: &Config,
    article: &Path,
    style: Option<&str>,
) -> Result<String, AppError> {
    let (provider, model, api_key) = resolve_ai_config(cfg)?;
    let art_path = resolve_article_path(articles_dir, article);
    let content = read_article(&art_path)?;
    let user_prompt = format!("请润色以下文章：\n\n{content}");
    let polished = crate::ai::call_ai(
        provider,
        Some(&model),
        crate::ai::POLISH_SYSTEM_PROMPT,
        &user_prompt,
        &api_key,
    )?;
    write_article_content(&art_path, &polished)?;
    ship_article(articles_dir, config_path, &art_path, style)
}

pub fn ai_cover_text(
    cfg: &Config,
    article: &Path,
    content: &str,
) -> Result<(String, String), AppError> {
    let (provider, model, api_key) = resolve_ai_config(cfg)?;
    let user_prompt = format!(
        "请根据下面这篇文章，提炼适合公众号封面的标题和副标题。\n\n文章路径：{}\n\n文章内容：\n\n{}",
        article.display(),
        content
    );
    let response = crate::ai::call_ai(
        provider,
        Some(&model),
        crate::ai::COVER_SYSTEM_PROMPT,
        &user_prompt,
        &api_key,
    )?;
    Ok(parse_cover_ai_response(&response))
}

fn parse_cover_ai_response(response: &str) -> (String, String) {
    let mut title = String::new();
    let mut subtitle = String::new();
    for line in response.lines() {
        let line = line.trim();
        if let Some(value) = line.strip_prefix("title:") {
            title = value.trim().to_owned();
        } else if let Some(value) = line.strip_prefix("subtitle:") {
            subtitle = value.trim().to_owned();
        }
    }
    (title, subtitle)
}

fn read_article(path: &Path) -> Result<String, AppError> {
    fs::read_to_string(path).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn write_article_content(path: &Path, content: &str) -> Result<(), AppError> {
    fs::write(path, content).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })
}

fn expanded_article_output(original: &str, expanded: &str) -> String {
    let front = if original.starts_with("---") {
        original
            .lines()
            .skip(1)
            .take_while(|line| line.trim() != "---")
            .map(|line| format!("{line}\n"))
            .collect::<String>()
    } else {
        String::new()
    };

    if front.is_empty() {
        expanded.to_owned()
    } else {
        format!("---\n{front}---\n\n{expanded}")
    }
}

#[cfg(test)]
mod tests {
    use super::{expanded_article_output, parse_cover_ai_response};

    #[test]
    fn expanded_output_preserves_original_frontmatter() {
        let original = "---\ntitle: Demo\ndigest: Keep me\n---\n\nrough notes";
        let expanded = "polished body";

        let output = expanded_article_output(original, expanded);

        assert_eq!(
            output,
            "---\ntitle: Demo\ndigest: Keep me\n---\n\npolished body"
        );
    }

    #[test]
    fn parse_cover_ai_response_extracts_title_and_subtitle() {
        let response = "title: 把书读厚的办法\nsubtitle: 关键不是速度，而是把零散感受重新组织起来";
        let (title, subtitle) = parse_cover_ai_response(response);

        assert_eq!(title, "把书读厚的办法");
        assert_eq!(subtitle, "关键不是速度，而是把零散感受重新组织起来");
    }
}
