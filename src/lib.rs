use std::fmt::{self, Display};
use std::fs::{self, OpenOptions};
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

mod cover;
mod fetch;
mod footer;
mod humanize;
mod illustrate;
mod publish;
mod radar;
mod theme;
mod wechat;
pub(crate) use radar::*;
pub use wechat::WechatClient;

const DEFAULT_CONFIG: &str = "moonpub.toml";

/// Consume the next argument as a value for a named flag (e.g., "--style dark").
fn flag_value(
    extra: &mut std::slice::Iter<String>,
    name: &'static str,
) -> Result<String, AppError> {
    let v = extra.next().ok_or(AppError::MissingValue(name))?;
    Ok(v.clone())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub vault: PathBuf,
    pub command: Command,
    pub json: bool,
    pub config: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Init {
        path: PathBuf,
    },
    Status,
    Check {
        article: PathBuf,
    },
    Render {
        article: PathBuf,
        author: Option<String>,
        thumb_media_id: Option<String>,
        humanize: bool,
    },
    Push {
        article: PathBuf,
        auto_render: bool,
    },
    UpdateDraft {
        article: PathBuf,
        media_id: Option<String>,
    },
    Export {
        article: PathBuf,
    },
    Preview {
        article: PathBuf,
    },
    MarkReady {
        article: PathBuf,
    },
    MarkPublished {
        article: PathBuf,
    },
    Cover {
        article: PathBuf,
        style: Option<String>,
        screenshot: bool,
    },
    Humanize {
        article: PathBuf,
    },
    Fetch {
        url: String,
    },
    Login,
    Configure,
    StepTest,
    TestZanshang,
    ListDrafts,
    DeleteDraft {
        media_id: String,
    },
    Ship {
        article: PathBuf,
        style: Option<String>,
    },
    Radar(RadarCommand),
    Help,
}

#[derive(Debug)]
pub enum AppError {
    MissingCommand,
    MissingValue(&'static str),
    UnknownOption(String),
    UnknownCommand(String),
    Io {
        path: PathBuf,
        source: io::Error,
    },
    InvalidArticlePath(PathBuf),
    ConfigExists(PathBuf),
    InvalidNumber {
        flag: &'static str,
        value: String,
    },
    InvalidCsv(String),
    MissingEnvVar(&'static str),
    PushFailed {
        message: String,
        ip_hint: Option<String>,
    },
    NoDraftJson(PathBuf),
    NoHtml(PathBuf),
    AutomationFailed {
        message: String,
    },
}

impl Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingCommand => write!(f, "missing command\n\n{}", help_text()),
            Self::MissingValue(flag) => write!(f, "missing value for {flag}"),
            Self::UnknownOption(option) => write!(f, "unknown option: {option}"),
            Self::UnknownCommand(command) => {
                write!(f, "unknown command: {command}\n\n{}", help_text())
            }
            Self::Io { path, source } => write!(f, "{}: {source}", path.display()),
            Self::InvalidArticlePath(path) => {
                write!(
                    f,
                    "article path must point to a .md file: {}",
                    path.display()
                )
            }
            Self::ConfigExists(path) => write!(f, "config already exists: {}", path.display()),
            Self::InvalidNumber { flag, value } => {
                write!(f, "invalid number for {flag}: {value}")
            }
            Self::InvalidCsv(msg) => write!(f, "invalid csv: {msg}"),
            Self::MissingEnvVar(name) => write!(f, "missing env var: {name}"),
            Self::PushFailed { message, ip_hint } => {
                write!(f, "push failed: {message}")?;
                if let Some(ip) = ip_hint {
                    write!(f, "\n  current IP: {ip} — add it to WeChat IP allowlist")?;
                }
                Ok(())
            }
            Self::NoDraftJson(path) => write!(
                f,
                "draft.json not found: {}\n  run 'moonpub render' first",
                path.display()
            ),
            Self::NoHtml(path) => write!(
                f,
                "html not found: {}\n  run 'moonpub render' first",
                path.display()
            ),
            Self::AutomationFailed { message } => {
                write!(f, "browser automation failed: {message}")
            }
        }
    }
}

impl std::error::Error for AppError {}

// ── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Config {
    pub vault_root: Option<PathBuf>,
    pub wechat_appid: Option<String>,
    pub wechat_author: Option<String>,
    pub wechat_thumb_media_id: Option<String>,
    pub wechat_account_type: Option<String>,
    pub wechat_auto_publish: bool,
    pub wechat_theme: Option<String>,
    pub wechat_collection: Option<String>,
    pub blog_kind: Option<String>,
    pub blog_root: Option<PathBuf>,
    pub author_bio: Option<String>,
    pub qrcode_path: Option<String>,
}

impl Config {
    /// Minimal TOML parser that extracts string values from our known keys.
    pub fn from_toml(content: &str) -> Self {
        let mut cfg = Self::default();

        for line in content.lines() {
            let line = line.trim();
            if line.starts_with('#') || line.starts_with('[') || line.is_empty() {
                continue;
            }
            if let Some((key, value)) = split_toml_pair(line) {
                match key {
                    "root" => {
                        // vault.root or blog.root — we use context; vault first if not set
                        if cfg.vault_root.is_none() {
                            cfg.vault_root = Some(PathBuf::from(value));
                        } else {
                            cfg.blog_root = Some(PathBuf::from(value));
                        }
                    }
                    "appid" => cfg.wechat_appid = Some(value.to_owned()),
                    "author" => cfg.wechat_author = Some(value.to_owned()),
                    "account_type" => cfg.wechat_account_type = Some(value.to_owned()),
                    "auto_publish" => cfg.wechat_auto_publish = value == "true",
                    "theme" => cfg.wechat_theme = Some(value.to_owned()),
                    "collection" => cfg.wechat_collection = Some(value.to_owned()),
                    "thumb_media_id" => cfg.wechat_thumb_media_id = Some(value.to_owned()),
                    "kind" => cfg.blog_kind = Some(value.to_owned()),
                    "author_bio" => cfg.author_bio = Some(value.to_owned()),
                    "qrcode" => cfg.qrcode_path = Some(value.to_owned()),
                    _ => {}
                }
            }
        }

        cfg
    }

    pub fn load(path: &Path) -> Result<Self, AppError> {
        let content = fs::read_to_string(path).map_err(|source| AppError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        Ok(Self::from_toml(&content))
    }
}

fn split_toml_pair(line: &str) -> Option<(&str, &str)> {
    let (key, rest) = line.split_once('=')?;
    let key = key.trim();
    let rest = rest.trim();
    let value = rest
        .strip_prefix('"')
        .and_then(|s| s.strip_suffix('"'))
        .unwrap_or(rest);
    Some((key, value))
}

// ── Options::parse ───────────────────────────────────────────────────────────

impl Options {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, AppError> {
        let mut vault = std::env::current_dir().map_err(|source| AppError::Io {
            path: PathBuf::from("."),
            source,
        })?;
        let mut rest = Vec::new();
        let mut json = false;
        let mut config: Option<PathBuf> = None;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--vault" => {
                    let value = args.next().ok_or(AppError::MissingValue("--vault"))?;
                    vault = PathBuf::from(value);
                }
                "--config" => {
                    let value = args.next().ok_or(AppError::MissingValue("--config"))?;
                    config = Some(PathBuf::from(value));
                }
                "--json" => json = true,
                "-h" | "--help" => {
                    return Ok(Self {
                        vault,
                        command: Command::Help,
                        json,
                        config,
                    });
                }
                value if value.starts_with('-') => {
                    return Err(AppError::UnknownOption(value.to_owned()));
                }
                value => {
                    rest.push(value.to_owned());
                    rest.extend(args);
                    break;
                }
            }
        }

        // Apply config file: if --config is given, load it and override vault.
        if let Some(cfg_path) = &config {
            let cfg = Config::load(cfg_path)?;
            if let Some(root) = cfg.vault_root {
                vault = root;
            }
        }

        let Some(command) = rest.first() else {
            return Err(AppError::MissingCommand);
        };

        // subcommand --help → show help text
        if rest.get(1).map(|s| s.as_str()) == Some("--help")
            || rest.get(1).map(|s| s.as_str()) == Some("-h")
        {
            return Ok(Self {
                vault,
                command: Command::Help,
                json,
                config,
            });
        }

        let command = match command.as_str() {
            "init" => {
                let path = rest
                    .get(1)
                    .map(PathBuf::from)
                    .unwrap_or_else(|| vault.join(DEFAULT_CONFIG));
                Command::Init { path }
            }
            "status" => Command::Status,
            "check" => {
                let value = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("check <article.md>"))?;
                Command::Check {
                    article: PathBuf::from(value),
                }
            }
            "push" => {
                let value = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("push <article.md>"))?;
                let mut auto_render = false;
                let extra = rest[2..].iter();
                for flag in extra {
                    match flag.as_str() {
                        "--render" => auto_render = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::Push {
                    article: PathBuf::from(value),
                    auto_render,
                }
            }
            "update-draft" => {
                let value = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("update-draft <article.md>"))?;
                let mut media_id = None;
                let mut extra = rest[2..].iter();
                while let Some(flag) = extra.next() {
                    match flag.as_str() {
                        "--media-id" => {
                            media_id = Some(flag_value(&mut extra, "--media-id")?);
                        }
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::UpdateDraft {
                    article: PathBuf::from(value),
                    media_id,
                }
            }
            "ship" => {
                let value = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("ship <article.md>"))?;
                let mut style = None;
                let mut extra = rest[2..].iter();
                while let Some(flag) = extra.next() {
                    match flag.as_str() {
                        "--style" => {
                            style = Some(flag_value(&mut extra, "--style")?);
                        }
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        _ => {}
                    }
                }
                Command::Ship {
                    article: PathBuf::from(value),
                    style,
                }
            }
            "mark-ready" => {
                let value = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("mark-ready <article.md>"))?;
                Command::MarkReady {
                    article: PathBuf::from(value),
                }
            }
            "cover" => {
                let value = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("cover <article.md>"))?;
                let mut style = None;
                let mut screenshot = false;
                let mut extra = rest[2..].iter();
                while let Some(flag) = extra.next() {
                    match flag.as_str() {
                        "--style" => {
                            style = Some(flag_value(&mut extra, "--style")?);
                        }
                        "--screenshot" => screenshot = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        _ => {}
                    }
                }
                Command::Cover {
                    article: PathBuf::from(value),
                    style,
                    screenshot,
                }
            }
            "humanize" => {
                let value = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("humanize <article.md>"))?;
                Command::Humanize {
                    article: PathBuf::from(value),
                }
            }
            "login" => Command::Login,
            "configure" => Command::Configure,
            "step-test" => Command::StepTest,
            "test-zanshang" => Command::TestZanshang,
            "list-drafts" => Command::ListDrafts,
            "delete-draft" => {
                let media_id = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("delete-draft <media_id>"))?
                    .clone();
                Command::DeleteDraft { media_id }
            }
            "fetch" => {
                let url = rest.get(1).ok_or(AppError::MissingValue("fetch <url>"))?;
                Command::Fetch {
                    url: url.to_owned(),
                }
            }
            "mark-published" => {
                let value = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("mark-published <article.md>"))?;
                Command::MarkPublished {
                    article: PathBuf::from(value),
                }
            }
            "export" => {
                let value = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("export <article.md>"))?;
                Command::Export {
                    article: PathBuf::from(value),
                }
            }
            "preview" => {
                let value = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("preview <article.md>"))?;
                Command::Preview {
                    article: PathBuf::from(value),
                }
            }
            "render" => {
                let value = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("render <article.md>"))?;
                let mut author = None;
                let mut thumb_media_id = None;
                let mut humanize = false;
                let mut extra = rest[2..].iter();
                while let Some(flag) = extra.next() {
                    match flag.as_str() {
                        "--author" => {
                            author = Some(flag_value(&mut extra, "--author")?);
                        }
                        "--thumb" => {
                            thumb_media_id = Some(flag_value(&mut extra, "--thumb")?);
                        }
                        "--humanize" => humanize = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::Render {
                    article: PathBuf::from(value),
                    author,
                    thumb_media_id,
                    humanize,
                }
            }
            "radar" => Command::Radar(parse_radar_command(&rest[1..])?),
            "help" => Command::Help,
            value => return Err(AppError::UnknownCommand(value.to_owned())),
        };

        Ok(Self {
            vault,
            command,
            json,
            config,
        })
    }
}

