use std::fmt::{self, Display};
use std::fs::{self, OpenOptions};
use std::io;
use std::io::Write;
use std::path::{Path, PathBuf};

mod cover;
mod humanize;
mod illustrate;
mod wechat;
pub use wechat::WechatClient;

const DEFAULT_CONFIG: &str = "moonpub.toml";

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
        }
    }
}

impl std::error::Error for AppError {}

// ── Config ──────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    pub vault_root: Option<PathBuf>,
    pub wechat_appid: Option<String>,
    pub wechat_author: Option<String>,
    pub wechat_thumb_media_id: Option<String>,
    pub wechat_account_type: Option<String>,
    pub wechat_auto_publish: bool,
    pub blog_kind: Option<String>,
    pub blog_root: Option<PathBuf>,
}

impl Config {
    /// Minimal TOML parser that extracts string values from our known keys.
    pub fn from_toml(content: &str) -> Self {
        let mut cfg = Self {
            vault_root: None,
            wechat_appid: None,
            wechat_author: None,
            wechat_thumb_media_id: None,
            wechat_account_type: None,
            wechat_auto_publish: false,
            blog_kind: None,
            blog_root: None,
        };

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
                    "thumb_media_id" => cfg.wechat_thumb_media_id = Some(value.to_owned()),
                    "kind" => cfg.blog_kind = Some(value.to_owned()),
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
                let extra = rest[2..].iter();
                let mut extra = extra.peekable();
                while let Some(flag) = extra.next() {
                    match flag.as_str() {
                        "--media-id" => {
                            media_id = Some(
                                extra
                                    .next()
                                    .cloned()
                                    .ok_or(AppError::MissingValue("--media-id"))?,
                            );
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
                let screenshot = false;
                let mut extra = rest[2..].iter();
                while let Some(flag) = extra.next() {
                    match flag.as_str() {
                        "--style" => {
                            style = Some(
                                extra
                                    .next()
                                    .cloned()
                                    .ok_or(AppError::MissingValue("--style"))?,
                            );
                        }
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
                            author = Some(
                                extra
                                    .next()
                                    .cloned()
                                    .ok_or(AppError::MissingValue("--author"))?,
                            );
                        }
                        "--thumb" => {
                            thumb_media_id = Some(
                                extra
                                    .next()
                                    .cloned()
                                    .ok_or(AppError::MissingValue("--thumb"))?,
                            );
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
                .unwrap_or(Config {
                    vault_root: None,
                    wechat_appid: None,
                    wechat_author: None,
                    wechat_thumb_media_id: None,
                    wechat_account_type: None,
                    wechat_auto_publish: false,
                    blog_kind: None,
                    blog_root: None,
                });
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
            render_article(&options.vault, article, &resolved_author, &resolved_thumb)
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
                Some("clean") => cover::CoverStyle::Clean,
                Some("minimal") => cover::CoverStyle::Minimal,
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
                let abs_html = std::fs::canonicalize(&out).unwrap_or_else(|_| out.clone());
                let _ = std::process::Command::new("npx")
                    .args([
                        "@playwright/cli",
                        "open",
                        &format!("file://{}", abs_html.display()),
                        "--headless",
                    ])
                    .output();
                std::thread::sleep(std::time::Duration::from_secs(2));
                let _ = std::process::Command::new("npx")
                    .args([
                        "@playwright/cli",
                        "screenshot",
                        &format!("--filename={}", png.display()),
                    ])
                    .output();
                let _ = std::process::Command::new("npx")
                    .args(["@playwright/cli", "close"])
                    .output();
                if png.exists() {
                    result.push_str(&format!("\n  png:   {}", png.display()));
                } else {
                    result.push_str("\n  (screenshot failed - ensure playwright-cli is installed: npm i -g @playwright/cli)");
                }
            }
            Ok(result)
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
        Command::Push {
            article,
            auto_render,
        } => {
            let cfg = options
                .config
                .as_deref()
                .map(Config::load)
                .transpose()?
                .unwrap_or(Config {
                    vault_root: None,
                    wechat_appid: None,
                    wechat_author: None,
                    wechat_thumb_media_id: None,
                    wechat_account_type: None,
                    wechat_auto_publish: false,
                    blog_kind: None,
                    blog_root: None,
                });
            push_article(&options.vault, article, *auto_render, &cfg)
        }
        Command::UpdateDraft { article, media_id } => {
            let cfg = options
                .config
                .as_deref()
                .map(Config::load)
                .transpose()?
                .unwrap_or(Config {
                    vault_root: None,
                    wechat_appid: None,
                    wechat_author: None,
                    wechat_thumb_media_id: None,
                    wechat_account_type: None,
                    wechat_auto_publish: false,
                    blog_kind: None,
                    blog_root: None,
                });
            update_draft(&options.vault, article, media_id.as_deref(), &cfg)
        }
        Command::MarkReady { article } => {
            let slug = article_slug(article)?;
            add_status(&options.vault, &slug, "ready", "confirmed")
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
                .unwrap_or(Config {
                    vault_root: None,
                    wechat_appid: None,
                    wechat_author: None,
                    wechat_thumb_media_id: None,
                    wechat_account_type: None,
                    wechat_auto_publish: false,
                    blog_kind: None,
                    blog_root: None,
                });
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

// ── radar ─────────────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RadarCommand {
    Add(TrendSample),
    List {
        platform: Option<String>,
        keyword: Option<String>,
    },
    Import {
        path: PathBuf,
        platform: Option<String>,
    },
    Analyze {
        article: PathBuf,
        platform: String,
        top: usize,
    },
    Suggest {
        article: PathBuf,
        platform: String,
        top: usize,
    },
    Scrape {
        platform: String,
        keyword: String,
        count: usize,
        url: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrendSample {
    pub platform: String,
    pub keyword: String,
    pub title: String,
    pub url: Option<String>,
    pub author: Option<String>,
    pub likes: Option<u64>,
    pub collects: Option<u64>,
    pub comments: Option<u64>,
    pub source: String,
}

impl TrendSample {
    fn engagement_score(&self) -> u64 {
        self.likes.unwrap_or(0) + self.collects.unwrap_or(0) * 2 + self.comments.unwrap_or(0) * 3
    }
}

fn parse_radar_command(args: &[String]) -> Result<RadarCommand, AppError> {
    let Some(command) = args.first() else {
        return Err(AppError::MissingValue("radar <add|list|import|analyze>"));
    };

    match command.as_str() {
        "add" => parse_radar_add(&args[1..]),
        "list" => parse_radar_list(&args[1..]),
        "import" => parse_radar_import(&args[1..]),
        "analyze" => parse_radar_analyze(&args[1..]),
        "suggest" => parse_radar_suggest(&args[1..]),
        "scrape" => parse_radar_scrape(&args[1..]),
        value => Err(AppError::UnknownCommand(format!("radar {value}"))),
    }
}

fn parse_radar_add(args: &[String]) -> Result<RadarCommand, AppError> {
    let mut platform = None;
    let mut keyword = None;
    let mut title = None;
    let mut url = None;
    let mut author = None;
    let mut likes = None;
    let mut collects = None;
    let mut comments = None;
    let mut source = String::from("manual");

    let mut args = args.iter();
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--platform" => platform = Some(next_arg(&mut args, "--platform")?),
            "--keyword" => keyword = Some(next_arg(&mut args, "--keyword")?),
            "--title" => title = Some(next_arg(&mut args, "--title")?),
            "--url" => url = Some(next_arg(&mut args, "--url")?),
            "--author" => author = Some(next_arg(&mut args, "--author")?),
            "--likes" => likes = Some(parse_u64("--likes", next_arg(&mut args, "--likes")?)?),
            "--collects" => {
                collects = Some(parse_u64("--collects", next_arg(&mut args, "--collects")?)?);
            }
            "--comments" => {
                comments = Some(parse_u64("--comments", next_arg(&mut args, "--comments")?)?);
            }
            "--source" => source = next_arg(&mut args, "--source")?,
            value if value.starts_with('-') => {
                return Err(AppError::UnknownOption(value.to_owned()));
            }
            value => return Err(AppError::UnknownCommand(value.to_owned())),
        }
    }

    Ok(RadarCommand::Add(TrendSample {
        platform: platform.ok_or(AppError::MissingValue("--platform"))?,
        keyword: keyword.ok_or(AppError::MissingValue("--keyword"))?,
        title: title.ok_or(AppError::MissingValue("--title"))?,
        url,
        author,
        likes,
        collects,
        comments,
        source,
    }))
}

fn parse_radar_list(args: &[String]) -> Result<RadarCommand, AppError> {
    let mut platform = None;
    let mut keyword = None;
    let mut args = args.iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--platform" => platform = Some(next_arg(&mut args, "--platform")?),
            "--keyword" => keyword = Some(next_arg(&mut args, "--keyword")?),
            value if value.starts_with('-') => {
                return Err(AppError::UnknownOption(value.to_owned()));
            }
            value => return Err(AppError::UnknownCommand(value.to_owned())),
        }
    }

    Ok(RadarCommand::List { platform, keyword })
}

fn parse_radar_import(args: &[String]) -> Result<RadarCommand, AppError> {
    let path = args
        .first()
        .map(PathBuf::from)
        .ok_or(AppError::MissingValue("radar import <file.csv>"))?;

    let mut platform = None;
    let mut args = args[1..].iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--platform" => platform = Some(next_arg(&mut args, "--platform")?),
            value if value.starts_with('-') => {
                return Err(AppError::UnknownOption(value.to_owned()));
            }
            value => return Err(AppError::UnknownCommand(value.to_owned())),
        }
    }

    Ok(RadarCommand::Import { path, platform })
}

fn parse_radar_suggest(args: &[String]) -> Result<RadarCommand, AppError> {
    let mut article = None;
    let mut platform = None;
    let mut top = 10usize;
    let mut args = args.iter();
    while let Some(arg) = args.next() {
        if let Some(a) = parse_radar_article_arg(arg, &mut args) { article = a; }
        else { match arg.as_str() {
            "--platform" => platform = Some(next_arg(&mut args, "--platform")?),
            "--top" => { let v = next_arg(&mut args, "--top")?; top = v.parse().map_err(|_| AppError::InvalidNumber { flag: "--top", value: v })?; }
            _ => {}
        }}
    }
    Ok(RadarCommand::Suggest { article: PathBuf::from(article.ok_or(AppError::MissingValue("suggest <article.md>"))?), platform: platform.ok_or(AppError::MissingValue("--platform"))?, top })
}

fn parse_radar_article_arg(arg: &str, _args: &mut std::slice::Iter<String>) -> Option<Option<String>> {
    if !arg.starts_with('-') { Some(Some(arg.to_owned())) } else { None }
}

fn parse_radar_analyze(args: &[String]) -> Result<RadarCommand, AppError> {
    let article = args
        .first()
        .map(PathBuf::from)
        .ok_or(AppError::MissingValue("radar analyze <article.md>"))?;

    let mut platform = None;
    let mut top = 10usize;
    let mut args = args[1..].iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--platform" => platform = Some(next_arg(&mut args, "--platform")?),
            "--top" => {
                let v = next_arg(&mut args, "--top")?;
                top = v.parse().map_err(|_| AppError::InvalidNumber {
                    flag: "--top",
                    value: v,
                })?;
            }
            value if value.starts_with('-') => {
                return Err(AppError::UnknownOption(value.to_owned()));
            }
            value => return Err(AppError::UnknownCommand(value.to_owned())),
        }
    }

