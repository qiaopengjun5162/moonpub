use std::fs;

use crate::article::{article_slug, parse_frontmatter, resolve_article_path};
use crate::cli::{Command, Options};
use crate::config::Config;
use crate::cover;
use crate::draft::{new_article, write_article_file};
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
            add_status(&options.articles, &slug, "published", "published")
        }
        Command::Export { article } => {
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
        Command::Write { idea } => {
            let api_key = crate::ai::default_api_key()?;
            let article = crate::ai::generate_article(idea, &api_key)?;
            let path = write_article_file(&options.articles, idea, &article)?;
            Ok(format!("generated\n  {}", path.display()))
        }
        Command::Polish { article } => {
            let api_key = crate::ai::default_api_key()?;
            let art_path = resolve_article_path(&options.articles, article);
            let content = fs::read_to_string(&art_path).map_err(|source| AppError::Io {
                path: art_path.clone(),
                source,
            })?;
            let polished = crate::ai::polish_article(&content, &api_key)?;
            fs::write(&art_path, &polished).map_err(|source| AppError::Io {
                path: art_path.clone(),
                source,
            })?;
            Ok(format!("polished\n  {}", art_path.display()))
        }
        Command::Expand { article } => {
            let api_key = crate::ai::default_api_key()?;
            let art_path = resolve_article_path(&options.articles, article);
            let content = fs::read_to_string(&art_path).map_err(|source| AppError::Io {
                path: art_path.clone(),
                source,
            })?;
            // Extract original frontmatter to prepend after expand
            let front = if content.starts_with("---") {
                content
                    .lines()
                    .skip(1)
                    .take_while(|l| l.trim() != "---")
                    .map(|l| format!("{l}\n"))
                    .collect::<String>()
            } else {
                String::new()
            };
            let expanded = crate::ai::expand_notes(&content, &api_key)?;
            // Reconstruct: original frontmatter + AI-generated body
            let output = if front.is_empty() {
                expanded
            } else {
                format!("---\n{front}---\n\n{expanded}")
            };
            fs::write(&art_path, &output).map_err(|source| AppError::Io {
                path: art_path.clone(),
                source,
            })?;
            Ok(format!("expanded\n  {}", art_path.display()))
        }
        Command::ShipAi { article, style } => {
            let api_key = crate::ai::default_api_key()?;
            let art_path = resolve_article_path(&options.articles, article);
            let content = fs::read_to_string(&art_path).map_err(|source| AppError::Io {
                path: art_path.clone(),
                source,
            })?;
            let polished = crate::ai::polish_article(&content, &api_key)?;
            fs::write(&art_path, &polished).map_err(|source| AppError::Io {
                path: art_path.clone(),
                source,
            })?;
            ship_article(
                &options.articles,
                options.config.as_deref(),
                &art_path,
                style.as_deref(),
            )
        }
        Command::Radar(command) => run_radar(&options.articles, command),
        Command::Version => Ok(format!("moonpub {}", env!("CARGO_PKG_VERSION"))),
        Command::Help => Ok(crate::error::help_text()),
    }?;

    if options.json {
        Ok(to_json_string(&raw))
    } else {
        Ok(raw)
    }
}

/// Wrap a plain-text output string into a single-field JSON object.
fn to_json_string(text: &str) -> String {
    format!("{{\"output\":\"{}\"}}", escape_json(text))
}
