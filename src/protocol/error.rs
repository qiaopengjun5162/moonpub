//! Machine-readable failure payload (`error/v1`).
//!
//! Text-mode failures are unchanged: the human message stays on stderr, the
//! process exits non-zero, and stdout stays empty. When the caller explicitly
//! asked for `--json`, stdout additionally carries this object so an agent can
//! branch on a stable `code` instead of parsing prose from stderr.

use serde::Serialize;

use crate::error::AppError;

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
            code,
            message: error.to_string(),
        },
        cause_chain: error
            .chain()
            .skip(1)
            .map(|cause| cause.to_string())
            .collect(),
        next_step,
    };
    super::serialize_json(&payload)
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
fn classify(error: &anyhow::Error, command: Option<&str>) -> (&'static str, String) {
    let help_step = || match command {
        Some(name) if crate::help::for_command(name).is_some() => format!("moonpub help {name}"),
        _ => "moonpub help".to_owned(),
    };
    let Some(app_error) = error.downcast_ref::<AppError>() else {
        if error.downcast_ref::<std::io::Error>().is_some() {
            return ("io_error", "moonpub doctor".to_owned());
        }
        return ("internal_error", "moonpub help".to_owned());
    };
    let (code, next_step) = match app_error {
        AppError::MissingCommand => ("missing_command", "moonpub help".to_owned()),
        AppError::MissingValue(_) | AppError::MissingValueString(_) => {
            ("missing_value", help_step())
        }
        AppError::UnknownOption(_) => ("unknown_option", help_step()),
        AppError::UnknownCommand(_) => ("unknown_command", "moonpub help".to_owned()),
        AppError::Io { .. } => ("io_error", "moonpub doctor".to_owned()),
        AppError::InvalidArticlePath(_) => ("invalid_article_path", help_step()),
        AppError::ConfigExists(_) => ("config_exists", "moonpub doctor".to_owned()),
        AppError::InvalidNumber { .. } => ("invalid_number", help_step()),
        AppError::InvalidCsv(_) => ("invalid_csv", help_step()),
        AppError::MissingEnvVar(_) => ("missing_env_var", "moonpub doctor".to_owned()),
        AppError::PhotoVisionProviderUnsupported => (
            "photo_vision_provider_unsupported",
            "moonpub doctor".to_owned(),
        ),
        AppError::PhotoVisionInput(_) => ("photo_vision_input_invalid", help_step()),
        AppError::PushFailed { .. } => ("push_failed", "moonpub wechat-health".to_owned()),
        AppError::NoDraftJson(_) => ("no_draft_json", "moonpub render <article.md>".to_owned()),
        AppError::NoHtml(_) => ("no_html", "moonpub render <article.md>".to_owned()),
        AppError::AutomationFailed { .. } => {
            ("automation_failed", "moonpub wechat-health".to_owned())
        }
        AppError::CookieSessionRequired { .. } => {
            ("cookie_session_required", "moonpub login".to_owned())
        }
        AppError::EvidenceMissing { next_command, .. } => {
            ("evidence_missing", (*next_command).to_owned())
        }
        AppError::ReleaseGateIncomplete { next_command, .. } => {
            ("release_gate_incomplete", next_command.clone())
        }
        AppError::PreflightGateFailed { next_command, .. } => {
            ("preflight_gate_failed", next_command.clone())
        }
        AppError::ImageGenerationProviderUnsupported => (
            "image_generation_provider_unsupported",
            "moonpub doctor".to_owned(),
        ),
        AppError::ImageGenerationFailed(_) => ("image_generation_failed", help_step()),
    };
    (code, next_step)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io;
    use std::path::PathBuf;

    fn payload(args: &[&str], error: AppError) -> serde_json::Value {
        let argv: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
        let error: anyhow::Error = error.into();
        serde_json::from_str(&error_json(&argv, &error)).expect("payload must be valid JSON")
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
}