    Ok(RadarCommand::Analyze {
        article,
        platform: platform.ok_or(AppError::MissingValue("--platform"))?,
        top,
    })
}

fn parse_radar_scrape(args: &[String]) -> Result<RadarCommand, AppError> {
    let mut platform = None;
    let mut keyword = None;
    let mut count = 10usize;
    let mut url = None;
    let mut args = args.iter();

    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--platform" => platform = Some(next_arg(&mut args, "--platform")?),
            "--keyword" => keyword = Some(next_arg(&mut args, "--keyword")?),
            "--count" => {
                let v = next_arg(&mut args, "--count")?;
                count = v.parse().map_err(|_| AppError::InvalidNumber {
                    flag: "--count",
                    value: v,
                })?;
            }
            "--url" => url = Some(next_arg(&mut args, "--url")?),
            value if value.starts_with('-') => {
                return Err(AppError::UnknownOption(value.to_owned()));
            }
            value => return Err(AppError::UnknownCommand(value.to_owned())),
        }
    }

    Ok(RadarCommand::Scrape {
        platform: platform.ok_or(AppError::MissingValue("--platform"))?,
        keyword: keyword.ok_or(AppError::MissingValue("--keyword"))?,
        count,
        url,
    })
}

fn next_arg<'a>(
    args: &mut impl Iterator<Item = &'a String>,
    flag: &'static str,
) -> Result<String, AppError> {
    args.next().cloned().ok_or(AppError::MissingValue(flag))
}

fn parse_u64(flag: &'static str, value: String) -> Result<u64, AppError> {
    value
        .parse()
        .map_err(|_| AppError::InvalidNumber { flag, value })
}

pub fn run_radar(vault: &Path, command: &RadarCommand) -> Result<String, AppError> {
    match command {
        RadarCommand::Add(sample) => add_trend_sample(vault, sample),
        RadarCommand::List { platform, keyword } => list_trend_samples(vault, platform, keyword),
        RadarCommand::Import { path, platform } => import_csv(vault, path, platform.as_deref()),
        RadarCommand::Analyze {
            article,
            platform,
            top,
        } => analyze_article(vault, article, platform, *top),
        RadarCommand::Suggest {
            article,
            platform,
            top,
        } => suggest_titles(vault, article, platform, *top),
        RadarCommand::Scrape {
            platform,
            keyword,
            count,
            url,
        } => scrape_radar(vault, platform, keyword, *count, url.as_deref()),
    }
}

pub fn add_trend_sample(vault: &Path, sample: &TrendSample) -> Result<String, AppError> {
    let path = trend_store_path(vault);
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
    writeln!(file, "{}", sample.to_json_line()).map_err(|source| AppError::Io {
        path: path.clone(),
        source,
    })?;

    Ok(format!("added trend sample to {}", path.display()))
}

