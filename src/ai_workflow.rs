use std::fs;
use std::path::Path;

use crate::article::resolve_article_path;
use crate::draft::write_article_file;
use crate::error::AppError;
use crate::ship::ship_article;

pub fn write_article(articles_dir: &Path, idea: &str) -> Result<String, AppError> {
    let api_key = crate::ai::default_api_key()?;
    let article = crate::ai::generate_article(idea, &api_key)?;
    let path = write_article_file(articles_dir, idea, &article)?;
    Ok(format!("generated\n  {}", path.display()))
}

pub fn polish_article(articles_dir: &Path, article: &Path) -> Result<String, AppError> {
    let api_key = crate::ai::default_api_key()?;
    let art_path = resolve_article_path(articles_dir, article);
    let content = read_article(&art_path)?;
    let polished = crate::ai::polish_article(&content, &api_key)?;
    write_article_content(&art_path, &polished)?;
    Ok(format!("polished\n  {}", art_path.display()))
}

pub fn expand_article(articles_dir: &Path, article: &Path) -> Result<String, AppError> {
    let api_key = crate::ai::default_api_key()?;
    let art_path = resolve_article_path(articles_dir, article);
    let content = read_article(&art_path)?;
    let expanded = crate::ai::expand_notes(&content, &api_key)?;
    let output = expanded_article_output(&content, &expanded);
    write_article_content(&art_path, &output)?;
    Ok(format!("expanded\n  {}", art_path.display()))
}

pub fn ship_ai_article(
    articles_dir: &Path,
    config_path: Option<&Path>,
    article: &Path,
    style: Option<&str>,
) -> Result<String, AppError> {
    let api_key = crate::ai::default_api_key()?;
    let art_path = resolve_article_path(articles_dir, article);
    let content = read_article(&art_path)?;
    let polished = crate::ai::polish_article(&content, &api_key)?;
    write_article_content(&art_path, &polished)?;
    ship_article(articles_dir, config_path, &art_path, style)
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
    use super::expanded_article_output;

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
}
