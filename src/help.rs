//! Single source of truth for CLI help text.
//!
//! Before this module the help was one long prose string that repeated
//! `[--articles <path>] [--config <moonpub.toml>] [--json]` on all 40+ usage
//! lines, could not answer `moonpub help render`, and dumped ~90 lines even for
//! a typo. Here every command is declared once, and the overview, the
//! per-command page and the "did you mean" hint are all derived from it.

const NAME_WIDTH: usize = 19;

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Group {
    Start,
    Write,
    Render,
    Publish,
    Import,
    Integrate,
}

impl Group {
    fn title(self) -> &'static str {
        match self {
            Group::Start => "Getting started",
            Group::Write => "Writing",
            Group::Render => "Render & review",
            Group::Publish => "Publish & WeChat backend",
            Group::Import => "Import & research",
            Group::Integrate => "Integration & release",
        }
    }

    /// Stable id for `--json help`. The titles are prose and may be reworded;
    /// consumers should group on these instead.
    pub fn id(self) -> &'static str {
        match self {
            Group::Start => "start",
            Group::Write => "write",
            Group::Render => "render",
            Group::Publish => "publish",
            Group::Import => "import",
            Group::Integrate => "integrate",
        }
    }

    const ALL: [Group; 6] = [
        Group::Start,
        Group::Write,
        Group::Render,
        Group::Publish,
        Group::Import,
        Group::Integrate,
    ];
}

pub struct CommandSpec {
    pub name: &'static str,
    pub group: Group,
    /// Everything after the command name, without the global-option prefix.
    pub usage: &'static str,
    pub summary: &'static str,
    /// Extra lines shown on `moonpub help <command>`.
    pub notes: &'static [&'static str],
}

