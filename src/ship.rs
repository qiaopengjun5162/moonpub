use std::fs;
use std::path::{Path, PathBuf};

use crate::article::{cover_title, parse_frontmatter};
use crate::config::Config;
use crate::cover;
use crate::error::AppError;
use crate::export::export_article;
use crate::push::push_article;
use crate::render::{render_article, resolve_cover_thumb};
use crate::wechat::WechatClient;

pub fn ship_article(
    articles_dir: &Path,
    config_path: Option<&Path>,
    art_path: &Path,
    style: Option<&str>,
) -> Result<String, AppError> {
    let slug = art_path.file_stem().and_then(|s| s.to_str()).unwrap_or("");

    let mut cfg = config_path
        .map(Config::load)
        .transpose()?
        .unwrap_or_default();
    let author = cfg.wechat_author.as_deref().unwrap_or("作者").to_owned();

    let mut results = Vec::new();

    let md = fs::read_to_string(art_path).map_err(|e| AppError::Io {
        path: art_path.to_path_buf(),
        source: e,
    })?;
    let front = parse_frontmatter(&md);
    let cover_html = if should_generate_cover(&front) {
        let cover_title = cover_title(&front, &md, art_path);
        let cover = cover::write_cover_html(
            art_path,
            &cover_title,
            front.digest.as_deref().unwrap_or(""),
            front.author.as_deref().unwrap_or(&author),
            cover::style_from_name(style),
        )?;
        results.push(format!("cover:  {}", cover.html_path.display()));

        let cover_png = cover::cover_png_path(art_path);
        if cover::capture_cover_png(&cover.html_path, &cover_png).is_none() {
            let appid = std::env::var("WECHAT_APPID")
                .ok()
                .or_else(|| cfg.wechat_appid.clone())
                .ok_or(AppError::MissingEnvVar("WECHAT_APPID"))?;
            let secret = std::env::var("WECHAT_SECRET")
                .map_err(|_| AppError::MissingEnvVar("WECHAT_SECRET"))?;
            let client = WechatClient::new(&appid, &secret);
            let token = client.access_token()?;
            match client.upload_image(&token, &cover_png) {
                Ok(media_id) => {
                    results.push(format!("thumb:  {media_id}"));
                    cfg.wechat_thumb_media_id = Some(media_id);
                }
                Err(e) => {
                    results.push(format!("⚠ cover upload failed: {e}"));
                }
            }
        }
        Some(cover.html)
    } else {
        let appid = std::env::var("WECHAT_APPID")
            .ok()
            .or_else(|| cfg.wechat_appid.clone())
            .ok_or(AppError::MissingEnvVar("WECHAT_APPID"))?;
        let secret =
            std::env::var("WECHAT_SECRET").map_err(|_| AppError::MissingEnvVar("WECHAT_SECRET"))?;
        let client = WechatClient::new(&appid, &secret);
        let token = client.access_token()?;
        if let Some(media_id) = resolve_cover_thumb(
            &front,
            &cfg,
            art_path.parent().unwrap_or_else(|| Path::new(".")),
            &client,
            &token,
        )? {
            results.push(format!(
                "cover:  {}",
                front.cover.as_deref().unwrap_or_default()
            ));
            results.push(format!("thumb:  {media_id}"));
            cfg.wechat_thumb_media_id = Some(media_id);
        }
        None
    };

    let thumb = cfg
        .wechat_thumb_media_id
        .as_deref()
        .unwrap_or("")
        .to_owned();
    let mut footer_cfg = cfg.footer.clone();
    if footer_cfg.qrcode.is_empty() {
        footer_cfg.qrcode = cfg.qrcode_path.clone().unwrap_or_default();
    }
    results.push(render_article(
        articles_dir,
        art_path,
        &author,
        &thumb,
        cfg.wechat_theme.as_deref().unwrap_or("default"),
        cover_html.as_deref(),
        &footer_cfg,
    )?);
    results.push(push_article(articles_dir, art_path, false, &cfg)?);

    if let Some(br) = cfg.blog_root.as_deref() {
        let src = export_source_for_ship(articles_dir, slug, art_path);
        results.push(export_article(articles_dir, &src, br)?);
    }
    Ok(results.join("\n\n"))
}

fn export_source_for_ship(articles_dir: &Path, slug: &str, current_article: &Path) -> PathBuf {
    let published = articles_dir
        .join("Articles/published")
        .join(slug)
        .with_extension("md");
    if published.exists() {
        published
    } else {
        current_article.to_path_buf()
    }
}

fn should_generate_cover(front: &crate::article::Frontmatter) -> bool {
    front.cover.is_none()
}

#[cfg(test)]
mod tests {
    use super::{export_source_for_ship, should_generate_cover};
    use crate::article::Frontmatter;
    use std::fs;

    fn temp_root(name: &str) -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_nanos();
        let root = std::env::temp_dir().join(format!("moonpub-ship-{name}-{nanos}"));
        fs::create_dir_all(root.join("Articles/drafts"))?;
        fs::create_dir_all(root.join("Articles/published"))?;
        Ok(root)
    }

    #[test]
    fn export_source_prefers_published_article_when_available()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("published")?;
        let draft = root.join("Articles/drafts/demo.md");
        let published = root.join("Articles/published/demo.md");
        fs::write(&draft, "# draft")?;
        fs::write(&published, "# published")?;

        assert_eq!(export_source_for_ship(&root, "demo", &draft), published);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn export_source_falls_back_to_current_article() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("draft")?;
        let draft = root.join("Articles/drafts/demo.md");
        fs::write(&draft, "# draft")?;

        assert_eq!(export_source_for_ship(&root, "demo", &draft), draft);

        fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn ship_skips_generated_cover_when_frontmatter_cover_is_set() {
        let front = Frontmatter {
            cover: Some("fixed-cover.png".to_owned()),
            ..Frontmatter::default()
        };

        assert!(!should_generate_cover(&front));
    }

    #[test]
    fn ship_generates_cover_when_frontmatter_cover_is_missing() {
        assert!(should_generate_cover(&Frontmatter::default()));
    }
}
