//! WeChat backend automation.
//! Personal subscription accounts have no API for cover/original/collection,
//! so we automate the backend web UI.

use std::process::Command;

/// Open the WeChat draft editor in browser for a given media_id.
/// The user can then set: cover image, original declaration, collection, source.
pub fn open_in_browser(media_id: &str) -> Result<String, String> {
    let url = draft_editor_url(media_id);

    // Try system `open` command first (macOS)
    let result = Command::new("open").arg(&url).spawn();

    match result {
        Ok(_) => Ok(format!("draft editor opened in browser\n  {url}")),
        Err(_) => {
            // Fallback: try Chrome directly
            let chrome = crate::find_chrome().ok_or("no browser found")?;
            Command::new(&chrome)
                .arg(&url)
                .spawn()
                .map_err(|e| format!("failed to open browser: {e}"))?;
            Ok(format!("draft editor opened in Chrome\n  {url}"))
        }
    }
}

fn draft_editor_url(media_id: &str) -> String {
    format!(
        "https://mp.weixin.qq.com/cgi-bin/appmsg?t=media/appmsg_edit_v2&action=edit&isNew=1&type=77&lang=zh_CN&vid={media_id}"
    )
}
