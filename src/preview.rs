use std::path::{Path, PathBuf};

use crate::error::AppError;

pub fn preview_article(articles_dir: &Path, article: &Path) -> Result<String, AppError> {
    preview_article_with_open(articles_dir, article, true)
}

pub fn preview_article_with_open(
    articles_dir: &Path,
    article: &Path,
    open_browser: bool,
) -> Result<String, AppError> {
    let article = crate::article::resolve_article_path(articles_dir, article);
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

    let next = format!("next: moonpub push {} --render", article.display());

    if !open_browser {
        return Ok(format!("preview ready {}\n{}", html.display(), next));
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

    Ok(format!("opening {}\n{}", html.display(), next))
}

#[cfg(test)]
mod tests {
    use crate::error::AppError;
    use crate::preview::{preview_article, preview_article_with_open};
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

    #[test]
    fn preview_can_skip_opening_browser() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("preview-no-open")?;
        let md = root.join("demo.md");
        let html = root.join("demo.html");
        create_file(&md, "---\ntitle: T\n---\n正文\n")?;
        create_file(&html, "<p>正文</p>")?;

        let output = preview_article_with_open(&root, &md, false)?;

        assert_eq!(
            output,
            format!(
                "preview ready {}\nnext: moonpub push {} --render",
                html.display(),
                md.display()
            )
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }
}