pub const COMMANDS: &[CommandSpec] = &[
    CommandSpec {
        name: "init",
        group: Group::Start,
        usage: "init [moonpub.toml] [--yes]",
        summary: "Create a sample moonpub.toml",
        notes: &[
            "Without --yes it runs an interactive wizard; --yes takes the defaults.",
            "Safe to re-run: it never overwrites an existing config.",
        ],
    },
    CommandSpec {
        name: "doctor",
        group: Group::Start,
        usage: "doctor",
        summary: "Check local readiness for first use",
        notes: &["Read-only: no WeChat API call, no browser."],
    },
    CommandSpec {
        name: "workspace",
        group: Group::Start,
        usage: "workspace",
        summary: "Summarize the workspace and suggest the next command",
        notes: &[],
    },
    CommandSpec {
        name: "status",
        group: Group::Start,
        usage: "status",
        summary: "List articles in drafts, ready and published",
        notes: &[],
    },
    CommandSpec {
        name: "help",
        group: Group::Start,
        usage: "help [<command>]",
        summary: "Show the command list, or one command's usage",
        notes: &[],
    },
    CommandSpec {
        name: "version",
        group: Group::Start,
        usage: "version",
        summary: "Print the moonpub version",
        notes: &[],
    },
    CommandSpec {
        name: "new",
        group: Group::Write,
        usage: "new <title>",
        summary: "Scaffold an article with frontmatter and body template",
        notes: &[],
    },
    CommandSpec {
        name: "write",
        group: Group::Write,
        usage: "write <idea>",
        summary: "Generate an article from an idea",
        notes: &["Requires DEEPSEEK_API_KEY."],
    },
    CommandSpec {
        name: "expand",
        group: Group::Write,
        usage: "expand <article.md>",
        summary: "Expand reading notes into a full article",
        notes: &["Requires DEEPSEEK_API_KEY."],
    },
    CommandSpec {
        name: "polish",
        group: Group::Write,
        usage: "polish <article.md>",
        summary: "AI polish and de-AI-ify an existing article",
        notes: &["Requires DEEPSEEK_API_KEY."],
    },
    CommandSpec {
        name: "humanize",
        group: Group::Write,
        usage: "humanize <article.md>",
        summary: "Strip AI writing patterns offline",
        notes: &["No API key needed."],
    },
    CommandSpec {
        name: "draft-from-inbox",
        group: Group::Write,
        usage: "draft-from-inbox <inbox.md> [--preview] [--no-open] [--push]",
        summary: "Turn Inbox source material into an editable draft",
        notes: &[
            "Requires an AI API key.",
            "--preview is the recommended path; --push explicitly continues into a WeChat draft.",
        ],
    },
    CommandSpec {
        name: "render",
        group: Group::Render,
        usage: "render <article.md> [--author <name>] [--thumb <media_id>]",
        summary: "Generate <slug>.html and <slug>.draft.json from Markdown",
        notes: &["--thumb sets the cover image media_id for the draft."],
    },
    CommandSpec {
        name: "check",
        group: Group::Render,
        usage: "check <article.md>",
        summary: "Check that an article bundle has md / html / draft.json",
        notes: &[],
    },
    CommandSpec {
        name: "preflight",
        group: Group::Render,
        usage: "preflight <article.md>",
        summary: "Read-only publish gate: bundle, layout audit, next action",
        notes: &["Does not call the WeChat API and does not open a browser."],
    },
    CommandSpec {
        name: "layout-audit",
        group: Group::Render,
        usage: "layout-audit <html>",
        summary: "Flag WeChat editor compatibility risks in rendered HTML",
        notes: &[],
    },
    CommandSpec {
        name: "layout-recipes",
        group: Group::Render,
        usage: "layout-recipes",
        summary: "List layout recipes and recommended themes",
        notes: &[],
    },
    CommandSpec {
        name: "preview",
        group: Group::Render,
        usage: "preview <article.md> [--no-open]",
        summary: "Open the rendered HTML in your browser",
        notes: &[
            "This is a local HTML preview, not the WeChat backend preview-send.",
            "--no-open only prints the HTML path.",
        ],
    },
    CommandSpec {
        name: "cover",
        group: Group::Render,
        usage: "cover <article.md> [--style <name>] [--screenshot]",
        summary: "Generate a cover from article frontmatter",
        notes: &[
            "Style order: --style > frontmatter cover_style > theme > editorial.",
            "ship uses the same order, so both commands render one article identically.",
            "Run `moonpub layout-recipes` for the full list of style names.",
            "Title: cover_title > title > first # heading > file stem. Subtitle: cover_subtitle > digest.",
        ],
    },
    CommandSpec {
        name: "cards",
        group: Group::Render,
        usage: "cards <article.md>",
        summary: "Split an article into Xiaohongshu-style vertical cards",
        notes: &["Offline: it produces card HTML and copy, not images."],
    },
    CommandSpec {
        name: "login",
        group: Group::Publish,
        usage: "login [--temporary-profile]",
        summary: "One-time WeChat backend login by QR scan",
        notes: &["Only needed when [wechat] auth_method = \"cookie\"."],
    },
    CommandSpec {
        name: "wechat-health",
        group: Group::Publish,
        usage: "wechat-health [--headed] [--temporary-profile]",
        summary: "Check whether the saved browser session is reusable",
        notes: &["Run this before configure or a backend preview-send."],
    },
    CommandSpec {
        name: "push",
        group: Group::Publish,
        usage: "push <article.md> [--render] [--temporary-profile]",
        summary: "Push a draft, write .media_id, move the bundle to ready/",
        notes: &[
            "Set [wechat] auth_method = \"cookie\" to bypass the IP whitelist via the browser session.",
            "--temporary-profile isolates the post-push browser automation profile.",
        ],
    },
    CommandSpec {
        name: "publish",
        group: Group::Publish,
        usage: "publish <article.md> --target wechat-draft [--render] [--temporary-profile]",
        summary: "Generic publish entrypoint",
        notes: &["Only wechat-draft is implemented today."],
    },
    CommandSpec {
        name: "update-draft",
        group: Group::Publish,
        usage: "update-draft <article.md> [--media-id <id>]",
        summary: "Re-push updated HTML to an existing WeChat draft",
        notes: &[],
    },
    CommandSpec {
        name: "ship",
        group: Group::Publish,
        usage: "ship <article.md> [--style <name>] [--ai]",
        summary: "Cover + render + push + configure + export in one go",
        notes: &["Final WeChat publication stays manual by design."],
    },
    CommandSpec {
        name: "configure",
        group: Group::Publish,
        usage: "configure [<step>..] [--headed] [--temporary-profile] [--evidence-dir <dir>]",
        summary: "Configure 原创 / 赞赏 / 留言 / 创作来源 and preview-send",
        notes: &[
            "The draft must already exist in the WeChat backend.",
            "The first run needs a recipient: --to, WECHAT_PREVIEW_TO, or .moonpub/preview_to.",
        ],
    },
    CommandSpec {
        name: "step-test",
        group: Group::Publish,
        usage: "step-test [--headed] [--temporary-profile]",
        summary: "Interactive browser automation test",
        notes: &[],
    },
    CommandSpec {
        name: "test-zanshang",
        group: Group::Publish,
        usage: "test-zanshang [--headed] [--temporary-profile]",
        summary: "Test the 赞赏 (reward) step only",
        notes: &[],
    },
    CommandSpec {
        name: "test-chuangzuo",
        group: Group::Publish,
        usage: "test-chuangzuo [--headed] [--temporary-profile]",
        summary: "Test the 创作来源 (creation source) step only",
        notes: &[],
    },
    CommandSpec {
        name: "test-yulan",
        group: Group::Publish,
        usage: "test-yulan [--to <wxid>] [--title <text>] [--headed] [--temporary-profile]",
        summary: "Test the backend preview-send step only",
        notes: &[
            "The recipient comes from --to, WECHAT_PREVIEW_TO, or .moonpub/preview_to.",
            "--title selects an exact draft.",
        ],
    },
    CommandSpec {
        name: "list-drafts",
        group: Group::Publish,
        usage: "list-drafts",
        summary: "List all drafts with media_id and title",
        notes: &[],
    },
    CommandSpec {
        name: "delete-draft",
        group: Group::Publish,
        usage: "delete-draft <media_id>",
        summary: "Delete a draft by media_id",
        notes: &[],
    },
    CommandSpec {
        name: "mark-ready",
        group: Group::Publish,
        usage: "mark-ready <article.md>",
        summary: "Move a bundle to Articles/ready",
        notes: &[],
    },
    CommandSpec {
        name: "mark-published",
        group: Group::Publish,
        usage: "mark-published <article.md>",
        summary: "Move a bundle to Articles/published",
        notes: &[],
    },
    CommandSpec {
        name: "export",
        group: Group::Publish,
        usage: "export <article.md> [--target zola]",
        summary: "Generic export entrypoint",
        notes: &["Only the zola target is implemented today."],
    },
    CommandSpec {
        name: "fetch",
        group: Group::Import,
        usage: "fetch <url>",
        summary: "Fetch a WeChat article and extract title + body",
        notes: &["Requires Chrome."],
    },
    CommandSpec {
        name: "intake",
        group: Group::Import,
        usage: "intake feishu <file> | --latest | --query <kw> | --minute-token <t>",
        summary: "Import Feishu minutes or photos into the Obsidian Inbox",
        notes: &[
            "intake photos <file-or-dir>... [--analyze-images] [--draft] [--preview] [--push]",
            "The conservative default for both sources is --draft --preview.",
        ],
    },
    CommandSpec {
        name: "radar",
        group: Group::Import,
        usage: "radar <add|list|import|analyze|suggest|scrape> ...",
        summary: "Store and analyze platform trend samples",
        notes: &[
            "radar add --platform <name> --keyword <text> --title <text> [--url <url>] [--author <name>] [--likes/--collects/--comments <n>] [--source <text>]",
            "radar list [--platform <name>] [--keyword <text>]",
            "radar import <file.csv> [--platform <name>]",
            "radar analyze <article.md> --platform <name> [--top <n>]",
            "radar suggest <article.md> --platform <name> [--top <n>]",
            "radar scrape --platform <name> --keyword <text> [--count <n>] [--url <url>]",
        ],
    },
    CommandSpec {
        name: "capabilities",
        group: Group::Integrate,
        usage: "capabilities",
        summary: "List built-in targets with risk and capability metadata",
        notes: &[],
    },
    CommandSpec {
        name: "workflow-registry",
        group: Group::Integrate,
        usage: "workflow-registry",
        summary: "List workflow contracts for plugins, apps and agents",
        notes: &[],
    },
    CommandSpec {
        name: "wechat-checklist",
        group: Group::Integrate,
        usage: "wechat-checklist",
        summary: "Read-only checklist for positioning, title and read-through",
        notes: &[],
    },
    CommandSpec {
        name: "evidence-status",
        group: Group::Integrate,
        usage: "evidence-status [--strict]",
        summary: "Check required release evidence files",
        notes: &[
            "Only checks that files exist; it does not open or inspect them.",
            "--strict exits non-zero when anything is missing.",
        ],
    },
    CommandSpec {
        name: "release-check",
        group: Group::Integrate,
        usage: "release-check [--strict]",
        summary: "Aggregate release gate status from docs and evidence",
        notes: &["Read-only: no WeChat API, no browser."],
    },
];

