//! WeChat backend automation — pure Rust, CDP DOM only, no JS eval.

use headless_chrome::{Browser, LaunchOptions};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

type HlcTab = Arc<headless_chrome::Tab>;

pub fn login() -> Result<String, String> {
    Ok("scan QR in browser".to_owned())
}

pub fn auto_configure(_mid: &str) -> Result<String, String> {
    let tab = open_tab()?;

    // Login
    println!("▶ Login...");
    tab.navigate_to("https://mp.weixin.qq.com")
        .map_err(|e| format!("nav: {e}"))?;
    std::thread::sleep(Duration::from_secs(5));
    if !tab.get_url().contains("cgi-bin/home") {
        println!("  Scan QR. Waiting 120s...");
        std::thread::sleep(Duration::from_secs(120));
    }
    println!("  ✅ Logged in");

    // Drafts list
    let home = tab.get_url();
    let token = home
        .split("token=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .unwrap_or("");
    let list_url = format!(
        "https://mp.weixin.qq.com/cgi-bin/appmsg?begin=0&count=10&type=77&action=list_card&token={token}&lang=zh_CN"
    );
    tab.navigate_to(&list_url)
        .map_err(|e| format!("list: {e}"))?;
    std::thread::sleep(Duration::from_secs(6));

    // Click edit
    println!("▶ Click edit...");
    let mut entered = false;
    for _ in 0..20 {
        // Find action area, click the 2nd icon button
        if let Ok(actions) = tab.find_elements(".weui-desktop-card__action") {
            if let Some(area) = actions.first() {
                area.click().ok();
                std::thread::sleep(Duration::from_secs(3));
                if tab.get_url().contains("appmsg_edit") {
                    entered = true;
                    break;
                }
            }
        }
        // Fallback: find by "编辑" text span and click adjacent element
        if let Ok(els) = tab.find_elements_by_xpath("//span[text()='编辑']") {
            if let Some(el) = els.first() {
                el.click().ok();
            }
            std::thread::sleep(Duration::from_secs(3));
            if tab.get_url().contains("appmsg_edit") {
                entered = true;
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    if !entered {
        println!("  ⚠ Manual entry needed — click edit in browser, then Enter...");
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).ok();
    }
    println!("  ✅ Editor");
    std::thread::sleep(Duration::from_secs(4));

    // Original
    println!("▶ Original...");
    for _ in 0..40 {
        if let Ok(els) = tab.find_elements_by_xpath("//*[text()='未声明']") {
            if let Some(el) = els.first() {
                el.click().ok();
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    std::thread::sleep(Duration::from_secs(2));
    xpath_click(&tab, "//*[contains(text(),'已阅读')]");
    xpath_click(&tab, "//button[text()='确定']");
    println!("  ✅");

    // Source
    println!("▶ Source...");
    xpath_click(&tab, "//*[contains(text(),'创作来源')]");
    std::thread::sleep(Duration::from_secs(2));
    xpath_click(&tab, "//*[contains(text(),'个人观点，仅供参考')]");
    xpath_click(&tab, "//button[text()='确认']");
    println!("  ✅");

    // Account card
    println!("▶ Account card...");
    css_click(&tab, "#editor_showmore");
    std::thread::sleep(Duration::from_secs(1));
    css_click(&tab, "#js_editor_insertProfile");
    std::thread::sleep(Duration::from_secs(2));
    xpath_click(&tab, "//button[text()='确定']");
    println!("  ✅");

    // Save
    println!("▶ Save...");
    xpath_click(&tab, "//button[text()='保存为草稿']");
    std::thread::sleep(Duration::from_secs(3));
    println!("  ✅");

    // Preview
    println!("▶ Preview...");
    xpath_click(&tab, "//button[text()='预览']");
    std::thread::sleep(Duration::from_secs(2));
    xpath_click(&tab, "//*[contains(text(),'公众号列表预览')]");
    xpath_click(&tab, "//button[text()='确定']");
    println!("  ✅");

    println!("Done. Browser stays open. Enter to close...");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).ok();
    Ok("done".to_owned())
}

fn xpath_click(tab: &HlcTab, xpath: &str) {
    if let Ok(els) = tab.find_elements_by_xpath(xpath) {
        if let Some(el) = els.first() {
            el.click().ok();
        }
    }
}

fn css_click(tab: &HlcTab, css: &str) {
    if let Ok(els) = tab.find_elements(css) {
        if let Some(el) = els.first() {
            el.click().ok();
        }
    }
}

fn open_tab() -> Result<HlcTab, String> {
    let home = std::env::var("HOME").unwrap_or_default();
    let dir = PathBuf::from(format!(
        "{home}/Library/Mobile Documents/com~apple~CloudDocs/ObsidianMain/.moonpub/chrome-profile"
    ));
    std::fs::create_dir_all(&dir).ok();
    Browser::new(
        LaunchOptions::default_builder()
            .headless(false)
            .user_data_dir(Some(dir))
            .build()
            .map_err(|e| format!("{e}"))?,
    )
    .map_err(|e| format!("browser: {e}"))?
    .new_tab()
    .map_err(|e| format!("tab: {e}"))
}
