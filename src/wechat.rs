use serde_json::Value;
use std::fs;
use std::path::Path;

use crate::error::{AppError, extract_ip_from_message};

const TOKEN_URL: &str = "https://api.weixin.qq.com/cgi-bin/token";
const DRAFT_ADD_URL: &str = "https://api.weixin.qq.com/cgi-bin/draft/add";
const DRAFT_UPDATE_URL: &str = "https://api.weixin.qq.com/cgi-bin/draft/update";
const DRAFT_BATCHGET_URL: &str = "https://api.weixin.qq.com/cgi-bin/draft/batchget";
const DRAFT_DELETE_URL: &str = "https://api.weixin.qq.com/cgi-bin/draft/delete";
const MATERIAL_ADD_URL: &str = "https://api.weixin.qq.com/cgi-bin/material/add_material";
const UPLOADIMG_URL: &str = "https://api.weixin.qq.com/cgi-bin/media/uploadimg";
const FREE_PUBLISH_URL: &str = "https://api.weixin.qq.com/cgi-bin/freepublish/submit";

pub struct DraftSummary {
    pub media_id: String,
    pub title: String,
    pub update_time: i64,
}

pub struct WechatClient {
    appid: String,
    secret: String,
}

impl WechatClient {
    pub fn new(appid: impl Into<String>, secret: impl Into<String>) -> Self {
        Self {
            appid: appid.into(),
            secret: secret.into(),
        }
    }

    /// Fetch a fresh access_token.
    pub fn access_token(&self) -> Result<String, AppError> {
        let url = format!(
            "{TOKEN_URL}?grant_type=client_credential&appid={}&secret={}",
            self.appid, self.secret
        );
        let body = agent_for_url(&url)?
            .get(&url)
            .call()
            .map_err(|e| api_err("get_access_token", &e.to_string(), None))?
            .into_string()
            .map_err(|e| api_err("get_access_token", &e.to_string(), None))?;

        let v: Value = serde_json::from_str(&body)
            .map_err(|e| api_err("get_access_token", &e.to_string(), None))?;

        check_errcode_value(&v, "get_access_token")?;
        v["access_token"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| api_err("get_access_token", "missing access_token", None))
    }

    /// Create a new WeChat draft from a draft.json file.
    pub fn create_draft(&self, token: &str, draft_json_path: &Path) -> Result<String, AppError> {
        let body = fs::read_to_string(draft_json_path).map_err(|source| AppError::Io {
            path: draft_json_path.to_path_buf(),
            source,
        })?;
        let url = format!("{DRAFT_ADD_URL}?access_token={token}");
        let resp = post_json(&url, &body)?;
        let v: Value = serde_json::from_str(&resp)
            .map_err(|e| api_err("create_draft", &e.to_string(), None))?;
        check_errcode_value(&v, "create_draft")?;
        v["media_id"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| api_err("create_draft", "missing media_id", None))
    }

    /// Update an existing draft by media_id.
    pub fn update_draft(
        &self,
        token: &str,
        media_id: &str,
        draft_json_path: &Path,
    ) -> Result<(), AppError> {
        let draft = fs::read_to_string(draft_json_path).map_err(|source| AppError::Io {
            path: draft_json_path.to_path_buf(),
            source,
        })?;
        let mut draft_val: Value = serde_json::from_str(&draft)
            .map_err(|e| api_err("update_draft", &e.to_string(), None))?;
        let first = draft_val["articles"][0].take();
        let body = serde_json::json!({
            "media_id": media_id,
            "index": 0,
            "articles": first,
        })
        .to_string();
        let url = format!("{DRAFT_UPDATE_URL}?access_token={token}");
        let resp = post_json(&url, &body)?;
        let v: Value = serde_json::from_str(&resp)
            .map_err(|e| api_err("update_draft", &e.to_string(), None))?;
        check_errcode_value(&v, "update_draft")
    }

    /// Submit draft for publishing (verified/service accounts only).
    pub fn free_publish(&self, token: &str, media_id: &str) -> Result<String, AppError> {
        let body = serde_json::json!({"media_id": media_id}).to_string();
        let url = format!("{FREE_PUBLISH_URL}?access_token={token}");
        let resp = post_json(&url, &body)?;
        let v: Value = serde_json::from_str(&resp)
            .map_err(|e| api_err("free_publish", &e.to_string(), None))?;
        check_errcode_value(&v, "free_publish")?;
        v["publish_id"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| api_err("free_publish", "missing publish_id", None))
    }

