//! Browser automation via Node.js Playwright (frameLocator — proven).

use std::path::PathBuf;
use std::process::Command;

pub fn login() -> Result<String, String> {
    run_script("login")
}
pub fn auto_configure(_: &str) -> Result<String, String> {
    run_script("configure")
}

fn run_script(mode: &str) -> Result<String, String> {
    let js = script_path()?;
    let modules = npm_module_dir()?;
    Command::new("node")
        .arg(&js)
        .arg(mode)
        .env("NODE_PATH", &modules)
        .spawn()
        .map_err(|e| format!("node: {e}"))?;
    Ok("started".to_owned())
}

fn script_path() -> Result<PathBuf, String> {
    if let Ok(d) = std::env::var("CARGO_MANIFEST_DIR") {
        let p = PathBuf::from(&d).join("src/publish.js");
        if p.exists() {
            return Ok(p);
        }
    }
    Err("src/publish.js not found".to_owned())
}

fn npm_module_dir() -> Result<String, String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let p = format!("{home}/.npm/_npx/31e32ef8478fbf80/node_modules");
    if PathBuf::from(&p).exists() {
        Ok(p)
    } else {
        Err("node_modules not found".to_owned())
    }
}
