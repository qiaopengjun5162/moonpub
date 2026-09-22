use super::escape_json;
use super::serialize_json;
use crate::article::parse_frontmatter;
use crate::bundle::ArticleBundle;
use crate::preflight::PreflightReport;
use crate::push::PushOutput;
use serde::Serialize;
use std::path::Path;
#[derive(Serialize)]
struct PreviewJsonPayload<'a> {
    command: &'static str,
    article_path: String,
    html_path: String,
    opened_browser: bool,
    next_command: &'a str,
}

pub(crate) fn preview_json(
    article_path: &Path,
    html_path: &Path,
    open_browser: bool,
    next_command: &str,
) -> String {
    serialize_json(&PreviewJsonPayload {
        command: "preview",
        article_path: article_path.display().to_string(),
        html_path: html_path.display().to_string(),
        opened_browser: open_browser,
        next_command,
    })
}

pub(crate) fn check_json(bundle: &ArticleBundle, configured_theme: Option<&str>) -> String {
    let frontmatter_theme = bundle
        .has_markdown()
        .then(|| std::fs::read_to_string(bundle.markdown_path()).ok())
        .flatten()
        .and_then(|markdown| parse_frontmatter(&markdown).theme);
    let configured_theme = configured_theme
        .map(str::trim)
        .filter(|theme| !theme.is_empty());
    let effective_theme = frontmatter_theme
        .as_deref()
        .or(configured_theme)
        .unwrap_or("default");
    let theme_json = frontmatter_theme
        .as_deref()
        .map(|theme| format!("\"{}\"", escape_json(theme)))
        .unwrap_or_else(|| "null".to_owned());
    let theme_source = if frontmatter_theme.is_some() {
        "article_frontmatter"
    } else if configured_theme.is_some() {
        "wechat_config"
    } else {
        "default"
    };
    let next_command = if !bundle.has_html() || !bundle.has_draft_json() {
        format!("moonpub render {}", bundle.markdown_path().display())
    } else if !bundle.has_media_id() {
        format!("moonpub push {} --render", bundle.markdown_path().display())
    } else if bundle.publishable() {
        format!("moonpub preview {}", bundle.markdown_path().display())
    } else {
        format!("moonpub check {}", bundle.markdown_path().display())
    };
    let next_step = if !bundle.has_html() || !bundle.has_draft_json() {
        "render the article to generate html and draft.json"
    } else if !bundle.has_media_id() {
        "push the article to WeChat drafts after review"
    } else if bundle.publishable() {
        "review the local preview or continue in the WeChat backend"
    } else {
        "inspect the missing bundle files and continue the publish flow"
    };
    format!(
        "{{\"command\":\"check\",\"article_path\":\"{}\",\"html_path\":\"{}\",\"draft_json_path\":\"{}\",\"media_id_path\":\"{}\",\"has_markdown\":{},\"has_html\":{},\"has_draft_json\":{},\"has_media_id\":{},\"publishable\":{},\"theme\":{},\"effective_theme\":\"{}\",\"theme_source\":\"{}\",\"next_command\":\"{}\",\"next_step\":\"{}\"}}",
        escape_json(&bundle.markdown_path().display().to_string()),
        escape_json(&bundle.html_path().display().to_string()),
        escape_json(&bundle.draft_json_path().display().to_string()),
        escape_json(&bundle.media_id_path().display().to_string()),
        bundle.has_markdown(),
        bundle.has_html(),
        bundle.has_draft_json(),
        bundle.has_media_id(),
        bundle.publishable(),
        theme_json,
        escape_json(effective_theme),
        theme_source,
        escape_json(&next_command),
        escape_json(next_step)
    )
}

pub(crate) fn preflight_text(report: &PreflightReport) -> String {
    let mut output = format!(
        "preflight {}\n  article: {}\n  html: {}\n  draft_json: {}\n  media_id: {}",
        if report.passed { "passed" } else { "failed" },
        report.article_path.display(),
        report.html_path.display(),
        report.draft_json_path.display(),
        report.media_id_path.display()
    );
    output.push_str("\n  checks:");
    for check in &report.checks {
        output.push_str(&format!(
            "\n    - {} [{}]: {}",
            check.id, check.status, check.message
        ));
    }
    output.push_str(&format!(
        "\n  next: {}\n  next_step: {}",
        report.next_command, report.next_step
    ));
    output
}

