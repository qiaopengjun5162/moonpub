use serde_json::Value;
use std::fs;
use std::path::Path;

use crate::error::{AppError, extract_ip_from_message};

/// Production API origin. Every endpoint lives under this base, which is why
/// the client keeps it as data instead of baking full URLs into each call.
const DEFAULT_BASE_URL: &str = "https://api.weixin.qq.com";

const TOKEN_PATH: &str = "/cgi-bin/token";
const DRAFT_ADD_PATH: &str = "/cgi-bin/draft/add";
const DRAFT_UPDATE_PATH: &str = "/cgi-bin/draft/update";
const DRAFT_BATCHGET_PATH: &str = "/cgi-bin/draft/batchget";
const DRAFT_DELETE_PATH: &str = "/cgi-bin/draft/delete";
const MATERIAL_ADD_PATH: &str = "/cgi-bin/material/add_material";
const UPLOADIMG_PATH: &str = "/cgi-bin/media/uploadimg";
const FREE_PUBLISH_PATH: &str = "/cgi-bin/freepublish/submit";

pub struct DraftSummary {
    pub media_id: String,
    pub title: String,
    pub update_time: i64,
}

pub struct WechatClient {
    appid: String,
    secret: String,
    base_url: String,
}

impl WechatClient {
    pub fn new(appid: impl Into<String>, secret: impl Into<String>) -> Self {
        Self::with_base_url(appid, secret, default_base_url())
    }

    /// Build a client against an explicit API origin. Used by tests to point at
    /// a local mock server, and by staging runs against a replay proxy.
    pub fn with_base_url(
        appid: impl Into<String>,
        secret: impl Into<String>,
        base_url: impl Into<String>,
    ) -> Self {
        Self {
            appid: appid.into(),
            secret: secret.into(),
            base_url: base_url.into(),
        }
    }

    fn endpoint(&self, path: &str) -> String {
        format!("{}{path}", self.base_url.trim_end_matches('/'))
    }

