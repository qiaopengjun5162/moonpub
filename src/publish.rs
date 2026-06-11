//! WeChat backend automation via Node.js Playwright (proven reliable).

use std::path::PathBuf;
use std::process::Command;

pub fn login() -> Result<String, String> {
    run_node("login")
}

pub fn auto_configure(_media_id: &str) -> Result<String, String> {
    run_node("configure")
}

fn run_node(mode: &str) -> Result<String, String> {
    let js = script_path()?;
    println!("Running: node {} {}", js.display(), mode);
    let status = Command::new("node")
        .arg(&js)
        .arg(mode)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .map_err(|e| format!("node: {e}"))?;
    let _ = status.wait_with_output();
    Ok("done".to_owned())
}

fn script_path() -> Result<PathBuf, String> {
    // Use CARGO_MANIFEST_DIR for reliable path resolution
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(&dir).join("src/publish.js");
        if p.exists() {
            return Ok(p);
        }
    }
    Err("src/publish.js not found. Ensure running from project root.".to_owned())
}