// ── run ──────────────────────────────────────────────────────────────────────

pub fn run(options: &Options) -> Result<String, AppError> {
    let raw = match &options.command {
        Command::Init { path } => init_config(path),
        Command::Status => status(&options.vault),
        Command::Check { article } => check_article(&options.vault, article),
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
                let article_path = resolve_article_path(&options.vault, article);
                let md = fs::read_to_string(&article_path).map_err(|source| AppError::Io {
                    path: article_path.clone(),
                    source,
                })?;
                let processed = humanize::humanize(&md);
                fs::write(&article_path, &processed).map_err(|source| AppError::Io {
                    path: article_path.clone(),
                    source,
                })?;
            }
            let theme_name = cfg.wechat_theme.as_deref().unwrap_or("default");
            let qrcode = cfg.qrcode_path.as_deref().unwrap_or("");
            render_article(
                &options.vault,
                article,
                &resolved_author,
                &resolved_thumb,
                theme_name,
                None,
                qrcode,
            )
        }
        Command::Cover {
            article,
            style,
            screenshot,
        } => {
            let article_path = resolve_article_path(&options.vault, article);
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
                _ => cover::CoverStyle::Clean,
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
        Command::Login => publish::login().map_err(|e| AppError::PushFailed {
            message: e,
            ip_hint: None,
        }),
        Command::Configure => {
            let cfg = options
                .config
                .as_deref()
                .map(Config::load)
                .transpose()?
                .unwrap_or_default();
            publish::auto_configure("", cfg.wechat_collection.as_deref().unwrap_or("书")).map_err(
                |e| AppError::PushFailed {
                    message: e,
                    ip_hint: None,
                },
            )
        }
        Command::StepTest => publish::step_test().map_err(|e| AppError::PushFailed {
            message: e,
            ip_hint: None,
        }),
        Command::TestZanshang => publish::test_zanshang().map_err(|e| AppError::PushFailed {
            message: e,
            ip_hint: None,
        }),
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
            let article_path = resolve_article_path(&options.vault, article);
            let md = fs::read_to_string(&article_path).map_err(|source| AppError::Io {
                path: article_path.clone(),
                source,
            })?;
            let processed = humanize::humanize(&md);
            fs::write(&article_path, &processed).map_err(|source| AppError::Io {
                path: article_path.clone(),
                source,
            })?;
            Ok(format!("humanized {}", article_path.display()))
        }
        Command::Fetch { url } => match fetch::fetch_article(url) {
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
            push_article(&options.vault, article, *auto_render, &cfg)
        }
        Command::UpdateDraft { article, media_id } => {
            let cfg = options
                .config
                .as_deref()
                .map(Config::load)
                .transpose()?
                .unwrap_or_default();
            update_draft(&options.vault, article, media_id.as_deref(), &cfg)
        }
        Command::MarkReady { article } => {
            let slug = article_slug(article)?;
            add_status(&options.vault, &slug, "ready", "confirmed")
        }
        Command::Ship { article, style } => {
            let vault = &options.vault;
            let art_path = resolve_article_path(vault, article);
            let slug = art_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
            let dir = art_path.parent().unwrap_or(&art_path);

            // Resolve config
            let cfg = options
                .config
                .as_deref()
                .map(Config::load)
                .transpose()?
                .unwrap_or_default();
            let author = cfg.wechat_author.as_deref().unwrap_or("作者").to_owned();
            let thumb = cfg
                .wechat_thumb_media_id
                .as_deref()
                .unwrap_or("")
                .to_owned();

            let mut results = Vec::new();
            // cover
            let front =
                parse_frontmatter(&fs::read_to_string(&art_path).map_err(|e| AppError::Io {
                    path: art_path.clone(),
                    source: e,
                })?);
            let cover_style = match style.as_deref() {
                Some("dark") => cover::CoverStyle::Dark,
                Some("minimal") => cover::CoverStyle::Minimal,
                Some("warm") => cover::CoverStyle::Warm,
                Some("serif") => cover::CoverStyle::Serif,
                Some("gradient") => cover::CoverStyle::Gradient,
                _ => cover::CoverStyle::Clean,
            };
            let html = cover::generate_cover_html(
                front.title.as_deref().unwrap_or(""),
                front.digest.as_deref().unwrap_or(""),
                &author,
                cover_style,
            );
            let cover_path = dir.join(format!("{slug}.cover.html"));
            fs::write(&cover_path, &html).map_err(|e| AppError::Io {
                path: cover_path.clone(),
                source: e,
            })?;
            results.push(format!("cover:  {}", cover_path.display()));
            // render with cover injected at top
            let qrcode_ship = cfg.qrcode_path.as_deref().unwrap_or("");
            results.push(render_article(
                vault,
                article,
                &author,
                &thumb,
                cfg.wechat_theme.as_deref().unwrap_or("default"),
                Some(&html),
                qrcode_ship,
            )?);
            // push (Phase 1: API)
            results.push(push_article(vault, article, false, &cfg)?);

            // Phase 2: browser automation (already called inside push_article)
            // No need to call again — push_article handles it

            // export
            let pub_path = vault.join("Articles/published").join(format!("{slug}.md"));
            let src = if pub_path.exists() {
                &pub_path
            } else {
                &art_path
            };
            if let Some(br) = cfg.blog_root.as_deref() {
                results.push(export_article(vault, src, br)?);
            }
            Ok(results.join("\n\n"))
        }
        Command::MarkPublished { article } => {
            let slug = article_slug(article)?;
            add_status(&options.vault, &slug, "published", "published")
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
            export_article(&options.vault, article, blog_root)
        }
        Command::Preview { article } => preview_article(&options.vault, article),
        Command::Radar(command) => run_radar(&options.vault, command),
        Command::Help => Ok(help_text()),
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

// ── init ─────────────────────────────────────────────────────────────────────

pub fn init_config(path: &Path) -> Result<String, AppError> {
    if path.exists() {
        return Err(AppError::ConfigExists(path.to_path_buf()));
    }

    let config = sample_config();
    fs::write(path, config).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    Ok(format!("created {}", path.display()))
}

// ── status ────────────────────────────────────────────────────────────────────

pub fn status(vault: &Path) -> Result<String, AppError> {
    let articles_dir = vault.join("Articles");
    let mut stages = Vec::new();

    for stage in ["drafts", "ready", "published"] {
        let dir = articles_dir.join(stage);
        stages.push((stage, list_markdown_files(&dir)?));
    }

    let statuses = read_statuses(vault).unwrap_or_default();

    Ok(format_status(&stages, &statuses))
}

// ── check ─────────────────────────────────────────────────────────────────────

pub fn check_article(vault: &Path, article: &Path) -> Result<String, AppError> {
    let article = resolve_article_path(vault, article);
    if article.extension().and_then(|ext| ext.to_str()) != Some("md") {
        return Err(AppError::InvalidArticlePath(article));
    }

    let slug = article
        .file_stem()
        .and_then(|stem| stem.to_str())
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?;
    let dir = article
        .parent()
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?;

    let bundle = ArticleBundle {
        markdown: article.clone(),
        html: dir.join(format!("{slug}.html")),
        draft_json: dir.join(format!("{slug}.draft.json")),
        media_id: dir.join(format!("{slug}.media_id")),
    };

    Ok(bundle.report())
}

fn status_store_path(vault: &Path) -> PathBuf {
    vault.join(".moonpub").join("status.jsonl")
}

fn article_slug(article: &Path) -> Result<String, AppError> {
    article
        .file_stem()
        .and_then(|s| s.to_str())
        .map(str::to_owned)
        .ok_or_else(|| AppError::InvalidArticlePath(article.to_path_buf()))
}

fn add_status(vault: &Path, slug: &str, status: &str, detail: &str) -> Result<String, AppError> {
    let path = status_store_path(vault);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|source| AppError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .map_err(|source| AppError::Io {
            path: path.clone(),
            source,
        })?;
    let line = format!(
        "{{\"slug\":\"{}\",\"status\":\"{}\",\"detail\":\"{}\"}}",
        escape_json(slug),
        status,
        detail
    );
    writeln!(file, "{line}").map_err(|source| AppError::Io {
        path: path.clone(),
        source,
    })?;
    Ok(format!("{slug}: {status}"))
}

fn read_statuses(vault: &Path) -> Result<Vec<(String, String, String)>, AppError> {
    let path = status_store_path(vault);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(&path).map_err(|source| AppError::Io {
        path: path.clone(),
        source,
    })?;
    let mut statuses = Vec::new();
    for line in content.lines().filter(|l| !l.trim().is_empty()) {
        let slug = extract_json_string(line, "slug").unwrap_or_default();
        let status = extract_json_string(line, "status").unwrap_or_default();
        let detail = extract_json_string(line, "detail").unwrap_or_default();
        if !slug.is_empty() {
            statuses.push((slug, status, detail));
        }
    }
    Ok(statuses)
}

pub(crate) fn resolve_article_path(vault: &Path, article: &Path) -> PathBuf {
    if article.is_absolute() {
        article.to_path_buf()
    } else {
        vault.join(article)
    }
}

fn list_markdown_files(dir: &Path) -> Result<Vec<String>, AppError> {
    if !dir.exists() {
        return Ok(Vec::new());
    }

    let mut files = Vec::new();
    let entries = fs::read_dir(dir).map_err(|source| AppError::Io {
        path: dir.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| AppError::Io {
            path: dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) == Some("md")
            && let Some(name) = path.file_name().and_then(|name| name.to_str())
        {
            files.push(name.to_owned());
        }
    }

    files.sort();
    Ok(files)
}

fn format_status(stages: &[(&str, Vec<String>)], statuses: &[(String, String, String)]) -> String {
    let mut output = String::new();
    for (stage, files) in stages {
        output.push_str(&format!("-- {stage} --\n"));
        if files.is_empty() {
            output.push_str("  (empty)\n");
        } else {
            for file in files {
                let slug = file.trim_end_matches(".md");
                let latest = statuses
                    .iter()
                    .rev()
                    .find(|(s, _, _)| s == slug)
                    .map(|(_, st, d)| format!(" [{st}] {d}"))
                    .unwrap_or_default();
                output.push_str(&format!("  {file}{latest}\n"));
            }
        }
    }
    output.trim_end().to_owned()
}

struct ArticleBundle {
    markdown: PathBuf,
    html: PathBuf,
    draft_json: PathBuf,
    media_id: PathBuf,
}

impl ArticleBundle {
    fn report(&self) -> String {
        let required = [
            ("markdown", &self.markdown),
            ("html", &self.html),
            ("draft_json", &self.draft_json),
        ];

        let mut output = String::from("article bundle\n");
        let mut complete = true;
        for (label, path) in required {
            let exists = path.exists();
            complete &= exists;
            output.push_str(&format!(
                "  {label}: {} {}\n",
                marker(exists),
                path.display()
            ));
        }

        let media_id_exists = self.media_id.exists();
        output.push_str(&format!(
            "  media_id: {} {}\n",
            marker(media_id_exists),
            self.media_id.display()
        ));
        output.push_str(&format!(
            "  publishable: {}",
            if complete { "yes" } else { "no" }
        ));
        output
    }
}

fn marker(exists: bool) -> &'static str {
    if exists { "ok" } else { "missing" }
}

pub fn sample_config() -> &'static str {
    r#"[vault]
root = "/path/to/ObsidianMain"

[wechat]
appid = ""
author = ""
account_type = "personal"
auto_publish = false
theme = "default"
collection = "书"
thumb_media_id = ""
author_bio = "每周分享读书笔记与思考。"
qrcode = "Context/assets/qrcode-group.jpg"

[blog]
kind = "zola"
root = "/path/to/blog"
"#
}

