//! Backend automation — calls proven Obsidian vault scripts.

use std::path::PathBuf;
use std::process::Command;

/// Run the backend automation script (playwright-cli).
pub fn auto_configure(media_id: &str) -> Result<String, String> {
    let vault = vault_dir()?;
    let script = vault.join("moonpub-backend.sh");
    if !script.exists() {
        // Try project scripts dir
        let alt = PathBuf::from("scripts/moonpub-backend.sh");
        if alt.exists() {
            return run_script(&alt, media_id, &vault);
        }
        return Err("moonpub-backend.sh not found".to_owned());
    }
    run_script(&script, media_id, &vault)
}

fn run_script(script: &PathBuf, media_id: &str, vault: &PathBuf) -> Result<String, String> {
    let status = Command::new("bash")
        .arg(script)
        .arg("--headless")
        .env("MOONPUB_MEDIA_ID", media_id)
        .current_dir(vault)
        .output()
        .map_err(|e| format!("script: {e}"))?;
    if status.status.success() {
        Ok("backend configured".to_owned())
    } else {
        Err(String::from_utf8_lossy(&status.stderr).to_string())
    }
}

#[allow(dead_code)]
pub fn open_in_browser(media_id: &str) -> Result<String, String> {
    let url = format!(
        "https://mp.weixin.qq.com/cgi-bin/appmsg?t=media/appmsg_edit_v2&action=edit&isNew=1&type=77&lang=zh_CN&vid={media_id}"
    );
    Command::new("open")
        .arg(&url)
        .spawn()
        .map(|_| url)
        .map_err(|e| format!("{e}"))
}

/// One-time login via playwright --headed.
pub fn login(vault: &PathBuf) -> Result<String, String> {
    std::fs::create_dir_all(vault.join(".playwright-cli")).map_err(|e| format!("mkdir: {e}"))?;
    let script = vault.join("moonpub-backend.sh");
    if script.exists() {
        Command::new("bash")
            .arg(&script)
            .arg("--headed")
            .current_dir(vault)
            .status()
            .map_err(|e| format!("script: {e}"))?;
        return Ok("Login complete. Run `moonpub login` again to verify.".to_owned());
    }
    Err("moonpub-backend.sh not found in vault. Copy it from scripts/ first.".to_owned())
}

fn vault_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let p = PathBuf::from(format!(
        "{home}/Library/Mobile Documents/com~apple~CloudDocs/ObsidianMain"
    ));
    if p.exists() {
        Ok(p)
    } else {
        Err("vault not found".to_owned())
    }
}
