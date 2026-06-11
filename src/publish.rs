//! WeChat backend automation via headless_chrome (CDP DOM + mouse events).

use headless_chrome::{Browser, LaunchOptions};
use std::path::PathBuf;
use std::time::Duration;

pub fn login() -> Result<String, String> {
    let browser = launch_headed()?;
    let tab = browser.new_tab().map_err(|e| format!("tab: {e}"))?;
    tab.navigate_to("https://mp.weixin.qq.com")
        .map_err(|e| format!("nav: {e}"))?;
    println!("Scan QR code. Waiting 120s...");
    std::thread::sleep(Duration::from_secs(120));
    Ok("done".to_owned())
}

pub fn auto_configure(_media_id: &str) -> Result<String, String> {
    let browser = launch_headed()?;
    let tab = browser.new_tab().map_err(|e| format!("tab: {e}"))?;

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

    // ── Drafts list ──
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
    std::thread::sleep(Duration::from_secs(5));

    // ── Hover first card to reveal edit button, then click ──
    println!("▶ 悬停第一张卡片...");
    let mut clicked = false;
    for _ in 0..20 {
        // Find any visible card element
        if let Ok(cards) = tab.find_elements_by_xpath("//*[contains(text(),'更新于')]") {
            if let Some(el) = cards.first() {
                // Get element position, move mouse there to trigger hover
                if let Ok(model) = el.get_box_model() {
                    let x = model.content[0] + (model.content[2] - model.content[0]) / 2.0;
                    let y = model.content[1] + (model.content[3] - model.content[1]) / 2.0;
                    tab.move_mouse_to_point(x, y).ok();
                    std::thread::sleep(Duration::from_millis(500));
                }
                // Click the card itself — might open the editor directly
                el.click().ok();
                std::thread::sleep(Duration::from_secs(3));
                let url = tab.get_url();
                if url.contains("appmsg_edit") && url.contains("appmsgid=") {
                    clicked = true;
                    break;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    if !clicked {
        // Try another approach: find the appmsgid from any link, navigate directly
        let editor_url = extract_appmsgid_and_navigate(&tab, &token)?;
        tab.navigate_to(&editor_url)
            .map_err(|e| format!("editor: {e}"))?;
    }
    std::thread::sleep(Duration::from_secs(5));
    println!("  ✅ 进入编辑器");

    // ── Original ──
    println!("▶ 原创声明...");
    click_by_xpath(&tab, "//*[text()='未声明']", 40)?;
    std::thread::sleep(Duration::from_secs(2));
    click_by_xpath(&tab, "//*[contains(text(),'已阅读')]", 10).ok();
    click_by_xpath(&tab, "//button[text()='确定']", 10)?;
    println!("  ✅ 原创");

    // ── Source ──
    println!("▶ 创作来源...");
    click_by_xpath(&tab, "//*[contains(text(),'创作来源')]", 10).ok();
    std::thread::sleep(Duration::from_secs(2));
    click_by_xpath(&tab, "//*[contains(text(),'个人观点，仅供参考')]", 10)?;
    click_by_xpath(&tab, "//button[text()='确认']", 10)?;
    println!("  ✅ 来源");

    // ── Save ──
    println!("▶ 保存...");
    click_by_xpath(&tab, "//button[text()='保存为草稿']", 10)?;
    println!("  ✅ 保存");

    // ── Preview ──
    println!("▶ 预览...");
    click_by_xpath(&tab, "//button[text()='预览']", 10)?;
    std::thread::sleep(Duration::from_secs(2));
    click_by_xpath(&tab, "//*[contains(text(),'公众号列表预览')]", 10).ok();
    click_by_xpath(&tab, "//button[text()='确定']", 10).ok();
    println!("  ✅ 预览");

    println!("🎉 完成！按 Enter 关闭...");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).ok();
    Ok("done".to_owned())
}

// ── helpers ──

fn click_by_xpath(tab: &headless_chrome::Tab, xpath: &str, retries: usize) -> Result<(), String> {
    for _ in 0..retries {
        if let Ok(els) = tab.find_elements_by_xpath(xpath) {
            if let Some(el) = els.first() {
                if el.click().is_ok() {
                    return Ok(());
                }
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Err(format!("xpath timeout: {xpath}"))
}

fn extract_appmsgid_and_navigate(
    tab: &headless_chrome::Tab,
    token: &str,
) -> Result<String, String> {
    // Try to extract appmsgid from page source
    for _ in 0..20 {
        // Use get_url() to see if we're already on an editor page
        let url = tab.get_url();
        if url.contains("appmsgid=") {
            if let Some(id) = url
                .split("appmsgid=")
                .nth(1)
                .and_then(|s| s.split('&').next())
            {
                return Ok(format!(
                    "https://mp.weixin.qq.com/cgi-bin/appmsg?t=media/appmsg_edit&action=edit&type=77&appmsgid={id}&isMul=1&replaceScene=0&isSend=0&isFreePublish=0&token={token}&lang=zh_CN"
                ));
            }
        }
        // Try clicking any link with appmsg_edit
        if let Ok(links) = tab.find_elements_by_xpath("//a[contains(@href,'appmsg_edit')]") {
            if let Some(link) = links.first() {
                link.click().ok();
                std::thread::sleep(Duration::from_secs(3));
                continue;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    Err("cannot extract appmsgid".to_string())
}

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
