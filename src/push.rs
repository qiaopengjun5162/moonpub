use std::fs;
use std::path::{Path, PathBuf};

use crate::article::{parse_frontmatter, resolve_article_path, wechat_title};
use crate::config::Config;
use crate::error::AppError;
use crate::render::{build_draft_json, render_article};
use crate::status::{add_status, dir_stage};
use crate::wechat::WechatClient;

pub fn push_article(
    vault: &Path,
    article: &Path,
    auto_render: bool,
    cfg: &Config,
) -> Result<String, AppError> {
    let article = resolve_article_path(vault, article);
    if article.extension().and_then(|e| e.to_str()) != Some("md") {
        return Err(AppError::InvalidArticlePath(article));
    }

    let slug = article
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?
        .to_owned();
    let dir = article
        .parent()
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?
        .to_path_buf();

    let draft_json = dir.join(format!("{slug}.draft.json"));

    // Auto-render if requested and draft.json is missing.
    if !draft_json.exists() {
        if auto_render {
            let author = cfg.wechat_author.as_deref().unwrap_or("作者").to_owned();
            let thumb = cfg
                .wechat_thumb_media_id
                .as_deref()
                .unwrap_or("")
                .to_owned();
            render_article(
                vault,
                &article,
                &author,
                &thumb,
                cfg.wechat_theme.as_deref().unwrap_or("default"),
                None,
                cfg.qrcode_path.as_deref().unwrap_or(""),
            )?;
        } else {
            return Err(AppError::NoDraftJson(draft_json));
        }
    }

    // Credentials: env vars take priority over config file.
    let appid = std::env::var("WECHAT_APPID")
        .ok()
        .or_else(|| cfg.wechat_appid.clone())
        .ok_or(AppError::MissingEnvVar("WECHAT_APPID"))?;
    let secret =
        std::env::var("WECHAT_SECRET").map_err(|_| AppError::MissingEnvVar("WECHAT_SECRET"))?;

    // Call WeChat API directly — no md2wechat dependency.
    let client = WechatClient::new(&appid, &secret);
    let token = client.access_token()?;

    // Upload local images in the HTML and rewrite draft.json before pushing.
    let html_path = dir.join(format!("{slug}.html"));
    let mut uploaded_images = 0usize;
    if html_path.exists() {
        let html = fs::read_to_string(&html_path).map_err(|source| AppError::Io {
            path: html_path.clone(),
            source,
        })?;
        let md = fs::read_to_string(&article).map_err(|source| AppError::Io {
            path: article.clone(),
            source,
        })?;
        let front = parse_frontmatter(&md);
        // Resolve cover regardless of body images: frontmatter `cover` takes priority over config.
        let cover_thumb = crate::render::resolve_cover_thumb(&front, cfg, &dir, &client, &token)?;
        let (updated, img_count) = upload_local_images(&html, &dir, &client, &token)?;
        let needs_rebuild = img_count > 0 || cover_thumb.is_some();
        if needs_rebuild {
            uploaded_images = img_count;
            let html_to_use = if img_count > 0 { &updated } else { &html };
            if img_count > 0 {
                fs::write(&html_path, &updated).map_err(|source| AppError::Io {
                    path: html_path.clone(),
                    source,
                })?;
            }
            let title = wechat_title(&front);
            let digest = front.digest.clone().unwrap_or_else(|| {
                crate::article::first_non_empty_line(crate::article::strip_frontmatter(&md))
                    .to_owned()
            });
            let author = front
                .wechat_author
                .as_deref()
                .unwrap_or_else(|| cfg.wechat_author.as_deref().unwrap_or("作者"));
            let thumb = cover_thumb
                .unwrap_or_else(|| cfg.wechat_thumb_media_id.clone().unwrap_or_default());
            let new_draft = build_draft_json(&title, author, &digest, html_to_use, &thumb);
            fs::write(&draft_json, &new_draft).map_err(|source| AppError::Io {
                path: draft_json.clone(),
                source,
            })?;
        }
    }

    let media_id = client.create_draft(&token, &draft_json)?;

    // Write .media_id file.
    let media_id_path = dir.join(format!("{slug}.media_id"));
    fs::write(&media_id_path, &media_id).map_err(|source| AppError::Io {
        path: media_id_path.clone(),
        source,
    })?;

    // Move article bundle to published/ if currently in drafts/ or ready/.
    let mut moved = String::new();
    if let Some(stage) = dir_stage(&dir)
        && (stage == "drafts" || stage == "ready")
    {
        let published = dir
            .parent()
            .map(|p| p.join("published"))
            .unwrap_or_else(|| dir.join("published"));
        fs::create_dir_all(&published).map_err(|source| AppError::Io {
            path: published.clone(),
            source,
        })?;
        for ext in &["md", "html", "draft.json", "media_id"] {
            let src = dir.join(format!("{slug}.{ext}"));
            if src.exists() {
                let dst = published.join(format!("{slug}.{ext}"));
                fs::rename(&src, &dst).map_err(|source| AppError::Io {
                    path: src.clone(),
                    source,
                })?;
            }
        }
        moved = format!("\n  moved to {}", published.display());
    }

    let _ = add_status(vault, &slug, "pushed", &media_id);
    let img_note = if uploaded_images > 0 {
        format!("\n  images: {uploaded_images} uploaded to WeChat CDN")
    } else {
        String::new()
    };
    let mut result = format!("pushed\n  media_id: {media_id}{moved}{img_note}");

    // Auto-publish for verified/service accounts
    if cfg.wechat_auto_publish {
        let acct_type = cfg.wechat_account_type.as_deref().unwrap_or("personal");
        if acct_type != "personal" {
            match client.free_publish(&token, &media_id) {
                Ok(publish_id) => {
                    let _ = add_status(vault, &slug, "published", &publish_id);
                    result.push_str(&format!(
                        "\n  auto-published ({}): {}",
                        acct_type, publish_id
                    ));
                }
                Err(e) => {
                    result.push_str(&format!("\n  auto-publish failed: {e}"));
                }
            }
        }
    }

    // Browser automation (single call)
    let collection = cfg.wechat_collection.as_deref().unwrap_or("书");
    match crate::publish::auto_configure(&media_id, collection, &[], false) {
        Ok(msg) => result.push_str(&format!("\n  ✓ {msg}")),
        Err(e) => result.push_str(&format!("\n  ⚠ automation: {e}")),
    }

    Ok(result)
}

