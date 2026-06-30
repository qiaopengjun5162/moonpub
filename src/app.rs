use std::fs;

use crate::ai_workflow::{
    draft_from_inbox, expand_article, polish_article, ship_ai_article, write_article,
};
use crate::article::{article_slug, cover_title, parse_frontmatter, resolve_article_path};
use crate::bundle::{ArticleStage, move_article_bundle};
use crate::cli::{Command, FeishuIntakeSource, Options};
use crate::config::Config;
use crate::cover;
use crate::draft::new_article;
use crate::error::AppError;
use crate::export::export_article;
use crate::init::init_config;
use crate::intake::{
    intake_feishu, intake_feishu_latest, intake_feishu_minute_token, intake_feishu_query,
};
use crate::json_util::escape_json;
use crate::preview::{preview_article_with_open, preview_paths};
use crate::push::{delete_draft, list_drafts, push_article, push_article_output, update_draft};
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
            let cfg = options
                .config
                .as_deref()
                .map(Config::load)
                .transpose()?
                .unwrap_or_default();
            let article_path = resolve_article_path(&options.articles, article);
            let md = fs::read_to_string(&article_path).map_err(|source| AppError::Io {
                path: article_path.clone(),
                source,
            })?;
            let front = parse_frontmatter(&md);
            let title = cover_title(&front, &md, &article_path);
            let digest = front.digest.as_deref().unwrap_or("");
            let author = front
                .wechat_author
                .as_deref()
                .or(cfg.wechat_author.as_deref())
                .unwrap_or("");
            let artifact = cover::write_cover_html(
                &article_path,
                &title,
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
                cfg.template_name.as_deref(),
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
        Command::IntakeFeishu {
            source,
            draft,
            preview,
            auto_push,
        } => {
            let output = match source {
                FeishuIntakeSource::File(input) => intake_feishu(&options.articles, input),
                FeishuIntakeSource::MinuteToken(token) => {
                    intake_feishu_minute_token(&options.articles, token)
                }
                FeishuIntakeSource::Latest => intake_feishu_latest(&options.articles),
                FeishuIntakeSource::Query(query) => intake_feishu_query(&options.articles, query),
            }?;
            if !draft {
                Ok(output.message)
            } else {
                let cfg = options
                    .config
                    .as_deref()
                    .map(Config::load)
                    .transpose()?
                    .unwrap_or_default();
                let draft_output = draft_from_inbox(&options.articles, &cfg, &output.path)?;
                let push_output = if *auto_push {
                    Some(push_article_output(
                        &options.articles,
                        &draft_output.path,
                        true,
                        &cfg,
                    )?)
                } else {
                    None
                };
                if options.json {
                    let html_path = if preview.enabled {
                        let (_, html_path) = preview_paths(&options.articles, &draft_output.path)?;
                        Some(html_path)
                    } else {
                        None
                    };
                    let next = push_output
                        .as_ref()
                        .map(|output| output.message.lines().last().unwrap_or_default())
                        .and_then(|line| line.trim().strip_prefix("next: "))
                        .unwrap_or("moonpub push <draft.md> --render");
                    Ok(intake_draft_preview_json(
                        &output.path,
                        &draft_output.path,
                        html_path.as_deref(),
                        output.action.as_str(),
                        next,
                        push_output.as_ref().map(PushJsonMeta::from),
                    ))
                } else {
                    let mut message = format!("{}\n{}", output.message, draft_output.message);
                    if let Some(push_output) = push_output {
                        message.push('\n');
                        message.push_str(&push_output.message);
                    } else if preview.enabled {
                        message.push('\n');
                        message.push_str(&render_and_preview_draft(
                            &options.articles,
                            &cfg,
                            &draft_output.path,
                            preview.open,
                        )?);
                    }
                    Ok(message)
                }
            }
        }
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
            if options.json {
                let article_path = resolve_article_path(&options.articles, article);
                let output = push_article_output(&options.articles, article, *auto_render, &cfg)?;
                let next = "check in WeChat backend, then publish manually";
                Ok(push_json(
                    &article_path,
                    &output.media_id,
                    output.stage,
                    next,
                ))
            } else {
                push_article(&options.articles, article, *auto_render, &cfg)
            }
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
        Command::Preview { article, open } => {
            if options.json {
                let (article_path, html_path) = preview_paths(&options.articles, article)?;
                let next = format!("moonpub push {} --render", article_path.display());
                Ok(preview_json(&article_path, &html_path, *open, &next))
            } else {
                preview_article_with_open(&options.articles, article, *open)
            }
        }
        Command::New { title } => new_article(&options.articles, title),
        Command::Write { idea } => {
            let cfg = options
                .config
                .as_deref()
                .map(Config::load)
                .transpose()?
                .unwrap_or_default();
            write_article(&options.articles, &cfg, idea)
        }
        Command::DraftFromInbox {
            input,
            preview,
            auto_push,
        } => {
            let cfg = options
                .config
                .as_deref()
                .map(Config::load)
                .transpose()?
                .unwrap_or_default();
            let output = draft_from_inbox(&options.articles, &cfg, input)?;
            let push_output = if *auto_push {
                Some(push_article_output(
                    &options.articles,
                    &output.path,
                    true,
                    &cfg,
                )?)
            } else {
                None
            };
            if options.json {
                let input_path = resolve_article_path(&options.articles, input);
                let html_path = if preview.enabled {
                    let (_, html_path) = preview_paths(&options.articles, &output.path)?;
                    Some(html_path)
                } else {
                    None
                };
                let next = push_output
                    .as_ref()
                    .map(|result| result.message.lines().last().unwrap_or_default())
                    .and_then(|line| line.trim().strip_prefix("next: "))
                    .unwrap_or("moonpub push <draft.md> --render");
                Ok(draft_from_inbox_json(
                    &input_path,
                    &output.path,
                    html_path.as_deref(),
                    output.action.as_str(),
                    next,
                    push_output.as_ref().map(PushJsonMeta::from),
                ))
            } else if let Some(push_output) = push_output {
                Ok(format!("{}\n{}", output.message, push_output.message))
            } else if !preview.enabled {
                Ok(output.message)
            } else {
                Ok(format!(
                    "{}\n{}",
                    output.message,
                    render_and_preview_draft(&options.articles, &cfg, &output.path, preview.open)?
                ))
            }
        }
        Command::Polish { article } => {
            let cfg = options
                .config
                .as_deref()
                .map(Config::load)
                .transpose()?
                .unwrap_or_default();
            polish_article(&options.articles, &cfg, article)
        }
        Command::Expand { article } => {
            let cfg = options
                .config
                .as_deref()
                .map(Config::load)
                .transpose()?
                .unwrap_or_default();
            expand_article(&options.articles, &cfg, article)
        }
        Command::ShipAi { article, style } => {
            let cfg = options
                .config
                .as_deref()
                .map(Config::load)
                .transpose()?
                .unwrap_or_default();
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
                | Command::Preview { .. }
                | Command::Push { .. }
                | Command::DraftFromInbox { .. }
                | Command::IntakeFeishu { draft: true, .. }
        )
    {
        Ok(to_json_string(&raw))
    } else {
        Ok(raw)
    }
}

