use std::fs;
use std::path::Path;

use crate::{AppError, extract_ip_from_message};

const TOKEN_URL: &str = "https://api.weixin.qq.com/cgi-bin/token";
const DRAFT_ADD_URL: &str = "https://api.weixin.qq.com/cgi-bin/draft/add";
const DRAFT_UPDATE_URL: &str = "https://api.weixin.qq.com/cgi-bin/draft/update";
const MATERIAL_ADD_URL: &str = "https://api.weixin.qq.com/cgi-bin/material/add_material";
const FREE_PUBLISH_URL: &str = "https://api.weixin.qq.com/cgi-bin/freepublish/submit";

// ── WechatClient ──────────────────────────────────────────────────────────────

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

    /// Fetch a fresh access_token. Tokens expire after 7200 seconds; we don't
    /// cache here — the CLI is short-lived so a fresh token per invocation is fine.
    pub fn access_token(&self) -> Result<String, AppError> {
        let url = format!(
            "{TOKEN_URL}?grant_type=client_credential&appid={}&secret={}",
            self.appid, self.secret
        );
        let body: String = ureq::get(&url)
            .call()
            .map_err(|e| api_err("get_access_token", &e.to_string(), None))?
            .into_string()
            .map_err(|e| api_err("get_access_token", &e.to_string(), None))?;

        // {"access_token":"...","expires_in":7200}
        // or {"errcode":40164,"errmsg":"invalid ip ..."}
        if let Some(errcode) = extract_json_number(&body, "errcode")
            && errcode != 0
        {
            let errmsg = extract_json_str(&body, "errmsg").unwrap_or_default();
            let ip = extract_ip_from_message(&errmsg);
            return Err(AppError::PushFailed {
                message: format!("get_access_token errcode={errcode}: {errmsg}"),
                ip_hint: ip,
            });
        }

        extract_json_str(&body, "access_token").ok_or_else(|| {
            api_err(
                "get_access_token",
                &format!("unexpected response: {body}"),
                None,
            )
        })
    }

    /// POST draft/add — create a new WeChat draft.
    /// `draft_json_path` points to a file whose content matches the WeChat
    /// `draft/add` request body (the `articles` array format).
    pub fn create_draft(&self, token: &str, draft_json_path: &Path) -> Result<String, AppError> {
        let body = fs::read_to_string(draft_json_path).map_err(|source| AppError::Io {
            path: draft_json_path.to_path_buf(),
            source,
        })?;

        let url = format!("{DRAFT_ADD_URL}?access_token={token}");
        let resp = post_json(&url, &body)?;

        // {"errcode":0,"errmsg":"ok","media_id":"..."}
        check_errcode(&resp, "create_draft")?;
        extract_json_str(&resp, "media_id")
            .ok_or_else(|| api_err("create_draft", &format!("no media_id in: {resp}"), None))
    }

    /// POST draft/update — replace content of an existing draft by media_id.
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

        // WeChat update body: {"media_id":"...","index":0,"articles":{...}}
        // draft_json_path contains {"articles":[{...}]} — extract first article.
        let article = extract_first_article(&draft).ok_or_else(|| {
            api_err(
                "update_draft",
                "cannot parse articles from draft.json",
                None,
            )
        })?;
        let body = format!(
            "{{\"media_id\":\"{}\",\"index\":0,\"articles\":{}}}",
            escape_json_value(media_id),
            article
        );

        let url = format!("{DRAFT_UPDATE_URL}?access_token={token}");
        let resp = post_json(&url, &body)?;
        check_errcode(&resp, "update_draft")
    }

    /// Submit a draft for publishing. Only available for verified/service accounts.
    /// Personal subscription accounts must publish manually in the WeChat backend.
    /// Returns publish_id on success.
    pub fn free_publish(&self, token: &str, media_id: &str) -> Result<String, AppError> {
        let body = format!("{{\"media_id\":\"{}\"}}", escape_json_value(media_id));
        let url = format!("{FREE_PUBLISH_URL}?access_token={token}");
        let resp = post_json(&url, &body)?;
        // Success: {"errcode":0,"errmsg":"ok","publish_id":"..."}
        // Personal accounts get errcode 48001 (unauthorized) or similar
        check_errcode(&resp, "free_publish")?;
        extract_json_str(&resp, "publish_id").ok_or_else(|| {
            api_err(
                "free_publish",
                &format!("no publish_id in response: {resp}"),
                None,
            )
        })
    }

    /// Upload a local image file to WeChat permanent material library.
    /// Returns the `media_id` of the uploaded image.
    pub fn upload_image(&self, token: &str, image_path: &Path) -> Result<String, AppError> {
        let data = fs::read(image_path).map_err(|source| AppError::Io {
            path: image_path.to_path_buf(),
            source,
        })?;

        let filename = image_path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("image.jpg");
        let mime = mime_for(filename);

        let url = format!("{MATERIAL_ADD_URL}?access_token={token}&type=image");

        // Multipart form: field "media" + file data
        let boundary = "moonpub_boundary_12345";
        let mut form = Vec::new();
        form.extend_from_slice(
            format!(
                "--{boundary}\r\nContent-Disposition: form-data; name=\"media\"; filename=\"{filename}\"\r\nContent-Type: {mime}\r\n\r\n",
            )
            .as_bytes(),
        );
        form.extend_from_slice(&data);
        form.extend_from_slice(format!("\r\n--{boundary}--\r\n").as_bytes());

        let resp = ureq::post(&url)
            .set(
                "Content-Type",
                &format!("multipart/form-data; boundary={boundary}"),
            )
            .send_bytes(&form)
            .map_err(|e| api_err("upload_image", &e.to_string(), None))?
            .into_string()
            .map_err(|e| api_err("upload_image", &e.to_string(), None))?;

        check_errcode(&resp, "upload_image")?;
        extract_json_str(&resp, "media_id")
            .ok_or_else(|| api_err("upload_image", &format!("no media_id in: {resp}"), None))
    }
}

