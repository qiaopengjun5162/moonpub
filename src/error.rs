use std::io;
use std::path::PathBuf;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(
        "missing command — first time? run `moonpub init --yes && moonpub doctor`\n\n{help}",
        help = help_text()
    )]
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

    #[error(
        "invalid theme name: {0}\n  run 'moonpub help render' or check [wechat] theme in moonpub.toml for valid names"
    )]
    InvalidThemeName(String),

    #[error("invalid csv: {0}")]
    InvalidCsv(String),

    /// 参数本身不成立，不是通道问题。
    ///
    /// 以前 `delete-draft` 收到非数字 id 时报 `push_failed`：`first_check`
    /// 让人跑 `wechat-health`（查凭据 / IP 白名单 / 登录态），`next_step`
    /// 也是 `moonpub wechat-health`——用户只是把媒体素材 id 当成了草稿 id，
    /// 却被送去诊断一个完全健康的通道。
    #[error(
        "invalid media_id: `{media_id}`\n  delete-draft takes the numeric AppMsgId printed by `moonpub push` (media_id: …) or `moonpub list-drafts`\n  {reason}"
    )]
    InvalidMediaId {
        media_id: String,
        reason: &'static str,
    },

    #[error("missing env var: {0}{hint}", hint = missing_env_var_hint(.0))]
    MissingEnvVar(&'static str),

    #[error(
        "cookie mode does not cover `{command}`\n  cookie mode covers: {covered}\n  {alternative}",
        covered = cookie_commands_text()
    )]
    CookieAuthUnsupported {
        command: &'static str,
        alternative: String,
        /// Machine-readable next step; kept out of the human text on purpose —
        /// the human already gets a fully spelled-out command in `alternative`.
        next_command: String,
    },

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
        "WeChat cookie session not ready: {message}\n  run 'moonpub login' to scan the QR code, then: {retry}"
    )]
    CookieSessionRequired { message: String, retry: String },

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

    #[error(
        "preflight gate failed: {count} check(s) failed{details}\n  next: {next_command}",
        count = .failed_checks.len(),
        details = preflight_failure_details(.failed_checks)
    )]
    PreflightGateFailed {
        failed_checks: Vec<String>,
        next_command: String,
    },

    #[error("AI image generation requires the OpenAI provider; set [ai] provider = \"openai\"")]
    ImageGenerationProviderUnsupported,

    #[error("AI image generation failed: {0}")]
    ImageGenerationFailed(String),
}

/// `push` 之前的整条链（`init` / `new` / `render` / `preview`）都不碰微信后台，
/// 所以缺凭据常常是新手撞上的第一堵墙，而它过去只有一句「缺了什么」：
/// 既不说去哪拿 AppSecret，也不提还有一条不需要 AppSecret 的 cookie 通道。
/// 报错必须当场给出两条能直接照抄的路，否则人就停在这儿了。
///
/// cookie 那句必须写明覆盖范围：`list-drafts` / `update-draft` 只有 appsecret
/// 实现，把它们一起含糊过去，等于把用户送回同一个错误（历史版本在
/// cookie 模式下跑 `list-drafts` 就是「缺 WECHAT_SECRET」的无限循环）。
fn missing_env_var_hint(name: &str) -> String {
    if !matches!(name, "WECHAT_APPID" | "WECHAT_SECRET") {
        return " — set it in your shell or .env".to_owned();
    }
    // 这里不能出现具体命令名。以前写死了 `moonpub push <article.md>`，于是
    // `delete-draft` 缺凭据时也在教人跑 `push`——照着做只会多出一篇草稿。
    // 改法：cookie 通道的开关写进 `.env`（启动时会被读成环境变量），
    // 然后重跑「刚才那条命令」，任何命令都适用。
    format!(
        "\n  two ways forward:\n    \
         cookie mode (no AppSecret, covers {}): run `moonpub login` once, add `WECHAT_AUTH_METHOD=cookie` to .env, then run this command again\n    \
         app credentials: WeChat backend -> Development -> Basic Configuration, then add to .env:\n      \
         WECHAT_APPID=wx...\n      WECHAT_SECRET=...",
        cookie_commands_text()
    )
}

