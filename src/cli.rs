use std::path::{Path, PathBuf};

use crate::config::Config;
use crate::error::AppError;
use crate::radar::{RadarCommand, parse_radar_command};

const DEFAULT_CONFIG: &str = "moonpub.toml";

/// Consume the next argument as a value for a named flag (e.g., "--style dark").
///
/// Accepts any iterator over `&String`, so both the argument slice and the
/// `Vec<&String>` returned by `path_and_flags_with_values` can drive it.
/// A value that itself looks like a long flag means the caller forgot the
/// value; reporting `MissingValue` beats silently swallowing the next switch.
pub fn flag_value<'a, I>(extra: &mut I, name: &'static str) -> Result<String, AppError>
where
    I: Iterator<Item = &'a String>,
{
    let v = extra.next().ok_or(AppError::MissingValue(name))?;
    if v.starts_with("--") {
        return Err(AppError::MissingValue(name));
    }
    Ok(v.clone())
}

/// Returns the positional argument at `index`, reporting `usage` when absent.
///
/// Most subcommands take exactly one path argument; without this every arm
/// repeated `rest.get(1).ok_or(...)` plus its own usage string.
fn positional(rest: &[String], index: usize, usage: &'static str) -> Result<String, AppError> {
    rest.get(index)
        .cloned()
        .ok_or(AppError::MissingValue(usage))
}

/// Returns the path argument at `index`.
fn path_arg(rest: &[String], index: usize, usage: &'static str) -> Result<PathBuf, AppError> {
    Ok(PathBuf::from(positional(rest, index, usage)?))
}

/// Rejects any token past the fixed number of positionals a command accepts.
///
/// `status`, `list-drafts`, `version`, `workspace`, `layout-recipes`, `fetch`
/// and `delete-draft` take a fixed arity and no options beyond the global
/// `--json`, but they used to ignore everything after their positional. That
/// is merely untidy for the read-only ones, and dangerous for `delete-draft`:
/// it has no dry-run mode, so `delete-draft <id> --dry-run` deleted the draft
/// the caller thought it was previewing.
///
/// `--json` stays allowed because the trailing spelling is documented for
/// machine-readable commands; `supports_subcommand_json` only strips it for
/// some of them, so the rest must tolerate it here.
fn reject_extra_args(rest: &[String], consumed: usize) -> Result<(), AppError> {
    let extra = rest.get(consumed..).unwrap_or(&[]);
    reject_unexpected(extra, &["--json"])
}

/// Joins free-form text arguments such as `new <title>` / `write <idea>`.
///
/// Titles with spaces are the normal case for Chinese and English prose, and
/// `draft.rs` already stores them verbatim (`我的 第一篇 文章`). Taking only
/// `rest[1]` silently truncated the title to its first word while still
/// reporting success. Unknown flags must not become part of the title; a
/// trailing `--json` stays tolerated because `new` / `write` are not in
/// `supports_subcommand_json`, so that spelling still reaches this arm.
fn join_free_text(rest: &[String], usage: &'static str) -> Result<String, AppError> {
    let mut words = Vec::new();
    for arg in &rest[1..] {
        if arg == "--json" {
            continue;
        }
        if arg.starts_with('-') {
            return Err(AppError::UnknownOption(arg.clone()));
        }
        words.push(arg.as_str());
    }
    let text = words.join(" ");
    if text.trim().is_empty() {
        return Err(AppError::MissingValue(usage));
    }
    Ok(text)
}

/// Rejects any argument outside `allowed`.
///
/// Keeps the CLI's two error shapes: a leading dash is an unknown option,
/// anything else an unknown command.
fn reject_unexpected(args: &[String], allowed: &[&str]) -> Result<(), AppError> {
    for arg in args {
        if allowed.contains(&arg.as_str()) {
            continue;
        }
        return Err(if arg.starts_with('-') {
            AppError::UnknownOption(arg.clone())
        } else {
            AppError::UnknownCommand(arg.clone())
        });
    }
    Ok(())
}

/// Zero-flag positional commands accept exactly `<path>`; everything after it
/// used to be silently dropped, so a typo like `--stric` or a second path went
/// unnoticed. A trailing `--json` stays allowed: `supports_subcommand_json`
/// only strips it for machine-readable commands, and these arms must not turn
/// the long-standing `cmd <path> --json` spelling into an error.
fn reject_stray_args_after_path(rest: &[String]) -> Result<(), AppError> {
    reject_unexpected(&rest[2..], &["--json"])
}

/// Parses the `--headed` / `--temporary-profile` pair shared by the
/// browser-driven subcommands.
fn browser_flags(args: &[String]) -> Result<(bool, bool), AppError> {
    let mut headed = false;
    let mut temporary_profile = false;
    for arg in args {
        match arg.as_str() {
            "--headed" => headed = true,
            "--temporary-profile" => temporary_profile = true,
            v if v.starts_with('-') => return Err(AppError::UnknownOption(v.to_owned())),
            v => return Err(AppError::UnknownCommand(v.to_owned())),
        }
    }
    Ok((headed, temporary_profile))
}

/// Splits `rest[1..]` into the single positional path and the remaining flag
/// tokens for commands whose flags are all boolean (e.g. `preview`, `push`,
/// `preflight`, `draft-from-inbox`).
///
/// Thin wrapper over [`path_and_flags_with_values`] so every single-path
/// command shares one scanner.
fn path_and_flags<'a>(
    rest: &'a [String],
    usage: &'static str,
) -> Result<(PathBuf, Vec<&'a String>), AppError> {
    path_and_flags_with_values(rest, usage, &[])
}

/// Splits `rest[1..]` into the single positional path and the remaining flag
/// tokens, tolerating flags before or after the positional.
///
/// `value_flags` names the flags that consume the next token as their value
/// (`--style <name>`, `--target <target>`, …). Pairing values with their flags
/// removes the ambiguity that used to keep these commands on the old "path must
/// be `rest[1]`" contract: the first leftover non-flag token is the path.
///
/// Flag/value pairs and stray tokens are returned **in their original order**,
/// leaving the caller's `match` to decide between `UnknownOption` and
/// `UnknownCommand`. That preserves the long-standing "unknown flag wins over
/// extra positional" error priority.
fn path_and_flags_with_values<'a>(
    rest: &'a [String],
    usage: &'static str,
    value_flags: &[&'static str],
) -> Result<(PathBuf, Vec<&'a String>), AppError> {
    let mut positional: Option<PathBuf> = None;
    let mut tokens: Vec<&'a String> = Vec::new();
    let mut index = 1;
    while index < rest.len() {
        let arg = &rest[index];
        if let Some(name) = value_flags
            .iter()
            .copied()
            .find(|flag| *flag == arg.as_str())
        {
            tokens.push(arg);
            let value = rest.get(index + 1).ok_or(AppError::MissingValue(name))?;
            if value.starts_with("--") {
                return Err(AppError::MissingValue(name));
            }
            tokens.push(value);
            index += 2;
        } else if arg.starts_with('-') {
            tokens.push(arg);
            index += 1;
        } else if positional.is_none() {
            positional = Some(PathBuf::from(arg));
            index += 1;
        } else {
            tokens.push(arg);
            index += 1;
        }
    }
    let positional = positional.ok_or(AppError::MissingValue(usage))?;
    Ok((positional, tokens))
}

/// Resolves the articles root together with the config file that applies.
///
/// An explicit `--config` always wins. Otherwise `moonpub.toml` is looked up in
/// the articles root, then upwards from the path-like arguments, because users
/// run `moonpub render Articles/drafts/x.md` from outside the articles root
/// and the old silent fallback to default config produced the wrong author
/// and theme.
///
/// Every non-flag token is a candidate, not just the first one: once flags were
/// allowed before the path (#168/#171), that first token is often a flag value
/// (`render --author Alice a.md`) or a subcommand name (`intake photos --draft
/// a.jpg`). Searching upward from either silently missed the real config or,
/// worse, adopted an unrelated project's `moonpub.toml`. Candidates that
/// resolve to an existing path are tried first.
fn resolve_config(
    mut articles_dir: PathBuf,
    mut config: Option<PathBuf>,
    rest: &[String],
) -> Result<(PathBuf, Option<PathBuf>), AppError> {
    if let Some(cfg_path) = &config {
        let cfg = Config::load(cfg_path)?;
        if let Some(root) = cfg.articles_root {
            articles_dir = root;
        }
        return Ok((articles_dir, config));
    }

    let auto = articles_dir.join(DEFAULT_CONFIG);
    if auto.exists() {
        let cfg = Config::load(&auto)?;
        config = Some(auto);
        if let Some(root) = cfg.articles_root {
            articles_dir = root;
        }
        return Ok((articles_dir, config));
    }

    for anchor in config_anchor_candidates(rest, &articles_dir) {
        let mut cur = anchor.parent().map(|p| p.to_path_buf());
        while let Some(dir) = cur {
            let candidate = dir.join(DEFAULT_CONFIG);
            if candidate.exists() {
                let cfg = Config::load(&candidate)?;
                config = Some(candidate);
                if let Some(root) = cfg.articles_root {
                    articles_dir = root;
                } else {
                    articles_dir = dir;
                }
                return Ok((articles_dir, config));
            }
            cur = dir.parent().map(|p| p.to_path_buf());
        }
    }
    Ok((articles_dir, config))
}