pub fn help_text() -> String {
    String::from(
        r#"MoonPub CLI

Usage:
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] init [moonpub.toml]
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] status
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] check <article.md>
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] render <article.md> [--author <name>] [--thumb <media_id>]
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] push <article.md> [--render]
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] update-draft <article.md> [--media-id <id>]
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] export <article.md>
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] preview <article.md>
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] mark-ready <article.md>
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] mark-published <article.md>
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] humanize <article.md>
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] login
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] fetch <url>
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] cover <article.md> [--style dark|clean|minimal|warm|serif|gradient] [--screenshot]
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] ship <article.md> [--style dark|clean|minimal|warm|serif|gradient]
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] radar add --platform <name> --keyword <text> --title <text> [--url <url>] [--likes <n>] [--collects <n>] [--comments <n>]
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] radar list [--platform <name>] [--keyword <text>]
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] radar import <file.csv> [--platform <name>]
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] radar analyze <article.md> --platform <name> [--top <n>]
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] radar suggest <article.md> --platform <name> [--top <n>]
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] radar scrape --platform <name> --keyword <text> [--count <n>] [--url <url>]

Commands:
  init         Create a sample moonpub.toml
  status       List article files in Articles/drafts, ready, and published
  check        Check whether an article bundle has md/html/draft.json files
  render       Generate <slug>.html and <slug>.draft.json from a Markdown article
  push         Push draft to WeChat (direct API), write .media_id, move to published/
  update-draft Re-push updated HTML to an existing WeChat draft by media_id
  export       Export article to Zola blog (YAML→TOML frontmatter, strip WeChat footer)
  preview      Open the rendered HTML in the system browser
  humanize     Strip AI patterns from article in-place
  login        One-time WeChat backend login (opens browser for QR scan)
  step-test    Interactive browser automation test (step-by-step with screenshots)
  list-drafts  List all drafts (shows media_id + title)
  delete-draft Delete a draft by media_id  (delete-draft <media_id>)
  fetch        Fetch a WeChat article and extract title + body (requires Chrome)
  cover        Generate a cover HTML file from article frontmatter
  ship         Cover + render + push + export in one command
  radar        Store and analyze platform trend samples (add/list/import/analyze/suggest/scrape)
"#,
    )
}

// ── export ───────────────────────────────────────────────────────────────────

pub fn export_article(vault: &Path, article: &Path, blog_root: &Path) -> Result<String, AppError> {
    let article = resolve_article_path(vault, article);
    if article.extension().and_then(|e| e.to_str()) != Some("md") {
        return Err(AppError::InvalidArticlePath(article));
    }

    let md = fs::read_to_string(&article).map_err(|source| AppError::Io {
        path: article.clone(),
        source,
    })?;

    let front = parse_frontmatter(&md);
    let body = strip_frontmatter(&md);
    let body = strip_wechat_footer(body);

    let title = front.title.as_deref().unwrap_or("").to_owned();
    let date = front.date.as_deref().unwrap_or("1970-01-01").to_owned();
    let tags = front.tags.clone();

    let slug = article
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?;

    // Replace WeChat CDN banner with local blog image.
    let body = replace_wechat_images(body);

    let zola_fm = build_zola_frontmatter(&title, &date, &tags);
    let content = format!("{zola_fm}\n<!-- more -->\n\n{}", body.trim_start());

    let filename = format!("{date}-{slug}.md");
    let content_dir = blog_root.join("content");
    fs::create_dir_all(&content_dir).map_err(|source| AppError::Io {
        path: content_dir.clone(),
        source,
    })?;
    let dst = content_dir.join(&filename);
    fs::write(&dst, &content).map_err(|source| AppError::Io {
        path: dst.clone(),
        source,
    })?;

    Ok(format!("exported\n  {}", dst.display()))
}

fn build_zola_frontmatter(title: &str, date: &str, tags: &[String]) -> String {
    let tags_toml = tags
        .iter()
        .map(|t| format!("\"{}\"", escape_toml_string(t)))
        .collect::<Vec<_>>()
        .join(", ");

    format!(
        "+++\ntitle = \"{}\"\ndescription = \"{}\"\ndate = {date}T00:00:00Z\n[taxonomies]\ncategories = [\"读书\"]\ntags = [{tags_toml}]\n+++\n",
        escape_toml_string(title),
        escape_toml_string(title),
    )
}

fn escape_toml_string(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
}

