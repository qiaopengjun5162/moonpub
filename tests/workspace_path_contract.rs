//! 路径契约：同一个文件，无论从哪条命令、文本还是 JSON 里看到，都是同一个字符串。
//!
//! 此前 `new` / `render` / `check` / `preview` 印绝对路径，`status` 印工作区相对
//! 路径：同一个文件两种写法，用户没法判断两次看到的是不是同一个东西；更糟的是
//! macOS 上绝对路径要过符号链接规范化，用户敲 `/tmp/x`，工具印 `/private/tmp/x`，
//! 两边字符串永远不等。工具印出来的 token 必须能原样粘回自己的下一条命令，
//! 所以这里在进程边界上钉三件事：
//!
//! 1. `new` 印出的路径不带工作区前缀，且能直接作为 `check` 的参数；
//! 2. `render` / `check` / `status` / `preflight` 对同一篇文章印同一个字符串；
//! 3. `status` 的 `next:` 引导拆成 argv 能跑通，且跑出来的还是那个字符串。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn workspace(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("moonpub-paths-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Articles/drafts")).expect("create temp workspace");
    // 本机真的存在 ~/.config/moonpub/session.json：不隔离 HOME，cookie 相关
    // 分支会读到真实登录态，测试就跟着本机状态飘。
    fs::create_dir_all(root.join("home")).expect("create isolated HOME");
    root
}

fn run(cwd: &Path, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_moonpub"))
        .args(args)
        .current_dir(cwd)
        .env("HOME", cwd.join("home"))
        .env_remove("WECHAT_APPID")
        .env_remove("WECHAT_SECRET")
        .output()
        .expect("run moonpub binary")
}

fn stdout(output: &Output) -> String {
    String::from_utf8_lossy(&output.stdout).into_owned()
}

/// 从 `next: moonpub …` 那一行拆出 argv（第一个 token 是命令名，替换成二进制路径）。
fn next_argv(text: &str) -> Vec<String> {
    let line = text
        .lines()
        .find_map(|line| line.strip_prefix("next: "))
        .unwrap_or_else(|| panic!("输出里没有 next: 行：\n{text}"));
    let mut argv: Vec<String> = line.split_whitespace().map(str::to_owned).collect();
    assert_eq!(argv.first().map(String::as_str), Some("moonpub"), "{line}");
    argv.remove(0);
    argv
}

#[test]
fn the_path_new_prints_can_be_pasted_back_as_the_next_argument() {
    let root = workspace("new");
    let created = run(&root, &["new", "路径 测试"]);
    assert!(created.status.success(), "{}", stdout(&created));
    let text = stdout(&created);

    // 第二行是文件路径本身。
    let printed = text.lines().nth(1).expect("created 行之后是路径").trim();
    assert_eq!(printed, "Articles/drafts/路径-测试.md");
    assert!(
        !printed.starts_with('/'),
        "印出来的路径不该是绝对路径：{printed}"
    );

    // 「工具自己印出的名字必须是自己的合法参数」——照着抄就能跑。
    let checked = run(&root, &["check", printed]);
    assert!(
        checked.status.success(),
        "`check {printed}` 失败：{}",
        stdout(&checked)
    );
    assert!(
        stdout(&checked).contains(&format!("markdown: ok {printed}")),
        "{}",
        stdout(&checked)
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn every_command_spells_the_same_file_the_same_way() {
    let root = workspace("same-spelling");
    fs::write(
        root.join("Articles/drafts/一致.md"),
        "---\ntitle: 一致\n---\n\n正文。\n",
    )
    .expect("write md");

    let rendered = run(&root, &["render", "Articles/drafts/一致.md"]);
    assert!(rendered.status.success(), "{}", stdout(&rendered));
    let render_text = stdout(&rendered);
    assert!(
        render_text.contains("  html:  Articles/drafts/一致.html"),
        "{render_text}"
    );
    assert!(
        render_text.contains("next: moonpub preview Articles/drafts/一致.md"),
        "{render_text}"
    );

    let check_text = stdout(&run(&root, &["check", "Articles/drafts/一致.md"]));
    let check_json = stdout(&run(&root, &["check", "Articles/drafts/一致.md", "--json"]));
    let status_text = stdout(&run(&root, &["status"]));
    let preflight_json = stdout(&run(
        &root,
        &["preflight", "Articles/drafts/一致.md", "--json"],
    ));

    let article = "Articles/drafts/一致.md";
    assert!(
        check_text.contains(&format!("  markdown: ok {article}")),
        "{check_text}"
    );
    assert!(
        check_text.contains(&format!("next: moonpub push {article} --render")),
        "{check_text}"
    );
    assert!(
        check_json.contains(&format!(r#""article_path":"{article}""#)),
        "{check_json}"
    );
    assert!(
        check_json.contains(&format!(
            r#""next_command":"moonpub push {article} --render""#
        )),
        "{check_json}"
    );
    assert!(
        status_text.contains(&format!("  {article} [rendered]")),
        "{status_text}"
    );
    assert!(
        preflight_json.contains(&format!(r#""article_path":"{article}""#)),
        "{preflight_json}"
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn the_next_step_status_prints_runs_as_written() {
    let root = workspace("next-runs");
    fs::write(
        root.join("Articles/drafts/引导.md"),
        "---\ntitle: 引导\n---\n\n正文。\n",
    )
    .expect("write md");

    let status_text = stdout(&run(&root, &["status"]));
    let argv = next_argv(&status_text);
    let args: Vec<&str> = argv.iter().map(String::as_str).collect();
    let followed = run(&root, &args);

    assert!(
        followed.status.success(),
        "照着 next: 跑失败：{}",
        stdout(&followed)
    );
    // 跑完还是同一篇文章、同一个字符串——引导没有把自己带偏。
    assert!(
        stdout(&followed).contains("Articles/drafts/引导.md"),
        "{}",
        stdout(&followed)
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn a_path_outside_the_workspace_keeps_its_absolute_form() {
    let root = workspace("outside");
    let outside = std::env::temp_dir().join(format!("moonpub-outside-{}", std::process::id()));
    fs::create_dir_all(&outside).expect("create outside dir");
    fs::write(outside.join("外.md"), "---\ntitle: 外\n---\n\n正文。\n").expect("write md");

    let article = outside.join("外.md");
    let text = stdout(&run(
        &root,
        &["check", article.to_str().expect("utf-8 path")],
    ));

    // 显式传了工作区外的绝对路径就原样印：裁成 `../…` 更难核对。
    assert!(
        text.contains(&format!("markdown: ok {}", article.display())),
        "{text}"
    );

    fs::remove_dir_all(root).ok();
    fs::remove_dir_all(outside).ok();
}
