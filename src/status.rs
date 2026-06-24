use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::bundle::{ArticleBundle, ArticleStage};
use crate::error::AppError;
use crate::json_util::escape_json;

pub fn status(root: &Path) -> Result<String, AppError> {
    let articles_dir = root.join("Articles");
    let mut stages = Vec::new();

    for stage in ["drafts", "ready", "published"] {
        let dir = articles_dir.join(stage);
        stages.push((stage, list_markdown_files(&dir)?));
    }

    let statuses = read_statuses(root).unwrap_or_default();

    Ok(format_status(&stages, &statuses))
}

pub fn check_article(articles_dir: &Path, article: &Path) -> Result<String, AppError> {
    let article = crate::article::resolve_article_path(articles_dir, article);
    if article.extension().and_then(|ext| ext.to_str()) != Some("md") {
        return Err(AppError::InvalidArticlePath(article));
    }

    let bundle = ArticleBundle::from_markdown(&article)?;
    Ok(bundle.report())
}

fn status_store_path(articles_dir: &Path) -> PathBuf {
    articles_dir.join(".moonpub").join("status.jsonl")
}

/// Return the stage name ("drafts" | "ready" | "published") if the dir ends with one.
pub fn dir_stage(dir: &Path) -> Option<&str> {
    ArticleStage::from_dir(dir).map(ArticleStage::as_str)
}

pub fn add_status(
    articles_dir: &Path,
    slug: &str,
    status: &str,
    detail: &str,
) -> Result<String, AppError> {
    let path = status_store_path(articles_dir);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| AppError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|source| AppError::Io {
            path: path.clone(),
            source,
        })?;
    let line = format!(
        "{{\"slug\":\"{}\",\"status\":\"{}\",\"detail\":\"{}\"}}",
        escape_json(slug),
        status,
        detail
    );
    writeln!(file, "{line}").map_err(|source| AppError::Io {
        path: path.clone(),
        source,
    })?;
    Ok(format!("{slug}: {status}"))
}

fn read_statuses(articles_dir: &Path) -> Result<Vec<(String, String, String)>, AppError> {
    let path = status_store_path(articles_dir);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path).map_err(|source| AppError::Io {
        path: path.clone(),
        source,
    })?;
    let mut statuses = Vec::new();
    for line in content.lines().filter(|l| !l.trim().is_empty()) {
        let slug = crate::json_util::extract_json_string(line, "slug").unwrap_or_default();
        let status = crate::json_util::extract_json_string(line, "status").unwrap_or_default();
        let detail = crate::json_util::extract_json_string(line, "detail").unwrap_or_default();
        if !slug.is_empty() {
            statuses.push((slug, status, detail));
        }
    }
    Ok(statuses)
}

fn list_markdown_files(dir: &Path) -> Result<Vec<String>, AppError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let entries = fs::read_dir(dir).map_err(|source| AppError::Io {
        path: dir.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| AppError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("md")
            && let Some(name) = path.file_name().and_then(|name| name.to_str())
        {
            files.push(name.to_owned());
        }
    }

    files.sort();
    Ok(files)
}

fn format_status(stages: &[(&str, Vec<String>)], statuses: &[(String, String, String)]) -> String {
    let mut output = String::new();
    for (stage, files) in stages {
        output.push_str(&format!("-- {stage} --\n"));
        if files.is_empty() {
            output.push_str("  (empty)\n");
        } else {
            for file in files {
                let slug = file.trim_end_matches(".md");
                let latest = statuses
                    .iter()
                    .rev()
                    .find(|(s, _, _)| s == slug)
                    .map(|(_, st, d)| format!(" [{st}] {d}"))
                    .unwrap_or_default();
                output.push_str(&format!("  {file}{latest}\n"));
            }
        }
    }
    output.trim_end().to_owned()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::status::{check_article, dir_stage, status};
    use crate::test_helpers::{create_file, temp_root};

    #[test]
    fn status_lists_markdown_files_by_stage() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("status")?;
        create_file(&root.join("Articles/drafts/a.md"), "")?;
        create_file(&root.join("Articles/drafts/a.html"), "")?;
        create_file(&root.join("Articles/published/z.md"), "")?;

        let output = status(&root)?;

        assert!(output.contains("-- drafts --"));
        assert!(output.contains("  a.md"));
        assert!(output.contains("-- ready --"));
        assert!(output.contains("  (empty)"));
        assert!(output.contains("-- published --"));
        assert!(output.contains("  z.md"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn check_reports_missing_bundle_parts() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("check")?;
        create_file(&root.join("Articles/drafts/demo.md"), "")?;
        create_file(&root.join("Articles/drafts/demo.html"), "")?;

        let output = check_article(&root, Path::new("Articles/drafts/demo.md"))?;

        assert!(output.contains("markdown: ok"));
        assert!(output.contains("html: ok"));
        assert!(output.contains("draft_json: missing"));
        assert!(output.contains("publishable: no"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn check_reports_publishable_bundle() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("publishable")?;
        create_file(&root.join("Articles/ready/demo.md"), "")?;
        create_file(&root.join("Articles/ready/demo.html"), "")?;
        create_file(&root.join("Articles/ready/demo.draft.json"), "{}")?;

        let output = check_article(&root, Path::new("Articles/ready/demo.md"))?;

        assert!(output.contains("publishable: yes"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn dir_stage_identifies_stages() {
        assert_eq!(
            dir_stage(Path::new("/vault/Articles/drafts")),
            Some("drafts")
        );
        assert_eq!(dir_stage(Path::new("/vault/Articles/ready")), Some("ready"));
        assert_eq!(
            dir_stage(Path::new("/vault/Articles/published")),
            Some("published")
        );
        assert_eq!(dir_stage(Path::new("/vault/Articles")), None);
    }
}
