//! WeChat backend automation via playwright-cli.
//! Reference: scripts/moonpub-backend.sh

use std::path::PathBuf;
use std::process::Command;

/// Run backend automation after draft push.
/// Uses playwright-cli to: login → original → source → AI cover → template ending → save.
pub fn auto_configure(media_id: &str) -> Result<String, String> {
    // Find the script relative to the project root
    let script = find_script()?;

    let status = Command::new("bash")
        .arg(&script)
        .arg("--headless")
        .env("MOONPUB_MEDIA_ID", media_id)
        .output()
        .map_err(|e| format!("failed to run backend script: {e}"))?;

    if status.status.success() {
        Ok("backend configured (原创/来源/封面/结尾)".to_owned())
    } else {
        let stderr = String::from_utf8_lossy(&status.stderr);
        // Fallback: open browser
        open_in_browser(media_id)?;
        Err(format!(
            "backend automation note: {}\nbrowser opened for manual setup",
            stderr
                .lines()
                .last()
                .unwrap_or("playwright-cli not available")
        ))
    }
}

fn find_script() -> Result<PathBuf, String> {
    // Look relative to the project root (where Cargo.toml lives)
    let candidates = [
        PathBuf::from("scripts/moonpub-backend.sh"),
        PathBuf::from("../scripts/moonpub-backend.sh"),
    ];
    for p in &candidates {
        if p.exists() {
            return Ok(p.clone());
        }
    }
    // Try to find via CARGO_MANIFEST_DIR
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(&dir).join("scripts/moonpub-backend.sh");
        if p.exists() {
            return Ok(p);
        }
    }
    Err("scripts/moonpub-backend.sh not found".to_owned())
}

/// Open the WeChat draft editor in browser (fallback).
pub fn open_in_browser(media_id: &str) -> Result<String, String> {
    let url = format!(
        "https://mp.weixin.qq.com/cgi-bin/appmsg?t=media/appmsg_edit_v2&action=edit&isNew=1&type=77&lang=zh_CN&vid={media_id}"
    );
    let result = Command::new("open").arg(&url).spawn();
    match result {
        Ok(_) => Ok(url),
        Err(e) => Err(format!("failed to open browser: {e}")),
    }
}