fn cookie_commands_text() -> String {
    crate::config::COOKIE_AUTH_COMMANDS.join(" / ")
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

/// `--strict` turns preflight into a process-level gate, so the failure has to
/// carry the same per-check detail as the report that would have gone to stdout.
/// Without this, a CI job only sees "1 check failed" and must re-run the command
/// to learn which check broke.
fn preflight_failure_details(checks: &[String]) -> String {
    checks.iter().fold(String::new(), |mut out, check| {
        out.push_str("\n  - ");
        out.push_str(check);
        out
    })
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
        // 小白敲 `moonpub`（不带参数）看到的第一行不能只是一句报错，
        // 必须当场给一条能跑的命令。
        let first_line = msg.lines().next().unwrap_or_default();
        assert!(first_line.contains("moonpub init --yes"), "{first_line}");
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
    fn missing_env_var_names_the_var_and_both_ways_forward() {
        let msg = AppError::MissingEnvVar("WECHAT_SECRET").to_string();
        // 第一行仍是事实（缺哪个变量），但紧跟其后必须是可照抄的两条路。
        assert_eq!(
            msg.lines().next().unwrap_or_default(),
            "missing env var: WECHAT_SECRET"
        );
        assert!(msg.contains("`moonpub login`"), "{msg}");
        assert!(msg.contains("WECHAT_AUTH_METHOD=cookie"), "{msg}");
        assert!(msg.contains("WECHAT_APPID=wx..."), "{msg}");
        assert!(msg.contains("WECHAT_SECRET=..."), "{msg}");
    }

    /// 提示不能点名某个具体命令：`delete-draft` 缺凭据时若还在教人跑
    /// `push`，照着做只会多出一篇草稿，草稿并没有被删掉。
    #[test]
    fn missing_env_var_hint_names_no_specific_command() {
        let msg = AppError::MissingEnvVar("WECHAT_SECRET").to_string();
        for command in ["moonpub push", "moonpub ship", "moonpub delete-draft"] {
            assert!(
                !msg.contains(command),
                "the hint must stay command-agnostic, but names `{command}`:\n{msg}"
            );
        }
        assert!(msg.contains("run this command again"), "{msg}");
    }

    /// 提示说 cookie 覆盖哪些命令，而这个范围必须只有一个来源：
    /// 写死在文案里就会与 `COOKIE_AUTH_COMMANDS` 漂移，然后重新把人引到
    /// 一条走不通的路上（`list-drafts` 就是这么暴露出来的）。
    #[test]
    fn missing_env_var_hint_states_the_cookie_coverage() {
        let msg = AppError::MissingEnvVar("WECHAT_SECRET").to_string();
        let covered = crate::config::COOKIE_AUTH_COMMANDS.join(" / ");
        assert!(msg.contains(&format!("covers {covered}")), "{msg}");
    }

    /// 参数不成立时不能报成通道失败：`push_failed` 的 first_check 让人去查
    /// 凭据 / IP 白名单 / 登录态，而这里的问题只是「你给的不是草稿 id」。
    #[test]
    fn invalid_media_id_names_the_bad_id_and_the_real_source() {
        let err = AppError::InvalidMediaId {
            media_id: "R1LkThumb".to_owned(),
            reason: "cookie sessions only delete numeric AppMsgIds",
        };
        let msg = err.to_string();
        assert_eq!(msg.lines().next().unwrap(), "invalid media_id: `R1LkThumb`");
        assert!(!msg.starts_with("push failed"), "{msg}");
        assert!(msg.contains("`moonpub list-drafts`"), "{msg}");
        assert!(msg.contains("media_id:"), "{msg}");
        assert!(
            msg.lines().count() <= 3,
            "a bad argument needs a short message:\n{msg}"
        );
    }

    /// cookie 会话没就绪时，重试的必须是**刚才那条命令**。模板里曾经写死
    /// 「retry push」，于是 `delete-draft` / `ship` 的报错都在教人跑 `push`。
    #[test]
    fn cookie_session_error_retries_the_command_that_failed() {
        let err = AppError::CookieSessionRequired {
            message: "session.json 不存在，cookie 模式无法删除草稿".to_owned(),
            retry: "moonpub delete-draft 2247483648".to_owned(),
        };
        let msg = err.to_string();
        assert!(
            msg.contains("then: moonpub delete-draft 2247483648"),
            "{msg}"
        );
        assert!(!msg.contains("retry push"), "{msg}");
        assert!(!msg.contains("moonpub push"), "{msg}");
    }

    #[test]
    fn cookie_auth_unsupported_names_the_command_and_the_way_out() {
        let err = AppError::CookieAuthUnsupported {
            command: "update-draft",
            alternative: "re-push instead: moonpub push a.md --render".to_owned(),
            next_command: "moonpub push a.md --render".to_owned(),
        };
        let msg = err.to_string();
        assert!(
            msg.starts_with("cookie mode does not cover `update-draft`"),
            "{msg}"
        );
        assert!(
            msg.contains("cookie mode covers: push / ship / delete-draft"),
            "{msg}"
        );
        assert!(msg.contains("moonpub push a.md --render"), "{msg}");
    }

    #[test]
    fn other_missing_env_vars_do_not_get_wechat_instructions() {
        let msg = AppError::MissingEnvVar("SOME_OTHER_TOKEN").to_string();
        assert!(msg.contains("SOME_OTHER_TOKEN"));
        assert!(!msg.contains("WECHAT"), "{msg}");
        assert_eq!(msg.lines().count(), 1, "{msg}");
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