/// Scan HTML for local `src="..."` img attributes, upload each to WeChat,
/// and return the HTML with those src values replaced by CDN URLs.
/// Remote URLs (http/https) are left untouched.
fn upload_local_images(
    html: &str,
    article_dir: &Path,
    client: &WechatClient,
    token: &str,
) -> Result<(String, usize), AppError> {
    let mut result = html.to_owned();
    let mut search = result.as_str();
    let mut replacements: Vec<(String, String)> = Vec::new();

    while let Some(pos) = search.find("src=\"") {
        let rest = &search[pos + 5..];
        let end = rest.find('"').unwrap_or(rest.len());
        let src = &rest[..end];

        if !src.starts_with("http://")
            && !src.starts_with("https://")
            && !src.is_empty()
            && !replacements.iter().any(|(k, _)| k == src)
        {
            let path = if src.starts_with('/') {
                PathBuf::from(src)
            } else {
                article_dir.join(src)
            };
            if path.exists() {
                let url = client.upload_image_url(token, &path)?;
                replacements.push((src.to_owned(), url));
            }
        }
        search = &search[pos + 5 + end..];
    }

    let count = replacements.len();
    for (src, url) in replacements {
        result = result.replace(&format!("src=\"{src}\""), &format!("src=\"{url}\""));
    }
    Ok((result, count))
}

