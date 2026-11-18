//! 首次运行契约：`init --yes` 写出来的配置，必须能被同版本的 `render` 直接消费。
//!
//! 这条链路以前只在 `release.yml` 的打包 smoke 里跑——等于「发版构建到一半才发现
//! 新用户第一步就崩」。v0.4.14 就是这么翻车的：`init` 生成的配置带行尾注释
//! （`theme = "default" # default | warm | ...`），配置解析却把注释一起当成值，
//! 主题校验一严就报 `invalid theme name`。
//!
//! 所以这里把「写配置 → 读配置 → 渲染」钉成进程级契约，跑在常规 CI 里。

use std::fs;
use std::path::PathBuf;
use std::process::{Command, Output};

fn workspace(name: &str) -> PathBuf {
    let root =
        std::env::temp_dir().join(format!("moonpub-first-run-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("create temp workspace");
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

#[test]
fn init_then_render_works_on_a_fresh_workspace() {
    let root = workspace("init-render");

    let init = run(&root, &["init", "--yes"]);
    assert!(
        init.status.success(),
        "init --yes 失败：{}",
        String::from_utf8_lossy(&init.stderr)
    );

    let article = root.join("Articles/hello/hello.md");
    fs::create_dir_all(article.parent().expect("article has a parent")).expect("create Articles");
    fs::write(&article, "---\ntitle: 首篇\n---\n\n正文一段。\n").expect("write article");

    let render = run(&root, &["render", "Articles/hello/hello.md"]);
    assert!(
        render.status.success(),
        "init 生成的配置不能被 render 直接使用：{}",
        String::from_utf8_lossy(&render.stderr)
    );
    assert!(
        root.join("Articles/hello/hello.html").exists(),
        "render 没有产出 HTML"
    );

    let _ = fs::remove_dir_all(&root);
}

#[test]
fn generated_config_values_are_parsed_without_the_trailing_comment() {
    // 上一版在这里翻车：`theme` 读出来是 `default" # default | warm | ...`。
    // 只看落盘内容抓不到（文件里本来就有注释），要看**解析后**的生效主题。
    let root = workspace("config-values");
    let init = run(&root, &["init", "--yes"]);
    assert!(init.status.success(), "init --yes 失败");

    let article = root.join("Articles/hello/hello.md");
    fs::create_dir_all(article.parent().expect("article has a parent")).expect("create Articles");
    fs::write(&article, "---\ntitle: 首篇\n---\n\n正文一段。\n").expect("write article");

    let check = run(&root, &["--json", "check", "Articles/hello/hello.md"]);
    assert!(
        check.status.success(),
        "check 失败：{}",
        String::from_utf8_lossy(&check.stderr)
    );
    let value: serde_json::Value =
        serde_json::from_slice(&check.stdout).expect("check --json 必须是单个 JSON 对象");
    assert_eq!(
        value["effective_theme"], "default",
        "生成配置里的默认主题被行尾注释污染了：{}",
        value["effective_theme"]
    );

    let _ = fs::remove_dir_all(&root);
}
