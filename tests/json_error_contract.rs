//! End-to-end contract for the `error/v1` payload.
//!
//! The contract lives at the process boundary — stdout, stderr and exit code
//! all matter — so these tests invoke the real binary instead of calling the
//! builder directly. Text-mode failures must stay byte-for-byte unchanged.

use moonpub::protocol::ErrorCode;
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

/// 失败对象必须带 `first_check`，而且必须和注册表里的那张表逐字一致。
///
/// 这条测试锁的是「失败 → 先查什么」的闭环：如果只把诊断写进 `help errors`
/// 清单，调用方拿到失败时仍然只有一个 `next_step`，会直接跳到改实现。
#[test]
fn failure_payload_carries_the_registered_first_check() {
    let cwd = workspace("first-check");
    let output = run(&cwd, &["--json", "badcmd"]);

    assert!(!output.status.success(), "failure must exit non-zero");
    let value = stdout_json(&output);
    let emitted = value["error"]["code"]
        .as_str()
        .expect("error.code is a string");
    let registered = ErrorCode::ALL
        .iter()
        .copied()
        .find(|code| code.as_str() == emitted)
        .expect("every emitted code must be registered in ErrorCode::ALL");

    let first_check = value["error"]["first_check"]
        .as_str()
        .expect("failure must carry error.first_check");
    assert_eq!(first_check, registered.first_check());
    assert!(
        !first_check.trim().is_empty(),
        "{emitted} needs a non-empty first_check"
    );
    fs::remove_dir_all(cwd).ok();
}

