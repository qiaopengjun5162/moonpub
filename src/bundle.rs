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

/// 一个文章包的下一步建议：命令 + 一句话说明。
///
/// 文本报告（`moonpub check`）和 `check --json` 共用这一份判断。
/// 此前 JSON 带着 `next_command`、文本只丢下一句 `publishable: no` 就结束，
/// 于是新手顺着 `moonpub status` 的推荐走过来，正好撞在墙上。
pub struct BundleNextStep {
    pub command: String,
    pub step: &'static str,
}

impl ArticleBundle {
    /// 缺 `html` / `draft_json` 就先渲染；渲染完还没推过就推草稿；
    /// 其余情况回本地预览看一眼。判断只看包里有什么，不猜用户意图。
    ///
    /// `root` 只用来决定路径怎么印（工作区相对形式）；判断本身与它无关。
    pub fn next_step(&self, root: &Path) -> BundleNextStep {
        let markdown = crate::human_path::display_path(root, &self.markdown);
        if !self.has_markdown() {
            // 构造之后 Markdown 被删了（少见但真实）：先让人去看包，别推 preview。
            BundleNextStep {
                command: format!("moonpub check {markdown}"),
                step: "the markdown file is missing; inspect the bundle",
            }
        } else if !self.has_html() || !self.has_draft_json() {
            BundleNextStep {
                command: format!("moonpub render {markdown}"),
                step: "render the article to generate html and draft.json",
            }
        } else if !self.has_media_id() {
            BundleNextStep {
                command: format!("moonpub push {markdown} --render"),
                step: "push the article to WeChat drafts after review",
            }
        } else {
            BundleNextStep {
                command: format!("moonpub preview {markdown}"),
                step: "review the local preview or continue in the WeChat backend",
            }
        }
    }

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

    /// 文本版报告。`root` 只决定路径怎么印：工作区内的文件一律印相对形式，
    /// 与 `status` 列表、`next:` 引导三处逐字相同。
    pub fn report(&self, root: &Path) -> String {
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
                crate::human_path::display_path(root, path)
            ));
        }

        let media_id_exists = self.media_id.exists();
        output.push_str(&format!(
            "  media_id: {} {}\n",
            marker(media_id_exists),
            crate::human_path::display_path(root, &self.media_id)
        ));
        let next = self.next_step(root);
        output.push_str(&format!(
            "  publishable: {}\n  next: {}\n  step: {}",
            if complete { "yes" } else { "no" },
            next.command,
            next.step
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

    /// 这份包对应的微信草稿 id（`<slug>.media_id` 的内容）。
    ///
    /// 与 [`Self::has_media_id`] 的区别是口径更严：文件存在但内容为空时返回 `None`。
    /// `update-draft` / `delete-draft` 拿它当参数，空串不是合法 id——「文件在」和
    /// 「有个 id 能用」是两件事，调用方真正需要的是后者。
    pub fn media_id(&self) -> Result<Option<String>, AppError> {
        read_media_id_file(&self.media_id)
    }

    pub fn publishable(&self) -> bool {
        self.has_markdown() && self.has_html() && self.has_draft_json()
    }
}

/// 读 `<slug>.media_id` 文件的内容，空文件当「没有 id」。
///
/// 抽成自由函数是因为三处都要读它：`status` 的清单、`preview` 的下一步建议、
/// `bundle` 自己的判断。「文件在不在」用 `exists()` 就够了，但只要有人拿这个值去
/// 当 `update-draft` / `delete-draft` 的参数，就必须确认它非空——空串会一路传进
/// 命令行变成 `delete-draft ""`，报的错跟真实原因毫无关系。
pub fn read_media_id_file(path: &Path) -> Result<Option<String>, AppError> {
    if !path.exists() {
        return Ok(None);
    }
    let media_id = fs::read_to_string(path).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let media_id = media_id.trim();
    Ok((!media_id.is_empty()).then(|| media_id.to_owned()))
}

/// 同 [`read_media_id_file`]，按「目录 + slug」拼路径，省掉调用方各自拼字符串。
pub fn read_media_id(dir: &Path, slug: &str) -> Result<Option<String>, AppError> {
    read_media_id_file(&dir.join(format!("{slug}.media_id")))
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
        let output = bundle.report(&root);

        assert!(output.contains("markdown: ok"));
        assert!(output.contains("html: ok"));
        assert!(output.contains("draft_json: missing"));
        assert!(output.contains("publishable: no"));
        // 缺 draft_json 就直说下一步是渲染，不能只丢一句 `publishable: no`。
        assert!(output.contains("next: moonpub render"), "{output}");
        assert!(!output.contains("next: moonpub push"), "{output}");

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    /// Markdown 在构造之后消失时，不该推荐 preview（那会直接失败）。
    #[test]
    fn bundle_next_step_falls_back_to_check_when_markdown_vanished()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("bundle-next-vanished")?;
        let article = root.join("Articles/drafts/demo.md");
        create_file(&article, "# demo")?;
        create_file(&root.join("Articles/drafts/demo.html"), "<p>demo</p>")?;
        create_file(&root.join("Articles/drafts/demo.draft.json"), "{}")?;

        let bundle = ArticleBundle::from_markdown(&article)?;
        std::fs::remove_file(&article)?;

        assert!(bundle.next_step(&root).command.starts_with("moonpub check"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn bundle_report_points_at_push_once_the_bundle_is_rendered()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("bundle-next-push")?;
        let article = root.join("Articles/drafts/demo.md");
        create_file(&article, "# demo")?;
        create_file(&root.join("Articles/drafts/demo.html"), "<p>demo</p>")?;
        create_file(&root.join("Articles/drafts/demo.draft.json"), "{}")?;

        let bundle = ArticleBundle::from_markdown(&article)?;
        let output = bundle.report(&root);

        assert!(output.contains("publishable: yes"), "{output}");
        assert!(output.contains("next: moonpub push"), "{output}");
        assert!(!output.contains("next: moonpub render"), "{output}");

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
