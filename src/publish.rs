//! WeChat backend automation — pure Rust headless_chrome 1.0.22.
//! Uses XPath + call_js_fn (per-node JS, not global eval).

use headless_chrome::{Browser, LaunchOptions};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

type HlcTab = Arc<headless_chrome::Tab>;

pub fn login() -> Result<String, String> {
    let tab = open_browser()?;
    tab.navigate_to("https://mp.weixin.qq.com")
        .map_err(|e| format!("{e}"))?;
    println!("Scan QR. Waiting 120s...");
    std::thread::sleep(Duration::from_secs(120));
    Ok("done".to_owned())
}

pub fn auto_configure(_media_id: &str) -> Result<String, String> {
    let tab = open_browser()?;

    // ── Login ──
    println!("▶ 检查登录...");
    tab.navigate_to("https://mp.weixin.qq.com")
        .map_err(|e| format!("nav: {e}"))?;
    std::thread::sleep(Duration::from_secs(4));
    if !tab.get_url().contains("cgi-bin/home") {
        println!("请扫码，等 120s...");
        std::thread::sleep(Duration::from_secs(120));
    }
    println!("  ✅ 已登录");

    // ── Navigate to drafts list ──
    let home_url = tab.get_url();
    let token = home_url
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

    // ── Click edit button ──
    println!("▶ 找编辑按钮...");
    let mut entered = false;
    // Try XPath: find the span with "编辑" text, get preceding-sibling <a>
    let edit_xpath = "//div[contains(@class,'weui-desktop-card__action')][1]//span[text()='编辑']/preceding-sibling::a";
    if let Ok(els) = tab.find_elements_by_xpath(edit_xpath) {
        if let Some(btn) = els.first() {
            // call_js_fn: per-node JS, bypasses global CSP
            let _ = btn.call_js_fn("function(){this.click();}", vec![], false);
            std::thread::sleep(Duration::from_secs(4));
            if tab.get_url().contains("appmsg_edit") {
                entered = true;
            }
        }
    }
    // Fallback: click the action area itself
    if !entered {
        if let Ok(actions) = tab.find_elements(".weui-desktop-card__action") {
            if let Some(act) = actions.first() {
                act.click().ok();
                std::thread::sleep(Duration::from_secs(4));
                if tab.get_url().contains("appmsg_edit") {
                    entered = true;
                }
            }
        }
    }
    if !entered {
        println!("🚨 进不去编辑器！浏览器保持打开供手动操作。");
        println!("手动点击编辑后按 Enter 继续...");
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).ok();
    } else {
        println!("  ✅ 进入编辑器");
    }
    std::thread::sleep(Duration::from_secs(4));

    // ── Original ──
    println!("▶ 原创声明...");
    for _ in 0..40 {
        if let Ok(els) = tab.find_elements_by_xpath("//span[text()='未声明']/..") {
            if let Some(el) = els.first() {
                let _ = el.call_js_fn("function(){this.click();}", vec![], false);
                std::thread::sleep(Duration::from_millis(500));
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    std::thread::sleep(Duration::from_secs(2));
    if let Ok(cbs) = tab.find_elements_by_xpath("//span[contains(text(),'已阅读')]") {
        if let Some(cb) = cbs.first() {
            cb.click().ok();
        }
    }
    if let Ok(btns) = tab.find_elements_by_xpath("//button[text()='确定']") {
        if let Some(btn) = btns.first() {
            btn.click().ok();
        }
    }
    println!("  ✅ 原创");

    // ── Source ──
    println!("▶ 创作来源...");
    if let Ok(els) = tab.find_elements_by_xpath("//*[contains(text(),'创作来源')]") {
        if let Some(el) = els.first() {
            el.click().ok();
        }
    }
    std::thread::sleep(Duration::from_secs(2));
    if let Ok(els) = tab.find_elements_by_xpath("//*[contains(text(),'个人观点，仅供参考')]")
    {
        if let Some(el) = els.first() {
            el.click().ok();
        }
    }
    if let Ok(btns) = tab.find_elements_by_xpath("//button[text()='确认']") {
        if let Some(btn) = btns.first() {
            btn.click().ok();
        }
    }
    println!("  ✅ 来源");

    // ── Account card ──
    println!("▶ 账号名片...");
    if let Ok(els) = tab.find_elements("#editor_showmore") {
        if let Some(el) = els.first() {
            el.click().ok();
        }
    }
    std::thread::sleep(Duration::from_secs(1));
    if let Ok(els) = tab.find_elements("#js_editor_insertProfile") {
        if let Some(el) = els.first() {
            el.click().ok();
        }
    }
    std::thread::sleep(Duration::from_secs(2));
    if let Ok(btns) = tab.find_elements_by_xpath("//button[text()='确定']") {
        if let Some(btn) = btns.first() {
            btn.click().ok();
        }
    }
    println!("  ✅ 名片");

    // ── Save ──
    println!("▶ 保存...");
    if let Ok(btns) = tab.find_elements_by_xpath("//button[text()='保存为草稿']") {
        if let Some(btn) = btns.first() {
            btn.click().ok();
        }
    }
    std::thread::sleep(Duration::from_secs(3));
    println!("  ✅ 保存");

    // ── Preview ──
    println!("▶ 预览...");
    if let Ok(btns) = tab.find_elements_by_xpath("//button[text()='预览']") {
        if let Some(btn) = btns.first() {
            btn.click().ok();
        }
    }
    std::thread::sleep(Duration::from_secs(2));
    if let Ok(els) = tab.find_elements_by_xpath("//*[contains(text(),'公众号列表预览')]") {
        if let Some(el) = els.first() {
            el.click().ok();
        }
    }
    if let Ok(btns) = tab.find_elements_by_xpath("//button[text()='确定']") {
        if let Some(btn) = btns.first() {
            btn.click().ok();
        }
    }
    println!("  ✅ 预览");

    println!("🎉 完成！按 Enter 关闭...");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).ok();
    Ok("done".to_owned())
}

fn profile_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    PathBuf::from(format!(
        "{home}/Library/Mobile Documents/com~apple~CloudDocs/ObsidianMain/.moonpub/chrome-profile"
    ))
}

fn open_browser() -> Result<HlcTab, String> {
    let dir = profile_dir();
    std::fs::create_dir_all(&dir).ok();
    let browser = Browser::new(
        LaunchOptions::default_builder()
            .headless(false)
            .user_data_dir(Some(dir))
            .build()
            .map_err(|e| format!("launch: {e}"))?,
    )
    .map_err(|e| format!("browser: {e}"))?;
    browser.new_tab().map_err(|e| format!("{e}"))
}
