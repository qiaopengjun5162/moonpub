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
    Publish {
        article: PathBuf,
        target: String,
        auto_render: bool,
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
    Configure {
        steps: Vec<String>,
        headed: bool,
    },
    StepTest {
        headed: bool,
    },
    TestZanshang {
        headed: bool,
    },
    TestChuangzuo {
        headed: bool,
    },
    TestYulan {
        headed: bool,
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
    Radar(RadarCommand),
    Capabilities,
    Version,
    Help,
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
            "publish" => {
                let value = rest
                    .get(1)
                    .ok_or(AppError::MissingValue("publish <article.md>"))?;
                let mut target = None;
                let mut auto_render = false;
                let mut extra = rest[2..].iter();
                while let Some(flag) = extra.next() {
                    match flag.as_str() {
                        "--target" => {
                            target = Some(flag_value(&mut extra, "--target")?);
                        }
                        "--render" => auto_render = true,
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
            "login" => Command::Login,
            "configure" => {
                let mut headed = false;
                let mut steps = Vec::new();
                for arg in &rest[1..] {
                    match arg.as_str() {
                        "--headed" => headed = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        s => steps.push(s.to_owned()),
                    }
                }
                Command::Configure { steps, headed }
            }
            "step-test" => {
                let mut headed = false;
                for flag in &rest[1..] {
                    match flag.as_str() {
                        "--headed" => headed = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::StepTest { headed }
            }
            "test-zanshang" => {
                let mut headed = false;
                for flag in &rest[1..] {
                    match flag.as_str() {
                        "--headed" => headed = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::TestZanshang { headed }
            }
            "test-chuangzuo" => {
                let mut headed = false;
                for flag in &rest[1..] {
                    match flag.as_str() {
                        "--headed" => headed = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::TestChuangzuo { headed }
            }
            "test-yulan" => {
                let mut headed = false;
                for flag in &rest[1..] {
                    match flag.as_str() {
                        "--headed" => headed = true,
                        v if v.starts_with('-') => {
                            return Err(AppError::UnknownOption(v.to_owned()));
                        }
                        v => return Err(AppError::UnknownCommand(v.to_owned())),
                    }
                }
                Command::TestYulan { headed }
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

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::cli::{Command, Options};

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

    #[test]
    fn parses_publish_target_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "publish".to_owned(),
            "Articles/ready/demo.md".to_owned(),
            "--target".to_owned(),
            "wechat-draft".to_owned(),
            "--render".to_owned(),
        ])?;
        let Command::Publish {
            article,
            target,
            auto_render,
        } = options.command
        else {
            panic!("expected Publish");
        };
        assert_eq!(article, PathBuf::from("Articles/ready/demo.md"));
        assert_eq!(target, "wechat-draft");
        assert!(auto_render);
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
}
