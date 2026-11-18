//! Machine-readable failure payload (`error/v1`).
//!
//! Text-mode failures are unchanged: the human message stays on stderr, the
//! process exits non-zero, and stdout stays empty. When the caller explicitly
//! asked for `--json`, stdout additionally carries this object so an agent can
//! branch on a stable `code` instead of parsing prose from stderr.

use serde::Serialize;

use crate::error::AppError;

/// Stable `error/v1` discriminator.
///
/// `as_str()` output is part of the wire contract: rename it only with a schema
/// version bump. `ALL` is the single source of truth advertised through
/// `help/v1`, so an agent can enumerate every code at runtime instead of
/// guessing from a prose list in the docs. Adding a variant forces all four
/// `match` arms below to be updated.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum ErrorCode {
    MissingCommand,
    MissingValue,
    UnknownOption,
    UnknownCommand,
    IoError,
    InvalidArticlePath,
    ConfigExists,
    InvalidNumber,
    InvalidThemeName,
    InvalidCsv,
    MissingEnvVar,
    PhotoVisionProviderUnsupported,
    PhotoVisionInputInvalid,
    PushFailed,
    NoDraftJson,
    NoHtml,
    AutomationFailed,
    CookieSessionRequired,
    EvidenceMissing,
    ReleaseGateIncomplete,
    PreflightGateFailed,
    ImageGenerationProviderUnsupported,
    ImageGenerationFailed,
    InternalError,
}

/// A pointer to the recorded post-mortem for a class of failures.
///
/// Kept in code (not parsed from prose at runtime) so `help errors` can advertise
/// a stable, machine-readable link. `heading` is the exact Chinese heading in
/// `docs/ENGINEERING_LESSONS_ZH.md`; a bridge test verifies it still exists so a
/// renamed lesson cannot silently leave a dangling reference.
#[derive(Clone, Copy, serde::Serialize)]
pub struct LessonRef {
    pub doc: &'static str,
    pub heading: &'static str,
}

