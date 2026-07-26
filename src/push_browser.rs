//! Cookie-session WeChat push — bypasses the IP whitelist.
//!
//! The default push path exchanges the appsecret for an `access_token` via
//! `api.weixin.qq.com/cgi-bin/token`. That endpoint enforces an IP whitelist
//! (errcode 40164), so it only works from an allowlisted egress IP.
//!
//! This module reuses the browser session created by `moonpub login`
//! (`~/.config/moonpub/session.json`), extracts the web-console `token` from
//! the logged-in URL, and calls the *same* draft endpoints on
//! `mp.weixin.qq.com` using the saved cookies. No IP whitelist is involved,
//! so it works from any network — at the cost of needing a valid login
//! session (re-run `moonpub login` when the cookies expire).

use std::fs;
use std::path::{Path, PathBuf};

use serde_json::Value;

use crate::article::{parse_frontmatter, wechat_title};
use crate::bundle::{ArticleStage, move_article_bundle};
use crate::cdp::{BrowserProfileMode, open_browser, session_file_for, try_restore_session};
use crate::config::Config;
use crate::error::AppError;
use crate::render::{build_draft_json, render_article};
use crate::status::add_status;

/// Entry point used by `push.rs`. Spawns its own Tokio runtime so it stays
/// synchronous and preserves the structured `AppError` (cookie-session failures
/// surface as `AppError::CookieSessionRequired`).
pub fn push_article_cookie(
    articles_dir: &Path,
    article: &Path,
    auto_render: bool,
    temporary_profile: bool,
    cfg: &Config,
) -> Result<String, AppError> {
    tokio::runtime::Runtime::new()
        .map_err(|e| AppError::AutomationFailed {
            message: format!("tokio runtime: {e}"),
        })?
        .block_on(push_wechat_draft_cookie(
            articles_dir,
            article,
            auto_render,
            temporary_profile,
            cfg,
        ))
        .map(|(result, media_id, title)| {
            // Mirror the appsecret push path: backend automation runs after the
            // draft exists, and failures stay soft so the push result stands.
            let collection = cfg.wechat_collection.as_deref().unwrap_or("书");
            match crate::publish::auto_configure(
                &media_id,
                collection,
                &[],
                false,
                temporary_profile,
                cfg.template_name.as_deref(),
                None,
                Some(&title),
            ) {
                Ok(msg) => format!("{result}\n  ✓ {msg}"),
                Err(e) => format!("{result}\n  ⚠ automation: {e}"),
            }
        })
}

