use std::fs;
use std::path::{Path, PathBuf};

use crate::article::{parse_frontmatter, resolve_article_path, wechat_title};
use crate::bundle::{ArticleStage, move_article_bundle};
use crate::config::Config;
use crate::error::AppError;
use crate::plugin::{PublishContext, PublishOutcome, PublishTarget, run_publish_target};
use crate::render::{build_draft_json, render_article};
use crate::status::add_status;
use crate::wechat::WechatClient;

pub struct WechatDraftTarget;

pub struct PushOutput {
    pub media_id: String,
    pub stage: &'static str,
    pub message: String,
}

impl PublishTarget for WechatDraftTarget {
    fn id(&self) -> &'static str {
        "wechat-draft"
    }

    fn display_name(&self) -> &'static str {
        "WeChat Draft"
    }

    fn requires_network(&self) -> bool {
        true
    }

    fn requires_browser(&self) -> bool {
        true
    }

    fn publish(&self, ctx: PublishContext<'_>) -> Result<PublishOutcome, AppError> {
        let message = push_wechat_draft(
            ctx.articles_dir,
            ctx.article,
            ctx.auto_render,
            ctx.temporary_profile,
            ctx.config,
        )?;
        Ok(PublishOutcome { message })
    }
}

pub fn push_article(
    articles_dir: &Path,
    article: &Path,
    auto_render: bool,
    cfg: &Config,
) -> Result<String, AppError> {
    let output = push_article_output(articles_dir, article, auto_render, false, cfg)?;
    Ok(output.message)
}

pub fn push_article_output(
    articles_dir: &Path,
    article: &Path,
    auto_render: bool,
    temporary_profile: bool,
    cfg: &Config,
) -> Result<PushOutput, AppError> {
    let outcome = run_publish_target(
        &WechatDraftTarget,
        PublishContext {
            articles_dir,
            article,
            auto_render,
            temporary_profile,
            config: cfg,
        },
    )?;
    let media_id =
        parse_media_id_from_message(&outcome.message).ok_or_else(|| AppError::PushFailed {
            message: "push output missing media_id".to_owned(),
            ip_hint: None,
        })?;
    Ok(PushOutput {
        media_id,
        stage: "ready",
        message: outcome.message,
    })
}

