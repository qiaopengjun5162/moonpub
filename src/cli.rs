use std::path::PathBuf;

use crate::config::Config;
use crate::error::AppError;
use crate::radar::{RadarCommand, parse_radar_command};

const DEFAULT_CONFIG: &str = "moonpub.toml";

/// Consume the next argument as a value for a named flag (e.g., "--style dark").
pub fn flag_value(
    extra: &mut std::slice::Iter<String>,
    name: &'static str,
) -> Result<String, AppError> {
    let v = extra.next().ok_or(AppError::MissingValue(name))?;
    Ok(v.clone())
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Options {
    pub articles: PathBuf,
    pub command: Command,
    pub json: bool,
    pub config: Option<PathBuf>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Command {
    Init {
        path: PathBuf,
    },
    Doctor,
    Workspace,
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
        temporary_profile: bool,
    },
    Publish {
        article: PathBuf,
        target: String,
        auto_render: bool,
        temporary_profile: bool,
    },
    UpdateDraft {
        article: PathBuf,
        media_id: Option<String>,
    },
    Export {
        article: PathBuf,
        target: Option<String>,
    },
    Preview {
        article: PathBuf,
        open: bool,
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
    Login {
        temporary_profile: bool,
    },
    WechatHealth {
        headed: bool,
        temporary_profile: bool,
    },
    Configure {
        steps: Vec<String>,
        headed: bool,
        temporary_profile: bool,
    },
    StepTest {
        headed: bool,
        temporary_profile: bool,
    },
    TestZanshang {
        headed: bool,
        temporary_profile: bool,
    },
    TestChuangzuo {
        headed: bool,
        temporary_profile: bool,
    },
    TestYulan {
        headed: bool,
        temporary_profile: bool,
    },
    ListDrafts,
    DeleteDraft {
        media_id: String,
    },
    Ship {
        article: PathBuf,
        style: Option<String>,
    },
    New {
        title: String,
    },
    Write {
        idea: String,
    },
    DraftFromInbox {
        input: PathBuf,
        preview: PreviewOptions,
        auto_push: bool,
    },
    Polish {
        article: PathBuf,
    },
    Expand {
        article: PathBuf,
    },
    ShipAi {
        article: PathBuf,
        style: Option<String>,
    },
    IntakeFeishu {
        source: FeishuIntakeSource,
        draft: bool,
        preview: PreviewOptions,
        auto_push: bool,
    },
    IntakePhotos {
        inputs: Vec<PathBuf>,
        draft: bool,
        preview: PreviewOptions,
        auto_push: bool,
    },
    LayoutRecipes,
    LayoutAudit {
        html: PathBuf,
    },
    Radar(RadarCommand),
    Capabilities,
    Version,
    Help,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeishuIntakeSource {
    File(PathBuf),
    MinuteToken(String),
    Latest,
    Query(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct PreviewOptions {
    pub enabled: bool,
    pub open: bool,
}

impl Options {
    pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Self, AppError> {
        let mut articles_dir = std::env::current_dir().map_err(|source| AppError::Io {
            path: PathBuf::from("."),
            source,
        })?;
        let mut rest = Vec::new();
        let mut json = false;
        let mut config: Option<PathBuf> = None;
        let mut args = args.into_iter();

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--articles" => {
                    let value = args.next().ok_or(AppError::MissingValue("--articles"))?;
                    articles_dir = PathBuf::from(value);
                }
                "--config" => {
                    let value = args.next().ok_or(AppError::MissingValue("--config"))?;
                    config = Some(PathBuf::from(value));
                }
                "--json" => json = true,
                "-h" | "--help" => {
                    return Ok(Self {
                        articles: articles_dir,
                        command: Command::Help,
                        json,
                        config,
                    });
                }
                "-V" | "--version" => {
                    return Ok(Self {
                        articles: articles_dir,
                        command: Command::Version,
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

        // Apply config file: if --config is given, load it and override articles_dir.
        // Otherwise auto-discover moonpub.toml:
        //   1. articles root (if --articles was given or cwd is the articles root)
        //   2. walk up from the first article-like arg to find the articles root
        //
        // WHY: Users run `moonpub render Articles/drafts/x.md` from outside the articles root.
        // Requiring an explicit `--config` every time is error-prone and led to silent
        // fallback to default config (wrong author/theme). Auto-discovery fixes that.
        if let Some(cfg_path) = &config {
            let cfg = Config::load(cfg_path)?;
            if let Some(root) = cfg.articles_root {
                articles_dir = root;
            }
        } else {
            // First try cwd/articles root path.
            let auto = articles_dir.join("moonpub.toml");
            if auto.exists() {
                config = Some(auto.clone());
                let cfg = Config::load(&auto)?;
                if let Some(root) = cfg.articles_root {
                    articles_dir = root;
                }
            } else {
                // Walk up from the first non-flag argument after the subcommand (likely the article path).
                let first_arg = rest
                    .iter()
                    .skip(1)
                    .find(|s| !s.starts_with('-'))
                    .map(PathBuf::from);
                if let Some(arg_path) = first_arg {
                    let abs = if arg_path.is_absolute() {
                        arg_path
                    } else {
                        articles_dir.join(arg_path)
                    };
                    let mut cur = abs.parent().map(|p| p.to_path_buf());
                    while let Some(dir) = cur {
                        let candidate = dir.join("moonpub.toml");
                        if candidate.exists() {
                            let cfg = Config::load(&candidate)?;
                            config = Some(candidate);
                            if let Some(root) = cfg.articles_root {
                                articles_dir = root;
                            } else {
                                articles_dir = dir;
                            }
                            break;
                        }
                        cur = dir.parent().map(|p| p.to_path_buf());
                    }
                }
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
                articles: articles_dir,
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
                    .unwrap_or_else(|| articles_dir.join(DEFAULT_CONFIG));
                Command::Init { path }
            }
            "new" => {
                let title = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("new <title>"))?
                    .clone();
                Command::New { title }
            }
            "write" => {
                let idea = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("write <idea>"))?
                    .clone();
                Command::Write { idea }
            }
            "draft-from-inbox" => {
                let value = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("draft-from-inbox <inbox.md>"))?;
                let mut enabled = false;
                let mut no_open = false;
                let mut auto_push = false;
                for flag in &rest[2..] {
                    match flag.as_str() {
                        "--preview" => enabled = true,
                        "--no-open" => no_open = true,
                        "--push" => auto_push = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                if no_open && !enabled {
                    return Err(AppError::MissingValue(
                        "draft-from-inbox --no-open requires --preview",
                    ));
                }
                if auto_push && enabled {
                    return Err(AppError::MissingValue(
                        "draft-from-inbox --push conflicts with --preview",
                    ));
                }
                let preview = PreviewOptions {
                    enabled,
                    open: enabled && !no_open,
                };
                Command::DraftFromInbox {
                    input: PathBuf::from(value),
                    preview,
                    auto_push,
                }
            }
            "polish" => {
                let value = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("polish <article.md>"))?;
                Command::Polish {
                    article: PathBuf::from(value),
                }
            }
            "expand" => {
                let value = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("expand <article.md>"))?;
                Command::Expand {
                    article: PathBuf::from(value),
                }
            }
            "workspace" => Command::Workspace,
            "doctor" => {
                for flag in &rest[1..] {
                    match flag.as_str() {
                        "--json" => json = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::Doctor
            }
            "layout-recipes" => Command::LayoutRecipes,
            "layout-audit" => {
                let value = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("layout-audit <html>"))?;
                Command::LayoutAudit {
                    html: PathBuf::from(value),
                }
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
                let mut temporary_profile = false;
                let extra = rest[2..].iter();
                for flag in extra {
                    match flag.as_str() {
                        "--render" => auto_render = true,
                        "--temporary-profile" => temporary_profile = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::Push {
                    article: PathBuf::from(value),
                    auto_render,
                    temporary_profile,
                }
            }
            "publish" => {
                let value = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("publish <article.md>"))?;
                let mut target = None;
                let mut auto_render = false;
                let mut temporary_profile = false;
                let mut extra = rest[2..].iter();
                while let Some(flag) = extra.next() {
                    match flag.as_str() {
                        "--target" => {
                            target = Some(flag_value(&mut extra, "--target")?);
                        }
                        "--render" => auto_render = true,
                        "--temporary-profile" => temporary_profile = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::Publish {
                    article: PathBuf::from(value),
                    target: target.ok_or(AppError::MissingValue("--target"))?,
                    auto_render,
                    temporary_profile,
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
                let mut with_ai = false;
                let mut extra = rest[2..].iter();
                while let Some(flag) = extra.next() {
                    match flag.as_str() {
                        "--style" => {
                            style = Some(flag_value(&mut extra, "--style")?);
                        }
                        "--ai" => with_ai = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        _ => {}
                    }
                }
                if with_ai {
                    Command::ShipAi {
                        article: PathBuf::from(value),
                        style,
                    }
                } else {
                    Command::Ship {
                        article: PathBuf::from(value),
                        style,
                    }
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
            "login" => {
                let mut temporary_profile = false;
                for flag in &rest[1..] {
                    match flag.as_str() {
                        "--temporary-profile" => temporary_profile = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::Login { temporary_profile }
            }
            "wechat-health" => {
                let mut headed = false;
                let mut temporary_profile = false;
                for flag in &rest[1..] {
                    match flag.as_str() {
                        "--headed" => headed = true,
                        "--temporary-profile" => temporary_profile = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::WechatHealth {
                    headed,
                    temporary_profile,
                }
            }
            "configure" => {
                let mut headed = false;
                let mut temporary_profile = false;
                let mut steps = Vec::new();
                for arg in &rest[1..] {
                    match arg.as_str() {
                        "--headed" => headed = true,
                        "--temporary-profile" => temporary_profile = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        s => steps.push(s.to_owned()),
                    }
                }
                Command::Configure {
                    steps,
                    headed,
                    temporary_profile,
                }
            }
            "step-test" => {
                let mut headed = false;
                let mut temporary_profile = false;
                for flag in &rest[1..] {
                    match flag.as_str() {
                        "--headed" => headed = true,
                        "--temporary-profile" => temporary_profile = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::StepTest {
                    headed,
                    temporary_profile,
                }
            }
            "test-zanshang" => {
                let mut headed = false;
                let mut temporary_profile = false;
                for flag in &rest[1..] {
                    match flag.as_str() {
                        "--headed" => headed = true,
                        "--temporary-profile" => temporary_profile = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::TestZanshang {
                    headed,
                    temporary_profile,
                }
            }
            "test-chuangzuo" => {
                let mut headed = false;
                let mut temporary_profile = false;
                for flag in &rest[1..] {
                    match flag.as_str() {
                        "--headed" => headed = true,
                        "--temporary-profile" => temporary_profile = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::TestChuangzuo {
                    headed,
                    temporary_profile,
                }
            }
            "test-yulan" => {
                let mut headed = false;
                let mut temporary_profile = false;
                for flag in &rest[1..] {
                    match flag.as_str() {
                        "--headed" => headed = true,
                        "--temporary-profile" => temporary_profile = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::TestYulan {
                    headed,
                    temporary_profile,
                }
            }
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
                let mut target = None;
                let mut extra = rest[2..].iter();
                while let Some(flag) = extra.next() {
                    match flag.as_str() {
                        "--target" => {
                            target = Some(flag_value(&mut extra, "--target")?);
                        }
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::Export {
                    article: PathBuf::from(value),
                    target,
                }
            }
            "preview" => {
                let value = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("preview <article.md>"))?;
                let mut open = true;
                for flag in &rest[2..] {
                    match flag.as_str() {
                        "--no-open" => open = false,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::Preview {
                    article: PathBuf::from(value),
                    open,
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
            "intake" => {
                let source = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("intake <source> <file>"))?;
                match source.as_str() {
                    "feishu" => {
                        let input = rest
                            .get(2)
                            .ok_or(AppError::MissingValue("intake feishu <file>"))?;
                        let source = if input == "--minute-token" {
                            let token = rest.get(3).ok_or(AppError::MissingValue(
                                "intake feishu --minute-token <token>",
                            ))?;
                            FeishuIntakeSource::MinuteToken(token.to_owned())
                        } else if input == "--latest" {
                            FeishuIntakeSource::Latest
                        } else if input == "--query" {
                            let query = rest
                                .get(3)
                                .ok_or(AppError::MissingValue("intake feishu --query <keyword>"))?;
                            FeishuIntakeSource::Query(query.to_owned())
                        } else {
                            if input.starts_with('-') {
                                return Err(AppError::UnknownOption(input.to_owned()));
                            }
                            FeishuIntakeSource::File(PathBuf::from(input))
                        };
                        let flag_start = match &source {
                            FeishuIntakeSource::File(_) | FeishuIntakeSource::Latest => 3,
                            FeishuIntakeSource::MinuteToken(_) | FeishuIntakeSource::Query(_) => 4,
                        };
                        let (draft, preview, auto_push) =
                            parse_intake_feishu_flags(&rest[flag_start..])?;
                        Command::IntakeFeishu {
                            source,
                            draft,
                            preview,
                            auto_push,
                        }
                    }
                    "photos" => {
                        if rest.len() < 3 {
                            return Err(AppError::MissingValue(
                                "intake photos <file-or-dir> [more files or dirs]",
                            ));
                        }
                        let mut inputs = Vec::new();
                        let mut flag_start = rest.len();
                        for (index, value) in rest.iter().enumerate().skip(2) {
                            if value.starts_with('-') {
                                flag_start = index;
                                break;
                            }
                            inputs.push(PathBuf::from(value));
                        }
                        if inputs.is_empty() {
                            return Err(AppError::MissingValue(
                                "intake photos <file-or-dir> [more files or dirs]",
                            ));
                        }
                        let (draft, preview, auto_push) =
                            parse_intake_feishu_flags(&rest[flag_start..])?;
                        Command::IntakePhotos {
                            inputs,
                            draft,
                            preview,
                            auto_push,
                        }
                    }
                    other => return Err(AppError::UnknownCommand(format!("intake {other}"))),
                }
            }
            "capabilities" => {
                for flag in &rest[1..] {
                    match flag.as_str() {
                        "--json" => json = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::Capabilities
            }
            "version" => Command::Version,
            "help" => Command::Help,
            value => return Err(AppError::UnknownCommand(value.to_owned())),
        };

        Ok(Self {
            articles: articles_dir,
            command,
            json,
            config,
        })
    }
}

fn parse_intake_feishu_flags(flags: &[String]) -> Result<(bool, PreviewOptions, bool), AppError> {
    let mut draft = false;
    let mut preview_enabled = false;
    let mut no_open = false;
    let mut auto_push = false;
    for flag in flags {
        match flag.as_str() {
            "--draft" => draft = true,
            "--preview" => preview_enabled = true,
            "--no-open" => no_open = true,
            "--push" => auto_push = true,
            v if v.starts_with('-') => return Err(AppError::UnknownOption(v.to_owned())),
            v => return Err(AppError::UnknownCommand(v.to_owned())),
        }
    }
    if preview_enabled && !draft {
        return Err(AppError::MissingValue(
            "intake feishu --preview requires --draft",
        ));
    }
    if no_open && !preview_enabled {
        return Err(AppError::MissingValue(
            "intake feishu --no-open requires --preview",
        ));
    }
    if auto_push && !draft {
        return Err(AppError::MissingValue(
            "intake feishu --push requires --draft",
        ));
    }
    if auto_push && preview_enabled {
        return Err(AppError::MissingValue(
            "intake feishu --push conflicts with --preview",
        ));
    }
    let preview = PreviewOptions {
        enabled: preview_enabled,
        open: preview_enabled && !no_open,
    };
    Ok((draft, preview, auto_push))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::cli::{Command, FeishuIntakeSource, Options, PreviewOptions};

    #[test]
    fn parses_status_with_articles() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "--articles".to_owned(),
            "/tmp/articles".to_owned(),
            "status".to_owned(),
        ])?;

        assert_eq!(options.articles, PathBuf::from("/tmp/articles"));
        assert_eq!(options.command, Command::Status);
        Ok(())
    }

    #[test]
    fn parses_workspace_with_articles() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "--articles".to_owned(),
            "/tmp/articles".to_owned(),
            "workspace".to_owned(),
        ])?;

        assert_eq!(options.articles, PathBuf::from("/tmp/articles"));
        assert_eq!(options.command, Command::Workspace);
        Ok(())
    }

    #[test]
    fn parses_doctor_with_articles_and_json() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "--articles".to_owned(),
            "/tmp/articles".to_owned(),
            "--json".to_owned(),
            "doctor".to_owned(),
        ])?;

        assert_eq!(options.articles, PathBuf::from("/tmp/articles"));
        assert_eq!(options.command, Command::Doctor);
        assert!(options.json);
        Ok(())
    }

    #[test]
    fn parses_json_flag() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "--articles".to_owned(),
            "/tmp/articles".to_owned(),
            "--json".to_owned(),
            "status".to_owned(),
        ])?;

        assert!(options.json);
        Ok(())
    }

    #[test]
    fn parses_capabilities_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["capabilities".to_owned()])?;

        assert_eq!(options.command, Command::Capabilities);
        assert!(!options.json);
        Ok(())
    }

    #[test]
    fn parses_capabilities_command_json_flag() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["capabilities".to_owned(), "--json".to_owned()])?;

        assert_eq!(options.command, Command::Capabilities);
        assert!(options.json);
        Ok(())
    }

    #[test]
    fn parses_layout_recipes_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["layout-recipes".to_owned()])?;

        assert_eq!(options.command, Command::LayoutRecipes);
        assert!(!options.json);
        Ok(())
    }

    #[test]
    fn parses_layout_audit_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["layout-audit".to_owned(), "demo.html".to_owned()])?;

        assert_eq!(
            options.command,
            Command::LayoutAudit {
                html: PathBuf::from("demo.html")
            }
        );
        Ok(())
    }

    #[test]
    fn parses_intake_feishu_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "intake".to_owned(),
            "feishu".to_owned(),
            "minutes.txt".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::IntakeFeishu {
                source: FeishuIntakeSource::File(PathBuf::from("minutes.txt")),
                draft: false,
                preview: PreviewOptions::default(),
                auto_push: false,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_intake_feishu_file_with_draft_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "intake".to_owned(),
            "feishu".to_owned(),
            "minutes.txt".to_owned(),
            "--draft".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::IntakeFeishu {
                source: FeishuIntakeSource::File(PathBuf::from("minutes.txt")),
                draft: true,
                preview: PreviewOptions::default(),
                auto_push: false,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_intake_feishu_file_with_draft_preview_command()
    -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "intake".to_owned(),
            "feishu".to_owned(),
            "minutes.txt".to_owned(),
            "--draft".to_owned(),
            "--preview".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::IntakeFeishu {
                source: FeishuIntakeSource::File(PathBuf::from("minutes.txt")),
                draft: true,
                preview: PreviewOptions {
                    enabled: true,
                    open: true,
                },
                auto_push: false,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_intake_feishu_file_with_draft_preview_no_open_command()
    -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "intake".to_owned(),
            "feishu".to_owned(),
            "minutes.txt".to_owned(),
            "--draft".to_owned(),
            "--preview".to_owned(),
            "--no-open".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::IntakeFeishu {
                source: FeishuIntakeSource::File(PathBuf::from("minutes.txt")),
                draft: true,
                preview: PreviewOptions {
                    enabled: true,
                    open: false,
                },
                auto_push: false,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_intake_feishu_file_with_draft_push_command() -> Result<(), Box<dyn std::error::Error>>
    {
        let options = Options::parse([
            "intake".to_owned(),
            "feishu".to_owned(),
            "minutes.txt".to_owned(),
            "--draft".to_owned(),
            "--push".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::IntakeFeishu {
                source: FeishuIntakeSource::File(PathBuf::from("minutes.txt")),
                draft: true,
                preview: PreviewOptions::default(),
                auto_push: true,
            }
        );
        Ok(())
    }

    #[test]
    fn intake_feishu_preview_requires_draft() {
        let err = Options::parse([
            "intake".to_owned(),
            "feishu".to_owned(),
            "minutes.txt".to_owned(),
            "--preview".to_owned(),
        ]);

        assert!(err.is_err());
    }

    #[test]
    fn intake_feishu_no_open_requires_preview() {
        let err = Options::parse([
            "intake".to_owned(),
            "feishu".to_owned(),
            "minutes.txt".to_owned(),
            "--draft".to_owned(),
            "--no-open".to_owned(),
        ]);

        assert!(err.is_err());
    }

    #[test]
    fn intake_feishu_push_requires_draft() {
        let err = Options::parse([
            "intake".to_owned(),
            "feishu".to_owned(),
            "minutes.txt".to_owned(),
            "--push".to_owned(),
        ]);

        assert!(err.is_err());
    }

    #[test]
    fn intake_feishu_push_conflicts_with_preview() {
        let err = Options::parse([
            "intake".to_owned(),
            "feishu".to_owned(),
            "minutes.txt".to_owned(),
            "--draft".to_owned(),
            "--preview".to_owned(),
            "--push".to_owned(),
        ]);

        assert!(err.is_err());
    }

    #[test]
    fn parses_intake_feishu_minute_token_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "intake".to_owned(),
            "feishu".to_owned(),
            "--minute-token".to_owned(),
            "obcn123".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::IntakeFeishu {
                source: FeishuIntakeSource::MinuteToken("obcn123".to_owned()),
                draft: false,
                preview: PreviewOptions::default(),
                auto_push: false,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_intake_feishu_latest_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "intake".to_owned(),
            "feishu".to_owned(),
            "--latest".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::IntakeFeishu {
                source: FeishuIntakeSource::Latest,
                draft: false,
                preview: PreviewOptions::default(),
                auto_push: false,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_intake_feishu_latest_with_draft_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "intake".to_owned(),
            "feishu".to_owned(),
            "--latest".to_owned(),
            "--draft".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::IntakeFeishu {
                source: FeishuIntakeSource::Latest,
                draft: true,
                preview: PreviewOptions::default(),
                auto_push: false,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_intake_feishu_query_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "intake".to_owned(),
            "feishu".to_owned(),
            "--query".to_owned(),
            "散步".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::IntakeFeishu {
                source: FeishuIntakeSource::Query("散步".to_owned()),
                draft: false,
                preview: PreviewOptions::default(),
                auto_push: false,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_intake_feishu_query_with_draft_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "intake".to_owned(),
            "feishu".to_owned(),
            "--query".to_owned(),
            "散步".to_owned(),
            "--draft".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::IntakeFeishu {
                source: FeishuIntakeSource::Query("散步".to_owned()),
                draft: true,
                preview: PreviewOptions::default(),
                auto_push: false,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_intake_photos_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "intake".to_owned(),
            "photos".to_owned(),
            "photos/day1".to_owned(),
            "photos/day2/a.jpg".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::IntakePhotos {
                inputs: vec![
                    PathBuf::from("photos/day1"),
                    PathBuf::from("photos/day2/a.jpg"),
                ],
                draft: false,
                preview: PreviewOptions::default(),
                auto_push: false,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_intake_photos_with_draft_preview_no_open_command()
    -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "intake".to_owned(),
            "photos".to_owned(),
            "photos/day1".to_owned(),
            "--draft".to_owned(),
            "--preview".to_owned(),
            "--no-open".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::IntakePhotos {
                inputs: vec![PathBuf::from("photos/day1")],
                draft: true,
                preview: PreviewOptions {
                    enabled: true,
                    open: false,
                },
                auto_push: false,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_version_flag() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["--version".to_owned()])?;

        assert_eq!(options.command, Command::Version);
        Ok(())
    }

    #[test]
    fn parses_push_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "--articles".to_owned(),
            "/tmp/articles".to_owned(),
            "push".to_owned(),
            "Articles/ready/demo.md".to_owned(),
            "--render".to_owned(),
            "--temporary-profile".to_owned(),
        ])?;
        let Command::Push {
            article,
            auto_render,
            temporary_profile,
        } = options.command
        else {
            panic!("expected Push");
        };
        assert_eq!(article, PathBuf::from("Articles/ready/demo.md"));
        assert!(auto_render);
        assert!(temporary_profile);
        Ok(())
    }

    #[test]
    fn parses_preview_no_open_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "preview".to_owned(),
            "Articles/drafts/demo.md".to_owned(),
            "--no-open".to_owned(),
        ])?;
        let Command::Preview { article, open } = options.command else {
            panic!("expected Preview");
        };
        assert_eq!(article, PathBuf::from("Articles/drafts/demo.md"));
        assert!(!open);
        Ok(())
    }

    #[test]
    fn parses_publish_target_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "publish".to_owned(),
            "Articles/ready/demo.md".to_owned(),
            "--target".to_owned(),
            "wechat-draft".to_owned(),
            "--render".to_owned(),
            "--temporary-profile".to_owned(),
        ])?;
        let Command::Publish {
            article,
            target,
            auto_render,
            temporary_profile,
        } = options.command
        else {
            panic!("expected Publish");
        };
        assert_eq!(article, PathBuf::from("Articles/ready/demo.md"));
        assert_eq!(target, "wechat-draft");
        assert!(auto_render);
        assert!(temporary_profile);
        Ok(())
    }

    #[test]
    fn parses_export_command() -> Result<(), Box<dyn std::error::Error>> {
        let options =
            Options::parse(["export".to_owned(), "Articles/published/demo.md".to_owned()])?;
        let Command::Export { article, target } = options.command else {
            panic!("expected Export");
        };
        assert_eq!(article, PathBuf::from("Articles/published/demo.md"));
        assert_eq!(target, None);
        Ok(())
    }

    #[test]
    fn parses_export_target_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "export".to_owned(),
            "Articles/published/demo.md".to_owned(),
            "--target".to_owned(),
            "zola".to_owned(),
        ])?;
        let Command::Export { article, target } = options.command else {
            panic!("expected Export");
        };
        assert_eq!(article, PathBuf::from("Articles/published/demo.md"));
        assert_eq!(target, Some("zola".to_owned()));
        Ok(())
    }

    #[test]
    fn parses_new_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["new".to_owned(), "我的文章标题".to_owned()])?;
        let Command::New { title } = options.command else {
            panic!("expected New");
        };
        assert_eq!(title, "我的文章标题");
        Ok(())
    }

    #[test]
    fn parses_write_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["write".to_owned(), "写一篇关于读书的文章".to_owned()])?;
        let Command::Write { idea } = options.command else {
            panic!("expected Write");
        };
        assert_eq!(idea, "写一篇关于读书的文章");
        Ok(())
    }

    #[test]
    fn parses_draft_from_inbox_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "draft-from-inbox".to_owned(),
            "Inbox/Feishu/demo.md".to_owned(),
        ])?;
        let Command::DraftFromInbox {
            input,
            preview,
            auto_push,
        } = options.command
        else {
            panic!("expected DraftFromInbox");
        };
        assert_eq!(input, PathBuf::from("Inbox/Feishu/demo.md"));
        assert_eq!(preview, PreviewOptions::default());
        assert!(!auto_push);
        Ok(())
    }

    #[test]
    fn parses_draft_from_inbox_with_preview_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "draft-from-inbox".to_owned(),
            "Inbox/Feishu/demo.md".to_owned(),
            "--preview".to_owned(),
        ])?;
        let Command::DraftFromInbox {
            input,
            preview,
            auto_push,
        } = options.command
        else {
            panic!("expected DraftFromInbox");
        };
        assert_eq!(input, PathBuf::from("Inbox/Feishu/demo.md"));
        assert_eq!(
            preview,
            PreviewOptions {
                enabled: true,
                open: true,
            }
        );
        assert!(!auto_push);
        Ok(())
    }

    #[test]
    fn parses_draft_from_inbox_with_preview_no_open_command()
    -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "draft-from-inbox".to_owned(),
            "Inbox/Feishu/demo.md".to_owned(),
            "--preview".to_owned(),
            "--no-open".to_owned(),
        ])?;
        let Command::DraftFromInbox {
            input,
            preview,
            auto_push,
        } = options.command
        else {
            panic!("expected DraftFromInbox");
        };
        assert_eq!(input, PathBuf::from("Inbox/Feishu/demo.md"));
        assert_eq!(
            preview,
            PreviewOptions {
                enabled: true,
                open: false,
            }
        );
        assert!(!auto_push);
        Ok(())
    }

    #[test]
    fn draft_from_inbox_no_open_requires_preview() {
        let err = Options::parse([
            "draft-from-inbox".to_owned(),
            "Inbox/Feishu/demo.md".to_owned(),
            "--no-open".to_owned(),
        ]);

        assert!(err.is_err());
    }

    #[test]
    fn parses_draft_from_inbox_with_push_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "draft-from-inbox".to_owned(),
            "Inbox/Feishu/demo.md".to_owned(),
            "--push".to_owned(),
        ])?;
        let Command::DraftFromInbox {
            input,
            preview,
            auto_push,
        } = options.command
        else {
            panic!("expected DraftFromInbox");
        };
        assert_eq!(input, PathBuf::from("Inbox/Feishu/demo.md"));
        assert_eq!(preview, PreviewOptions::default());
        assert!(auto_push);
        Ok(())
    }

    #[test]
    fn draft_from_inbox_push_conflicts_with_preview() {
        let err = Options::parse([
            "draft-from-inbox".to_owned(),
            "Inbox/Feishu/demo.md".to_owned(),
            "--preview".to_owned(),
            "--push".to_owned(),
        ]);

        assert!(err.is_err());
    }

    #[test]
    fn parses_expand_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["expand".to_owned(), "drafts/notes.md".to_owned()])?;
        let Command::Expand { article } = options.command else {
            panic!("expected Expand");
        };
        assert_eq!(article, PathBuf::from("drafts/notes.md"));
        Ok(())
    }

    #[test]
    fn parses_polish_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["polish".to_owned(), "drafts/draft.md".to_owned()])?;
        let Command::Polish { article } = options.command else {
            panic!("expected Polish");
        };
        assert_eq!(article, PathBuf::from("drafts/draft.md"));
        Ok(())
    }

    #[test]
    fn parses_ship_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["ship".to_owned(), "drafts/post.md".to_owned()])?;
        let Command::Ship { article, .. } = options.command else {
            panic!("expected Ship");
        };
        assert_eq!(article, PathBuf::from("drafts/post.md"));
        Ok(())
    }

    #[test]
    fn parses_ship_with_ai_flag() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "ship".to_owned(),
            "drafts/post.md".to_owned(),
            "--ai".to_owned(),
        ])?;
        let Command::ShipAi { article, .. } = options.command else {
            panic!("expected ShipAi");
        };
        assert_eq!(article, PathBuf::from("drafts/post.md"));
        Ok(())
    }

    #[test]
    fn parses_ship_with_style() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "ship".to_owned(),
            "drafts/post.md".to_owned(),
            "--style".to_owned(),
            "dark".to_owned(),
        ])?;
        let Command::Ship { style, .. } = options.command else {
            panic!("expected Ship");
        };
        assert_eq!(style, Some("dark".to_owned()));
        Ok(())
    }

    #[test]
    fn parses_ship_style_and_ai() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "ship".to_owned(),
            "drafts/post.md".to_owned(),
            "--style".to_owned(),
            "warm".to_owned(),
            "--ai".to_owned(),
        ])?;
        let Command::ShipAi { style, .. } = options.command else {
            panic!("expected ShipAi with style");
        };
        assert_eq!(style, Some("warm".to_owned()));
        Ok(())
    }

    #[test]
    fn new_requires_title() {
        let err = Options::parse(["new".to_owned()]);
        assert!(err.is_err());
    }

    #[test]
    fn write_requires_idea() {
        let err = Options::parse(["write".to_owned()]);
        assert!(err.is_err());
    }

    #[test]
    fn parses_login_with_temporary_profile() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["login".to_owned(), "--temporary-profile".to_owned()])?;

        assert_eq!(
            options.command,
            Command::Login {
                temporary_profile: true,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_wechat_health_with_browser_flags() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "wechat-health".to_owned(),
            "--headed".to_owned(),
            "--temporary-profile".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::WechatHealth {
                headed: true,
                temporary_profile: true,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_configure_with_temporary_profile() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "configure".to_owned(),
            "--temporary-profile".to_owned(),
            "--headed".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::Configure {
                steps: vec![],
                headed: true,
                temporary_profile: true,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_test_yulan_with_temporary_profile() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["test-yulan".to_owned(), "--temporary-profile".to_owned()])?;

        assert_eq!(
            options.command,
            Command::TestYulan {
                headed: false,
                temporary_profile: true,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_step_test_with_temporary_profile() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "step-test".to_owned(),
            "--temporary-profile".to_owned(),
            "--headed".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::StepTest {
                headed: true,
                temporary_profile: true,
            }
        );
        Ok(())
    }
}
