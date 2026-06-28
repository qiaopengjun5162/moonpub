use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("missing command\n\n{help}", help = help_text())]
    MissingCommand,

    #[error("missing value for {0}")]
    MissingValue(&'static str),

    #[error("missing value for {0}")]
    MissingValueString(String),

    #[error("unknown option: {0}")]
    UnknownOption(String),

    #[error("unknown command: {0}\n\n{help}", help = help_text())]
    UnknownCommand(String),

    #[error("{}: {source}", .path.display())]
    Io { path: PathBuf, source: io::Error },

    #[error("article path must point to a .md file: {}", .0.display())]
    InvalidArticlePath(PathBuf),

    #[error("config already exists: {}", .0.display())]
    ConfigExists(PathBuf),

    #[error("invalid number for {flag}: {value}")]
    InvalidNumber { flag: &'static str, value: String },

    #[error("invalid csv: {0}")]
    InvalidCsv(String),

    #[error("missing env var: {0}")]
    MissingEnvVar(&'static str),

    #[error("push failed: {message}{hint}", hint = ip_hint.as_ref().map(|ip| format!("\n  current IP: {ip} — add it to WeChat IP allowlist")).unwrap_or_default())]
    PushFailed {
        message: String,
        ip_hint: Option<String>,
    },

    #[error("draft.json not found: {}\n  run 'moonpub render' first", .0.display())]
    NoDraftJson(PathBuf),

    #[error("html not found: {}\n  run 'moonpub render' first", .0.display())]
    NoHtml(PathBuf),

    #[error("browser automation failed: {message}")]
    AutomationFailed { message: String },
}

/// Try to pull the current IP from a WeChat error message like "invalid ip 1.2.3.4".
pub fn extract_ip_from_message(msg: &str) -> Option<String> {
    let marker = "invalid ip ";
    let start = msg.find(marker)? + marker.len();
    let ip: String = msg[start..]
        .chars()
        .take_while(|c| c.is_ascii_digit() || *c == '.')
        .collect();
    if ip.is_empty() { None } else { Some(ip) }
}

pub fn help_text() -> String {
    String::from(
        r#"MoonPub CLI

Usage:
  moonpub --version
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] write <idea>
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] draft-from-inbox <inbox.md> [--preview] [--no-open]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] expand <article.md>
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] polish <article.md>
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] new <title>
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] init [moonpub.toml]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] status
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] capabilities [--json]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] check <article.md>
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] render <article.md> [--author <name>] [--thumb <media_id>]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] push <article.md> [--render]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] publish <article.md> --target wechat-draft [--render]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] update-draft <article.md> [--media-id <id>]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] export <article.md> [--target zola]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] preview <article.md>
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] mark-ready <article.md>
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] mark-published <article.md>
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] humanize <article.md>
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] login
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] configure [<step>..] [--headed]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] step-test [--headed]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] test-zanshang [--headed]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] test-chuangzuo [--headed]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] test-yulan [--headed]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] fetch <url>
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] intake feishu <file> [--draft] [--preview] [--no-open]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] intake feishu --minute-token <token> [--draft] [--preview] [--no-open]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] intake feishu --latest [--draft] [--preview] [--no-open]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] intake feishu --query <keyword> [--draft] [--preview] [--no-open]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] cover <article.md> [--style dark|clean|minimal|warm|serif|gradient|literary|ink|sunset|forest] [--screenshot]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] ship <article.md> [--style dark|literary|ink|sunset|forest|...] [--ai]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] radar add --platform <name> --keyword <text> --title <text> [--url <url>] [--likes <n>] [--collects <n>] [--comments <n>]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] radar list [--platform <name>] [--keyword <text>]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] radar import <file.csv> [--platform <name>]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] radar analyze <article.md> --platform <name> [--top <n>]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] radar suggest <article.md> --platform <name> [--top <n>]
  moonpub [--articles <path>] [--config <moonpub.toml>] [--json] radar scrape --platform <name> --keyword <text> [--count <n>] [--url <url>]

Commands:
  version      Print the moonpub version
  write        Generate article from an idea (requires DEEPSEEK_API_KEY)
  draft-from-inbox Generate an editable draft from Inbox source material (requires AI API key)
  expand       Expand reading notes into a full article (requires DEEPSEEK_API_KEY)
  polish       AI polish + de-AI-ify existing article (requires DEEPSEEK_API_KEY)
  new          Scaffold a new article with frontmatter template
  init         Create a sample moonpub.toml
  status       List article files in Articles/drafts, ready, and published
  capabilities List built-in targets and risk/capability metadata
  check        Check whether an article bundle has md/html/draft.json files
  render       Generate <slug>.html and <slug>.draft.json from a Markdown article
  push         Push draft to WeChat (direct API), write .media_id, move bundle to ready/
  publish      Generic publish target entrypoint (currently: --target wechat-draft)
  update-draft Re-push updated HTML to an existing WeChat draft by media_id
  export       Generic export target entrypoint (currently: --target zola)
  preview      Open the rendered HTML in the system browser
  humanize     Strip AI patterns from article (offline, no API key needed)
  login        One-time WeChat backend login (opens browser for QR scan)
  configure    Auto-configure WeChat draft settings (headless by default, --headed to debug)
  step-test    Interactive browser automation test (--headed to see browser)
  test-zanshang Test reward step only (--headed to see browser)
  test-chuangzuo Test creation source step only (--headed to see browser)
  test-yulan   Test preview step only (--headed to see browser)
  list-drafts  List all drafts (shows media_id + title)
  delete-draft Delete a draft by media_id  (delete-draft <media_id>)
  fetch        Fetch a WeChat article and extract title + body (requires Chrome)
  intake      Import upstream source material into Obsidian Inbox (currently: feishu)
  cover        Generate a cover HTML file from article frontmatter
  ship         Cover + render + push + configure + export; final publish stays manual
  radar        Store and analyze platform trend samples (add/list/import/analyze/suggest/scrape)
"#,
    )
}