pub(crate) fn preflight_json(report: &PreflightReport) -> String {
    let checks = report
        .checks
        .iter()
        .map(|check| {
            format!(
                "{{\"id\":\"{}\",\"status\":\"{}\",\"message\":\"{}\"}}",
                escape_json(check.id),
                escape_json(check.status),
                escape_json(&check.message)
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"command\":\"preflight\",\"article_path\":\"{}\",\"html_path\":\"{}\",\"draft_json_path\":\"{}\",\"media_id_path\":\"{}\",\"passed\":{},\"checks\":[{}],\"next_command\":\"{}\",\"next_step\":\"{}\"}}",
        escape_json(&report.article_path.display().to_string()),
        escape_json(&report.html_path.display().to_string()),
        escape_json(&report.draft_json_path.display().to_string()),
        escape_json(&report.media_id_path.display().to_string()),
        report.passed,
        checks,
        escape_json(&report.next_command),
        escape_json(report.next_step)
    )
}

pub(crate) fn push_json(
    article_path: &Path,
    media_id: &str,
    stage: &str,
    next_step: &str,
) -> String {
    format!(
        "{{\"command\":\"push\",\"article_path\":\"{}\",\"media_id\":\"{}\",\"stage\":\"{}\",\"next_step\":\"{}\"}}",
        escape_json(&article_path.display().to_string()),
        escape_json(media_id),
        escape_json(stage),
        escape_json(next_step)
    )
}

pub(crate) struct PushJsonMeta<'a> {
    pub media_id: &'a str,
    pub stage: &'a str,
    pub next_step: &'a str,
}

impl<'a> From<&'a PushOutput> for PushJsonMeta<'a> {
    fn from(output: &'a PushOutput) -> Self {
        let next_step = output
            .message
            .lines()
            .last()
            .unwrap_or_default()
            .trim()
            .strip_prefix("next: ")
            .unwrap_or("check in WeChat backend, then publish manually");
        Self {
            media_id: &output.media_id,
            stage: output.stage,
            next_step,
        }
    }
}

pub(crate) fn draft_from_inbox_json(
    input_path: &Path,
    draft_path: &Path,
    html_path: Option<&Path>,
    action: &str,
    next_command: &str,
    push: Option<PushJsonMeta<'_>>,
) -> String {
    let html = html_path
        .map(|path| format!("\"{}\"", escape_json(&path.display().to_string())))
        .unwrap_or_else(|| "null".to_owned());
    let push_fields = push.map_or_else(String::new, |push| {
        format!(
            ",\"pushed\":true,\"media_id\":\"{}\",\"stage\":\"{}\",\"next_step\":\"{}\"",
            escape_json(push.media_id),
            escape_json(push.stage),
            escape_json(push.next_step)
        )
    });
    format!(
        "{{\"command\":\"draft-from-inbox\",\"input_path\":\"{}\",\"draft_path\":\"{}\",\"html_path\":{},\"action\":\"{}\",\"next_command\":\"{}\"{}}}",
        escape_json(&input_path.display().to_string()),
        escape_json(&draft_path.display().to_string()),
        html,
        escape_json(action),
        escape_json(next_command),
        push_fields
    )
}

pub(crate) fn intake_draft_preview_json(
    command_name: &str,
    inbox_path: &Path,
    draft_path: &Path,
    html_path: Option<&Path>,
    action: &str,
    next_command: &str,
    push: Option<PushJsonMeta<'_>>,
) -> String {
    let html = html_path
        .map(|path| format!("\"{}\"", escape_json(&path.display().to_string())))
        .unwrap_or_else(|| "null".to_owned());
    let push_fields = push.map_or_else(String::new, |push| {
        format!(
            ",\"pushed\":true,\"media_id\":\"{}\",\"stage\":\"{}\",\"next_step\":\"{}\"",
            escape_json(push.media_id),
            escape_json(push.stage),
            escape_json(push.next_step)
        )
    });
    format!(
        "{{\"command\":\"{}\",\"inbox_path\":\"{}\",\"draft_path\":\"{}\",\"html_path\":{},\"action\":\"{}\",\"next_command\":\"{}\"{}}}",
        escape_json(command_name),
        escape_json(&inbox_path.display().to_string()),
        escape_json(&draft_path.display().to_string()),
        html,
        escape_json(action),
        escape_json(next_command),
        push_fields
    )
}