// ── helpers ───────────────────────────────────────────────────────────────────

fn post_json(url: &str, body: &str) -> Result<String, AppError> {
    ureq::post(url)
        .set("Content-Type", "application/json; charset=utf-8")
        .send_string(body)
        .map_err(|e| api_err("http_post", &e.to_string(), None))?
        .into_string()
        .map_err(|e| api_err("http_post", &e.to_string(), None))
}

fn check_errcode(resp: &str, op: &str) -> Result<(), AppError> {
    if let Some(code) = extract_json_number(resp, "errcode")
        && code != 0
    {
        let errmsg = extract_json_str(resp, "errmsg").unwrap_or_default();
        let ip = extract_ip_from_message(&errmsg);
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

/// Minimal JSON string extractor: find `"key":"value"` and return value.
pub fn extract_json_str(json: &str, key: &str) -> Option<String> {
    let marker = format!("\"{key}\":\"");
    let start = json.find(&marker)? + marker.len();
    let mut out = String::new();
    let mut chars = json[start..].chars();
    while let Some(ch) = chars.next() {
        match ch {
            '"' => return Some(out),
            '\\' => match chars.next()? {
                '"' => out.push('"'),
                '\\' => out.push('\\'),
                'n' => out.push('\n'),
                'r' => out.push('\r'),
                't' => out.push('\t'),
                c => out.push(c),
            },
            c => out.push(c),
        }
    }
    None
}

/// Extract a numeric value: `"key":12345`
fn extract_json_number(json: &str, key: &str) -> Option<i64> {
    let marker = format!("\"{key}\":");
    let start = json.find(&marker)? + marker.len();
    let s = json[start..]
        .trim_start()
        .split([',', '}', '\n'])
        .next()?
        .trim();
    if s == "null" || s.starts_with('"') {
        return None;
    }
    s.parse().ok()
}

/// Extract the first element of the `articles` array as a raw JSON object string.
fn extract_first_article(draft_json: &str) -> Option<String> {
    // Find "articles": [...] then the first { after the [
    let mark = "\"articles\"";
    let pos = draft_json.find(mark)?;
    let after_mark = &draft_json[pos + mark.len()..];
    let bracket = after_mark.find('[')?;
    let inner = &after_mark[bracket + 1..]; // skip '['
    // inner now starts right after '[', skip whitespace to find '{'
    let brace = inner.find('{')?;
    let inner = &inner[brace..]; // start from '{'
    // Find the matching closing brace of the first object
    let mut depth = 0i32;
    let mut end = 0;
    for (i, ch) in inner.char_indices() {
        match ch {
            '{' => depth += 1,
            '}' => {
                depth -= 1;
                if depth == 0 {
                    end = i + 1;
                    break;
                }
            }
            _ => {}
        }
    }
    if end == 0 {
        return None;
    }
    Some(inner[..end].to_owned())
}

fn escape_json_value(s: &str) -> String {
    s.replace('\\', "\\\\").replace('"', "\\\"")
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
    fn extract_json_str_basic() {
        let json = r#"{"access_token":"abc123","expires_in":7200}"#;
        assert_eq!(
            extract_json_str(json, "access_token").as_deref(),
            Some("abc123")
        );
        assert_eq!(extract_json_str(json, "expires_in"), None); // it's a number
    }

    #[test]
    fn extract_json_number_basic() {
        let json = r#"{"errcode":40164,"errmsg":"invalid ip"}"#;
        assert_eq!(extract_json_number(json, "errcode"), Some(40164));
        assert_eq!(extract_json_number(json, "errcode_missing"), None);
    }

    #[test]
    fn extract_json_number_zero() {
        let json = r#"{"errcode":0,"errmsg":"ok","media_id":"XYZ"}"#;
        assert_eq!(extract_json_number(json, "errcode"), Some(0));
    }

    #[test]
    fn extract_first_article_parses() {
        let draft = r#"{"articles":[{"title":"T","content":"<p>hi</p>"}]}"#;
        let art = extract_first_article(draft).unwrap();
        assert!(art.contains("\"title\":\"T\""));
        assert!(art.contains("\"content\""));
    }

    #[test]
    fn check_errcode_passes_on_zero() {
        let resp = r#"{"errcode":0,"errmsg":"ok","media_id":"ABC"}"#;
        assert!(check_errcode(resp, "test").is_ok());
    }

    #[test]
    fn check_errcode_fails_on_nonzero() {
        let resp = r#"{"errcode":40164,"errmsg":"invalid ip 1.2.3.4 not in whitelist"}"#;
        let err = check_errcode(resp, "create_draft").unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("40164"));
        assert!(msg.contains("1.2.3.4"));
    }

    #[test]
    fn extract_first_article_nested_braces() {
        let draft = r#"{"articles":[{"title":"T","meta":{"a":1},"content":"ok"}]}"#;
        let art = extract_first_article(draft).unwrap();
        assert!(art.contains("\"meta\""));
        assert!(art.contains("\"content\":\"ok\""));
    }

    #[test]
    fn mime_detection() {
        assert_eq!(mime_for("cover.png"), "image/png");
        assert_eq!(mime_for("img.JPG"), "image/jpeg");
        assert_eq!(mime_for("anim.GIF"), "image/gif");
        assert_eq!(mime_for("photo.webp"), "image/webp");
    }

    // free_publish response parsing (mock data, no live API)

    #[test]
    fn free_publish_success_response() {
        let resp = r#"{"errcode":0,"errmsg":"ok","publish_id":"pub_12345"}"#;
        assert_eq!(
            extract_json_str(resp, "publish_id").as_deref(),
            Some("pub_12345")
        );
        assert!(check_errcode(resp, "free_publish").is_ok());
    }

    #[test]
    fn free_publish_personal_account_blocked() {
        // Personal accounts get errcode 48001 (API unauthorized)
        let resp = r#"{"errcode":48001,"errmsg":"api unauthorized"}"#;
        assert!(check_errcode(resp, "free_publish").is_err());
    }

    #[test]
    fn free_publish_missing_media_id() {
        let resp = r#"{"errcode":0,"errmsg":"ok","publish_id":""}"#;
        assert_eq!(extract_json_str(resp, "publish_id").as_deref(), Some(""));
    }
}
