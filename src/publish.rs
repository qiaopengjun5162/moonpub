//! WeChat backend automation via Chrome DevTools Protocol (pure Rust).

use headless_chrome::{Browser, LaunchOptions};
use std::time::Duration;

/// Open headed browser for manual WeChat login.
pub fn login() -> Result<String, String> {
    let opts = LaunchOptions::default_builder()
        .headless(false)
        .build()
        .map_err(|e| format!("{e}"))?;
    let browser = Browser::new(opts).map_err(|e| format!("{e}"))?;
    let tab = browser.new_tab().map_err(|e| format!("{e}"))?;
    tab.navigate_to("https://mp.weixin.qq.com")
        .map_err(|e| format!("{e}"))?;
    std::thread::sleep(Duration::from_secs(60));
    Ok("Login complete.".to_owned())
}

/// Steps:
/// 1. Navigate to draft editor
/// 2. Click "未声明" → check "已阅读" → "确定" (original declaration)
/// 3. Click "#js_claim_source_area" → select "个人观点，仅供参考" → "确认"
/// 4. Click "保存为草稿"
pub fn auto_configure(media_id: &str) -> Result<String, String> {
    let opts = LaunchOptions::default_builder()
        .headless(false) // need login cookies from user's browser session
        .build()
        .map_err(|e| format!("launch: {e}"))?;

    let browser = Browser::new(opts).map_err(|e| format!("browser: {e}"))?;
    let tab = browser.new_tab().map_err(|e| format!("tab: {e}"))?;

    // Step 1: Navigate to draft editor
    let url = format!(
        "https://mp.weixin.qq.com/cgi-bin/appmsg?t=media/appmsg_edit_v2&action=edit&isNew=1&type=77&lang=zh_CN&vid={media_id}"
    );
    tab.navigate_to(&url)
        .map_err(|e| format!("navigate: {e}"))?;
    tab.wait_until_navigated()
        .map_err(|e| format!("wait: {e}"))?;
    std::thread::sleep(Duration::from_secs(5));

    // Check if login needed
    let current_url = tab.get_url();
    if !current_url.contains("appmsg_edit") {
        return Err("Not logged into WeChat. Run `moonpub login` first.".to_owned());
    }

    // Step 2: Original declaration — find "未声明" text, click parent element
    tab.evaluate(
        "var a=document.querySelectorAll('*');for(var i=0;i<a.length;i++){if(a[i].textContent.trim()==='未声明'){a[i].parentElement.click();break;}}",
        false,
    ).ok();
    std::thread::sleep(Duration::from_secs(2));

    // Confirm original dialog: check "已阅读" checkbox, click "确定"
    tab.evaluate(
        "var a=document.querySelectorAll('*');for(var i=0;i<a.length;i++){if(a[i].textContent.includes('已阅读'))a[i].click()}var b=document.querySelectorAll('button');for(var j=0;j<b.length;j++){if(b[j].textContent.trim()==='确定'){b[j].click()}}",
        false,
    ).ok();
    std::thread::sleep(Duration::from_secs(2));

    // Step 3: Creation source — click source area, select "个人观点", confirm
    tab.evaluate(
        "document.querySelector('#js_claim_source_area')?.click()",
        false,
    )
    .ok();
    std::thread::sleep(Duration::from_secs(2));
    tab.evaluate(
        "var a=document.querySelectorAll('*');for(var i=0;i<a.length;i++){if(a[i].textContent.trim()==='个人观点，仅供参考'){a[i].click();break;}}var b=document.querySelectorAll('button');for(var j=0;j<b.length;j++){if(b[j].textContent.trim()==='确认'){b[j].click()}}",
        false,
    ).ok();
    std::thread::sleep(Duration::from_secs(2));

    // Step 4: Save draft
    tab.evaluate(
        "var b=document.querySelectorAll('button');for(var i=0;i<b.length;i++){if(b[i].textContent.trim()==='保存为草稿'){b[i].click()}}",
        false,
    ).ok();
    std::thread::sleep(Duration::from_secs(3));

    Ok("backend configured: 原创 + 来源 + 保存".to_owned())
}
