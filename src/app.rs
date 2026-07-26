use crate::ai_workflow::{
    draft_from_inbox, expand_article, polish_article, ship_ai_article, write_article,
};
use crate::app_article_commands::{
    CoverCommand, RenderCommand, run_cover_command, run_humanize_command, run_preview_command,
    run_render_command,
};
use crate::app_draft_follow_up::{DraftFollowUp, DraftJsonKind, finalize_draft_follow_up};
use crate::app_publish_commands::{
    PushCommand, run_publish_automation, run_publish_command, run_wechat_draft_command,
};
use crate::app_support::{load_config, run_feishu_intake_source};
use crate::article::{article_slug, resolve_article_path};
use crate::bundle::{ArticleStage, move_article_bundle};
use crate::cli::{Command, Options};
use crate::draft::new_article;
use crate::error::AppError;
use crate::evidence::{EvidenceReport, evidence_status};
use crate::export::export_article;
use crate::init::init_config;
use crate::intake::{intake_photos, vision_photo_paths};
use crate::layout_audit::{audit_html_file, layout_audit_text};
use crate::preflight::preflight_article;
use crate::protocol::{
    DoctorReport, check_json, doctor_json, doctor_text, evidence_status_json, evidence_status_text,
    layout_audit_json, layout_recipes_json, layout_recipes_text, preflight_json, preflight_text,
    release_check_json, release_check_text, status_json, to_json_string, workflow_registry_json,
    workflow_registry_text, workspace_json, workspace_text,
};
use crate::push::{delete_draft, list_drafts, update_draft};
use crate::radar::run_radar;
use crate::release_check::{ReleaseCheckReport, release_check};
use crate::ship::ship_article;
use crate::status::{add_status, check_article, check_article_bundle, status, status_report};