pub fn update_draft(
    vault: &Path,
    article: &Path,
    media_id_arg: Option<&str>,
    cfg: &Config,
) -> Result<String, AppError> {
    let article = resolve_article_path(vault, article);
    if article.extension().and_then(|e| e.to_str()) != Some("md") {
        return Err(AppError::InvalidArticlePath(article));
    }

    let slug = article
        .file_stem()
        .and_then(|s| s.to_str())
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?
        .to_owned();
    let dir = article
        .parent()
        .ok_or_else(|| AppError::InvalidArticlePath(article.clone()))?
        .to_path_buf();

    let draft_json = dir.join(format!("{slug}.draft.json"));
    if !draft_json.exists() {
        return Err(AppError::NoDraftJson(draft_json));
    }

    // media_id: CLI arg > .media_id file
    let media_id = if let Some(id) = media_id_arg {
        id.to_owned()
    } else {
        let media_id_path = dir.join(format!("{slug}.media_id"));
        fs::read_to_string(&media_id_path)
            .map(|s| s.trim().to_owned())
            .map_err(|_| AppError::PushFailed {
                message: format!(
                    "no media_id found — pass --media-id or ensure {slug}.media_id exists"
                ),
                ip_hint: None,
            })?
    };

    let appid = std::env::var("WECHAT_APPID")
        .ok()
        .or_else(|| cfg.wechat_appid.clone())
        .ok_or(AppError::MissingEnvVar("WECHAT_APPID"))?;
    let secret =
        std::env::var("WECHAT_SECRET").map_err(|_| AppError::MissingEnvVar("WECHAT_SECRET"))?;

    let client = WechatClient::new(&appid, &secret);
    let token = client.access_token()?;
    client.update_draft(&token, &media_id, &draft_json)?;

    Ok(format!(
        "updated draft\n  media_id: {media_id}\n  next: preview in WeChat backend, then publish"
    ))
}

fn wechat_client(cfg: &Config) -> Result<WechatClient, AppError> {
    let appid = std::env::var("WECHAT_APPID")
        .ok()
        .or_else(|| cfg.wechat_appid.clone())
        .ok_or(AppError::MissingEnvVar("WECHAT_APPID"))?;
    let secret =
        std::env::var("WECHAT_SECRET").map_err(|_| AppError::MissingEnvVar("WECHAT_SECRET"))?;
    Ok(WechatClient::new(appid, secret))
}

pub fn list_drafts(cfg: &Config) -> Result<String, AppError> {
    let client = wechat_client(cfg)?;
    let token = client.access_token()?;
    let (items, total) = client.list_drafts(&token, 0, 20)?;
    if items.is_empty() {
        return Ok("草稿箱为空".to_owned());
    }
    let mut out = format!("草稿总数: {total}\n");
    for item in &items {
        out.push_str(&format!("  {} | {}\n", item.media_id, item.title));
    }
    Ok(out.trim_end().to_owned())
}

pub fn delete_draft(media_id: &str, cfg: &Config) -> Result<String, AppError> {
    let client = wechat_client(cfg)?;
    let token = client.access_token()?;
    client.delete_draft(&token, media_id)?;
    Ok(format!("已删除草稿: {media_id}"))
}

#[cfg(test)]
mod tests {
    use crate::config::Config;
    use crate::error::AppError;
    use crate::error::extract_ip_from_message;
    use crate::push::push_article;
    use crate::test_helpers::{create_file, temp_root};

    #[test]
    fn push_fails_without_draft_json() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("push-no-draft")?;
        let md = root.join("Articles/drafts/demo.md");
        create_file(&md, "---\ntitle: T\n---\n\n正文。\n")?;

        let cfg = Config::default();
        let err = push_article(&root, &md, false, &cfg).unwrap_err();
        assert!(
            matches!(err, AppError::NoDraftJson(_)),
            "expected NoDraftJson, got: {err}"
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn push_auto_render_creates_draft_json() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("push-auto-render")?;
        let md = root.join("Articles/drafts/demo.md");
        create_file(&md, "---\ntitle: 自动渲染测试\n---\n\n正文段落。\n")?;

        let cfg = Config {
            wechat_author: Some("寻月隐君".to_owned()),
            wechat_thumb_media_id: Some("thumb_abc".to_owned()),
            ..Config::default()
        };
        let _ = push_article(&root, &md, true, &cfg);

        assert!(
            root.join("Articles/drafts/demo.draft.json").exists(),
            "draft.json should be created by auto-render"
        );

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn extract_ip_from_wechat_error() {
        let msg = "create draft: get access_token error : errcode=40164 , errormsg=invalid ip 1.2.3.4 ipv6";
        assert_eq!(extract_ip_from_message(msg).as_deref(), Some("1.2.3.4"));
    }
}
