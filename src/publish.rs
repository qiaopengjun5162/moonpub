//! WeChat backend automation via playwright-cli.
//! Reference: scripts/moonpub-backend.sh
//!
//! ## User flow
//! 1. First run: user runs `moonpub login` — playwright-cli opens browser,
//!    user scans WeChat QR code once. Session saved to .playwright-cli/.
//! 2. Subsequent runs: `moonpub push` with `auto_publish = true` —
//!    downloads images → creates draft → auto-configures backend (headless).
//!
//! ## Distribution
//! Every user only needs to login ONCE. After that, everything is headless.

use std::path::PathBuf;
use std::process::Command;

/// One-time login: auto-click WeChat quick-login, save session.
/// WeChat backend supports "微信快捷登录" which logs in without QR scan.
pub fn login(vault: &PathBuf) -> Result<String, String> {
    let playwright_dir = vault.join(".playwright-cli");
    std::fs::create_dir_all(&playwright_dir).map_err(|e| format!("mkdir: {e}"))?;

    // Open WeChat backend
    run_playwright(vault, &["open", "https://mp.weixin.qq.com", "--no-headed"])?;

    // Restore previous session if exists (might already be logged in)
    let _ = run_playwright(vault, &["state-load"]);

    // Check if already logged in
    let logged_in = run_playwright_eval(vault, "location.href.includes('/cgi-bin/home')");
    if logged_in {
        let _ = run_playwright(vault, &["state-save"]);
        let _ = run_playwright(vault, &["close"]);
        return Ok("Already logged in. Session saved.".to_owned());
    }

    // Click "微信快捷登录" button — WeChat supports password-less quick login
    run_playwright_eval(
        vault,
        r#"
        (function(){
            var btns = document.querySelectorAll('button, a, .btn, [class*=login], [class*=quick]');
            for(var i=0;i<btns.length;i++){
                var t = btns[i].textContent || '';
                if(t.includes('快捷登录')||t.includes('微信登录')||t.includes('扫码登录')){
                    btns[i].click();
                    return 'clicked: '+t.trim();
                }
            }
            return 'no button found';
        })()
    "#,
    );

    std::thread::sleep(std::time::Duration::from_secs(5));

    // Check login status again
    let ok = run_playwright_eval(vault, "location.href.includes('/cgi-bin/home')");
    if ok {
        run_playwright(vault, &["state-save"])?;
        run_playwright(vault, &["close"])?;
        return Ok("Login successful. Session saved for future headless use.".to_owned());
    }

    // Fallback: open browser for manual login
    run_playwright(vault, &["close"])?;
    run_playwright(vault, &["open", "https://mp.weixin.qq.com", "--headed"])?;
    println!("Auto-login failed. Please scan WeChat QR code in the browser, then press Enter...");
    let _ = std::io::stdin().read_line(&mut String::new());
    run_playwright(vault, &["state-save"])?;
    run_playwright(vault, &["close"])?;

    Ok("Login session saved. Subsequent pushes are fully automatic.".to_owned())
}

fn run_playwright(vault: &PathBuf, args: &[&str]) -> Result<String, String> {
    let output = Command::new("npx")
        .arg("@playwright/cli")
        .args(args)
        .current_dir(vault)
        .output()
        .map_err(|e| format!("npx: {e}"))?;
    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

fn run_playwright_eval(vault: &PathBuf, js: &str) -> bool {
    let output = Command::new("npx")
        .args(["@playwright/cli", "eval", js])
        .current_dir(vault)
        .output();
    match output {
        Ok(o) => {
            let s = String::from_utf8_lossy(&o.stdout);
            s.contains("true") || s.contains("clicked") || s.contains("home")
        }
        Err(_) => false,
    }
}

/// Run the full backend automation (headless, requires prior login).
/// Steps: open draft → set original → set source → AI cover → template ending → save.
pub fn auto_configure(media_id: &str) -> Result<String, String> {
    let vault = vault_dir()?;
    let script = find_script()?;
    let state = vault
        .join(".playwright-cli")
        .join("storage-state-2026-06-10T10-29-23-983Z.json");

    // Check if login session exists
    if !state.exists() {
        return Err("No playwright session found. Run `moonpub login` first.".to_owned());
    }

    let status = Command::new("bash")
        .arg(&script)
        .arg("--headless")
        .env("MOONPUB_MEDIA_ID", media_id)
        .current_dir(&vault)
        .output()
        .map_err(|e| format!("backend script failed: {e}"))?;

    if status.status.success() {
        Ok("backend configured (原创/来源/封面/结尾)".to_owned())
    } else {
        let stderr = String::from_utf8_lossy(&status.stderr);
        open_in_browser(media_id)?;
        Err(format!(
            "backend note: {}\nbrowser opened for manual setup",
            stderr.lines().last().unwrap_or("unknown error")
        ))
    }
}

/// Open the WeChat draft editor in browser (fallback for manual setup).
pub fn open_in_browser(media_id: &str) -> Result<String, String> {
    let url = format!(
        "https://mp.weixin.qq.com/cgi-bin/appmsg?t=media/appmsg_edit_v2&action=edit&isNew=1&type=77&lang=zh_CN&vid={media_id}"
    );
    Command::new("open")
        .arg(&url)
        .spawn()
        .map(|_| url)
        .map_err(|e| format!("failed to open browser: {e}"))
}

fn vault_dir() -> Result<PathBuf, String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let vault = PathBuf::from(format!(
        "{home}/Library/Mobile Documents/com~apple~CloudDocs/ObsidianMain"
    ));
    if vault.exists() {
        Ok(vault)
    } else {
        Err("Obsidian vault not found".to_owned())
    }
}

fn find_script() -> Result<PathBuf, String> {
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(&dir).join("scripts/moonpub-backend.sh");
        if p.exists() {
            return Ok(p);
        }
    }
    let p = PathBuf::from("scripts/moonpub-backend.sh");
    if p.exists() {
        return Ok(p);
    }
    Err("scripts/moonpub-backend.sh not found".to_owned())
}
