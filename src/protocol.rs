use std::path::{Path, PathBuf};

use crate::bundle::ArticleBundle;
use crate::cdp::{WechatHealthReport, WechatHealthStatus};
use crate::evidence::EvidenceReport;
use crate::json_util::escape_json;
use crate::layout_audit::LayoutAuditReport;
use crate::preflight::PreflightReport;
use crate::push::PushOutput;
use crate::status::StatusStageReport;

pub(crate) struct DoctorReport {
    pub moonpub_version: &'static str,
    pub articles_root: PathBuf,
    pub config_status: &'static str,
    pub capabilities_summary: Vec<&'static str>,
    pub warnings: Vec<String>,
    pub next_step: &'static str,
    pub next_command: String,
}

pub(crate) fn to_json_string(text: &str) -> String {
    format!("{{\"output\":\"{}\"}}", escape_json(text))
}

pub(crate) fn doctor_text(report: &DoctorReport) -> String {
    let warnings = if report.warnings.is_empty() {
        "none".to_owned()
    } else {
        report.warnings.join("; ")
    };
    format!(
        "doctor\n  moonpub_version: {}\n  articles_root: {}\n  config_status: {}\n  capabilities: {}\n  warnings: {}\n  next: {}\n  next_step: {}",
        report.moonpub_version,
        report.articles_root.display(),
        report.config_status,
        report.capabilities_summary.join(" / "),
        warnings,
        report.next_command,
        report.next_step
    )
}

pub(crate) fn doctor_json(report: &DoctorReport) -> String {
    let capabilities = report
        .capabilities_summary
        .iter()
        .map(|value| format!("\"{}\"", escape_json(value)))
        .collect::<Vec<_>>()
        .join(",");
    let warnings = json_string_array(&report.warnings);
    format!(
        "{{\"command\":\"doctor\",\"moonpub_version\":\"{}\",\"articles_root\":\"{}\",\"config_status\":\"{}\",\"capabilities_summary\":[{}],\"warnings\":{},\"next_step\":\"{}\",\"next_command\":\"{}\"}}",
        escape_json(report.moonpub_version),
        escape_json(&report.articles_root.display().to_string()),
        escape_json(report.config_status),
        capabilities,
        warnings,
        escape_json(report.next_step),
        escape_json(&report.next_command)
    )
}

pub(crate) fn wechat_health_text(report: &WechatHealthReport) -> String {
    let status = match report.status {
        WechatHealthStatus::Ready => "ready",
        WechatHealthStatus::NeedsLogin => "needs_login",
    };
    let session_file = report
        .session_file
        .as_ref()
        .map(|path| path.display().to_string())
        .unwrap_or_else(|| "<temporary profile>".to_owned());
    format!(
        "wechat browser automation health\n  status: {status}\n  profile_mode: {}\n  session_file: {session_file}\n  session_file_exists: {}\n  current_url: {}\n  next: {}\n  next_step: {}",
        report.profile_mode,
        report.session_file_exists,
        report.current_url,
        report.next_command,
        report.next_step
    )
}

pub(crate) fn wechat_health_json(report: &WechatHealthReport) -> String {
    let status = match report.status {
        WechatHealthStatus::Ready => "ready",
        WechatHealthStatus::NeedsLogin => "needs_login",
    };
    let session_file = report
        .session_file
        .as_ref()
        .map(|path| format!("\"{}\"", escape_json(&path.display().to_string())))
        .unwrap_or_else(|| "null".to_owned());
    format!(
        "{{\"command\":\"wechat-health\",\"status\":\"{}\",\"profile_mode\":\"{}\",\"session_file\":{},\"session_file_exists\":{},\"current_url\":\"{}\",\"next_command\":\"{}\",\"next_step\":\"{}\"}}",
        escape_json(status),
        escape_json(report.profile_mode),
        session_file,
        report.session_file_exists,
        escape_json(&report.current_url),
        escape_json(report.next_command),
        escape_json(report.next_step)
    )
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

pub(crate) struct WorkflowRegistryEntry {
    pub id: &'static str,
    pub title: &'static str,
    pub package: &'static str,
    pub status: &'static str,
    pub owner: &'static str,
    pub entry_command: &'static str,
    pub safe_start_command: &'static str,
    pub next_command: &'static str,
    pub requires_network: bool,
    pub requires_browser: bool,
    pub production_boundary: &'static str,
    pub evidence_status: &'static str,
    pub docs: &'static [&'static str],
}

