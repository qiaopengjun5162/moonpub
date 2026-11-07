//! End-to-end contract for the `error/v1` payload.
//!
//! The contract lives at the process boundary — stdout, stderr and exit code
//! all matter — so these tests invoke the real binary instead of calling the
//! builder directly. Text-mode failures must stay byte-for-byte unchanged.

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn workspace(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("moonpub-error-v1-{name}-{}", std::process::id()));
    fs::create_dir_all(root.join("Articles/drafts")).expect("create temp workspace");
    root
}

fn run(cwd: &PathBuf, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_moonpub"))
        .args(args)
        .current_dir(cwd)
        .env_remove("WECHAT_APPID")
        .env_remove("WECHAT_SECRET")
        .output()
        .expect("run moonpub binary")
}

fn stdout_json(output: &Output) -> serde_json::Value {
    serde_json::from_slice(&output.stdout).expect("stdout must be one valid JSON object")
}

#[test]
fn json_failure_emits_error_v1_and_keeps_human_stderr() {
    let cwd = workspace("unknown");
    let output = run(&cwd, &["--json", "badcmd"]);

    assert!(!output.status.success(), "failure must exit non-zero");
    let value = stdout_json(&output);
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
    assert!(String::from_utf8_lossy(&output.stderr).contains("unknown command"));
    fs::remove_dir_all(cwd).ok();
}

#[test]
fn text_failure_still_leaves_stdout_empty() {
    let cwd = workspace("text");
    let output = run(&cwd, &["badcmd"]);

    assert!(!output.status.success());
    assert!(output.stdout.is_empty(), "text mode must not emit JSON");
    assert!(!output.stderr.is_empty(), "human error stays on stderr");
    fs::remove_dir_all(cwd).ok();
}

#[test]
fn json_success_is_unchanged() {
    let cwd = workspace("success");
    let output = run(&cwd, &["--json", "help"]);

    assert!(output.status.success());
    assert_eq!(stdout_json(&output)["schema_version"], "help/v1");
    fs::remove_dir_all(cwd).ok();
}

#[test]
fn missing_value_points_at_that_commands_help() {
    let cwd = workspace("missing-value");
    let output = run(&cwd, &["--json", "render"]);

    assert!(!output.status.success());
    let value = stdout_json(&output);
    assert_eq!(value["error"]["code"], "missing_value");
    assert_eq!(value["command"], "render");
    assert_eq!(value["next_step"], "moonpub help render");
    fs::remove_dir_all(cwd).ok();
}

#[test]
fn help_errors_exposes_the_full_error_code_registry() {
    let cwd = workspace("help-errors");

    let page = run(&cwd, &["--json", "help", "errors"]);
    assert!(page.status.success());
    let value = stdout_json(&page);
    assert_eq!(value["schema_version"], "help/v1");
    assert_eq!(value["error_schema_version"], "error/v1");
    let codes = value["error_codes"]
        .as_array()
        .expect("error_codes should be an array");
    assert!(!codes.is_empty(), "the registry must not be empty");
    for entry in codes {
        assert!(entry["code"].as_str().is_some_and(|code| !code.is_empty()));
        assert!(
            entry["summary"]
                .as_str()
                .is_some_and(|summary| !summary.is_empty())
        );
        assert!(
            entry["example_next_step"]
                .as_str()
                .is_some_and(|step| step.starts_with("moonpub "))
        );
    }

    // The overview carries the very same list; a caller should not have to
    // reconcile two registries.
    let overview = run(&cwd, &["--json", "help"]);
    assert!(overview.status.success());
    assert_eq!(stdout_json(&overview)["error_codes"], value["error_codes"]);

    // Trailing --json and the human-readable page must agree as well.
    let trailing = run(&cwd, &["help", "errors", "--json"]);
    assert!(trailing.status.success());
    assert_eq!(stdout_json(&trailing)["error_codes"], value["error_codes"]);

    let text = run(&cwd, &["help", "errors"]);
    assert!(text.status.success());
    let text = String::from_utf8_lossy(&text.stdout);
    for entry in codes {
        assert!(
            text.contains(entry["code"].as_str().unwrap()),
            "{} missing from `moonpub help errors`",
            entry["code"]
        );
    }
    fs::remove_dir_all(cwd).ok();
}

#[test]
fn a_real_failures_code_is_in_the_help_registry() {
    let cwd = workspace("code-in-registry");
    let failure = run(&cwd, &["--json", "badcmd"]);
    assert!(!failure.status.success());
    let code = stdout_json(&failure)["error"]["code"]
        .as_str()
        .expect("failure must carry a code")
        .to_owned();

    let page = run(&cwd, &["--json", "help", "errors"]);
    let page_json = stdout_json(&page);
    let codes = page_json["error_codes"]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry["code"].as_str().unwrap())
        .collect::<Vec<_>>();
    assert!(
        codes.contains(&code.as_str()),
        "{code} is emitted but not advertised by help"
    );
    fs::remove_dir_all(cwd).ok();
}

#[test]
fn strict_gate_failure_carries_an_actionable_next_step() {
    let cwd = workspace("strict");
    let output = run(
        &cwd,
        &[
            "--json",
            "preflight",
            "Articles/drafts/missing.md",
            "--strict",
        ],
    );

    assert!(!output.status.success());
    let value = stdout_json(&output);
    assert_eq!(value["error"]["code"], "preflight_gate_failed");
    assert!(value["next_step"].as_str().unwrap().starts_with("moonpub "));
    assert!(!String::from_utf8_lossy(&output.stderr).is_empty());
    fs::remove_dir_all(cwd).ok();
}