/// Builds the ordered list of paths whose ancestors may hold `moonpub.toml`.
///
/// A token is not necessarily a path: it can be a value for `--author` /
/// `--style`, a subcommand name (`intake photos`), or free text (`new 我的 文章`).
/// Rather than maintaining a global list of every value-taking flag, prefer
/// tokens that actually resolve to a file and keep the original argv order
/// otherwise. `sort_by_key` is stable, so this never reshuffles equally
/// plausible candidates.
fn config_anchor_candidates(rest: &[String], articles_dir: &Path) -> Vec<PathBuf> {
    let mut paths: Vec<PathBuf> = rest
        .iter()
        .skip(1)
        .filter(|token| !token.starts_with('-'))
        .map(|token| {
            let path = PathBuf::from(token);
            if path.is_absolute() {
                path
            } else {
                articles_dir.join(path)
            }
        })
        .collect();
    paths.sort_by_key(|path| !path.exists());
    paths
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
        yes: bool,
    },
    Doctor,
    Workspace,
    Status,
    Check {
        article: PathBuf,
    },
    Preflight {
        article: PathBuf,
        /// Fail the process (non-zero exit) when any preflight check fails.
        /// Without it the gate is advisory: the report always exits 0 so
        /// plugins can read `passed` without treating a failed check as a crash.
        strict: bool,
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
    Cards {
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
        evidence_dir: Option<PathBuf>,
        title: Option<String>,
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
        title: Option<String>,
        to_wxname: Option<String>,
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
        analyze_images: bool,
        draft: bool,
        preview: PreviewOptions,
        auto_push: bool,
    },
    WorkflowRegistry,
    EvidenceStatus {
        strict: bool,
    },
    ReleaseCheck {
        strict: bool,
    },
    WechatChecklist,
    LayoutRecipes,
    LayoutAudit {
        html: PathBuf,
    },
    Radar(RadarCommand),
    Capabilities,
    Version,
    /// `moonpub help` lists commands; `moonpub help <command>` explains one;
    /// `moonpub help errors` lists the `error/v1` codes.
    Help {
        topic: Option<String>,
    },
}

impl Command {
    pub(crate) fn has_structured_json_output(&self) -> bool {
        matches!(
            self,
            Self::Capabilities
                | Self::Help { .. }
                | Self::Doctor
                | Self::Workspace
                | Self::WorkflowRegistry
                | Self::EvidenceStatus { .. }
                | Self::ReleaseCheck { .. }
                | Self::WechatChecklist
                | Self::LayoutRecipes
                | Self::LayoutAudit { .. }
                | Self::WechatHealth { .. }
                | Self::Status
                | Self::Check { .. }
                | Self::Preflight { .. }
                | Self::Preview { .. }
                | Self::Push { .. }
                | Self::DraftFromInbox { .. }
                | Self::IntakeFeishu { draft: true, .. }
                | Self::IntakePhotos { draft: true, .. }
                | Self::Cards { .. }
        )
    }
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
                        command: Command::Help { topic: None },
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

        let (articles_dir, config) = resolve_config(articles_dir, config, &rest)?;

        let Some(command_name) = rest.first().cloned() else {
            return Err(AppError::MissingCommand);
        };

        // subcommand --help → help for that command, not the whole list.
        // `--help`/`-h` may appear after other flags (`init --yes --help`), so
        // scan every trailing arg instead of only the one at index 1; otherwise
        // `init --yes --help` fell through to the subcommand parser and errored
        // with "unknown option: --help".
        if rest.iter().skip(1).any(|s| s == "--help" || s == "-h") {
            return Ok(Self {
                articles: articles_dir,
                command: Command::Help {
                    topic: rest.first().cloned(),
                },
                json,
                config,
            });
        }

        if supports_subcommand_json(&command_name) {
            let mut filtered = Vec::with_capacity(rest.len());
            filtered.push(command_name.clone());
            for arg in &rest[1..] {
                if arg == "--json" {
                    json = true;
                } else {
                    filtered.push(arg.clone());
                }
            }
            rest = filtered;
        }

        let command = parse_command(&command_name, &rest, &articles_dir)?;

        Ok(Self {
            articles: articles_dir,
            command,
            json,
            config,
        })
    }
}