    /// Fetch a page of draft summaries (title + media_id). Returns (items, total_count).
    pub fn list_drafts(
        &self,
        token: &str,
        offset: u32,
        count: u32,
    ) -> Result<(Vec<DraftSummary>, u64), AppError> {
        let body = serde_json::json!({
            "offset": offset,
            "count": count,
            "no_content": 1,
        })
        .to_string();
        let url = format!("{DRAFT_BATCHGET_URL}?access_token={token}");
        let resp = post_json(&url, &body)?;
        let v: Value = serde_json::from_str(&resp)
            .map_err(|e| api_err("list_drafts", &e.to_string(), None))?;
        check_errcode_value(&v, "list_drafts")?;
        let total = v["total_count"].as_u64().unwrap_or(0);
        let items = v["item"]
            .as_array()
            .map(|arr| {
                arr.iter()
                    .map(|item| DraftSummary {
                        media_id: item["media_id"].as_str().unwrap_or("").to_owned(),
                        title: item["content"]["news_item"][0]["title"]
                            .as_str()
                            .unwrap_or("")
                            .to_owned(),
                        update_time: item["update_time"].as_i64().unwrap_or(0),
                    })
                    .collect()
            })
            .unwrap_or_default();
        Ok((items, total))
    }

    /// Delete a draft by media_id.
    pub fn delete_draft(&self, token: &str, media_id: &str) -> Result<(), AppError> {
        let body = serde_json::json!({"media_id": media_id}).to_string();
        let url = format!("{DRAFT_DELETE_URL}?access_token={token}");
        let resp = post_json(&url, &body)?;
        let v: Value = serde_json::from_str(&resp)
            .map_err(|e| api_err("delete_draft", &e.to_string(), None))?;
        check_errcode_value(&v, "delete_draft")
    }

    /// Upload image to permanent material library, returning media_id.
    pub fn upload_image(&self, token: &str, image_path: &Path) -> Result<String, AppError> {
        let url = format!("{MATERIAL_ADD_URL}?access_token={token}&type=image");
        let resp = upload_raw(&url, image_path)?;
        let v: Value = serde_json::from_str(&resp)
            .map_err(|e| api_err("upload_image", &e.to_string(), None))?;
        check_errcode_value(&v, "upload_image")?;
        v["media_id"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| api_err("upload_image", "missing media_id", None))
    }

    /// Upload article-content image via /cgi-bin/media/uploadimg (no quota cost), returning CDN URL.
    /// API constraint: jpg/png only, max 1 MB.
    pub fn upload_image_url(&self, token: &str, image_path: &Path) -> Result<String, AppError> {
        let filename = image_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("");
        let lower = filename.to_lowercase();
        if !lower.ends_with(".jpg") && !lower.ends_with(".jpeg") && !lower.ends_with(".png") {
            return Err(api_err(
                "upload_image_url",
                &format!("{filename}: /media/uploadimg only accepts jpg/png"),
                None,
            ));
        }
        let size = fs::metadata(image_path).map(|m| m.len()).unwrap_or(0);
        if size > 1024 * 1024 {
            return Err(api_err(
                "upload_image_url",
                &format!("{filename}: {size} bytes exceeds 1 MB limit"),
                None,
            ));
        }
        let url = format!("{UPLOADIMG_URL}?access_token={token}");
        let resp = upload_raw(&url, image_path)?;
        let v: Value = serde_json::from_str(&resp)
            .map_err(|e| api_err("upload_image_url", &e.to_string(), None))?;
        check_errcode_value(&v, "upload_image_url")?;
        v["url"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| api_err("upload_image_url", "missing url", None))
    }

