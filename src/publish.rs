//! WeChat backend automation via Node.js Playwright (proven reliable).

use std::path::PathBuf;
use std::process::Command;

/// One-time WeChat login via Playwright headed browser.
pub fn login() -> Result<String, String> {
    run_node("login")?;
    Ok("Login saved.".to_owned())
}

/// Auto-configure draft: navigate to editor, set all properties, preview.
pub fn auto_configure(_media_id: &str) -> Result<String, String> {
    run_node("configure")?;
    println!("按 Enter 关闭浏览器...");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).ok();
    Ok("done".to_owned())
}

fn run_node(mode: &str) -> Result<String, String> {
    let js = script_path()?;
    Command::new("node")
        .arg(&js)
        .arg(mode)
        .spawn()
        .map_err(|e| format!("node: {e}"))?;
    Ok("started".to_owned())
}

fn script_path() -> Result<PathBuf, String> {
    let p = PathBuf::from("src/publish.js");
    if p.exists() {
        return Ok(p);
    }
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p2 = PathBuf::from(&dir).join("src/publish.js");
        if p2.exists() {
            return Ok(p2);
        }
    }
    Err("src/publish.js not found".to_owned())
}