async fn push_wechat_draft_cookie(
    articles_dir: &Path,
    article: &Path,
    auto_render: bool,
    temporary_profile: bool,
    cfg: &Config,
) -> Result<(String, String, String), AppError> {
    let article = crate::article::resolve_article_path(articles_dir, article);
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

    if !draft_json.exists() {
        if auto_render {
            let author = cfg.wechat_author.as_deref().unwrap_or("作者").to_owned();
            let thumb = cfg.wechat_thumb_media_id.clone().unwrap_or_default();
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

    // ── browser session (reuses the login saved by `moonpub login`) ──
    let mode = BrowserProfileMode::from_temporary_flag(temporary_profile);
    let session_path = match session_file_for(&mode) {
        Some(p) if p.exists() => p,
        _ => {
            return Err(AppError::CookieSessionRequired {
                message: "session.json 不存在，cookie 模式无法推送".to_owned(),
            });
        }
    };

    let session = open_browser(true, &mode)
        .await
        .map_err(|e| AppError::AutomationFailed { message: e })?;
    let browser = session.browser;
    let page = session.page;
    if !try_restore_session(&browser, &page, &mode).await {
        return Err(AppError::CookieSessionRequired {
            message: "cookie 会话已过期或失效（跳转到了登录页），请先 `moonpub login` 重新扫码"
                .to_owned(),
        });
    }

    let url = page.url().await.unwrap_or(None).unwrap_or_default();
    let token = url
        .split("token=")
        .nth(1)
        .and_then(|t| t.split('&').next())
        .unwrap_or("")
        .to_owned();
    if token.is_empty() {
        return Err(AppError::CookieSessionRequired {
            message: "未能从后台 URL 提取 token".to_owned(),
        });
    }

    // Build the cookie header from session.json; the HTTP calls below use ureq
    // + these cookies (independent of the browser process).
    let cookie_header = build_cookie_header(&session_path)?;
    drop(browser);

    // ── read content + frontmatter ──
    let md = fs::read_to_string(&article).map_err(|source| AppError::Io {
        path: article.clone(),
        source,
    })?;
    let front = parse_frontmatter(&md);
    let title = wechat_title(&front, &md);
    let digest = front.digest.clone().unwrap_or_default();
    let author = front
        .wechat_author
        .as_deref()
        .or(cfg.wechat_author.as_deref())
        .unwrap_or("作者")
        .to_owned();
    let thumb = cfg.wechat_thumb_media_id.clone().unwrap_or_default();
    // Upload article-specific cover image if one was generated or downloaded by ship.
    let cover_img = crate::cover::cover_image_path(&article);
    let mut cover_url = String::new();
    let mut cover_fileid = String::new();
    if let Some(ref path) = cover_img {
        match upload_image_material_cookie(&token, &cookie_header, path) {
            Ok(up) => {
                cover_url = up.url.unwrap_or_default();
                cover_fileid = up.fileid.unwrap_or_default();
            }
            Err(e) => {
                eprintln!("  ⚠ cover upload failed: {e}");
            }
        }
    }

    let html_path = dir.join(format!("{slug}.html"));
    let html = if html_path.exists() {
        fs::read_to_string(&html_path).map_err(|source| AppError::Io {
            path: html_path.clone(),
            source,
        })?
    } else {
        String::new()
    };
    let (html, uploaded_images) = upload_local_images_cookie(&html, &dir, &token, &cookie_header)?;

    let draft = build_draft_json(&title, &author, &digest, &html, &thumb);
    fs::write(&draft_json, &draft).map_err(|source| AppError::Io {
        path: draft_json.clone(),
        source,
    })?;

    let media_id = create_draft_cookie(&token, &cookie_header, &draft, &cover_url, &cover_fileid)?;
    fs::write(&media_id_path, &media_id).map_err(|source| AppError::Io {
        path: media_id_path.clone(),
        source,
    })?;

    let mut moved = String::new();
    if let Some(target) = move_article_bundle(&dir, &slug, ArticleStage::Ready)? {
        moved = format!("\n  moved to {}", target.display());
    }
    let _ = add_status(articles_dir, &slug, "ready", &media_id);
    let img_note = if uploaded_images > 0 {
        format!("\n  images: {uploaded_images} uploaded to WeChat CDN (cookie mode)")
    } else {
        String::new()
    };
    let mut result = format!(
        "pushed to WeChat draft (cookie session, no IP whitelist)\n  media_id: {media_id}{moved}{img_note}\n  next: check in WeChat backend, then publish manually"
    );
    if let Some(old) = old_media_id.filter(|o| o != &media_id) {
        match delete_draft_cookie(&token, &cookie_header, &old) {
            Ok(()) => result.push_str(&format!("\n  deleted old draft: {old}")),
            Err(e) => result.push_str(&format!("\n  old draft cleanup failed: {e}")),
        }
    }

    // Auto-publish is only supported on the appsecret path today. Cookie-session
    // mode would need a web-console publish endpoint or browser click-through;
    // make the limitation explicit instead of silently ignoring the flag.
    if cfg.wechat_auto_publish {
        let acct_type = cfg.wechat_account_type.as_deref().unwrap_or("personal");
        if acct_type != "personal" {
            result.push_str(&format!(
                "\n  ⚠ auto-publish skipped: not yet supported in cookie mode (account type: {acct_type})"
            ));
        }
    }

    Ok((result, media_id, title))
}

// ── helpers ───────────────────────────────────────────────────────────────────

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

/// Build a `Cookie:` header value from the saved `session.json`, keeping only
/// `weixin.qq.com` cookies.
fn build_cookie_header(session_path: &Path) -> Result<String, AppError> {
    let json = fs::read_to_string(session_path).map_err(|source| AppError::Io {
        path: session_path.to_path_buf(),
        source,
    })?;
    let cookies: Vec<Value> = serde_json::from_str(&json).map_err(|e| AppError::PushFailed {
        message: format!("session.json parse failed: {e}"),
        ip_hint: None,
    })?;
    let mut parts: Vec<String> = Vec::new();
    for c in cookies {
        let name = c["name"].as_str().unwrap_or("");
        let value = c["value"].as_str().unwrap_or("");
        let domain = c["domain"].as_str().unwrap_or("");
        if domain.contains("weixin.qq.com") && !name.is_empty() {
            parts.push(format!("{name}={value}"));
        }
    }
    if parts.is_empty() {
        return Err(AppError::CookieSessionRequired {
            message: "session.json 中无 weixin.qq.com 的 cookie".to_owned(),
        });
    }
    Ok(parts.join("; "))
}

/// Percent-encode a string for `application/x-www-form-urlencoded` bodies.
fn form_escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(b as char)
            }
            _ => out.push_str(&format!("%{:02X}", b)),
        }
    }
    out
}

