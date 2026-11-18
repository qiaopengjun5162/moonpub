use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::bundle::{ArticleBundle, ArticleStage};
use crate::error::AppError;
use crate::json_util::escape_json;

pub struct StatusFileEntry {
    pub file: String,
    pub slug: String,
    pub latest_status: Option<String>,
    pub latest_detail: Option<String>,
    /// 同目录下是否已经有 `<slug>.html`。用来判断「该渲染」还是「该检查」，
    /// 不能只看 `latest_status`——状态日志可能缺、可能过期。
    pub has_html: bool,
}

pub struct StatusStageReport {
    pub stage: String,
    pub files: Vec<StatusFileEntry>,
}

pub fn status(root: &Path) -> Result<String, AppError> {
    let stages = status_report(root)?;
    Ok(format_status(&stages))
}

pub fn status_report(root: &Path) -> Result<Vec<StatusStageReport>, AppError> {
    let articles_dir = root.join("Articles");
    let statuses = read_statuses(root).unwrap_or_default();
    let mut stages = Vec::new();

    for stage in ["drafts", "ready", "published"] {
        let dir = articles_dir.join(stage);
        stages.push(StatusStageReport {
            stage: stage.to_owned(),
            files: list_markdown_files(&dir, &statuses)?,
        });
    }

    Ok(stages)
}

pub fn check_article(articles_dir: &Path, article: &Path) -> Result<String, AppError> {
    let article = crate::article::resolve_article_path(articles_dir, article);
    if article.extension().and_then(|ext| ext.to_str()) != Some("md") {
        return Err(AppError::InvalidArticlePath(article));
    }

    let bundle = ArticleBundle::from_markdown(&article)?;
    Ok(bundle.report())
}

pub fn check_article_bundle(
    articles_dir: &Path,
    article: &Path,
) -> Result<ArticleBundle, AppError> {
    let article = crate::article::resolve_article_path(articles_dir, article);
    if article.extension().and_then(|ext| ext.to_str()) != Some("md") {
        return Err(AppError::InvalidArticlePath(article));
    }

    ArticleBundle::from_markdown(&article)
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

fn list_markdown_files(
    dir: &Path,
    statuses: &[(String, String, String)],
) -> Result<Vec<StatusFileEntry>, AppError> {
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
            let file = name.to_owned();
            let slug = file.trim_end_matches(".md").to_owned();
            let latest = statuses
                .iter()
                .rev()
                .find(|(entry_slug, _, _)| entry_slug == &slug);
            let (latest_status, latest_detail) =
                effective_status_for_stage(dir, &slug, latest.cloned())?;
            let has_html = path.with_extension("html").exists();
            files.push(StatusFileEntry {
                file,
                slug,
                latest_status,
                latest_detail,
                has_html,
            });
        }
    }

    files.sort_by(|left, right| left.file.cmp(&right.file));
    Ok(files)
}

fn effective_status_for_stage(
    dir: &Path,
    slug: &str,
    latest: Option<(String, String, String)>,
) -> Result<(Option<String>, Option<String>), AppError> {
    let Some(stage) = dir_stage(dir) else {
        return Ok(match latest {
            Some((_, status, detail)) => (Some(status), empty_string_as_none(detail)),
            None => (None, None),
        });
    };

    let fallback_detail = read_bundle_media_id(dir, slug)?;
    let fallback = match stage {
        "ready" => Some(("ready".to_owned(), fallback_detail.clone())),
        "published" => Some(("published".to_owned(), fallback_detail.clone())),
        _ => None,
    };

    let Some((_, status, detail)) = latest else {
        return Ok(split_status_detail(fallback));
    };

    let status_matches_stage = matches!(
        (stage, status.as_str()),
        ("ready", "ready" | "published") | ("published", "published")
    );
    if status_matches_stage {
        return Ok((Some(status), empty_string_as_none(detail)));
    }

    Ok(split_status_detail(
        fallback.or(Some((status, empty_string_as_none(detail)))),
    ))
}

fn split_status_detail(
    status: Option<(String, Option<String>)>,
) -> (Option<String>, Option<String>) {
    match status {
        Some((status, detail)) => (Some(status), detail),
        None => (None, None),
    }
}

fn empty_string_as_none(value: String) -> Option<String> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        None
    } else {
        Some(trimmed.to_owned())
    }
}

