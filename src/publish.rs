//! WeChat backend automation via headless_chrome (CDP).
//!
//! Key design:
//! - `--user-data-dir` → persistent Chrome profile, scan QR once, reuse forever
//! - All selectors use textContent matching → stable against DOM hash changes
//! - `wait_and_execute` with boolean return → avoid race conditions

use headless_chrome::{Browser, LaunchOptions, Tab};
use std::path::PathBuf;
use std::time::Duration;

// ── public API ───────────────────────────────────────────────────────────────

pub fn login() -> Result<String, String> {
    let browser = launch_headed()?;
    let tab = browser.new_tab().map_err(|e| format!("tab: {e}"))?;
    tab.navigate_to("https://mp.weixin.qq.com")
        .map_err(|e| format!("navigate: {e}"))?;
    println!("请在浏览器中微信扫码登录（仅需一次）...");
    wait_for_login(&tab, 120)?;
    Ok("Login saved.".to_owned())
}

/// Full automation after API push: original → reward → source → collection → cover → preview.
pub fn auto_configure(media_id: &str) -> Result<String, String> {
    let browser = launch_headed()?;
    let tab = browser.new_tab().map_err(|e| format!("tab: {e}"))?;

    // Ensure logged in
    tab.navigate_to("https://mp.weixin.qq.com/cgi-bin/home")
        .ok();
    std::thread::sleep(Duration::from_secs(2));
    if !tab.get_url().contains("cgi-bin/home") {
        println!("需要登录，扫码中...");
        tab.navigate_to("https://mp.weixin.qq.com").ok();
        wait_for_login(&tab, 120)?;
    }

    let draft_url = format!(
        "https://mp.weixin.qq.com/cgi-bin/appmsg?\
         t=media/appmsg_edit_v2&action=edit&isNew=1&type=77&lang=zh_CN&vid={media_id}"
    );
    tab.navigate_to(&draft_url)
        .map_err(|e| format!("draft nav: {e}"))?;
    tab.wait_until_navigated().ok();
    std::thread::sleep(Duration::from_secs(3));

    // ── DOM-level login check: detect "登录超时" impostor page ──────────────
    let is_timeout = (0..10).any(|_| {
        if let Ok(res) = tab.evaluate(
            "return document.body.textContent.includes('登录超时，请重新');",
            false,
        ) {
            if res.value.and_then(|v| v.as_bool()).unwrap_or(false) {
                return true;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
        false
    });

    if is_timeout {
        println!("⚠️ Session 过期（登录超时）。重新登录...");
        tab.navigate_to("https://mp.weixin.qq.com").ok();
        wait_for_login(&tab, 120)?;
        tab.navigate_to(&draft_url)
            .map_err(|e| format!("draft nav: {e}"))?;
        tab.wait_until_navigated().ok();
    }
    // Editor renders async
    std::thread::sleep(Duration::from_secs(8));
    // ────────────────────────────────────────────────────────────────────────

    // ── Original ──
    println!("▶ 原创声明...");
    wait_and_execute(
        &tab,
        "var a=document.querySelectorAll('*');for(var i=0;i<a.length;i++){if(a[i].textContent.trim()==='未声明'){a[i].parentElement.click();return true;}}return false;",
        40,
    )?;
    wait_and_execute(
        &tab,
        "var a=document.querySelectorAll('*');for(var i=0;i<a.length;i++){if(a[i].textContent.includes('已阅读'))a[i].click();}var b=document.querySelectorAll('button');for(var j=0;j<b.length;j++){if(b[j].textContent.trim()==='确定'){b[j].click();return true;}}return false;",
        10,
    )?;
    println!("  ✅ 原创");

    // ── Reward (赞赏) ──
    println!("▶ 开启赞赏...");
    wait_and_execute(&tab, "var b=document.querySelectorAll('button');for(var i=0;i<b.length;i++){if(b[i].textContent.includes('赞赏')){b[i].click();return true;}}return false;", 10).ok();
    std::thread::sleep(Duration::from_millis(500));
    wait_and_execute(&tab, "var a=document.querySelectorAll('*');for(var i=0;i<a.length;i++){if(a[i].textContent.trim()==='开启赞赏'||a[i].textContent.trim()==='赞赏'){a[i].click();return true;}}return false;", 10).ok();
    println!("  ✅ 赞赏");

    // ── Source ──
    println!("▶ 创作来源...");
    tab.evaluate(
        "document.querySelector('#js_claim_source_area')?.click()",
        false,
    )
    .ok();
    std::thread::sleep(Duration::from_secs(2));
    wait_and_execute(
        &tab,
        "var a=document.querySelectorAll('*');for(var i=0;i<a.length;i++){if(a[i].textContent.trim()==='个人观点，仅供参考'){a[i].click();return true;}}return false;",
        10,
    )?;
    wait_and_execute(
        &tab,
        "var b=document.querySelectorAll('button');for(var j=0;j<b.length;j++){if(b[j].textContent.trim()==='确认'){b[j].click();return true;}}return false;",
        10,
    )?;
    println!("  ✅ 来源");

    // ── Collection (合集) ──
    println!("▶ 文章合集...");
    wait_and_execute(&tab, "var a=document.querySelectorAll('*');for(var i=0;i<a.length;i++){if(a[i].textContent.trim()==='合集'){a[i].click();return true;}}return false;", 10).ok();
    std::thread::sleep(Duration::from_millis(800));
    // Select first collection in dropdown
    wait_and_execute(&tab, "var items=document.querySelectorAll('[class*=dropdown] li, [class*=dropdown] span, [class*=menu] li, [class*=menu] span, [class*=list] li');for(var i=0;i<items.length;i++){if(items[i].offsetHeight>0&&items[i].textContent.trim().length>0){items[i].click();return true;}}return false;", 10).ok();
    println!("  ✅ 合集");

    // ── AI Cover ──
    println!("▶ AI 封面...");
    wait_and_execute(&tab, "document.querySelector('.js_cover_btn_area')?.dispatchEvent(new MouseEvent('mouseover',{bubbles:true}));return true;", 5).ok();
    std::thread::sleep(Duration::from_millis(500));
    wait_and_execute(&tab, "var a=document.querySelectorAll('*');for(var i=0;i<a.length;i++){if(a[i].textContent.includes('AI')&&a[i].textContent.includes('配图')){a[i].click();return true;}}return false;", 10).ok();
    std::thread::sleep(Duration::from_secs(3));
    // Wait for AI image generation, select first result
    wait_and_execute(&tab, "var imgs=document.querySelectorAll('img');for(var i=imgs.length-1;i>=0;i--){if(imgs[i].src.includes('mpimageai')&&imgs[i].naturalWidth>500){imgs[i].click();return true;}}return false;", 40).ok();
    wait_and_execute(&tab, "var b=document.querySelectorAll('button');for(var j=0;j<b.length;j++){if(b[j].textContent.trim()==='下一步'||b[j].textContent.trim()==='确定'){b[j].click();return true;}}return false;", 10).ok();
    std::thread::sleep(Duration::from_secs(2));
    wait_and_execute(&tab, "var b=document.querySelectorAll('button');for(var j=0;j<b.length;j++){if(b[j].textContent.trim()==='确定'){b[j].click();return true;}}return false;", 10).ok();
    println!("  ✅ 封面");

    // ── Save ──
    println!("▶ 保存草稿...");
    wait_and_execute(
        &tab,
        "var b=document.querySelectorAll('button');for(var i=0;i<b.length;i++){if(b[i].textContent.trim()==='保存为草稿'){b[i].click();return true;}}return false;",
        10,
    )?;
    std::thread::sleep(Duration::from_secs(3));
    println!("  ✅ 保存");

    // ── Preview ──
    println!("▶ 生成预览...");
    wait_and_execute(
        &tab,
        "var b=document.querySelectorAll('button');for(var i=0;i<b.length;i++){if(b[i].textContent.trim()==='预览'){b[i].click();return true;}}return false;",
        10,
    )?;
    std::thread::sleep(Duration::from_secs(2));
    tab.evaluate("var a=document.querySelectorAll('label');for(var i=0;i<a.length;i++){if(a[i].textContent.includes('公众号列表预览'))a[i].click();}", false).ok();
    wait_and_execute(&tab, "var b=document.querySelectorAll('button');for(var j=0;j<b.length;j++){if(b[j].textContent.trim()==='确定'){b[j].click();return true;}}return false;", 10).ok();

    println!("🎉 全部完成！请在浏览器确认后点「发表」。");
    println!("按 Enter 关闭浏览器...");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).ok();
    Ok("done".to_owned())
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn profile_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(format!(
        "{home}/Library/Mobile Documents/com~apple~CloudDocs/ObsidianMain/.moonpub/chrome-profile"
    ))
}

fn launch_headed() -> Result<Browser, String> {
    let dir = profile_dir();
    std::fs::create_dir_all(&dir).ok();
    Browser::new(
        LaunchOptions::default_builder()
            .headless(false)
            .user_data_dir(Some(dir))
            .build()
            .map_err(|e| format!("launch: {e}"))?,
    )
    .map_err(|e| format!("browser: {e}"))
}

fn wait_for_login(tab: &Tab, timeout_secs: u64) -> Result<(), String> {
    for _ in 0..(timeout_secs * 2) {
        std::thread::sleep(Duration::from_millis(500));
        let url = tab.get_url();
        if url.contains("cgi-bin/home") || url.contains("appmsg_edit") {
            return Ok(());
        }
    }
    Err("Login timeout".to_owned())
}

/// Execute JS that returns `true` on success. Retry up to `max_retries` times, 500ms apart.
fn wait_and_execute(tab: &Tab, js: &str, max_retries: usize) -> Result<(), String> {
    let wrapped = format!("return (function(){{{js}}})();");
    for _ in 0..max_retries {
        if let Ok(result) = tab.evaluate(&wrapped, false) {
            if let Some(val) = result.value {
                if val.as_bool().unwrap_or(false) {
                    return Ok(());
                }
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Err(format!("timeout after {max_retries} retries"))
}