impl ErrorCode {
    pub const ALL: [Self; 24] = [
        Self::MissingCommand,
        Self::MissingValue,
        Self::UnknownOption,
        Self::UnknownCommand,
        Self::IoError,
        Self::InvalidArticlePath,
        Self::ConfigExists,
        Self::InvalidNumber,
        Self::InvalidThemeName,
        Self::InvalidCsv,
        Self::MissingEnvVar,
        Self::PhotoVisionProviderUnsupported,
        Self::PhotoVisionInputInvalid,
        Self::PushFailed,
        Self::NoDraftJson,
        Self::NoHtml,
        Self::AutomationFailed,
        Self::CookieSessionRequired,
        Self::EvidenceMissing,
        Self::ReleaseGateIncomplete,
        Self::PreflightGateFailed,
        Self::ImageGenerationProviderUnsupported,
        Self::ImageGenerationFailed,
        Self::InternalError,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Self::MissingCommand => "missing_command",
            Self::MissingValue => "missing_value",
            Self::UnknownOption => "unknown_option",
            Self::UnknownCommand => "unknown_command",
            Self::IoError => "io_error",
            Self::InvalidArticlePath => "invalid_article_path",
            Self::ConfigExists => "config_exists",
            Self::InvalidNumber => "invalid_number",
            Self::InvalidThemeName => "invalid_theme_name",
            Self::InvalidCsv => "invalid_csv",
            Self::MissingEnvVar => "missing_env_var",
            Self::PhotoVisionProviderUnsupported => "photo_vision_provider_unsupported",
            Self::PhotoVisionInputInvalid => "photo_vision_input_invalid",
            Self::PushFailed => "push_failed",
            Self::NoDraftJson => "no_draft_json",
            Self::NoHtml => "no_html",
            Self::AutomationFailed => "automation_failed",
            Self::CookieSessionRequired => "cookie_session_required",
            Self::EvidenceMissing => "evidence_missing",
            Self::ReleaseGateIncomplete => "release_gate_incomplete",
            Self::PreflightGateFailed => "preflight_gate_failed",
            Self::ImageGenerationProviderUnsupported => "image_generation_provider_unsupported",
            Self::ImageGenerationFailed => "image_generation_failed",
            Self::InternalError => "internal_error",
        }
    }

    pub fn summary(self) -> &'static str {
        match self {
            Self::MissingCommand => "No command was given.",
            Self::MissingValue => "A required argument is missing.",
            Self::UnknownOption => "An unrecognized flag was passed.",
            Self::UnknownCommand => "The command name is not in the registry.",
            Self::IoError => "A file or network I/O operation failed.",
            Self::InvalidArticlePath => "The article path does not point to a .md file.",
            Self::ConfigExists => "init would overwrite an existing config.",
            Self::InvalidNumber => "A numeric flag could not be parsed.",
            Self::InvalidThemeName => "A theme name is not in the registered theme list.",
            Self::InvalidCsv => "A CSV argument could not be parsed.",
            Self::MissingEnvVar => "A required environment variable is unset.",
            Self::PhotoVisionProviderUnsupported => "Photo vision needs the OpenAI provider.",
            Self::PhotoVisionInputInvalid => "Photo vision input is malformed.",
            Self::PushFailed => "WeChat rejected the draft push.",
            Self::NoDraftJson => "The article bundle has no <slug>.draft.json.",
            Self::NoHtml => "The article bundle has no <slug>.html.",
            Self::AutomationFailed => "Browser automation did not complete.",
            Self::CookieSessionRequired => "A fresh WeChat login cookie is required.",
            Self::EvidenceMissing => "Required release evidence files are missing.",
            Self::ReleaseGateIncomplete => "Release checks did not all pass.",
            Self::PreflightGateFailed => "preflight --strict found failing checks.",
            Self::ImageGenerationProviderUnsupported => {
                "AI image generation needs the OpenAI provider."
            }
            Self::ImageGenerationFailed => "AI image generation did not complete.",
            Self::InternalError => "An unclassified internal failure.",
        }
    }

    /// Representative `next_step`. Gate failures compute theirs from runtime
    /// data, so this is documentation, not a guarantee for a specific failure.
    pub fn example_next_step(self) -> &'static str {
        match self {
            Self::MissingCommand | Self::UnknownCommand | Self::InternalError => "moonpub help",
            Self::MissingValue
            | Self::UnknownOption
            | Self::InvalidArticlePath
            | Self::InvalidNumber
            | Self::InvalidThemeName
            | Self::InvalidCsv
            | Self::PhotoVisionInputInvalid
            | Self::ImageGenerationFailed => "moonpub help <command>",
            Self::IoError
            | Self::ConfigExists
            | Self::MissingEnvVar
            | Self::PhotoVisionProviderUnsupported
            | Self::ImageGenerationProviderUnsupported => "moonpub doctor",
            Self::PushFailed | Self::AutomationFailed => "moonpub wechat-health",
            Self::NoDraftJson | Self::NoHtml | Self::PreflightGateFailed => {
                "moonpub render <article.md>"
            }
            Self::CookieSessionRequired => "moonpub login",
            Self::EvidenceMissing => "moonpub evidence-status",
            Self::ReleaseGateIncomplete => "moonpub release-check",
        }
    }

    /// First thing to check, following the project's fixed diagnostic order:
    /// evaluation method → environment/config → system itself. This is advisory
    /// only; it never changes `next_step` or the failure's exit code.
    pub fn first_check(self) -> &'static str {
        match self {
            Self::MissingCommand | Self::UnknownCommand => {
                "先确认命令名在 `moonpub help` 注册表里，别先改代码。"
            }
            Self::MissingValue | Self::UnknownOption => {
                "先核对该命令的 `moonpub help <command>` 用法与 flag 顺序。"
            }
            Self::InvalidArticlePath | Self::NoDraftJson | Self::NoHtml => {
                "先确认文章路径与 render 产物存在，再怀疑渲染逻辑。"
            }
            Self::InvalidNumber | Self::InvalidCsv | Self::InvalidThemeName => {
                "先看输入值的边界与字符口径，很多「非法」其实是被静默吞掉的脏值。"
            }
            Self::IoError | Self::ConfigExists | Self::MissingEnvVar => {
                "先查环境/配置：路径、权限、环境变量来源（env > .env > toml）。"
            }
            Self::PushFailed | Self::AutomationFailed | Self::CookieSessionRequired => {
                "先跑 `moonpub wechat-health`：区分凭证/IP 白名单/登录态，别直接重试。"
            }
            Self::EvidenceMissing | Self::ReleaseGateIncomplete | Self::PreflightGateFailed => {
                "先读门禁报告里失败的具体项，按 next_step 补证据/产物，不要跳过门禁。"
            }
            Self::PhotoVisionProviderUnsupported | Self::ImageGenerationProviderUnsupported => {
                "先跑 `moonpub doctor` 确认 AI provider 配置，核心发布不依赖 AI。"
            }
            Self::PhotoVisionInputInvalid | Self::ImageGenerationFailed => {
                "先看输入素材是否合规、provider 是否可达，再判断是否要换方案。"
            }
            Self::InternalError => "先确认评估方法本身没测错东西，再归因到环境，最后才动代码。",
        }
    }

    /// A recorded post-mortem to read before fixing, where one exists.
    /// `None` means no lesson has been recorded for this failure class yet.
    pub fn lesson(self) -> Option<LessonRef> {
        let doc = "docs/ENGINEERING_LESSONS_ZH.md";
        let lesson = |heading| Some(LessonRef { doc, heading });
        match self {
            Self::InvalidThemeName => lesson(
                "`theme: blu-print` 渲染成功却换了主题：`check --json` 还谎报 `publishable: true`",
            ),
            Self::InvalidCsv => {
                lesson("`radar import` 把 `likes=abc` 当成空值：rc=0 报成功，互动数据全丢")
            }
            Self::InvalidNumber => {
                lesson("带值 flag 命令的路径必须固定在第 2 个参数：同一个 bug 修了第三遍")
            }
            Self::PushFailed => lesson("微信 IP 白名单失败可能来自公网出口漂移"),
            Self::CookieSessionRequired => lesson("持久 Chrome profile 被占用不是 token 失效"),
            Self::AutomationFailed => lesson("后台配置的 ✅ 必须读落库状态，不能信点击日志"),
            Self::EvidenceMissing | Self::ReleaseGateIncomplete => {
                lesson("本地有真实凭据，测出来的是「本机能跑」，不是「CI 能跑」")
            }
            Self::PreflightGateFailed => lesson(
                "微信摘要 120 字上限被当成 120 字节：preflight 报 pass，draft.json 只剩 40 字",
            ),
            Self::MissingValue | Self::UnknownOption | Self::UnknownCommand => {
                lesson("带值 flag 命令的路径必须固定在第 2 个参数：同一个 bug 修了第三遍")
            }
            Self::InternalError => lesson(
                "CI 直接继承 `.cargo/config.toml` 的 target-cpu=native，rustc 以 SIGILL 崩溃",
            ),
            _ => None,
        }
    }
}