/// Parses the subcommand name and its trailing flags.
///
/// `--json` has already been stripped by the caller for subcommands that
/// support structured output, so no arm has to handle it.
fn parse_command(
    command_name: &str,
    rest: &[String],
    articles_dir: &Path,
) -> Result<Command, AppError> {
    let command = match command_name {
        "init" => {
            let mut yes = false;
            let mut path_arg: Option<String> = None;
            for item in &rest[1..] {
                match item.as_str() {
                    "--yes" => yes = true,
                    v if v.starts_with('-') => {
                        return Err(AppError::UnknownOption(v.to_owned()));
                    }
                    v => {
                        if path_arg.is_some() {
                            // `init` takes at most one positional path; a second
                            // one used to silently overwrite the first while the
                            // command still reported success, so `init a.toml b.toml`
                            // wrote `b.toml` and dropped `a.toml`.
                            return Err(AppError::UnknownCommand(v.to_owned()));
                        }
                        path_arg = Some(v.to_owned());
                    }
                }
            }
            let path = path_arg
                .map(PathBuf::from)
                .unwrap_or_else(|| articles_dir.join(DEFAULT_CONFIG));
            Command::Init { path, yes }
        }
        "new" => Command::New {
            title: join_free_text(rest, "new <title>")?,
        },
        "write" => Command::Write {
            idea: join_free_text(rest, "write <idea>")?,
        },
        "draft-from-inbox" => {
            let (article, flags) = path_and_flags(rest, "draft-from-inbox <inbox.md>")?;
            let mut enabled = false;
            let mut no_open = false;
            let mut auto_push = false;
            for flag in &flags {
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
                input: article,
                preview,
                auto_push,
            }
        }
        "polish" => {
            let article = path_arg(rest, 1, "polish <article.md>")?;
            reject_stray_args_after_path(rest)?;
            Command::Polish { article }
        }
        "expand" => {
            let article = path_arg(rest, 1, "expand <article.md>")?;
            reject_stray_args_after_path(rest)?;
            Command::Expand { article }
        }
        "workspace" => {
            reject_extra_args(rest, 1)?;
            Command::Workspace
        }
        "doctor" => {
            reject_unexpected(&rest[1..], &["--json"])?;
            Command::Doctor
        }
        "workflow-registry" => {
            reject_unexpected(&rest[1..], &["--json"])?;
            Command::WorkflowRegistry
        }
        "evidence-status" => {
            reject_unexpected(&rest[1..], &["--json", "--strict"])?;
            Command::EvidenceStatus {
                strict: rest[1..].iter().any(|arg| arg == "--strict"),
            }
        }
        "release-check" => {
            reject_unexpected(&rest[1..], &["--json", "--strict"])?;
            Command::ReleaseCheck {
                strict: rest[1..].iter().any(|arg| arg == "--strict"),
            }
        }
        "wechat-checklist" => {
            reject_unexpected(&rest[1..], &["--json"])?;
            Command::WechatChecklist
        }
        "layout-recipes" => {
            reject_extra_args(rest, 1)?;
            Command::LayoutRecipes
        }
        "layout-audit" => {
            let html = path_arg(rest, 1, "layout-audit <html>")?;
            reject_stray_args_after_path(rest)?;
            Command::LayoutAudit { html }
        }
        "status" => {
            reject_extra_args(rest, 1)?;
            Command::Status
        }
        "check" => {
            let article = path_arg(rest, 1, "check <article.md>")?;
            reject_stray_args_after_path(rest)?;
            Command::Check { article }
        }
        "preflight" => {
            let (article, flags) = path_and_flags(rest, "preflight <article.md>")?;
            let mut strict = false;
            for flag in &flags {
                match flag.as_str() {
                    "--strict" => strict = true,
                    v if v.starts_with('-') => {
                        return Err(AppError::UnknownOption(v.to_owned()));
                    }
                    v => return Err(AppError::UnknownCommand(v.to_owned())),
                }
            }
            Command::Preflight { article, strict }
        }
        "push" => {
            let (article, flags) = path_and_flags(rest, "push <article.md>")?;
            let mut auto_render = false;
            let mut temporary_profile = false;
            for flag in &flags {
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
                article,
                auto_render,
                temporary_profile,
            }
        }
        "publish" => {
            let (article, tokens) =
                path_and_flags_with_values(rest, "publish <article.md>", &["--target"])?;
            let mut target = None;
            let mut auto_render = false;
            let mut temporary_profile = false;
            let mut extra = tokens.into_iter();
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
                article,
                target: target.ok_or(AppError::MissingValue("--target"))?,
                auto_render,
                temporary_profile,
            }
        }
        "update-draft" => {
            let (article, tokens) =
                path_and_flags_with_values(rest, "update-draft <article.md>", &["--media-id"])?;
            let mut media_id = None;
            let mut extra = tokens.into_iter();
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
            Command::UpdateDraft { article, media_id }
        }
        "ship" => {
            let (article, tokens) =
                path_and_flags_with_values(rest, "ship <article.md>", &["--style"])?;
            let mut style = None;
            let mut with_ai = false;
            let mut extra = tokens.into_iter();
            while let Some(flag) = extra.next() {
                match flag.as_str() {
                    "--style" => {
                        style = Some(flag_value(&mut extra, "--style")?);
                    }
                    "--ai" => with_ai = true,
                    v if v.starts_with('-') => {
                        return Err(AppError::UnknownOption(v.to_owned()));
                    }
                    v => return Err(AppError::UnknownCommand(v.to_owned())),
                }
            }
            if with_ai {
                Command::ShipAi { article, style }
            } else {
                Command::Ship { article, style }
            }
        }
        "mark-ready" => {
            let article = path_arg(rest, 1, "mark-ready <article.md>")?;
            reject_stray_args_after_path(rest)?;
            Command::MarkReady { article }
        }
        "cover" => {
            let (article, tokens) =
                path_and_flags_with_values(rest, "cover <article.md>", &["--style"])?;
            let mut style = None;
            let mut screenshot = false;
            let mut extra = tokens.into_iter();
            while let Some(flag) = extra.next() {
                match flag.as_str() {
                    "--style" => {
                        style = Some(flag_value(&mut extra, "--style")?);
                    }
                    "--screenshot" => screenshot = true,
                    v if v.starts_with('-') => {
                        return Err(AppError::UnknownOption(v.to_owned()));
                    }
                    v => return Err(AppError::UnknownCommand(v.to_owned())),
                }
            }
            Command::Cover {
                article,
                style,
                screenshot,
            }
        }
        "humanize" => {
            let article = path_arg(rest, 1, "humanize <article.md>")?;
            reject_stray_args_after_path(rest)?;
            Command::Humanize { article }
        }
        "cards" => {
            let article = path_arg(rest, 1, "cards <article.md>")?;
            reject_stray_args_after_path(rest)?;
            Command::Cards { article }
        }
        "login" => {
            reject_unexpected(&rest[1..], &["--temporary-profile"])?;
            Command::Login {
                temporary_profile: rest[1..].iter().any(|arg| arg == "--temporary-profile"),
            }
        }
        "wechat-health" => {
            let (headed, temporary_profile) = browser_flags(&rest[1..])?;
            Command::WechatHealth {
                headed,
                temporary_profile,
            }
        }
        "configure" => {
            let mut headed = false;
            let mut temporary_profile = false;
            let mut evidence_dir = None;
            let mut title = None;
            let mut steps = Vec::new();
            let mut extra = rest[1..].iter();
            while let Some(arg) = extra.next() {
                match arg.as_str() {
                    "--headed" => headed = true,
                    "--temporary-profile" => temporary_profile = true,
                    "--evidence-dir" => {
                        evidence_dir =
                            Some(PathBuf::from(flag_value(&mut extra, "--evidence-dir")?));
                    }
                    "--title" => {
                        title = Some(flag_value(&mut extra, "--title")?);
                    }
                    v if v.starts_with('-') => {
                        return Err(AppError::UnknownOption(v.to_owned()));
                    }
                    s => {
                        if !crate::publish::is_valid_configure_step(s) {
                            return Err(AppError::UnknownCommand(format!("configure {s}")));
                        }
                        steps.push(s.to_owned());
                    }
                }
            }
            Command::Configure {
                steps,
                headed,
                temporary_profile,
                evidence_dir,
                title,
            }
        }
        "step-test" => {
            let (headed, temporary_profile) = browser_flags(&rest[1..])?;
            Command::StepTest {
                headed,
                temporary_profile,
            }
        }
        "test-zanshang" => {
            let (headed, temporary_profile) = browser_flags(&rest[1..])?;
            Command::TestZanshang {
                headed,
                temporary_profile,
            }
        }
        "test-chuangzuo" => {
            let (headed, temporary_profile) = browser_flags(&rest[1..])?;
            Command::TestChuangzuo {
                headed,
                temporary_profile,
            }
        }
        "test-yulan" => {
            let mut headed = false;
            let mut temporary_profile = false;
            let mut title = None;
            let mut to_wxname = None;
            // `flag_value` owns the "next token looks like a flag" guard that
            // PR #171 added; a hand-rolled index loop bypassed it and silently
            // turned `--title --headed` into a title of "--headed".
            let mut extra = rest[1..].iter();
            while let Some(flag) = extra.next() {
                match flag.as_str() {
                    "--headed" => headed = true,
                    "--temporary-profile" => temporary_profile = true,
                    "--title" => {
                        title = Some(flag_value(&mut extra, "test-yulan --title <title>")?);
                    }
                    "--to" => {
                        to_wxname = Some(flag_value(&mut extra, "test-yulan --to <wxid>")?);
                    }
                    v if v.starts_with('-') => {
                        return Err(AppError::UnknownOption(v.to_owned()));
                    }
                    v => return Err(AppError::UnknownCommand(v.to_owned())),
                }
            }
            Command::TestYulan {
                headed,
                temporary_profile,
                title,
                to_wxname,
            }
        }
        "list-drafts" => {
            reject_extra_args(rest, 1)?;
            Command::ListDrafts
        }
        "delete-draft" => {
            let media_id = rest
                .get(1)
                .ok_or(AppError::MissingValue("delete-draft <media_id>"))?
                .clone();
            // Destructive command with no dry-run mode: a stray `--dry-run`
            // must fail loudly instead of deleting the draft.
            reject_extra_args(rest, 2)?;
            Command::DeleteDraft { media_id }
        }
        "fetch" => {
            let url = rest.get(1).ok_or(AppError::MissingValue("fetch <url>"))?;
            reject_extra_args(rest, 2)?;
            Command::Fetch {
                url: url.to_owned(),
            }
        }
        "mark-published" => {
            let article = path_arg(rest, 1, "mark-published <article.md>")?;
            reject_stray_args_after_path(rest)?;
            Command::MarkPublished { article }
        }
        "export" => {
            let (article, tokens) =
                path_and_flags_with_values(rest, "export <article.md>", &["--target"])?;
            let mut target = None;
            let mut extra = tokens.into_iter();
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
            Command::Export { article, target }
        }
        "preview" => {
            let (article, flags) = path_and_flags(rest, "preview <article.md>")?;
            let mut open = true;
            for flag in &flags {
                match flag.as_str() {
                    "--no-open" => open = false,
                    v if v.starts_with('-') => {
                        return Err(AppError::UnknownOption(v.to_owned()));
                    }
                    v => return Err(AppError::UnknownCommand(v.to_owned())),
                }
            }
            Command::Preview { article, open }
        }
        "render" => {
            let (article, tokens) =
                path_and_flags_with_values(rest, "render <article.md>", &["--author", "--thumb"])?;
            let mut author = None;
            let mut thumb_media_id = None;
            let mut humanize = false;
            let mut extra = tokens.into_iter();
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
                article,
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
                "feishu" => parse_intake_feishu(&rest[2..])?,
                "photos" => parse_intake_photos(&rest[2..])?,
                other => return Err(AppError::UnknownCommand(format!("intake {other}"))),
            }
        }
        "capabilities" => {
            reject_unexpected(&rest[1..], &["--json"])?;
            Command::Capabilities
        }
        "version" => {
            reject_extra_args(rest, 1)?;
            Command::Version
        }
        "help" => {
            reject_extra_args(rest, 2)?;
            Command::Help {
                topic: rest.get(1).cloned(),
            }
        }
        value => return Err(AppError::UnknownCommand(value.to_owned())),
    };
    Ok(command)
}
fn supports_subcommand_json(command: &str) -> bool {
    matches!(
        command,
        // `help` has a structured `help/v1` payload and `version` has the
        // fallback `{"output":...}`; both are documented with a trailing
        // `--json`. Leaving either out makes the flag silently do nothing
        // (or, for `help --json`, become a bogus command name).
        "help"
            | "version"
            | "doctor"
            | "workspace"
            | "workflow-registry"
            | "evidence-status"
            | "release-check"
            | "wechat-checklist"
            | "layout-recipes"
            | "layout-audit"
            | "wechat-health"
            | "capabilities"
            | "status"
            | "check"
            | "preflight"
            | "preview"
            | "push"
            | "draft-from-inbox"
            | "intake"
            | "cards"
    )
}

