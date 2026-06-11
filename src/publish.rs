//! WeChat backend automation — pure Rust CDP (no JS eval).

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

    // ── Step 1: Extract token ──
    let home_url = tab.get_url();
    let token = home_url
        .split("token=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .ok_or("无法提取 token".to_string())?
        .to_string();
    println!("  Token: {token}");

    // ── Step 2: Navigate to drafts list ──
    let list_url = format!(
        "https://mp.weixin.qq.com/cgi-bin/appmsg?begin=0&count=10&type=77&action=list_card&token={token}&lang=zh_CN"
    );
    tab.navigate_to(&list_url)
        .map_err(|e| format!("list: {e}"))?;
    std::thread::sleep(Duration::from_secs(5));

    // ── Step 3: Extract appmsgid from any <a> link ──
    println!("▶ 提取 appmsgid...");
    let mut appmsgid = String::new();
    for i in 1..=40 {
        if let Ok(elements) = tab.find_elements("a[href*='appmsgid=']") {
            for elem in elements {
                if let Ok(Some(href)) = elem.get_attribute_value("href") {
                    if href.contains("appmsg_edit") && href.contains("appmsgid=") {
                        if let Some(id) = href
                            .split("appmsgid=")
                            .nth(1)
                            .and_then(|s| s.split('&').next())
                        {
                            if !id.is_empty() {
                                appmsgid = id.to_string();
                                break;
                            }
                        }
                    }
                }
            }
        }
        if !appmsgid.is_empty() {
            break;
        }
        if i % 5 == 0 {
            println!("    ... 第 {i} 次扫描...");
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    if appmsgid.is_empty() {
        println!("🚨 未提取到 appmsgid，浏览器保持打开供调试。");
        println!("按 Enter 关闭...");
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).ok();
        return Err("appmsgid not found".to_string());
    }
    println!("  appmsgid={appmsgid}");

    // ── Step 4: Navigate to editor ──
    let editor_url = format!(
        "https://mp.weixin.qq.com/cgi-bin/appmsg?t=media/appmsg_edit&action=edit&type=77&appmsgid={appmsgid}&isMul=1&replaceScene=0&isSend=0&isFreePublish=0&token={token}&lang=zh_CN"
    );
    tab.navigate_to(&editor_url)
        .map_err(|e| format!("editor nav: {e}"))?;

    // ── Step 5: State guard — wait for editor to load ──
    println!("▶ [状态守卫] 等待编辑器加载...");
    let mut loaded = false;
    for _ in 0..60 {
        if tab.wait_for_element("div#edui1_iframeholder").is_ok()
            || tab.find_element(".main_bd").is_ok()
        {
            loaded = true;
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    if !loaded {
        println!("⚠️ 编辑器加载超时！浏览器保持打开。");
        println!("按 Enter 关闭...");
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).ok();
        return Err("编辑器加载超时".to_string());
    }
    println!("  ✅ 编辑器就绪");

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

    println!("🎉 全部完成！按 Enter 关闭...");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).ok();
    Ok("done".to_owned())
}

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
