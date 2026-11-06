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

    #[error(
        "unknown option: {0}\n\nRun `moonpub help <command>` to see the options that command accepts."
    )]
    UnknownOption(String),

    #[error("unknown command: {0}{hint}", hint = unknown_command_hint(.0))]
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

    #[error("photo vision requires the OpenAI provider; set [ai] provider = \"openai\"")]
    PhotoVisionProviderUnsupported,

    #[error("photo vision input invalid: {0}")]
    PhotoVisionInput(String),

    #[error("push failed: {message}{hint}", hint = ip_hint.as_ref().map(|ip| format!("\n  current IP: {ip} — add it to WeChat IP allowlist\n  if this IP keeps changing, disable rotating proxies or use a stable egress IP before allowlisting")).unwrap_or_default())]
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

    #[error(
        "WeChat cookie session not ready: {message}\n  run 'moonpub login' to scan the QR code, then retry push (cookie mode bypasses the IP whitelist)"
    )]
    CookieSessionRequired { message: String },

    #[error(
        "release evidence incomplete: {missing_count} required file(s) missing\n  run '{next_command}' to list missing files\n  add redacted screenshots before preparing the release"
    )]
    EvidenceMissing {
        missing_count: usize,
        next_command: &'static str,
    },

    #[error("release gate incomplete: {failed_count} check(s) failed\n  next: {next_command}")]
    ReleaseGateIncomplete {
        failed_count: usize,
        next_command: String,
    },

    #[error("AI image generation requires the OpenAI provider; set [ai] provider = \"openai\"")]
    ImageGenerationProviderUnsupported,

    #[error("AI image generation failed: {0}")]
    ImageGenerationFailed(String),
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

/// Full command list shown by `moonpub help` and by `moonpub` with no args.
///
/// The content lives in [`crate::help`] so the overview, per-command pages and
/// the typo hint all come from one command registry.
pub fn help_text() -> String {
    crate::help::overview()
}

/// Appended to "unknown command" so a typo points at the intended command
/// instead of dumping the whole command list.
fn unknown_command_hint(name: &str) -> String {
    match crate::help::suggest(name) {
        Some(candidate) => format!(
            "\n\nDid you mean `{candidate}`? Run `moonpub help {candidate}` for details."
        ),
        None => "\n\nRun `moonpub help` for the command list, or `moonpub help <command>` for one command."
            .to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;

    #[test]
    fn extract_ip_from_message_basic() {
        assert_eq!(
            extract_ip_from_message("push failed: invalid ip 1.2.3.4 blocked"),
            Some("1.2.3.4".to_string())
        );
    }

    #[test]
    fn extract_ip_from_message_none_when_absent() {
        assert_eq!(extract_ip_from_message("push failed: unknown error"), None);
        assert_eq!(extract_ip_from_message("invalid ip "), None);
    }

    #[test]
    fn missing_command_error_includes_help() {
        let msg = AppError::MissingCommand.to_string();
        assert!(msg.to_lowercase().contains("missing command"));
        assert!(msg.contains("Usage:"));
    }

    #[test]
    fn unknown_command_suggests_the_closest_command() {
        let msg = AppError::UnknownCommand("rendr".to_string()).to_string();
        assert!(msg.contains("unknown command: rendr"));
        assert!(msg.contains("Did you mean `render`?"));
    }

    /// A typo used to dump the whole ~90 line command list into the error.
    #[test]
    fn unknown_command_does_not_dump_the_full_list() {
        let msg = AppError::UnknownCommand("banana".to_string()).to_string();
        assert!(!msg.contains("Publish & WeChat backend"));
        assert!(msg.contains("moonpub help"));
        assert!(msg.lines().count() <= 4, "error grew to:\n{msg}");
    }

    #[test]
    fn push_failed_includes_ip_hint_when_present() {
        let err = AppError::PushFailed {
            message: "boom".to_string(),
            ip_hint: Some("1.2.3.4".to_string()),
        };
        let msg = err.to_string();
        assert!(msg.contains("push failed: boom"));
        assert!(msg.contains("current IP: 1.2.3.4"));
    }

    #[test]
    fn push_failed_omits_ip_hint_when_absent() {
        let err = AppError::PushFailed {
            message: "boom".to_string(),
            ip_hint: None,
        };
        let msg = err.to_string();
        assert!(msg.contains("push failed: boom"));
        assert!(!msg.contains("current IP"));
    }

    #[test]
    fn draft_and_article_path_errors_reference_paths() {
        assert!(
            AppError::NoDraftJson(PathBuf::from("/a/b.md"))
                .to_string()
                .contains("/a/b.md")
        );
        let p = AppError::InvalidArticlePath(PathBuf::from("note.txt"));
        let msg = p.to_string();
        assert!(msg.contains("note.txt"));
        assert!(msg.contains(".md file"));
    }

    #[test]
    fn io_error_surfaces_path_and_source() {
        let err = AppError::Io {
            path: PathBuf::from("/x/y.md"),
            source: io::Error::new(io::ErrorKind::NotFound, "no such file"),
        };
        let msg = err.to_string();
        assert!(msg.contains("/x/y.md"));
    }
}