pub fn list_trend_samples(
    vault: &Path,
    platform: &Option<String>,
    keyword: &Option<String>,
) -> Result<String, AppError> {
    let path = trend_store_path(vault);
    if !path.exists() {
        return Ok("trend samples\n  (empty)".to_owned());
    }

    let content = fs::read_to_string(&path).map_err(|source| AppError::Io {
        path: path.clone(),
        source,
    })?;

    let mut rows = Vec::new();
    for line in content.lines().filter(|line| !line.trim().is_empty()) {
        if let Some(sample) = TrendSample::from_json_line(line) {
            if let Some(expected) = platform
                && &sample.platform != expected
            {
                continue;
            }
            if let Some(expected) = keyword
                && &sample.keyword != expected
            {
                continue;
            }
            rows.push(sample);
        }
    }

    Ok(format_trend_samples(&rows))
}

// ── radar import ──────────────────────────────────────────────────────────────

/// CSV column header names we recognise (case-insensitive).
const COL_PLATFORM: &[&str] = &["platform", "平台"];
const COL_KEYWORD: &[&str] = &["keyword", "关键词", "keywords"];
const COL_TITLE: &[&str] = &["title", "标题"];
const COL_URL: &[&str] = &["url", "链接"];
const COL_AUTHOR: &[&str] = &["author", "作者"];
const COL_LIKES: &[&str] = &["likes", "点赞", "like_count"];
const COL_COLLECTS: &[&str] = &["collects", "收藏", "collect_count", "favorites"];
const COL_COMMENTS: &[&str] = &["comments", "评论", "comment_count"];
const COL_SOURCE: &[&str] = &["source", "来源"];