pub fn run(options: &Options) -> Result<String, AppError> {
    let raw = match &options.command {
        Command::Init { path } => init_config(path),
        Command::Workspace => {
            let stages = status_report(&options.articles)?;
            if options.json {
                Ok(workspace_json(&stages))
            } else {
                Ok(workspace_text(&stages))
            }
        }
        Command::Doctor => {
            let report = doctor_report(options);
            if options.json {
                Ok(doctor_json(&report))
            } else {
                Ok(doctor_text(&report))
            }
        }
        Command::WorkflowRegistry => {
            if options.json {
                Ok(workflow_registry_json())
            } else {
                Ok(workflow_registry_text())
            }
        }
        Command::EvidenceStatus { strict } => {
            let report = evidence_status()?;
            if *strict {
                ensure_release_evidence_complete(&report)?;
            }
            if options.json {
                Ok(evidence_status_json(&report))
            } else {
                Ok(evidence_status_text(&report))
            }
        }
        Command::ReleaseCheck { strict } => {
            let report = release_check()?;
            if *strict {
                ensure_release_gate_complete(&report)?;
            }
            if options.json {
                Ok(release_check_json(&report))
            } else {
                Ok(release_check_text(&report))
            }
        }
        Command::LayoutRecipes => {
            if options.json {
                Ok(layout_recipes_json())
            } else {
                Ok(layout_recipes_text())
            }
        }
        Command::LayoutAudit { html } => {
            let report = audit_html_file(&resolve_article_path(&options.articles, html))?;
            if options.json {
                Ok(layout_audit_json(&report))
            } else {
                Ok(layout_audit_text(&report))
            }
        }
        Command::Status => {
            if options.json {
                let stages = status_report(&options.articles)?;
                Ok(status_json(&stages))
            } else {
                status(&options.articles)
            }
        }
        Command::Check { article } => {
            if options.json {
                let bundle = check_article_bundle(&options.articles, article)?;
                Ok(check_json(&bundle))
            } else {
                check_article(&options.articles, article)
            }
        }
        Command::Preflight { article } => {
            let report = preflight_article(&options.articles, article)?;
            if options.json {
                Ok(preflight_json(&report))
            } else {
                Ok(preflight_text(&report))
            }
        }
        Command::Render {
            article,
            author,
            thumb_media_id,
            humanize: do_humanize,
        } => {
            let cfg = load_config(options)?;
            run_render_command(
                &options.articles,
                &cfg,
                RenderCommand {
                    article,
                    author: author.as_deref(),
                    thumb_media_id: thumb_media_id.as_deref(),
                    humanize: *do_humanize,
                },
            )
        }
        Command::Cover {
            article,
            style,
            screenshot,
        } => {
            let cfg = load_config(options)?;
            run_cover_command(
                &options.articles,
                &cfg,
                CoverCommand {
                    article,
                    style: style.as_deref(),
                    screenshot: *screenshot,
                },
            )
        }
        Command::Login { temporary_profile } => {
            crate::publish::login(*temporary_profile).map_err(|e| AppError::PushFailed {
                message: e,
                ip_hint: None,
            })
        }
        Command::WechatHealth {
            headed,
            temporary_profile,
        } => crate::publish::health(*headed, *temporary_profile, options.json).map_err(|e| {
            AppError::PushFailed {
                message: e,
                ip_hint: None,
            }
        }),
        Command::Configure {
            steps,
            headed,
            temporary_profile,
            evidence_dir,
        } => {
            let cfg = load_config(options)?;
            crate::publish::auto_configure(
                "",
                cfg.wechat_collection.as_deref().unwrap_or("书"),
                steps,
                *headed,
                *temporary_profile,
                cfg.template_name.as_deref(),
                evidence_dir.as_deref(),
                None,
            )
            .map_err(|e| AppError::PushFailed {
                message: e,
                ip_hint: None,
            })
        }
        Command::StepTest {
            headed,
            temporary_profile,
        } => run_publish_automation(*headed, *temporary_profile, crate::publish::step_test),
        Command::TestZanshang {
            headed,
            temporary_profile,
        } => run_publish_automation(*headed, *temporary_profile, crate::publish::test_zanshang),
        Command::TestYulan {
            headed,
            temporary_profile,
            title,
            to_wxname,
        } => {
            let result = if let Some(title) = title {
                crate::publish::test_yulan_for_title(
                    *headed,
                    *temporary_profile,
                    Some(title),
                    to_wxname.as_deref(),
                )
            } else {
                crate::publish::test_yulan_for_title(
                    *headed,
                    *temporary_profile,
                    None,
                    to_wxname.as_deref(),
                )
                .map_err(|error| error.to_string())
            };
            result.map_err(|message| AppError::PushFailed {
                message,
                ip_hint: None,
            })
        }
        Command::TestChuangzuo {
            headed,
            temporary_profile,
        } => run_publish_automation(*headed, *temporary_profile, crate::publish::test_chuangzuo),
        Command::ListDrafts => {
            let cfg = load_config(options)?;
            list_drafts(&cfg)
        }
        Command::DeleteDraft { media_id } => {
            let cfg = load_config(options)?;
            delete_draft(media_id, &cfg)
        }
        Command::Humanize { article } => run_humanize_command(&options.articles, article),
        Command::Fetch { url } => match crate::fetch::fetch_article(url) {
            Ok(article) => Ok(format!(
                "title:  {}\nauthor: {}\n\n{}",
                article.title, article.author, article.body
            )),
            Err(e) => Ok(format!("fetch failed: {e}")),
        },
        Command::IntakeFeishu {
            source,
            draft,
            preview,
            auto_push,
        } => {
            let output = run_feishu_intake_source(&options.articles, source)?;
            if !draft {
                Ok(output.message)
            } else {
                let cfg = load_config(options)?;
                let draft_output = draft_from_inbox(&options.articles, &cfg, &output.path)?;
                finalize_draft_follow_up(
                    &options.articles,
                    &cfg,
                    DraftFollowUp {
                        preview: *preview,
                        auto_push: *auto_push,
                        json: options.json,
                        leading_message: Some(output.message.as_str()),
                        json_kind: DraftJsonKind::Intake {
                            command_name: "intake-feishu",
                            inbox_path: &output.path,
                        },
                        draft_output: &draft_output,
                    },
                )
            }
        }
        Command::IntakePhotos {
            inputs,
            analyze_images,
            draft,
            preview,
            auto_push,
        } => {
            let mut output = intake_photos(&options.articles, inputs)?;
            if *analyze_images {
                let cfg = load_config(options)?;
                let images = vision_photo_paths(inputs)?;
                crate::ai_workflow::add_photo_vision_to_inbox(&cfg, &output.path, &images)?;
                output
                    .message
                    .push_str("\n  visual analysis: added to Inbox");
            }
            if !draft {
                Ok(output.message)
            } else {
                let cfg = load_config(options)?;
                let draft_output = draft_from_inbox(&options.articles, &cfg, &output.path)?;
                finalize_draft_follow_up(
                    &options.articles,
                    &cfg,
                    DraftFollowUp {
                        preview: *preview,
                        auto_push: *auto_push,
                        json: options.json,
                        leading_message: Some(output.message.as_str()),
                        json_kind: DraftJsonKind::Intake {
                            command_name: "intake-photos",
                            inbox_path: &output.path,
                        },
                        draft_output: &draft_output,
                    },
                )
            }
        }
        Command::Push {
            article,
            auto_render,
            temporary_profile,
        } => {
            let cfg = load_config(options)?;
            run_wechat_draft_command(
                &options.articles,
                &cfg,
                PushCommand {
                    article,
                    auto_render: *auto_render,
                    temporary_profile: *temporary_profile,
                    json: options.json,
                },
            )
        }
        Command::Publish {
            article,
            target,
            auto_render,
            temporary_profile,
        } => {
            let cfg = load_config(options)?;
            run_publish_command(
                &options.articles,
                &cfg,
                target,
                PushCommand {
                    article,
                    auto_render: *auto_render,
                    temporary_profile: *temporary_profile,
                    json: false,
                },
            )
        }
        Command::UpdateDraft { article, media_id } => {
            let cfg = load_config(options)?;
            update_draft(&options.articles, article, media_id.as_deref(), &cfg)
        }
        Command::MarkReady { article } => {
            let slug = article_slug(article)?;
            add_status(&options.articles, &slug, "ready", "confirmed")
        }
        Command::Ship { article, style } => {
            let art_path = resolve_article_path(&options.articles, article);
            ship_article(
                &options.articles,
                options.config.as_deref(),
                &art_path,
                style.as_deref(),
            )
        }
        Command::MarkPublished { article } => {
            let slug = article_slug(article)?;
            let article_path = resolve_article_path(&options.articles, article);
            if let Some(dir) = article_path.parent() {
                let _ = move_article_bundle(dir, &slug, ArticleStage::Published)?;
            }
            add_status(&options.articles, &slug, "published", "published")
        }
        Command::Export { article, target } => {
            if let Some(target) = target
                && target != "zola"
            {
                return Err(AppError::UnknownCommand(format!("export target {target}")));
            }
            let cfg = load_config(options)?;
            let blog_root = cfg
                .blog_root
                .as_deref()
                .ok_or(AppError::MissingValue("blog.root in config"))?;
            export_article(&options.articles, article, blog_root)
        }
        Command::Preview { article, open } => {
            run_preview_command(&options.articles, article, *open, options.json)
        }
        Command::New { title } => new_article(&options.articles, title),
        Command::Write { idea } => {
            let cfg = load_config(options)?;
            write_article(&options.articles, &cfg, idea)
        }
        Command::DraftFromInbox {
            input,
            preview,
            auto_push,
        } => {
            let cfg = load_config(options)?;
            let output = draft_from_inbox(&options.articles, &cfg, input)?;
            let input_path = resolve_article_path(&options.articles, input);
            finalize_draft_follow_up(
                &options.articles,
                &cfg,
                DraftFollowUp {
                    preview: *preview,
                    auto_push: *auto_push,
                    json: options.json,
                    leading_message: None,
                    json_kind: DraftJsonKind::FromInbox {
                        input_path: &input_path,
                    },
                    draft_output: &output,
                },
            )
        }
        Command::Polish { article } => {
            let cfg = load_config(options)?;
            polish_article(&options.articles, &cfg, article)
        }
        Command::Expand { article } => {
            let cfg = load_config(options)?;
            expand_article(&options.articles, &cfg, article)
        }
        Command::ShipAi { article, style } => {
            let cfg = load_config(options)?;
            ship_ai_article(
                &options.articles,
                options.config.as_deref(),
                &cfg,
                article,
                style.as_deref(),
            )
        }
        Command::Radar(command) => run_radar(&options.articles, command),
        Command::Capabilities => {
            if options.json {
                Ok(crate::plugin::capabilities_json())
            } else {
                Ok(crate::plugin::capabilities_text())
            }
        }
        Command::Version => Ok(format!("moonpub {}", env!("CARGO_PKG_VERSION"))),
        Command::Help => Ok(crate::error::help_text()),
    }?;

    if options.json && !options.command.has_structured_json_output() {
        Ok(to_json_string(&raw))
    } else {
        Ok(raw)
    }
}

