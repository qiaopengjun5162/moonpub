use super::{escape_json, optional_json_string};
use crate::status::StatusStageReport;
pub(crate) fn status_json(stages: &[StatusStageReport]) -> String {
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

    let (next_command, next_step) = if let Some(file) = first_draft {
        (
            format!("moonpub check Articles/drafts/{}", file.file),
            "inspect the first draft article and continue render or push",
        )
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
    };

    let stages_json = stages
        .iter()
        .map(|stage| {
            let files_json = stage
                .files
                .iter()
                .map(|file| {
                    format!(
                        "{{\"file\":\"{}\",\"slug\":\"{}\",\"latest_status\":{},\"latest_detail\":{}}}",
                        escape_json(&file.file),
                        escape_json(&file.slug),
                        optional_json_string(file.latest_status.as_deref()),
                        optional_json_string(file.latest_detail.as_deref())
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"stage\":\"{}\",\"count\":{},\"files\":[{}]}}",
                escape_json(&stage.stage),
                stage.files.len(),
                files_json
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"command\":\"status\",\"stages\":[{}],\"next_command\":\"{}\",\"next_step\":\"{}\"}}",
        stages_json,
        escape_json(&next_command),
        escape_json(next_step)
    )
}

pub(crate) fn workspace_json(stages: &[StatusStageReport]) -> String {
    let (next_command, next_step) = next_workspace_action(stages);
    let total_articles = stages.iter().map(|stage| stage.files.len()).sum::<usize>();
    let stage_counts = stages
        .iter()
        .map(|stage| format!("\"{}\":{}", escape_json(&stage.stage), stage.files.len()))
        .collect::<Vec<_>>()
        .join(",");
    let has_drafts = stages
        .iter()
        .find(|stage| stage.stage == "drafts")
        .is_some_and(|stage| !stage.files.is_empty());
    let has_ready = stages
        .iter()
        .find(|stage| stage.stage == "ready")
        .is_some_and(|stage| !stage.files.is_empty());
    let entry_path = if total_articles == 0 || has_drafts {
        "existing-markdown"
    } else if has_ready {
        "wechat-review"
    } else {
        "published-library"
    };
    let entry_label = match entry_path {
        "existing-markdown" => "existing Markdown article -> local preview -> WeChat draft",
        "wechat-review" => "review ready drafts -> WeChat backend preview-send -> manual publish",
        _ => "published library -> inspect previous bundles or start the next article",
    };
    let stages_json = stages
        .iter()
        .map(|stage| {
            let files_json = stage
                .files
                .iter()
                .map(|file| {
                    format!(
                        "{{\"file\":\"{}\",\"slug\":\"{}\",\"latest_status\":{},\"latest_detail\":{}}}",
                        escape_json(&file.file),
                        escape_json(&file.slug),
                        optional_json_string(file.latest_status.as_deref()),
                        optional_json_string(file.latest_detail.as_deref())
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"stage\":\"{}\",\"count\":{},\"files\":[{}]}}",
                escape_json(&stage.stage),
                stage.files.len(),
                files_json
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    let capabilities = crate::plugin::builtin_capabilities();
    let capabilities_json = capabilities
        .iter()
        .map(|capability| {
            format!(
                "{{\"id\":\"{}\",\"kind\":\"{}\",\"requires_network\":{},\"requires_browser\":{},\"next_step\":\"{}\"}}",
                escape_json(capability.id),
                escape_json(capability.kind),
                capability.requires_network,
                capability.requires_browser,
                escape_json(capability.next_step)
            )
        })
        .collect::<Vec<_>>()
        .join(",");

    format!(
        "{{\"command\":\"workspace\",\"workspace_kind\":\"local-publishing-core\",\"entry_path\":\"{}\",\"entry_path_label\":\"{}\",\"total_articles\":{},\"stage_counts\":{{{}}},\"stages\":[{}],\"capabilities\":[{}],\"next_command\":\"{}\",\"next_step\":\"{}\"}}",
        escape_json(entry_path),
        escape_json(entry_label),
        total_articles,
        stage_counts,
        stages_json,
        capabilities_json,
        escape_json(&next_command),
        escape_json(next_step)
    )
}

pub(crate) fn workspace_text(stages: &[StatusStageReport]) -> String {
    let (next_command, next_step) = next_workspace_action(stages);
    let total_articles = stages.iter().map(|stage| stage.files.len()).sum::<usize>();
    let has_drafts = stages
        .iter()
        .find(|stage| stage.stage == "drafts")
        .is_some_and(|stage| !stage.files.is_empty());
    let has_ready = stages
        .iter()
        .find(|stage| stage.stage == "ready")
        .is_some_and(|stage| !stage.files.is_empty());
    let entry_label = if total_articles == 0 || has_drafts {
        "existing Markdown article -> local preview -> WeChat draft"
    } else if has_ready {
        "review ready drafts -> WeChat backend preview-send -> manual publish"
    } else {
        "published library -> inspect previous bundles or start the next article"
    };
    let mut output = String::new();
    output.push_str("workspace\n");
    output.push_str("  kind: local-publishing-core\n");
    output.push_str(&format!("  entry: {entry_label}\n"));
    output.push_str(&format!("  total_articles: {total_articles}\n"));
    for stage in stages {
        output.push_str(&format!("  {}: {}\n", stage.stage, stage.files.len()));
    }
    output.push_str(&format!("  next: {next_command}\n"));
    output.push_str(&format!("  step: {next_step}"));
    output
}

fn next_workspace_action(stages: &[StatusStageReport]) -> (String, &'static str) {
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
        (
            format!("moonpub check Articles/drafts/{}", file.file),
            "inspect the first draft article and continue render or push",
        )
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

#[cfg(test)]
mod tests {
    use crate::status::{StatusFileEntry, StatusStageReport};

    #[test]
    fn status_json_includes_stage_counts_and_latest_status() {
        let output = super::status_json(&[
            StatusStageReport {
                stage: "drafts".to_owned(),
                files: vec![StatusFileEntry {
                    file: "demo.md".to_owned(),
                    slug: "demo".to_owned(),
                    latest_status: Some("ready".to_owned()),
                    latest_detail: Some("confirmed".to_owned()),
                }],
            },
            StatusStageReport {
                stage: "ready".to_owned(),
                files: Vec::new(),
            },
        ]);

        assert!(output.contains(r#""command":"status""#), "{output}");
        assert!(output.contains(r#""stage":"drafts""#), "{output}");
        assert!(output.contains(r#""count":1"#), "{output}");
        assert!(output.contains(r#""file":"demo.md""#), "{output}");
        assert!(output.contains(r#""latest_status":"ready""#), "{output}");
        assert!(
            output.contains(r#""latest_detail":"confirmed""#),
            "{output}"
        );
        assert!(
            output.contains(r#""next_command":"moonpub check Articles/drafts/demo.md""#),
            "{output}"
        );
        assert!(
            output.contains(
                r#""next_step":"inspect the first draft article and continue render or push""#
            ),
            "{output}"
        );
        assert!(output.contains(r#""stage":"ready""#), "{output}");
        assert!(output.contains(r#""count":0"#), "{output}");
    }

    #[test]
    fn workspace_json_includes_entry_path_and_capabilities() {
        let output = super::workspace_json(&[
            StatusStageReport {
                stage: "drafts".to_owned(),
                files: vec![StatusFileEntry {
                    file: "demo.md".to_owned(),
                    slug: "demo".to_owned(),
                    latest_status: Some("ready".to_owned()),
                    latest_detail: Some("confirmed".to_owned()),
                }],
            },
            StatusStageReport {
                stage: "ready".to_owned(),
                files: Vec::new(),
            },
            StatusStageReport {
                stage: "published".to_owned(),
                files: Vec::new(),
            },
        ]);

        assert!(output.contains(r#""command":"workspace""#), "{output}");
        assert!(
            output.contains(r#""workspace_kind":"local-publishing-core""#),
            "{output}"
        );
        assert!(
            output.contains(r#""entry_path":"existing-markdown""#),
            "{output}"
        );
        assert!(
            output.contains(
                r#""entry_path_label":"existing Markdown article -> local preview -> WeChat draft""#
            ),
            "{output}"
        );
        assert!(output.contains(r#""total_articles":1"#), "{output}");
        assert!(
            output.contains(r#""stage_counts":{"drafts":1,"ready":0,"published":0}"#),
            "{output}"
        );
        assert!(output.contains(r#""id":"wechat-draft""#), "{output}");
        assert!(
            output.contains(r#""next_command":"moonpub check Articles/drafts/demo.md""#),
            "{output}"
        );
    }
}