pub fn import_csv(
    vault: &Path,
    csv_path: &Path,
    default_platform: Option<&str>,
) -> Result<String, AppError> {
    let content = fs::read_to_string(csv_path).map_err(|source| AppError::Io {
        path: csv_path.to_path_buf(),
        source,
    })?;

    let mut lines = content.lines();
    let header_line = lines
        .next()
        .ok_or_else(|| AppError::InvalidCsv("empty file".to_owned()))?;
    let headers: Vec<String> = parse_csv_row(header_line);

    fn col_index(headers: &[String], names: &[&str]) -> Option<usize> {
        headers.iter().position(|h| {
            let lower = h.to_lowercase();
            names.iter().any(|n| lower == *n)
        })
    }

    let idx_platform = col_index(&headers, COL_PLATFORM);
    let idx_keyword = col_index(&headers, COL_KEYWORD)
        .ok_or_else(|| AppError::InvalidCsv("missing 'keyword' column".to_owned()))?;
    let idx_title = col_index(&headers, COL_TITLE)
        .ok_or_else(|| AppError::InvalidCsv("missing 'title' column".to_owned()))?;
    let idx_url = col_index(&headers, COL_URL);
    let idx_author = col_index(&headers, COL_AUTHOR);
    let idx_likes = col_index(&headers, COL_LIKES);
    let idx_collects = col_index(&headers, COL_COLLECTS);
    let idx_comments = col_index(&headers, COL_COMMENTS);
    let idx_source = col_index(&headers, COL_SOURCE);

    let mut count = 0u32;
    let store_path = trend_store_path(vault);
    if let Some(parent) = store_path.parent() {
        fs::create_dir_all(parent).map_err(|source| AppError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&store_path)
        .map_err(|source| AppError::Io {
            path: store_path.clone(),
            source,
        })?;

    for line in lines.filter(|l| !l.trim().is_empty()) {
        let cols = parse_csv_row(line);
        let get = |idx: Option<usize>| -> Option<String> {
            idx.and_then(|i| cols.get(i))
                .map(|s| s.trim().to_owned())
                .filter(|s| !s.is_empty())
        };
        let get_u64 = |idx: Option<usize>| -> Option<u64> { get(idx).and_then(|v| v.parse().ok()) };

        let platform = get(idx_platform)
            .or_else(|| default_platform.map(str::to_owned))
            .unwrap_or_else(|| "unknown".to_owned());
        let keyword = match get(Some(idx_keyword)) {
            Some(v) => v,
            None => continue,
        };
        let title = match get(Some(idx_title)) {
            Some(v) => v,
            None => continue,
        };

        let sample = TrendSample {
            platform,
            keyword,
            title,
            url: get(idx_url),
            author: get(idx_author),
            likes: get_u64(idx_likes),
            collects: get_u64(idx_collects),
            comments: get_u64(idx_comments),
            source: get(idx_source).unwrap_or_else(|| "csv".to_owned()),
        };

        writeln!(file, "{}", sample.to_json_line()).map_err(|source| AppError::Io {
            path: store_path.clone(),
            source,
        })?;
        count += 1;
    }

    Ok(format!(
        "imported {count} samples from {}",
        csv_path.display()
    ))
}

/// Parse a single CSV row, respecting double-quoted fields.
pub fn parse_csv_row(line: &str) -> Vec<String> {
    let mut fields = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = line.chars().peekable();

    while let Some(ch) = chars.next() {
        match ch {
            '"' if in_quotes => {
                if chars.peek() == Some(&'"') {
                    chars.next();
                    current.push('"');
                } else {
                    in_quotes = false;
                }
            }
            '"' => in_quotes = true,
            ',' if !in_quotes => {
                fields.push(current.clone());
                current.clear();
            }
            other => current.push(other),
        }
    }
    fields.push(current);
    fields
}

// ── radar analyze ─────────────────────────────────────────────────────────────

pub fn analyze_article(
    vault: &Path,
    article: &Path,
    platform: &str,
    top: usize,
) -> Result<String, AppError> {
    let article = resolve_article_path(vault, article);
    let content = fs::read_to_string(&article).map_err(|source| AppError::Io {
        path: article.clone(),
        source,
    })?;

    let article_tokens = tokenize(&content);

    let store_path = trend_store_path(vault);
    let samples = load_all_samples(&store_path)?;

    let mut scored: Vec<(u64, &TrendSample)> = samples
        .iter()
        .filter(|s| s.platform == platform)
        .map(|s| {
            let title_tokens = tokenize(&s.title);
            let keyword_tokens = tokenize(&s.keyword);
            let overlap = count_overlap(&article_tokens, &title_tokens)
                + count_overlap(&article_tokens, &keyword_tokens) * 2;
            let engagement = s.engagement_score();
            let score = engagement.saturating_add(overlap as u64 * 100);
            (score, s)
        })
        .collect();

    scored.sort_by_key(|b| std::cmp::Reverse(b.0));

    let top_n: Vec<_> = scored.into_iter().take(top).collect();

    Ok(format_analyze_results(platform, &top_n))
}

fn load_all_samples(path: &Path) -> Result<Vec<TrendSample>, AppError> {
    if !path.exists() {
        return Ok(Vec::new());
    }
    let content = fs::read_to_string(path).map_err(|source| AppError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    Ok(content
        .lines()
        .filter(|l| !l.trim().is_empty())
        .filter_map(TrendSample::from_json_line)
        .collect())
}

fn tokenize(text: &str) -> Vec<String> {
    // Split on whitespace and common Chinese/ASCII boundaries; keep non-empty 2+ char tokens.
    let mut tokens = Vec::new();
    let mut current = String::new();
    for ch in text.chars() {
        if ch.is_ascii_alphanumeric() || ch.is_alphabetic() {
            current.push(ch);
        } else {
            if current.chars().count() >= 2 {
                tokens.push(current.to_lowercase());
            }
            current.clear();
        }
    }
    if current.chars().count() >= 2 {
        tokens.push(current.to_lowercase());
    }
    tokens
}

fn count_overlap(a: &[String], b: &[String]) -> usize {
    b.iter().filter(|t| a.contains(t)).count()
}

fn format_analyze_results(platform: &str, scored: &[(u64, &TrendSample)]) -> String {
    let mut output = format!("title suggestions for [{platform}]\n");
    if scored.is_empty() {
        output.push_str("  (no trend samples for this platform)");
        return output;
    }
    for (rank, (score, sample)) in scored.iter().enumerate() {
        output.push_str(&format!("  {}. {} (score={score}", rank + 1, sample.title));
        if let Some(likes) = sample.likes {
            output.push_str(&format!(", likes={likes}"));
        }
        output.push_str(&format!(", keyword={})\n", sample.keyword));
    }
    output.trim_end().to_owned()
}

// ── radar suggest ─────────────────────────────────────────────────────────────

/// Apply 4 golden title formulas to suggest titles based on article content
/// and trending data. Reference: "如何写出好标题" (green planet PPT).
pub fn suggest_titles(
    vault: &Path,
    article: &Path,
    platform: &str,
    top: usize,
) -> Result<String, AppError> {
    let article = resolve_article_path(vault, article);
    let content = fs::read_to_string(&article).map_err(|source| AppError::Io {
        path: article.clone(),
        source,
    })?;

    let front = parse_frontmatter(&content);
    let body = strip_frontmatter(&content);
    let orig_title = front.title.as_deref().unwrap_or("");
    let digest = front.digest.as_deref().unwrap_or("");

    // Extract trending titles for reference
    let store_path = trend_store_path(vault);
    let samples = load_all_samples(&store_path).unwrap_or_default();
    let platform_samples: Vec<&TrendSample> = samples
        .iter()
        .filter(|s| s.platform == platform)
        .collect();

    let article_tokens = tokenize(body);

    // Find top trending titles (by engagement) as reference patterns
    let mut scored: Vec<(u64, &TrendSample)> = platform_samples
        .iter()
        .map(|s| (s.engagement_score(), *s))
        .collect();
    scored.sort_by_key(|(score, _)| std::cmp::Reverse(*score));
    let top_trends: Vec<&TrendSample> = scored
        .iter()
        .take(top.min(10))
        .map(|(_, s)| *s)
        .collect();

    // Build enhanced keyword list from article tokens
    let mut phrases: Vec<&str> = article_tokens
        .iter()
        .map(|s| s.as_str())
        .collect();
    phrases.sort_by_key(|p| std::cmp::Reverse(p.chars().count()));
    let key_phrase = phrases.first().copied().unwrap_or("");

    // Count sections

    let mut output = format!("title suggestions for [{platform}]");
    if !orig_title.is_empty() {
        output.push_str(&format!(" (current: {orig_title})"));
    }
    output.push('\n');
    output.push_str("────────────────────────────────────────\n\n");

    // ── Formula 1: 痛点 + 解决方案 ──
    output.push_str("▎痛点 + 解决方案\n");
    let pain_raw = extract_pain_point(body).unwrap_or("努力却没有成果");
    let pain_short: String = pain_raw.chars().take(12).collect();
    let solution = first_paragraph_hook(body).unwrap_or("这里有答案");
    let solution_short: String = solution.chars().take(15).collect();
    let f1 = format!("总是{}？{}", pain_short, solution_short);
    output.push_str(&format!("  {f1}\n"));
    if let Some(ref_trend) = top_trends.first() {
        output.push_str(&format!("  ↳ 参考: {} (likes={})\n\n", ref_trend.title, ref_trend.likes.unwrap_or(0)));
    } else {
        output.push('\n');
    }

    // ── Formula 2: 数字 + 利益结果 ──
    output.push_str("▎数字 + 利益结果\n");
    let real_sections: Vec<&str> = body.lines().filter(|l| l.trim().starts_with("## ")).collect();
    let h2_count = real_sections.len().max(2).min(8);
    let themes: Vec<&str> = real_sections.iter().take(3).map(|l| l.trim().trim_start_matches("## ").trim()).collect();
    let theme = themes.first().copied().unwrap_or("改变认知");
    let f2 = format!("这本书我读了{}遍，总结出{}条关于{}的真相", h2_count, h2_count, theme);
    output.push_str(&format!("  {f2}\n"));
    if let Some(ref_trend) = top_trends.get(1) {
        output.push_str(&format!("  ↳ 参考: {} (likes={})\n\n", ref_trend.title, ref_trend.likes.unwrap_or(0)));
    } else {
        output.push('\n');
    }

    // ── Formula 3: 故事悬念/冲突 ──
    output.push_str("▎故事悬念 / 冲突\n");
    let hook = first_paragraph_hook(body).unwrap_or(digest);
    let hook_short: String = hook.chars().take(20).collect();
    let contrast = extract_contrast(body).unwrap_or("完全不同的答案");
    let contrast_short: String = contrast.chars().take(25).collect();
    let f3 = if !hook.is_empty() {
        format!("{}……这不是{}, 而是{}", hook_short, key_phrase, contrast_short)
    } else {
        format!("我原本以为{}，没想到却是{}", key_phrase, contrast_short)
    };
    output.push_str(&format!("  {f3}\n"));
    if let Some(ref_trend) = top_trends.get(2) {
        output.push_str(&format!("  ↳ 参考: {} (likes={})\n\n", ref_trend.title, ref_trend.likes.unwrap_or(0)));
    } else {
        output.push('\n');
    }

    // ── Formula 4: 用户标签 + 情感共鸣 ──
    output.push_str("▎用户标签 + 情感共鸣\n");
    let label_raw = extract_reader_label(body).unwrap_or("每一个还在坚持的人");
    let label_short: String = label_raw.chars().take(8).collect();
    let f4 = format!("致所有{}的人：{}", label_short, orig_title);
    output.push_str(&format!("  {f4}\n"));
    if let Some(ref_trend) = top_trends.get(3) {
        output.push_str(&format!("  ↳ 参考: {} (likes={})\n\n", ref_trend.title, ref_trend.likes.unwrap_or(0)));
    } else {
        output.push('\n');
    }

    // ── trending references ──
    if !top_trends.is_empty() {
        output.push_str("────────────────────────────────────────\n");
        output.push_str("trending on this platform (for reference):\n");
        for (i, t) in top_trends.iter().take(top).enumerate() {
            let eng = t.engagement_score();
            output.push_str(&format!("  {}. {} (score={})\n", i + 1, t.title, eng));
        }
    }

    Ok(output.trim_end().to_owned())
}

/// Strip block syntax and headings, return only plain paragraph text lines.
fn body_text_only(body: &str) -> Vec<&str> {
    let mut in_block = false;
    body.lines()
        .filter(|l| {
            let t = l.trim();
            if t.starts_with(":::") { in_block = !in_block; return false; }
            if in_block { return false; }
            if t.starts_with('#') || t.starts_with('>') || t.is_empty() { return false; }
            if t.starts_with("---") || t.starts_with("***") { return false; }
            true
        })
        .collect()
}

fn extract_pain_point(body: &str) -> Option<&str> {
    let keywords = ["很难", "不容易", "崩溃", "放弃", "痛苦", "没有", "不知道", "怎么办"];
    for line in body.lines() {
        let t = line.trim();
        if t.starts_with(':') || t.starts_with('#') || t.starts_with('>') || t.is_empty() { continue; }
        for kw in &keywords { if t.contains(kw) { return Some(t); } }
    }
    // Fallback: first real paragraph
    body.lines().find(|l| {
        let t = l.trim();
        !t.is_empty() && !t.starts_with(':') && !t.starts_with('#') && !t.starts_with('>') && t.chars().count() > 10
    }).map(|l| l.trim())
}


fn extract_contrast(body: &str) -> Option<&str> {
    let paragraphs = body_text_only(body);
    for line in &paragraphs {
        if line.contains("不是") && line.contains("而是") { return Some(line); }
    }
    // Fallback: find characteristic phrase
    paragraphs.iter().filter(|l| l.chars().count() > 10).nth(2).copied()
}

fn extract_reader_label(body: &str) -> Option<&str> {
    let labels = ["读书", "写作", "坚持", "努力", "成长", "挣扎", "孤独", "选择", "热爱", "艺术"];
    for label in &labels { if body.contains(label) { return Some(label); } }
    let paragraphs = body_text_only(body);
    paragraphs.first().copied()
}

fn first_paragraph_hook(body: &str) -> Option<&str> {
    let paragraphs = body_text_only(body);
    paragraphs.first().copied()
}


// ── helpers ───────────────────────────────────────────────────────────────────

fn trend_store_path(vault: &Path) -> PathBuf {
    vault.join(".moonpub").join("trends.jsonl")
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

fn resolve_article_path(vault: &Path, article: &Path) -> PathBuf {
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
thumb_media_id = ""

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
  moonpub [--vault <path>] mark-ready <article.md>
  moonpub [--vault <path>] mark-published <article.md>
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] radar add --platform <name> --keyword <text> --title <text> [--url <url>] [--likes <n>] [--collects <n>] [--comments <n>]
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] radar list [--platform <name>] [--keyword <text>]
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] radar import <file.csv> [--platform <name>]
  moonpub [--vault <path>] [--config <moonpub.toml>] [--json] radar analyze <article.md> --platform <name> [--top <n>]

Commands:
  init      Create a sample moonpub.toml
  status    List article files in Articles/drafts, ready, and published
  check     Check whether an article bundle has md/html/draft.json files
  render    Generate <slug>.html and <slug>.draft.json from a Markdown article
  push         Push draft to WeChat (direct API), write .media_id, move to published/
  update-draft Re-push updated HTML to an existing WeChat draft by media_id
  export    Export article to Zola blog (YAML→TOML frontmatter, strip WeChat footer)
  preview   Open the rendered HTML in the system browser
  radar     Store and list platform trend samples
"#,
    )
}