pub(crate) const WORKFLOW_REGISTRY: &[WorkflowRegistryEntry] = &[
    WorkflowRegistryEntry {
        id: "current-article",
        title: "当前 Markdown 文章",
        package: "core/current-article",
        status: "active",
        owner: "moonpub-core",
        entry_command: "moonpub check <article.md>",
        safe_start_command: "moonpub preview <article.md>",
        next_command: "moonpub push <article.md> --render",
        requires_network: false,
        requires_browser: false,
        production_boundary: "local render and preview are safe; push explicitly touches WeChat API",
        evidence_status: "code-and-ci",
        docs: &[
            "docs/FIRST_RUN_WALKTHROUGH_ZH.md",
            "docs/AGENT_PROTOCOL_ZH.md",
        ],
    },
    WorkflowRegistryEntry {
        id: "feishu-minutes",
        title: "飞书妙记到草稿",
        package: "input/feishu-minutes",
        status: "active",
        owner: "moonpub-input",
        entry_command: "moonpub intake feishu --latest --draft --preview --json",
        safe_start_command: "moonpub intake feishu --latest --draft --preview --no-open --json",
        next_command: "moonpub intake feishu --latest --draft --push --json",
        requires_network: true,
        requires_browser: false,
        production_boundary: "reads Feishu Minutes and may call AI for draft generation; push is explicit",
        evidence_status: "cli-verified-needs-plugin-screenshot",
        docs: &[
            "docs/FIRST_RUN_EVIDENCE_CHECKLIST_ZH.md",
            "docs/INPUT_MODEL_ZH.md",
        ],
    },
    WorkflowRegistryEntry {
        id: "photo-memory",
        title: "照片素材到草稿",
        package: "input/photos",
        status: "active",
        owner: "moonpub-input",
        entry_command: "moonpub intake photos <file-or-dir> --draft --preview --json",
        safe_start_command: "moonpub intake photos <file-or-dir> --draft --preview --no-open --json",
        next_command: "moonpub intake photos <file-or-dir> --draft --push --json",
        requires_network: false,
        requires_browser: false,
        production_boundary: "local photo metadata import and AI draft generation; push is explicit",
        evidence_status: "code-and-ci-needs-real-sample",
        docs: &[
            "docs/FIRST_RUN_EVIDENCE_CHECKLIST_ZH.md",
            "docs/INPUT_MODEL_ZH.md",
        ],
    },
    WorkflowRegistryEntry {
        id: "wechat-draft",
        title: "微信公众号草稿推进",
        package: "publish/wechat-draft",
        status: "active",
        owner: "moonpub-publish",
        entry_command: "moonpub push <article.md> --render",
        safe_start_command: "moonpub wechat-health",
        next_command: "moonpub configure --headed",
        requires_network: true,
        requires_browser: true,
        production_boundary: "touches WeChat API and assisted browser automation; final publish remains manual",
        evidence_status: "real-command-verified-needs-redacted-screenshots",
        docs: &[
            "docs/RELEASE_GATE_v0.4.2_ZH.md",
            "docs/WECHAT_REGRESSION_CHECKLIST_ZH.md",
        ],
    },
];

pub(crate) fn workflow_registry_text() -> String {
    let mut output = String::from("workflow registry\n");
    output.push_str("  source: built-in MoonPub workflow contracts\n");
    for workflow in WORKFLOW_REGISTRY {
        output.push_str(&format!(
            "\n  {} ({})\n    package: {}\n    status: {}\n    owner: {}\n    safe_start: {}\n    next: {}\n    network: {}\n    browser: {}\n    boundary: {}\n    evidence: {}\n",
            workflow.title,
            workflow.id,
            workflow.package,
            workflow.status,
            workflow.owner,
            workflow.safe_start_command,
            workflow.next_command,
            workflow.requires_network,
            workflow.requires_browser,
            workflow.production_boundary,
            workflow.evidence_status
        ));
    }
    output.push_str("\n  tip: 先走 safe_start_command，确认后再进入 next_command。");
    output
}