#[derive(Serialize)]
struct ErrorPayload<'a> {
    schema_version: &'static str,
    ok: bool,
    command: Option<&'a str>,
    error: ErrorDetail,
    cause_chain: Vec<String>,
    next_step: String,
}

#[derive(Serialize)]
struct ErrorDetail {
    code: &'static str,
    message: String,
    /// 诊断首查：拿到失败后先核什么，按「评估方法 → 环境/配置 → 系统本身」的顺序。
    ///
    /// 和 `next_step` 的分工：`next_step` 是"下一步该跑哪条命令"，
    /// `first_check` 是"在改代码之前先排除什么"。两者都必填，缺一个调用方就会
    /// 直接跳到改实现（历史上反复发生的正是这种跳步）。
    first_check: &'static str,
    /// 这个错误码对应的历史踩坑记录；没登记过的码省略该字段，而不是给 `null`。
    #[serde(skip_serializing_if = "Option::is_none")]
    lesson: Option<LessonRef>,
}

/// Serialize a failure for `--json` callers.
///
/// `argv` is the raw argument list without the binary name; it is used only to
/// infer the attempted command, because parsing may have failed before an
/// [`crate::cli::Options`] value existed.
pub fn error_json(argv: &[String], error: &anyhow::Error) -> String {
    let command = infer_command(argv);
    let (code, next_step) = classify(error, command);
    let payload = ErrorPayload {
        schema_version: "error/v1",
        ok: false,
        command,
        error: ErrorDetail {
            code: code.as_str(),
            message: error.to_string(),
            first_check: code.first_check(),
            lesson: code.lesson(),
        },
        cause_chain: error
            .chain()
            .skip(1)
            .map(|cause| cause.to_string())
            .collect(),
        next_step: executable_next_step(code, next_step),
    };
    super::serialize_json(&payload)
}