fn read_bundle_media_id(dir: &Path, slug: &str) -> Result<Option<String>, AppError> {
    let path = dir.join(format!("{slug}.media_id"));
    if !path.exists() {
        return Ok(None);
    }
    let media_id = fs::read_to_string(&path).map_err(|source| AppError::Io {
        path: path.clone(),
        source,
    })?;
    Ok(empty_string_as_none(media_id))
}

/// 建议的下一步：`(command, 一句话说明)`。
///
/// 文本输出（`status` / `workspace`）和 JSON（`next_command` / `next_step`）
/// 共用这一份判断，避免两处各长出一个口径、慢慢漂移。
pub(crate) fn next_action(stages: &[StatusStageReport]) -> (String, &'static str) {
    let first_draft = stages
        .iter()
        .find(|stage| stage.stage == "drafts")
        .and_then(|stage| stage.files.first());
    let first_ready = stages
        .iter()
        .find(|stage| stage.stage == "ready")
        .and_then(|stage| stage.files.first());
    let first_published = stages
        .iter()
        .find(|stage| stage.stage == "published")
        .and_then(|stage| stage.files.first());

    if let Some(file) = first_draft {
        // 还没渲染就推 `check`，等于把人送进「publishable: no」的死胡同：
        // 检查不能推进任何东西，渲染才能。
        if file.has_html {
            (
                format!("moonpub check Articles/drafts/{}", file.file),
                "inspect the first draft article and continue preview or push",
            )
        } else {
            (
                format!("moonpub render Articles/drafts/{}", file.file),
                "render the article to generate html and draft.json",
            )
        }
    } else if let Some(file) = first_ready {
        (
            format!("moonpub check Articles/ready/{}", file.file),
            "inspect the first ready article and continue preview or publish",
        )
    } else if let Some(file) = first_published {
        (
            format!("moonpub check Articles/published/{}", file.file),
            "inspect the latest published bundle or start a new article",
        )
    } else {
        (
            "moonpub new \"你的第一篇文章\"".to_owned(),
            "create your first article draft to start the workflow",
        )
    }
}

