//! 平台相关的外部依赖探测：目前只有 Chrome 可执行文件查找。
//!
//! 这里刻意把「候选清单 + 命中顺序」与「怎么判断一个候选是否存在」拆开：
//! 前者是跨平台的产品契约（macOS 的 app bundle 路径优先于 Linux 的命令名），
//! 后者要碰文件系统和 `which` / `where`，没法在单测里断言。
//! `find_chrome_with()` 收一个探测函数进来，让顺序契约可以用 fake 验证。

/// Chrome / Chromium 候选，按优先级排列。
///
/// 顺序即契约：macOS 的 app bundle 绝对路径排在最前，其次是 Linux 的命令名，
/// 最后是 Windows 的安装路径。命中即返回，不再往下试。
const CHROME_CANDIDATES: [&str; 10] = [
    // macOS
    "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome",
    "/Applications/Chromium.app/Contents/MacOS/Chromium",
    // Linux
    "google-chrome",
    "google-chrome-stable",
    "chromium",
    "chromium-browser",
    // Windows
    r"C:\Program Files\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files (x86)\Google\Chrome\Application\chrome.exe",
    r"C:\Program Files\Microsoft\Edge\Application\msedge.exe",
    r"C:\Program Files (x86)\Microsoft\Edge\Application\msedge.exe",
];

/// 在候选清单里找第一个真实存在的 Chrome，返回可执行的路径或命令名。
pub(crate) fn find_chrome() -> Option<String> {
    if let Some(found) = find_chrome_with(default_probe) {
        return Some(found);
    }
    // Windows：再看用户级安装（LOCALAPPDATA），这两条不在静态候选清单里。
    #[cfg(windows)]
    {
        if let Ok(local) = std::env::var("LOCALAPPDATA") {
            let extra = [
                r"Google\Chrome\Application\chrome.exe",
                r"Microsoft\Edge\Application\msedge.exe",
            ];
            for e in &extra {
                let p = std::path::PathBuf::from(&local).join(e);
                if p.exists() {
                    return Some(p.to_string_lossy().into_owned());
                }
            }
        }
    }
    None
}

/// 按 [`CHROME_CANDIDATES`] 的顺序返回第一个 `probe` 认账的候选。
///
/// 探测动作由调用方注入：生产路径传 [`default_probe`]，单测传 fake，
/// 这样「顺序」这条契约不必依赖运行机器上装没装 Chrome。
fn find_chrome_with(probe: impl Fn(&str) -> bool) -> Option<String> {
    CHROME_CANDIDATES
        .iter()
        .find(|candidate| probe(candidate))
        .map(|candidate| (*candidate).to_string())
}

/// 真实探测：绝对路径查文件是否存在，裸命令名查 `which` / `where`。
///
/// 裸命令名这一支用 `#[cfg]` 做**编译期**切分而不是 `if cfg!(windows)`：
/// 后者两个分支在两端都会编译，非本平台的 `where` / `which` 调用永远执行不到，
/// 在覆盖率报告里留下一条永远红的假分支。
fn default_probe(candidate: &str) -> bool {
    if candidate.starts_with('/') || candidate.starts_with(r"C:\") {
        return std::path::Path::new(candidate).exists();
    }
    #[cfg(windows)]
    {
        command_succeeds("where", candidate)
    }
    #[cfg(not(windows))]
    {
        command_succeeds("which", candidate)
    }
}

fn command_succeeds(program: &str, arg: &str) -> bool {
    std::process::Command::new(program)
        .arg(arg)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn returns_none_when_no_candidate_is_present() {
        assert_eq!(find_chrome_with(|_| false), None);
    }

    #[test]
    fn returns_the_first_matching_candidate() {
        let found = find_chrome_with(|candidate| candidate.contains("Chromium"))
            .expect("Chromium 候选应当命中");
        assert_eq!(found, "/Applications/Chromium.app/Contents/MacOS/Chromium");
    }

    #[test]
    fn mac_app_bundle_paths_outrank_linux_command_names() {
        // 全命中时返回第一条，说明 macOS 的 app bundle 优先级没有被 Linux 命令名盖过。
        let found = find_chrome_with(|_| true).expect("全命中时应当有返回值");
        assert_eq!(
            found,
            "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"
        );
        // Chrome 缺失但 Chromium 在时，应退化到 Chromium，而不是去试 which google-chrome。
        let fallback = find_chrome_with(|candidate| !candidate.contains("Google Chrome.app"))
            .expect("Chromium 候选应当命中");
        assert_eq!(
            fallback,
            "/Applications/Chromium.app/Contents/MacOS/Chromium"
        );
    }

    #[test]
    fn candidate_list_covers_every_platform_and_has_no_duplicates() {
        let mut seen = std::collections::HashSet::new();
        for candidate in CHROME_CANDIDATES {
            assert!(seen.insert(candidate), "候选重复：{candidate}");
        }
        assert!(
            CHROME_CANDIDATES
                .iter()
                .any(|c| c.starts_with("/Applications/")),
            "缺 macOS app bundle 候选"
        );
        assert!(
            CHROME_CANDIDATES
                .iter()
                .any(|c| c.contains("Program Files")),
            "缺 Windows 安装路径候选"
        );
        assert!(
            CHROME_CANDIDATES
                .iter()
                .any(|c| !c.starts_with('/') && !c.contains("Program Files")),
            "缺 Linux 裸命令名候选"
        );
    }

    #[test]
    fn default_probe_checks_absolute_paths_on_disk() {
        // 根目录一定存在，伪造的绝对路径一定为假：覆盖「绝对路径走 exists」这一支。
        assert!(default_probe("/"));
        assert!(!default_probe("/definitely-not-a-real-chrome-binary-9f3a"));
    }

    #[test]
    #[cfg(not(windows))]
    fn default_probe_resolves_bare_command_names_through_path() {
        // `sh` 在任何 POSIX 环境的 PATH 里；乱写的名字应当为假。
        assert!(default_probe("sh"));
        assert!(!default_probe("definitely-not-a-real-chrome-9f3a"));
    }

    #[test]
    fn find_chrome_returns_a_non_empty_path_when_it_finds_one() {
        // 不断言「一定找到」——CI 上可能没装 Chrome；只断言返回值形状。
        if let Some(path) = find_chrome() {
            assert!(!path.trim().is_empty());
        }
    }
}