    /// Fetch a fresh access_token.
    pub fn access_token(&self) -> Result<String, AppError> {
        let url = format!(
            "{}?grant_type=client_credential&appid={}&secret={}",
            self.endpoint(TOKEN_PATH),
            self.appid,
            self.secret
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
        let url = format!("{}?access_token={token}", self.endpoint(DRAFT_ADD_PATH));
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
        let url = format!("{}?access_token={token}", self.endpoint(DRAFT_UPDATE_PATH));
        let resp = post_json(&url, &body)?;
        let v: Value = serde_json::from_str(&resp)
            .map_err(|e| api_err("update_draft", &e.to_string(), None))?;
        check_errcode_value(&v, "update_draft")
    }

    /// Submit draft for publishing (verified/service accounts only).
    pub fn free_publish(&self, token: &str, media_id: &str) -> Result<String, AppError> {
        let body = serde_json::json!({"media_id": media_id}).to_string();
        let url = format!("{}?access_token={token}", self.endpoint(FREE_PUBLISH_PATH));
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
        let url = format!(
            "{}?access_token={token}",
            self.endpoint(DRAFT_BATCHGET_PATH)
        );
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
        let url = format!("{}?access_token={token}", self.endpoint(DRAFT_DELETE_PATH));
        let resp = post_json(&url, &body)?;
        let v: Value = serde_json::from_str(&resp)
            .map_err(|e| api_err("delete_draft", &e.to_string(), None))?;
        check_errcode_value(&v, "delete_draft")
    }

    /// Upload image to permanent material library, returning media_id.
    pub fn upload_image(&self, token: &str, image_path: &Path) -> Result<String, AppError> {
        let url = format!(
            "{}?access_token={token}&type=image",
            self.endpoint(MATERIAL_ADD_PATH)
        );
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
        let url = format!("{}?access_token={token}", self.endpoint(UPLOADIMG_PATH));
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
        let url = format!("{}?access_token={token}", self.endpoint(UPLOADIMG_PATH));
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

/// Resolve the API origin. `WECHAT_API_BASE` lets tests and staging point the
/// client at a mock or replay server without editing production defaults.
fn default_base_url() -> String {
    std::env::var("WECHAT_API_BASE")
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .unwrap_or_else(|| DEFAULT_BASE_URL.to_owned())
}

/// True for `localhost`, `127.0.0.0/8` and `[::1]`.
///
/// Loopback traffic must never be sent through `HTTPS_PROXY`: developers (and
/// CI) run with a proxy exported in the environment, and routing a local mock
/// server through it turns a local request into a 502.
fn is_loopback_url(url: &str) -> bool {
    let Some((_scheme, rest)) = url.split_once("://") else {
        return false;
    };
    let authority = rest.split(['/', '?', '#']).next().unwrap_or(rest);
    let host = authority.rsplit('@').next().unwrap_or(authority);
    let host = match host.strip_prefix('[') {
        Some(inner) => inner.split(']').next().unwrap_or(inner),
        None => host.split(':').next().unwrap_or(host),
    };
    host.eq_ignore_ascii_case("localhost") || host == "::1" || host.starts_with("127.")
}

pub fn proxy_url_for(url: &str) -> Option<String> {
    if is_loopback_url(url) {
        return None;
    }

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

    /// These tests drive the real `WechatClient` methods against a local mock
    /// server, so URL building, request bodies, multipart encoding and
    /// `errcode` handling are all exercised — not just isolated JSON parsing.
    mod client_against_mock {
        use super::*;
        use crate::mock_wechat::MockWechat;
        use crate::test_helpers::{create_file, temp_root};

        fn client(mock: &MockWechat) -> WechatClient {
            WechatClient::with_base_url("wx_app", "wx_secret", mock.base_url())
        }

        fn png_fixture(root: &std::path::Path, name: &str, size: usize) -> std::path::PathBuf {
            let path = root.join(name);
            let mut bytes = vec![0u8; size];
            bytes[..8].copy_from_slice(&[0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A]);
            create_file(&path, "").unwrap();
            std::fs::write(&path, &bytes).unwrap();
            path
        }

        #[test]
        fn access_token_sends_credentials_and_parses_token() {
            let mock = MockWechat::start([r#"{"access_token":"TOK_1","expires_in":7200}"#]);

            let token = client(&mock).access_token().expect("token");

            assert_eq!(token, "TOK_1");
            let requests = mock.wait_for(1);
            assert_eq!(requests[0].method, "GET");
            assert_eq!(requests[0].path(), "/cgi-bin/token");
            assert!(requests[0].query().contains("grant_type=client_credential"));
            assert!(requests[0].query().contains("appid=wx_app"));
            assert!(requests[0].query().contains("secret=wx_secret"));
        }

        #[test]
        fn access_token_errcode_surfaces_ip_hint() {
            let mock = MockWechat::start([
                r#"{"errcode":40164,"errmsg":"invalid ip 9.9.9.9, not in whitelist"}"#,
            ]);

            let err = client(&mock).access_token().expect_err("errcode must fail");

            let text = err.to_string();
            assert!(text.contains("40164"), "message: {text}");
            assert!(text.contains("9.9.9.9"), "message: {text}");
            match err {
                AppError::PushFailed { ip_hint, .. } => {
                    assert_eq!(ip_hint.as_deref(), Some("9.9.9.9"));
                }
                _ => panic!("expected PushFailed"),
            }
        }

        #[test]
        fn access_token_missing_field_is_an_error() {
            let mock = MockWechat::start([r#"{"expires_in":7200}"#]);

            let err = client(&mock).access_token().expect_err("missing token");

            assert!(err.to_string().contains("missing access_token"));
        }

        #[test]
        fn create_draft_posts_article_json_and_returns_media_id() {
            let root = temp_root("wechat-create-draft").unwrap();
            let draft = root.join("demo.draft.json");
            create_file(
                &draft,
                r#"{"articles":[{"title":"标题甲","content":"<p>正文</p>","thumb_media_id":"t1"}]}"#,
            )
            .unwrap();
            let mock = MockWechat::start([r#"{"errcode":0,"errmsg":"ok","media_id":"MEDIA_1"}"#]);

            let media_id = client(&mock)
                .create_draft("TOK", &draft)
                .expect("create_draft");

            assert_eq!(media_id, "MEDIA_1");
            let requests = mock.wait_for(1);
            assert_eq!(requests[0].method, "POST");
            assert_eq!(requests[0].path(), "/cgi-bin/draft/add");
            assert!(requests[0].query().contains("access_token=TOK"));
            assert!(requests[0].body_text().contains("标题甲"));
            assert!(requests[0].body_text().contains("<p>正文</p>"));

            std::fs::remove_dir_all(root).unwrap();
        }

        #[test]
        fn create_draft_without_media_id_fails() {
            let root = temp_root("wechat-create-draft-empty").unwrap();
            let draft = root.join("demo.draft.json");
            create_file(&draft, r#"{"articles":[]}"#).unwrap();
            let mock = MockWechat::start([r#"{"errcode":0,"errmsg":"ok"}"#]);

            let err = client(&mock)
                .create_draft("TOK", &draft)
                .expect_err("missing media_id");

            assert!(err.to_string().contains("missing media_id"));

            std::fs::remove_dir_all(root).unwrap();
        }

        #[test]
        fn update_draft_sends_only_the_first_article_with_index_zero() {
            let root = temp_root("wechat-update-draft").unwrap();
            let draft = root.join("demo.draft.json");
            create_file(
                &draft,
                r#"{"articles":[{"title":"第一篇"},{"title":"第二篇"}]}"#,
            )
            .unwrap();
            let mock = MockWechat::start([r#"{"errcode":0,"errmsg":"ok"}"#]);

            client(&mock)
                .update_draft("TOK", "MEDIA_9", &draft)
                .expect("update_draft");

            let body = mock.wait_for(1)[0].body_text();
            assert!(body.contains("\"media_id\":\"MEDIA_9\""), "body: {body}");
            assert!(body.contains("\"index\":0"), "body: {body}");
            assert!(body.contains("第一篇"), "body: {body}");
            assert!(!body.contains("第二篇"), "body: {body}");

            std::fs::remove_dir_all(root).unwrap();
        }

        #[test]
        fn list_drafts_parses_titles_and_total_count() {
            let mock = MockWechat::start([r#"{"errcode":0,"total_count":7,"item_count":2,"item":[
                {"media_id":"M1","update_time":1700000000,"content":{"news_item":[{"title":"标题一"}]}},
                {"media_id":"M2","update_time":1700000100,"content":{"news_item":[{"title":"标题二"}]}}
            ]}"#]);

            let (items, total) = client(&mock).list_drafts("TOK", 0, 20).expect("list");

            assert_eq!(total, 7);
            assert_eq!(items.len(), 2);
            assert_eq!(items[0].media_id, "M1");
            assert_eq!(items[0].title, "标题一");
            assert_eq!(items[1].update_time, 1_700_000_100);
            let body = mock.wait_for(1)[0].body_text();
            assert!(body.contains("\"no_content\":1"), "body: {body}");
        }

        #[test]
        fn delete_draft_posts_media_id() {
            let mock = MockWechat::start([r#"{"errcode":0,"errmsg":"ok"}"#]);

            client(&mock)
                .delete_draft("TOK", "MEDIA_DEL")
                .expect("delete_draft");

            let request = &mock.wait_for(1)[0];
            assert_eq!(request.path(), "/cgi-bin/draft/delete");
            assert!(request.body_text().contains("MEDIA_DEL"));
        }

        #[test]
        fn delete_draft_rejects_errcode() {
            let mock = MockWechat::start([r#"{"errcode":40007,"errmsg":"invalid media_id"}"#]);

            let err = client(&mock)
                .delete_draft("TOK", "BAD")
                .expect_err("errcode 40007");

            assert!(err.to_string().contains("40007"));
        }

        #[test]
        fn free_publish_returns_publish_id() {
            let mock = MockWechat::start([r#"{"errcode":0,"publish_id":"PUB_1"}"#]);

            let publish_id = client(&mock)
                .free_publish("TOK", "MEDIA_1")
                .expect("publish");

            assert_eq!(publish_id, "PUB_1");
            let request = &mock.wait_for(1)[0];
            assert_eq!(request.path(), "/cgi-bin/freepublish/submit");
            assert!(request.body_text().contains("MEDIA_1"));
        }

        #[test]
        fn free_publish_unauthorized_is_an_error() {
            let mock = MockWechat::start([r#"{"errcode":48001,"errmsg":"api unauthorized"}"#]);

            let err = client(&mock)
                .free_publish("TOK", "MEDIA_1")
                .expect_err("48001");

            assert!(err.to_string().contains("48001"));
        }

        #[test]
        fn upload_image_uses_material_endpoint_with_multipart_body() {
            let root = temp_root("wechat-upload-material").unwrap();
            let image = png_fixture(&root, "cover.png", 32);
            let mock = MockWechat::start([r#"{"errcode":0,"media_id":"THUMB_1"}"#]);

            let media_id = client(&mock).upload_image("TOK", &image).expect("upload");

            assert_eq!(media_id, "THUMB_1");
            let request = &mock.wait_for(1)[0];
            assert_eq!(request.path(), "/cgi-bin/material/add_material");
            assert!(request.query().contains("type=image"));
            let raw = String::from_utf8_lossy(&request.body);
            assert!(raw.contains("name=\"media\""), "multipart field missing");
            assert!(raw.contains("filename=\"cover.png\""), "filename missing");
            assert!(raw.contains("Content-Type: image/png"), "mime missing");

            std::fs::remove_dir_all(root).unwrap();
        }

        #[test]
        fn upload_image_url_returns_cdn_url() {
            let root = temp_root("wechat-upload-imgurl").unwrap();
            let image = png_fixture(&root, "body.png", 64);
            let mock = MockWechat::start([r#"{"errcode":0,"url":"https://mmbiz.qpic.cn/x.png"}"#]);

            let url = client(&mock)
                .upload_image_url("TOK", &image)
                .expect("upload_image_url");

            assert_eq!(url, "https://mmbiz.qpic.cn/x.png");
            assert_eq!(mock.wait_for(1)[0].path(), "/cgi-bin/media/uploadimg");

            std::fs::remove_dir_all(root).unwrap();
        }

        #[test]
        fn upload_image_url_rejects_gif_before_any_request() {
            let root = temp_root("wechat-upload-gif").unwrap();
            let image = png_fixture(&root, "anim.gif", 16);
            let mock = MockWechat::start([r#"{"errcode":0,"url":"unused"}"#]);

            let err = client(&mock)
                .upload_image_url("TOK", &image)
                .expect_err("gif rejected");

            assert!(err.to_string().contains("only accepts jpg/png"));
            std::thread::sleep(std::time::Duration::from_millis(50));
            assert_eq!(mock.request_count(), 0, "must not hit the API");

            std::fs::remove_dir_all(root).unwrap();
        }

        #[test]
        fn upload_image_url_rejects_oversized_file_before_any_request() {
            let root = temp_root("wechat-upload-big").unwrap();
            let image = png_fixture(&root, "big.png", 1024 * 1024 + 1);
            let mock = MockWechat::start([r#"{"errcode":0,"url":"unused"}"#]);

            let err = client(&mock)
                .upload_image_url("TOK", &image)
                .expect_err("oversized rejected");

            assert!(err.to_string().contains("exceeds 1 MB"));
            std::thread::sleep(std::time::Duration::from_millis(50));
            assert_eq!(mock.request_count(), 0, "must not hit the API");

            std::fs::remove_dir_all(root).unwrap();
        }

        #[test]
        fn upload_image_url_bytes_sends_filename_and_payload() {
            let mock = MockWechat::start([r#"{"errcode":0,"url":"https://cdn/embed.png"}"#]);
            let payload = [0x89, 0x50, 0x4E, 0x47, 0x00, 0x01];

            let url = client(&mock)
                .upload_image_url_bytes("TOK", "moonpub-embed.png", &payload)
                .expect("upload bytes");

            assert_eq!(url, "https://cdn/embed.png");
            let request = &mock.wait_for(1)[0];
            let raw = String::from_utf8_lossy(&request.body);
            assert!(raw.contains("filename=\"moonpub-embed.png\""));
            assert!(
                request.body.windows(payload.len()).any(|w| w == payload),
                "payload bytes missing from multipart body"
            );
        }

        #[test]
        fn upload_image_url_bytes_enforces_size_limit() {
            let mock = MockWechat::start([r#"{"errcode":0,"url":"unused"}"#]);
            let payload = vec![0u8; 1024 * 1024 + 1];

            let err = client(&mock)
                .upload_image_url_bytes("TOK", "big.png", &payload)
                .expect_err("oversized");

            assert!(err.to_string().contains("exceeds 1 MB"));
            assert_eq!(mock.request_count(), 0);
        }

        #[test]
        fn http_error_status_is_reported_as_push_failure() {
            // No response queued: the client connects, gets EOF and must fail
            // rather than panic or silently return an empty value.
            let mock = MockWechat::start(Vec::<String>::new());

            let err = client(&mock)
                .access_token()
                .expect_err("no server response");

            assert!(matches!(err, AppError::PushFailed { .. }));
        }
    }
}
