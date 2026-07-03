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
use crate::export::export_article;
use crate::init::init_config;
use crate::intake::intake_photos;
use crate::protocol::{check_json, status_json, to_json_string, workspace_json, workspace_text};
use crate::push::{delete_draft, list_drafts, update_draft};
use crate::radar::run_radar;
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
        Command::Configure {
            steps,
            headed,
            temporary_profile,
        } => {
            let cfg = load_config(options)?;
            crate::publish::auto_configure(
                "",
                cfg.wechat_collection.as_deref().unwrap_or("书"),
                steps,
                *headed,
                *temporary_profile,
                cfg.template_name.as_deref(),
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
        } => run_publish_automation(*headed, *temporary_profile, crate::publish::test_yulan),
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
            draft,
            preview,
            auto_push,
        } => {
            let output = intake_photos(&options.articles, inputs)?;
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

    if options.json
        && !matches!(
            options.command,
            Command::Capabilities
                | Command::Workspace
                | Command::Status
                | Command::Check { .. }
                | Command::Preview { .. }
                | Command::Push { .. }
                | Command::DraftFromInbox { .. }
                | Command::IntakeFeishu { draft: true, .. }
                | Command::IntakePhotos { draft: true, .. }
        )
    {
        Ok(to_json_string(&raw))
    } else {
        Ok(raw)
    }
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::path::PathBuf;

    use crate::app::run;
    use crate::cli::{Command, Options};
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
