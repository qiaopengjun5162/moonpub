use std::fs;
use std::path::{Path, PathBuf};

use crate::error::AppError;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArticleStage {
    Drafts,
    Ready,
    Published,
}

impl ArticleStage {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Drafts => "drafts",
            Self::Ready => "ready",
            Self::Published => "published",
        }
    }

    pub fn from_dir(dir: &Path) -> Option<Self> {
        match dir.file_name()?.to_str()? {
            "drafts" => Some(Self::Drafts),
            "ready" => Some(Self::Ready),
            "published" => Some(Self::Published),
            _ => None,
        }
    }
}

pub struct ArticleBundle {
    markdown: PathBuf,
    html: PathBuf,
    draft_json: PathBuf,
    media_id: PathBuf,
}

impl ArticleBundle {
    pub fn from_markdown(markdown: &Path) -> Result<Self, AppError> {
        if markdown.extension().and_then(|ext| ext.to_str()) != Some("md") {
            return Err(AppError::InvalidArticlePath(markdown.to_path_buf()));
        }

        let slug = markdown
            .file_stem()
            .and_then(|stem| stem.to_str())
            .ok_or_else(|| AppError::InvalidArticlePath(markdown.to_path_buf()))?;
        let dir = markdown
            .parent()
            .ok_or_else(|| AppError::InvalidArticlePath(markdown.to_path_buf()))?;

        Ok(Self {
            markdown: markdown.to_path_buf(),
            html: dir.join(format!("{slug}.html")),
            draft_json: dir.join(format!("{slug}.draft.json")),
            media_id: dir.join(format!("{slug}.media_id")),
        })
    }

    pub fn report(&self) -> String {
        let required = [
            ("markdown", &self.markdown),
            ("html", &self.html),
            ("draft_json", &self.draft_json),
        ];

        let mut output = String::from("article bundle\n");
        let mut complete = true;
        for (label, path) in required {
            let exists = path.exists();
            complete &= exists;
            output.push_str(&format!(
                "  {label}: {} {}\n",
                marker(exists),
                path.display()
            ));
        }

        let media_id_exists = self.media_id.exists();
        output.push_str(&format!(
            "  media_id: {} {}\n",
            marker(media_id_exists),
            self.media_id.display()
        ));
        output.push_str(&format!(
            "  publishable: {}",
            if complete { "yes" } else { "no" }
        ));
        output
    }

    pub fn markdown_path(&self) -> &Path {
        &self.markdown
    }

    pub fn html_path(&self) -> &Path {
        &self.html
    }

    pub fn draft_json_path(&self) -> &Path {
        &self.draft_json
    }

    pub fn media_id_path(&self) -> &Path {
        &self.media_id
    }

    pub fn has_markdown(&self) -> bool {
        self.markdown.exists()
    }

    pub fn has_html(&self) -> bool {
        self.html.exists()
    }

    pub fn has_draft_json(&self) -> bool {
        self.draft_json.exists()
    }

    pub fn has_media_id(&self) -> bool {
        self.media_id.exists()
    }

    pub fn publishable(&self) -> bool {
        self.has_markdown() && self.has_html() && self.has_draft_json()
    }
}

pub fn move_article_bundle(
    current_dir: &Path,
    slug: &str,
    target_stage: ArticleStage,
) -> Result<Option<PathBuf>, AppError> {
    let Some(stage) = ArticleStage::from_dir(current_dir) else {
        return Ok(None);
    };
    if stage == target_stage {
        return Ok(None);
    }
    if stage != ArticleStage::Drafts && stage != ArticleStage::Ready {
        return Ok(None);
    }

    let target = current_dir
        .parent()
        .map(|parent| parent.join(target_stage.as_str()))
        .unwrap_or_else(|| current_dir.join(target_stage.as_str()));
    fs::create_dir_all(&target).map_err(|source| AppError::Io {
        path: target.clone(),
        source,
    })?;

    for ext in ["md", "html", "draft.json", "media_id"] {
        let src = current_dir.join(format!("{slug}.{ext}"));
        if src.exists() {
            let dst = target.join(format!("{slug}.{ext}"));
            fs::rename(&src, &dst).map_err(|source| AppError::Io {
                path: src.clone(),
                source,
            })?;
        }
    }

    Ok(Some(target))
}

fn marker(exists: bool) -> &'static str {
    if exists { "ok" } else { "missing" }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::bundle::{ArticleBundle, ArticleStage, move_article_bundle};
    use crate::test_helpers::{create_file, temp_root};

    #[test]
    fn bundle_report_marks_missing_parts() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("bundle-report")?;
        let article = root.join("Articles/drafts/demo.md");
        create_file(&article, "# demo")?;
        create_file(&root.join("Articles/drafts/demo.html"), "<p>demo</p>")?;

        let bundle = ArticleBundle::from_markdown(&article)?;
        let output = bundle.report();

        assert!(output.contains("markdown: ok"));
        assert!(output.contains("html: ok"));
        assert!(output.contains("draft_json: missing"));
        assert!(output.contains("publishable: no"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn bundle_stage_detects_article_stages() {
        assert_eq!(
            ArticleStage::from_dir(Path::new("/vault/Articles/drafts")),
            Some(ArticleStage::Drafts)
        );
        assert_eq!(
            ArticleStage::from_dir(Path::new("/vault/Articles/ready")),
            Some(ArticleStage::Ready)
        );
        assert_eq!(
            ArticleStage::from_dir(Path::new("/vault/Articles/published")),
            Some(ArticleStage::Published)
        );
        assert_eq!(ArticleStage::from_dir(Path::new("/vault/Articles")), None);
    }

    #[test]
    fn move_article_bundle_moves_known_files_to_target_stage()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("bundle-move")?;
        let drafts = root.join("Articles/drafts");
        create_file(&drafts.join("demo.md"), "# demo")?;
        create_file(&drafts.join("demo.html"), "<p>demo</p>")?;
        create_file(&drafts.join("demo.draft.json"), "{}")?;
        create_file(&drafts.join("demo.media_id"), "media_id")?;

        let target = move_article_bundle(&drafts, "demo", ArticleStage::Ready)?.expect("moved");

        assert_eq!(target, root.join("Articles/ready"));
        assert!(root.join("Articles/ready/demo.md").exists());
        assert!(root.join("Articles/ready/demo.html").exists());
        assert!(root.join("Articles/ready/demo.draft.json").exists());
        assert!(root.join("Articles/ready/demo.media_id").exists());
        assert!(!root.join("Articles/drafts/demo.md").exists());

        std::fs::remove_dir_all(root)?;
        Ok(())
    }
}