fn replace_wechat_images(body: &str) -> String {
    // Replace inline WeChat CDN banner references with the local blog image path.
    let re_cdn = "mmbiz.qpic.cn";
    let local = "/images/wechat-follow.png";
    body.lines()
        .map(|line| {
            if line.contains(re_cdn) {
                // Replace the whole src URL inside markdown image syntax.
                // Pattern: ![alt](http://mmbiz.qpic.cn/...)
                let mut out = String::new();
                let mut rest = line;
                while let Some(start) = rest.find("![") {
                    out.push_str(&rest[..start]);
                    let after = &rest[start..];
                    // Find matching )
                    if let Some(url_end) = after.find(')') {
                        let img_tag = &after[..url_end + 1];
                        if img_tag.contains(re_cdn) {
                            // Extract alt text
                            let alt_start = 2;
                            let alt_end = img_tag.find(']').unwrap_or(alt_start);
                            let alt = &img_tag[alt_start..alt_end];
                            out.push_str(&format!("![{alt}]({local})"));
                        } else {
                            out.push_str(img_tag);
                        }
                        rest = &after[url_end + 1..];
                    } else {
                        out.push_str(after);
                        rest = "";
                        break;
                    }
                }
                out.push_str(rest);
                out
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}

// ── preview ───────────────────────────────────────────────────────────────────

pub fn preview_article(vault: &Path, article: &Path) -> Result<String, AppError> {
    let article = resolve_article_path(vault, article);
    let slug = article
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?;
    let dir = article
        .parent()
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?;
    let html = dir.join(format!("{slug}.html"));

    if !html.exists() {
        return Err(AppError::NoHtml(html));
    }

    #[cfg(target_os = "macos")]
    let opener = "open";
    #[cfg(target_os = "linux")]
    let opener = "xdg-open";
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let opener = "start";

    std::process::Command::new(opener)
        .arg(&html)
        .status()
        .map_err(|source| AppError::Io {
            path: PathBuf::from(opener),
            source,
        })?;

    Ok(format!("opening {}", html.display()))
}

// ── update-draft ─────────────────────────────────────────────────────────────

pub fn update_draft(
    vault: &Path,
    article: &Path,
    media_id_arg: Option<&str>,
    cfg: &Config,
) -> Result<String, AppError> {
    let article = resolve_article_path(vault, article);
    if article.extension().and_then(|e| e.to_str()) != Some("md") {
        return Err(AppError::InvalidArticlePath(article));
    }

    let slug = article
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?
        .to_owned();
    let dir = article
        .parent()
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?
        .to_path_buf();

    let draft_json = dir.join(format!("{slug}.draft.json"));
    if !draft_json.exists() {
        return Err(AppError::NoDraftJson(draft_json));
    }

    // media_id: CLI arg > .media_id file
    let media_id = if let Some(id) = media_id_arg {
        id.to_owned()
    } else {
        let media_id_path = dir.join(format!("{slug}.media_id"));
        fs::read_to_string(&media_id_path)
            .map(|s| s.trim().to_owned())
            .map_err(|_| AppError::PushFailed {
                message: format!(
                    "no media_id found — pass --media-id or ensure {slug}.media_id exists"
                ),
                ip_hint: None,
            })?
    };

    let appid = std::env::var("WECHAT_APPID")
        .ok()
        .or_else(|| cfg.wechat_appid.clone())
        .ok_or(AppError::MissingEnvVar("WECHAT_APPID"))?;
    let secret =
        std::env::var("WECHAT_SECRET").map_err(|_| AppError::MissingEnvVar("WECHAT_SECRET"))?;

    let client = WechatClient::new(&appid, &secret);
    let token = client.access_token()?;
    client.update_draft(&token, &media_id, &draft_json)?;

    Ok(format!(
        "updated draft\n  media_id: {media_id}\n  next: preview in WeChat backend, then publish"
    ))
}

// ── list-drafts / delete-draft ────────────────────────────────────────────────

fn wechat_client(cfg: &Config) -> Result<WechatClient, AppError> {
    let appid = std::env::var("WECHAT_APPID")
        .ok()
        .or_else(|| cfg.wechat_appid.clone())
        .ok_or(AppError::MissingEnvVar("WECHAT_APPID"))?;
    let secret =
        std::env::var("WECHAT_SECRET").map_err(|_| AppError::MissingEnvVar("WECHAT_SECRET"))?;
    Ok(WechatClient::new(appid, secret))
}

pub fn list_drafts(cfg: &Config) -> Result<String, AppError> {
    let client = wechat_client(cfg)?;
    let token = client.access_token()?;
    let (items, total) = client.list_drafts(&token, 0, 20)?;
    if items.is_empty() {
        return Ok("草稿箱为空".to_owned());
    }
    let mut out = format!("草稿总数: {total}\n");
    for item in &items {
        out.push_str(&format!("  {} | {}\n", item.media_id, item.title));
    }
    Ok(out.trim_end().to_owned())
}

pub fn delete_draft(media_id: &str, cfg: &Config) -> Result<String, AppError> {
    let client = wechat_client(cfg)?;
    let token = client.access_token()?;
    client.delete_draft(&token, media_id)?;
    Ok(format!("已删除草稿: {media_id}"))
}

// ── push ──────────────────────────────────────────────────────────────────────

/// Scan HTML for local `src="..."` img attributes, upload each to WeChat,
/// and return the HTML with those src values replaced by CDN URLs.
/// Remote URLs (http/https) are left untouched.
fn upload_local_images(
    html: &str,
    article_dir: &Path,
    client: &WechatClient,
    token: &str,
) -> Result<(String, usize), AppError> {
    let mut result = html.to_owned();
    let mut search = result.as_str();
    let mut replacements: Vec<(String, String)> = Vec::new();

    while let Some(pos) = search.find("src=\"") {
        let rest = &search[pos + 5..];
        let end = rest.find('"').unwrap_or(rest.len());
        let src = &rest[..end];

        if !src.starts_with("http://")
            && !src.starts_with("https://")
            && !src.is_empty()
            && !replacements.iter().any(|(k, _)| k == src)
        {
            let path = if src.starts_with('/') {
                PathBuf::from(src)
            } else {
                article_dir.join(src)
            };
            if path.exists() {
                let url = client.upload_image_url(token, &path)?;
                replacements.push((src.to_owned(), url));
            }
        }
        search = &search[pos + 5 + end..];
    }

    let count = replacements.len();
    for (src, url) in replacements {
        result = result.replace(&format!("src=\"{src}\""), &format!("src=\"{url}\""));
    }
    Ok((result, count))
}

pub fn push_article(
    vault: &Path,
    article: &Path,
    auto_render: bool,
    cfg: &Config,
) -> Result<String, AppError> {
    let article = resolve_article_path(vault, article);
    if article.extension().and_then(|e| e.to_str()) != Some("md") {
        return Err(AppError::InvalidArticlePath(article));
    }

    let slug = article
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?
        .to_owned();
    let dir = article
        .parent()
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?
        .to_path_buf();

    let draft_json = dir.join(format!("{slug}.draft.json"));

    // Auto-render if requested and draft.json is missing.
    if !draft_json.exists() {
        if auto_render {
            let author = cfg.wechat_author.as_deref().unwrap_or("作者").to_owned();
            let thumb = cfg
                .wechat_thumb_media_id
                .as_deref()
                .unwrap_or("")
                .to_owned();
            render_article(
                vault,
                &article,
                &author,
                &thumb,
                cfg.wechat_theme.as_deref().unwrap_or("default"),
                None,
                cfg.qrcode_path.as_deref().unwrap_or(""),
            )?;
        } else {
            return Err(AppError::NoDraftJson(draft_json));
        }
    }

    // Credentials: env vars take priority over config file.
    let appid = std::env::var("WECHAT_APPID")
        .ok()
        .or_else(|| cfg.wechat_appid.clone())
        .ok_or(AppError::MissingEnvVar("WECHAT_APPID"))?;
    let secret =
        std::env::var("WECHAT_SECRET").map_err(|_| AppError::MissingEnvVar("WECHAT_SECRET"))?;

    // Call WeChat API directly — no md2wechat dependency.
    let client = WechatClient::new(&appid, &secret);
    let token = client.access_token()?;

    // Upload local images in the HTML and rewrite draft.json before pushing.
    let html_path = dir.join(format!("{slug}.html"));
    let mut uploaded_images = 0usize;
    if html_path.exists() {
        let html = fs::read_to_string(&html_path).map_err(|source| AppError::Io {
            path: html_path.clone(),
            source,
        })?;
        let (updated, img_count) = upload_local_images(&html, &dir, &client, &token)?;
        if img_count > 0 {
            uploaded_images = img_count;
            fs::write(&html_path, &updated).map_err(|source| AppError::Io {
                path: html_path.clone(),
                source,
            })?;
            // Rebuild draft.json with the image-replaced HTML.
            let md = fs::read_to_string(&article).map_err(|source| AppError::Io {
                path: article.clone(),
                source,
            })?;
            let front = parse_frontmatter(&md);
            let title = front.title.as_deref().unwrap_or("").to_owned();
            let digest = front
                .digest
                .clone()
                .unwrap_or_else(|| first_non_empty_line(strip_frontmatter(&md)).to_owned());
            let author = cfg.wechat_author.as_deref().unwrap_or("作者");
            let thumb = cfg.wechat_thumb_media_id.as_deref().unwrap_or("");
            let new_draft = build_draft_json(&title, author, &digest, &updated, thumb);
            fs::write(&draft_json, &new_draft).map_err(|source| AppError::Io {
                path: draft_json.clone(),
                source,
            })?;
        }
    }

    let media_id = client.create_draft(&token, &draft_json)?;

    // Write .media_id file.
    let media_id_path = dir.join(format!("{slug}.media_id"));
    fs::write(&media_id_path, &media_id).map_err(|source| AppError::Io {
        path: media_id_path.clone(),
        source,
    })?;

    // Move article bundle to published/ if currently in drafts/ or ready/.
    let mut moved = String::new();
    if let Some(stage) = dir_stage(&dir)
        && (stage == "drafts" || stage == "ready")
    {
        let published = dir
            .parent()
            .map(|p| p.join("published"))
            .unwrap_or_else(|| dir.join("published"));
        fs::create_dir_all(&published).map_err(|source| AppError::Io {
            path: published.clone(),
            source,
        })?;
        for ext in &["md", "html", "draft.json", "media_id"] {
            let src = dir.join(format!("{slug}.{ext}"));
            if src.exists() {
                let dst = published.join(format!("{slug}.{ext}"));
                fs::rename(&src, &dst).map_err(|source| AppError::Io {
                    path: src.clone(),
                    source,
                })?;
            }
        }
        moved = format!("\n  moved to {}", published.display());
    }

    let _ = add_status(vault, &slug, "pushed", &media_id);
    let img_note = if uploaded_images > 0 {
        format!("\n  images: {uploaded_images} uploaded to WeChat CDN")
    } else {
        String::new()
    };
    let mut result = format!("pushed\n  media_id: {media_id}{moved}{img_note}");

    // Auto-publish for verified/service accounts
    if cfg.wechat_auto_publish {
        let acct_type = cfg.wechat_account_type.as_deref().unwrap_or("personal");
        if acct_type != "personal" {
            match client.free_publish(&token, &media_id) {
                Ok(publish_id) => {
                    let _ = add_status(vault, &slug, "published", &publish_id);
                    result.push_str(&format!(
                        "\n  auto-published ({}): {}",
                        acct_type, publish_id
                    ));
                }
                Err(e) => {
                    result.push_str(&format!("\n  auto-publish failed: {e}"));
                }
            }
        }
    }

    // Browser automation (single call)
    let collection = cfg.wechat_collection.as_deref().unwrap_or("书");
    match publish::auto_configure(&media_id, collection) {
        Ok(msg) => result.push_str(&format!("\n  ✓ {msg}")),
        Err(e) => result.push_str(&format!("\n  ⚠ automation: {e}")),
    }

    Ok(result)
}

/// Extract media_id from md2wechat --json response.
/// Success shape: {"success":true,"data":{"media_id":"..."}}
/// Try to pull the current IP from a WeChat error message like "invalid ip 1.2.3.4".
fn extract_ip_from_message(msg: &str) -> Option<String> {
    let marker = "invalid ip ";
    let start = msg.find(marker)? + marker.len();
    let ip: String = msg[start..]
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    if ip.is_empty() { None } else { Some(ip) }
}

/// Return the stage name ("drafts" | "ready" | "published") if the dir ends with one.
fn dir_stage(dir: &Path) -> Option<&str> {
    dir.file_name()?.to_str().and_then(|name| {
        ["drafts", "ready", "published"]
            .iter()
            .find(|&&s| s == name)
            .copied()
    })
}

// ── render ────────────────────────────────────────────────────────────────────

pub fn render_article(
    vault: &Path,
    article: &Path,
    author: &str,
    thumb_media_id: &str,
    theme_name: &str,
    cover_html: Option<&str>,
    qrcode_path: &str,
) -> Result<String, AppError> {
    let article = resolve_article_path(vault, article);
    if article.extension().and_then(|e| e.to_str()) != Some("md") {
        return Err(AppError::InvalidArticlePath(article));
    }

    let md = fs::read_to_string(&article).map_err(|source| AppError::Io {
        path: article.clone(),
        source,
    })?;

    let front = parse_frontmatter(&md);
    let body = strip_frontmatter(&md);
    let body = strip_wechat_footer(body);
    let t = theme::Theme::from_name(theme_name);
    let html_body = md_to_wechat_html(body, &t);
    let body_with_cover = match cover_html {
        Some(cover) => format!("{cover}\n{html_body}"),
        None => html_body,
    };
    let footer_cfg = footer::FooterConfig::from_config(author, qrcode_path);
    let full_html = wrap_wechat_html(&body_with_cover, &t, &footer_cfg);

    let title = front.title.as_deref().unwrap_or("").to_owned();
    let digest = front
        .digest
        .clone()
        .unwrap_or_else(|| first_non_empty_line(body).to_owned());

    let slug = article
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?;
    let dir = article
        .parent()
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?;

    let html_path = dir.join(format!("{slug}.html"));
    let json_path = dir.join(format!("{slug}.draft.json"));

    fs::write(&html_path, &full_html).map_err(|source| AppError::Io {
        path: html_path.clone(),
        source,
    })?;

    let draft_json = build_draft_json(&title, author, &digest, &full_html, thumb_media_id);
    fs::write(&json_path, &draft_json).map_err(|source| AppError::Io {
        path: json_path.clone(),
        source,
    })?;

    let _ = add_status(vault, slug, "rendered", "");

    Ok(format!(
        "rendered\n  html:  {}\n  draft: {}",
        html_path.display(),
        json_path.display()
    ))
}

// ── frontmatter ───────────────────────────────────────────────────────────────

#[derive(Debug, Default)]
struct Frontmatter {
    title: Option<String>,
    digest: Option<String>,
    date: Option<String>,
    tags: Vec<String>,
}

pub(crate) fn parse_frontmatter(md: &str) -> Frontmatter {
    let mut fm = Frontmatter::default();
    let body = md.trim_start();
    if !body.starts_with("---") {
        return fm;
    }
    let rest = &body[3..];
    let end = rest.find("\n---").unwrap_or(rest.len());
    for line in rest[..end].lines() {
        let line = line.trim();
        // tags: ["a", "b", "c"]
        if line.starts_with("tags:") {
            fm.tags = parse_yaml_string_array(line);
            continue;
        }
        if let Some((k, v)) = line.split_once(':') {
            let k = k.trim();
            let v = v.trim().trim_matches('"');
            match k {
                "title" => fm.title = Some(v.to_owned()),
                "digest" | "description" => fm.digest = Some(v.to_owned()),
                "date" => fm.date = Some(v.to_owned()),
                _ => {}
            }
        }
    }
    fm
}

/// Parse `tags: ["a", "b"]` or `tags: [a, b]` into a Vec<String>.
fn parse_yaml_string_array(line: &str) -> Vec<String> {
    let Some(bracket_start) = line.find('[') else {
        return vec![];
    };
    let Some(bracket_end) = line.rfind(']') else {
        return vec![];
    };
    let inner = &line[bracket_start + 1..bracket_end];
    inner
        .split(',')
        .map(|s| s.trim().trim_matches('"').trim_matches('\'').to_owned())
        .filter(|s| !s.is_empty())
        .collect()
}

pub(crate) fn strip_frontmatter(md: &str) -> &str {
    let body = md.trim_start();
    if !body.starts_with("---") {
        return md;
    }
    let rest = &body[3..];
    if let Some(pos) = rest.find("\n---") {
        rest[pos + 4..].trim_start()
    } else {
        md
    }
}

/// Strip the standard banner+CTA footer that some articles already have in their Markdown.
fn strip_wechat_footer(body: &str) -> &str {
    // The footer always starts with a standalone `---` followed by a banner image line.
    // Walk backwards to find the last `---` that precedes a banner image URL.
    let lines: Vec<&str> = body.lines().collect();
    for i in (0..lines.len()).rev() {
        if lines[i].trim() == "---" {
            // Check if any line after this is the banner image
            let rest = &lines[i + 1..];
            if rest
                .iter()
                .any(|l| l.contains("mmbiz.qpic.cn") || l.contains("寻月者"))
            {
                // trim back to just before this `---`
                let cut = lines[..i]
                    .iter()
                    .rfind(|l| !l.trim().is_empty())
                    .map(|last| {
                        let pos = body.rfind(last).unwrap_or(body.len());
                        pos + last.len()
                    })
                    .unwrap_or(body.len());
                return body[..cut].trim_end();
            }
        }
    }
    body.trim_end()
}

fn first_non_empty_line(text: &str) -> &str {
    text.lines()
        .find(|l| !l.trim().is_empty() && !l.trim_start().starts_with('#'))
        .unwrap_or("")
        .trim()
}

// ── Markdown → WeChat HTML ────────────────────────────────────────────────────

fn md_to_wechat_html(md: &str, theme: &theme::Theme) -> String {
    let blocks = parse_blocks(md);
    let mut out = String::new();

    for block in &blocks {
        match block {
            MdBlock::Fence(name, props, body) => {
                out.push_str(&render_fence_block(name, props, body, theme))
            }
            MdBlock::Markdown(text) => out.push_str(&render_markdown_segment(text, theme)),
        }
    }

    out
}

#[derive(Debug)]
enum MdBlock<'a> {
    /// A `:::name` fenced block with optional YAML-like properties and body content.
    Fence(&'a str, Vec<(&'a str, &'a str)>, &'a str),
    /// Plain markdown text to be rendered as usual.
    Markdown(&'a str),
}

/// Split markdown into block segments. `:::name` fences and plain markdown.
fn parse_blocks(md: &str) -> Vec<MdBlock<'_>> {
    let mut blocks = Vec::new();
    let mut rest = md;

    while !rest.is_empty() {
        // Check if current position starts with `:::` at line start
        let is_line_start = rest.as_ptr() == md.as_ptr()
            || rest.as_bytes()[0] == b'\n'
            || (rest.len() > 1 && rest.as_bytes()[0] == b'\r' && rest.as_bytes()[1] == b'\n');
        let starts_fence =
            rest.starts_with(":::") || rest.starts_with("\n:::") || rest.starts_with("\r\n:::");

        if starts_fence {
            // Skip leading whitespace/newline to get to :::
            let _fence_start = if rest.starts_with("\r\n:::") {
                rest = &rest[2..];
                rest
            } else if rest.starts_with("\n:::") {
                rest = &rest[1..];
                rest
            } else {
                rest
            };

            // rest now starts with ":::"
            // Read block name
            let after_fence = &rest[3..]; // skip :::
            let name_end = after_fence.find('\n').unwrap_or(after_fence.len());
            let name_line = after_fence[..name_end].trim();
            let name = name_line.split_whitespace().next().unwrap_or("");

            // Find closing `:::`
            let inner_start = name_end + 1; // skip past \n
            let after_name = &after_fence[inner_start..];

            // Search for `\n:::` as closing marker
            let close_offset = after_name.find("\n:::");
            let (inner, remaining) = if let Some(off) = close_offset {
                let inner_text = &after_name[..off];
                // skip past \n:::\n
                let after_close = &after_name[off + 4..]; // skip \n:::
                let after_newline = after_close
                    .find('\n')
                    .map(|n| n + 1)
                    .unwrap_or(after_close.len());
                (inner_text, &after_close[after_newline..])
            } else {
                // No closing found, treat remaining as block body (maybe end of file)
                (after_name, "")
            };

            if !name.is_empty() {
                let (props, body) = split_fence_props(inner);
                blocks.push(MdBlock::Fence(name, props, body));
            }

            rest = remaining;
            continue;
        }

        // Regular markdown — find next `:::` fence or EOF
        if is_line_start {
            // Already at line start, just accumulate
        }
        let next_fence = rest.find("\n:::");
        if let Some(pos) = next_fence {
            let segment = &rest[..pos + 1]; // include the \n before :::
            let trimmed = segment.trim();
            if !trimmed.is_empty() {
                blocks.push(MdBlock::Markdown(trimmed));
            }
            rest = &rest[pos + 1..]; // point to ::: for next iteration
        } else {
            // No more fences, everything is markdown
            let trimmed = rest.trim();
            if !trimmed.is_empty() {
                blocks.push(MdBlock::Markdown(trimmed));
            }
            break;
        }
    }

    blocks
}

/// Parse key: value lines at the start of a fence body; rest is body content.
fn split_fence_props(inner: &str) -> (Vec<(&str, &str)>, &str) {
    let mut props = Vec::new();
    let mut body_start = 0;
    for line in inner.lines() {
        let trimmed = line.trim();
        if let Some((k, v)) = trimmed.split_once(':') {
            let k = k.trim();
            let v = v.trim().trim_matches('"');
            if !k.is_empty() && !k.contains(' ') && k.len() < 30 {
                props.push((k, v));
                body_start += line.len() + 1;
                continue;
            }
        }
        if trimmed.is_empty() {
            body_start += line.len() + 1;
            continue;
        }
        break;
    }
    let body = if body_start < inner.len() {
        &inner[body_start..]
    } else {
        ""
    };
    (props, body.trim_start())
}

// ── Fence block renderers ────────────────────────────────────────────────────

fn render_fence_block(
    name: &str,
    props: &[(&str, &str)],
    body: &str,
    theme: &theme::Theme,
) -> String {
    match name {
        "book-info" => render_book_info(props, theme),
        "intro" => render_intro(body, theme),
        "callout" => render_callout(props, body, theme),
        "steps" => render_steps(body, theme),
        "summary" => render_summary(body, theme),
        "figure" => render_figure(props, theme),
        "checklist" => render_checklist(body, theme),
        "cover" => render_cover(props, theme),
        "quote-card" => {
            let text = body.trim().to_owned();
            let source = props
                .iter()
                .find(|(k, _)| *k == "source")
                .map(|(_, v)| *v)
                .unwrap_or("");
            illustrate::render_illustration(
                &illustrate::IllustType::QuoteCard {
                    text,
                    source: source.to_owned(),
                },
                theme,
            )
        }
        "divider" => {
            let label = props
                .iter()
                .find(|(k, _)| *k == "label")
                .map(|(_, v)| *v)
                .unwrap_or("");
            illustrate::render_illustration(
                &illustrate::IllustType::Divider {
                    label: label.to_owned(),
                },
                theme,
            )
        }
        "concept-card" => {
            let number: u32 = props
                .iter()
                .find(|(k, _)| *k == "number")
                .and_then(|(_, v)| v.parse().ok())
                .unwrap_or(1);
            let title = body.lines().next().unwrap_or("").trim().to_owned();
            let desc = body
                .lines()
                .skip(1)
                .collect::<Vec<_>>()
                .join(
                    "
",
                )
                .trim()
                .to_owned();
            illustrate::render_illustration(
                &illustrate::IllustType::ConceptCard {
                    number,
                    title,
                    desc,
                },
                theme,
            )
        }
        "emotion-card" => {
            let mood = props
                .iter()
                .find(|(k, _)| *k == "mood")
                .map(|(_, v)| *v)
                .unwrap_or("think");
            illustrate::render_illustration(
                &illustrate::IllustType::EmotionCard {
                    mood: mood.to_owned(),
                    text: body.trim().to_owned(),
                },
                theme,
            )
        }
        "code" => {
            let lang = props
                .iter()
                .find(|(k, _)| *k == "lang")
                .map(|(_, v)| *v)
                .unwrap_or("");
            illustrate::render_code_block(lang, body.trim(), theme)
        }
        "timeline" => {
            let items: Vec<(String, String)> = body
                .lines()
                .filter(|l| l.trim().starts_with("- "))
                .filter_map(|l| {
                    let s = l.trim().trim_start_matches("- ").trim();
                    s.split_once(": ")
                        .map(|(d, t)| (d.to_owned(), t.to_owned()))
                })
                .collect();
            if items.is_empty() {
                render_generic_fence("timeline", body, theme)
            } else {
                illustrate::render_timeline(&items, theme)
            }
        }
        "comparison" => {
            let left = props
                .iter()
                .find(|(k, _)| *k == "left")
                .map(|(_, v)| *v)
                .unwrap_or("A");
            let right = props
                .iter()
                .find(|(k, _)| *k == "right")
                .map(|(_, v)| *v)
                .unwrap_or("B");
            let rows: Vec<(String, String)> = body
                .lines()
                .filter(|l| l.trim().starts_with("- "))
                .filter_map(|l| {
                    let s = l.trim().trim_start_matches("- ").trim();
                    s.split_once(" | ")
                        .map(|(a, b)| (a.to_owned(), b.to_owned()))
                })
                .collect();
            if rows.is_empty() {
                render_generic_fence("comparison", body, theme)
            } else {
                illustrate::render_comparison(left, right, &rows, theme)
            }
        }
        "tip" => {
            let icon = props
                .iter()
                .find(|(k, _)| *k == "icon")
                .map(|(_, v)| *v)
                .unwrap_or("");
            illustrate::render_tip(icon, body.trim(), theme)
        }
        _ => {
            // Unknown block — render as a styled container
            render_generic_fence(name, body, theme)
        }
    }
}

fn render_book_info(props: &[(&str, &str)], theme: &theme::Theme) -> String {
    let get = |key: &str| -> &str {
        props
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| *v)
            .unwrap_or("")
    };
    let title = get("title");
    let author = get("author");
    let cover = get("cover");
    let publisher = get("publisher");
    let rating = get("rating");
    let has_cover = !cover.is_empty();

    let mut html = String::new();
    html.push_str(&format!(
        "<section style=\"margin: 24px 0; background: {}; border: 1px solid #e8e8e8; border-radius: 6px; overflow: hidden;\">\n",
        theme.block_bg
    ));
    html.push_str("<table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\"><tr>\n");

    if has_cover {
        html.push_str(&format!(
            "<td style=\"width:90px;padding:16px;vertical-align:top;\"><img src=\"{cover}\" style=\"width:90px;height:auto;border-radius:4px;box-shadow:0 2px 8px rgba(0,0,0,0.12);\" /></td>\n"
        ));
    }
    html.push_str("<td style=\"padding:16px;vertical-align:middle;\">\n");
    html.push_str(&format!(
        "<p style=\"margin:0 0 6px;font-size:16px;font-weight:bold;color:{};\">《{title}》</p>\n",
        theme.heading_color
    ));
    if !author.is_empty() {
        html.push_str(&format!(
            "<p style=\"margin:0 0 4px;font-size:13px;color:{};\">{author} 著</p>\n",
            theme.text_muted
        ));
    }
    if !publisher.is_empty() || !rating.is_empty() {
        let pub_str = if rating.is_empty() {
            publisher.to_owned()
        } else {
            format!("{publisher} | 豆瓣 {rating}")
        };
        html.push_str(&format!(
            "<p style=\"margin:0;font-size:12px;color:{};\">{pub_str}</p>\n",
            theme.text_muted
        ));
    }
    html.push_str("</td>\n");
    html.push_str("</tr></table>\n");
    html.push_str("</section>\n\n");
    html
}

fn render_intro(body: &str, theme: &theme::Theme) -> String {
    format!(
        "<section style=\"margin: 24px 0; padding: 20px 24px; background: {}; border-left: 4px solid {}; font-size: 16px; color: {}; line-height: 1.9; letter-spacing: 0.5px;\">\n{}\n</section>\n\n",
        theme.block_bg,
        theme.accent,
        theme.text_color,
        inline_md(body.trim(), theme)
    )
}

fn render_callout(props: &[(&str, &str)], body: &str, theme: &theme::Theme) -> String {
    let label = props
        .iter()
        .find(|(k, _)| *k == "label")
        .map(|(_, v)| *v)
        .unwrap_or("重点");
    format!(
        "<section style=\"margin: 24px 0;\"><table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\"><tr>\n<td style=\"background:{};color:#fff;font-weight:bold;font-size:13px;padding:12px 16px;white-space:nowrap;letter-spacing:2px;vertical-align:top;\">{label}</td>\n<td style=\"background:{};border:1px solid {};border-left:none;padding:14px 18px;font-size:15px;line-height:1.85;color:{};\">{}</td>\n</tr></table></section>\n\n",
        theme.accent,
        theme.block_bg,
        theme.accent,
        theme.heading_color,
        inline_md(body.trim(), theme)
    )
}

fn render_steps(body: &str, theme: &theme::Theme) -> String {
    let items: Vec<&str> = body
        .lines()
        .filter(|l| l.trim().starts_with(|c: char| c.is_ascii_digit()) && l.trim().contains(". "))
        .filter_map(|l| l.trim().split_once(". ").map(|(_, rest)| rest))
        .collect();

    if items.is_empty() {
        return render_generic_fence("steps", body, theme);
    }

    let count = items.len();
    let mut html = String::new();
    html.push_str("<section style=\"margin: 24px 0;\"><table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\"><tr>\n");

    let pct = 100usize.div_ceil(count);
    for (i, item) in items.iter().enumerate() {
        if i > 0 {
            html.push_str("<td style=\"width:8px;\"></td>\n");
        }
        html.push_str(&format!(
            "<td style=\"width:{pct}%;background:#fff;border:1px solid #e8e8e8;padding:14px 12px;vertical-align:top;\">\n<section style=\"display:inline-block;width:24px;height:24px;background:{};color:#fff;font-weight:bold;text-align:center;line-height:24px;border-radius:50%;font-size:13px;margin-bottom:8px;\">{}</section>\n<p style=\"margin:0;font-size:13px;color:{};line-height:1.7;\">{}</p>\n</td>\n",
            theme.accent, i + 1, theme.text_color, inline_md(item, theme),
        ));
    }
    html.push_str("</tr></table></section>\n\n");
    html
}

fn render_summary(body: &str, theme: &theme::Theme) -> String {
    format!(
        "<section style=\"margin: 24px 0;\"><table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\"><tr>\n<td style=\"background:{};color:#fff;font-weight:bold;font-size:13px;padding:10px 14px;white-space:nowrap;letter-spacing:1px;vertical-align:top;\">总 结</td>\n<td style=\"background:#fff;border:1px solid {};border-left:none;padding:12px 16px;font-size:14px;line-height:1.8;color:{};\">{}</td>\n</tr></table></section>\n\n",
        theme.accent,
        theme.accent,
        theme.heading_color,
        inline_md(body.trim(), theme)
    )
}

fn render_figure(props: &[(&str, &str)], theme: &theme::Theme) -> String {
    let image = props
        .iter()
        .find(|(k, _)| *k == "image")
        .map(|(_, v)| *v)
        .unwrap_or("");
    let caption = props
        .iter()
        .find(|(k, _)| *k == "caption")
        .map(|(_, v)| *v)
        .unwrap_or("");
    if image.is_empty() {
        return String::new();
    }
    let cap_html = if caption.is_empty() {
        String::new()
    } else {
        format!(
            "<p style=\"margin:0;padding:10px 14px;background:{};color:{};font-size:12px;text-align:center;\">{caption}</p>",
            theme.block_bg, theme.text_muted
        )
    };
    format!(
        "<section style=\"margin: 24px 0;\"><section style=\"border:2px solid #e8e8e8;padding:0;background:{};\">\n<img src=\"{image}\" style=\"display:block;width:100%;height:auto;\" />\n{cap_html}</section></section>\n\n",
        theme.block_bg
    )
}

fn render_checklist(body: &str, theme: &theme::Theme) -> String {
    let items: Vec<&str> = body
        .lines()
        .filter(|l| l.trim().starts_with("- [") || l.trim().starts_with("- ["))
        .collect();
    if items.is_empty() {
        return render_generic_fence("checklist", body, theme);
    }
    let mut html = String::new();
    html.push_str(&format!(
        "<section style=\"margin:18px 0;\"><section style=\"background:{};border:1px solid #e8e8e8;padding:18px 20px;\"><table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\">\n",
        theme.block_bg
    ));
    let half = items.len().div_ceil(2);
    for row in 0..half {
        html.push_str("<tr>\n");
        for col in 0..2 {
            let idx = if col == 0 { row } else { row + half };
            if idx < items.len() {
                let item = items[idx]
                    .trim()
                    .trim_start_matches("- [")
                    .trim_start_matches("- [")
                    .trim_end_matches(']');
                let rest = if item.starts_with('x') || item.starts_with("x ") {
                    let content = item[1..].trim();
                    format!(
                        "<span style=\"color:{};font-weight:bold;\">✔</span>&nbsp;&nbsp;{content}",
                        theme.accent
                    )
                } else {
                    let content = item[1..].trim();
                    format!(
                        "<span style=\"color:#ccc;font-weight:bold;\">○</span>&nbsp;&nbsp;{content}"
                    )
                };
                html.push_str(&format!(
                    "<td style=\"width:50%;padding:6px 0;font-size:14px;color:{};vertical-align:top;\">{rest}</td>\n",
                    theme.text_color
                ));
            } else {
                html.push_str("<td style=\"width:50%;\"></td>\n");
            }
        }
        html.push_str("</tr>\n");
    }
    html.push_str("</table></section></section>\n\n");
    html
}

fn render_cover(props: &[(&str, &str)], theme: &theme::Theme) -> String {
    let get = |key: &str| -> &str {
        props
            .iter()
            .find(|(k, _)| *k == key)
            .map(|(_, v)| *v)
            .unwrap_or("")
    };
    let title = get("title");
    let subtitle = get("subtitle");
    format!(
        "<section style=\"margin:0;background:{};padding:48px 24px 36px;color:#fff;\">\n<section style=\"display:inline-block;background:#fff;color:{};font-size:11px;font-weight:bold;letter-spacing:2px;padding:4px 10px;margin-bottom:18px;\">READING · NOTES</section>\n<h1 style=\"margin:0 0 8px;font-size:28px;font-weight:900;line-height:1.2;color:#fff;\">{title}</h1>\n<p style=\"margin:8px 0 0;font-size:14px;color:{};\">{subtitle}</p>\n</section>\n\n",
        theme.accent, theme.accent, theme.text_muted
    )
}

fn render_generic_fence(_name: &str, body: &str, theme: &theme::Theme) -> String {
    format!(
        "<section style=\"margin: 18px 0; padding: 16px 20px; background: {}; border: 1px solid #e8e8e8; border-radius: 4px;\">\n{}\n</section>\n\n",
        theme.block_bg,
        inline_md(body.trim(), theme)
    )
}

// ── Plain markdown segment renderer ───────────────────────────────────────────

fn render_markdown_segment(md: &str, theme: &theme::Theme) -> String {
    let mut out = String::new();
    let mut in_blockquote = false;
    let mut blockquote_buf = String::new();

    for line in md.lines() {
        if let Some(rest) = line
            .strip_prefix("> ")
            .or_else(|| if line == ">" { Some("") } else { None })
        {
            in_blockquote = true;
            if !rest.is_empty() {
                if !blockquote_buf.is_empty() {
                    blockquote_buf.push('\n');
                }
                blockquote_buf.push_str(rest);
            }
            continue;
        }
        if in_blockquote {
            out.push_str(&render_blockquote(&blockquote_buf, theme));
            blockquote_buf.clear();
            in_blockquote = false;
        }

        if line.trim() == "---" || line.trim() == "***" || line.trim() == "___" {
            out.push_str(
                "<hr style=\"border: none; border-top: 1px solid #eee; margin: 2em 0;\" />\n\n",
            );
            continue;
        }

        if let Some(rest) = line.strip_prefix("### ") {
            out.push_str(&render_h3(rest, theme));
            continue;
        }
        if let Some(rest) = line.strip_prefix("## ") {
            out.push_str(&render_h2(rest, theme));
            continue;
        }
        if let Some(rest) = line.strip_prefix("# ") {
            out.push_str(&render_h2(rest, theme));
            continue;
        }

        if line.trim().is_empty() {
            continue;
        }

        let trimmed = line.trim();
        if trimmed.starts_with("![") {
            let chars: Vec<char> = trimmed.chars().collect();
            if let Some((alt, url, _)) = parse_image(&chars) {
                out.push_str(&format!(
                    "<p style=\"margin: 1.5em 0; text-align: center;\"><img src=\"{url}\" alt=\"{alt}\" style=\"max-width: 100%; display: block; margin: 0 auto;\" /></p>\n\n"
                ));
                continue;
            }
        }

        out.push_str(&render_p(line, theme));
    }

    if in_blockquote {
        out.push_str(&render_blockquote(&blockquote_buf, theme));
    }

    out
}

fn inline_md(text: &str, theme: &theme::Theme) -> String {
    // **bold**, *italic*, `code` — applied in order
    let mut s = String::new();
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        if chars[i] == '`' {
            let end = chars[i + 1..].iter().position(|&c| c == '`');
            if let Some(rel) = end {
                let code: String = chars[i + 1..i + 1 + rel].iter().collect();
                s.push_str(&format!(
                    "<code style=\"background:#f5f5f5;padding:2px 4px;border-radius:3px;font-size:14px;\">{}</code>",
                    html_escape(&code)
                ));
                i += rel + 2;
                continue;
            }
        }
        if chars[i] == '*' && i + 1 < chars.len() && chars[i + 1] == '*' {
            let end = chars[i + 2..].windows(2).position(|w| w == ['*', '*']);
            if let Some(rel) = end {
                let inner: String = chars[i + 2..i + 2 + rel].iter().collect();
                s.push_str(&format!(
                    "<strong style=\"color: {};\">{}</strong>",
                    theme.heading_color,
                    inline_md(&inner, theme)
                ));
                i += rel + 4;
                continue;
            }
        }
        if chars[i] == '*' {
            let end = chars[i + 1..].iter().position(|&c| c == '*');
            if let Some(rel) = end {
                let inner: String = chars[i + 1..i + 1 + rel].iter().collect();
                s.push_str(&format!("<em>{}</em>", inline_md(&inner, theme)));
                i += rel + 2;
                continue;
            }
        }
        // image ![alt](url)
        if chars[i] == '!'
            && i + 1 < chars.len()
            && chars[i + 1] == '['
            && let Some((alt, url, consumed)) = parse_image(&chars[i..])
        {
            s.push_str(&format!(
                "<p style=\"margin: 1.5em 0; text-align: center;\"><img src=\"{url}\" alt=\"{alt}\" style=\"max-width: 100%; display: block; margin: 0 auto;\" /></p>"
            ));
            i += consumed;
            continue;
        }
        s.push(chars[i]);
        i += 1;
    }
    s
}

