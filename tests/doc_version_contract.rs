//! 文档版本口径契约。
//!
//! 两类漂移会让新用户照着文档装到旧版本、或读到过期版本号：
//!
//! ① 活文档里写死 `releases/download/vX.Y.Z` 安装链接 —— 每次发版都会自动过期，
//!    新人照装必拿到旧二进制（2026-11-06 修过一次，漏了 `USER_GUIDE` / `USER_ONBOARDING`）；
//! ② 入口文档自报的「当前源码版本」与 `Cargo.toml` 脱钩（`ROADMAP.md` 曾滞后一个版本）。
//!
//! 这里把两件事变成 CI 能抓的失败，而不是靠发版时想起来。

use std::fs;
use std::path::{Path, PathBuf};

/// 扫描时跳过的目录：构建产物、依赖、VCS 元数据，以及本地记忆库
/// （`.workbuddy/memory` 的历史记录会合法地引用旧版本链接）。
const SKIP_DIRS: &[&str] = &["target", "node_modules", ".git", ".workbuddy"];

/// 历史快照文档：它们的职责就是记录**当时**的版本事实，写死版本号是正确的。
/// 新增白名单必须写清为什么这份文档不能跟着版本走。
fn is_snapshot_doc(rel: &str) -> bool {
    let name = rel.rsplit('/').next().unwrap_or(rel);
    // 发布说明 / release gate 用文件名冻结版本，例如 RELEASE_NOTES_v0.4.1.md。
    if name.starts_with("RELEASE_NOTES_v") || name.starts_with("RELEASE_GATE_v") {
        return true;
    }
    // v0.4.1 对外发布稿是按当次发布定稿的文章，链接指向那一版是内容本身。
    rel == "docs/LAUNCH_ARTICLE_ZH.md"
}

fn collect_docs(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().to_string();
        if path.is_dir() {
            if SKIP_DIRS.contains(&name.as_str()) {
                continue;
            }
            collect_docs(&path, out);
        } else if matches!(
            path.extension().and_then(|ext| ext.to_str()),
            Some("md") | Some("html")
        ) {
            out.push(path);
        }
    }
}

/// 只匹配真实安装链接（带仓库名），不匹配回顾历史写法的片段——
/// 例如 ENGINEERING_LESSONS 里「某文档曾写死 `releases/download/v0.4.2`」是叙述，不是可点链接。
const HARDCODED_LINK: &str = "moonpub/releases/download/v";

/// 取出 `marker` 之后紧跟的 `X.Y.Z` 形态版本号。
fn declared_version<'a>(text: &'a str, marker: &str) -> Option<&'a str> {
    let start = text.find(marker)? + marker.len();
    let rest = &text[start..];
    let end = rest
        .char_indices()
        .find(|(_, ch)| !(ch.is_ascii_digit() || *ch == '.'))
        .map(|(idx, _)| idx)
        .unwrap_or(rest.len());
    match &rest[..end] {
        "" => None,
        version => Some(version),
    }
}

#[test]
fn live_docs_never_hardcode_a_release_version_in_install_links() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut files = Vec::new();
    collect_docs(root, &mut files);
    assert!(
        files.len() > 20,
        "扫描到的文档太少（{}），可疑；检查 collect_docs 的跳过规则",
        files.len()
    );

    let mut violations = Vec::new();
    for path in files {
        let rel = path
            .strip_prefix(root)
            .expect("scanned file lives under the crate root")
            .to_string_lossy()
            .replace('\\', "/");
        if is_snapshot_doc(&rel) {
            continue;
        }
        let text = fs::read_to_string(&path).unwrap_or_default();
        for (idx, line) in text.lines().enumerate() {
            if let Some(pos) = line.find(HARDCODED_LINK) {
                violations.push(format!("{rel}:{}  {}", idx + 1, line[pos..].trim()));
            }
        }
    }

    assert!(
        violations.is_empty(),
        "活文档不能写死 release 版本链接，请改用 releases/latest/download/<asset>：\n{}\n\
         若这份文档确实是历史快照，把它加进 is_snapshot_doc 并写明原因。",
        violations.join("\n")
    );
}

#[test]
fn roadmap_reports_the_crate_version() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let roadmap = fs::read_to_string(root.join("ROADMAP.md")).expect("ROADMAP.md must exist");
    let version = declared_version(&roadmap, "当前源码版本：v")
        .expect("ROADMAP.md 必须用「当前源码版本：vX.Y.Z」声明当前版本");
    assert_eq!(
        version,
        env!("CARGO_PKG_VERSION"),
        "ROADMAP.md 的当前源码版本与 Cargo.toml 不一致：发版抬版本后要同步这一行"
    );
}

#[test]
fn progress_stub_does_not_restate_version_facts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let stub =
        fs::read_to_string(root.join("docs/PROGRESS.md")).expect("docs/PROGRESS.md must exist");
    assert!(
        !stub.contains("当前源码版本"),
        "docs/PROGRESS.md 是给旧链接用的重定向存根，不能再抄一份版本事实——它必然会过期；\
         版本统一看 Cargo.toml / `moonpub --version`，进度统一看根目录 PROGRESS.md"
    );
}

/// 根目录 PROGRESS.md 的「Status / Progress Bar」是当前状态摘要，不是编年史。
/// 写死版本号 / 测试数 / 覆盖率百分比会在下次发版或加测试后立刻过期（曾滞后 v0.4.12
/// 与 625 tests）。这些实时事实统一由 Cargo.toml、`moonpub --version`、
/// `cargo nextest` 和 `./scripts/coverage.sh` 提供；摘要只引用来源，不复述数值。
#[test]
fn progress_summary_never_hardcodes_live_facts() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let text = fs::read_to_string(root.join("PROGRESS.md")).expect("PROGRESS.md must exist");
    // 摘要区截止到「Current Milestone」标题；之后的带日期条目是历史快照，保留版本合法。
    let summary = text
        .split("## Current Milestone")
        .next()
        .expect("PROGRESS.md must have a Current Milestone section");

    let mut violations = Vec::new();
    // 只抓「当前实时事实」的断言句式。摘要区合法地会提到稳定引用（路线图
    // 「v0.5 插件化设计」、历史「v0.4.1 快照 / v0.4.2 归档」），它们不会随发版过期，
    // 不能一刀切禁止所有 vX.Y.Z。
    const LIVE_FACTS: &[&str] = &[
        "is publicly released",
        "publicly released (",
        "tests passed",
        "nextest cases",
        "总行覆盖 ",
        "行覆盖率 ",
    ];
    for (idx, line) in summary.lines().enumerate() {
        for needle in LIVE_FACTS {
            if line.contains(needle) {
                violations.push(format!(
                    "第 {} 行写死实时事实（{}）：{}",
                    idx + 1,
                    needle,
                    line.trim()
                ));
                break;
            }
        }
    }

    assert!(
        violations.is_empty(),
        "PROGRESS.md 摘要区不能写死版本号/测试数/覆盖率，它们每次发版或加测试都会过期；\
         改为引用 GitHub Releases / Cargo.toml / cargo nextest / ./scripts/coverage.sh：\n{}",
        violations.join("\n")
    );
}
