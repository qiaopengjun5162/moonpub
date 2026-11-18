//! 印给人看的路径。
//!
//! 工具自己打印出来的路径必须能原样粘回下一条命令。此前 `new` / `render` /
//! `check` / `preview` 印的是绝对路径，`status` 印的是工作区相对路径——同一个
//! 工具、同一篇文章，两种写法：用户从 `render` 的 `next:` 里复制出来的那串
//! 东西，和 `status` 列表里看到的那串东西对不上，照着敲还会因为换行、
//! 空格、中文被截断而报 `No such file or directory`。
//!
//! macOS 上还有第二层问题：绝对路径要经过符号链接规范化，`/tmp/x` 会被印成
//! `/private/tmp/x`。用户自己敲的是前者，工具印的是后者，两边字符串永远不等，
//! 目视比对和 `diff` 都没法用。
//!
//! 所以只有一套口径：**只要路径落在工作区里，就印工作区相对形式**
//! （`Articles/drafts/x.md`），与 `status` 列表完全一致。落在工作区外面
//! （用户显式传了别处的绝对路径）就原样印绝对路径——把它裁成一串 `../../`
//! 只会更难用，也更难核对。

use std::path::Path;

/// 把 `path` 印成工作区 `root` 下的相对路径；不在工作区内则原样返回。
pub(crate) fn display_path(root: &Path, path: &Path) -> String {
    match relative_to(root, path) {
        Some(relative) => join_components(relative),
        None => path.display().to_string(),
    }
}

fn relative_to<'a>(root: &Path, path: &'a Path) -> Option<&'a Path> {
    let relative = path.strip_prefix(root).ok()?;
    // `root` 自己（相对形式为空）不该被印成空字符串。
    if relative.as_os_str().is_empty() {
        return None;
    }
    Some(relative)
}

/// 统一用 `/` 分隔。
///
/// `Path::display` 在 Windows 上印反斜杠，于是同一条命令在 macOS 与 Windows
/// 上长得不一样：复制进文档、工单、聊天记录之后就成了两种写法，而它们其实是
/// 同一条命令。工具自己输出的是路径的「人类形式」，不是要被 `PathBuf` 解析的
/// 系统形式——`moonpub render Articles/drafts/x.md` 在两边都能跑。
fn join_components(path: &Path) -> String {
    path.components()
        .map(|component| component.as_os_str().to_string_lossy().into_owned())
        .collect::<Vec<_>>()
        .join("/")
}

#[cfg(test)]
mod tests {
    use crate::human_path::display_path;
    use std::path::Path;

    #[test]
    fn workspace_path_is_printed_without_the_workspace_prefix() {
        assert_eq!(
            display_path(
                Path::new("/tmp/demo"),
                Path::new("/tmp/demo/Articles/drafts/x.md")
            ),
            "Articles/drafts/x.md"
        );
    }

    #[test]
    fn a_path_outside_the_workspace_keeps_its_absolute_form() {
        // 用户显式渲染别处的文章是合法用法；裁成 `../other/x.md` 更难用。
        assert_eq!(
            display_path(Path::new("/tmp/demo"), Path::new("/other/place/x.md")),
            "/other/place/x.md"
        );
    }

    #[test]
    fn a_relative_workspace_root_still_yields_a_clean_relative_path() {
        // `moonpub status` 在 `--articles .` 下拿到的是相对根；`./` 不该漏进输出。
        assert_eq!(
            display_path(Path::new("."), Path::new("./Articles/drafts/x.md")),
            "Articles/drafts/x.md"
        );
    }

    #[test]
    fn a_sibling_directory_with_the_same_prefix_is_not_mistaken_for_a_child() {
        // `/tmp/demo2` 的字符串前缀是 `/tmp/demo`，但它不是工作区里的文件。
        assert_eq!(
            display_path(Path::new("/tmp/demo"), Path::new("/tmp/demo2/x.md")),
            "/tmp/demo2/x.md"
        );
    }

    #[test]
    fn the_workspace_root_itself_is_not_printed_as_an_empty_string() {
        assert_eq!(
            display_path(Path::new("/tmp/demo"), Path::new("/tmp/demo")),
            "/tmp/demo"
        );
    }

    #[test]
    fn nested_directories_are_joined_with_forward_slashes() {
        assert_eq!(
            display_path(
                Path::new("/tmp/demo"),
                Path::new("/tmp/demo/Articles/ready/deep/x.md")
            ),
            "Articles/ready/deep/x.md"
        );
    }
}
