//! WeChat backend automation via headless_chrome (CDP).
//!
//! Key design decisions:
//! - `--user-data-dir` → persistent Chrome profile, scan QR once, reuse forever
//! - XPath text-based selectors → stable against WeChat DOM hash changes
//! - API (push) + Browser (settings) → two-phase "semi-auto" workflow

use headless_chrome::{Browser, LaunchOptions, Tab};
use std::path::PathBuf;
use std::time::Duration;

/// Where the persistent Chrome profile lives (relative to vault).
fn profile_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(format!(
        "{home}/Library/Mobile Documents/com~apple~CloudDocs/ObsidianMain/.moonpub/chrome-profile"
    ))
}

// ── public API ───────────────────────────────────────────────────────────────

/// Open headed browser, wait for manual WeChat login, save profile.
pub fn login() -> Result<String, String> {
    let browser = launch_headed()?;
    let tab = browser.new_tab().map_err(|e| format!("tab: {e}"))?;

    tab.navigate_to("https://mp.weixin.qq.com")
        .map_err(|e| format!("navigate: {e}"))?;

    println!("请在浏览器中微信扫码登录（仅需一次）...");
    wait_for_login(&tab, 120)?;
    println!("✅ 登录成功，Profile 已保存。");
    Ok("Login saved.".to_owned())
}

/// Auto-configure draft in browser (requires prior login via `moonpub login`).
/// Steps: open draft → original → source → save → preview
pub fn auto_configure(media_id: &str) -> Result<String, String> {
    let browser = launch_headed()?;
    let tab = browser.new_tab().map_err(|e| format!("tab: {e}"))?;

    // Step 1 — Ensure logged in
    tab.navigate_to("https://mp.weixin.qq.com/cgi-bin/home")
        .map_err(|e| format!("navigate: {e}"))?;
    tab.wait_until_navigated().ok();
    std::thread::sleep(Duration::from_secs(2));

    if !tab.get_url().contains("cgi-bin/home") {
        println!("需要登录，请在浏览器中扫码...");
        tab.navigate_to("https://mp.weixin.qq.com").ok();
        wait_for_login(&tab, 120)?;
    }

    // Step 2 — Navigate to draft editor
    let draft_url = format!(
        "https://mp.weixin.qq.com/cgi-bin/appmsg?\
         t=media/appmsg_edit_v2&action=edit&isNew=1&type=77&lang=zh_CN&vid={media_id}"
    );
    tab.navigate_to(&draft_url)
        .map_err(|e| format!("draft nav: {e}"))?;
    tab.wait_until_navigated().ok();
    std::thread::sleep(Duration::from_secs(5));

    // Step 3 — Original declaration (via text-based JS)
    click_by_text(&tab, "未声明");
    std::thread::sleep(Duration::from_secs(2));
    click_by_text(&tab, "已阅读并同意");
    click_by_text(&tab, "确定");
    std::thread::sleep(Duration::from_secs(2));
    println!("  ✅ 原创");

    // Step 4 — Source
    tab.evaluate(
        "document.querySelector('#js_claim_source_area')?.click()",
        false,
    )
    .ok();
    std::thread::sleep(Duration::from_secs(2));
    click_by_text(&tab, "个人观点，仅供参考");
    click_by_text(&tab, "确认");
    std::thread::sleep(Duration::from_secs(2));
    println!("  ✅ 来源");

    // Step 5 — Save
    click_by_text(&tab, "保存为草稿");
    std::thread::sleep(Duration::from_secs(3));
    println!("  ✅ 保存");

    // Step 6 — Preview (open QR dialog)
    click_by_text(&tab, "预览");
    std::thread::sleep(Duration::from_secs(2));
    // Check "通过公众号列表预览" if visible
    tab.evaluate(
        "var a=document.querySelectorAll('label');for(var i=0;i<a.length;i++){if(a[i].textContent.includes('公众号列表预览'))a[i].click()}",
        false,
    ).ok();
    click_by_text(&tab, "确定");
    std::thread::sleep(Duration::from_secs(2));
    println!("  ✅ 预览二维码已弹出");
    println!("=== 请扫码预览，确认后在浏览器点「发表」===");

    // Keep browser open
    std::thread::sleep(Duration::from_secs(300));
    Ok("done".to_owned())
}

// ── helpers ──────────────────────────────────────────────────────────────────

fn launch_headed() -> Result<Browser, String> {
    let dir = profile_dir();
    std::fs::create_dir_all(&dir).ok();

    let opts = LaunchOptions::default_builder()
        .headless(false)
        .user_data_dir(Some(dir))
        .build()
        .map_err(|e| format!("launch: {e}"))?;

    Browser::new(opts).map_err(|e| format!("browser: {e}"))
}

/// Wait for user to login by polling URL. Times out after `timeout_secs`.
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

/// Click the first visible element whose textContent equals `text`.
fn click_by_text(tab: &Tab, text: &str) {
    let js = format!(
        "var a=document.querySelectorAll('*');\
         for(var i=0;i<a.length;i++){{\
           var el=a[i];\
           if(el.offsetHeight>0 && el.textContent.trim()==='{text}'){{el.click();break;}}\
         }}"
    );
    tab.evaluate(&js, false).ok();
}