    /// Upload raw image bytes (e.g. decoded from an embedded data URI) via
    /// /cgi-bin/media/uploadimg, returning a CDN URL. jpg/png only, max 1 MB.
    pub fn upload_image_url_bytes(
        &self,
        token: &str,
        filename: &str,
        data: &[u8],
    ) -> Result<String, AppError> {
        if data.len() > 1024 * 1024 {
            return Err(api_err(
                "upload_image_url",
                &format!("{filename}: {} bytes exceeds 1 MB limit", data.len()),
                None,
            ));
        }
        let url = format!("{UPLOADIMG_URL}?access_token={token}");
        let resp = upload_raw_bytes(&url, filename, data)?;
        let v: Value = serde_json::from_str(&resp)
            .map_err(|e| api_err("upload_image_url", &e.to_string(), None))?;
        check_errcode_value(&v, "upload_image_url")?;
        v["url"]
            .as_str()
            .map(String::from)
            .ok_or_else(|| api_err("upload_image_url", "missing url", None))
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

/// Decode a `data:image/{png,jpeg};base64,...` URI into an uploadable
/// filename + payload. Returns None for anything else. Embedded data URIs
/// (e.g. the footer QR code) are stripped by the WeChat editor, so both push
/// paths upload them to the CDN and rewrite the src.
pub(crate) fn decode_data_uri(src: &str) -> Option<(String, Vec<u8>)> {
    use base64::Engine;
    let rest = src.strip_prefix("data:image/")?;
    let (kind, b64) = rest.split_once(";base64,")?;
    let ext = match kind {
        "png" => "png",
        "jpeg" | "jpg" => "jpg",
        _ => return None,
    };
    let data = base64::engine::general_purpose::STANDARD.decode(b64).ok()?;
    Some((format!("moonpub-embed.{ext}"), data))
}

fn post_json(url: &str, body: &str) -> Result<String, AppError> {
    agent_for_url(url)?
        .post(url)
        .set("Content-Type", "application/json; charset=utf-8")
        .send_string(body)
        .map_err(|e| api_err("http_post", &e.to_string(), None))?
        .into_string()
        .map_err(|e| api_err("http_post", &e.to_string(), None))
}

fn check_errcode_value(v: &Value, op: &str) -> Result<(), AppError> {
    if let Some(code) = v["errcode"].as_i64()
        && code != 0
    {
        let errmsg = v["errmsg"].as_str().unwrap_or_default();
        let ip = extract_ip_from_message(errmsg);
        return Err(AppError::PushFailed {
            message: format!("{op} errcode={code}: {errmsg}"),
            ip_hint: ip,
        });
    }
    Ok(())
}

fn api_err(op: &str, msg: &str, ip_hint: Option<String>) -> AppError {
    AppError::PushFailed {
        message: format!("{op}: {msg}"),
        ip_hint,
    }
}

fn upload_raw(url: &str, image_path: &Path) -> Result<String, AppError> {
    let data = fs::read(image_path).map_err(|source| AppError::Io {
        path: image_path.to_path_buf(),
        source,
    })?;
    let filename = image_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("image.jpg");
    upload_raw_bytes(url, filename, &data)
}

fn upload_raw_bytes(url: &str, filename: &str, data: &[u8]) -> Result<String, AppError> {
    let mime = mime_for(filename);
    let boundary = "moonpub_boundary_12345";
    let mut form = Vec::new();
    form.extend_from_slice(
        format!(
            "--{boundary}\r\nContent-Disposition: form-data; name=\"media\"; filename=\"{filename}\"\r\nContent-Type: {mime}\r\n\r\n",
        )
        .as_bytes(),
    );
    form.extend_from_slice(data);
    form.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());
    agent_for_url(url)?
        .post(url)
        .set(
            "Content-Type",
            &format!("multipart/form-data; boundary={boundary}"),
        )
        .send_bytes(&form)
        .map_err(|e| api_err("upload_image", &e.to_string(), None))?
        .into_string()
        .map_err(|e| api_err("upload_image", &e.to_string(), None))
}

fn agent_for_url(url: &str) -> Result<ureq::Agent, AppError> {
    let mut builder = ureq::AgentBuilder::new();

    if let Some(proxy_url) = proxy_url_for(url) {
        debug_proxy(url, Some(&proxy_url));
        let proxy = ureq::Proxy::new(&proxy_url)
            .map_err(|e| api_err("configure_proxy", &e.to_string(), None))?;
        builder = builder.proxy(proxy);
    } else {
        debug_proxy(url, None);
    }

    Ok(builder.build())
}

fn debug_proxy(url: &str, proxy_url: Option<&str>) {
    if std::env::var("MOONPUB_DEBUG_PROXY").ok().as_deref() != Some("1") {
        return;
    }
    let proxy = proxy_url.unwrap_or("<none>");
    eprintln!(
        "moonpub debug proxy: url={} proxy={proxy}",
        redact_url_query(url)
    );
}

fn redact_url_query(url: &str) -> &str {
    url.split('?').next().unwrap_or(url)
}

fn proxy_url_for(url: &str) -> Option<String> {
    let candidates = if url.starts_with("https://") {
        ["HTTPS_PROXY", "https_proxy", "ALL_PROXY", "all_proxy"]
    } else {
        ["HTTP_PROXY", "http_proxy", "ALL_PROXY", "all_proxy"]
    };

    candidates
        .into_iter()
        .filter_map(|key| std::env::var(key).ok())
        .map(|value| value.trim().to_string())
        .find(|value| !value.is_empty())
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

// ── tests ─────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decode_data_uri_extracts_png_payload() {
        let (filename, data) = decode_data_uri("data:image/png;base64,iVBORw0KGgo=").unwrap();
        assert_eq!(filename, "moonpub-embed.png");
        assert_eq!(data, vec![0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
    }

    #[test]
    fn decode_data_uri_rejects_non_image() {
        assert!(decode_data_uri("data:text/html;base64,PGI+").is_none());
        assert!(decode_data_uri("https://x.com/a.png").is_none());
    }

    #[test]
    fn access_token_parses() {
        let v: Value =
            serde_json::from_str(r#"{"access_token":"abc123","expires_in":7200}"#).unwrap();
        assert_eq!(v["access_token"].as_str(), Some("abc123"));
        assert!(check_errcode_value(&v, "test").is_ok());
    }

    #[test]
    fn errcode_detected() {
        let v: Value = serde_json::from_str(
            r#"{"errcode":40164,"errmsg":"invalid ip 1.2.3.4 not in whitelist"}"#,
        )
        .unwrap();
        let err = check_errcode_value(&v, "create_draft").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("40164"));
        assert!(msg.contains("1.2.3.4"));
        assert!(msg.contains("stable egress IP"));
    }

    #[test]
    fn errcode_zero_passes() {
        let v: Value =
            serde_json::from_str(r#"{"errcode":0,"errmsg":"ok","media_id":"ABC"}"#).unwrap();
        assert!(check_errcode_value(&v, "test").is_ok());
    }

    #[test]
    fn update_draft_article_extraction() {
        let draft = r#"{"articles":[{"title":"T","content":"<p>hi</p>"}]}"#;
        let mut v: Value = serde_json::from_str(draft).unwrap();
        let first = v["articles"][0].take();
        let body = serde_json::json!({"media_id":"M","index":0,"articles":first}).to_string();
        assert!(body.contains("\"media_id\":\"M\""));
        assert!(body.contains("\"title\":\"T\""));
    }

    #[test]
    fn update_draft_nested_article() {
        let draft = r#"{"articles":[{"title":"T","meta":{"a":1}}]}"#;
        let mut v: Value = serde_json::from_str(draft).unwrap();
        let first = v["articles"][0].take();
        assert!(first["meta"]["a"].as_i64() == Some(1));
    }

    #[test]
    fn mime_detection() {
        assert_eq!(mime_for("cover.png"), "image/png");
        assert_eq!(mime_for("img.JPG"), "image/jpeg");
        assert_eq!(mime_for("anim.GIF"), "image/gif");
        assert_eq!(mime_for("photo.webp"), "image/webp");
    }

    #[test]
    fn proxy_url_prefers_https_proxy_for_https_requests() {
        unsafe {
            std::env::set_var("HTTPS_PROXY", "http://127.0.0.1:7890");
            std::env::set_var("HTTP_PROXY", "http://127.0.0.1:8888");
        }

        let proxy = proxy_url_for("https://api.weixin.qq.com").unwrap();

        assert_eq!(proxy, "http://127.0.0.1:7890");

        unsafe {
            std::env::remove_var("HTTPS_PROXY");
            std::env::remove_var("HTTP_PROXY");
        }
    }

    #[test]
    fn proxy_url_falls_back_to_http_proxy_for_http_requests() {
        unsafe {
            std::env::set_var("HTTP_PROXY", "http://127.0.0.1:7897");
        }

        let proxy = proxy_url_for("http://example.com").unwrap();

        assert_eq!(proxy, "http://127.0.0.1:7897");

        unsafe {
            std::env::remove_var("HTTP_PROXY");
        }
    }

    #[test]
    fn redact_url_query_removes_token_from_debug_url() {
        let url = "https://api.weixin.qq.com/cgi-bin/draft/add?access_token=secret";

        assert_eq!(
            redact_url_query(url),
            "https://api.weixin.qq.com/cgi-bin/draft/add"
        );
    }

    #[test]
    fn free_publish_response() {
        let v: Value =
            serde_json::from_str(r#"{"errcode":0,"errmsg":"ok","publish_id":"pub_12345"}"#)
                .unwrap();
        assert_eq!(v["publish_id"].as_str(), Some("pub_12345"));
        assert!(check_errcode_value(&v, "free_publish").is_ok());
    }

    #[test]
    fn free_publish_unauthorized() {
        let v: Value =
            serde_json::from_str(r#"{"errcode":48001,"errmsg":"api unauthorized"}"#).unwrap();
        assert!(check_errcode_value(&v, "free_publish").is_err());
    }
}
