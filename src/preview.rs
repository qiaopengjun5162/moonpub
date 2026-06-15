use std::path::{Path, PathBuf};

use crate::error::AppError;

pub fn preview_article(vault: &Path, article: &Path) -> Result<String, AppError> {
    let article = crate::article::resolve_article_path(vault, article);
    let slug = article
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?;
    let dir = article
        .parent()
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?;
    let html = dir.join(format!("{slug}.html"));

    if !html.exists() {
        return Err(AppError::NoHtml(html));
    }

    #[cfg(target_os = "macos")]
    let opener = "open";
    #[cfg(target_os = "linux")]
    let opener = "xdg-open";
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let opener = "start";

    std::process::Command::new(opener)
        .arg(&html)
        .status()
        .map_err(|source| AppError::Io {
            path: PathBuf::from(opener),
            source,
        })?;

    Ok(format!("opening {}", html.display()))
}

#[cfg(test)]
mod tests {
    use crate::error::AppError;
    use crate::preview::preview_article;
    use crate::test_helpers::{create_file, temp_root};

    #[test]
    fn preview_fails_without_html() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("preview-no-html")?;
        let md = root.join("demo.md");
        create_file(&md, "---\ntitle: T\n---\n正文\n")?;

        let err = preview_article(&root, &md).unwrap_err();
        assert!(matches!(err, AppError::NoHtml(_)), "应报 NoHtml 错误");

        std::fs::remove_dir_all(root)?;
        Ok(())
    }
}