/// One global option. Kept as data so the text block and `--json help` cannot
/// drift apart the way the old repeated usage prefix did.
struct GlobalOption {
    flag: &'static str,
    value: Option<&'static str>,
    summary: &'static str,
}

const GLOBAL_OPTIONS_SPEC: &[GlobalOption] = &[
    GlobalOption {
        flag: "--articles",
        value: Some("<path>"),
        summary: "Workspace root that holds Articles/ (default: current dir)",
    },
    GlobalOption {
        flag: "--config",
        value: Some("<path>"),
        summary: "Config file (default: <root>/moonpub.toml)",
    },
    GlobalOption {
        flag: "--json",
        value: None,
        summary: "Machine-readable output",
    },
];

fn global_options_text() -> String {
    let rendered: Vec<(String, &'static str)> = GLOBAL_OPTIONS_SPEC
        .iter()
        .map(|option| {
            let mut flag = option.flag.to_owned();
            if let Some(value) = option.value {
                flag.push(' ');
                flag.push_str(value);
            }
            (flag, option.summary)
        })
        .collect();
    let width = rendered
        .iter()
        .map(|(flag, _)| flag.len())
        .max()
        .unwrap_or(0)
        + 4;
    let mut out = String::from("Global options (before the command):\n");
    for (flag, summary) in rendered {
        out.push_str(&format!("  {flag:<width$}{summary}\n"));
    }
    out.trim_end().to_owned()
}

/// Machine-readable command list for `moonpub --json help`.
///
/// The Obsidian plugin and other callers used to receive the whole help text
/// inside `{"output": "..."}` and had to parse prose to find command names.
///
/// Field order is declaration order (structs, not `json!`): the project's other
/// JSON payloads all lead with `schema_version`, and `json!` would sort the keys
/// alphabetically instead.
pub fn overview_json() -> String {
    let payload = HelpOverview {
        schema_version: HELP_SCHEMA_VERSION,
        moonpub_version: env!("CARGO_PKG_VERSION"),
        global_options: GLOBAL_OPTIONS_SPEC
            .iter()
            .map(GlobalOptionJson::from)
            .collect(),
        groups: Group::ALL
            .iter()
            .map(|group| GroupJson {
                id: group.id(),
                title: group.title(),
            })
            .collect(),
        commands: COMMANDS.iter().map(CommandJson::from).collect(),
    };
    serde_json::to_string(&payload).expect("help payload is serializable")
}

/// Machine-readable page for one command, or `None` when the name is unknown.
pub fn for_command_json(name: &str) -> Option<String> {
    let spec = find(name)?;
    let payload = HelpCommandPage {
        schema_version: HELP_SCHEMA_VERSION,
        moonpub_version: env!("CARGO_PKG_VERSION"),
        command: CommandJson::from(spec),
    };
    Some(serde_json::to_string(&payload).expect("help payload is serializable"))
}

const HELP_SCHEMA_VERSION: &str = "help/v1";

#[derive(serde::Serialize)]
struct HelpOverview {
    schema_version: &'static str,
    moonpub_version: &'static str,
    global_options: Vec<GlobalOptionJson>,
    groups: Vec<GroupJson>,
    commands: Vec<CommandJson>,
}

#[derive(serde::Serialize)]
struct HelpCommandPage {
    schema_version: &'static str,
    moonpub_version: &'static str,
    command: CommandJson,
}

#[derive(serde::Serialize)]
struct GlobalOptionJson {
    flag: &'static str,
    value: Option<&'static str>,
    summary: &'static str,
}

impl From<&GlobalOption> for GlobalOptionJson {
    fn from(option: &GlobalOption) -> Self {
        Self {
            flag: option.flag,
            value: option.value,
            summary: option.summary,
        }
    }
}

#[derive(serde::Serialize)]
struct GroupJson {
    id: &'static str,
    title: &'static str,
}

#[derive(serde::Serialize)]
struct CommandJson {
    name: &'static str,
    group: &'static str,
    usage: &'static str,
    summary: &'static str,
    notes: &'static [&'static str],
}