/// `intake` tokens split without assuming where a switch may appear.
///
/// `feishu` takes exactly one source selector (a file, `--latest`,
/// `--query <kw>` or `--minute-token <t>`) while `photos` takes any number
/// of paths. Both arms used to read the positional from a fixed index and
/// scan everything after it as flags, so a leading flag was reported as a
/// missing file and `--query --draft` silently made `--draft` the keyword.
/// One scanner makes position irrelevant for both shapes.
#[derive(Debug, Default)]
struct IntakeTokens {
    positionals: Vec<String>,
    flags: Vec<String>,
    values: Vec<(&'static str, String)>,
}

/// Separates positionals, boolean switches and value-taking switches.
///
/// Values carry the same guard as [`flag_value`]: a following switch means
/// the value is missing, never that the switch *is* the value. Unknown
/// switches stay in `flags` so the existing validator keeps reporting
/// `UnknownOption` instead of silently adopting them.
fn split_intake_tokens(
    tokens: &[String],
    value_flags: &[(&'static str, &'static str)],
) -> Result<IntakeTokens, AppError> {
    let mut parsed = IntakeTokens::default();
    let mut index = 0;
    while index < tokens.len() {
        let token = &tokens[index];
        let valued = value_flags
            .iter()
            .find(|(flag, _)| *flag == token.as_str())
            .copied();
        if let Some((flag, usage)) = valued {
            let value = tokens
                .get(index + 1)
                .filter(|next| !next.starts_with('-'))
                .ok_or(AppError::MissingValue(usage))?;
            parsed.values.push((flag, value.clone()));
            index += 2;
        } else if token.starts_with('-') {
            parsed.flags.push(token.clone());
            index += 1;
        } else {
            parsed.positionals.push(token.clone());
            index += 1;
        }
    }
    Ok(parsed)
}

const INTAKE_FEISHU_USAGE: &str =
    "intake feishu <file> | --latest | --query <kw> | --minute-token <t>";
const INTAKE_PHOTOS_USAGE: &str = "intake photos <file-or-dir> [more files or dirs]";

fn parse_intake_feishu(tokens: &[String]) -> Result<Command, AppError> {
    let parsed = split_intake_tokens(
        tokens,
        &[
            ("--minute-token", "intake feishu --minute-token <token>"),
            ("--query", "intake feishu --query <keyword>"),
        ],
    )?;
    let mut flags = Vec::new();
    let mut latest = false;
    for flag in &parsed.flags {
        if flag == "--latest" {
            latest = true;
        } else {
            flags.push(flag.clone());
        }
    }

    // A source selector is exclusive: mixing `--latest`, a valued selector
    // and a file used to pick whichever appeared first and then report the
    // rest as an unrelated unknown command (or, worse, silently drop it).
    // Report the documented usage instead so the caller can fix the shape.
    let ambiguous = || AppError::MissingValue(INTAKE_FEISHU_USAGE);
    let source = if parsed.values.len() > 1 || parsed.positionals.len() > 1 {
        return Err(ambiguous());
    } else if let Some((flag, value)) = parsed.values.first() {
        if latest || !parsed.positionals.is_empty() {
            return Err(ambiguous());
        }
        match *flag {
            "--minute-token" => FeishuIntakeSource::MinuteToken(value.clone()),
            "--query" => FeishuIntakeSource::Query(value.clone()),
            _ => return Err(ambiguous()),
        }
    } else if latest {
        if !parsed.positionals.is_empty() {
            return Err(ambiguous());
        }
        FeishuIntakeSource::Latest
    } else if let Some(file) = parsed.positionals.first() {
        FeishuIntakeSource::File(PathBuf::from(file))
    } else {
        return Err(ambiguous());
    };
    let (draft, preview, auto_push) = parse_intake_feishu_flags(&flags)?;
    Ok(Command::IntakeFeishu {
        source,
        draft,
        preview,
        auto_push,
    })
}

fn parse_intake_photos(tokens: &[String]) -> Result<Command, AppError> {
    let parsed = split_intake_tokens(tokens, &[])?;
    if parsed.positionals.is_empty() {
        return Err(AppError::MissingValue(INTAKE_PHOTOS_USAGE));
    }
    let (draft, preview, auto_push, analyze_images) = parse_intake_photo_flags(&parsed.flags)?;
    Ok(Command::IntakePhotos {
        inputs: parsed.positionals.into_iter().map(PathBuf::from).collect(),
        analyze_images,
        draft,
        preview,
        auto_push,
    })
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

fn parse_intake_photo_flags(
    flags: &[String],
) -> Result<(bool, PreviewOptions, bool, bool), AppError> {
    let mut analyze_images = false;
    let mut remaining = Vec::new();
    for flag in flags {
        if flag == "--analyze-images" {
            analyze_images = true;
        } else {
            remaining.push(flag.clone());
        }
    }
    let (draft, preview, auto_push) = parse_intake_feishu_flags(&remaining)?;
    Ok((draft, preview, auto_push, analyze_images))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use crate::cli::{
        Command, FeishuIntakeSource, INTAKE_FEISHU_USAGE, INTAKE_PHOTOS_USAGE, Options,
        PreviewOptions, path_and_flags_with_values,
    };
    use crate::error::AppError;

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

    /// Every command with a documented machine-readable payload must also strip
    /// a trailing `--json`. `help` used to be missing: `help render --json`
    /// silently returned prose and `help --json` reported "unknown command".
    #[test]
    fn trailing_json_is_stripped_for_every_machine_readable_command() {
        for name in [
            "help",
            "version",
            "doctor",
            "workspace",
            "workflow-registry",
            "evidence-status",
            "release-check",
            "wechat-checklist",
            "layout-recipes",
            "layout-audit",
            "wechat-health",
            "capabilities",
            "status",
            "check",
            "preflight",
            "preview",
            "push",
            "draft-from-inbox",
            "intake",
            "cards",
        ] {
            assert!(
                crate::cli::supports_subcommand_json(name),
                "`{name}` has a documented JSON payload but ignores a trailing --json"
            );
        }
    }

    #[test]
    fn parses_help_topic_with_trailing_json() -> Result<(), Box<dyn std::error::Error>> {
        let options =
            Options::parse(["help".to_owned(), "render".to_owned(), "--json".to_owned()])?;

        assert!(options.json);
        assert_eq!(
            options.command,
            Command::Help {
                topic: Some("render".to_owned())
            }
        );
        Ok(())
    }

    #[test]
    fn parses_help_overview_with_trailing_json() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["help".to_owned(), "--json".to_owned()])?;

        assert!(options.json);
        assert_eq!(options.command, Command::Help { topic: None });
        Ok(())
    }

