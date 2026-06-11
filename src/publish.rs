//! WeChat backend automation — calls Playwright Node script.

use std::path::PathBuf;
use std::process::Command;

pub fn login() -> Result<String, String> {
    run_script("login")
}

pub fn auto_configure(_media_id: &str) -> Result<String, String> {
    run_script("configure")
}

fn run_script(mode: &str) -> Result<String, String> {
    let js = script_path()?;
    let node_modules = npm_global_root()?;
    Command::new("node")
        .arg(&js)
        .arg(mode)
        .env("NODE_PATH", &node_modules)
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .map_err(|e| format!("node: {e}"))?;
    // Don't wait — Playwright keeps browser open
    Ok("started".to_owned())
}

fn script_path() -> Result<PathBuf, String> {
    if let Ok(dir) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(&dir).join("src/publish.js");
        if p.exists() {
            return Ok(p);
        }
    }
    Err("src/publish.js not found".to_owned())
}

fn npm_global_root() -> Result<String, String> {
    if let Ok(p) = std::env::var("HOME") {
        let np = format!("{p}/.npm/_npx/31e32ef8478fbf80/node_modules");
        if PathBuf::from(&np).exists() {
            return Ok(np);
        }
    }
    Err("node_modules not found".to_owned())
}