impl From<&CommandSpec> for CommandJson {
    fn from(spec: &CommandSpec) -> Self {
        Self {
            name: spec.name,
            group: spec.group.id(),
            usage: spec.usage,
            summary: spec.summary,
            notes: spec.notes,
        }
    }
}

/// Full command list, grouped by the task a user is trying to finish.
pub fn overview() -> String {
    let mut out = String::new();
    out.push_str(&format!("MoonPub CLI {}\n", env!("CARGO_PKG_VERSION")));
    out.push_str("Markdown in, WeChat-ready draft out. Local-first, manual publish.\n\n");
    out.push_str("Usage:\n");
    out.push_str("  moonpub <command> [args]\n");
    out.push_str("  moonpub help <command>\n\n");
    out.push_str(&global_options_text());
    out.push_str("\n\nFirst run:\n  moonpub init --yes && moonpub doctor\n");

    for group in Group::ALL {
        out.push('\n');
        out.push_str(group.title());
        out.push('\n');
        for spec in COMMANDS.iter().filter(|spec| spec.group == group) {
            out.push_str(&format!(
                "  {:<width$}{}\n",
                spec.name,
                spec.summary,
                width = NAME_WIDTH
            ));
        }
    }

    out
}

/// Help page for one command, or `None` when the name is not a command.
pub fn for_command(name: &str) -> Option<String> {
    let spec = find(name)?;
    let mut out = String::new();
    out.push_str(&format!("moonpub {}\n\n", spec.name));
    out.push_str(&format!("  {}\n\n", spec.summary));
    out.push_str("Usage:\n");
    out.push_str(&format!("  moonpub {}\n", spec.usage));
    if !spec.notes.is_empty() {
        out.push_str("\nNotes:\n");
        for note in spec.notes {
            out.push_str(&format!("  - {note}\n"));
        }
    }
    out.push('\n');
    out.push_str(&global_options_text());
    out.push('\n');
    out.push_str(&format!(
        "\nGroup: {} — run `moonpub help` for the full list.\n",
        spec.group.title()
    ));
    Some(out)
}

