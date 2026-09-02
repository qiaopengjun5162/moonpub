use std::fs;
use std::path::{Path, PathBuf};

use crate::article::{cover_title, parse_frontmatter, resolve_article_path};
use crate::cards;
use crate::config::Config;
use crate::cover;
use crate::error::AppError;
use crate::preview::{preview_article_with_open, preview_paths};
use crate::protocol::preview_json;
use crate::render::render_article;

pub(crate) struct RenderCommand<'a> {
    pub(crate) article: &'a Path,
    pub(crate) author: Option<&'a str>,
    pub(crate) thumb_media_id: Option<&'a str>,
    pub(crate) humanize: bool,
}

pub(crate) struct CoverCommand<'a> {
    pub(crate) article: &'a Path,
    pub(crate) style: Option<&'a str>,
    pub(crate) screenshot: bool,
}

pub(crate) fn run_render_command(
    articles_dir: &Path,
    cfg: &Config,
    command: RenderCommand<'_>,
) -> Result<String, AppError> {
    if command.humanize {
        humanize_article_file(articles_dir, command.article)?;
    }

    let resolved_author = command
        .author
        .or(cfg.wechat_author.as_deref())
        .unwrap_or("作者")
        .to_owned();
    let resolved_thumb = command
        .thumb_media_id
        .or(cfg.wechat_thumb_media_id.as_deref())
        .unwrap_or("")
        .to_owned();
    let theme_name = cfg.wechat_theme.as_deref().unwrap_or("default");
    let mut footer_cfg = cfg.footer.clone();
    if footer_cfg.qrcode.is_empty() {
        footer_cfg.qrcode = cfg.qrcode_path.clone().unwrap_or_default();
    }

    render_article(
        articles_dir,
        command.article,
        &resolved_author,
        &resolved_thumb,
        theme_name,
        None,
        &footer_cfg,
    )
}

pub(crate) fn run_cover_command(
    articles_dir: &Path,
    cfg: &Config,
    command: CoverCommand<'_>,
) -> Result<String, AppError> {
    let article_path = resolve_article_path(articles_dir, command.article);
    let md = fs::read_to_string(&article_path).map_err(|source| AppError::Io {
        path: article_path.clone(),
        source,
    })?;
    let front = parse_frontmatter(&md);
    let title = cover_title(&front, &md, &article_path);
    let digest = front.digest.as_deref().unwrap_or("");
    let author = front
        .wechat_author
        .as_deref()
        .or(cfg.wechat_author.as_deref())
        .unwrap_or("");
    let artifact = cover::write_cover_html(
        &article_path,
        &title,
        digest,
        author,
        cover::style_from_name(command.style),
        front.cover_tag.as_deref(),
    )?;

    let mut result = format!("cover generated\n  {}", artifact.html_path.display());
    if command.screenshot {
        let png = cover::cover_png_path(&article_path);
        if let Some(message) = cover::capture_cover_png(&artifact.html_path, &png) {
            result.push_str(&format!("\n  ({message})"));
        } else {
            result.push_str(&format!("\n  png:   {}", png.display()));
        }
    }
    Ok(result)
}

pub(crate) fn run_humanize_command(
    articles_dir: &Path,
    article: &Path,
) -> Result<String, AppError> {
    let article_path = humanize_article_file(articles_dir, article)?;
    Ok(format!("humanized {}", article_path.display()))
}

pub(crate) fn run_cards_command(
    articles_dir: &Path,
    cfg: &Config,
    article: &Path,
    json: bool,
) -> Result<String, AppError> {
    let result = cards::generate_cards(articles_dir, cfg, article)?;
    if json {
        let payload = serde_json::to_string_pretty(&result)
            .unwrap_or_else(|e| format!("{{\"serialization_error\":\"{e}\"}}"));
        Ok(payload)
    } else {
        Ok(cards::cards_text(&result))
    }
}

pub(crate) fn run_preview_command(
    articles_dir: &Path,
    article: &Path,
    open: bool,
    json: bool,
) -> Result<String, AppError> {
    if json {
        let (article_path, html_path) = preview_paths(articles_dir, article)?;
        let next = format!("moonpub push {} --render", article_path.display());
        Ok(preview_json(&article_path, &html_path, open, &next))
    } else {
        preview_article_with_open(articles_dir, article, open)
    }
}

fn humanize_article_file(articles_dir: &Path, article: &Path) -> Result<PathBuf, AppError> {
    let article_path = resolve_article_path(articles_dir, article);
    let md = fs::read_to_string(&article_path).map_err(|source| AppError::Io {
        path: article_path.clone(),
        source,
    })?;
    let processed = crate::humanize::humanize(&md);
    fs::write(&article_path, &processed).map_err(|source| AppError::Io {
        path: article_path.clone(),
        source,
    })?;
    Ok(article_path)
}

#[cfg(test)]
mod tests {
    use serde_json::Value;

    use super::{run_cards_command, run_humanize_command, run_preview_command};
    use crate::config::Config;
    use crate::test_helpers::{create_file, temp_root};

    #[test]
    fn preview_command_json_reports_paths_without_opening_browser()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("app-preview-json")?;
        let md = root.join("demo.md");
        let html = root.join("demo.html");
        create_file(&md, "---\ntitle: Demo\n---\n正文\n")?;
        create_file(&html, "<p>正文</p>")?;

        let output = run_preview_command(&root, &md, false, true)?;
        let json: Value = serde_json::from_str(&output)?;

        assert_eq!(json["command"], "preview");
        assert_eq!(json["article_path"], md.display().to_string());
        assert_eq!(json["html_path"], html.display().to_string());
        assert_eq!(json["opened_browser"], false);
        assert_eq!(
            json["next_command"],
            format!("moonpub push {} --render", md.display())
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn humanize_command_writes_processed_article() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("app-humanize")?;
        let md = root.join("demo.md");
        create_file(&md, "在当今社会，赋能创作者是至关重要的事情。")?;

        let output = run_humanize_command(&root, &md)?;
        let rewritten = std::fs::read_to_string(&md)?;

        assert_eq!(output, format!("humanized {}", md.display()));
        assert!(!rewritten.contains("在当今社会"));
        assert!(rewritten.contains("帮助创作者"));
        assert!(rewritten.contains("关键的事情"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn cards_command_json_generates_machine_readable_artifacts()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("app-cards-json")?;
        let md = root.join("demo.md");
        create_file(
            &md,
            "---\ntitle: Demo\ntheme: geek-black\ntags: [AI, Rust]\n---\n\n# Demo\n\n## 第一张\n\n正文。",
        )?;

        let output = run_cards_command(&root, &Config::default(), &md, true)?;
        let json: Value = serde_json::from_str(&output)?;

        assert_eq!(json["title"], "Demo");
        assert_eq!(json["theme"], "geek-black");
        assert_eq!(json["accent_color"], "#22C55E");
        assert_eq!(json["card_count"], 1);
        assert!(root.join("demo.card-plan.md").exists());
        assert!(root.join("demo.publish-copy.md").exists());

        std::fs::remove_dir_all(root)?;
        Ok(())
    }
}
