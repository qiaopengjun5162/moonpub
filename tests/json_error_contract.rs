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