#[cfg(test)]
mod tests {
    use crate::bundle::ArticleBundle;
    use crate::preflight::{PreflightCheck, PreflightReport};
    use crate::test_helpers::{create_file, temp_root};
    use std::path::PathBuf;

    #[test]
    fn preflight_json_reports_checks_and_next_step() {
        let report = PreflightReport {
            article_path: PathBuf::from("Articles/drafts/demo.md"),
            html_path: PathBuf::from("Articles/drafts/demo.html"),
            draft_json_path: PathBuf::from("Articles/drafts/demo.draft.json"),
            media_id_path: PathBuf::from("Articles/drafts/demo.media_id"),
            passed: true,
            checks: vec![
                PreflightCheck {
                    id: "html",
                    status: "pass",
                    message: "rendered HTML exists".to_owned(),
                },
                PreflightCheck {
                    id: "media_id",
                    status: "warn",
                    message: "not pushed yet".to_owned(),
                },
            ],
            next_command: "moonpub push Articles/drafts/demo.md --render".to_owned(),
            next_step: "review local preview, then explicitly push to WeChat draft when ready",
        };

        let output = super::preflight_json(&report);

        assert!(output.contains(r#""command":"preflight""#), "{output}");
        assert!(output.contains(r#""passed":true"#), "{output}");
        assert!(
            output.contains(r#""checks":[{"id":"html","status":"pass""#),
            "{output}"
        );
        assert!(
            output.contains(r#""next_command":"moonpub push Articles/drafts/demo.md --render""#),
            "{output}"
        );
    }

    #[test]
    fn draft_from_inbox_json_builder_includes_paths_and_next_command() {
        let input = std::path::Path::new("Inbox/Feishu/demo.md");
        let draft = std::path::Path::new("Articles/drafts/demo.md");
        let html = std::path::Path::new("Articles/drafts/demo.html");

        let output = super::draft_from_inbox_json(
            input,
            draft,
            Some(html),
            "created",
            "moonpub push Articles/drafts/demo.md --render",
            None,
        );

        assert!(
            output.contains(r#""command":"draft-from-inbox""#),
            "{output}"
        );
        assert!(
            output.contains(r#""input_path":"Inbox/Feishu/demo.md""#),
            "{output}"
        );
        assert!(
            output.contains(r#""draft_path":"Articles/drafts/demo.md""#),
            "{output}"
        );
        assert!(
            output.contains(r#""html_path":"Articles/drafts/demo.html""#),
            "{output}"
        );
        assert!(output.contains(r#""action":"created""#), "{output}");
        assert!(
            output.contains(r#""next_command":"moonpub push Articles/drafts/demo.md --render""#),
            "{output}"
        );
    }

    #[test]
    fn intake_draft_preview_json_builder_includes_paths_and_next_command() {
        let inbox = std::path::Path::new("Inbox/Feishu/demo.md");
        let draft = std::path::Path::new("Articles/drafts/demo.md");
        let html = std::path::Path::new("Articles/drafts/demo.html");

        let output = super::intake_draft_preview_json(
            "intake-feishu",
            inbox,
            draft,
            Some(html),
            "updated",
            "moonpub push Articles/drafts/demo.md --render",
            None,
        );

        assert!(output.contains(r#""command":"intake-feishu""#), "{output}");
        assert!(
            output.contains(r#""inbox_path":"Inbox/Feishu/demo.md""#),
            "{output}"
        );
        assert!(
            output.contains(r#""draft_path":"Articles/drafts/demo.md""#),
            "{output}"
        );
        assert!(
            output.contains(r#""html_path":"Articles/drafts/demo.html""#),
            "{output}"
        );
        assert!(output.contains(r#""action":"updated""#), "{output}");
        assert!(
            output.contains(r#""next_command":"moonpub push Articles/drafts/demo.md --render""#),
            "{output}"
        );
    }

    #[test]
    fn intake_draft_preview_json_builder_supports_photos_command_name() {
        let inbox = std::path::Path::new("Inbox/Photos/day1.md");
        let draft = std::path::Path::new("Articles/drafts/day1.md");

        let output = super::intake_draft_preview_json(
            "intake-photos",
            inbox,
            draft,
            None,
            "created",
            "moonpub push Articles/drafts/day1.md --render",
            None,
        );

        assert!(output.contains(r#""command":"intake-photos""#), "{output}");
        assert!(
            output.contains(r#""inbox_path":"Inbox/Photos/day1.md""#),
            "{output}"
        );
    }

    #[test]
    fn draft_from_inbox_json_builder_includes_push_metadata_when_present() {
        let input = std::path::Path::new("Inbox/Feishu/demo.md");
        let draft = std::path::Path::new("Articles/drafts/demo.md");

        let output = super::draft_from_inbox_json(
            input,
            draft,
            None,
            "updated",
            "moonpub push Articles/drafts/demo.md --render",
            Some(super::PushJsonMeta {
                media_id: "123",
                stage: "ready",
                next_step: "check in WeChat backend, then publish manually",
            }),
        );

        assert!(output.contains(r#""action":"updated""#), "{output}");
        assert!(output.contains(r#""pushed":true"#), "{output}");
        assert!(output.contains(r#""media_id":"123""#), "{output}");
        assert!(output.contains(r#""stage":"ready""#), "{output}");
        assert!(
            output.contains(r#""next_step":"check in WeChat backend, then publish manually""#),
            "{output}"
        );
    }

    #[test]
    fn check_json_reports_bundle_paths_and_next_step() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("protocol-check-json")?;
        let article = root.join("Articles/drafts/demo.md");
        create_file(&article, "---\ntitle: Demo\ntheme: geek-black\n---\n正文\n")?;
        create_file(&root.join("Articles/drafts/demo.html"), "<p>正文</p>")?;

        let bundle = ArticleBundle::from_markdown(&article)?;
        let output = super::check_json(&bundle, Some("blueprint"));

        assert!(output.contains(r#""command":"check""#), "{output}");
        assert!(output.contains(r#""has_markdown":true"#), "{output}");
        assert!(output.contains(r#""has_html":true"#), "{output}");
        assert!(output.contains(r#""has_draft_json":false"#), "{output}");
        assert!(output.contains(r#""theme":"geek-black""#), "{output}");
        assert!(
            output.contains(r#""effective_theme":"geek-black""#),
            "{output}"
        );
        assert!(
            output.contains(r#""theme_source":"article_frontmatter""#),
            "{output}"
        );
        assert!(
            output.contains(r#""next_command":"moonpub render "#),
            "{output}"
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn check_json_reports_config_theme_when_article_theme_missing()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("protocol-check-json-config-theme")?;
        let article = root.join("Articles/drafts/demo.md");
        create_file(&article, "---\ntitle: Demo\n---\n正文\n")?;

        let bundle = ArticleBundle::from_markdown(&article)?;
        let output = super::check_json(&bundle, Some("blueprint"));

        assert!(output.contains(r#""theme":null"#), "{output}");
        assert!(
            output.contains(r#""effective_theme":"blueprint""#),
            "{output}"
        );
        assert!(
            output.contains(r#""theme_source":"wechat_config""#),
            "{output}"
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn check_json_reports_default_theme_when_no_theme_is_set()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("protocol-check-json-default-theme")?;
        let article = root.join("Articles/drafts/demo.md");
        create_file(&article, "---\ntitle: Demo\n---\n正文\n")?;

        let bundle = ArticleBundle::from_markdown(&article)?;
        let output = super::check_json(&bundle, None);

        assert!(output.contains(r#""theme":null"#), "{output}");
        assert!(
            output.contains(r#""effective_theme":"default""#),
            "{output}"
        );
        assert!(output.contains(r#""theme_source":"default""#), "{output}");

        std::fs::remove_dir_all(root)?;
        Ok(())
    }
}
