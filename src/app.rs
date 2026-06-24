use std::fs;
use std::path::Path;

use crate::article::{article_slug, parse_frontmatter, resolve_article_path};
use crate::cli::{Command, Options};
use crate::config::Config;
use crate::cover;
use crate::draft::{new_article, write_article_file};
use crate::error::AppError;
use crate::export::export_article;
use crate::json_util::escape_json;
use crate::preview::preview_article;
use crate::push::{delete_draft, list_drafts, push_article, update_draft};
use crate::radar::run_radar;
use crate::render::render_article;
use crate::status::{add_status, check_article, status};
use crate::system::find_chrome;
use crate::wechat::WechatClient;

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
            let s = match style.as_deref() {
                Some("dark") => cover::CoverStyle::Dark,
                Some("clean") => cover::CoverStyle::Clean,
                Some("minimal") => cover::CoverStyle::Minimal,
                Some("warm") => cover::CoverStyle::Warm,
                Some("serif") => cover::CoverStyle::Serif,
                Some("gradient") => cover::CoverStyle::Gradient,
                Some("literary") => cover::CoverStyle::Literary,
                Some("ink") => cover::CoverStyle::Ink,
                Some("sunset") => cover::CoverStyle::Sunset,
                Some("forest") => cover::CoverStyle::Forest,
                _ => cover::CoverStyle::Literary,
            };
            let html = cover::generate_cover_html(title, digest, author, s);
            let slug = article_path
                .file_stem()
                .and_then(|s| s.to_str())
                .unwrap_or("cover");
            let dir = article_path.parent().unwrap_or(&article_path);
            let out = dir.join(format!("{slug}.cover.html"));
            fs::write(&out, &html).map_err(|source| AppError::Io {
                path: out.clone(),
                source,
            })?;
            let mut result = format!("cover generated\n  {}", out.display());
            if *screenshot {
                let png = dir.join(format!("{slug}.cover.png"));
                let abs_html = std::fs::canonicalize(&out).unwrap_or_else(|e| {
                    eprintln!(
                        "moonpub: cannot resolve absolute path for {}: {e}",
                        out.display()
                    );
                    out.clone()
                });
                let chrome = find_chrome();
                match chrome {
                    Some(bin) => {
                        let status = std::process::Command::new(&bin)
                            .args([
                                "--headless",
                                "--disable-gpu",
                                "--no-sandbox",
                                "--window-size=900,500",
                                &format!("--screenshot={}", png.display()),
                                &format!("file://{}", abs_html.display()),
                            ])
                            .output();
                        if png.exists() {
                            result.push_str(&format!("\n  png:   {}", png.display()));
                        } else {
                            let err = status
                                .err()
                                .map(|e| e.to_string())
                                .unwrap_or_else(|| "unknown error".to_owned());
                            result.push_str(&format!("\n  (screenshot failed: {err})"));
                        }
                    }
                    None => {
                        result.push_str("\n  (screenshot skipped: Chrome/Chromium not found)");
                    }
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
            ship_article(options, &art_path, style.as_deref())
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
            ship_article(options, &art_path, style.as_deref())
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

fn ship_article(
    options: &Options,
    art_path: &Path,
    style: Option<&str>,
) -> Result<String, AppError> {
    let articles_dir = &options.articles;
    let slug = art_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
    let dir = art_path.parent().unwrap_or(art_path);

    let mut cfg = options
        .config
        .as_deref()
        .map(Config::load)
        .transpose()?
        .unwrap_or_default();
    let author = cfg.wechat_author.as_deref().unwrap_or("作者").to_owned();

    let mut results = Vec::new();

    // cover
    let front = parse_frontmatter(&fs::read_to_string(art_path).map_err(|e| AppError::Io {
        path: art_path.to_path_buf(),
        source: e,
    })?);
    let cover_style = match style {
        Some("dark") => cover::CoverStyle::Dark,
        Some("minimal") => cover::CoverStyle::Minimal,
        Some("warm") => cover::CoverStyle::Warm,
        Some("serif") => cover::CoverStyle::Serif,
        Some("gradient") => cover::CoverStyle::Gradient,
        Some("literary") => cover::CoverStyle::Literary,
        Some("ink") => cover::CoverStyle::Ink,
        Some("sunset") => cover::CoverStyle::Sunset,
        Some("forest") => cover::CoverStyle::Forest,
        _ => cover::CoverStyle::Literary,
    };
    let cover_html = cover::generate_cover_html(
        front.title.as_deref().unwrap_or(""),
        front.digest.as_deref().unwrap_or(""),
        front.author.as_deref().unwrap_or(&author),
        cover_style,
    );
    let cover_path = dir.join(format!("{slug}.cover.html"));
    fs::write(&cover_path, &cover_html).map_err(|e| AppError::Io {
        path: cover_path.clone(),
        source: e,
    })?;
    results.push(format!("cover:  {}", cover_path.display()));

    // Screenshot cover → upload to WeChat
    let cover_png = dir.join(format!("{slug}.cover.png"));
    if let Some(bin) = find_chrome() {
        let abs_html = std::fs::canonicalize(&cover_path).unwrap_or(cover_path.clone());
        let _ = std::process::Command::new(&bin)
            .args([
                "--headless",
                "--disable-gpu",
                "--no-sandbox",
                "--window-size=900,500",
                &format!("--screenshot={}", cover_png.display()),
                &format!("file://{}", abs_html.display()),
            ])
            .output();
        if cover_png.exists() {
            let appid = std::env::var("WECHAT_APPID")
                .ok()
                .or_else(|| cfg.wechat_appid.clone())
                .ok_or(AppError::MissingEnvVar("WECHAT_APPID"))?;
            let secret = std::env::var("WECHAT_SECRET")
                .map_err(|_| AppError::MissingEnvVar("WECHAT_SECRET"))?;
            let client = WechatClient::new(&appid, &secret);
            let token = client.access_token()?;
            match client.upload_image(&token, &cover_png) {
                Ok(media_id) => {
                    results.push(format!("thumb:  {media_id}"));
                    cfg.wechat_thumb_media_id = Some(media_id);
                }
                Err(e) => {
                    results.push(format!("⚠ cover upload failed: {e}"));
                }
            }
        }
    }

    let thumb = cfg
        .wechat_thumb_media_id
        .as_deref()
        .unwrap_or("")
        .to_owned();
    let mut footer_cfg = cfg.footer.clone();
    if footer_cfg.qrcode.is_empty() {
        footer_cfg.qrcode = cfg.qrcode_path.clone().unwrap_or_default();
    }
    results.push(render_article(
        articles_dir,
        art_path,
        &author,
        &thumb,
        cfg.wechat_theme.as_deref().unwrap_or("default"),
        Some(&cover_html),
        &footer_cfg,
    )?);
    results.push(push_article(articles_dir, art_path, false, &cfg)?);
    // push_article handles configure internally

    // export
    let pub_path = articles_dir
        .join("Articles/published")
        .join(format!("{slug}.md"));
    let src = if pub_path.exists() {
        &pub_path
    } else {
        art_path
    };
    if let Some(br) = cfg.blog_root.as_deref() {
        results.push(export_article(articles_dir, src, br)?);
    }
    Ok(results.join("\n\n"))
}

/// Wrap a plain-text output string into a single-field JSON object.
fn to_json_string(text: &str) -> String {
    format!("{{\"output\":\"{}\"}}", escape_json(text))
}

// ── init ─────────────────────────────────────────────────────────────────────

use std::io::{self, BufRead, IsTerminal, Write};

fn prompt(question: &str, default: &str) -> String {
    if default.is_empty() {
        print!("{} ", question);
    } else {
        print!("{} [{}] ", question, default);
    }
    io::stdout().flush().ok();
    let mut line = String::new();
    io::stdin().lock().read_line(&mut line).ok();
    let trimmed = line.trim().to_owned();
    if trimmed.is_empty() {
        default.to_owned()
    } else {
        trimmed
    }
}

pub fn init_config(path: &std::path::Path) -> Result<String, AppError> {
    if path.exists() {
        return Err(AppError::ConfigExists(path.to_path_buf()));
    }

    let is_tty = io::stdin().is_terminal();

    if !is_tty {
        let articles_root =
            std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("."));
        let config = crate::config::sample_config_for_articles_root(&articles_root);
        fs::write(path, config).map_err(|source| AppError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        return Ok(format!("created {}", path.display()));
    }

    println!("\n  MoonPub 配置向导\n");
    println!("  按回车使用默认值，Ctrl+C 退出\n");

    // Articles root
    let root_default = std::env::current_dir()
        .map(|d| d.display().to_string())
        .unwrap_or_default();
    let articles_root = prompt("  文章根目录（存放文章的文件夹）:", &root_default);

    // WeChat config
    println!();
    let appid = prompt("  公众号 AppID（wx 开头）:", "");
    let secret = prompt("  公众号 AppSecret:", "");

    let author = prompt("  公众号作者名:", "");

    // Theme
    println!("\n  选择主题风格:");
    println!("    [1] default   — 白底简洁");
    println!("    [2] warm      — 暖色阅读风");
    println!("    [3] dark      — 深蓝黑");
    println!("    [4] geek      — GitHub 风格（推荐）");
    let theme_choice = prompt("  选择 [1-4]:", "4");
    let theme = match theme_choice.as_str() {
        "1" => "default",
        "2" => "warm",
        "3" => "dark",
        _ => "geek",
    };

    // Footer
    println!();
    let want_footer = prompt(
        "  需要文章结尾模板吗？（社群二维码、关注引导等）[y/N]:",
        "N",
    );
    let footer_enabled = want_footer.to_lowercase().starts_with('y');

    let mut footer_title = String::new();
    let mut footer_description = String::new();
    let mut footer_rules = String::new();
    let mut footer_qrcode = String::new();
    let mut footer_qrcode_note = String::new();
    let mut footer_follow_image = String::new();
    let mut footer_follow_text = String::new();
    let mut footer_divider = String::new();

    if footer_enabled {
        println!("\n  --- 结尾模板配置 ---");
        footer_title = prompt("  社群名称（如「我的社群」）:", "");
        footer_description = prompt("  社群描述（一行话）:", "");
        footer_rules = prompt("  群规（用 \\n 分隔多条规则）:", "");
        footer_qrcode = prompt("  群二维码图片路径:", "");
        footer_qrcode_note = prompt("  二维码说明文字:", "");
        footer_follow_image = prompt("  关注引导图片 URL:", "");
        footer_follow_text = prompt("  结尾文案:", "");
        footer_divider = prompt("  分隔符:", "— · —");
    }

    // Blog
    println!();
    let has_blog = prompt("  有博客需要同步导出吗？[y/N]:", "N");
    let blog_enabled = has_blog.to_lowercase().starts_with('y');
    let mut blog_kind = String::new();
    let mut blog_root = String::new();
    if blog_enabled {
        blog_kind = prompt("  博客类型 [zola]:", "zola");
        blog_root = prompt("  博客根目录:", "");
    }

    // Build toml
    let mut toml = format!(
        "[articles]\nroot = \"{articles_root}\"\n\n\
         [wechat]\nappid = \"{appid}\"\n\
         author = \"{author}\"\n\
         account_type = \"personal\"\n\
         auto_publish = false\n\
         theme = \"{theme}\"\n\
         collection = \"\"\n\
         thumb_media_id = \"\"\n\
         author_bio = \"\"\n\
         qrcode = \"\"\n"
    );

    if footer_enabled {
        toml.push_str("\n[footer]\n");
        toml.push_str("enabled = true\n");
        toml.push_str(&format!("title = \"{footer_title}\"\n"));
        toml.push_str(&format!(
            "description = \"{}\"\n",
            footer_description.replace('\n', "\\n")
        ));
        toml.push_str(&format!(
            "rules = \"{}\"\n",
            footer_rules.replace('\n', "\\n")
        ));
        toml.push_str(&format!("qrcode = \"{footer_qrcode}\"\n"));
        toml.push_str(&format!(
            "qrcode_note = \"{}\"\n",
            footer_qrcode_note.replace('\n', "\\n")
        ));
        toml.push_str(&format!("follow_image = \"{footer_follow_image}\"\n"));
        toml.push_str(&format!("follow_text = \"{footer_follow_text}\"\n"));
        toml.push_str(&format!("divider = \"{footer_divider}\"\n"));
    }

    if blog_enabled {
        toml.push_str(&format!(
            "\n[blog]\nkind = \"{blog_kind}\"\nroot = \"{blog_root}\"\n"
        ));
    }

    // Write moonpub.toml
    fs::write(path, &toml).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    // Write .env if secret provided
    if !secret.is_empty() {
        let env_path = path.parent().unwrap_or(path).join(".env");
        let existing = fs::read_to_string(&env_path).unwrap_or_default();
        let mut env_content = String::new();
        let mut wrote_appid = false;
        let mut wrote_secret = false;

        for line in existing.lines() {
            if line.starts_with("WECHAT_APPID=") {
                env_content.push_str(&format!("WECHAT_APPID={appid}\n"));
                wrote_appid = true;
            } else if line.starts_with("WECHAT_SECRET=") {
                env_content.push_str(&format!("WECHAT_SECRET={secret}\n"));
                wrote_secret = true;
            } else {
                env_content.push_str(line);
                env_content.push('\n');
            }
        }
        if !wrote_appid && !appid.is_empty() {
            env_content.push_str(&format!("WECHAT_APPID={appid}\n"));
        }
        if !wrote_secret {
            env_content.push_str(&format!("WECHAT_SECRET={secret}\n"));
        }
        fs::write(&env_path, env_content).map_err(|source| AppError::Io {
            path: env_path.clone(),
            source,
        })?;
    }

    println!();
    println!("  ✅ 配置已创建: {}", path.display());
    if !secret.is_empty() {
        let env_path = path.parent().unwrap_or(path).join(".env");
        println!("  ✅ 凭证已写入: {}", env_path.display());
    }
    println!();
    println!("  下一步:");
    println!("    moonpub login          # 扫码登录微信");
    println!("    moonpub new \"标题\"     # 创建文章");
    println!("    moonpub ship 文章.md    # 发布");

    Ok(format!("created {}", path.display()))
}