fn parse_image(chars: &[char]) -> Option<(String, String, usize)> {
    // ![alt](url)
    if chars.len() < 5 || chars[0] != '!' || chars[1] != '[' {
        return None;
    }
    let alt_end = chars[2..].iter().position(|&c| c == ']')?;
    let alt: String = chars[2..2 + alt_end].iter().collect();
    let rest = &chars[2 + alt_end + 1..];
    if rest.first() != Some(&'(') {
        return None;
    }
    let url_end = rest[1..].iter().position(|&c| c == ')')?;
    let url: String = rest[1..1 + url_end].iter().collect();
    Some((alt, url, 2 + alt_end + 1 + 1 + url_end + 1))
}

fn html_escape(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

fn render_h2(text: &str, theme: &theme::Theme) -> String {
    format!(
        "<h2 style=\"font-size: 18px; font-weight: bold; color: {}; margin: 2em 0 0.8em; padding: 0 0 8px 12px; border-left: 4px solid {}; border-bottom: 1px solid #f0f0f0; letter-spacing: 1px;\">{}</h2>\n\n",
        theme.heading_color,
        theme.heading_border,
        inline_md(text, theme)
    )
}

fn render_h3(text: &str, theme: &theme::Theme) -> String {
    format!(
        "<h3 style=\"font-size: 16px; font-weight: bold; color: {}; margin: 1.5em 0 0.6em; letter-spacing: 0.5px;\">{}</h3>\n\n",
        theme.heading_color,
        inline_md(text, theme)
    )
}

fn render_p(text: &str, theme: &theme::Theme) -> String {
    format!(
        "<p style=\"margin: 1.2em 0; color: {}; font-size: 15px; line-height: 1.85; letter-spacing: 0.3px;\">{}</p>\n\n",
        theme.text_color,
        inline_md(text, theme)
    )
}

fn render_blockquote(text: &str, theme: &theme::Theme) -> String {
    format!(
        "<blockquote style=\"margin: 1.8em 0; padding: 18px 20px 18px 24px; background: {}; border-left: 4px solid {}; color: {}; font-size: 15px; line-height: 1.85; letter-spacing: 0.3px;\">{}</blockquote>\n\n",
        theme.block_bg,
        theme.accent,
        theme.text_muted,
        inline_md(text, theme)
    )
}

pub(crate) fn find_chrome() -> Option<String> {
    let candidates = [
        "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
        "/Applications/Chromium.app/Contents/MacOS/Chromium",
        "google-chrome",
        "google-chrome-stable",
        "chromium",
        "chromium-browser",
    ];
    for c in &candidates {
        if c.starts_with('/') {
            if std::path::Path::new(c).exists() {
                return Some(c.to_string());
            }
        } else if std::process::Command::new("which")
            .arg(c)
            .output()
            .map(|o| o.status.success())
            .unwrap_or(false)
        {
            return Some(c.to_string());
        }
    }
    None
}

fn wrap_wechat_html(body: &str, theme: &theme::Theme, footer_cfg: &footer::FooterConfig) -> String {
    let ending = footer::render_footer(footer_cfg, theme);
    format!(
        "<section style=\"{}\">\n\n{body}\n\n{ending}\n\n</section>\n",
        theme.section_style()
    )
}

// ── draft.json builder ────────────────────────────────────────────────────────

fn build_draft_json(
    title: &str,
    author: &str,
    digest: &str,
    content: &str,
    thumb_media_id: &str,
) -> String {
    // Hand-build JSON to keep zero deps; escape strings.
    format!(
        "{{\n  \"articles\": [\n    {{\n      \"title\": \"{}\",\n      \"author\": \"{}\",\n      \"digest\": \"{}\",\n      \"content\": \"{}\",\n      \"thumb_media_id\": \"{}\",\n      \"show_cover_pic\": 0,\n      \"content_source_url\": \"\"\n    }}\n  ]\n}}\n",
        escape_json(title),
        escape_json(author),
        escape_json(digest),
        escape_json(content),
        escape_json(thumb_media_id),
    )
}

// ── JSON helpers ─────────────────────────────────────────────────────────────

pub(crate) fn escape_json(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            value => escaped.push(value),
        }
    }
    escaped
}

