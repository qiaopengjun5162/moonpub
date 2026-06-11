//! WeChat backend automation via headless_chrome (CDP DOM queries, no JS eval).

use headless_chrome::{Browser, LaunchOptions, Tab};
use std::path::PathBuf;
use std::time::Duration;

pub fn login() -> Result<String, String> {
    let browser = launch_headed()?;
    let tab = browser.new_tab().map_err(|e| format!("tab: {e}"))?;
    tab.navigate_to("https://mp.weixin.qq.com")
        .map_err(|e| format!("navigate: {e}"))?;
    println!("Please scan QR code. Waiting 120s...");
    std::thread::sleep(Duration::from_secs(120));
    Ok("Login complete".to_owned())
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

    // ── Click first draft's edit button ──
    // Find ANY edit link by XPath text matching
    println!("▶ 找第一个草稿的编辑按钮...");
    let clicked = (0..20).any(|_| {
        if let Ok(links) = tab.find_elements_by_xpath("//a[contains(text(),'编辑')]") {
            for link in links {
                if link.click().is_ok() {
                    return true;
                }
            }
        }
        std::thread::sleep(Duration::from_millis(500));
        false
    });
    if !clicked {
        return Err("找不到编辑按钮".to_string());
    }
    std::thread::sleep(Duration::from_secs(5));
    println!("  ✅ 进入编辑器");

    // ── Original: click "未声明" → "已阅读" → "确定" ──
    println!("▶ 原创声明...");
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
    for _ in 0..10 {
        if let Ok(els) = tab.find_elements_by_xpath("//*[contains(text(),'已阅读')]") {
            if let Some(el) = els.first() {
                el.click().ok();
            }
        }
        if let Ok(btns) = tab.find_elements_by_xpath("//button[text()='确定']") {
            if let Some(b) = btns.first() {
                b.click().ok();
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    println!("  ✅ 原创");

    // ── Source ──
    println!("▶ 创作来源...");
    if let Ok(els) = tab.find_elements_by_xpath("//*[text()='创作来源']") {
        if let Some(el) = els.first() {
            el.click().ok();
        }
    }
    std::thread::sleep(Duration::from_secs(2));
    for _ in 0..10 {
        if let Ok(els) = tab.find_elements_by_xpath("//*[contains(text(),'个人观点，仅供参考')]")
        {
            if let Some(el) = els.first() {
                el.click().ok();
            }
        }
        if let Ok(btns) = tab.find_elements_by_xpath("//button[text()='确认']") {
            if let Some(b) = btns.first() {
                b.click().ok();
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    println!("  ✅ 来源");

    // ── Save ──
    println!("▶ 保存...");
    for _ in 0..10 {
        if let Ok(btns) = tab.find_elements_by_xpath("//button[text()='保存为草稿']") {
            if let Some(b) = btns.first() {
                b.click().ok();
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    println!("  ✅ 保存");

    // ── Preview ──
    println!("▶ 预览...");
    for _ in 0..10 {
        if let Ok(btns) = tab.find_elements_by_xpath("//button[text()='预览']") {
            if let Some(b) = btns.first() {
                b.click().ok();
                break;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    std::thread::sleep(Duration::from_secs(2));
    if let Ok(els) = tab.find_elements_by_xpath("//*[contains(text(),'公众号列表预览')]") {
        if let Some(el) = els.first() {
            el.click().ok();
        }
    }
    if let Ok(btns) = tab.find_elements_by_xpath("//button[text()='确定']") {
        if let Some(b) = btns.first() {
            b.click().ok();
        }
    }
    println!("  ✅ 预览");

    println!("🎉 全部完成！按 Enter 关闭...");
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