/// `lesson` 是「这个坑以前踩过」的指针：登记过的码给出 doc + heading，
/// 没登记的码必须整个字段省略——不要让 `null` 假装成「查过了，没有教训」。
#[test]
fn lesson_pointer_is_present_exactly_when_registered() {
    let cwd = workspace("lesson-pointer");
    let output = run(&cwd, &["--json", "badcmd"]);

    let value = stdout_json(&output);
    let emitted = value["error"]["code"]
        .as_str()
        .expect("error.code is a string");
    let registered = ErrorCode::ALL
        .iter()
        .copied()
        .find(|code| code.as_str() == emitted)
        .expect("every emitted code must be registered");

    match registered.lesson() {
        Some(lesson) => {
            assert_eq!(value["error"]["lesson"]["doc"], lesson.doc);
            assert_eq!(value["error"]["lesson"]["heading"], lesson.heading);
        }
        None => assert!(
            value["error"].get("lesson").is_none(),
            "an unregistered code must omit `lesson` entirely, not emit null"
        ),
    }
    fs::remove_dir_all(cwd).ok();

    // 第二个方向：故意选一个没有登记教训的失败，确认字段真的是“省略”而不是“null”。
    let cwd = workspace("lesson-pointer-none");
    let output = run(&cwd, &["--json", "push", "Articles/drafts/nope.md"]);
    assert!(
        !output.status.success(),
        "push without draft.json must fail"
    );
    let value = stdout_json(&output);
    let emitted = value["error"]["code"].as_str().expect("error.code");
    let registered = ErrorCode::ALL
        .iter()
        .copied()
        .find(|code| code.as_str() == emitted)
        .expect("every emitted code must be registered");
    // 这个探针必须是「无教训」的码，否则省略分支就没被覆盖——宁可红，也不要静默跳过。
    assert!(
        registered.lesson().is_none(),
        "probe stale: {emitted} now has a registered lesson; pick another failure to cover the omit branch"
    );
    assert!(
        value["error"].get("lesson").is_none(),
        "{emitted} has no registered lesson, so the key must be absent: {value}"
    );
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

/// `release-check`'s report keeps human checklist prose in `next_command`.
/// `--strict` used to copy that prose straight into `error/v1.next_step`, which
/// the protocol promises is an executable command. Run in an empty workspace
/// so the release gate document is missing and the first failing check is the
/// prose one — no dependency on this repository's real gate state.
#[test]
fn strict_release_gate_failure_never_emits_prose_as_next_step() {
    let cwd = workspace("release-gate");
    let output = run(&cwd, &["--json", "release-check", "--strict"]);
    assert!(!output.status.success());
    let value = stdout_json(&output);
    assert_eq!(value["error"]["code"], "release_gate_incomplete");
    let next_step = value["next_step"].as_str().expect("next_step is a string");
    assert!(
        next_step.starts_with("moonpub "),
        "release-check --strict leaked a non-executable next_step: {next_step:?}"
    );
    assert_eq!(next_step, "moonpub release-check");
    assert!(
        value["error"]["message"]
            .as_str()
            .unwrap()
            .contains("docs/RELEASE_GATE"),
        "the original human hint must survive in error.message"
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

/// cookie 模式下没有 appsecret 实现的命令，必须给专属错误码和一条能推进的
/// `next_step`，而不是笼统的「缺 WECHAT_SECRET」。
///
/// 这条路径以前是个闭环：缺凭据的提示把人引向 cookie 模式，cookie 模式又把人
/// 推回缺凭据——同一句话看两遍，命令还是没跑成。
#[test]
fn cookie_mode_names_the_commands_it_cannot_run() {
    let cwd = workspace("cookie-uncovered");
    fs::write(cwd.join("moonpub.toml"), "[articles]\nroot = \".\"\n").expect("write config");
    fs::write(
        cwd.join("Articles/drafts/x.md"),
        "---\ntitle: X\n---\n\n正文。\n",
    )
    .expect("write article");
    fs::write(
        cwd.join("Articles/drafts/x.draft.json"),
        "{\"articles\":[]}",
    )
    .expect("write draft json");

    let cases: [(&[&str], &str, &str); 2] = [
        (&["--json", "list-drafts"], "list-drafts", "moonpub status"),
        (
            &[
                "--json",
                "update-draft",
                "Articles/drafts/x.md",
                "--media-id",
                "123",
            ],
            "update-draft",
            "moonpub push",
        ),
    ];

    for (args, command, expect_step) in cases {
        let output = Command::new(env!("CARGO_BIN_EXE_moonpub"))
            .args(args)
            .current_dir(&cwd)
            .env_remove("WECHAT_APPID")
            .env_remove("WECHAT_SECRET")
            .env("WECHAT_AUTH_METHOD", "cookie")
            .output()
            .expect("run moonpub binary");

        assert!(!output.status.success(), "{command} must fail");
        let value = stdout_json(&output);
        assert_eq!(
            value["error"]["code"], "cookie_auth_unsupported",
            "{command}: {value}"
        );
        assert_eq!(value["command"], command);
        let message = value["error"]["message"].as_str().unwrap();
        assert!(
            message.contains("cookie mode covers"),
            "{command} must state the cookie coverage: {message}"
        );
        let step = value["next_step"].as_str().unwrap();
        assert!(
            step.starts_with(expect_step),
            "{command} next_step should be actionable: {step}"
        );
        // 文本模式同一句话留在 stderr，stdout 不掺人类文案。
        let stderr = String::from_utf8_lossy(&output.stderr);
        assert!(
            stderr.contains(&format!("cookie mode does not cover `{command}`")),
            "{command}: {stderr}"
        );
    }

    fs::remove_dir_all(cwd).ok();
}

/// cookie 模式下删草稿、会话没就绪、又没有 AppSecret 凭据：报的必须是 cookie
/// 会话本身的问题，而不是 `missing_env_var`。
///
/// 以前这里无条件回退 appsecret，于是真实原因（会话没就绪）被盖成「缺
/// WECHAT_SECRET」，而那条报错给的出路正是刚刚失败的 cookie 通道——一个
/// 闭环，`error.code` 也跟着撒了谎。
#[test]
fn cookie_mode_delete_reports_the_session_problem_not_a_missing_env_var() {
    let cwd = workspace("cookie-delete");
    fs::write(cwd.join("moonpub.toml"), "[articles]\nroot = \".\"\n").expect("write config");

    let output = Command::new(env!("CARGO_BIN_EXE_moonpub"))
        .args(["--json", "delete-draft", "2247483648"])
        .current_dir(&cwd)
        // 换掉 HOME：既保证找不到真实登录态，也不会碰到跑测试那台机器的
        // `~/.config/moonpub/session.json`。
        .env("HOME", &cwd)
        .env_remove("WECHAT_APPID")
        .env_remove("WECHAT_SECRET")
        .env("WECHAT_AUTH_METHOD", "cookie")
        .output()
        .expect("run moonpub binary");

    assert!(
        !output.status.success(),
        "delete-draft must fail without a login session"
    );
    let value = stdout_json(&output);
    assert_eq!(value["error"]["code"], "cookie_session_required", "{value}");
    assert_eq!(value["command"], "delete-draft");
    assert_eq!(value["next_step"], "moonpub login");

    let message = value["error"]["message"].as_str().unwrap();
    assert!(
        message.contains("moonpub delete-draft 2247483648"),
        "the retry must name the command the user actually ran: {message}"
    );
    assert!(
        !message.contains("missing env var"),
        "a cookie-session failure must not be reported as a missing variable: {message}"
    );
    assert!(
        !message.contains("moonpub push"),
        "delete-draft must not be told to run push: {message}"
    );

    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(!stderr.contains("missing env var"), "{stderr}");
    // 不能同时出现「cookie 失败」和「回退」：回退跑不起来，只会埋掉真实原因。
    assert!(!stderr.contains("改用 AppSecret 通道"), "{stderr}");

    fs::remove_dir_all(cwd).ok();
}

/// 非数字 media_id 是**参数**问题，不是通道问题。
///
/// 以前它被包成 `push_failed`：`first_check` 让人跑 `wechat-health` 查凭据 /
/// IP 白名单 / 登录态，`next_step` 也是 `moonpub wechat-health`——用户只是把
/// 素材 id 当成了草稿 id，却被送去诊断一个完全健康的通道。
#[test]
fn non_numeric_media_id_is_an_argument_error_not_a_channel_error() {
    let cwd = workspace("bad-media-id");
    fs::write(cwd.join("moonpub.toml"), "[articles]\nroot = \".\"\n").expect("write config");

    let output = Command::new(env!("CARGO_BIN_EXE_moonpub"))
        .args(["--json", "delete-draft", "R1LkThumb"])
        .current_dir(&cwd)
        .env("HOME", &cwd)
        .env_remove("WECHAT_APPID")
        .env_remove("WECHAT_SECRET")
        .env("WECHAT_AUTH_METHOD", "cookie")
        .output()
        .expect("run moonpub binary");

    assert!(!output.status.success(), "a bad media_id must fail");
    let value = stdout_json(&output);
    assert_eq!(value["error"]["code"], "invalid_media_id", "{value}");
    assert_eq!(value["command"], "delete-draft");
    assert_eq!(value["next_step"], "moonpub help delete-draft");

    let message = value["error"]["message"].as_str().unwrap();
    assert!(message.contains("R1LkThumb"), "{message}");
    assert!(
        message.contains("moonpub list-drafts"),
        "the message must say where a real id comes from: {message}"
    );
    assert!(
        !message.contains("push failed"),
        "an argument error must not be reported as a push failure: {message}"
    );
    // 通道是好的，别把人送去诊断它。
    let first_check = value["error"]["first_check"].as_str().unwrap();
    assert!(!first_check.contains("wechat-health"), "{first_check}");
    assert!(
        value["error"]["lesson"].is_null(),
        "no lesson is registered for a bad argument: {value}"
    );

    fs::remove_dir_all(cwd).ok();
}