pub(crate) fn extract_json_string(line: &str, name: &str) -> Option<String> {
    let marker = format!("\"{name}\":\"");
    let start = line.find(&marker)? + marker.len();
    let mut output = String::new();
    let mut chars = line[start..].chars();

    while let Some(ch) = chars.next() {
        match ch {
            '"' => return Some(output),
            '\\' => match chars.next()? {
                '"' => output.push('"'),
                '\\' => output.push('\\'),
                'n' => output.push('\n'),
                'r' => output.push('\r'),
                't' => output.push('\t'),
                other => output.push(other),
            },
            other => output.push(other),
        }
    }

    None
}

pub(crate) fn extract_json_optional_string(line: &str, name: &str) -> Option<String> {
    if line.contains(&format!("\"{name}\":null")) {
        None
    } else {
        extract_json_string(line, name)
    }
}

pub(crate) fn extract_json_optional_u64(line: &str, name: &str) -> Option<u64> {
    let marker = format!("\"{name}\":");
    let start = line.find(&marker)? + marker.len();
    let value = line[start..].split([',', '}']).next().unwrap_or("").trim();
    if value == "null" {
        None
    } else {
        value.parse().ok()
    }
}

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn parses_status_with_vault() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "--vault".to_owned(),
            "/tmp/vault".to_owned(),
            "status".to_owned(),
        ])?;

        assert_eq!(options.vault, PathBuf::from("/tmp/vault"));
        assert_eq!(options.command, Command::Status);
        Ok(())
    }

    #[test]
    fn parses_json_flag() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "--vault".to_owned(),
            "/tmp/vault".to_owned(),
            "--json".to_owned(),
            "status".to_owned(),
        ])?;

        assert!(options.json);
        Ok(())
    }

    #[test]
    fn json_output_wraps_text() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("json-status")?;
        let options = Options {
            vault: root.clone(),
            command: Command::Status,
            json: true,
            config: None,
        };

        let output = run(&options)?;

        assert!(output.starts_with("{\"output\":\""));
        assert!(output.ends_with('}'));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn status_lists_markdown_files_by_stage() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("status")?;
        create_file(&root.join("Articles/drafts/a.md"), "")?;
        create_file(&root.join("Articles/drafts/a.html"), "")?;
        create_file(&root.join("Articles/published/z.md"), "")?;

        let output = status(&root)?;

        assert!(output.contains("-- drafts --"));
        assert!(output.contains("  a.md"));
        assert!(output.contains("-- ready --"));
        assert!(output.contains("  (empty)"));
        assert!(output.contains("-- published --"));
        assert!(output.contains("  z.md"));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn check_reports_missing_bundle_parts() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("check")?;
        create_file(&root.join("Articles/drafts/demo.md"), "")?;
        create_file(&root.join("Articles/drafts/demo.html"), "")?;

        let output = check_article(&root, Path::new("Articles/drafts/demo.md"))?;

        assert!(output.contains("markdown: ok"));
        assert!(output.contains("html: ok"));
        assert!(output.contains("draft_json: missing"));
        assert!(output.contains("publishable: no"));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn check_reports_publishable_bundle() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("publishable")?;
        create_file(&root.join("Articles/ready/demo.md"), "")?;
        create_file(&root.join("Articles/ready/demo.html"), "")?;
        create_file(&root.join("Articles/ready/demo.draft.json"), "{}")?;

        let output = check_article(&root, Path::new("Articles/ready/demo.md"))?;

        assert!(output.contains("publishable: yes"));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn parses_radar_add_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "radar".to_owned(),
            "add".to_owned(),
            "--platform".to_owned(),
            "xiaohongshu".to_owned(),
            "--keyword".to_owned(),
            "AI写作".to_owned(),
            "--title".to_owned(),
            "我的标题".to_owned(),
            "--likes".to_owned(),
            "42".to_owned(),
        ])?;

        let Command::Radar(RadarCommand::Add(sample)) = options.command else {
            panic!("expected radar add");
        };
        assert_eq!(sample.platform, "xiaohongshu");
        assert_eq!(sample.keyword, "AI写作");
        assert_eq!(sample.title, "我的标题");
        assert_eq!(sample.likes, Some(42));
        Ok(())
    }

    #[test]
    fn radar_add_and_list_samples() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("radar")?;
        add_trend_sample(
            &root,
            &TrendSample {
                platform: "wechat".to_owned(),
                keyword: "AI写作".to_owned(),
                title: "一个值得参考的标题".to_owned(),
                url: Some("https://example.com/post".to_owned()),
                author: Some("demo".to_owned()),
                likes: Some(100),
                collects: Some(50),
                comments: Some(8),
                source: "manual".to_owned(),
            },
        )?;

        let output = list_trend_samples(&root, &Some("wechat".to_owned()), &None)?;

        assert!(output.contains("[wechat] AI写作 | 一个值得参考的标题"));
        assert!(output.contains("likes=100"));
        assert!(output.contains("collects=50"));
        assert!(output.contains("comments=8"));
        assert!(output.contains("https://example.com/post"));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn radar_list_filters_by_platform() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("radar-filter")?;
        add_sample(&root, "wechat", "公众号标题")?;
        add_sample(&root, "xiaohongshu", "小红书标题")?;

        let output = list_trend_samples(&root, &Some("xiaohongshu".to_owned()), &None)?;

        assert!(output.contains("小红书标题"));
        assert!(!output.contains("公众号标题"));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn trend_sample_json_roundtrip_escapes_text() {
        let sample = TrendSample {
            platform: "wechat".to_owned(),
            keyword: "AI\"写作".to_owned(),
            title: "第一行\n第二行".to_owned(),
            url: None,
            author: None,
            likes: None,
            collects: None,
            comments: None,
            source: "manual".to_owned(),
        };

        let line = sample.to_json_line();
        let parsed = TrendSample::from_json_line(&line).expect("valid json line");

        assert_eq!(parsed.keyword, sample.keyword);
        assert_eq!(parsed.title, sample.title);
    }

    // ── config tests ──────────────────────────────────────────────────────────

    #[test]
    fn config_parses_vault_root() {
        let toml = r#"
[vault]
root = "/my/vault"

[wechat]
appid = "wx123"
"#;
        let cfg = Config::from_toml(toml);
        assert_eq!(cfg.vault_root, Some(PathBuf::from("/my/vault")));
        assert_eq!(cfg.wechat_appid.as_deref(), Some("wx123"));
    }

    #[test]
    fn config_overrides_vault_in_options() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("config-vault")?;
        let config_path = root.join("moonpub.toml");
        let vault_path = root.join("my-vault");
        fs::create_dir_all(&vault_path)?;
        fs::write(
            &config_path,
            format!("[vault]\nroot = \"{}\"\n", vault_path.display()),
        )?;

        let options = Options::parse([
            "--config".to_owned(),
            config_path.to_str().unwrap().to_owned(),
            "status".to_owned(),
        ])?;

        assert_eq!(options.vault, vault_path);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    // ── CSV import tests ──────────────────────────────────────────────────────

    #[test]
    fn csv_import_basic() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("csv-import")?;
        let csv = root.join("trends.csv");
        fs::write(
            &csv,
            "platform,keyword,title,likes,source\nwechat,AI写作,标题一,100,csv\nwechat,AI写作,标题二,200,csv\n",
        )?;

        let msg = import_csv(&root, &csv, None)?;
        assert!(msg.contains("imported 2 samples"));

        let output = list_trend_samples(&root, &Some("wechat".to_owned()), &None)?;
        assert!(output.contains("标题一"));
        assert!(output.contains("标题二"));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn csv_import_uses_default_platform() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("csv-default-platform")?;
        let csv = root.join("trends.csv");
        fs::write(&csv, "keyword,title\nAI写作,一篇好文章\n")?;

        import_csv(&root, &csv, Some("xiaohongshu"))?;

        let output = list_trend_samples(&root, &Some("xiaohongshu".to_owned()), &None)?;
        assert!(output.contains("一篇好文章"));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn csv_import_quoted_fields() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("csv-quoted")?;
        let csv = root.join("trends.csv");
        fs::write(&csv, "platform,keyword,title\nwechat,AI,\"标题含,逗号\"\n")?;

        import_csv(&root, &csv, None)?;

        let output = list_trend_samples(&root, &Some("wechat".to_owned()), &None)?;
        assert!(output.contains("标题含,逗号"));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn csv_parse_row_handles_quoted_commas() {
        let row = r#""hello,world",foo,"bar""#;
        let fields = parse_csv_row(row);
        assert_eq!(fields, vec!["hello,world", "foo", "bar"]);
    }

    // ── analyze tests ─────────────────────────────────────────────────────────

    #[test]
    fn analyze_ranks_by_engagement() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("analyze")?;
        add_trend_sample(
            &root,
            &TrendSample {
                platform: "wechat".to_owned(),
                keyword: "AI写作".to_owned(),
                title: "高互动标题".to_owned(),
                url: None,
                author: None,
                likes: Some(500),
                collects: Some(200),
                comments: Some(50),
                source: "manual".to_owned(),
            },
        )?;
        add_trend_sample(
            &root,
            &TrendSample {
                platform: "wechat".to_owned(),
                keyword: "AI写作".to_owned(),
                title: "低互动标题".to_owned(),
                url: None,
                author: None,
                likes: Some(5),
                collects: None,
                comments: None,
                source: "manual".to_owned(),
            },
        )?;

        let article = root.join("demo.md");
        create_file(&article, "# AI写作技巧\n这篇文章讨论AI写作。")?;

        let output = analyze_article(&root, &article, "wechat", 10)?;

        let high_pos = output.find("高互动标题").expect("高互动标题 not found");
        let low_pos = output.find("低互动标题").expect("低互动标题 not found");
        assert!(high_pos < low_pos, "高互动应排在低互动之前");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn analyze_filters_by_platform() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("analyze-platform")?;
        add_sample(&root, "wechat", "公众号专属标题")?;
        add_sample(&root, "xiaohongshu", "小红书专属标题")?;

        let article = root.join("demo.md");
        create_file(&article, "# AI写作")?;

        let output = analyze_article(&root, &article, "wechat", 10)?;

        assert!(output.contains("公众号专属标题"));
        assert!(!output.contains("小红书专属标题"));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn parses_radar_import_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "radar".to_owned(),
            "import".to_owned(),
            "trends.csv".to_owned(),
            "--platform".to_owned(),
            "wechat".to_owned(),
        ])?;

        let Command::Radar(RadarCommand::Import { path, platform }) = options.command else {
            panic!("expected radar import");
        };
        assert_eq!(path, PathBuf::from("trends.csv"));
        assert_eq!(platform.as_deref(), Some("wechat"));
        Ok(())
    }

    #[test]
    fn parses_radar_analyze_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "radar".to_owned(),
            "analyze".to_owned(),
            "demo.md".to_owned(),
            "--platform".to_owned(),
            "wechat".to_owned(),
            "--top".to_owned(),
            "5".to_owned(),
        ])?;

        let Command::Radar(RadarCommand::Analyze {
            article,
            platform,
            top,
        }) = options.command
        else {
            panic!("expected radar analyze");
        };
        assert_eq!(article, PathBuf::from("demo.md"));
        assert_eq!(platform, "wechat");
        assert_eq!(top, 5);
        Ok(())
    }

    // ── helpers ───────────────────────────────────────────────────────────────

    fn temp_root(name: &str) -> io::Result<PathBuf> {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or(0);
        let root = std::env::temp_dir().join(format!("moonpub-{name}-{nanos}"));
        fs::create_dir_all(&root)?;
        Ok(root)
    }

    fn create_file(path: &Path, content: &str) -> io::Result<()> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(path, content)
    }

    fn add_sample(root: &Path, platform: &str, title: &str) -> Result<(), AppError> {
        add_trend_sample(
            root,
            &TrendSample {
                platform: platform.to_owned(),
                keyword: "AI".to_owned(),
                title: title.to_owned(),
                url: None,
                author: None,
                likes: None,
                collects: None,
                comments: None,
                source: "manual".to_owned(),
            },
        )?;
        Ok(())
    }

    // ── render tests ──────────────────────────────────────────────────────────

    #[test]
    fn render_produces_html_and_draft_json() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("render-basic")?;
        let md_path = root.join("demo.md");
        create_file(
            &md_path,
            "---\ntitle: 测试文章标题\ndigest: 这是摘要\n---\n\n正文第一段。\n",
        )?;

        render_article(&root, &md_path, "寻月隐君", "thumb123", "default", None, "")?;

        let html = fs::read_to_string(root.join("demo.html"))?;
        assert!(html.contains("<section"), "缺少 section 容器");
        assert!(html.contains("正文第一段"), "正文未渲染");

        let json_str = fs::read_to_string(root.join("demo.draft.json"))?;
        assert!(json_str.contains("\"title\": \"测试文章标题\""));
        assert!(json_str.contains("\"author\": \"寻月隐君\""));
        assert!(json_str.contains("\"digest\": \"这是摘要\""));
        assert!(json_str.contains("\"thumb_media_id\": \"thumb123\""));
        assert!(json_str.contains("\"show_cover_pic\": 0"));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn render_digest_falls_back_to_first_paragraph() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("render-digest")?;
        let md_path = root.join("article.md");
        create_file(
            &md_path,
            "---\ntitle: 标题\n---\n\n## 一级标题\n\n第一段文字内容。\n",
        )?;

        render_article(&root, &md_path, "作者", "", "default", None, "")?;

        let json_str = fs::read_to_string(root.join("article.draft.json"))?;
        assert!(json_str.contains("第一段文字内容"), "摘要应取自第一段正文");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn render_markdown_elements() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("render-elements")?;
        let md_path = root.join("elem.md");
        create_file(
            &md_path,
            "---\ntitle: T\n---\n\n## 章节标题\n\n**粗体** 和 *斜体* 和 `代码`。\n\n> 引用文字\n\n---\n",
        )?;

        render_article(&root, &md_path, "a", "", "default", None, "")?;

        let html = fs::read_to_string(root.join("elem.html"))?;
        assert!(html.contains("<h2 "), "h2 未渲染");
        assert!(html.contains("<strong "), "strong 未渲染");
        assert!(html.contains("<em>"), "em 未渲染");
        assert!(html.contains("<code "), "code 未渲染");
        assert!(html.contains("<blockquote "), "blockquote 未渲染");
        assert!(html.contains("<hr "), "hr 未渲染");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn render_uses_config_author_and_thumb() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("render-config")?;
        let cfg_path = root.join("moonpub.toml");
        create_file(
            &cfg_path,
            "[wechat]\nauthor = \"从配置读\"\nthumb_media_id = \"cfg_thumb\"\n",
        )?;
        let md_path = root.join("article.md");
        create_file(&md_path, "---\ntitle: T\n---\n\n正文。\n")?;

        let options = Options::parse([
            "--config".to_owned(),
            cfg_path.to_str().unwrap().to_owned(),
            "--vault".to_owned(),
            root.to_str().unwrap().to_owned(),
            "render".to_owned(),
            md_path.to_str().unwrap().to_owned(),
        ])?;
        run(&options)?;

        let json_str = fs::read_to_string(root.join("article.draft.json"))?;
        assert!(json_str.contains("\"author\": \"从配置读\""));
        assert!(json_str.contains("\"thumb_media_id\": \"cfg_thumb\""));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn render_cli_flag_overrides_config() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("render-override")?;
        let cfg_path = root.join("moonpub.toml");
        create_file(
            &cfg_path,
            "[wechat]\nauthor = \"配置作者\"\nthumb_media_id = \"cfg_thumb\"\n",
        )?;
        let md_path = root.join("article.md");
        create_file(&md_path, "---\ntitle: T\n---\n\n正文。\n")?;

        let options = Options::parse([
            "--config".to_owned(),
            cfg_path.to_str().unwrap().to_owned(),
            "--vault".to_owned(),
            root.to_str().unwrap().to_owned(),
            "render".to_owned(),
            md_path.to_str().unwrap().to_owned(),
            "--author".to_owned(),
            "命令行作者".to_owned(),
            "--thumb".to_owned(),
            "cli_thumb".to_owned(),
        ])?;
        run(&options)?;

        let json_str = fs::read_to_string(root.join("article.draft.json"))?;
        assert!(json_str.contains("\"author\": \"命令行作者\""));
        assert!(json_str.contains("\"thumb_media_id\": \"cli_thumb\""));

        fs::remove_dir_all(root)?;
        Ok(())
    }

    // ── push tests ────────────────────────────────────────────────────────────

    #[test]
    fn push_fails_without_draft_json() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("push-no-draft")?;
        let md = root.join("Articles/drafts/demo.md");
        create_file(&md, "---\ntitle: T\n---\n\n正文。\n")?;

        let cfg = Config::default();
        let err = push_article(&root, &md, false, &cfg).unwrap_err();
        assert!(
            matches!(err, AppError::NoDraftJson(_)),
            "expected NoDraftJson, got: {err}"
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn push_auto_render_creates_draft_json() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("push-auto-render")?;
        let md = root.join("Articles/drafts/demo.md");
        create_file(&md, "---\ntitle: 自动渲染测试\n---\n\n正文段落。\n")?;

        let cfg = Config {
            wechat_author: Some("寻月隐君".to_owned()),
            wechat_thumb_media_id: Some("thumb_abc".to_owned()),
            ..Config::default()
        };
        // --render flag triggers render; push will then fail at md2wechat (no real credentials),
        // but draft.json must exist before that failure.
        let _ = push_article(&root, &md, true, &cfg);

        assert!(
            root.join("Articles/drafts/demo.draft.json").exists(),
            "draft.json should be created by auto-render"
        );

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn extract_ip_from_wechat_error() {
        let msg = "create draft: get access_token error : errcode=40164 , errormsg=invalid ip 1.2.3.4 ipv6";
        assert_eq!(extract_ip_from_message(msg).as_deref(), Some("1.2.3.4"));
    }

    #[test]
    fn dir_stage_identifies_stages() {
        assert_eq!(
            dir_stage(Path::new("/vault/Articles/drafts")),
            Some("drafts")
        );
        assert_eq!(dir_stage(Path::new("/vault/Articles/ready")), Some("ready"));
        assert_eq!(
            dir_stage(Path::new("/vault/Articles/published")),
            Some("published")
        );
        assert_eq!(dir_stage(Path::new("/vault/Articles")), None);
    }

    #[test]
    fn parses_push_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "--vault".to_owned(),
            "/tmp/vault".to_owned(),
            "push".to_owned(),
            "Articles/ready/demo.md".to_owned(),
            "--render".to_owned(),
        ])?;
        let Command::Push {
            article,
            auto_render,
        } = options.command
        else {
            panic!("expected Push");
        };
        assert_eq!(article, PathBuf::from("Articles/ready/demo.md"));
        assert!(auto_render);
        Ok(())
    }

    // ── export tests ──────────────────────────────────────────────────────────

    #[test]
    fn export_creates_zola_file() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("export-basic")?;
        let blog = root.join("blog");
        let md_path = root.join("Articles/published/demo.md");
        create_file(
            &md_path,
            "---\ntitle: \"我的文章\"\ndate: 2026-06-10\ntags: [\"Rust\", \"AI\"]\n---\n\n正文段落。\n\n## 章节\n\n更多内容。\n",
        )?;

        export_article(&root, &md_path, &blog)?;

        let out = blog.join("content/2026-06-10-demo.md");
        assert!(out.exists(), "Zola 文件应已创建");

        let content = fs::read_to_string(&out)?;
        assert!(content.starts_with("+++"), "应以 +++ 开头");
        assert!(content.contains("title = \"我的文章\""));
        assert!(content.contains("date = 2026-06-10T00:00:00Z"));
        assert!(content.contains("tags = [\"Rust\", \"AI\"]"));
        assert!(content.contains("<!-- more -->"));
        assert!(content.contains("正文段落"));
        assert!(!content.contains("---"), "YAML frontmatter 不应保留");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_replaces_cdn_image() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("export-img")?;
        let blog = root.join("blog");
        let md_path = root.join("article.md");
        create_file(
            &md_path,
            "---\ntitle: T\ndate: 2026-01-01\n---\n\n正文。\n\n![banner](http://mmbiz.qpic.cn/xxx/0?wx_fmt=png)\n",
        )?;

        export_article(&root, &md_path, &blog)?;

        let content = fs::read_to_string(blog.join("content/2026-01-01-article.md"))?;
        assert!(
            content.contains("/images/wechat-follow.png"),
            "CDN 图片应替换为本地路径"
        );
        assert!(!content.contains("mmbiz.qpic.cn"), "不应保留 CDN URL");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_strips_wechat_footer() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("export-footer")?;
        let blog = root.join("blog");
        let md_path = root.join("article.md");
        create_file(
            &md_path,
            "---\ntitle: T\ndate: 2026-01-01\n---\n\n正文内容。\n\n---\n\n![banner](http://mmbiz.qpic.cn/xxx)\n\n点个\"赞\"让我知道你喜欢，点个\"推荐\"让更多「寻月者」看到。\n",
        )?;

        export_article(&root, &md_path, &blog)?;

        let content = fs::read_to_string(blog.join("content/2026-01-01-article.md"))?;
        assert!(content.contains("正文内容"), "正文应保留");
        assert!(!content.contains("寻月者"), "WeChat footer 应被剥离");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_parses_tags() {
        let md =
            "---\ntitle: T\ndate: 2026-01-01\ntags: [\"读书\", \"Rust\", \"AI\"]\n---\n\n正文。\n";
        let fm = parse_frontmatter(md);
        assert_eq!(fm.tags, vec!["读书", "Rust", "AI"]);
        assert_eq!(fm.date.as_deref(), Some("2026-01-01"));
    }

    #[test]
    fn parses_export_command() -> Result<(), Box<dyn std::error::Error>> {
        let options =
            Options::parse(["export".to_owned(), "Articles/published/demo.md".to_owned()])?;
        let Command::Export { article } = options.command else {
            panic!("expected Export");
        };
        assert_eq!(article, PathBuf::from("Articles/published/demo.md"));
        Ok(())
    }

    #[test]
    fn preview_fails_without_html() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("preview-no-html")?;
        let md = root.join("demo.md");
        create_file(&md, "---\ntitle: T\n---\n正文\n")?;

        let err = preview_article(&root, &md).unwrap_err();
        assert!(matches!(err, AppError::NoHtml(_)), "应报 NoHtml 错误");

        fs::remove_dir_all(root)?;
        Ok(())
    }

    // ── radar scrape tests ────────────────────────────────────────────────────

    #[test]
    fn parses_radar_scrape_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "radar".to_owned(),
            "scrape".to_owned(),
            "--platform".to_owned(),
            "wechat".to_owned(),
            "--keyword".to_owned(),
            "AI写作".to_owned(),
            "--count".to_owned(),
            "5".to_owned(),
        ])?;
        let Command::Radar(RadarCommand::Scrape {
            platform,
            keyword,
            count,
            url,
        }) = options.command
        else {
            panic!("expected Scrape");
        };
        assert_eq!(platform, "wechat");
        assert_eq!(keyword, "AI写作");
        assert_eq!(count, 5);
        assert!(url.is_none());
        Ok(())
    }

    #[test]
    fn url_encode_handles_chinese() {
        let encoded = url_encode("AI写作");
        assert!(encoded.starts_with("AI"));
        assert!(encoded.contains('%'), "汉字应被百分比编码");
        assert!(!encoded.contains(' '));
    }

    #[test]
    fn extract_from_snapshot_parses_titles() {
        let snapshot = r#"
- document
  - main
    - heading "AI时代的写作技巧：10个让你效率翻倍的方法" [ref=e1]
    - link "普通人如何用AI写出爆款文章" [ref=e2]
    - link "关注" [ref=e3]
    - link "更多" [ref=e4]
"#;
        let titles = extract_from_snapshot(snapshot);
        assert!(titles.contains(&"AI时代的写作技巧：10个让你效率翻倍的方法".to_owned()));
        assert!(titles.contains(&"普通人如何用AI写出爆款文章".to_owned()));
        assert!(!titles.contains(&"关注".to_owned()), "太短的导航文字应过滤");
    }

    #[test]
    fn is_good_title_filters_short_and_nav() {
        assert!(is_good_title("AI时代的写作技巧让你效率翻倍"));
        assert!(!is_good_title("关注"), "太短");
        assert!(!is_good_title("var x = function(){}"), "JS代码");
        assert!(!is_good_title("ab"), "太短ASCII");
    }

    #[test]
    fn scrape_stores_samples_in_jsonl() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("scrape-store")?;

        // Build a fake HTML page with article titles
        let html = r#"<html><body>
            <h3><a href="/a1">坚持每天写作：我用AI辅助的30天实验</a></h3>
            <h3><a href="/a2">公众号涨粉秘诀：内容为王还是运营为王</a></h3>
            <h3><a href="/a3">短</a></h3>
        </body></html>"#;

        // Directly test extract_samples (bypasses network)
        let samples = extract_samples(html, "wechat", "AI写作", 10);
        assert!(!samples.is_empty(), "应提取到文章标题");
        assert!(samples.iter().all(|s| s.platform == "wechat"));
        assert!(samples.iter().all(|s| s.keyword == "AI写作"));
        assert!(samples.iter().all(|s| s.source == "scrape"));

        // Verify titles look reasonable (long enough, not nav)
        for s in &samples {
            assert!(s.title.chars().count() >= 6, "标题太短: {}", s.title);
        }

        fs::remove_dir_all(root)?;
        Ok(())
    }
}
// test hook trigger
