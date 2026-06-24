use std::fs;

use crate::ai_workflow::{expand_article, polish_article, ship_ai_article, write_article};
use crate::article::{article_slug, parse_frontmatter, resolve_article_path};
use crate::bundle::{ArticleStage, move_article_bundle};
use crate::cli::{Command, Options};
use crate::config::Config;
use crate::cover;
use crate::draft::new_article;
use crate::error::AppError;
use crate::export::export_article;
use crate::init::init_config;
use crate::json_util::escape_json;
use crate::preview::preview_article;
use crate::push::{delete_draft, list_drafts, push_article, update_draft};
use crate::radar::run_radar;
use crate::render::render_article;
use crate::ship::ship_article;
use crate::status::{add_status, check_article, status};

pub fn run(options: &Options) -> Result<String, AppError> {
    let raw = match &options.command {
        Command::Init { path } => init_config(path),
        Command::Status => status(&options.articles),
        Command::Check { article } => check_article(&options.articles, article),
        Command::Render {
            article,
            author,
            thumb_media_id,
            humanize: do_humanize,
        } => {
            let cfg = options
                .config
                .as_deref()
                .map(Config::load)
                .transpose()?
                .unwrap_or_default();
            let resolved_author = author
                .as_deref()
                .or(cfg.wechat_author.as_deref())
                .unwrap_or("作者")
                .to_owned();
            let resolved_thumb = thumb_media_id
                .as_deref()
                .or(cfg.wechat_thumb_media_id.as_deref())
                .unwrap_or("")
                .to_owned();
            if *do_humanize {
                let article_path = resolve_article_path(&options.articles, article);
                let md = fs::read_to_string(&article_path).map_err(|source| AppError::Io {
                    path: article_path.clone(),
                    source,
                })?;
                let processed = crate::humanize::humanize(&md);
                fs::write(&article_path, &processed).map_err(|source| AppError::Io {
                    path: article_path.clone(),
                    source,
                })?;
            }
            let theme_name = cfg.wechat_theme.as_deref().unwrap_or("default");
            let mut footer_cfg = cfg.footer.clone();
            if footer_cfg.qrcode.is_empty() {
                footer_cfg.qrcode = cfg.qrcode_path.clone().unwrap_or_default();
            }
            render_article(
                &options.articles,
                article,
                &resolved_author,
                &resolved_thumb,
                theme_name,
                None,
                &footer_cfg,
            )
        }
        Command::Cover {
            article,
            style,
            screenshot,
        } => {
            let article_path = resolve_article_path(&options.articles, article);
            let md = fs::read_to_string(&article_path).map_err(|source| AppError::Io {
                path: article_path.clone(),
                source,
            })?;
            let front = parse_frontmatter(&md);
            let title = front.title.as_deref().unwrap_or("无标题");
            let digest = front.digest.as_deref().unwrap_or("");
            let author = front.tags.first().map(|s| s.as_str()).unwrap_or("寻月隐君");
            let artifact = cover::write_cover_html(
                &article_path,
                title,
                digest,
                author,
                cover::style_from_name(style.as_deref()),
            )?;
            let mut result = format!("cover generated\n  {}", artifact.html_path.display());
            if *screenshot {
                let png = cover::cover_png_path(&article_path);
                if let Some(message) = cover::capture_cover_png(&artifact.html_path, &png) {
                    result.push_str(&format!("\n  ({message})"));
                } else {
                    result.push_str(&format!("\n  png:   {}", png.display()));
                }
            }
            Ok(result)
        }
        Command::Login => crate::publish::login().map_err(|e| AppError::PushFailed {
            message: e,
            ip_hint: None,
        }),
        Command::Configure { steps, headed } => {
            let cfg = options
                .config
                .as_deref()
                .map(Config::load)
                .transpose()?
                .unwrap_or_default();
            crate::publish::auto_configure(
                "",
                cfg.wechat_collection.as_deref().unwrap_or("书"),
                steps,
                *headed,
            )
            .map_err(|e| AppError::PushFailed {
                message: e,
                ip_hint: None,
            })
        }
        Command::StepTest { headed } => {
            crate::publish::step_test(*headed).map_err(|e| AppError::PushFailed {
                message: e,
                ip_hint: None,
            })
        }
        Command::TestZanshang { headed } => {
            crate::publish::test_zanshang(*headed).map_err(|e| AppError::PushFailed {
                message: e,
                ip_hint: None,
            })
        }
        Command::TestYulan { headed } => {
            crate::publish::test_yulan(*headed).map_err(|e| AppError::PushFailed {
                message: e,
                ip_hint: None,
            })
        }
        Command::TestChuangzuo { headed } => {
            crate::publish::test_chuangzuo(*headed).map_err(|e| AppError::PushFailed {
                message: e,
                ip_hint: None,
            })
        }
        Command::ListDrafts => {
            let cfg = options
                .config
                .as_deref()
                .map(Config::load)
                .transpose()?
                .unwrap_or_default();
            list_drafts(&cfg)
        }
        Command::DeleteDraft { media_id } => {
            let cfg = options
                .config
                .as_deref()
                .map(Config::load)
                .transpose()?
                .unwrap_or_default();
            delete_draft(media_id, &cfg)
        }
        Command::Humanize { article } => {
            let article_path = resolve_article_path(&options.articles, article);
            let md = fs::read_to_string(&article_path).map_err(|source| AppError::Io {
                path: article_path.clone(),
                source,
            })?;
            let processed = crate::humanize::humanize(&md);
            fs::write(&article_path, &processed).map_err(|source| AppError::Io {
                path: article_path.clone(),
                source,
            })?;
            Ok(format!("humanized {}", article_path.display()))
        }
        Command::Fetch { url } => match crate::fetch::fetch_article(url) {
            Ok(article) => Ok(format!(
                "title:  {}\nauthor: {}\n\n{}",
                article.title, article.author, article.body
            )),
            Err(e) => Ok(format!("fetch failed: {e}")),
        },
        Command::Push {
            article,
            auto_render,
        } => {
            let cfg = options
                .config
                .as_deref()
                .map(Config::load)
                .transpose()?
                .unwrap_or_default();
            push_article(&options.articles, article, *auto_render, &cfg)
        }
        Command::Publish {
            article,
            target,
            auto_render,
        } => match target.as_str() {
            "wechat-draft" => {
                let cfg = options
                    .config
                    .as_deref()
                    .map(Config::load)
                    .transpose()?
                    .unwrap_or_default();
                push_article(&options.articles, article, *auto_render, &cfg)
            }
            other => Err(AppError::UnknownCommand(format!("publish target {other}"))),
        },
        Command::UpdateDraft { article, media_id } => {
            let cfg = options
                .config
                .as_deref()
                .map(Config::load)
                .transpose()?
                .unwrap_or_default();
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
            let cfg = options
                .config
                .as_deref()
                .map(Config::load)
                .transpose()?
                .unwrap_or_default();
            let blog_root = cfg
                .blog_root
                .as_deref()
                .ok_or(AppError::MissingValue("blog.root in config"))?;
            export_article(&options.articles, article, blog_root)
        }
        Command::Preview { article } => preview_article(&options.articles, article),
        Command::New { title } => new_article(&options.articles, title),
        Command::Write { idea } => write_article(&options.articles, idea),
        Command::Polish { article } => polish_article(&options.articles, article),
        Command::Expand { article } => expand_article(&options.articles, article),
        Command::ShipAi { article, style } => ship_ai_article(
            &options.articles,
            options.config.as_deref(),
            article,
            style.as_deref(),
        ),
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

    if options.json && !matches!(options.command, Command::Capabilities) {
        Ok(to_json_string(&raw))
    } else {
        Ok(raw)
    }
}

/// Wrap a plain-text output string into a single-field JSON object.
fn to_json_string(text: &str) -> String {
    format!("{{\"output\":\"{}\"}}", escape_json(text))
}

#[cfg(test)]
mod tests {
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
}