fn doctor_report(options: &Options) -> DoctorReport {
    let mut warnings = Vec::new();
    let config_path = options
        .config
        .clone()
        .unwrap_or_else(|| options.articles.join("moonpub.toml"));
    let config_status = if config_path.exists() {
        "found"
    } else {
        warnings.push(format!(
            "moonpub.toml not found at {}; run moonpub init or set Articles root",
            config_path.display()
        ));
        "missing"
    };
    if !options.articles.exists() {
        warnings.push(format!(
            "articles root does not exist: {}",
            options.articles.display()
        ));
    }
    if !options.articles.join("Articles").exists() {
        warnings
            .push("Articles/ directory not found; first run can start with moonpub new".to_owned());
    }
    let next_command = if config_status == "missing" {
        format!("moonpub init {}", shell_quote_path(&config_path))
    } else if !options.articles.join("Articles").exists() {
        "moonpub new \"我的第一篇文章\"".to_owned()
    } else {
        "moonpub workspace --json".to_owned()
    };
    let next_step = if config_status == "missing" {
        "create or select the Articles root before using the Obsidian homepage"
    } else if warnings.is_empty() {
        "open the Obsidian MoonPub homepage and choose current article, Feishu, or photos"
    } else {
        "review the local setup warnings before entering the first-run workflow"
    };
    DoctorReport {
        moonpub_version: env!("CARGO_PKG_VERSION"),
        articles_root: options.articles.clone(),
        config_status,
        capabilities_summary: vec![
            "local preview",
            "draft generation",
            "wechat draft push requires explicit action",
        ],
        warnings,
        next_step,
        next_command,
    }
}