fn cookie_agent() -> ureq::Agent {
    ureq::AgentBuilder::new().build()
}

/// POST `application/x-www-form-urlencoded` to a `mp.weixin.qq.com` endpoint
/// with the saved cookies. Returns the parsed JSON response.
fn post_form_with_cookies(
    url: &str,
    form: &[(&str, String)],
    cookie: &str,
) -> Result<Value, AppError> {
    let body: String = form
        .iter()
        .map(|(k, v)| format!("{}={}", form_escape(k), form_escape(v)))
        .collect::<Vec<_>>()
        .join("&");
    let resp = cookie_agent()
        .post(url)
        .set(
            "Content-Type",
            "application/x-www-form-urlencoded; charset=utf-8",
        )
        .set("Cookie", cookie)
        .set("Referer", "https://mp.weixin.qq.com/")
        .set("X-Requested-With", "XMLHttpRequest")
        .send_string(&body)
        .map_err(|e| AppError::PushFailed {
            message: format!("http_post: {e}"),
            ip_hint: None,
        })?
        .into_string()
        .map_err(|e| AppError::PushFailed {
            message: format!("http_read: {e}"),
            ip_hint: None,
        })?;
    serde_json::from_str(&resp).map_err(|e| AppError::PushFailed {
        message: format!("json parse failed: {e}\n  raw: {resp}"),
        ip_hint: None,
    })
}

fn mime_for(filename: &str) -> &'static str {
    let lower = filename.to_lowercase();
    if lower.ends_with(".png") {
        "image/png"
    } else if lower.ends_with(".gif") {
        "image/gif"
    } else if lower.ends_with(".webp") {
        "image/webp"
    } else {
        "image/jpeg"
    }
}

/// Upload a local article image to the WeChat image library via the
/// web-console `upload_material` endpoint (the `upload_mass_image` action
/// returns ret=200002 here). Returns the CDN URL usable for inline images.
fn upload_image_cookie(token: &str, cookie: &str, image_path: &Path) -> Result<String, AppError> {
    let filename = image_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let lower = filename.to_lowercase();
    if !lower.ends_with(".jpg") && !lower.ends_with(".jpeg") && !lower.ends_with(".png") {
        return Err(AppError::PushFailed {
            message: format!("{filename}: 仅支持 jpg/png 图片"),
            ip_hint: None,
        });
    }
    let size = fs::metadata(image_path).map(|m| m.len()).unwrap_or(0);
    if size > 1024 * 1024 {
        return Err(AppError::PushFailed {
            message: format!("{filename}: {size} 字节超过 1MB 限制"),
            ip_hint: None,
        });
    }
    let data = fs::read(image_path).map_err(|source| AppError::Io {
        path: image_path.to_path_buf(),
        source,
    })?;
    let outcome = upload_image_material_bytes(token, cookie, filename, &data)?;
    outcome.url.ok_or_else(|| AppError::PushFailed {
        message: format!("{filename}: upload_material 响应缺少 CDN URL"),
        ip_hint: None,
    })
}