pub(crate) fn workflow_registry_json() -> String {
    let workflows = WORKFLOW_REGISTRY
        .iter()
        .map(|workflow| {
            let docs = workflow
                .docs
                .iter()
                .map(|doc| format!("\"{}\"", escape_json(doc)))
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"id\":\"{}\",\"title\":\"{}\",\"package\":\"{}\",\"status\":\"{}\",\"owner\":\"{}\",\"entry_command\":\"{}\",\"safe_start_command\":\"{}\",\"next_command\":\"{}\",\"requires_network\":{},\"requires_browser\":{},\"production_boundary\":\"{}\",\"evidence_status\":\"{}\",\"docs\":[{}]}}",
                escape_json(workflow.id),
                escape_json(workflow.title),
                escape_json(workflow.package),
                escape_json(workflow.status),
                escape_json(workflow.owner),
                escape_json(workflow.entry_command),
                escape_json(workflow.safe_start_command),
                escape_json(workflow.next_command),
                workflow.requires_network,
                workflow.requires_browser,
                escape_json(workflow.production_boundary),
                escape_json(workflow.evidence_status),
                docs
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"command\":\"workflow-registry\",\"source\":\"built-in\",\"workflows\":[{}]}}",
        workflows
    )
}

pub(crate) fn evidence_status_text(report: &EvidenceReport) -> String {
    let mut output = String::from("evidence status\n");
    output.push_str(&format!("  base_dir: {}\n", report.base_dir.display()));
    output.push_str(&format!("  passed: {}\n", report.passed));
    output.push_str(&format!(
        "  summary: {}/{} present, {} missing\n",
        report.present_count, report.required_count, report.missing_count
    ));
    for section in &report.sections {
        output.push_str(&format!("\n  {} ({})\n", section.title, section.id));
        for item in &section.items {
            output.push_str(&format!(
                "    [{}] {}: {}\n",
                if item.exists { "x" } else { " " },
                item.id,
                item.path.display()
            ));
        }
    }
    if !report.missing_paths.is_empty() {
        output.push_str("\n  missing_paths:\n");
        for path in &report.missing_paths {
            output.push_str(&format!("    - {}\n", path.display()));
        }
    }
    output.push_str(&format!("\n  next: {}\n", report.next_command));
    output.push_str(&format!("  step: {}", report.next_step));
    output
}

pub(crate) fn evidence_status_json(report: &EvidenceReport) -> String {
    let missing_paths = report
        .missing_paths
        .iter()
        .map(|path| format!("\"{}\"", escape_json(&path.display().to_string())))
        .collect::<Vec<_>>()
        .join(",");
    let sections = report
        .sections
        .iter()
        .map(|section| {
            let items = section
                .items
                .iter()
                .map(|item| {
                    format!(
                        "{{\"id\":\"{}\",\"path\":\"{}\",\"exists\":{}}}",
                        escape_json(item.id),
                        escape_json(&item.path.display().to_string()),
                        item.exists
                    )
                })
                .collect::<Vec<_>>()
                .join(",");
            format!(
                "{{\"id\":\"{}\",\"title\":\"{}\",\"items\":[{}]}}",
                escape_json(section.id),
                escape_json(section.title),
                items
            )
        })
        .collect::<Vec<_>>()
        .join(",");
    format!(
        "{{\"command\":\"evidence-status\",\"base_dir\":\"{}\",\"passed\":{},\"required_count\":{},\"present_count\":{},\"missing_count\":{},\"missing_paths\":[{}],\"sections\":[{}],\"next_step\":\"{}\",\"next_command\":\"{}\"}}",
        escape_json(&report.base_dir.display().to_string()),
        report.passed,
        report.required_count,
        report.present_count,
        report.missing_count,
        missing_paths,
        sections,
        escape_json(report.next_step),
        escape_json(report.next_command)
    )
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
        id: "spoken-note",
        title: "口述随记",
        best_for: "飞书妙记、散步录音、随口想法整理成文",
        themes: &["letter", "mist", "notebook"],
        blocks: &[
            "meta-strip",
            "intro",
            "letter-card",
            "summary",
            "closing-card",
        ],
    },
    LayoutRecipe {
        id: "collection-opener",
        title: "合集开篇",
        best_for: "栏目第一篇、付费合集序章、个人小专栏开场",
        themes: &["editorial", "mist", "letter"],
        blocks: &[
            "meta-strip",
            "intro",
            "letter-card",
            "scene-card",
            "closing-card",
        ],
    },
    LayoutRecipe {
        id: "quiet-opening",
        title: "静谧开篇",
        best_for: "闲月隐林、私人合集开场、需要克制边界感的第一篇",
        themes: &["moonlit", "porcelain", "letter"],
        blocks: &[
            "meta-strip",
            "intro",
            "letter-card",
            "scene-card",
            "closing-card",
        ],
    },
    LayoutRecipe {
        id: "photo-story",
        title: "照片记录",
        best_for: "同一天多张照片、跑步风景、旅行碎片、生活留档",
        themes: &["gallery", "mist", "warm"],
        blocks: &["intro", "photo-grid", "scene-card"],
    },
    LayoutRecipe {
        id: "memory-note",
        title: "记忆留档",
        best_for: "同一天照片、散步跑步记录、手机相册里的真实生活片段",
        themes: &["fieldnote", "gallery", "porcelain"],
        blocks: &[
            "meta-strip",
            "intro",
            "photo-grid",
            "scene-card",
            "closing-card",
        ],
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
    LayoutRecipe {
        id: "daily-report",
        title: "日报周报",
        best_for: "AI/Web3 日报、资料索引、可追溯信息流",
        themes: &["notebook", "newsletter", "editorial"],
        blocks: &["intro", "divider", "summary", "callout", "compact-links"],
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

pub(crate) fn layout_audit_json(report: &LayoutAuditReport) -> String {
    let errors = json_string_array(&report.errors);
    let warnings = json_string_array(&report.warnings);
    format!(
        "{{\"command\":\"layout-audit\",\"html_path\":\"{}\",\"passed\":{},\"errors\":{},\"warnings\":{},\"next_step\":\"{}\"}}",
        escape_json(&report.html_path.display().to_string()),
        report.passed,
        errors,
        warnings,
        escape_json(if report.passed {
            "local preview or WeChat draft push"
        } else {
            "remove forbidden tags / attributes before publishing"
        })
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

fn json_string_array(values: &[String]) -> String {
    let items = values
        .iter()
        .map(|value| format!("\"{}\"", escape_json(value)))
        .collect::<Vec<_>>()
        .join(",");
    format!("[{items}]")
}

fn optional_json_string(value: Option<&str>) -> String {
    value
        .map(|text| format!("\"{}\"", escape_json(text)))
        .unwrap_or_else(|| "null".to_owned())
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::bundle::ArticleBundle;
    use crate::cdp::{WechatHealthReport, WechatHealthStatus};
    use crate::preflight::{PreflightCheck, PreflightReport};
    use crate::status::{StatusFileEntry, StatusStageReport};
    use crate::test_helpers::{create_file, temp_root};

    #[test]
    fn wechat_health_text_reports_next_command() {
        let report = WechatHealthReport {
            status: WechatHealthStatus::NeedsLogin,
            profile_mode: "persistent",
            session_file: Some(PathBuf::from("/tmp/session.json")),
            session_file_exists: false,
            current_url: "https://mp.weixin.qq.com/".to_owned(),
            next_command: "moonpub login",
            next_step: "scan the WeChat QR code once, then rerun wechat-health or configure",
        };

        let output = super::wechat_health_text(&report);

        assert!(output.contains("status: needs_login"));
        assert!(output.contains("session_file: /tmp/session.json"));
        assert!(output.contains("next: moonpub login"));
    }

    #[test]
    fn wechat_health_json_reports_ready_status() {
        let report = WechatHealthReport {
            status: WechatHealthStatus::Ready,
            profile_mode: "persistent",
            session_file: Some(PathBuf::from("/tmp/session.json")),
            session_file_exists: true,
            current_url: "https://mp.weixin.qq.com/cgi-bin/home".to_owned(),
            next_command: "moonpub configure --headed",
            next_step: "browser automation login is reusable",
        };

        let output = super::wechat_health_json(&report);

        assert!(output.contains(r#""command":"wechat-health""#));
        assert!(output.contains(r#""status":"ready""#));
        assert!(output.contains(r#""session_file":"/tmp/session.json""#));
        assert!(output.contains(r#""next_command":"moonpub configure --headed""#));
    }

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
        assert!(output.contains(r#""id":"spoken-note""#), "{output}");
        assert!(
            output.contains(
                r#""blocks":["meta-strip","intro","letter-card","summary","closing-card"]"#
            ),
            "{output}"
        );
        assert!(output.contains(r#""id":"collection-opener""#), "{output}");
        assert!(
            output.contains(
                r#""blocks":["meta-strip","intro","letter-card","scene-card","closing-card"]"#
            ),
            "{output}"
        );
        assert!(output.contains(r#""id":"quiet-opening""#), "{output}");
        assert!(
            output.contains(r#""themes":["moonlit","porcelain","letter"]"#),
            "{output}"
        );
        assert!(output.contains(r#""id":"memory-note""#), "{output}");
        assert!(
            output.contains(r#""themes":["fieldnote","gallery","porcelain"]"#),
            "{output}"
        );
        assert!(output.contains(r#""id":"daily-report""#), "{output}");
        assert!(
            output.contains(r#""blocks":["intro","divider","summary","callout","compact-links"]"#),
            "{output}"
        );
    }

    #[test]
    fn workflow_registry_json_lists_first_run_contracts() {
        let output = super::workflow_registry_json();

        assert!(
            output.contains(r#""command":"workflow-registry""#),
            "{output}"
        );
        assert!(output.contains(r#""id":"current-article""#), "{output}");
        assert!(
            output.contains(r#""package":"input/feishu-minutes""#),
            "{output}"
        );
        assert!(
            output.contains(r#""safe_start_command":"moonpub intake photos <file-or-dir> --draft --preview --no-open --json""#),
            "{output}"
        );
        assert!(output.contains(r#""requires_browser":true"#), "{output}");
        assert!(
            output.contains(r#""docs":["docs/RELEASE_GATE_v0.4.2_ZH.md","docs/WECHAT_REGRESSION_CHECKLIST_ZH.md"]"#),
            "{output}"
        );
    }

    #[test]
    fn evidence_status_json_lists_required_release_files() {
        let report = crate::evidence::evidence_status_from(Path::new("docs/first-run-evidence"));

        let output = super::evidence_status_json(&report);

        assert!(
            output.contains(r#""command":"evidence-status""#),
            "{output}"
        );
        assert!(
            output.contains(r#""base_dir":"docs/first-run-evidence""#),
            "{output}"
        );
        assert!(output.contains(r#""required_count":11"#), "{output}");
        assert!(output.contains(r#""present_count":0"#), "{output}");
        assert!(output.contains(r#""missing_count":11"#), "{output}");
        assert!(
            output.contains(
                r#""missing_paths":["docs/first-run-evidence/homepage/homepage-workspace.png""#
            ),
            "{output}"
        );
        assert!(output.contains(r#""id":"homepage-workspace""#), "{output}");
        assert!(output.contains(r#""id":"preview-sent""#), "{output}");
        assert!(
            output.contains(r#""next_command":"moonpub evidence-status --json""#),
            "{output}"
        );
    }

    #[test]
    fn layout_audit_json_reports_errors_warnings_and_next_step() {
        let report = crate::layout_audit::LayoutAuditReport {
            html_path: std::path::PathBuf::from("Articles/drafts/demo.html"),
            passed: false,
            errors: vec!["contains forbidden tag `<div`".to_owned()],
            warnings: vec!["contains full HTML document shell".to_owned()],
        };

        let output = super::layout_audit_json(&report);

        assert!(output.contains(r#""command":"layout-audit""#), "{output}");
        assert!(
            output.contains(r#""html_path":"Articles/drafts/demo.html""#),
            "{output}"
        );
        assert!(output.contains(r#""passed":false"#), "{output}");
        assert!(
            output.contains(r#""errors":["contains forbidden tag `<div`"]"#),
            "{output}"
        );
        assert!(
            output.contains(r#""warnings":["contains full HTML document shell"]"#),
            "{output}"
        );
        assert!(
            output
                .contains(r#""next_step":"remove forbidden tags / attributes before publishing""#),
            "{output}"
        );
    }

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