/// 文本版清单。
///
/// 每条都打印 **工作区相对路径**（`Articles/drafts/x.md`），不是裸文件名：
/// 此前打印的是 `x.md`，用户照着敲 `moonpub render x.md` 会得到
/// `No such file or directory`——工具自己印出的名字不是自己的合法参数。
/// 同一个工作区里 `--json` 早就有可执行的 `next_command`，文本版却没有，
/// 于是「机器拿到的信息比人拿到的更有用」，这里把两端对齐。
fn format_status(stages: &[StatusStageReport]) -> String {
    let mut output = String::new();
    for stage in stages {
        output.push_str(&format!("-- {} --\n", stage.stage));
        if stage.files.is_empty() {
            output.push_str("  (empty)\n");
        } else {
            for file in &stage.files {
                let latest = match (&file.latest_status, &file.latest_detail) {
                    (Some(status), Some(detail)) => format!(" [{status}] {detail}"),
                    (Some(status), None) => format!(" [{status}]"),
                    _ => String::new(),
                };
                output.push_str(&format!(
                    "  Articles/{}/{}{}\n",
                    stage.stage, file.file, latest
                ));
            }
        }
    }
    let (next_command, next_step) = next_action(stages);
    output.push_str(&format!("next: {next_command}\n"));
    output.push_str(&format!("step: {next_step}"));
    output.trim_end().to_owned()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::status::{check_article, dir_stage, status, status_report};
    use crate::test_helpers::{create_file, temp_root};

    #[test]
    fn status_lists_markdown_files_by_stage() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("status")?;
        create_file(&root.join("Articles/drafts/a.md"), "")?;
        create_file(&root.join("Articles/drafts/a.html"), "")?;
        create_file(&root.join("Articles/published/z.md"), "")?;

        let output = status(&root)?;

        assert!(output.contains("-- drafts --"));
        assert!(output.contains("  Articles/drafts/a.md"));
        assert!(output.contains("-- ready --"));
        assert!(output.contains("  (empty)"));
        assert!(output.contains("-- published --"));
        assert!(output.contains("  Articles/published/z.md"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    /// 清单里打印出来的那一行，必须是它自己下一条命令的合法参数。
    ///
    /// 此前打印的是裸文件名 `a.md`，用户照着敲 `moonpub render a.md` 会得到
    /// `No such file or directory`——工具印出的名字在工具自己的解析里不成立。
    #[test]
    fn status_text_prints_paths_that_are_valid_arguments() -> Result<(), Box<dyn std::error::Error>>
    {
        let root = temp_root("status-usable")?;
        create_file(&root.join("Articles/drafts/带标注.md"), "# 标题\n\n正文\n")?;
        create_file(&root.join("Articles/drafts/无标注.md"), "# 标题\n\n正文\n")?;
        create_file(
            &root.join(".moonpub/status.jsonl"),
            "{\"slug\":\"带标注\",\"status\":\"rendered\",\"detail\":\"ok\"}\n",
        )?;

        let output = status(&root)?;
        let mut checked = 0;
        for line in output
            .lines()
            .map(str::trim_start)
            .filter(|line| line.starts_with("Articles/"))
        {
            // 状态标注（`[rendered] ok`）排在路径之后，取第一个空白前的部分。
            let printed = line.split_whitespace().next().expect("清单行不可能为空");
            check_article(&root, Path::new(printed))?;
            checked += 1;
        }
        assert_eq!(checked, 2, "清单里应有两条文章路径：{output}");

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    /// 未渲染的草稿，推荐的下一步必须是 `render`。
    ///
    /// 此前一律推 `check`，而 `check` 对空包只会回 `publishable: no` ——
    /// 工具把人送到一个不能推进任何东西的地方，等于让人再猜一次。
    #[test]
    fn status_text_recommends_render_before_check() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("status-next-render")?;
        create_file(&root.join("Articles/drafts/未渲染.md"), "# 标题\n")?;

        let output = status(&root)?;
        assert!(
            output.contains("next: moonpub render Articles/drafts/未渲染.md"),
            "{output}"
        );

        // 渲染之后才轮到检查。
        create_file(&root.join("Articles/drafts/未渲染.html"), "<p>x</p>")?;
        let output = status(&root)?;
        assert!(
            output.contains("next: moonpub check Articles/drafts/未渲染.md"),
            "{output}"
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    /// 空工作区不能只回三行 `(empty)` 就把人晾在那儿。
    #[test]
    fn status_text_gives_next_step_when_workspace_is_empty()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("status-empty-next")?;

        let output = status(&root)?;

        assert!(output.contains("next: moonpub new"), "缺少下一步：{output}");
        assert!(!output.contains("next: moonpub check"), "{output}");
        assert!(output.contains("step: "), "缺少说明：{output}");

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn status_report_includes_latest_status_metadata() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("status-report")?;
        create_file(&root.join("Articles/drafts/a.md"), "")?;
        create_file(
            &root.join(".moonpub/status.jsonl"),
            "{\"slug\":\"a\",\"status\":\"ready\",\"detail\":\"confirmed\"}\n",
        )?;

        let report = status_report(&root)?;
        let drafts = report
            .iter()
            .find(|stage| stage.stage == "drafts")
            .expect("drafts stage");
        let entry = drafts
            .files
            .iter()
            .find(|file| file.slug == "a")
            .expect("draft entry");

        assert_eq!(entry.file, "a.md");
        assert_eq!(entry.latest_status.as_deref(), Some("ready"));
        assert_eq!(entry.latest_detail.as_deref(), Some("confirmed"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn status_report_prefers_ready_stage_over_stale_rendered_status()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("status-stage-fallback")?;
        create_file(&root.join("Articles/ready/a.md"), "")?;
        create_file(&root.join("Articles/ready/a.media_id"), "media-123")?;
        create_file(
            &root.join(".moonpub/status.jsonl"),
            "{\"slug\":\"a\",\"status\":\"rendered\",\"detail\":\"\"}\n",
        )?;

        let report = status_report(&root)?;
        let ready = report
            .iter()
            .find(|stage| stage.stage == "ready")
            .expect("ready stage");
        let entry = ready
            .files
            .iter()
            .find(|file| file.slug == "a")
            .expect("ready entry");

        assert_eq!(entry.latest_status.as_deref(), Some("ready"));
        assert_eq!(entry.latest_detail.as_deref(), Some("media-123"));

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