/// Upload a local image as permanent material for use as the draft cover.
fn upload_image_material_cookie(
    token: &str,
    cookie: &str,
    image_path: &Path,
) -> Result<UploadOutcome, AppError> {
    let filename = image_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("");
    let lower = filename.to_lowercase();
    if !lower.ends_with(".jpg") && !lower.ends_with(".jpeg") && !lower.ends_with(".png") {
        return Err(AppError::PushFailed {
            message: format!("{filename}: 仅支持 jpg/png 图片"),
            ip_hint: None,
        });
    }
    let size = fs::metadata(image_path).map(|m| m.len()).unwrap_or(0);
    if size > 1024 * 1024 {
        return Err(AppError::PushFailed {
            message: format!("{filename}: {size} 字节超过 1MB 限制"),
            ip_hint: None,
        });
    }
    let data = fs::read(image_path).map_err(|source| AppError::Io {
        path: image_path.to_path_buf(),
        source,
    })?;
    upload_image_material_bytes(token, cookie, filename, &data)
}

/// Upload raw image bytes as permanent material for use as the draft cover.
/// Uses the `upload_material` endpoint which returns a numeric fileid and
/// optionally a CDN URL.
fn upload_image_material_bytes(
    token: &str,
    cookie: &str,
    filename: &str,
    data: &[u8],
) -> Result<UploadOutcome, AppError> {
    let raw = upload_image_to_endpoint(
        token,
        cookie,
        filename,
        data,
        "upload_material",
        "type=image&writetype=doublewrite&groupid=1",
    )?;
    parse_material_response(&raw)
}

fn upload_image_to_endpoint(
    token: &str,
    cookie: &str,
    filename: &str,
    data: &[u8],
    action: &str,
    query: &str,
) -> Result<String, AppError> {
    if data.len() > 1024 * 1024 {
        return Err(AppError::PushFailed {
            message: format!("{filename}: {} 字节超过 1MB 限制", data.len()),
            ip_hint: None,
        });
    }
    let mime = mime_for(filename);
    let boundary = "moonpub_cookie_boundary_7";
    let mut form = Vec::new();
    form.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"file\"; filename=\"{filename}\"\r\nContent-Type: {mime}\r\n\r\n"
        )
        .as_bytes(),
    );
    form.extend_from_slice(data);
    form.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

    let url = format!(
        "https://mp.weixin.qq.com/cgi-bin/filetransfer?token={token}&lang=zh_CN&f=json&ajax=1&action={action}&{query}"
    );
    let resp = cookie_agent()
        .post(&url)
        .set(
            "Content-Type",
            &format!("multipart/form-data; boundary={boundary}"),
        )
        .set("Cookie", cookie)
        .set("Referer", "https://mp.weixin.qq.com/")
        .set("X-Requested-With", "XMLHttpRequest")
        .send_bytes(&form)
        .map_err(|e| AppError::PushFailed {
            message: format!("upload_image: {e}"),
            ip_hint: None,
        })?
        .into_string()
        .map_err(|e| AppError::PushFailed {
            message: format!("upload_image read: {e}"),
            ip_hint: None,
        })?;
    Ok(resp)
}

fn parse_material_response(resp: &str) -> Result<UploadOutcome, AppError> {
    let v: Value = serde_json::from_str(resp).map_err(|e| AppError::PushFailed {
        message: format!("upload_image json: {e}\n  raw: {resp}"),
        ip_hint: None,
    })?;
    let ret = v["base_resp"]["ret"].as_i64().unwrap_or(-1);
    if ret != 0 {
        return Err(AppError::PushFailed {
            message: format!(
                "upload_image: ret={ret} {}",
                v["base_resp"]["err_msg"].as_str().unwrap_or("")
            ),
            ip_hint: None,
        });
    }
    let url = ["cdn_url", "cdn_url_235_1", "cdn_url_1_1", "url"]
        .iter()
        .filter_map(|k| v[k].as_str())
        .find(|s| s.starts_with("http"))
        .map(str::to_owned);
    let fileid = v["content"]
        .as_str()
        .filter(|s| s.chars().all(|c| c.is_ascii_digit()))
        .map(str::to_owned)
        .or_else(|| {
            v["fileid"]
                .as_str()
                .filter(|s| s.chars().all(|c| c.is_ascii_digit()))
                .map(str::to_owned)
        });
    if url.is_none() && fileid.is_none() {
        return Err(AppError::PushFailed {
            message: format!("upload_image: 响应缺少可用 url/fileid\n  raw: {resp}"),
            ip_hint: None,
        });
    }
    Ok(UploadOutcome { url, fileid })
}

