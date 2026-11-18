//! 工作区清单的契约：`status` 必须看得见「这篇文章已经推到微信草稿箱了」。
//!
//! 三处表面（`status` 文本、`status --json`、`preview` 的下一步）共用同一份判断，
//! 这里在进程边界上各钉一条——判定散落就会漂移，而漂移的方向总是「把已推送的
//! 文章当成没推送」，代价是重新 `push`：那不是更新，它会新建草稿、删掉旧的、
//! 换掉 `media_id`，用户手里那个 id 和微信后台里那篇草稿的链接一起作废。

use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn workspace(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!("moonpub-status-{name}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(root.join("Articles/drafts")).expect("create temp workspace");
    // 本机真的存在 ~/.config/moonpub/session.json：不隔离 HOME，cookie 相关的
    // 分支会读到真实登录态，测试就跟着本机状态飘。
    fs::create_dir_all(root.join("home")).expect("create isolated HOME");
    root
}

fn run(cwd: &PathBuf, args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_moonpub"))
        .args(args)
        .current_dir(cwd)
        .env("HOME", cwd.join("home"))
        .env_remove("WECHAT_APPID")
        .env_remove("WECHAT_SECRET")
        .output()
        .expect("run moonpub binary")
}

/// 造一篇「已经推过」的草稿：`push` 会留下 html + draft.json + media_id。
fn pushed_draft(root: &Path, slug: &str, media_id: &str) {
    fs::write(root.join(format!("Articles/drafts/{slug}.md")), "# 标题\n").expect("write md");
    fs::write(
        root.join(format!("Articles/drafts/{slug}.html")),
        "<p>正文</p>",
    )
    .expect("write html");
    fs::write(
        root.join(format!("Articles/drafts/{slug}.draft.json")),
        "{}",
    )
    .expect("write draft.json");
    fs::write(
        root.join(format!("Articles/drafts/{slug}.media_id")),
        format!("{media_id}\n"),
    )
    .expect("write media_id");
}

#[test]
fn status_text_names_the_media_id_of_a_pushed_draft() {
    let root = workspace("text");
    pushed_draft(&root, "已推", "R1LkThumbAbCdEf");

    let output = run(&root, &["status"]);
    let text = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "status 失败：{text}");
    assert!(
        text.contains("media_id: R1LkThumbAbCdEf"),
        "清单里没有 id，用户没法 update-draft / delete-draft：{text}"
    );
    assert!(
        text.contains("next: moonpub preview Articles/drafts/已推.md"),
        "已推送的文章下一步该去看预览：{text}"
    );
    assert!(
        !text.contains("next: moonpub push"),
        "已经推过的还推 push，会把 media_id 换掉：{text}"
    );

    fs::remove_dir_all(root).ok();
}

#[test]
fn status_json_carries_the_media_id_too() {
    let root = workspace("json");
    pushed_draft(&root, "已推", "R1LkThumbAbCdEf");

    let output = run(&root, &["status", "--json"]);
    let value: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("stdout must be one valid JSON object");

    let files = &value["stages"][0]["files"];
    assert_eq!(files[0]["file"], "已推.md");
    assert_eq!(
        files[0]["media_id"], "R1LkThumbAbCdEf",
        "JSON 里少了 media_id，脚本拿不到 id：{value}"
    );
    assert_eq!(
        value["next_command"], "moonpub preview Articles/drafts/已推.md",
        "{value}"
    );

    fs::remove_dir_all(root).ok();
}

/// 空 `.media_id` 文件不是 id。
///
/// `fs::write` 中途失败会留下空文件；把它当成「已推送」，下一步会跳到 `preview`，
/// 而 `preview` 打开的是一个用户其实还没推上去的包。
#[test]
fn an_empty_media_id_file_does_not_count_as_pushed() {
    let root = workspace("empty-id");
    pushed_draft(&root, "半截", "");
    fs::write(root.join("Articles/drafts/半截.media_id"), "  \n").expect("empty media_id");

    let output = run(&root, &["status"]);
    let text = String::from_utf8_lossy(&output.stdout);

    assert!(!text.contains("media_id:"), "空 id 被当成真 id 了：{text}");
    assert!(
        text.contains("next: moonpub check Articles/drafts/半截.md"),
        "没推成还是该走 check：{text}"
    );

    fs::remove_dir_all(root).ok();
}

/// 看完本地预览，已经推过的文章不该被推回 `push`。
#[test]
fn preview_after_push_points_at_update_draft() {
    let root = workspace("preview");
    pushed_draft(&root, "已推", "R1LkThumbAbCdEf");

    let output = run(&root, &["preview", "Articles/drafts/已推.md", "--no-open"]);
    let text = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success(), "preview 失败：{text}");
    assert!(text.contains("media_id: R1LkThumbAbCdEf"), "{text}");
    assert!(
        text.contains("next: moonpub update-draft"),
        "已推送的文章该指向 update-draft：{text}"
    );
    assert!(
        !text.contains("next: moonpub push"),
        "再 push 会换掉 media_id：{text}"
    );

    fs::remove_dir_all(root).ok();
}

/// 同一条命令，人看到的和脚本拿到的必须是同一个下一步。
///
/// `preview --json` 曾经自己拼 `moonpub push … --render`，而文本版走 `bundle`
/// 的判定——于是已推送的文章，人看到 `update-draft`、脚本拿到 `push`。
/// 判断分散时，出错的那一份总是脚本读的那一份。
#[test]
fn preview_json_and_text_agree_on_the_next_command() {
    let root = workspace("preview-json");
    pushed_draft(&root, "已推", "R1LkThumbAbCdEf");

    let json_out = run(
        &root,
        &["preview", "Articles/drafts/已推.md", "--no-open", "--json"],
    );
    let value: serde_json::Value =
        serde_json::from_slice(&json_out.stdout).expect("stdout must be one valid JSON object");

    let text_out = run(&root, &["preview", "Articles/drafts/已推.md", "--no-open"]);
    let text = String::from_utf8_lossy(&text_out.stdout);
    let text_next = text
        .lines()
        .find_map(|line| line.strip_prefix("next: "))
        .expect("text preview must print a next line");

    assert_eq!(
        value["next_command"], text_next,
        "文本和 JSON 的下一步不一致：{value} / {text}"
    );
    assert!(
        !text_next.contains("moonpub push"),
        "已推送的文章不该被推回 push：{text}"
    );

    fs::remove_dir_all(root).ok();
}
