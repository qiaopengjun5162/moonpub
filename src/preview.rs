use std::path::{Path, PathBuf};

use crate::error::AppError;

pub fn preview_article(articles_dir: &Path, article: &Path) -> Result<String, AppError> {
    preview_article_with_open(articles_dir, article, true)
}

pub fn preview_paths(articles_dir: &Path, article: &Path) -> Result<(PathBuf, PathBuf), AppError> {
    let article = crate::article::resolve_article_path(articles_dir, article);
    let slug = article
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?;
    let dir = article
        .parent()
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?;
    let html = dir.join(format!("{slug}.html"));

    if !html.exists() {
        return Err(AppError::NoHtml(html));
    }

    Ok((article, html))
}

pub fn preview_article_with_open(
    articles_dir: &Path,
    article: &Path,
    open_browser: bool,
) -> Result<String, AppError> {
    let (article, html) = preview_paths(articles_dir, article)?;

    let next = next_step_after_preview(articles_dir, &article)?.render_text();
    // 预览打开的是本地文件；印出来的路径要能直接在新终端里 `open`/`cat`，
    // 也要和 `status`、`check` 里的同一篇文章长得一模一样。
    let shown_html = crate::human_path::display_path(articles_dir, &html);

    if !open_browser {
        return Ok(format!("preview ready {shown_html}\n{next}"));
    }

    #[cfg(target_os = "macos")]
    let opener = "open";
    #[cfg(target_os = "linux")]
    let opener = "xdg-open";
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    let opener = "start";

    std::process::Command::new(opener)
        .arg(&html)
        .status()
        .map_err(|source| AppError::Io {
            path: PathBuf::from(opener),
            source,
        })?;

    Ok(format!("opening {shown_html}\n{next}"))
}

/// 看完预览之后的下一步：命令、一句话说明、以及「已经推过」时的 media_id。
pub struct PreviewNext {
    pub command: String,
    pub step: &'static str,
    /// 文章已经在微信草稿箱里时才有。文本版用它多印一行 id，JSON 版只取命令。
    pub media_id: Option<String>,
}

impl PreviewNext {
    /// 文本版的下一步提示（`media_id:` 行只在已推送时出现）。
    pub fn render_text(&self) -> String {
        match &self.media_id {
            Some(media_id) => format!(
                "media_id: {media_id}\nnext: {}\nstep: {}",
                self.command, self.step
            ),
            None => format!("next: {}\nstep: {}", self.command, self.step),
        }
    }
}

/// 预览之后的下一步建议。
///
/// **文本版和 `--json` 版必须共用这一份**：`--json` 那边曾经自己拼
/// `moonpub push … --render`，于是同一条命令，人看到的是 `update-draft`、
/// 脚本拿到的是 `push` —— 而 `push` 不是「更新」：它会新建一份草稿、删掉旧的，
/// `media_id` 随之改变，用户手里那个 id 和微信后台里那篇草稿的链接一起作废。
/// 判断分散的时候，出错的那一份总是脚本读的那一份。
pub fn next_step_after_preview(root: &Path, article: &Path) -> Result<PreviewNext, AppError> {
    let dir = article.parent().unwrap_or(Path::new("."));
    let slug = article
        .file_stem()
        .and_then(|stem| stem.to_str())
        .unwrap_or_default();
    let display = crate::human_path::display_path(root, article);

    if let Some(media_id) = crate::bundle::read_media_id(dir, slug)? {
        return Ok(PreviewNext {
            command: format!("moonpub update-draft {display}"),
            step: "already in the WeChat draft box; update-draft edits it in place (needs appid/secret), push replaces it and changes the media_id",
            media_id: Some(media_id),
        });
    }

    Ok(PreviewNext {
        command: format!("moonpub push {display} --render"),
        step: "push the article to WeChat drafts when the layout looks right",
        media_id: None,
    })
}

#[cfg(test)]
mod tests {
    use crate::error::AppError;
    use crate::preview::{preview_article, preview_article_with_open};
    use crate::test_helpers::{create_file, temp_root};

    #[test]
    fn preview_fails_without_html() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("preview-no-html")?;
        let md = root.join("demo.md");
        create_file(&md, "---\ntitle: T\n---\n正文\n")?;

        let err = preview_article(&root, &md).unwrap_err();
        assert!(matches!(err, AppError::NoHtml(_)), "应报 NoHtml 错误");

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn preview_can_skip_opening_browser() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("preview-no-open")?;
        let md = root.join("demo.md");
        let html = root.join("demo.html");
        create_file(&md, "---\ntitle: T\n---\n正文\n")?;
        create_file(&html, "<p>正文</p>")?;

        let output = preview_article_with_open(&root, &md, false)?;

        // 路径印工作区相对形式：与 `status` 列表逐字相同，可直接粘回下一条命令。
        assert_eq!(
            output,
            "preview ready demo.html\nnext: moonpub push demo.md --render\nstep: push the article to WeChat drafts when the layout looks right"
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    /// 已经推过的文章，看完预览不该再推 `push`。
    ///
    /// `push` 不是「更新」：它会新建一份草稿、再删掉旧的，`media_id` 随之改变，
    /// 用户手里那个 id 和微信后台里那篇草稿的链接一起作废。想改已经在草稿箱里的
    /// 文章，正确动作是 `update-draft`——所以下一步要先把 id 摆出来。
    #[test]
    fn preview_points_at_update_draft_when_a_draft_already_exists()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("preview-pushed")?;
        let md = root.join("demo.md");
        let html = root.join("demo.html");
        create_file(&md, "---\ntitle: T\n---\n正文\n")?;
        create_file(&html, "<p>正文</p>")?;
        create_file(&root.join("demo.media_id"), "R1LkThumb\n")?;

        let output = preview_article_with_open(&root, &md, false)?;

        assert!(
            output.contains("media_id: R1LkThumb"),
            "预览没把 id 给出来：{output}"
        );
        assert!(
            output.contains("next: moonpub update-draft"),
            "已推送的文章不该还指 push：{output}"
        );
        assert!(
            !output.contains("next: moonpub push"),
            "再 push 一次会换掉 media_id：{output}"
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }
}