/// `error/v1.next_step` is a command the caller is expected to run. Some
/// upstream reports keep human checklist prose or shell snippets in their
/// advisory `next_command` (`release-check` does this on purpose). That is fine
/// in a successful report, but leaking it into a failure object breaks the
/// contract: agents would try to execute "review README.md ..." as a command.
/// Fall back to the code's registered example instead; the original hint stays
/// in `error.message`, so no diagnostic information is lost.
fn executable_next_step(code: ErrorCode, candidate: String) -> String {
    if candidate.starts_with("moonpub ") {
        candidate
    } else {
        code.example_next_step().to_owned()
    }
}

/// First non-flag token, skipping the values of global options that take one.
/// This is deliberately best-effort: on a parse failure it still tells the
/// caller which command it tried to run.
fn infer_command(argv: &[String]) -> Option<&str> {
    let mut skip_value = false;
    for arg in argv {
        if skip_value {
            skip_value = false;
            continue;
        }
        match arg.as_str() {
            "--articles" | "--config" => skip_value = true,
            value if value.starts_with('-') => {}
            value => return Some(value),
        }
    }
    None
}

/// Map an error to a stable machine-readable code and an actionable command.
/// Codes are part of the `error/v1` contract: rename them only with a version
/// bump, never as a side effect of refactoring an error message.
fn classify(error: &anyhow::Error, command: Option<&str>) -> (ErrorCode, String) {
    let help_step = || match command {
        Some(name) if crate::help::for_command(name).is_some() => format!("moonpub help {name}"),
        _ => "moonpub help".to_owned(),
    };
    let Some(app_error) = error.downcast_ref::<AppError>() else {
        if error.downcast_ref::<std::io::Error>().is_some() {
            return (ErrorCode::IoError, "moonpub doctor".to_owned());
        }
        return (ErrorCode::InternalError, "moonpub help".to_owned());
    };
    let (code, next_step) = match app_error {
        AppError::MissingCommand => (ErrorCode::MissingCommand, "moonpub help".to_owned()),
        AppError::MissingValue(_) | AppError::MissingValueString(_) => {
            (ErrorCode::MissingValue, help_step())
        }
        AppError::UnknownOption(_) => (ErrorCode::UnknownOption, help_step()),
        AppError::UnknownCommand(_) => (ErrorCode::UnknownCommand, "moonpub help".to_owned()),
        AppError::Io { .. } => (ErrorCode::IoError, "moonpub doctor".to_owned()),
        AppError::InvalidArticlePath(_) => (ErrorCode::InvalidArticlePath, help_step()),
        AppError::ConfigExists(_) => (ErrorCode::ConfigExists, "moonpub doctor".to_owned()),
        AppError::InvalidNumber { .. } => (ErrorCode::InvalidNumber, help_step()),
        AppError::InvalidThemeName(_) => (ErrorCode::InvalidThemeName, help_step()),
        AppError::InvalidCsv(_) => (ErrorCode::InvalidCsv, help_step()),
        AppError::MissingEnvVar(_) => (ErrorCode::MissingEnvVar, "moonpub doctor".to_owned()),
        AppError::PhotoVisionProviderUnsupported => (
            ErrorCode::PhotoVisionProviderUnsupported,
            "moonpub doctor".to_owned(),
        ),
        AppError::PhotoVisionInput(_) => (ErrorCode::PhotoVisionInputInvalid, help_step()),
        AppError::PushFailed { .. } => (ErrorCode::PushFailed, "moonpub wechat-health".to_owned()),
        AppError::NoDraftJson(_) => (
            ErrorCode::NoDraftJson,
            "moonpub render <article.md>".to_owned(),
        ),
        AppError::NoHtml(_) => (ErrorCode::NoHtml, "moonpub render <article.md>".to_owned()),
        AppError::AutomationFailed { .. } => (
            ErrorCode::AutomationFailed,
            "moonpub wechat-health".to_owned(),
        ),
        AppError::CookieSessionRequired { .. } => {
            (ErrorCode::CookieSessionRequired, "moonpub login".to_owned())
        }
        AppError::EvidenceMissing { next_command, .. } => {
            (ErrorCode::EvidenceMissing, (*next_command).to_owned())
        }
        AppError::ReleaseGateIncomplete { next_command, .. } => {
            (ErrorCode::ReleaseGateIncomplete, next_command.clone())
        }
        AppError::PreflightGateFailed { next_command, .. } => {
            (ErrorCode::PreflightGateFailed, next_command.clone())
        }
        AppError::ImageGenerationProviderUnsupported => (
            ErrorCode::ImageGenerationProviderUnsupported,
            "moonpub doctor".to_owned(),
        ),
        AppError::ImageGenerationFailed(_) => (ErrorCode::ImageGenerationFailed, help_step()),
    };
    (code, next_step)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeSet;
    use std::io;
    use std::path::PathBuf;

    fn payload(args: &[&str], error: AppError) -> serde_json::Value {
        let argv: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
        let error: anyhow::Error = error.into();
        serde_json::from_str(&error_json(&argv, &error)).expect("payload must be valid JSON")
    }

    #[test]
    fn invalid_theme_name_has_the_stable_error_v1_envelope() {
        let value = payload(
            &["--json", "render", "a.md"],
            AppError::InvalidThemeName("blu-print".into()),
        );
        assert_eq!(value["schema_version"], "error/v1");
        assert_eq!(value["ok"], false);
        assert_eq!(value["command"], "render");
        assert_eq!(value["error"]["code"], "invalid_theme_name");
        assert!(
            value["error"]["message"]
                .as_str()
                .unwrap()
                .contains("blu-print")
        );
        assert!(
            value["next_step"].as_str().unwrap().starts_with("moonpub "),
            "next_step must be an executable command"
        );
    }

    #[test]
    fn unknown_command_has_the_stable_error_v1_envelope() {
        let value = payload(
            &["--json", "badcmd"],
            AppError::UnknownCommand("badcmd".into()),
        );
        assert_eq!(value["schema_version"], "error/v1");
        assert_eq!(value["ok"], false);
        assert_eq!(value["command"], "badcmd");
        assert_eq!(value["error"]["code"], "unknown_command");
        assert!(
            value["error"]["message"]
                .as_str()
                .unwrap()
                .contains("badcmd")
        );
        assert_eq!(value["next_step"], "moonpub help");
        assert_eq!(value["cause_chain"].as_array().unwrap().len(), 0);
    }

    #[test]
    fn command_inference_skips_global_option_values() {
        let argv = vec![
            "--articles".to_owned(),
            "/tmp/vault".to_owned(),
            "--config".to_owned(),
            "/tmp/moonpub.toml".to_owned(),
            "--json".to_owned(),
            "cover".to_owned(),
            "a.md".to_owned(),
        ];
        let error: anyhow::Error = AppError::MissingValue("cover <article.md>").into();
        let value: serde_json::Value = serde_json::from_str(&error_json(&argv, &error)).unwrap();
        assert_eq!(value["command"], "cover");
        assert_eq!(value["next_step"], "moonpub help cover");
    }

    #[test]
    fn unknown_command_does_not_point_the_caller_at_itself() {
        let value = payload(
            &["--json", "rendr"],
            AppError::UnknownCommand("rendr".into()),
        );
        assert_eq!(value["next_step"], "moonpub help");
    }

    #[test]
    fn gate_failures_reuse_the_reports_next_command() {
        let value = payload(
            &["--json", "preflight", "a.md", "--strict"],
            AppError::PreflightGateFailed {
                failed_checks: vec!["markdown: missing".to_owned()],
                next_command: "moonpub render a.md".to_owned(),
            },
        );
        assert_eq!(value["error"]["code"], "preflight_gate_failed");
        assert_eq!(value["next_step"], "moonpub render a.md");
        assert!(
            value["error"]["message"]
                .as_str()
                .unwrap()
                .contains("markdown: missing")
        );
    }

    #[test]
    fn io_errors_expose_the_underlying_cause() {
        let value = payload(
            &["--json", "render", "a.md"],
            AppError::Io {
                path: PathBuf::from("a.md"),
                source: io::Error::new(io::ErrorKind::NotFound, "no such file"),
            },
        );
        assert_eq!(value["error"]["code"], "io_error");
        let causes = value["cause_chain"].as_array().unwrap();
        assert_eq!(causes.len(), 1);
        assert!(causes[0].as_str().unwrap().contains("no such file"));
    }

    #[test]
    fn messages_with_quotes_and_newlines_stay_valid_json() {
        let value = payload(
            &["--json", "push", "a.md"],
            AppError::AutomationFailed {
                message: "line \"one\"\nline two".to_owned(),
            },
        );
        assert_eq!(
            value["error"]["message"],
            "browser automation failed: line \"one\"\nline two"
        );
        assert_eq!(value["next_step"], "moonpub wechat-health");
    }

    #[test]
    fn non_app_errors_are_classified_without_panicking() {
        let argv = vec!["--json".to_owned(), "doctor".to_owned()];
        let error = anyhow::anyhow!("unexpected failure");
        let value: serde_json::Value = serde_json::from_str(&error_json(&argv, &error)).unwrap();
        assert_eq!(value["error"]["code"], "internal_error");
        assert_eq!(value["ok"], false);
        assert_eq!(value["next_step"], "moonpub help");
    }

    #[test]
    fn registered_codes_are_unique_well_formed_and_documented() {
        let mut seen = BTreeSet::new();
        for code in ErrorCode::ALL {
            let name = code.as_str();
            assert!(seen.insert(name), "duplicate error code: {name}");
            assert!(
                name.chars()
                    .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_'),
                "error code must be snake_case: {name}"
            );
            assert!(!code.summary().is_empty(), "{name} needs a summary");
            assert!(
                code.example_next_step().starts_with("moonpub "),
                "{name} needs an executable example_next_step"
            );
            assert!(
                !code.first_check().is_empty(),
                "{name} needs a first_check hint"
            );
        }
    }

    /// `lesson()` points at a real heading in the lessons library. The doc is
    /// embedded at compile time (`include_str!`) so the test never depends on
    /// the process cwd; renaming a heading without updating the code — or
    /// inventing a heading that was never written — turns this red.
    #[test]
    fn every_lesson_reference_points_at_a_real_heading() {
        const LESSONS_ZH: &str = include_str!("../../docs/ENGINEERING_LESSONS_ZH.md");
        let with_lesson: Vec<_> = ErrorCode::ALL
            .iter()
            .filter(|code| code.lesson().is_some())
            .collect();
        assert!(
            with_lesson.len() >= 8,
            "error/v1 should advertise several recorded lessons, got {}",
            with_lesson.len()
        );
        for code in with_lesson {
            let lesson = code.lesson().expect("filtered to codes with lessons");
            assert_eq!(
                lesson.doc,
                "docs/ENGINEERING_LESSONS_ZH.md",
                "{} points at an unexpected lesson doc",
                code.as_str()
            );
            let heading = format!("### {}", lesson.heading);
            assert!(
                LESSONS_ZH.lines().any(|line| line == heading),
                "{} advertises a lesson heading that no longer exists: {heading}",
                code.as_str()
            );
        }
    }

    /// One instance of every failure path that can reach `error_json`, plus the
    /// non-`AppError` fallbacks. The exhaustive `match` in `classify()` is the
    /// compile-time guard for new variants; keeping this list in sync gives the
    /// runtime contract tests something real to walk.
    fn all_failures() -> Vec<anyhow::Error> {
        let path = || PathBuf::from("a.md");
        vec![
            AppError::MissingCommand.into(),
            AppError::MissingValue("cover <article.md>").into(),
            AppError::MissingValueString("cover".into()).into(),
            AppError::UnknownOption("--nope".into()).into(),
            AppError::UnknownCommand("badcmd".into()).into(),
            AppError::Io {
                path: path(),
                source: io::Error::other("disk"),
            }
            .into(),
            AppError::InvalidArticlePath(path()).into(),
            AppError::ConfigExists(path()).into(),
            AppError::InvalidNumber {
                flag: "--top",
                value: "many".into(),
            }
            .into(),
            AppError::InvalidThemeName("blu-print".into()).into(),
            AppError::InvalidCsv("a,,b".into()).into(),
            AppError::MissingEnvVar("WECHAT_APPID").into(),
            AppError::PhotoVisionProviderUnsupported.into(),
            AppError::PhotoVisionInput("bad".into()).into(),
            AppError::PushFailed {
                message: "401".into(),
                ip_hint: None,
            }
            .into(),
            AppError::NoDraftJson(path()).into(),
            AppError::NoHtml(path()).into(),
            AppError::AutomationFailed {
                message: "timeout".into(),
            }
            .into(),
            AppError::CookieSessionRequired {
                message: "expired".into(),
            }
            .into(),
            AppError::EvidenceMissing {
                missing_count: 1,
                next_command: "moonpub evidence-status",
            }
            .into(),
            AppError::ReleaseGateIncomplete {
                failed_count: 1,
                // Real `release-check` reports put human checklist prose here.
                next_command: "open docs/RELEASE_GATE_v0.4.13_ZH.md".into(),
            }
            .into(),
            AppError::PreflightGateFailed {
                failed_checks: vec!["markdown: missing".into()],
                next_command: "moonpub render a.md".into(),
            }
            .into(),
            AppError::ImageGenerationProviderUnsupported.into(),
            AppError::ImageGenerationFailed("timeout".into()).into(),
            io::Error::other("raw io").into(),
            anyhow::anyhow!("unexpected failure"),
        ]
    }

    /// The registry is only useful if it cannot drift from `classify()`. Adding
    /// an `AppError` variant already breaks the exhaustive match above; this
    /// test also catches a new code that was added to `classify` but not to
    /// `ALL`, and a registry entry that no failure can ever produce.
    #[test]
    fn every_classified_code_is_in_the_registry_and_vice_versa() {
        let errors = all_failures();
        let produced: BTreeSet<&str> = errors
            .iter()
            .map(|error| classify(error, None).0.as_str())
            .collect();
        let registered: BTreeSet<&str> = ErrorCode::ALL.iter().map(|code| code.as_str()).collect();
        let unregistered: Vec<_> = produced.difference(&registered).collect();
        let unreachable: Vec<_> = registered.difference(&produced).collect();
        assert!(
            unregistered.is_empty(),
            "classify() emits codes missing from ErrorCode::ALL: {unregistered:?}"
        );
        assert!(
            unreachable.is_empty(),
            "ErrorCode::ALL lists codes no failure can produce: {unreachable:?}"
        );
    }

    /// The failure contract promises an executable command, not prose. Walk
    /// every failure path through the real payload builder so a future report
    /// cannot smuggle a human checklist line into `next_step` again.
    #[test]
    fn every_failure_next_step_is_an_executable_moonpub_command() {
        let argv = vec!["--json".to_owned(), "doctor".to_owned()];
        for error in all_failures() {
            let value: serde_json::Value =
                serde_json::from_str(&error_json(&argv, &error)).unwrap();
            let step = value["next_step"].as_str().expect("next_step is a string");
            assert!(
                step.starts_with("moonpub "),
                "{} produced a non-executable next_step: {step:?}",
                value["error"]["code"]
            );
        }
    }

    #[test]
    fn release_gate_prose_and_shell_snippets_never_leak_into_next_step() {
        for candidate in [
            "open docs/RELEASE_GATE_v0.4.13_ZH.md",
            "review README.md README_zh.md docs/USER_GUIDE.md PROGRESS.md",
            "prepare release notes",
            "git status --short",
            "",
        ] {
            let value = payload(
                &["--json", "release-check", "--strict"],
                AppError::ReleaseGateIncomplete {
                    failed_count: 1,
                    next_command: candidate.into(),
                },
            );
            assert_eq!(value["error"]["code"], "release_gate_incomplete");
            assert_eq!(value["next_step"], "moonpub release-check");
            assert!(
                value["error"]["message"]
                    .as_str()
                    .unwrap()
                    .contains(candidate),
                "the original hint must stay in error.message: {candidate:?}"
            );
        }

        // A concrete moonpub command from a report is still worth keeping.
        let dynamic = payload(
            &["--json", "preflight", "a.md", "--strict"],
            AppError::PreflightGateFailed {
                failed_checks: vec!["markdown: missing".into()],
                next_command: "moonpub render a.md".into(),
            },
        );
        assert_eq!(dynamic["next_step"], "moonpub render a.md");
    }
}