    #[test]
    fn parses_version_with_trailing_json() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["version".to_owned(), "--json".to_owned()])?;

        assert!(options.json);
        assert_eq!(options.command, Command::Version);
        Ok(())
    }

    #[test]
    fn parses_subcommand_json_suffix_for_workspace() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["workspace".to_owned(), "--json".to_owned()])?;

        assert_eq!(options.command, Command::Workspace);
        assert!(options.json);
        Ok(())
    }

    #[test]
    fn parses_subcommand_json_suffix_for_check() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "check".to_owned(),
            "demo.md".to_owned(),
            "--json".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::Check {
                article: PathBuf::from("demo.md")
            }
        );
        assert!(options.json);
        Ok(())
    }

    #[test]
    fn parses_subcommand_json_suffix_for_push_with_flags() -> Result<(), Box<dyn std::error::Error>>
    {
        let options = Options::parse([
            "push".to_owned(),
            "demo.md".to_owned(),
            "--render".to_owned(),
            "--json".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::Push {
                article: PathBuf::from("demo.md"),
                auto_render: true,
                temporary_profile: false,
            }
        );
        assert!(options.json);
        Ok(())
    }

    #[test]
    fn parses_subcommand_json_suffix_for_intake_feishu_draft_preview()
    -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "intake".to_owned(),
            "feishu".to_owned(),
            "--latest".to_owned(),
            "--draft".to_owned(),
            "--preview".to_owned(),
            "--json".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::IntakeFeishu {
                source: FeishuIntakeSource::Latest,
                draft: true,
                preview: PreviewOptions {
                    enabled: true,
                    open: true,
                },
                auto_push: false,
            }
        );
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
    fn parses_workflow_registry_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["workflow-registry".to_owned()])?;

        assert_eq!(options.command, Command::WorkflowRegistry);
        assert!(!options.json);
        Ok(())
    }

    #[test]
    fn parses_workflow_registry_command_json_flag() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["workflow-registry".to_owned(), "--json".to_owned()])?;

        assert_eq!(options.command, Command::WorkflowRegistry);
        assert!(options.json);
        Ok(())
    }

    #[test]
    fn parses_wechat_checklist_command_json_flag() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["wechat-checklist".to_owned(), "--json".to_owned()])?;

        assert_eq!(options.command, Command::WechatChecklist);
        assert!(options.json);
        Ok(())
    }

    #[test]
    fn parses_evidence_status_command_json_flag() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["evidence-status".to_owned(), "--json".to_owned()])?;

        assert_eq!(options.command, Command::EvidenceStatus { strict: false });
        assert!(options.json);
        Ok(())
    }

    #[test]
    fn parses_evidence_status_strict_flag() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["evidence-status".to_owned(), "--strict".to_owned()])?;

        assert_eq!(options.command, Command::EvidenceStatus { strict: true });
        assert!(!options.json);
        Ok(())
    }

    #[test]
    fn parses_evidence_status_json_strict_suffix() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "evidence-status".to_owned(),
            "--json".to_owned(),
            "--strict".to_owned(),
        ])?;

        assert_eq!(options.command, Command::EvidenceStatus { strict: true });
        assert!(options.json);
        Ok(())
    }

    #[test]
    fn parses_release_check_json_strict_suffix() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "release-check".to_owned(),
            "--json".to_owned(),
            "--strict".to_owned(),
        ])?;

        assert_eq!(options.command, Command::ReleaseCheck { strict: true });
        assert!(options.json);
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
    fn parses_help_with_a_topic() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["help".to_owned(), "render".to_owned()])?;

        assert_eq!(
            options.command,
            Command::Help {
                topic: Some("render".to_owned())
            }
        );
        Ok(())
    }

    #[test]
    fn parses_bare_help_without_a_topic() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["help".to_owned()])?;

        assert_eq!(options.command, Command::Help { topic: None });
        Ok(())
    }

    /// `moonpub push --help` used to print the whole command list.
    #[test]
    fn subcommand_help_targets_that_subcommand() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["push".to_owned(), "--help".to_owned()])?;

        assert_eq!(
            options.command,
            Command::Help {
                topic: Some("push".to_owned())
            }
        );
        Ok(())
    }

    /// `--help` after another flag must still resolve to help for the
    /// subcommand, not error with "unknown option: --help".
    #[test]
    fn subcommand_help_after_flag_is_recognized() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["init".to_owned(), "--yes".to_owned(), "--help".to_owned()])?;

        assert_eq!(
            options.command,
            Command::Help {
                topic: Some("init".to_owned())
            }
        );
        Ok(())
    }

    /// `init a.toml b.toml` used to silently write `b.toml` and drop `a.toml`
    /// while reporting success.
    #[test]
    fn init_rejects_extra_positional() {
        let err = Options::parse(["init".to_owned(), "a.toml".to_owned(), "b.toml".to_owned()])
            .unwrap_err();
        assert!(
            matches!(err, AppError::UnknownCommand(_)),
            "expected UnknownCommand, got {err:?}"
        );
    }

    /// `init` with a single positional still resolves to that path.
    #[test]
    fn init_accepts_single_positional() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["init".to_owned(), "config.toml".to_owned()])?;

        assert_eq!(
            options.command,
            Command::Init {
                path: PathBuf::from("config.toml"),
                yes: false,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_preflight_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["preflight".to_owned(), "demo.md".to_owned()])?;

        assert_eq!(
            options.command,
            Command::Preflight {
                article: PathBuf::from("demo.md"),
                strict: false,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_preflight_strict_flag() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "preflight".to_owned(),
            "demo.md".to_owned(),
            "--strict".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::Preflight {
                article: PathBuf::from("demo.md"),
                strict: true,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_preflight_json_and_strict_suffix() -> Result<(), Box<dyn std::error::Error>> {
        // --json in the trailing position is stripped for structured commands;
        // --strict must survive that filter instead of being swallowed.
        let options = Options::parse([
            "preflight".to_owned(),
            "demo.md".to_owned(),
            "--json".to_owned(),
            "--strict".to_owned(),
        ])?;

        assert!(options.json);
        assert_eq!(
            options.command,
            Command::Preflight {
                article: PathBuf::from("demo.md"),
                strict: true,
            }
        );
        Ok(())
    }

    #[test]
    fn preflight_rejects_unknown_flag() {
        let err = Options::parse([
            "preflight".to_owned(),
            "demo.md".to_owned(),
            "--stric".to_owned(),
        ])
        .expect_err("misspelled --strict must not be silently ignored");
        assert!(matches!(err, AppError::UnknownOption(ref flag) if flag == "--stric"));
    }

    /// Commands whose documented usage is exactly `<path>` — no options at all.
    /// Their `rest[2..]` used to be silently dropped, so a stray flag or a
    /// second path was accepted as if the invocation were valid.
    const ZERO_FLAG_POSITIONAL_COMMANDS: &[&str] = &[
        "polish",
        "expand",
        "layout-audit",
        "check",
        "mark-ready",
        "humanize",
        "cards",
        "mark-published",
    ];

    #[test]
    fn zero_flag_positional_commands_reject_stray_flags() {
        for command in ZERO_FLAG_POSITIONAL_COMMANDS {
            let err = Options::parse([
                (*command).to_owned(),
                "demo.md".to_owned(),
                "--bogus".to_owned(),
            ])
            .expect_err("stray flag must not be silently ignored");
            assert!(
                matches!(err, AppError::UnknownOption(ref flag) if flag == "--bogus"),
                "{command}: expected UnknownOption(--bogus), got {err:?}"
            );
        }
    }

    #[test]
    fn zero_flag_positional_commands_reject_extra_positional() {
        for command in ZERO_FLAG_POSITIONAL_COMMANDS {
            let err = Options::parse([
                (*command).to_owned(),
                "demo.md".to_owned(),
                "extra.md".to_owned(),
            ])
            .expect_err("a second positional must not be silently ignored");
            assert!(
                matches!(err, AppError::UnknownCommand(ref value) if value == "extra.md"),
                "{command}: expected UnknownCommand(extra.md), got {err:?}"
            );
        }
    }

    #[test]
    fn zero_flag_positional_commands_keep_trailing_json_spelling() {
        // `supports_subcommand_json` only strips a trailing `--json` for
        // machine-readable commands. For the rest it stays in `rest[2..]`, so
        // the stray-arg guard must allow it or the long-standing
        // `cmd <path> --json` spelling turns into a spurious unknown option.
        for command in ZERO_FLAG_POSITIONAL_COMMANDS {
            Options::parse([
                (*command).to_owned(),
                "demo.md".to_owned(),
                "--json".to_owned(),
            ])
            .unwrap_or_else(|err| panic!("{command} <path> --json must still parse: {err:?}"));
        }
    }

    /// Commands that take no positional at all, only the global `--json`.
    /// Their `rest[1..]` used to be dropped wholesale, so `status junk` looked
    /// like a successful invocation.
    const ZERO_ARG_COMMANDS: &[&str] = &[
        "status",
        "version",
        "workspace",
        "layout-recipes",
        "list-drafts",
    ];

    #[test]
    fn zero_arg_commands_reject_stray_flags() {
        for command in ZERO_ARG_COMMANDS {
            let err = Options::parse([(*command).to_owned(), "--bogus".to_owned()])
                .expect_err("stray flag must not be silently ignored");
            assert!(
                matches!(err, AppError::UnknownOption(ref flag) if flag == "--bogus"),
                "{command}: expected UnknownOption(--bogus), got {err:?}"
            );
        }
    }

    #[test]
    fn zero_arg_commands_reject_extra_positional() {
        for command in ZERO_ARG_COMMANDS {
            let err = Options::parse([(*command).to_owned(), "junk".to_owned()])
                .expect_err("stray positional must not be silently ignored");
            assert!(
                matches!(err, AppError::UnknownCommand(ref value) if value == "junk"),
                "{command}: expected UnknownCommand(junk), got {err:?}"
            );
        }
    }

    #[test]
    fn zero_arg_commands_keep_trailing_json_spelling() {
        for command in ZERO_ARG_COMMANDS {
            Options::parse([(*command).to_owned(), "--json".to_owned()])
                .unwrap_or_else(|err| panic!("{command} --json must still parse: {err:?}"));
        }
    }

    /// `delete-draft` and `fetch` take exactly one positional and no options.
    /// `delete-draft` is destructive and has no dry-run mode, so a caller (or
    /// an agent) appending `--dry-run` used to delete the draft for real.
    #[test]
    fn delete_draft_rejects_dry_run_instead_of_deleting() {
        let err = Options::parse(argv("delete-draft", &["1234567890", "--dry-run"]))
            .expect_err("delete-draft has no dry-run mode; the flag must not be ignored");
        assert!(
            matches!(err, AppError::UnknownOption(ref flag) if flag == "--dry-run"),
            "expected UnknownOption(--dry-run), got {err:?}"
        );
    }

    #[test]
    fn fixed_arity_commands_reject_extra_positional() {
        for (command, args) in [
            ("delete-draft", &["1234567890", "extra"][..]),
            ("fetch", &["https://example.com/a", "extra"][..]),
        ] {
            let err = Options::parse(argv(command, args))
                .expect_err("a second positional must not be silently ignored");
            assert!(
                matches!(err, AppError::UnknownCommand(ref value) if value == "extra"),
                "{command}: expected UnknownCommand(extra), got {err:?}"
            );
        }
    }

    #[test]
    fn fixed_arity_commands_keep_trailing_json_spelling() {
        for (command, args) in [
            ("delete-draft", &["1234567890", "--json"][..]),
            ("fetch", &["https://example.com/a", "--json"][..]),
        ] {
            Options::parse(argv(command, args))
                .unwrap_or_else(|err| panic!("{command} <arg> --json must still parse: {err:?}"));
        }
    }

    #[test]
    fn fixed_arity_commands_still_parse_their_documented_form() {
        let parsed = Options::parse(argv("delete-draft", &["1234567890"]))
            .expect("delete-draft <media_id> is the documented form");
        assert_eq!(
            parsed.command,
            Command::DeleteDraft {
                media_id: "1234567890".to_owned()
            }
        );

        let parsed = Options::parse(argv("fetch", &["https://example.com/a"]))
            .expect("fetch <url> is the documented form");
        assert_eq!(
            parsed.command,
            Command::Fetch {
                url: "https://example.com/a".to_owned()
            }
        );
    }

    /// `new <title>` / `write <idea>` take free-form prose. `draft.rs` stores
    /// spaced titles verbatim, but the parser used to keep only `rest[1]` and
    /// silently drop every following word while still reporting success.
    const FREE_TEXT_COMMANDS: &[&str] = &["new", "write"];

    fn free_text_of(command: &str, args: &[&str]) -> String {
        match Options::parse(argv(command, args))
            .unwrap_or_else(|err| panic!("{command} {args:?} must parse: {err:?}"))
            .command
        {
            Command::New { title } => title,
            Command::Write { idea } => idea,
            other => panic!("{command}: expected a free-text command, got {other:?}"),
        }
    }

    #[test]
    fn free_text_commands_keep_every_word_of_a_spaced_title() {
        // The exact argv from a real terminal: no quotes around a Chinese title.
        for command in FREE_TEXT_COMMANDS {
            assert_eq!(
                free_text_of(command, &["我的", "第一篇", "文章"]),
                "我的 第一篇 文章",
                "{command} truncated the title"
            );
            assert_eq!(
                free_text_of(command, &["My", "First", "Post"]),
                "My First Post",
                "{command} truncated an English title"
            );
        }
    }

    #[test]
    fn free_text_commands_keep_the_documented_single_word_form() {
        for command in FREE_TEXT_COMMANDS {
            assert_eq!(free_text_of(command, &["我的文章标题"]), "我的文章标题");
            assert_eq!(
                free_text_of(command, &["写一篇关于读书的文章"]),
                "写一篇关于读书的文章"
            );
        }
    }

    #[test]
    fn free_text_commands_keep_trailing_json_out_of_the_title() {
        // `new` / `write` are not machine-readable commands, so a trailing
        // `--json` is not stripped by `supports_subcommand_json` and reaches
        // the arm. It must stay tolerated without becoming part of the title.
        for command in FREE_TEXT_COMMANDS {
            assert_eq!(free_text_of(command, &["demo", "--json"]), "demo");
            assert_eq!(
                free_text_of(command, &["我的", "文章", "--json"]),
                "我的 文章"
            );
        }
    }

    #[test]
    fn free_text_commands_reject_unknown_flags() {
        for command in FREE_TEXT_COMMANDS {
            let err = Options::parse(argv(command, &["demo", "--bogus"]))
                .expect_err("an unknown flag must not become part of the title");
            assert!(
                matches!(err, AppError::UnknownOption(ref flag) if flag == "--bogus"),
                "{command}: expected UnknownOption(--bogus), got {err:?}"
            );
        }
    }

    #[test]
    fn free_text_commands_require_actual_text() {
        for command in FREE_TEXT_COMMANDS {
            for args in [&[][..], &["--json"][..]] {
                let err = Options::parse(argv(command, args))
                    .expect_err("an empty title must not create an empty draft");
                assert!(
                    matches!(err, AppError::MissingValue(_)),
                    "{command} {args:?}: expected MissingValue, got {err:?}"
                );
            }
        }
    }

    #[test]
    fn help_rejects_extra_arguments() {
        for (args, expected) in [
            (&["preflight", "extra"][..], "extra"),
            (&["errors", "junk"][..], "junk"),
        ] {
            let err = Options::parse(argv("help", args))
                .expect_err("help used to silently drop everything after the topic");
            assert!(
                matches!(err, AppError::UnknownCommand(ref value) if value == expected),
                "help {args:?}: expected UnknownCommand({expected}), got {err:?}"
            );
        }
        let err = Options::parse(argv("help", &["preflight", "--bogus"]))
            .expect_err("help must not swallow an unknown flag either");
        assert!(matches!(err, AppError::UnknownOption(ref flag) if flag == "--bogus"));
    }

    #[test]
    fn help_keeps_documented_forms() {
        assert_eq!(
            Options::parse(argv("help", &[]))
                .expect("bare help")
                .command,
            Command::Help { topic: None }
        );
        assert_eq!(
            Options::parse(argv("help", &["preflight"]))
                .expect("help <command>")
                .command,
            Command::Help {
                topic: Some("preflight".to_owned())
            }
        );
        // `help` is in `supports_subcommand_json`, so every trailing `--json`
        // is stripped before the guard runs.
        for args in [
            &["--json"][..],
            &["preflight", "--json"][..],
            &["errors", "--json"][..],
        ] {
            Options::parse(argv("help", args))
                .unwrap_or_else(|err| panic!("help {args:?} must still parse: {err:?}"));
        }
    }

    #[test]
    fn test_yulan_flag_values_reject_a_following_flag() {
        // PR #171 added the guard inside `flag_value`; `test-yulan` kept its
        // own index loop and bypassed it, turning `--title --headed` into a
        // title of "--headed".
        for flag in ["--title", "--to"] {
            let err = Options::parse(argv("test-yulan", &[flag, "--headed"]))
                .expect_err("a flag is not a value");
            assert!(
                matches!(err, AppError::MissingValue(_)),
                "test-yulan {flag} --headed: expected MissingValue, got {err:?}"
            );
        }
    }

    #[test]
    fn test_yulan_missing_value_at_end_reports_flag_name() {
        for flag in ["--title", "--to"] {
            let err =
                Options::parse(argv("test-yulan", &[flag])).expect_err("missing value at end");
            assert!(
                matches!(err, AppError::MissingValue(_)),
                "test-yulan {flag}: expected MissingValue, got {err:?}"
            );
        }
    }

    #[test]
    fn test_yulan_still_parses_its_documented_form() {
        let parsed = Options::parse(argv(
            "test-yulan",
            &["--to", "wxid_demo", "--title", "AI 日报", "--headed"],
        ))
        .expect("documented test-yulan form");
        assert_eq!(
            parsed.command,
            Command::TestYulan {
                headed: true,
                temporary_profile: false,
                title: Some("AI 日报".to_owned()),
                to_wxname: Some("wxid_demo".to_owned()),
            }
        );
    }

    /// Commands whose usage is `<article.md>` plus flags that may take a value.
    /// Each entry is `(command, flags in their canonical after-the-path form)`.
    const VALUE_FLAG_POSITIONAL_COMMANDS: &[(&str, &[&str])] = &[
        (
            "render",
            &["--author", "qiao", "--thumb", "m1", "--humanize"],
        ),
        ("cover", &["--style", "ai-lab", "--screenshot"]),
        (
            "publish",
            &[
                "--target",
                "wechat-draft",
                "--render",
                "--temporary-profile",
            ],
        ),
        ("ship", &["--style", "ai-lab", "--ai"]),
        ("update-draft", &["--media-id", "42"]),
        ("export", &["--target", "zola"]),
    ];

    fn argv(command: &str, args: &[&str]) -> Vec<String> {
        std::iter::once(command.to_owned())
            .chain(args.iter().map(|arg| (*arg).to_owned()))
            .collect()
    }

    #[test]
    fn value_flag_commands_accept_flags_before_path() {
        for (command, flags) in VALUE_FLAG_POSITIONAL_COMMANDS {
            let mut after = vec![(*command).to_owned(), "demo.md".to_owned()];
            after.extend(flags.iter().map(|flag| (*flag).to_owned()));
            let mut before = vec![(*command).to_owned()];
            before.extend(flags.iter().map(|flag| (*flag).to_owned()));
            before.push("demo.md".to_owned());

            let after = Options::parse(after)
                .unwrap_or_else(|err| panic!("{command} <path> <flags> must parse: {err:?}"));
            let before = Options::parse(before)
                .unwrap_or_else(|err| panic!("{command} <flags> <path> must parse: {err:?}"));
            assert_eq!(
                before.command, after.command,
                "{command}: moving flags before the path changed the parsed command"
            );
        }
    }

    #[test]
    fn value_flag_commands_reject_extra_positional() {
        for (command, _) in VALUE_FLAG_POSITIONAL_COMMANDS {
            let err = Options::parse(argv(command, &["demo.md", "extra.md"]))
                .expect_err("a second positional must not be silently ignored");
            assert!(
                matches!(err, AppError::UnknownCommand(ref value) if value == "extra.md"),
                "{command}: expected UnknownCommand(extra.md), got {err:?}"
            );
        }
    }

    #[test]
    fn value_flag_commands_reject_unknown_flag_on_either_side() {
        for (command, _) in VALUE_FLAG_POSITIONAL_COMMANDS {
            for args in [
                argv(command, &["demo.md", "--bogus"]),
                argv(command, &["--bogus", "demo.md"]),
            ] {
                let err = Options::parse(args).expect_err("unknown flag must be rejected");
                assert!(
                    matches!(err, AppError::UnknownOption(ref flag) if flag == "--bogus"),
                    "{command}: expected UnknownOption(--bogus), got {err:?}"
                );
            }
        }
    }

    #[test]
    fn value_flag_commands_reject_flag_as_value() {
        let cases: &[(&str, &[&str], &str)] = &[
            ("cover", &["--style", "--screenshot", "demo.md"], "--style"),
            ("ship", &["--style", "--ai", "demo.md"], "--style"),
            ("render", &["--author", "--humanize", "demo.md"], "--author"),
            ("publish", &["--target", "--render", "demo.md"], "--target"),
            (
                "update-draft",
                &["--media-id", "--bogus", "demo.md"],
                "--media-id",
            ),
            ("export", &["--target", "--bogus", "demo.md"], "--target"),
        ];
        for (command, args, flag) in cases {
            let err = Options::parse(argv(command, args))
                .expect_err("a flag must not be consumed as a value");
            assert!(
                matches!(err, AppError::MissingValue(name) if name == *flag),
                "{command}: expected MissingValue({flag}), got {err:?}"
            );
        }
    }

    #[test]
    fn value_flag_commands_require_a_path() {
        for (command, _) in VALUE_FLAG_POSITIONAL_COMMANDS {
            let err = Options::parse([(*command).to_owned()]).expect_err("path is required");
            assert!(
                matches!(err, AppError::MissingValue(usage) if usage.contains(*command)),
                "{command}: expected MissingValue(usage), got {err:?}"
            );
        }

        let err = Options::parse(argv("cover", &["--style", "ai-lab"]))
            .expect_err("flags without a path must fail");
        assert!(
            matches!(err, AppError::MissingValue(usage) if usage.contains("cover")),
            "cover: expected MissingValue(usage), got {err:?}"
        );
    }

    #[test]
    fn value_flag_missing_value_at_end_reports_flag_name() {
        let cases: &[(&str, &[&str], &str)] = &[
            ("cover", &["demo.md", "--style"], "--style"),
            ("render", &["demo.md", "--thumb"], "--thumb"),
            ("export", &["demo.md", "--target"], "--target"),
        ];
        for (command, args, flag) in cases {
            let err =
                Options::parse(argv(command, args)).expect_err("missing value must be rejected");
            assert!(
                matches!(err, AppError::MissingValue(name) if name == *flag),
                "{command}: expected MissingValue({flag}), got {err:?}"
            );
        }
    }

    #[test]
    fn ship_ai_before_path_matches_obsidian_plugin_argv() -> Result<(), Box<dyn std::error::Error>>
    {
        // obsidian-plugin/main.ts builds `args.push(...subArgs, filePath)` with
        // subArgs = "ship --ai", so the flag sits before the path. This argv
        // used to parse the path as "--ai".
        let options = Options::parse([
            "--articles".to_owned(),
            "/art".to_owned(),
            "ship".to_owned(),
            "--ai".to_owned(),
            "drafts/post.md".to_owned(),
        ])?;
        let Command::ShipAi { article, style } = options.command else {
            panic!("expected ShipAi");
        };
        assert_eq!(article, PathBuf::from("drafts/post.md"));
        assert_eq!(style, None);
        Ok(())
    }

    #[test]
    fn path_and_flags_with_values_preserves_token_order() {
        let rest = argv(
            "render",
            &["--author", "qiao", "--bogus", "demo.md", "extra.md"],
        );
        let (path, tokens) =
            path_and_flags_with_values(&rest, "render <article.md>", &["--author"]).unwrap();

        assert_eq!(path, PathBuf::from("demo.md"));
        assert_eq!(
            tokens
                .iter()
                .map(|token| token.as_str())
                .collect::<Vec<_>>(),
            vec!["--author", "qiao", "--bogus", "extra.md"]
        );
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
                analyze_images: false,
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
                analyze_images: false,
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
    fn parses_intake_photos_with_visual_analysis() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "intake".to_owned(),
            "photos".to_owned(),
            "photos/day1".to_owned(),
            "--analyze-images".to_owned(),
            "--draft".to_owned(),
            "--preview".to_owned(),
            "--no-open".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::IntakePhotos {
                inputs: vec![PathBuf::from("photos/day1")],
                analyze_images: true,
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

    /// Both intake sources used to assume the positional came first. A flag in
    /// front of it was reported as a missing file, and a second path after a
    /// flag became an unrelated unknown command.
    #[test]
    fn intake_photos_accepts_flags_in_any_position() -> Result<(), Box<dyn std::error::Error>> {
        let cases = [
            (
                vec!["--draft", "photos/day1"],
                vec!["photos/day1"],
                true,
                false,
            ),
            (
                vec!["--analyze-images", "a.jpg", "b.jpg"],
                vec!["a.jpg", "b.jpg"],
                false,
                true,
            ),
            (
                vec!["a.jpg", "--draft", "b.jpg", "--analyze-images"],
                vec!["a.jpg", "b.jpg"],
                true,
                true,
            ),
        ];
        for (args, inputs, draft, analyze_images) in cases {
            let mut argv = vec!["intake".to_owned(), "photos".to_owned()];
            argv.extend(args.into_iter().map(str::to_owned));
            let options = Options::parse(argv)?;
            let Command::IntakePhotos {
                inputs: got,
                analyze_images: got_analyze,
                draft: got_draft,
                ..
            } = options.command
            else {
                panic!("expected IntakePhotos");
            };
            assert_eq!(
                got,
                inputs.into_iter().map(PathBuf::from).collect::<Vec<_>>()
            );
            assert_eq!(got_draft, draft);
            assert_eq!(got_analyze, analyze_images);
        }
        Ok(())
    }

    #[test]
    fn intake_feishu_all_source_shapes_accept_flags_first() -> Result<(), Box<dyn std::error::Error>>
    {
        let cases = [
            (
                vec!["--draft", "minutes.txt"],
                FeishuIntakeSource::File(PathBuf::from("minutes.txt")),
            ),
            (vec!["--draft", "--latest"], FeishuIntakeSource::Latest),
            (
                vec!["--draft", "--query", "散步"],
                FeishuIntakeSource::Query("散步".to_owned()),
            ),
            (
                vec!["--draft", "--minute-token", "obcn123"],
                FeishuIntakeSource::MinuteToken("obcn123".to_owned()),
            ),
        ];
        for (args, source) in cases {
            let mut argv = vec!["intake".to_owned(), "feishu".to_owned()];
            argv.extend(args.into_iter().map(str::to_owned));
            let options = Options::parse(argv)?;
            assert_eq!(
                options.command,
                Command::IntakeFeishu {
                    source,
                    draft: true,
                    preview: PreviewOptions::default(),
                    auto_push: false,
                }
            );
        }
        Ok(())
    }

    #[test]
    fn intake_feishu_valued_flags_work_with_flags_on_both_sides()
    -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(
            [
                "intake",
                "feishu",
                "--draft",
                "--query",
                "散步",
                "--preview",
                "--no-open",
            ]
            .map(str::to_owned),
        )?;
        assert_eq!(
            options.command,
            Command::IntakeFeishu {
                source: FeishuIntakeSource::Query("散步".to_owned()),
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

    /// `--query --draft --push` used to make `--draft` the keyword and then
    /// complain that `--push` needs `--draft`: the switch the caller did pass
    /// vanished into a value.
    #[test]
    fn intake_feishu_valued_flags_reject_a_following_switch() {
        let cases = [
            (
                vec!["--query", "--draft", "--push"],
                "intake feishu --query <keyword>",
            ),
            (
                vec!["--minute-token", "--draft"],
                "intake feishu --minute-token <token>",
            ),
            (vec!["--query"], "intake feishu --query <keyword>"),
            (
                vec!["--minute-token"],
                "intake feishu --minute-token <token>",
            ),
        ];
        for (args, usage) in cases {
            let mut argv = vec!["intake".to_owned(), "feishu".to_owned()];
            argv.extend(args.into_iter().map(str::to_owned));
            let err = Options::parse(argv).unwrap_err();
            assert_eq!(
                err.to_string(),
                format!("missing value for {usage}"),
                "{err}"
            );
        }
    }

    #[test]
    fn intake_feishu_rejects_conflicting_or_missing_sources() {
        let cases = [
            vec!["--latest", "minutes.txt"],
            vec!["--query", "散步", "minutes.txt"],
            vec!["--query", "a", "--minute-token", "b"],
            vec!["--latest", "--query", "散步"],
            vec!["minutes.txt", "other.txt"],
            vec!["--draft"],
        ];
        for args in cases {
            let mut argv = vec!["intake".to_owned(), "feishu".to_owned()];
            argv.extend(args.into_iter().map(str::to_owned));
            let err = Options::parse(argv).unwrap_err();
            assert_eq!(
                err.to_string(),
                format!("missing value for {INTAKE_FEISHU_USAGE}"),
                "{err}"
            );
        }
    }

    #[test]
    fn intake_rejects_unknown_flags_in_any_position() {
        let cases = [
            vec!["intake", "photos", "--bogus", "a.jpg"],
            vec!["intake", "photos", "a.jpg", "--bogus"],
            vec!["intake", "feishu", "--bogus", "minutes.txt"],
            vec!["intake", "feishu", "minutes.txt", "--bogus"],
        ];
        for args in cases {
            let err = Options::parse(args.into_iter().map(str::to_owned)).unwrap_err();
            assert!(err.to_string().contains("unknown option: --bogus"), "{err}");
        }
    }

    #[test]
    fn intake_photos_requires_a_path_even_when_flags_are_present() {
        let cases = [
            vec!["intake", "photos"],
            vec!["intake", "photos", "--draft"],
            vec!["intake", "photos", "--analyze-images"],
        ];
        for args in cases {
            let err = Options::parse(args.into_iter().map(str::to_owned)).unwrap_err();
            assert_eq!(
                err.to_string(),
                format!("missing value for {INTAKE_PHOTOS_USAGE}"),
                "{err}"
            );
        }
    }

    #[test]
    fn intake_flag_order_does_not_change_the_command() -> Result<(), Box<dyn std::error::Error>> {
        let trailing = Options::parse(
            [
                "intake",
                "photos",
                "a.jpg",
                "b.jpg",
                "--draft",
                "--preview",
                "--no-open",
                "--json",
            ]
            .map(str::to_owned),
        )?;
        let interleaved = Options::parse(
            [
                "intake",
                "photos",
                "--draft",
                "a.jpg",
                "--preview",
                "b.jpg",
                "--no-open",
                "--json",
            ]
            .map(str::to_owned),
        )?;
        assert_eq!(trailing.command, interleaved.command);
        assert!(trailing.json && interleaved.json);

        let trailing_feishu = Options::parse(
            ["intake", "feishu", "minutes.txt", "--draft", "--push"].map(str::to_owned),
        )?;
        let leading_feishu = Options::parse(
            ["intake", "feishu", "--draft", "--push", "minutes.txt"].map(str::to_owned),
        )?;
        assert_eq!(trailing_feishu.command, leading_feishu.command);
        Ok(())
    }

    /// Mirrors the real-binary probe: `work/moonpub.toml` (blueprint) belongs
    /// to the article, `decoy/moonpub.toml` (ai-lab) belongs to an unrelated
    /// directory that happens to be an ancestor of `--articles`.
    fn config_probe_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "moonpub-config-anchor-{}-{name}",
            std::process::id()
        ));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(root.join("work/Articles/drafts")).unwrap();
        std::fs::create_dir_all(root.join("empty")).unwrap();
        std::fs::create_dir_all(root.join("decoy/sub")).unwrap();
        std::fs::write(
            root.join("work/moonpub.toml"),
            "[wechat]\ntheme = \"blueprint\"\n",
        )
        .unwrap();
        std::fs::write(
            root.join("decoy/moonpub.toml"),
            "[wechat]\ntheme = \"ai-lab\"\n",
        )
        .unwrap();
        std::fs::write(
            root.join("work/Articles/drafts/demo.md"),
            "---\ntitle: Demo\n---\n\n正文\n",
        )
        .unwrap();
        root
    }

    /// `render --author Alice <article>` used to pick `Alice` as the config
    /// anchor, silently producing a different theme than the same command
    /// with the article first. The same applied to `--style` and to the
    /// `intake photos` subcommand name.
    #[test]
    fn config_discovery_does_not_depend_on_flag_position() {
        let root = config_probe_root("flag-position");
        let article = root
            .join("work/Articles/drafts/demo.md")
            .display()
            .to_string();
        let empty = root.join("empty").display().to_string();
        let expected = root.join("work/moonpub.toml");
        let cases = [
            vec!["render", article.as_str(), "--author", "FLAG_AUTHOR"],
            vec!["render", "--author", "FLAG_AUTHOR", article.as_str()],
            vec!["cover", "--style", "dark", article.as_str()],
            vec!["cover", article.as_str(), "--style", "dark"],
            vec!["intake", "photos", "--draft", article.as_str()],
            vec!["intake", "photos", article.as_str(), "--draft"],
            vec!["preview", "--no-open", article.as_str()],
            vec!["check", article.as_str()],
        ];
        for args in cases {
            let mut argv = vec!["--articles".to_owned(), empty.clone(), "--json".to_owned()];
            argv.extend(args.iter().map(|s| (*s).to_owned()));
            let options = Options::parse(argv).unwrap();
            assert_eq!(options.config, Some(expected.clone()), "argv={args:?}");
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    /// A stray token must not make the parser adopt a config from a completely
    /// different project (the `--articles` ancestor), even when that config
    /// exists and the real article's config is further away.
    #[test]
    fn config_discovery_does_not_adopt_an_unrelated_ancestor_config() {
        let root = config_probe_root("decoy");
        let article = root
            .join("work/Articles/drafts/demo.md")
            .display()
            .to_string();
        let decoy_sub = root.join("decoy/sub").display().to_string();
        let expected = root.join("work/moonpub.toml");
        for args in [
            vec!["render", article.as_str(), "--author", "FLAG_AUTHOR"],
            vec!["render", "--author", "FLAG_AUTHOR", article.as_str()],
        ] {
            let mut argv = vec![
                "--articles".to_owned(),
                decoy_sub.clone(),
                "--json".to_owned(),
            ];
            argv.extend(args.iter().map(|s| (*s).to_owned()));
            let options = Options::parse(argv).unwrap();
            assert_eq!(options.config, Some(expected.clone()), "argv={args:?}");
        }
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn config_discovery_keeps_explicit_config_and_no_path_commands() {
        let root = config_probe_root("explicit");
        let empty = root.join("empty").display().to_string();
        let explicit = root.join("decoy/moonpub.toml");
        let options = Options::parse([
            "--articles".to_owned(),
            empty.clone(),
            "--config".to_owned(),
            explicit.display().to_string(),
            "--json".to_owned(),
            "render".to_owned(),
            "--author".to_owned(),
            "FLAG_AUTHOR".to_owned(),
            root.join("work/Articles/drafts/demo.md")
                .display()
                .to_string(),
        ])
        .unwrap();
        assert_eq!(options.config, Some(explicit));

        let status = Options::parse(["--articles".to_owned(), empty, "status".to_owned()]).unwrap();
        assert_eq!(status.config, None);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn config_anchor_candidates_prefer_existing_paths() {
        let root = config_probe_root("candidates");
        let article = root
            .join("work/Articles/drafts/demo.md")
            .display()
            .to_string();
        let rest = vec![
            "render".to_owned(),
            "--author".to_owned(),
            "FLAG_AUTHOR".to_owned(),
            article.clone(),
        ];
        let candidates = super::config_anchor_candidates(&rest, &root.join("empty"));
        assert_eq!(candidates.first(), Some(&PathBuf::from(article)));
        let _ = std::fs::remove_dir_all(&root);
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

    /// Boolean flags may appear before the positional argument, not only after.
    /// Regression guard for the fixed-index parsing that used to read the flag
    /// as the article path and report the real path as an unknown command.
    #[test]
    fn parses_preview_no_open_before_article() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "preview".to_owned(),
            "--no-open".to_owned(),
            "Articles/drafts/demo.md".to_owned(),
        ])?;
        let Command::Preview { article, open } = options.command else {
            panic!("expected Preview");
        };
        assert_eq!(article, PathBuf::from("Articles/drafts/demo.md"));
        assert!(!open);
        Ok(())
    }

    #[test]
    fn parses_push_flags_before_article() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "push".to_owned(),
            "--render".to_owned(),
            "--temporary-profile".to_owned(),
            "Articles/ready/demo.md".to_owned(),
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
    fn parses_preflight_strict_before_article() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "preflight".to_owned(),
            "--strict".to_owned(),
            "Articles/ready/demo.md".to_owned(),
        ])?;
        let Command::Preflight { article, strict } = options.command else {
            panic!("expected Preflight");
        };
        assert_eq!(article, PathBuf::from("Articles/ready/demo.md"));
        assert!(strict);
        Ok(())
    }

    #[test]
    fn parses_draft_from_inbox_flags_before_article() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "draft-from-inbox".to_owned(),
            "--preview".to_owned(),
            "--no-open".to_owned(),
            "Articles/Inbox/demo.md".to_owned(),
        ])?;
        let Command::DraftFromInbox {
            input,
            preview,
            auto_push,
        } = options.command
        else {
            panic!("expected DraftFromInbox");
        };
        assert_eq!(input, PathBuf::from("Articles/Inbox/demo.md"));
        assert!(preview.enabled);
        assert!(!preview.open);
        assert!(!auto_push);
        Ok(())
    }

    #[test]
    fn preview_rejects_unknown_flag_regardless_of_position() {
        let result = Options::parse([
            "preview".to_owned(),
            "Articles/drafts/demo.md".to_owned(),
            "--bogus".to_owned(),
        ]);
        assert!(result.is_err());
    }

    #[test]
    fn preview_rejects_extra_positional_regardless_of_position() {
        let result = Options::parse([
            "preview".to_owned(),
            "Articles/drafts/demo.md".to_owned(),
            "extra.md".to_owned(),
        ]);
        assert!(result.is_err());
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
    fn parses_cards_command() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse(["cards".to_owned(), "ready/post.md".to_owned()])?;
        let Command::Cards { article } = options.command else {
            panic!("expected Cards");
        };
        assert_eq!(article, PathBuf::from("ready/post.md"));
        Ok(())
    }

    #[test]
    fn cards_command_supports_subcommand_json_suffix() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "cards".to_owned(),
            "ready/post.md".to_owned(),
            "--json".to_owned(),
        ])?;
        let Command::Cards { article } = options.command else {
            panic!("expected Cards");
        };
        assert_eq!(article, PathBuf::from("ready/post.md"));
        assert!(options.json);
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
                evidence_dir: None,
                title: None,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_configure_with_evidence_dir() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "configure".to_owned(),
            "--headed".to_owned(),
            "--evidence-dir".to_owned(),
            "docs/first-run-evidence/wechat".to_owned(),
            "yulan".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::Configure {
                steps: vec!["yulan".to_owned()],
                headed: true,
                temporary_profile: false,
                evidence_dir: Some(PathBuf::from("docs/first-run-evidence/wechat")),
                title: None,
            }
        );
        Ok(())
    }

    /// `configure <step>..` accepts a fixed set of step names; a typo used to
    /// be collected as a step that never matches any `run_step` guard, so the
    /// step silently did not run while `configure` still reported success.
    #[test]
    fn configure_rejects_unknown_step() {
        for bogus in ["zanshangg", "YUANZHUANG", "clips", "原创"] {
            let err = Options::parse(vec!["configure".to_owned(), bogus.to_owned()]).unwrap_err();
            assert!(
                matches!(err, AppError::UnknownCommand(_)),
                "step {bogus:?} should be rejected, got {err:?}"
            );
        }
    }

    #[test]
    fn configure_accepts_every_known_step() -> Result<(), Box<dyn std::error::Error>> {
        let steps = [
            "yuanzhuang",
            "zanshang",
            "liuyan",
            "chuangzuo",
            "moban",
            "yulan",
        ];
        let args: Vec<String> = std::iter::once("configure".to_owned())
            .chain(steps.iter().map(|s| (*s).to_owned()))
            .collect();
        let options = Options::parse(args)?;
        match options.command {
            Command::Configure { steps: parsed, .. } => assert_eq!(parsed, steps),
            other => panic!("expected Configure, got {other:?}"),
        }
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
                title: None,
                to_wxname: None,
            }
        );
        Ok(())
    }

    #[test]
    fn parses_test_yulan_with_title() -> Result<(), Box<dyn std::error::Error>> {
        let options = Options::parse([
            "test-yulan".to_owned(),
            "--title".to_owned(),
            "AI · Web3 最新日报｜2026-07-14".to_owned(),
        ])?;

        assert_eq!(
            options.command,
            Command::TestYulan {
                headed: false,
                temporary_profile: false,
                title: Some("AI · Web3 最新日报｜2026-07-14".to_owned()),
                to_wxname: None,
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

    #[test]
    fn structured_json_output_is_declared_by_command() {
        assert!(Command::Doctor.has_structured_json_output());
        assert!(
            Command::IntakeFeishu {
                source: FeishuIntakeSource::Latest,
                draft: true,
                preview: PreviewOptions::default(),
                auto_push: false,
            }
            .has_structured_json_output()
        );
        assert!(
            Command::IntakePhotos {
                inputs: vec![PathBuf::from("photos")],
                analyze_images: false,
                draft: true,
                preview: PreviewOptions::default(),
                auto_push: false,
            }
            .has_structured_json_output()
        );
        assert!(
            !Command::IntakeFeishu {
                source: FeishuIntakeSource::Latest,
                draft: false,
                preview: PreviewOptions::default(),
                auto_push: false,
            }
            .has_structured_json_output()
        );
        assert!(
            !Command::Init {
                path: PathBuf::from("moonpub.toml"),
                yes: false,
            }
            .has_structured_json_output()
        );
    }
}
