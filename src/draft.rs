use std::fs;
use std::path::{Path, PathBuf};

use crate::error::AppError;

pub fn new_article(articles_dir: &Path, title: &str) -> Result<String, AppError> {
    let path = draft_article_path(articles_dir, title)?;
    let template = format!(
        r#"---
title: {title}
digest:
date:
tags: []
---

:::intro

:::

:::summary

:::
"#
    );

    fs::write(&path, &template).map_err(|source| AppError::Io {
        path: path.clone(),
        source,
    })?;

    Ok(format!("created\n  {}", path.display()))
}

pub fn write_article_file(
    articles_dir: &Path,
    title: &str,
    content: &str,
) -> Result<PathBuf, AppError> {
    let path = draft_article_path(articles_dir, title)?;
    fs::write(&path, content).map_err(|source| AppError::Io {
        path: path.clone(),
        source,
    })?;
    Ok(path)
}

fn draft_article_path(articles_dir: &Path, title: &str) -> Result<PathBuf, AppError> {
    let drafts = articles_dir.join("Articles").join("drafts");
    fs::create_dir_all(&drafts).map_err(|source| AppError::Io {
        path: drafts.clone(),
        source,
    })?;

    let slug = title
        .chars()
        .map(|c| if c.is_whitespace() { '-' } else { c })
        .collect::<String>()
        .split('-')
        .filter(|s| !s.is_empty())
        .collect::<Vec<_>>()
        .join("-");

    let path = drafts.join(format!("{slug}.md"));
    if path.exists() {
        return Err(AppError::Io {
            path,
            source: std::io::Error::new(
                std::io::ErrorKind::AlreadyExists,
                "article already exists",
            ),
        });
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use std::fs;

    use crate::draft::{new_article, write_article_file};
    use crate::error::AppError;
    use crate::test_helpers::temp_root;

    #[test]
    fn new_article_creates_template_in_drafts() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("draft-new")?;

        let output = new_article(&root, "我的 第一篇 文章")?;

        let path = root.join("Articles/drafts/我的-第一篇-文章.md");
        let content = fs::read_to_string(&path)?;
        assert!(output.contains(&path.display().to_string()));
        assert!(content.contains("title: 我的 第一篇 文章"));
        assert!(content.contains(":::intro"));
        assert!(content.contains(":::summary"));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn write_article_file_rejects_existing_draft() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("draft-existing")?;
        let path = write_article_file(&root, "same title", "first")?;

        let err = write_article_file(&root, "same title", "second").unwrap_err();

        assert_eq!(path, root.join("Articles/drafts/same-title.md"));
        let AppError::Io { source, .. } = err else {
            panic!("expected Io error for existing draft");
        };
        assert_eq!(source.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(fs::read_to_string(&path)?, "first");

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