// ── radar scrape ─────────────────────────────────────────────────────────────

/// Scrape trending articles for a platform keyword and store them in trends.jsonl.
///
/// Uses playwright-cli if found in PATH, otherwise falls back to curl.
/// Default search URL for wechat: Sogou WeChat search (public, no auth).
pub fn scrape_radar(
    vault: &Path,
    platform: &str,
    keyword: &str,
    count: usize,
    custom_url: Option<&str>,
) -> Result<String, AppError> {
    let url = custom_url
        .map(str::to_owned)
        .unwrap_or_else(|| default_search_url(platform, keyword));

    let raw = fetch_page(&url)?;
    let samples = extract_samples(&raw, platform, keyword, count);

    if samples.is_empty() {
        return Ok(format!(
            "scraped 0 samples from {url}\n  tip: try --url to specify a direct search page"
        ));
    }

    let store = trend_store_path(vault);
    if let Some(p) = store.parent() {
        fs::create_dir_all(p).map_err(|source| AppError::Io {
            path: p.to_path_buf(),
            source,
        })?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&store)
        .map_err(|source| AppError::Io {
            path: store.clone(),
            source,
        })?;
    for s in &samples {
        writeln!(file, "{}", s.to_json_line()).map_err(|source| AppError::Io {
            path: store.clone(),
            source,
        })?;
    }

    Ok(format!(
        "scraped {} samples → {}\n{}",
        samples.len(),
        store.display(),
        samples
            .iter()
            .map(|s| format!("  · {}", s.title))
            .collect::<Vec<_>>()
            .join("\n")
    ))
}

fn default_search_url(platform: &str, keyword: &str) -> String {
    let encoded = url_encode(keyword);
    match platform {
        "wechat" | "微信" => {
            format!("https://weixin.sogou.com/weixin?type=2&query={encoded}")
        }
        "xiaohongshu" | "xhs" | "小红书" => {
            format!("https://www.xiaohongshu.com/search_result/?keyword={encoded}")
        }
        _ => format!("https://weixin.sogou.com/weixin?type=2&query={encoded}"),
    }
}

fn url_encode(s: &str) -> String {
    let mut out = String::new();
    for byte in s.bytes() {
        match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(byte as char);
            }
            _ => out.push_str(&format!("%{byte:02X}")),
        }
    }
    out
}

/// Fetch a page using playwright-cli (if available) or curl.
fn fetch_page(url: &str) -> Result<String, AppError> {
    if let Some(content) = try_playwright_cli(url) {
        return Ok(content);
    }
    fetch_with_curl(url)
}