fn push_wechat_draft(
    articles_dir: &Path,
    article: &Path,
    auto_render: bool,
    temporary_profile: bool,
    cfg: &Config,
) -> Result<String, AppError> {
    let article = resolve_article_path(articles_dir, article);
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
    let media_id_path = dir.join(format!("{slug}.media_id"));
    let old_media_id = previous_media_id(&media_id_path)?;

    // Auto-render if requested and draft.json is missing.
    if !draft_json.exists() {
        if auto_render {
            let author = cfg.wechat_author.as_deref().unwrap_or("作者").to_owned();
            let thumb = cfg
                .wechat_thumb_media_id
                .as_deref()
                .unwrap_or("")
                .to_owned();
            let mut footer_cfg = cfg.footer.clone();
            if footer_cfg.qrcode.is_empty() {
                footer_cfg.qrcode = cfg.qrcode_path.clone().unwrap_or_default();
            }
            render_article(
                articles_dir,
                &article,
                &author,
                &thumb,
                cfg.wechat_theme.as_deref().unwrap_or("default"),
                None,
                &footer_cfg,
            )?;
        } else {
            return Err(AppError::NoDraftJson(draft_json));
        }
    }

    let md = fs::read_to_string(&article).map_err(|source| AppError::Io {
        path: article.clone(),
        source,
    })?;
    let front = parse_frontmatter(&md);
    let draft_title = wechat_title(&front, &md);

    // Auth method: cookie session (no IP whitelist) or appsecret (default).
    let auth_method = cfg
        .wechat_auth_method
        .clone()
        .or_else(|| std::env::var("WECHAT_AUTH_METHOD").ok())
        .unwrap_or_else(|| "appsecret".to_owned());
    if auth_method == "cookie" {
        // Fully delegate to the cookie-session path (bypasses the IP whitelist).
        return crate::push_browser::push_article_cookie(
            articles_dir,
            &article,
            auto_render,
            temporary_profile,
            cfg,
        );
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
            let thumb = cover_thumb
                .unwrap_or_else(|| cfg.wechat_thumb_media_id.clone().unwrap_or_default());
            let new_draft = draft_json_with_thumb(&md, html_to_use, cfg, &thumb);
            fs::write(&draft_json, &new_draft).map_err(|source| AppError::Io {
                path: draft_json.clone(),
                source,
            })?;
        }
    }

    let media_id = client.create_draft(&token, &draft_json)?;

    // Write .media_id file.
    fs::write(&media_id_path, &media_id).map_err(|source| AppError::Io {
        path: media_id_path.clone(),
        source,
    })?;

    let mut bundle_dir = dir.clone();
    let mut moved = String::new();
    if let Some(target) = move_article_bundle(&dir, &slug, ArticleStage::Ready)? {
        moved = format!("\n  moved to {}", target.display());
        bundle_dir = target;
    }

    let _ = add_status(articles_dir, &slug, "ready", &media_id);
    let img_note = if uploaded_images > 0 {
        format!("\n  images: {uploaded_images} uploaded to WeChat CDN")
    } else {
        String::new()
    };
    let mut result = format!(
        "pushed to WeChat draft\n  media_id: {media_id}{moved}{img_note}\n  next: check in WeChat backend, then publish manually"
    );
    if let Some(old_media_id) = old_media_id.filter(|old| old != &media_id) {
        match client.delete_draft(&token, &old_media_id) {
            Ok(()) => result.push_str(&format!("\n  deleted old draft: {old_media_id}")),
            Err(e) => result.push_str(&format!("\n  old draft cleanup failed: {e}")),
        }
    }

    // Auto-publish for verified/service accounts
    if cfg.wechat_auto_publish {
        let acct_type = cfg.wechat_account_type.as_deref().unwrap_or("personal");
        if acct_type != "personal" {
            match client.free_publish(&token, &media_id) {
                Ok(publish_id) => {
                    let _ = add_status(articles_dir, &slug, "published", &publish_id);
                    if let Some(target) =
                        move_article_bundle(&bundle_dir, &slug, ArticleStage::Published)?
                    {
                        result.push_str(&format!("\n  moved to {}", target.display()));
                    }
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
    match crate::publish::auto_configure(
        &media_id,
        collection,
        &[],
        false,
        temporary_profile,
        cfg.template_name.as_deref(),
        None,
        Some(&draft_title),
    ) {
        Ok(msg) => result.push_str(&format!("\n  ✓ {msg}")),
        Err(e) => result.push_str(&format!("\n  ⚠ automation: {e}")),
    }

    Ok(result)
}

fn previous_media_id(media_id_path: &Path) -> Result<Option<String>, AppError> {
    if !media_id_path.exists() {
        return Ok(None);
    }
    let media_id = fs::read_to_string(media_id_path).map_err(|source| AppError::Io {
        path: media_id_path.to_path_buf(),
        source,
    })?;
    let media_id = media_id.trim();
    if media_id.is_empty() {
        Ok(None)
    } else {
        Ok(Some(media_id.to_owned()))
    }
}

fn parse_media_id_from_message(message: &str) -> Option<String> {
    let marker = "media_id: ";
    let start = message.find(marker)? + marker.len();
    let value = message[start..].lines().next()?.trim();
    if value.is_empty() {
        None
    } else {
        Some(value.to_owned())
    }
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
            // Embedded data URIs (e.g. the footer QR code) are stripped by
            // the WeChat editor — upload them to the CDN as well.
            if let Some((filename, data)) = crate::wechat::decode_data_uri(src) {
                match client.upload_image_url_bytes(token, &filename, &data) {
                    Ok(url) => replacements.push((src.to_owned(), url)),
                    Err(e) => eprintln!(
                        "  ⚠ embedded image upload failed ({filename}, {} bytes): {e}",
                        data.len()
                    ),
                }
                search = &search[pos + 5 + end..];
                continue;
            }
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
    articles_dir: &Path,
    article: &Path,
    media_id_arg: Option<&str>,
    cfg: &Config,
) -> Result<String, AppError> {
    let article = resolve_article_path(articles_dir, article);
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

    let md = fs::read_to_string(&article).map_err(|source| AppError::Io {
        path: article.clone(),
        source,
    })?;
    let front = parse_frontmatter(&md);
    let cover_thumb = crate::render::resolve_cover_thumb(&front, cfg, &dir, &client, &token)?;
    let html_path = dir.join(format!("{slug}.html"));
    let html = fs::read_to_string(&html_path).map_err(|source| AppError::Io {
        path: html_path.clone(),
        source,
    })?;
    let (updated_html, uploaded_images) = upload_local_images(&html, &dir, &client, &token)?;
    if uploaded_images > 0 {
        fs::write(&html_path, &updated_html).map_err(|source| AppError::Io {
            path: html_path,
            source,
        })?;
    }
    if uploaded_images > 0 || cover_thumb.is_some() {
        let thumb = cover_thumb
            .or_else(|| cfg.wechat_thumb_media_id.clone())
            .unwrap_or_default();
        let new_draft = draft_json_with_thumb(&md, &updated_html, cfg, &thumb);
        fs::write(&draft_json, new_draft).map_err(|source| AppError::Io {
            path: draft_json.clone(),
            source,
        })?;
    }

    client.update_draft(&token, &media_id, &draft_json)?;

    Ok(format!(
        "updated draft\n  media_id: {media_id}\n  next: preview in WeChat backend, then publish"
    ))
}

fn draft_json_with_thumb(md: &str, html: &str, cfg: &Config, thumb: &str) -> String {
    let front = parse_frontmatter(md);
    let title = wechat_title(&front, md);
    let digest = front.digest.clone().unwrap_or_default();
    let author = front
        .wechat_author
        .as_deref()
        .unwrap_or_else(|| cfg.wechat_author.as_deref().unwrap_or("作者"));
    build_draft_json(&title, author, &digest, html, thumb)
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
    // Cookie-session path bypasses the IP whitelist (errcode 40164) that blocks
    // the appsecret delete endpoint from non-allowlisted egress IPs. Prefer it
    // for numeric AppMsgIds; fall back to appsecret for non-numeric ids.
    if media_id.chars().all(|c| c.is_ascii_digit()) {
        match crate::push_browser::delete_draft_cookie_session(media_id) {
            Ok(msg) => return Ok(msg),
            Err(e) => {
                eprintln!("  ⚠ cookie 会话删除失败 ({e})，回退 appsecret 通道");
            }
        }
    }
    let client = wechat_client(cfg)?;
    let token = client.access_token()?;
    client.delete_draft(&token, media_id)?;
    Ok(format!("已删除草稿: {media_id}"))
}

#[cfg(test)]
mod tests {
    use crate::bundle::{ArticleStage, move_article_bundle};
    use crate::config::Config;
    use crate::error::AppError;
    use crate::error::extract_ip_from_message;
    use crate::plugin::PublishTarget;
    use crate::push::{
        draft_json_with_thumb, parse_media_id_from_message, previous_media_id, push_article,
    };
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
    fn previous_media_id_trims_existing_bundle_id() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("push-previous-media-id")?;
        let media_id = root.join("Articles/ready/demo.media_id");
        create_file(&media_id, " old_media_id \n")?;

        assert_eq!(
            previous_media_id(&media_id)?,
            Some("old_media_id".to_owned())
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
            wechat_author: Some("Test Author".to_owned()),
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
    fn draft_json_with_thumb_sets_fixed_cover_media_id() {
        let cfg = Config {
            wechat_author: Some("配置作者".to_owned()),
            ..Config::default()
        };
        let json = draft_json_with_thumb(
            "---\ntitle: 固定封面\ncover: fixed.png\n---\n\n正文。",
            "<p>正文。</p>",
            &cfg,
            "fixed_thumb_media_id",
        );

        assert!(json.contains("\"thumb_media_id\": \"fixed_thumb_media_id\""));
    }

    #[test]
    fn parse_media_id_from_message_reads_first_media_id_line() {
        let message = "pushed to WeChat draft\n  media_id: abc123\n  moved to Articles/ready\n  next: check in WeChat backend, then publish manually";

        assert_eq!(
            parse_media_id_from_message(message).as_deref(),
            Some("abc123")
        );
    }

    #[test]
    fn extract_ip_from_wechat_error() {
        let msg = "create draft: get access_token error : errcode=40164 , errormsg=invalid ip 1.2.3.4 ipv6";
        assert_eq!(extract_ip_from_message(msg).as_deref(), Some("1.2.3.4"));
    }

    #[test]
    fn wechat_draft_target_reports_capabilities() {
        let target = super::WechatDraftTarget;

        assert_eq!(target.id(), "wechat-draft");
        assert_eq!(target.display_name(), "WeChat Draft");
        assert!(target.requires_network());
        assert!(target.requires_browser());
    }

    #[test]
    fn pushed_bundle_moves_to_ready_not_published() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("push-ready-stage")?;
        let drafts = root.join("Articles/drafts");
        create_file(&drafts.join("demo.md"), "")?;
        create_file(&drafts.join("demo.html"), "")?;
        create_file(&drafts.join("demo.draft.json"), "{}")?;
        create_file(&drafts.join("demo.media_id"), "media_id")?;

        let target = move_article_bundle(&drafts, "demo", ArticleStage::Ready)?.expect("moved");

        assert_eq!(target, root.join("Articles/ready"));
        assert!(root.join("Articles/ready/demo.md").exists());
        assert!(root.join("Articles/ready/demo.html").exists());
        assert!(root.join("Articles/ready/demo.draft.json").exists());
        assert!(root.join("Articles/ready/demo.media_id").exists());
        assert!(!root.join("Articles/published/demo.md").exists());

        std::fs::remove_dir_all(root)?;
        Ok(())
    }
}