/// Result of a web-console image upload: a usable CDN URL for inline
/// images, and/or a numeric fileid usable as the draft cover (`fileid0`).
struct UploadOutcome {
    url: Option<String>,
    fileid: Option<String>,
}

/// Mirror of `push::upload_local_images` but uploads via the web-console
/// endpoint so it works without the IP-whitelisted API.
fn upload_local_images_cookie(
    html: &str,
    article_dir: &Path,
    token: &str,
    cookie: &str,
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
            // Embedded data URIs (e.g. the footer QR code) would be stripped
            // by the WeChat editor — decode and upload them to the CDN too.
            if let Some((filename, data)) = crate::wechat::decode_data_uri(src) {
                // Soft-fail: a broken QR image must not take down the push.
                // Use the same `upload_material` web-console endpoint that the
                // cover upload uses — `upload_mass_image` returns ret=200002 here.
                match upload_image_material_bytes(token, cookie, &filename, &data) {
                    Ok(outcome) => match outcome.url {
                        Some(url) => replacements.push((src.to_owned(), url)),
                        None => eprintln!(
                            "  ⚠ embedded image upload failed: 响应缺少 CDN URL; keeping data URI"
                        ),
                    },
                    Err(e) => eprintln!("  ⚠ embedded image upload failed: {e}; keeping data URI"),
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
                let url = upload_image_cookie(token, cookie, &path)?;
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

/// Extract the first `https://` image URL from the article HTML (used as the
/// draft cover `cdn_url` when no explicit cover is available in cookie mode).
fn first_image_url(html: &str) -> Option<String> {
    let mut search = html;
    while let Some(pos) = search.find("src=\"") {
        let rest = &search[pos + 5..];
        let end = rest.find('"').unwrap_or(rest.len());
        let src = &rest[..end];
        if src.starts_with("https://") {
            return Some(src.to_owned());
        }
        search = &search[pos + 5 + end..];
    }
    None
}

/// Create a draft via the web-console `operate_appmsg` endpoint
/// (cookie-authenticated; this is what the mp.weixin.qq.com editor itself
/// calls, so no IP whitelist applies). Per-article fields carry a `0` suffix.
/// `cover_url` is the CDN URL of the uploaded article cover; when empty the
/// draft falls back to the first https image in the body.
fn create_draft_cookie(
    token: &str,
    cookie: &str,
    draft_json: &str,
    cover_url: &str,
    cover_fileid: &str,
) -> Result<String, AppError> {
    let v: Value = serde_json::from_str(draft_json).map_err(|e| AppError::PushFailed {
        message: format!("draft_json parse: {e}"),
        ip_hint: None,
    })?;
    let art = &v["articles"][0];
    let title = art["title"].as_str().unwrap_or("").to_owned();
    let author = art["author"].as_str().unwrap_or("").to_owned();
    let digest = art["digest"].as_str().unwrap_or("").to_owned();
    let content = art["content"].as_str().unwrap_or("").to_owned();
    let cover = if !cover_url.is_empty() {
        cover_url.to_owned()
    } else if !cover_fileid.is_empty() {
        // A numeric fileid is sufficient for the backend to render the cover;
        // leave the CDN URL empty so it does not fall back to a body image.
        String::new()
    } else {
        first_image_url(&content).unwrap_or_default()
    };
    let random = format!(
        "0.{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.subsec_nanos())
            .unwrap_or(123_456_789)
    );
    let form: Vec<(&str, String)> = vec![
        ("token", token.to_owned()),
        ("lang", "zh_CN".to_string()),
        ("f", "json".to_string()),
        ("ajax", "1".to_string()),
        ("random", random),
        ("AppMsgId", String::new()),
        ("count", "1".to_string()),
        ("data_seq", "0".to_string()),
        ("operate_from", "Chrome".to_string()),
        ("isnew", "0".to_string()),
        ("articlenum", "1".to_string()),
        ("pre_timesend_set", "0".to_string()),
        ("is_multi", "0".to_string()),
        ("title0", title),
        ("author0", author),
        ("digest0", digest),
        ("content0", content),
        ("fileid0", cover_fileid.to_owned()),
        ("cdn_url0", cover.clone()),
        ("cdn_235_1_url0", cover.clone()),
        ("cdn_1_1_url0", cover),
        ("show_cover_pic0", "1".to_string()),
        ("sourceurl0", String::new()),
        ("need_open_comment0", "0".to_string()),
        ("only_fans_can_comment0", "0".to_string()),
        ("free_content0", String::new()),
        ("music_id0", String::new()),
        ("video_id0", String::new()),
        ("copyright_type0", "0".to_string()),
    ];
    let url = format!(
        "https://mp.weixin.qq.com/cgi-bin/operate_appmsg?t=ajax-response&sub=create&type=77&token={token}&lang=zh_CN"
    );
    let resp = post_form_with_cookies(&url, &form, cookie)?;
    let ret = resp["base_resp"]["ret"].as_i64().unwrap_or(-1);
    if ret != 0 {
        return Err(AppError::PushFailed {
            message: format!(
                "create_draft(operate_appmsg): ret={ret} {}",
                resp["base_resp"]["err_msg"].as_str().unwrap_or("")
            ),
            ip_hint: None,
        });
    }
    // The web console returns an appMsgId (number or string) — it identifies
    // the draft in the mp.weixin.qq.com backend.
    let media_id = resp["appMsgId"]
        .as_str()
        .map(str::to_owned)
        .or_else(|| resp["appMsgId"].as_i64().map(|n| n.to_string()))
        .unwrap_or_default();
    if media_id.is_empty() {
        return Err(AppError::PushFailed {
            message: format!("create_draft: 响应缺少 appMsgId\n  raw: {resp}"),
            ip_hint: None,
        });
    }
    Ok(media_id)
}

/// Best-effort cleanup of a superseded draft via the web-console endpoint.
/// Only works with web-console appMsgIds (not API media_ids) — failures are
/// tolerated by the caller.
fn delete_draft_cookie(token: &str, cookie: &str, media_id: &str) -> Result<(), AppError> {
    // API media_ids are long base64-ish strings; the web console uses numeric
    // appMsgIds. Skip deletion for IDs we cannot address via this endpoint.
    if !media_id.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::PushFailed {
            message: "old draft uses an API media_id; delete it in the WeChat backend manually"
                .to_owned(),
            ip_hint: None,
        });
    }
    let url = format!(
        "https://mp.weixin.qq.com/cgi-bin/operate_appmsg?t=ajax-response&sub=del&token={token}&lang=zh_CN"
    );
    let form: Vec<(&str, String)> = vec![
        ("token", token.to_owned()),
        ("lang", "zh_CN".to_string()),
        ("f", "json".to_string()),
        ("ajax", "1".to_string()),
        ("AppMsgId", media_id.to_owned()),
    ];
    let resp = post_form_with_cookies(&url, &form, cookie)?;
    let ret = resp["base_resp"]["ret"].as_i64().unwrap_or(-1);
    if ret != 0 {
        return Err(AppError::PushFailed {
            message: format!("delete_draft: ret={ret}"),
            ip_hint: None,
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{first_image_url, form_escape};

    #[test]
    fn first_image_url_picks_first_https_src() {
        let html = r#"<p><img src="local.png"><img src="https://mmbiz.qpic.cn/a.jpg"><img src="https://x.com/b.png"></p>"#;
        assert_eq!(
            first_image_url(html).as_deref(),
            Some("https://mmbiz.qpic.cn/a.jpg")
        );
    }

    #[test]
    fn first_image_url_none_without_https() {
        assert_eq!(first_image_url(r#"<img src="cover.png">"#), None);
    }

    #[test]
    fn form_escape_keeps_unreserved_chars() {
        assert_eq!(form_escape("abcXYZ0129-_.~"), "abcXYZ0129-_.~");
    }

    #[test]
    fn form_escape_encodes_spaces_and_symbols() {
        assert_eq!(form_escape("a b&c=d"), "a%20b%26c%3Dd");
    }

    #[test]
    fn form_escape_encodes_chinese() {
        // 中文 → percent-encoded UTF-8 bytes
        assert_eq!(form_escape("标题"), "%E6%A0%87%E9%A2%98");
    }
}