fn try_playwright_cli(url: &str) -> Option<String> {
    // playwright-cli needs a persistent session; open then snapshot in sequence.
    // Requires playwright-cli binary in PATH.
    let open_ok = std::process::Command::new("playwright-cli")
        .args(["open", url, "--headless"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .is_some();

    if !open_ok {
        return None;
    }

    let snap = std::process::Command::new("playwright-cli")
        .arg("snapshot")
        .output()
        .ok()?;

    let _ = std::process::Command::new("playwright-cli")
        .arg("close")
        .output();

    let content = String::from_utf8_lossy(&snap.stdout).into_owned();
    if content.trim().is_empty() {
        None
    } else {
        Some(content)
    }
}

fn fetch_with_curl(url: &str) -> Result<String, AppError> {
    let out = std::process::Command::new("curl")
        .args([
            "-sL",
            "--max-time",
            "15",
            "-H",
            "User-Agent: Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36",
            "-H",
            "Accept-Language: zh-CN,zh;q=0.9",
            "--compressed",
            url,
        ])
        .output()
        .map_err(|source| AppError::Io {
            path: PathBuf::from("curl"),
            source,
        })?;

    if !out.status.success() {
        return Err(AppError::PushFailed {
            message: format!("curl failed for {url}"),
            ip_hint: None,
        });
    }

    Ok(String::from_utf8_lossy(&out.stdout).into_owned())
}

/// Extract TrendSample candidates from raw HTML or a playwright snapshot.
fn extract_samples(raw: &str, platform: &str, keyword: &str, limit: usize) -> Vec<TrendSample> {
    let mut samples = Vec::new();

    // Heuristic 1: playwright-cli snapshot lines look like:
    //   - link "文章标题" [ref=e5]
    //   - heading "文章标题"
    // Heuristic 2: HTML <title> / <h3> / <a> text content
    let is_snapshot = raw.contains("[ref=");

    let titles: Vec<String> = if is_snapshot {
        extract_from_snapshot(raw)
    } else {
        extract_from_html(raw)
    };

    for title in titles.into_iter().take(limit) {
        samples.push(TrendSample {
            platform: platform.to_owned(),
            keyword: keyword.to_owned(),
            title,
            url: None,
            author: None,
            likes: None,
            collects: None,
            comments: None,
            source: "scrape".to_owned(),
        });
    }
    samples
}

fn extract_from_snapshot(snapshot: &str) -> Vec<String> {
    let mut titles = Vec::new();
    for line in snapshot.lines() {
        let trimmed = line.trim();
        for prefix in &["- link \"", "- heading \"", "  - link \"", "  - heading \""] {
            if let Some(rest) = trimmed.strip_prefix(prefix)
                && let Some(end) = rest.find('"')
            {
                let text = rest[..end].trim();
                if is_good_title(text) {
                    titles.push(text.to_owned());
                }
            }
        }
    }
    titles
}

fn extract_from_html(html: &str) -> Vec<String> {
    let mut titles = Vec::new();

    // Remove script/style blocks first
    let text = {
        let mut s = html.to_owned();
        // Remove <script> blocks
        while let (Some(a), Some(b)) = (s.find("<script"), s.find("</script>")) {
            if a < b {
                s = format!("{}{}", &s[..a], &s[b + 9..]);
            } else {
                break;
            }
        }
        s
    };

    // Extract text from <a>, <h3>, <h2> tags
    let tag_re_patterns = ["<h2", "<h3", "<a "];
    for &tag in &tag_re_patterns {
        let mut pos = 0;
        while let Some(start) = text[pos..].find(tag) {
            let abs_start = pos + start;
            // Find end of opening tag
            let Some(gt) = text[abs_start..].find('>') else {
                break;
            };
            let content_start = abs_start + gt + 1;
            // Find closing tag
            let close_tag = format!("</{}", &tag[1..].trim_end_matches(' '));
            let content_end = text[content_start..]
                .find(close_tag.as_str())
                .map(|i| content_start + i)
                .unwrap_or(content_start + 200.min(text.len() - content_start));

            if content_end > content_start {
                let inner = &text[content_start..content_end];
                let plain = strip_html_tags(inner);
                let plain = plain.trim();
                if is_good_title(plain) {
                    titles.push(plain.to_owned());
                }
            }

            pos = abs_start + 1;
            if pos >= text.len() {
                break;
            }
        }
    }

    // Deduplicate preserving order
    let mut seen = std::collections::HashSet::new();
    titles.retain(|t| seen.insert(t.clone()));
    titles
}

fn strip_html_tags(html: &str) -> String {
    let mut out = String::new();
    let mut in_tag = false;
    for ch in html.chars() {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            c if !in_tag => out.push(c),
            _ => {}
        }
    }
    // Decode common entities
    out.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&nbsp;", " ")
        .replace("&#34;", "\"")
        .replace("&#39;", "'")
}

fn is_good_title(text: &str) -> bool {
    let char_count = text.chars().count();
    // At least 6 chars, not too long, contains some CJK or meaningful ASCII
    if !(6..=80).contains(&char_count) {
        return false;
    }
    // Must have at least a few non-whitespace chars
    if text.chars().filter(|c| !c.is_whitespace()).count() < 4 {
        return false;
    }
    // Skip nav-like strings
    let lower = text.to_lowercase();
    let nav_words = [
        "javascript",
        "function",
        "var ",
        "onclick",
        "copyright",
        "©",
    ];
    !nav_words.iter().any(|w| lower.contains(w))
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

// ── push ──────────────────────────────────────────────────────────────────────

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
            render_article(vault, &article, &author, &thumb)?;
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
    let mut result = format!("pushed\n  media_id: {media_id}{moved}");

    // Auto-publish for verified/service accounts
    if cfg.wechat_auto_publish {
        let acct_type = cfg.wechat_account_type.as_deref().unwrap_or("personal");
        if acct_type != "personal" {
            match client.free_publish(&token, &media_id) {
                Ok(publish_id) => {
                    let _ = add_status(vault, &slug, "published", &publish_id);
                    result.push_str(&format!("\n  auto-published ({}): {}", acct_type, publish_id));
                }
                Err(e) => {
                    result.push_str(&format!("\n  auto-publish failed: {e}"));
                }
            }
        } else {
            result.push_str("\n  (auto_publish: personal accounts need manual publish)");
        }
    } else {
        result.push_str("\n  next: set cover/original/collection in WeChat backend, then publish");
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
    let html_body = md_to_wechat_html(body);
    let full_html = wrap_wechat_html(&html_body);

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

fn parse_frontmatter(md: &str) -> Frontmatter {
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

fn strip_frontmatter(md: &str) -> &str {
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

fn md_to_wechat_html(md: &str) -> String {
    let blocks = parse_blocks(md);
    let mut out = String::new();

    for block in &blocks {
        match block {
            MdBlock::Fence(name, props, body) => {
                out.push_str(&render_fence_block(name, props, body))
            }
            MdBlock::Markdown(text) => out.push_str(&render_markdown_segment(text)),
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

fn render_fence_block(name: &str, props: &[(&str, &str)], body: &str) -> String {
    match name {
        "book-info" => render_book_info(props),
        "intro" => render_intro(body),
        "callout" => render_callout(props, body),
        "steps" => render_steps(body),
        "summary" => render_summary(body),
        "figure" => render_figure(props),
        "checklist" => render_checklist(body),
        "cover" => render_cover(props),
        "quote-card" => {
            let text = body.trim().to_owned();
            let source = props.iter().find(|(k, _)| *k == "source").map(|(_, v)| *v).unwrap_or("");
            illustrate::render_illustration(&illustrate::IllustType::QuoteCard { text, source: source.to_owned() })
        }
        "divider" => {
            let label = props.iter().find(|(k, _)| *k == "label").map(|(_, v)| *v).unwrap_or("");
            illustrate::render_illustration(&illustrate::IllustType::Divider { label: label.to_owned() })
        }
        "concept-card" => {
            let number: u32 = props.iter().find(|(k, _)| *k == "number").and_then(|(_, v)| v.parse().ok()).unwrap_or(1);
            let title = body.lines().next().unwrap_or("").trim().to_owned();
            let desc = body.lines().skip(1).collect::<Vec<_>>().join("
").trim().to_owned();
            illustrate::render_illustration(&illustrate::IllustType::ConceptCard { number, title, desc })
        }
        "emotion-card" => {
            let mood = props.iter().find(|(k, _)| *k == "mood").map(|(_, v)| *v).unwrap_or("think");
            illustrate::render_illustration(&illustrate::IllustType::EmotionCard { mood: mood.to_owned(), text: body.trim().to_owned() })
        }
        "code" => {
            let lang = props.iter().find(|(k, _)| *k == "lang").map(|(_, v)| *v).unwrap_or("");
            illustrate::render_code_block(lang, body.trim())
        }
        "timeline" => {
            let items: Vec<(String, String)> = body.lines()
                .filter(|l| l.trim().starts_with("- "))
                .filter_map(|l| {
                    let s = l.trim().trim_start_matches("- ").trim();
                    s.split_once(": ").map(|(d, t)| (d.to_owned(), t.to_owned()))
                })
                .collect();
            if items.is_empty() { render_generic_fence("timeline", body) }
            else { illustrate::render_timeline(&items) }
        }
        "comparison" => {
            let left = props.iter().find(|(k, _)| *k == "left").map(|(_, v)| *v).unwrap_or("A");
            let right = props.iter().find(|(k, _)| *k == "right").map(|(_, v)| *v).unwrap_or("B");
            let rows: Vec<(String, String)> = body.lines()
                .filter(|l| l.trim().starts_with("- "))
                .filter_map(|l| {
                    let s = l.trim().trim_start_matches("- ").trim();
                    s.split_once(" | ").map(|(a, b)| (a.to_owned(), b.to_owned()))
                })
                .collect();
            if rows.is_empty() { render_generic_fence("comparison", body) }
            else { illustrate::render_comparison(left, right, &rows) }
        }
        "tip" => {
            let icon = props.iter().find(|(k, _)| *k == "icon").map(|(_, v)| *v).unwrap_or("");
            illustrate::render_tip(icon, body.trim())
        }
        _ => {
            // Unknown block — render as a styled container
            render_generic_fence(name, body)
        }
    }
}

fn render_book_info(props: &[(&str, &str)]) -> String {
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
    html.push_str("<section style=\"margin: 24px 0; background: #fff; border: 1px solid #e8e8e8; border-radius: 6px; overflow: hidden;\">\n");
    html.push_str("<table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\"><tr>\n");

    if has_cover {
        html.push_str(&format!(
            "<td style=\"width:90px;padding:16px;vertical-align:top;\"><img src=\"{cover}\" style=\"width:90px;height:auto;border-radius:4px;box-shadow:0 2px 8px rgba(0,0,0,0.12);\" /></td>\n"
        ));
    }
    html.push_str("<td style=\"padding:16px;vertical-align:middle;\">\n");
    html.push_str(&format!("<p style=\"margin:0 0 6px;font-size:16px;font-weight:bold;color:#1a1a1a;\">《{title}》</p>\n"));
    if !author.is_empty() {
        html.push_str(&format!(
            "<p style=\"margin:0 0 4px;font-size:13px;color:#888;\">{author} 著</p>\n"
        ));
    }
    if !publisher.is_empty() || !rating.is_empty() {
        let pub_str = if rating.is_empty() {
            publisher.to_owned()
        } else {
            format!("{publisher} | 豆瓣 {rating}")
        };
        html.push_str(&format!(
            "<p style=\"margin:0;font-size:12px;color:#aaa;\">{pub_str}</p>\n"
        ));
    }
    html.push_str("</td>\n");
    html.push_str("</tr></table>\n");
    html.push_str("</section>\n\n");
    html
}

fn render_intro(body: &str) -> String {
    format!(
        "<section style=\"margin: 20px 0; padding: 16px 20px; background: linear-gradient(135deg, #fafafa, #f5f5f5); border-left: 4px solid #2c2c2c; font-size: 15px; color: #555; line-height: 1.85;\">\n{}\n</section>\n\n",
        inline_md(body.trim())
    )
}

fn render_callout(props: &[(&str, &str)], body: &str) -> String {
    let label = props
        .iter()
        .find(|(k, _)| *k == "label")
        .map(|(_, v)| *v)
        .unwrap_or("重点");
    format!(
        "<section style=\"margin: 18px 0;\"><table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\"><tr>\n<td style=\"background:#1a1a1a;color:#fff;font-weight:bold;font-size:13px;padding:10px 14px;white-space:nowrap;letter-spacing:1px;vertical-align:top;\">{label}</td>\n<td style=\"background:#fff;border:1px solid #1a1a1a;border-left:none;padding:12px 16px;font-size:14px;line-height:1.8;color:#1a1a1a;\">{}</td>\n</tr></table></section>\n\n",
        inline_md(body.trim())
    )
}

fn render_steps(body: &str) -> String {
    let items: Vec<&str> = body
        .lines()
        .filter(|l| l.trim().starts_with(|c: char| c.is_ascii_digit()) && l.trim().contains(". "))
        .filter_map(|l| l.trim().split_once(". ").map(|(_, rest)| rest))
        .collect();

    if items.is_empty() {
        return render_generic_fence("steps", body);
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
            "<td style=\"width:{pct}%;background:#fff;border:1px solid #e8e8e8;padding:14px 12px;vertical-align:top;\">\n<section style=\"display:inline-block;width:24px;height:24px;background:#2c2c2c;color:#fff;font-weight:bold;text-align:center;line-height:24px;border-radius:50%;font-size:13px;margin-bottom:8px;\">{}</section>\n<p style=\"margin:0;font-size:13px;color:#555;line-height:1.7;\">{}</p>\n</td>\n",
            i + 1,
            inline_md(item),
        ));
    }
    html.push_str("</tr></table></section>\n\n");
    html
}

fn render_summary(body: &str) -> String {
    format!(
        "<section style=\"margin: 24px 0;\"><table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\"><tr>\n<td style=\"background:#1a1a1a;color:#fff;font-weight:bold;font-size:13px;padding:10px 14px;white-space:nowrap;letter-spacing:1px;vertical-align:top;\">总 结</td>\n<td style=\"background:#fff;border:1px solid #1a1a1a;border-left:none;padding:12px 16px;font-size:14px;line-height:1.8;color:#1a1a1a;\">{}</td>\n</tr></table></section>\n\n",
        inline_md(body.trim())
    )
}

fn render_figure(props: &[(&str, &str)]) -> String {
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
            "<p style=\"margin:0;padding:10px 14px;background:#f8f8f8;color:#888;font-size:12px;text-align:center;\">{caption}</p>"
        )
    };
    format!(
        "<section style=\"margin: 24px 0;\"><section style=\"border:2px solid #e8e8e8;padding:0;background:#fafafa;\">\n<img src=\"{image}\" style=\"display:block;width:100%;height:auto;\" />\n{cap_html}</section></section>\n\n"
    )
}

fn render_checklist(body: &str) -> String {
    let items: Vec<&str> = body
        .lines()
        .filter(|l| l.trim().starts_with("- [") || l.trim().starts_with("- ["))
        .collect();
    if items.is_empty() {
        return render_generic_fence("checklist", body);
    }
    let mut html = String::new();
    html.push_str("<section style=\"margin:18px 0;\"><section style=\"background:#fff;border:1px solid #e8e8e8;padding:18px 20px;\"><table cellpadding=\"0\" cellspacing=\"0\" border=\"0\" style=\"border-collapse:collapse;width:100%;\">\n");
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
                        "<span style=\"color:#2c2c2c;font-weight:bold;\">✔</span>&nbsp;&nbsp;{content}"
                    )
                } else {
                    let content = item[1..].trim();
                    format!(
                        "<span style=\"color:#ccc;font-weight:bold;\">○</span>&nbsp;&nbsp;{content}"
                    )
                };
                html.push_str(&format!(
                    "<td style=\"width:50%;padding:6px 0;font-size:14px;color:#555;vertical-align:top;\">{rest}</td>\n"
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

fn render_cover(props: &[(&str, &str)]) -> String {
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
        "<section style=\"margin:0;background:#1a1a1a;padding:48px 24px 36px;color:#fff;\">\n<section style=\"display:inline-block;background:#fff;color:#1a1a1a;font-size:11px;font-weight:bold;letter-spacing:2px;padding:4px 10px;margin-bottom:18px;\">READING · NOTES</section>\n<h1 style=\"margin:0 0 8px;font-size:28px;font-weight:900;line-height:1.2;color:#fff;\">{title}</h1>\n<p style=\"margin:8px 0 0;font-size:14px;color:#aaa;\">{subtitle}</p>\n</section>\n\n"
    )
}

fn render_generic_fence(_name: &str, body: &str) -> String {
    format!(
        "<section style=\"margin: 18px 0; padding: 16px 20px; background: #fafafa; border: 1px solid #e8e8e8; border-radius: 4px;\">\n{}\n</section>\n\n",
        inline_md(body.trim())
    )
}

// ── Plain markdown segment renderer ───────────────────────────────────────────

fn render_markdown_segment(md: &str) -> String {
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
            out.push_str(&render_blockquote(&blockquote_buf));
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
            out.push_str(&render_h3(rest));
            continue;
        }
        if let Some(rest) = line.strip_prefix("## ") {
            out.push_str(&render_h2(rest));
            continue;
        }
        if let Some(rest) = line.strip_prefix("# ") {
            out.push_str(&render_h2(rest));
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

        out.push_str(&render_p(line));
    }

    if in_blockquote {
        out.push_str(&render_blockquote(&blockquote_buf));
    }

    out
}

fn inline_md(text: &str) -> String {
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
                    "<strong style=\"color: #1a1a1a;\">{}</strong>",
                    inline_md(&inner)
                ));
                i += rel + 4;
                continue;
            }
        }
        if chars[i] == '*' {
            let end = chars[i + 1..].iter().position(|&c| c == '*');
            if let Some(rel) = end {
                let inner: String = chars[i + 1..i + 1 + rel].iter().collect();
                s.push_str(&format!("<em>{}</em>", inline_md(&inner)));
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

fn render_h2(text: &str) -> String {
    format!(
        "<h2 style=\"font-size: 18px; font-weight: bold; color: #1a1a1a; margin: 1.8em 0 0.8em; padding-left: 12px; border-left: 4px solid #2c2c2c;\">{}</h2>\n\n",
        inline_md(text)
    )
}

fn render_h3(text: &str) -> String {
    format!(
        "<h3 style=\"font-size: 16px; font-weight: bold; color: #1a1a1a; margin: 1.5em 0 0.6em;\">{}</h3>\n\n",
        inline_md(text)
    )
}

fn render_p(text: &str) -> String {
    format!(
        "<p style=\"margin: 1.2em 0; color: #555; font-size: 15px;\">{}</p>\n\n",
        inline_md(text)
    )
}

fn render_blockquote(text: &str) -> String {
    format!(
        "<blockquote style=\"margin: 1.5em 0; padding: 16px 20px; background: #f8f8f8; border-left: 4px solid #2c2c2c; color: #444; font-size: 15px; line-height: 1.8;\">{}</blockquote>\n\n",
        inline_md(text)
    )
}

fn wrap_wechat_html(body: &str) -> String {
    // 结尾由「寻月阁标准结尾」模板在微信后台插入，不在此处硬编码 footer
    format!(
        "<section style=\"font-family: -apple-system, 'PingFang SC', 'Hiragino Sans GB', 'Microsoft YaHei', sans-serif; font-size: 16px; line-height: 1.8; color: #333; padding: 0 4px;\">\n\n{body}\n\n</section>\n"
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

// ── JSONL serialisation ───────────────────────────────────────────────────────

impl TrendSample {
    fn to_json_line(&self) -> String {
        let fields = [
            json_string_field("platform", &self.platform),
            json_string_field("keyword", &self.keyword),
            json_string_field("title", &self.title),
            json_optional_string_field("url", self.url.as_deref()),
            json_optional_string_field("author", self.author.as_deref()),
            json_optional_u64_field("likes", self.likes),
            json_optional_u64_field("collects", self.collects),
            json_optional_u64_field("comments", self.comments),
            json_string_field("source", &self.source),
        ];
        format!("{{{}}}", fields.join(","))
    }

    fn from_json_line(line: &str) -> Option<Self> {
        Some(Self {
            platform: extract_json_string(line, "platform")?,
            keyword: extract_json_string(line, "keyword")?,
            title: extract_json_string(line, "title")?,
            url: extract_json_optional_string(line, "url"),
            author: extract_json_optional_string(line, "author"),
            likes: extract_json_optional_u64(line, "likes"),
            collects: extract_json_optional_u64(line, "collects"),
            comments: extract_json_optional_u64(line, "comments"),
            source: extract_json_string(line, "source")?,
        })
    }
}

fn json_string_field(name: &str, value: &str) -> String {
    format!("\"{name}\":\"{}\"", escape_json(value))
}

fn json_optional_string_field(name: &str, value: Option<&str>) -> String {
    value.map_or_else(
        || format!("\"{name}\":null"),
        |value| json_string_field(name, value),
    )
}

fn json_optional_u64_field(name: &str, value: Option<u64>) -> String {
    value.map_or_else(
        || format!("\"{name}\":null"),
        |value| format!("\"{name}\":{value}"),
    )
}

fn escape_json(value: &str) -> String {
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

fn extract_json_string(line: &str, name: &str) -> Option<String> {
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

fn extract_json_optional_string(line: &str, name: &str) -> Option<String> {
    if line.contains(&format!("\"{name}\":null")) {
        None
    } else {
        extract_json_string(line, name)
    }
}

fn extract_json_optional_u64(line: &str, name: &str) -> Option<u64> {
    let marker = format!("\"{name}\":");
    let start = line.find(&marker)? + marker.len();
    let value = line[start..].split([',', '}']).next().unwrap_or("").trim();
    if value == "null" {
        None
    } else {
        value.parse().ok()
    }
}

fn format_trend_samples(samples: &[TrendSample]) -> String {
    let mut output = String::from("trend samples\n");
    if samples.is_empty() {
        output.push_str("  (empty)");
        return output;
    }

    for sample in samples {
        output.push_str(&format!(
            "  [{}] {} | {}",
            sample.platform, sample.keyword, sample.title
        ));
        if let Some(likes) = sample.likes {
            output.push_str(&format!(" | likes={likes}"));
        }
        if let Some(collects) = sample.collects {
            output.push_str(&format!(" | collects={collects}"));
        }
        if let Some(comments) = sample.comments {
            output.push_str(&format!(" | comments={comments}"));
        }
        if let Some(url) = &sample.url {
            output.push_str(&format!(" | {url}"));
        }
        output.push('\n');
    }
    output.trim_end().to_owned()
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

        render_article(&root, &md_path, "寻月隐君", "thumb123")?;

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

        render_article(&root, &md_path, "作者", "")?;

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

        render_article(&root, &md_path, "a", "")?;

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

        let cfg = Config {
            vault_root: None,
            wechat_appid: None,
            wechat_author: None,
            wechat_thumb_media_id: None,
            wechat_account_type: None,
            wechat_auto_publish: false,
            blog_kind: None,
            blog_root: None,
        };
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
            vault_root: None,
            wechat_appid: None,
            wechat_author: Some("寻月隐君".to_owned()),
            wechat_thumb_media_id: Some("thumb_abc".to_owned()),
            wechat_account_type: None,
            wechat_auto_publish: false,
            blog_kind: None,
            blog_root: None,
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
