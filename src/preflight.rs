use std::path::{Path, PathBuf};

use crate::article::resolve_article_path;
use crate::bundle::ArticleBundle;
use crate::error::AppError;
use crate::layout_audit::audit_html_file;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightCheck {
    pub id: &'static str,
    pub status: &'static str,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreflightReport {
    pub article_path: PathBuf,
    pub html_path: PathBuf,
    pub draft_json_path: PathBuf,
    pub media_id_path: PathBuf,
    pub passed: bool,
    pub checks: Vec<PreflightCheck>,
    pub next_command: String,
    pub next_step: &'static str,
}

pub fn preflight_article(articles_dir: &Path, article: &Path) -> Result<PreflightReport, AppError> {
    let article = resolve_article_path(articles_dir, article);
    let bundle = ArticleBundle::from_markdown(&article)?;
    let mut checks = Vec::new();

    push_check(
        &mut checks,
        "markdown",
        bundle.has_markdown(),
        format!("markdown exists: {}", bundle.markdown_path().display()),
        format!("markdown missing: {}", bundle.markdown_path().display()),
    );
    push_check(
        &mut checks,
        "html",
        bundle.has_html(),
        format!("rendered HTML exists: {}", bundle.html_path().display()),
        format!("rendered HTML missing: {}", bundle.html_path().display()),
    );
    push_check(
        &mut checks,
        "draft_json",
        bundle.has_draft_json(),
        format!("draft JSON exists: {}", bundle.draft_json_path().display()),
        format!("draft JSON missing: {}", bundle.draft_json_path().display()),
    );

    if bundle.has_html() {
        let audit = audit_html_file(bundle.html_path())?;
        if audit.passed {
            checks.push(PreflightCheck {
                id: "layout_audit",
                status: if audit.warnings.is_empty() {
                    "pass"
                } else {
                    "warn"
                },
                message: if audit.warnings.is_empty() {
                    "layout audit passed".to_owned()
                } else {
                    format!(
                        "layout audit passed with warnings: {}",
                        audit.warnings.join("; ")
                    )
                },
            });
        } else {
            checks.push(PreflightCheck {
                id: "layout_audit",
                status: "fail",
                message: format!("layout audit failed: {}", audit.errors.join("; ")),
            });
        }
    } else {
        checks.push(PreflightCheck {
            id: "layout_audit",
            status: "skip",
            message: "rendered HTML missing; run render before layout audit".to_owned(),
        });
    }

    checks.push(PreflightCheck {
        id: "media_id",
        status: if bundle.has_media_id() {
            "pass"
        } else {
            "warn"
        },
        message: if bundle.has_media_id() {
            format!("media_id exists: {}", bundle.media_id_path().display())
        } else {
            "media_id missing; article has not been pushed to WeChat draft yet".to_owned()
        },
    });

    let passed = checks.iter().all(|check| check.status != "fail");
    let (next_command, next_step) = if !bundle.has_html() || !bundle.has_draft_json() {
        (
            format!("moonpub render {}", bundle.markdown_path().display()),
            "render the article before preview or WeChat draft push",
        )
    } else if checks
        .iter()
        .any(|check| check.id == "layout_audit" && check.status == "fail")
    {
        (
            format!("moonpub layout-audit {}", bundle.html_path().display()),
            "fix the rendered HTML compatibility issues before publishing",
        )
    } else if !bundle.has_media_id() {
        (
            format!("moonpub push {} --render", bundle.markdown_path().display()),
            "review local preview, then explicitly push to WeChat draft when ready",
        )
    } else {
        (
            "moonpub wechat-health".to_owned(),
            "check browser automation login before backend preview-send",
        )
    };

    Ok(PreflightReport {
        article_path: bundle.markdown_path().to_path_buf(),
        html_path: bundle.html_path().to_path_buf(),
        draft_json_path: bundle.draft_json_path().to_path_buf(),
        media_id_path: bundle.media_id_path().to_path_buf(),
        passed,
        checks,
        next_command,
        next_step,
    })
}

fn push_check(
    checks: &mut Vec<PreflightCheck>,
    id: &'static str,
    ok: bool,
    ok_message: String,
    fail_message: String,
) {
    checks.push(PreflightCheck {
        id,
        status: if ok { "pass" } else { "fail" },
        message: if ok { ok_message } else { fail_message },
    });
}

#[cfg(test)]
mod tests {
    use crate::preflight::preflight_article;
    use crate::test_helpers::{create_file, temp_root};

    #[test]
    fn preflight_reports_missing_render_outputs() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("preflight-missing-render")?;
        let article = root.join("Articles/drafts/demo.md");
        create_file(&article, "# demo")?;

        let report = preflight_article(&root, &article)?;

        assert!(!report.passed);
        assert!(
            report
                .checks
                .iter()
                .any(|check| check.id == "html" && check.status == "fail")
        );
        assert_eq!(
            report.next_command,
            format!("moonpub render {}", article.display())
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn preflight_passes_local_ready_bundle_without_media_id()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("preflight-local-ready")?;
        let article = root.join("Articles/drafts/demo.md");
        create_file(&article, "# demo")?;
        create_file(
            &root.join("Articles/drafts/demo.html"),
            r#"<section style="margin:0;"><p style="color:#333;">正文</p></section>"#,
        )?;
        create_file(&root.join("Articles/drafts/demo.draft.json"), "{}")?;

        let report = preflight_article(&root, &article)?;

        assert!(report.passed);
        assert!(
            report
                .checks
                .iter()
                .any(|check| check.id == "media_id" && check.status == "warn")
        );
        assert_eq!(
            report.next_command,
            format!("moonpub push {} --render", article.display())
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }
}