/// Wrap a plain-text output string into a single-field JSON object.
fn to_json_string(text: &str) -> String {
    format!("{{\"output\":\"{}\"}}", escape_json(text))
}

fn preview_json(
    article_path: &std::path::Path,
    html_path: &std::path::Path,
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

fn push_json(
    article_path: &std::path::Path,
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

struct PushJsonMeta<'a> {
    media_id: &'a str,
    stage: &'a str,
    next_step: &'a str,
}

impl<'a> From<&'a crate::push::PushOutput> for PushJsonMeta<'a> {
    fn from(output: &'a crate::push::PushOutput) -> Self {
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

fn draft_from_inbox_json(
    input_path: &std::path::Path,
    draft_path: &std::path::Path,
    html_path: Option<&std::path::Path>,
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

fn intake_draft_preview_json(
    inbox_path: &std::path::Path,
    draft_path: &std::path::Path,
    html_path: Option<&std::path::Path>,
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
        "{{\"command\":\"intake-feishu\",\"inbox_path\":\"{}\",\"draft_path\":\"{}\",\"html_path\":{},\"action\":\"{}\",\"next_command\":\"{}\"{}}}",
        escape_json(&inbox_path.display().to_string()),
        escape_json(&draft_path.display().to_string()),
        html,
        escape_json(action),
        escape_json(next_command),
        push_fields
    )
}

fn render_and_preview_draft(
    articles_dir: &std::path::Path,
    cfg: &Config,
    article: &std::path::Path,
    open_browser: bool,
) -> Result<String, AppError> {
    let resolved_author = cfg.wechat_author.as_deref().unwrap_or("作者");
    let resolved_thumb = cfg.wechat_thumb_media_id.as_deref().unwrap_or("");
    let theme_name = cfg.wechat_theme.as_deref().unwrap_or("default");
    let mut footer_cfg = cfg.footer.clone();
    if footer_cfg.qrcode.is_empty() {
        footer_cfg.qrcode = cfg.qrcode_path.clone().unwrap_or_default();
    }
    let rendered = render_article(
        articles_dir,
        article,
        resolved_author,
        resolved_thumb,
        theme_name,
        None,
        &footer_cfg,
    )?;
    let previewed = preview_article_with_open(articles_dir, article, open_browser)?;
    Ok(format!("{rendered}\n{previewed}"))
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
}