fn shell_quote_path(path: &std::path::Path) -> String {
    let value = path.display().to_string();
    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '.' | '_' | '-'))
    {
        value
    } else {
        format!("'{}'", value.replace('\'', "'\\''"))
    }
}

fn ensure_release_evidence_complete(report: &EvidenceReport) -> Result<(), AppError> {
    if report.passed {
        Ok(())
    } else {
        Err(AppError::EvidenceMissing {
            missing_count: report.missing_count,
            next_command: report.next_command,
        })
    }
}

fn ensure_release_gate_complete(report: &ReleaseCheckReport) -> Result<(), AppError> {
    if report.passed {
        Ok(())
    } else {
        let failed_count = report
            .checks
            .iter()
            .filter(|check| check.status == crate::release_check::ReleaseCheckStatus::Fail)
            .count();
        Err(AppError::ReleaseGateIncomplete {
            failed_count,
            next_command: report.next_command.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use crate::app::{ensure_release_evidence_complete, run};
    use crate::cli::{Command, Options};
    use crate::evidence::evidence_status_from;
    use crate::test_helpers::{create_file, temp_root};

    #[test]
    fn mark_published_moves_ready_bundle_to_published() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("mark-published-move")?;
        let ready = root.join("Articles/ready");
        create_file(&ready.join("demo.md"), "# demo")?;
        create_file(&ready.join("demo.html"), "<p>demo</p>")?;
        create_file(&ready.join("demo.draft.json"), "{}")?;
        create_file(&ready.join("demo.media_id"), "media_id")?;

        let output = run(&Options {
            articles: root.clone(),
            command: Command::MarkPublished {
                article: PathBuf::from("Articles/ready/demo.md"),
            },
            json: false,
            config: None,
        })?;

        assert_eq!(output, "demo: published");
        assert!(root.join("Articles/published/demo.md").exists());
        assert!(root.join("Articles/published/demo.html").exists());
        assert!(root.join("Articles/published/demo.draft.json").exists());
        assert!(root.join("Articles/published/demo.media_id").exists());
        assert!(!root.join("Articles/ready/demo.md").exists());

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn capabilities_outputs_text() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("capabilities-text")?;

        let output = run(&Options {
            articles: root.clone(),
            command: Command::Capabilities,
            json: false,
            config: None,
        })?;

        assert!(output.contains("wechat-draft"));
        assert!(output.contains("network: yes"));
        assert!(output.contains("manual final confirmation"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn capabilities_outputs_json_without_wrapping() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("capabilities-json")?;

        let output = run(&Options {
            articles: root.clone(),
            command: Command::Capabilities,
            json: true,
            config: None,
        })?;

        assert!(output.starts_with(r#"{"schema_version":"capabilities/v1","moonpub_version":""#));
        assert!(output.contains(r#""targets":["#));
        assert!(output.contains(r#""id":"wechat-draft""#));
        assert!(!output.contains("{\"output\":"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn subcommand_json_suffix_outputs_json_without_wrapping()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("workspace-json-suffix")?;
        let options = Options::parse([
            "--articles".to_owned(),
            root.display().to_string(),
            "workspace".to_owned(),
            "--json".to_owned(),
        ])?;

        let output = run(&options)?;

        assert!(output.starts_with(r#"{"command":"workspace""#), "{output}");
        assert!(!output.contains("{\"output\":"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn doctor_json_outputs_local_readiness_without_wrapping()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("doctor-json")?;

        let output = run(&Options {
            articles: root.clone(),
            command: Command::Doctor,
            json: true,
            config: None,
        })?;

        assert!(output.starts_with(r#"{"command":"doctor""#), "{output}");
        assert!(output.contains(r#""moonpub_version":"#), "{output}");
        assert!(output.contains(r#""config_status":"missing""#), "{output}");
        assert!(output.contains("moonpub.toml not found"), "{output}");
        assert!(
            output.contains(r#""next_command":"moonpub init "#),
            "{output}"
        );
        assert!(!output.contains("{\"output\":"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn doctor_json_reports_ready_local_setup() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("doctor-ready")?;
        create_file(&root.join("moonpub.toml"), "[articles]\nroot = \".\"\n")?;
        std::fs::create_dir_all(root.join("Articles/drafts"))?;

        let output = run(&Options {
            articles: root.clone(),
            command: Command::Doctor,
            json: true,
            config: Some(root.join("moonpub.toml")),
        })?;

        assert!(output.contains(r#""command":"doctor""#), "{output}");
        assert!(output.contains(r#""config_status":"found""#), "{output}");
        assert!(output.contains(r#""warnings":[]"#), "{output}");
        assert!(
            output.contains(r#""next_command":"moonpub workspace --json""#),
            "{output}"
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn layout_recipes_outputs_text() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("layout-recipes-text")?;

        let output = run(&Options {
            articles: root.clone(),
            command: Command::LayoutRecipes,
            json: false,
            config: None,
        })?;

        assert!(output.contains("layout recipes"));
        assert!(output.contains("生活随笔"));
        assert!(output.contains("photo-grid"));
        assert!(output.contains("docs/LAYOUT_RECIPES_ZH.md"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn layout_recipes_outputs_json_without_wrapping() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("layout-recipes-json")?;

        let output = run(&Options {
            articles: root.clone(),
            command: Command::LayoutRecipes,
            json: true,
            config: None,
        })?;

        assert!(output.starts_with(r#"{"command":"layout-recipes""#));
        assert!(output.contains(r#""id":"life-essay""#));
        assert!(output.contains(r#""themes":["mist","letter","forest"]"#));
        assert!(!output.contains("{\"output\":"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn workflow_registry_outputs_json_without_wrapping() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("workflow-registry-json")?;

        let output = run(&Options {
            articles: root.clone(),
            command: Command::WorkflowRegistry,
            json: true,
            config: None,
        })?;

        assert!(output.starts_with(r#"{"command":"workflow-registry""#));
        assert!(output.contains(r#""id":"feishu-minutes""#));
        assert!(output.contains(r#""safe_start_command":"moonpub wechat-health""#));
        assert!(!output.contains("{\"output\":"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn evidence_status_outputs_json_without_wrapping() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("evidence-status-json")?;

        let output = run(&Options {
            articles: root.clone(),
            command: Command::EvidenceStatus { strict: false },
            json: true,
            config: None,
        })?;

        assert!(output.starts_with(r#"{"command":"evidence-status""#));
        assert!(output.contains(r#""id":"wechat-draft-created""#));
        assert!(output.contains(r#""passed":"#));
        assert!(!output.contains("{\"output\":"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn strict_evidence_status_fails_when_required_files_are_missing()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("evidence-strict-missing")?;
        let report = evidence_status_from(&root);

        let error = ensure_release_evidence_complete(&report).unwrap_err();

        assert!(error.to_string().contains("release evidence incomplete"));
        assert!(error.to_string().contains("11 required file(s) missing"));
        assert!(error.to_string().contains("moonpub evidence-status --json"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn strict_evidence_status_passes_when_required_files_exist()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("evidence-strict-complete")?;
        for path in [
            "homepage/homepage-workspace.png",
            "homepage/homepage-context.png",
            "feishu/feishu-home-entry.png",
            "feishu/feishu-result-modal.png",
            "feishu/feishu-draft-opened.png",
            "photos/photos-image-opened.png",
            "photos/photos-result-modal.png",
            "photos/photos-draft-opened.png",
            "wechat/wechat-draft-created.png",
            "wechat/configure-headed.png",
            "wechat/preview-sent.png",
        ] {
            create_file(&root.join(path), "redacted screenshot placeholder")?;
        }
        let report = evidence_status_from(&root);

        ensure_release_evidence_complete(&report)?;

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn release_check_outputs_json_without_wrapping() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("release-check-json")?;

        let output = run(&Options {
            articles: root.clone(),
            command: Command::ReleaseCheck { strict: false },
            json: true,
            config: None,
        })?;

        assert!(output.starts_with(r#"{"command":"release-check""#));
        assert!(output.contains(r#""id":"release-evidence-files""#));
        assert!(!output.contains("{\"output\":"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn layout_audit_outputs_json_without_wrapping() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("layout-audit-json")?;
        let html = root.join("demo.html");
        create_file(
            &html,
            r#"<section style="margin:0;"><p style="color:#333;">正文</p></section>"#,
        )?;

        let output = run(&Options {
            articles: root.clone(),
            command: Command::LayoutAudit {
                html: html.strip_prefix(&root)?.to_path_buf(),
            },
            json: true,
            config: None,
        })?;

        assert!(output.starts_with(r#"{"command":"layout-audit""#));
        assert!(output.contains(r#""passed":true"#));
        assert!(!output.contains("{\"output\":"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn preflight_outputs_json_without_wrapping() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("preflight-json")?;
        let article = root.join("Articles/drafts/demo.md");
        create_file(&article, "# demo")?;
        create_file(
            &root.join("Articles/drafts/demo.html"),
            r#"<section style="margin:0;"><p style="color:#333;">正文</p></section>"#,
        )?;
        create_file(&root.join("Articles/drafts/demo.draft.json"), "{}")?;

        let output = run(&Options {
            articles: root.clone(),
            command: Command::Preflight {
                article: article.strip_prefix(&root)?.to_path_buf(),
            },
            json: true,
            config: None,
        })?;

        assert!(output.starts_with(r#"{"command":"preflight""#));
        assert!(output.contains(r#""passed":true"#));
        assert!(output.contains(r#""id":"layout_audit","status":"pass""#));
        assert!(output.contains(r#""id":"media_id","status":"warn""#));
        assert!(!output.contains("{\"output\":"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn publish_unknown_target_fails_before_network() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("publish-unknown-target")?;

        let err = run(&Options {
            articles: root.clone(),
            command: Command::Publish {
                article: PathBuf::from("Articles/ready/demo.md"),
                target: "unknown".to_owned(),
                auto_render: false,
                temporary_profile: false,
            },
            json: false,
            config: None,
        })
        .expect_err("unknown target should fail");

        assert!(err.to_string().contains("publish target unknown"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_unknown_target_fails_before_config() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("export-unknown-target")?;

        let err = run(&Options {
            articles: root.clone(),
            command: Command::Export {
                article: PathBuf::from("Articles/published/demo.md"),
                target: Some("unknown".to_owned()),
            },
            json: false,
            config: None,
        })
        .expect_err("unknown target should fail");

        assert!(err.to_string().contains("export target unknown"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn preview_json_includes_paths_and_next_command() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("preview-json")?;
        let drafts = root.join("Articles/drafts");
        let md = drafts.join("demo.md");
        let html = drafts.join("demo.html");
        create_file(&md, "---\ntitle: Demo\n---\n正文\n")?;
        create_file(&html, "<p>正文</p>")?;

        let output = run(&Options {
            articles: root.clone(),
            command: Command::Preview {
                article: PathBuf::from("Articles/drafts/demo.md"),
                open: false,
            },
            json: true,
            config: None,
        })?;

        assert!(output.contains(r#""command":"preview""#), "{output}");
        assert!(
            output.contains(&format!(r#""article_path":"{}""#, md.display())),
            "{output}"
        );
        assert!(
            output.contains(&format!(r#""html_path":"{}""#, html.display())),
            "{output}"
        );
        assert!(
            output.contains(r#""next_command":"moonpub push "#),
            "{output}"
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn push_json_fails_with_no_draft_before_network() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("push-json-no-draft")?;
        let md = root.join("Articles/drafts/demo.md");
        create_file(&md, "---\ntitle: Demo\n---\n正文\n")?;

        let err = run(&Options {
            articles: root.clone(),
            command: Command::Push {
                article: PathBuf::from("Articles/drafts/demo.md"),
                auto_render: false,
                temporary_profile: false,
            },
            json: true,
            config: None,
        })
        .expect_err("push without draft.json should fail before network");

        assert!(
            matches!(err, crate::error::AppError::NoDraftJson(_)),
            "{err}"
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn intake_photos_json_without_draft_still_uses_default_wrapper()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("intake-photos-json-wrapper")?;
        create_file(&root.join("camera/day1/a.jpg"), "fake-jpg")?;

        let output = run(&Options {
            articles: root.clone(),
            command: Command::IntakePhotos {
                inputs: vec![root.join("camera/day1")],
                analyze_images: false,
                draft: false,
                preview: crate::cli::PreviewOptions::default(),
                auto_push: false,
            },
            json: true,
            config: None,
        })?;

        assert!(
            output.starts_with(r#"{"output":"intake created"#),
            "{output}"
        );
        assert!(output.contains("Inbox/Photos/"), "{output}");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn intake_photos_visual_analysis_writes_verified_inbox_notes()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("intake-photos-visual-analysis")?;
        create_file(&root.join("camera/day1/a.jpg"), "fixture-jpg")?;
        let config_path = root.join("moonpub.toml");
        create_file(
            &config_path,
            "[ai]\nprovider = \"openai\"\nmodel = \"gpt-4o\"\napi_key = \"test-key\"\n",
        )?;
        crate::ai::set_test_ai_response(Some("a.jpg：可见一段石阶。"));

        let output = run(&Options {
            articles: root.clone(),
            command: Command::IntakePhotos {
                inputs: vec![root.join("camera/day1")],
                analyze_images: true,
                draft: false,
                preview: crate::cli::PreviewOptions::default(),
                auto_push: false,
            },
            json: false,
            config: Some(config_path),
        })?;
        crate::ai::set_test_ai_response(None);

        assert!(
            output.contains("visual analysis: added to Inbox"),
            "{output}"
        );
        let inbox = std::fs::read_dir(root.join("Inbox/Photos"))?
            .next()
            .expect("photo inbox should contain one file")?
            .path();
        let content = std::fs::read_to_string(inbox)?;
        assert!(content.contains("图像可见信息（AI，需人工核对）"));
        assert!(content.contains("可见一段石阶"));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn intake_photos_draft_preview_json_creates_inbox_draft_and_html()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("intake-photos-draft-preview-json")?;
        create_file(&root.join("camera/day1/a.jpg"), "fake-jpg-a")?;
        create_file(&root.join("camera/day1/b.png"), "fake-png-b")?;
        crate::ai::set_test_ai_response(Some(
            "---\ntitle: Day1\ndigest: 简短记录\ndate: 2026-07-02\ntags: [生活]\n---\n\n:::intro\n今天留一份简单记录。\n:::\n\n照片里的内容先按事实留档。\n\n:::summary\n先记下来，后面再慢慢整理。\n:::",
        ));
        let config_path = root.join("moonpub.toml");
        create_file(
            &config_path,
            "[ai]\nprovider = \"openai\"\nmodel = \"gpt-4o\"\napi_key = \"test-key\"\n",
        )?;

        let output = run(&Options {
            articles: root.clone(),
            command: Command::IntakePhotos {
                inputs: vec![root.join("camera/day1")],
                analyze_images: false,
                draft: true,
                preview: crate::cli::PreviewOptions {
                    enabled: true,
                    open: false,
                },
                auto_push: false,
            },
            json: true,
            config: Some(config_path),
        })?;

        assert!(output.contains(r#""command":"intake-photos""#), "{output}");
        assert!(output.contains(r#""inbox_path":"#), "{output}");
        assert!(output.contains(r#""draft_path":"#), "{output}");
        assert!(output.contains(r#""html_path":"#), "{output}");
        assert!(root.join("Inbox/Photos").exists());
        let payload: serde_json::Value = serde_json::from_str(&output)?;
        let draft_path = payload["draft_path"]
            .as_str()
            .expect("draft_path should exist in intake photos json");
        let html_path = payload["html_path"]
            .as_str()
            .expect("html_path should exist in intake photos json");
        assert!(std::path::Path::new(draft_path).exists(), "{draft_path}");
        assert!(std::path::Path::new(html_path).exists(), "{html_path}");

        crate::ai::set_test_ai_response(None);
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn intake_feishu_draft_preview_json_creates_inbox_draft_and_html()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("intake-feishu-draft-preview-json")?;
        create_file(
            &root.join("exports/minutes.txt"),
            "晨跑后想到的事情\n\n今天跑完步，想先留一份简单记录。",
        )?;
        crate::ai::set_test_ai_response(Some(
            "---\ntitle: 晨跑后想到的事情\ndigest: 一份简短记录\ndate: 2026-07-02\ntags: [生活]\n---\n\n:::intro\n先把今天的念头记下来。\n:::\n\n这是一份基于原始转写整理的短文。\n\n:::summary\n以后再慢慢展开。\n:::",
        ));
        let config_path = root.join("moonpub.toml");
        create_file(
            &config_path,
            "[ai]\nprovider = \"openai\"\nmodel = \"gpt-4o\"\napi_key = \"test-key\"\n",
        )?;

        let output = run(&Options {
            articles: root.clone(),
            command: Command::IntakeFeishu {
                source: crate::cli::FeishuIntakeSource::File(root.join("exports/minutes.txt")),
                draft: true,
                preview: crate::cli::PreviewOptions {
                    enabled: true,
                    open: false,
                },
                auto_push: false,
            },
            json: true,
            config: Some(config_path),
        })?;

        assert!(output.contains(r#""command":"intake-feishu""#), "{output}");
        assert!(output.contains(r#""inbox_path":"#), "{output}");
        assert!(output.contains(r#""draft_path":"#), "{output}");
        assert!(output.contains(r#""html_path":"#), "{output}");
        assert!(root.join("Inbox/Feishu").exists());
        let payload: serde_json::Value = serde_json::from_str(&output)?;
        let draft_path = payload["draft_path"]
            .as_str()
            .expect("draft_path should exist in intake feishu json");
        let html_path = payload["html_path"]
            .as_str()
            .expect("html_path should exist in intake feishu json");
        assert!(std::path::Path::new(draft_path).exists(), "{draft_path}");
        assert!(std::path::Path::new(html_path).exists(), "{html_path}");

        crate::ai::set_test_ai_response(None);
        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn ensure_preview_html_renders_html_before_returning_path()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("ensure-preview-html")?;
        let draft = root.join("Articles/drafts/day1.md");
        create_file(&draft, "---\ntitle: Day1\ndigest: hello\n---\n\n正文\n")?;

        let html_path = crate::app_draft_follow_up::ensure_preview_html(
            &root,
            &crate::config::Config::default(),
            &draft,
            crate::cli::PreviewOptions {
                enabled: true,
                open: false,
            },
        )?
        .expect("html path should exist");

        assert!(html_path.exists(), "{}", html_path.display());
        assert_eq!(html_path, root.join("Articles/drafts/day1.html"));

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
