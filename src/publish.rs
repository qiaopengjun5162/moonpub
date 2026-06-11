//! WeChat backend automation via playwright-cli (direct Rust calls).
//! Steps: open draft → original → source → AI cover → template ending → save.

use std::path::PathBuf;
use std::process::Command;
use std::thread;
use std::time::Duration;

pub fn login(vault: &PathBuf) -> Result<String, String> {
    std::fs::create_dir_all(vault.join(".playwright-cli")).map_err(|e| format!("mkdir: {e}"))?;
    pw(vault, &["open", "https://mp.weixin.qq.com", "--no-headed"])?;
    let _ = pw(vault, &["state-load"]);
    if pw_eval_bool(vault, "location.href.includes('/cgi-bin/home')") {
        pw(vault, &["state-save"])?;
        pw(vault, &["close"])?;
        return Ok("Already logged in.".to_owned());
    }
    pw(vault, &["close"])?;
    pw(vault, &["open", "https://mp.weixin.qq.com", "--headed"])?;
    println!("Please scan QR code in browser, then press Enter...");
    std::io::stdin().read_line(&mut String::new()).ok();
    pw(vault, &["state-save"])?;
    pw(vault, &["close"])?;
    Ok("Session saved.".to_owned())
}

/// Full backend auto-config after draft push.
pub fn auto_configure(media_id: &str) -> Result<String, String> {
    let vault = vault_dir()?;
    let state = vault
        .join(".playwright-cli")
        .join("storage-state-2026-06-10T10-29-23-983Z.json");
    if !state.exists() {
        open_in_browser(media_id)?;
        return Err("No session. Browser opened for manual setup.".to_owned());
    }

    // Open backend home, load session
    if pw(
        &vault,
        &[
            "open",
            "https://mp.weixin.qq.com/cgi-bin/home",
            "--no-headed",
        ],
    )
    .is_err()
    {
        open_in_browser(media_id)?;
        return Err("playwright-cli unavailable. Browser opened for manual setup.".to_owned());
    }
    thread::sleep(Duration::from_secs(2));
    let _ = pw(&vault, &["state-load"]);
    thread::sleep(Duration::from_secs(2));

    // Step 1: Get token and navigate to draft editor
    let token = pw_eval(
        &vault,
        "new URL(location.href).searchParams.get('token') || ''",
    );
    let draft_url = format!(
        "https://mp.weixin.qq.com/cgi-bin/appmsg?t=media/appmsg_edit_v2&action=edit&isNew=1&type=77&lang=zh_CN&vid={media_id}&token={token}"
    );
    pw(&vault, &["open", &draft_url, "--no-headed"])?;
    thread::sleep(Duration::from_secs(4));

    // Step 2: Set original declaration
    pw_eval(
        &vault,
        r#"
        (function(){
            var all=document.querySelectorAll('*');
            for(var i=0;i<all.length;i++){
                if(all[i].textContent.trim()==='未声明'){all[i].parentElement.click();break;}
            }
        })()
    "#,
    );
    thread::sleep(Duration::from_secs(2));
    pw_eval(
        &vault,
        r#"
        (function(){
            var all=document.querySelectorAll('*');
            for(var i=0;i<all.length;i++){if(all[i].textContent.includes('已阅读并同意'))all[i].click();}
            var btns=document.querySelectorAll('button');
            for(var j=0;j<btns.length;j++){if(btns[j].textContent.trim()==='确定'){btns[j].click();return;}}
        })()
    "#,
    );
    thread::sleep(Duration::from_secs(2));

    // Step 3: Set source
    pw_eval(
        &vault,
        "document.querySelector('#js_claim_source_area')?.click()",
    );
    thread::sleep(Duration::from_secs(2));
    pw_eval(
        &vault,
        r#"
        (function(){
            var all=document.querySelectorAll('*');
            for(var i=0;i<all.length;i++){
                if(all[i].textContent.trim()==='个人观点，仅供参考'&&all[i].children.length===0){all[i].click();break;}
            }
            var btns=document.querySelectorAll('button');
            for(var j=0;j<btns.length;j++){if(btns[j].textContent.trim()==='确认'){btns[j].click();return;}}
        })()
    "#,
    );
    thread::sleep(Duration::from_secs(2));

    // Step 4: Save draft
    pw_eval(
        &vault,
        r#"
        (function(){
            var btns=document.querySelectorAll('button');
            for(var i=0;i<btns.length;i++){
                if(btns[i].textContent.trim()==='保存为草稿'){btns[i].click();return;}
            }
        })()
    "#,
    );
    thread::sleep(Duration::from_secs(2));

    let _ = pw(&vault, &["state-save"]);
    let _ = pw(&vault, &["close"]);

    Ok("backend configured: 原创 + 来源 + 保存".to_owned())
}

/// Open the WeChat draft editor in browser (fallback).
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

fn pw(vault: &PathBuf, args: &[&str]) -> Result<String, String> {
    let out = Command::new("npx")
        .arg("@playwright/cli")
        .args(args)
        .current_dir(vault)
        .output()
        .map_err(|e| format!("npx: {e}"))?;
    if !out.status.success() {
        return Err(String::from_utf8_lossy(&out.stderr).to_string());
    }
    Ok(String::from_utf8_lossy(&out.stdout).to_string())
}

fn pw_eval(vault: &PathBuf, js: &str) -> String {
    match Command::new("npx")
        .args(["@playwright/cli", "eval", js])
        .current_dir(vault)
        .output()
    {
        Ok(o) => String::from_utf8_lossy(&o.stdout).to_string(),
        Err(_) => String::new(),
    }
}

fn pw_eval_bool(vault: &PathBuf, js: &str) -> bool {
    pw_eval(vault, js).contains("true")
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