pub fn is_command(name: &str) -> bool {
    find(name).is_some()
}

fn find(name: &str) -> Option<&'static CommandSpec> {
    COMMANDS.iter().find(|spec| spec.name == name)
}

/// Closest command name for a typo, when one is close enough to be useful.
pub fn suggest(name: &str) -> Option<&'static str> {
    let lowered = name.to_ascii_lowercase();
    // Every name "starts with" the empty string, so guard short inputs first.
    if lowered.len() < 2 {
        return None;
    }
    let mut best: Option<(usize, &'static str)> = None;
    for spec in COMMANDS {
        let distance = edit_distance(&lowered, spec.name);
        let close_enough = distance <= 2 || (lowered.len() >= 3 && spec.name.starts_with(&lowered));
        if close_enough && best.is_none_or(|(best_distance, _)| distance < best_distance) {
            best = Some((distance, spec.name));
        }
    }
    best.map(|(_, name)| name)
}

/// Optimal-string-alignment-lite: plain Levenshtein, small inputs only.
fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0usize; b.len() + 1];
    for (i, ca) in a.iter().enumerate() {
        cur[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let substitution = prev[j] + usize::from(ca != cb);
            cur[j + 1] = substitution.min(prev[j + 1] + 1).min(cur[j] + 1);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_command_has_usage_and_summary() {
        for spec in COMMANDS {
            assert!(!spec.name.is_empty());
            assert!(!spec.usage.is_empty(), "{} has no usage", spec.name);
            assert!(!spec.summary.is_empty(), "{} has no summary", spec.name);
            assert!(
                !spec.summary.ends_with('.'),
                "{} summary should not end with a period",
                spec.name
            );
        }
    }

    #[test]
    fn command_names_are_unique() {
        let mut seen = HashSet::new();
        for spec in COMMANDS {
            assert!(seen.insert(spec.name), "duplicate command: {}", spec.name);
        }
    }

    #[test]
    fn overview_lists_every_group_and_command() {
        let text = overview();
        for group in Group::ALL {
            assert!(
                text.contains(group.title()),
                "missing group {}",
                group.title()
            );
        }
        for spec in COMMANDS {
            assert!(text.contains(spec.name), "missing command {}", spec.name);
        }
    }

    /// The old help repeated the global-option prefix on every usage line.
    #[test]
    fn overview_states_global_options_once() {
        let text = overview();
        assert_eq!(text.matches("[--articles <path>]").count(), 0);
        assert_eq!(text.matches("--articles <path>").count(), 1);
    }

    #[test]
    fn overview_fits_on_one_screen() {
        let lines = overview().lines().count();
        assert!(lines < 75, "overview grew to {lines} lines");
    }

    #[test]
    fn command_page_shows_usage_and_group() {
        let page = for_command("render").expect("render is a command");
        assert!(page.contains("moonpub render <article.md>"));
        assert!(page.contains("Render & review"));
    }

    #[test]
    fn command_page_carries_notes_for_commands_that_need_them() {
        let page = for_command("push").expect("push is a command");
        assert!(page.contains("auth_method"));
        assert!(page.contains("--temporary-profile"));
    }

    /// Regression guard: the radar page used to document only `add` and
    /// `analyze`, so `list` / `import` / `suggest` / `scrape` were invisible
    /// to anyone reading `moonpub help radar`.
    #[test]
    fn radar_help_page_lists_every_subcommand() {
        let page = for_command("radar").expect("radar is a command");
        for sub in ["add", "list", "import", "analyze", "suggest", "scrape"] {
            assert!(
                page.contains(&format!("radar {sub}")),
                "help radar omits `radar {sub}`"
            );
        }
    }

    /// The human-facing command lists (both READMEs and docs/COMMANDS*.md) used
    /// to drift from the registry — both READMEs missed 7 registered commands.
    /// Every registered name must appear in each list.
    #[test]
    fn human_facing_command_lists_cover_every_command() {
        const README: &str = include_str!("../README.md");
        const README_ZH: &str = include_str!("../README_zh.md");
        const COMMANDS_DOC: &str = include_str!("../docs/COMMANDS.md");
        const COMMANDS_DOC_ZH: &str = include_str!("../docs/COMMANDS_ZH.md");
        for spec in COMMANDS {
            let needle = format!("moonpub {}", spec.name);
            for (label, text) in [
                ("README.md", README),
                ("README_zh.md", README_ZH),
                ("docs/COMMANDS.md", COMMANDS_DOC),
                ("docs/COMMANDS_ZH.md", COMMANDS_DOC_ZH),
            ] {
                assert!(
                    text.contains(&needle),
                    "{label} is missing `moonpub {}`",
                    spec.name
                );
            }
        }
    }

    /// When a usage line ends in `...` it punts per-variant details to the
    /// notes, so every `<a|b|c>` variant must be named there. Without this,
    /// adding a seventh `radar` subcommand would again leave it undocumented.
    #[test]
    fn usage_variants_that_punt_details_are_all_in_notes() {
        for spec in COMMANDS {
            if !spec.usage.ends_with("...") {
                continue;
            }
            let Some((_, tail)) = spec.usage.split_once('<') else {
                continue;
            };
            let Some((variants, _)) = tail.split_once('>') else {
                continue;
            };
            if !variants.contains('|') {
                continue;
            }
            let notes_tokens: HashSet<&str> = spec
                .notes
                .iter()
                .flat_map(|note| note.split(|c: char| !c.is_alphanumeric()))
                .collect();
            for variant in variants.split('|') {
                assert!(
                    notes_tokens.contains(variant),
                    "{}: variant `{variant}` missing from notes",
                    spec.name
                );
            }
        }
    }

    #[test]
    fn unknown_command_has_no_page() {
        assert!(for_command("rendr").is_none());
        assert!(!is_command("rendr"));
        assert!(is_command("render"));
    }

    #[test]
    fn suggest_finds_close_typos() {
        assert_eq!(suggest("rendr"), Some("render"));
        assert_eq!(suggest("prefligth"), Some("preflight"));
        assert_eq!(suggest("pus"), Some("push"));
    }

    #[test]
    fn suggest_stays_quiet_for_unrelated_words() {
        assert_eq!(suggest("banana"), None);
        assert_eq!(suggest(""), None);
    }

    #[test]
    fn edit_distance_matches_known_values() {
        assert_eq!(edit_distance("", ""), 0);
        assert_eq!(edit_distance("abc", "abc"), 0);
        assert_eq!(edit_distance("cat", "cot"), 1);
        assert_eq!(edit_distance("abc", ""), 3);
    }

    #[test]
    fn overview_json_lists_every_command_once() -> Result<(), Box<dyn std::error::Error>> {
        let payload: serde_json::Value = serde_json::from_str(&overview_json())?;
        assert_eq!(payload["schema_version"], "help/v1");
        assert_eq!(payload["moonpub_version"], env!("CARGO_PKG_VERSION"));

        let commands = payload["commands"]
            .as_array()
            .ok_or("commands should be an array")?;
        assert_eq!(commands.len(), COMMANDS.len());

        let mut seen = HashSet::new();
        for command in commands {
            let name = command["name"].as_str().ok_or("name should be a string")?;
            assert!(seen.insert(name), "duplicate command in json: {name}");
            assert!(
                command["usage"]
                    .as_str()
                    .is_some_and(|usage| !usage.is_empty()),
                "{name} has no usage"
            );
            assert!(
                command["notes"].is_array(),
                "{name} notes should be an array, even when empty"
            );
        }
        Ok(())
    }

    #[test]
    fn overview_json_group_ids_cover_every_group() -> Result<(), Box<dyn std::error::Error>> {
        let payload: serde_json::Value = serde_json::from_str(&overview_json())?;
        let groups = payload["groups"]
            .as_array()
            .ok_or("groups should be an array")?;
        let ids: Vec<&str> = groups
            .iter()
            .filter_map(|group| group["id"].as_str())
            .collect();
        assert_eq!(
            ids,
            ["start", "write", "render", "publish", "import", "integrate"]
        );
        // 每个命令的 group 必须是已声明的 id 之一，否则消费方按 id 分组会漏。
        let commands = payload["commands"]
            .as_array()
            .ok_or("commands should be an array")?;
        for command in commands {
            let group = command["group"]
                .as_str()
                .ok_or("group should be a string")?;
            assert!(ids.contains(&group), "undeclared group id: {group}");
        }
        Ok(())
    }

    #[test]
    fn overview_json_global_options_match_the_text_block() -> Result<(), Box<dyn std::error::Error>>
    {
        let payload: serde_json::Value = serde_json::from_str(&overview_json())?;
        let options = payload["global_options"]
            .as_array()
            .ok_or("global_options should be an array")?;
        assert_eq!(options.len(), GLOBAL_OPTIONS_SPEC.len());

        let text = overview();
        for (option, spec) in options.iter().zip(GLOBAL_OPTIONS_SPEC) {
            assert_eq!(option["flag"], spec.flag);
            assert_eq!(option["summary"], spec.summary);
            assert_eq!(
                text.matches(spec.flag).count(),
                1,
                "{} should appear once in the text overview",
                spec.flag
            );
        }
        Ok(())
    }

    #[test]
    fn command_json_page_matches_the_registry() -> Result<(), Box<dyn std::error::Error>> {
        let payload: serde_json::Value =
            serde_json::from_str(&for_command_json("cover").ok_or("cover is a command")?)?;
        assert_eq!(payload["schema_version"], "help/v1");
        assert_eq!(payload["command"]["name"], "cover");
        assert_eq!(payload["command"]["group"], "render");
        assert!(
            payload["command"]["usage"]
                .as_str()
                .is_some_and(|usage| usage.starts_with("cover <article.md>"))
        );
        Ok(())
    }

    #[test]
    fn command_json_page_rejects_unknown_names() {
        assert!(for_command_json("rendr").is_none());
        assert!(for_command_json("").is_none());
    }
}
