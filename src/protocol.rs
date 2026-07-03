use std::path::Path;

use crate::bundle::ArticleBundle;
use crate::json_util::escape_json;
use crate::push::PushOutput;
use crate::status::StatusStageReport;

pub(crate) fn to_json_string(text: &str) -> String {
    format!("{{\"output\":\"{}\"}}", escape_json(text))
}

pub(crate) fn preview_json(
    article_path: &Path,
    html_path: &Path,
    open_browser: bool,
    next_command: &str,
) -> String {
    format!(
        "{{\"command\":\"preview\",\"article_path\":\"{}\",\"html_path\":\"{}\",\"opened_browser\":{},\"next_command\":\"{}\"}}",
        escape_json(&article_path.display().to_string()),
        escape_json(&html_path.display().to_string()),
        open_browser,
        escape_json(next_command)
    )
}

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

pub(crate) struct LayoutRecipe {
    pub id: &'static str,
    pub title: &'static str,
    pub best_for: &'static str,
    pub themes: &'static [&'static str],
    pub blocks: &'static [&'static str],
}

pub(crate) const LAYOUT_RECIPES: &[LayoutRecipe] = &[
    LayoutRecipe {
        id: "life-essay",
        title: "生活随笔",
        best_for: "日常、散步、跑步、心绪记录",
        themes: &["mist", "letter", "forest"],
        blocks: &["meta-strip", "intro", "scene-card", "closing-card"],
    },
    LayoutRecipe {
        id: "photo-story",
        title: "照片记录",
        best_for: "同一天多张照片、跑步风景、旅行碎片、生活留档",
        themes: &["gallery", "mist", "warm"],
        blocks: &["intro", "photo-grid", "scene-card"],
    },
    LayoutRecipe {
        id: "book-note",
        title: "读书笔记",
        best_for: "书摘、微信读书导入、阅读后的结构化思考",
        themes: &["paper", "classic", "academic"],
        blocks: &["book-info", "intro", "key-points", "pull-quote"],
    },
    LayoutRecipe {
        id: "tech-post",
        title: "技术文章",
        best_for: "教程、踩坑记录、项目复盘、工程说明",
        themes: &["geek", "notebook", "ocean"],
        blocks: &["intro", "callout", "steps", "summary"],
    },
];

pub(crate) fn layout_recipes_text() -> String {
    let mut output = String::from("layout recipes\n");
    output.push_str("  guide: docs/LAYOUT_RECIPES_ZH.md\n");
    for recipe in LAYOUT_RECIPES {
        output.push_str(&format!(
            "\n  {} ({})\n    best_for: {}\n    themes: {}\n    blocks: {}\n",
            recipe.title,
            recipe.id,
            recipe.best_for,
            recipe.themes.join(" / "),
            recipe.blocks.join(" -> ")
        ));
    }
    output.push_str("\n  tip: 一篇文章通常用 2-4 个视觉块就够了。");
    output
}

pub(crate) fn layout_recipes_json() -> String {
    let recipes = LAYOUT_RECIPES
        .iter()
        .map(|recipe| {
            let themes = recipe
                .themes
                .iter()
                .map(|theme| format!("\"{}\"", escape_json(theme)))
                .collect::<Vec<_>>()
                .join(",");
            let blocks = recipe
                .blocks
                .iter()
                .map(|block| format!("\"{}\"", escape_json(block)))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"id\":\"{}\",\"title\":\"{}\",\"best_for\":\"{}\",\"themes\":[{}],\"blocks\":[{}]}}",
                escape_json(recipe.id),
                escape_json(recipe.title),
                escape_json(recipe.best_for),
                themes,
                blocks
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"command\":\"layout-recipes\",\"guide\":\"docs/LAYOUT_RECIPES_ZH.md\",\"recipes\":[{}]}}",
        recipes
    )
}

pub(crate) fn check_json(bundle: &ArticleBundle) -> String {
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
        "{{\"command\":\"check\",\"article_path\":\"{}\",\"html_path\":\"{}\",\"draft_json_path\":\"{}\",\"media_id_path\":\"{}\",\"has_markdown\":{},\"has_html\":{},\"has_draft_json\":{},\"has_media_id\":{},\"publishable\":{},\"next_command\":\"{}\",\"next_step\":\"{}\"}}",
        escape_json(&bundle.markdown_path().display().to_string()),
        escape_json(&bundle.html_path().display().to_string()),
        escape_json(&bundle.draft_json_path().display().to_string()),
        escape_json(&bundle.media_id_path().display().to_string()),
        bundle.has_markdown(),
        bundle.has_html(),
        bundle.has_draft_json(),
        bundle.has_media_id(),
        bundle.publishable(),
        escape_json(&next_command),
        escape_json(next_step)
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

fn optional_json_string(value: Option<&str>) -> String {
    value
        .map(|text| format!("\"{}\"", escape_json(text)))
        .unwrap_or_else(|| "null".to_owned())
}

#[cfg(test)]
mod tests {
    use crate::bundle::ArticleBundle;
    use crate::status::{StatusFileEntry, StatusStageReport};
    use crate::test_helpers::{create_file, temp_root};

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

    #[test]
    fn layout_recipes_json_lists_recipe_choices() {
        let output = super::layout_recipes_json();

        assert!(output.contains(r#""command":"layout-recipes""#), "{output}");
        assert!(
            output.contains(r#""guide":"docs/LAYOUT_RECIPES_ZH.md""#),
            "{output}"
        );
        assert!(output.contains(r#""id":"photo-story""#), "{output}");
        assert!(
            output.contains(r#""blocks":["intro","photo-grid","scene-card"]"#),
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
        create_file(&article, "---\ntitle: Demo\n---\n正文\n")?;
        create_file(&root.join("Articles/drafts/demo.html"), "<p>正文</p>")?;

        let bundle = ArticleBundle::from_markdown(&article)?;
        let output = super::check_json(&bundle);

        assert!(output.contains(r#""command":"check""#), "{output}");
        assert!(output.contains(r#""has_markdown":true"#), "{output}");
        assert!(output.contains(r#""has_html":true"#), "{output}");
        assert!(output.contains(r#""has_draft_json":false"#), "{output}");
        assert!(
            output.contains(r#""next_command":"moonpub render "#),
            "{output}"
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }
}
