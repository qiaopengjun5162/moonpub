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
pub fn auto_configure(_media_id: &str) -> Result<String, String> {
    let browser = launch_headed()?;
    let tab = browser.new_tab().map_err(|e| format!("tab: {e}"))?;

    // ── Step 1: Navigate to ROOT (not home!) — triggers Cookie-based 302 redirect ──
    println!("▶ 检查微信登录状态...");
    tab.navigate_to("https://mp.weixin.qq.com")
        .map_err(|e| format!("navigate: {e}"))?;
    std::thread::sleep(Duration::from_secs(4));

    let mut current_url = tab.get_url();

    // ── Step 2: If no redirect to home, Cookie expired → scan QR ──
    if !current_url.contains("cgi-bin/home") {
        println!("⚠️ 凭证过期，请扫码登录...");
        wait_for_login(&tab, 120)?;
        current_url = tab.get_url();
    }
    println!("  ✅ 已登录");

    // ── Step 3: Extract dynamic web token from the home URL ──
    let web_token = current_url
        .split("token=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .ok_or("无法从主页 URL 提取动态 token".to_string())?;
    println!("  Token: {web_token}");

    // ── Step 4: Go to drafts list page (not direct editor URL) ──
    // WeChat's web backend uses a different ID system from the API.
    // vid ≠ appmsgid, so direct editor URL navigation shows empty draft.
    // Instead: simulate human — open drafts list, hover first card, click "编辑".
    let list_url = format!(
        "https://mp.weixin.qq.com/cgi-bin/appmsg?begin=0&count=10&type=77&action=list_card&token={web_token}&lang=zh_CN"
    );
    tab.navigate_to(&list_url)
        .map_err(|e| format!("list nav: {e}"))?;
    std::thread::sleep(Duration::from_secs(5));

    // WeChat draft cards: 3 hidden action buttons appear on hover.
    // The 2nd button (index 1) in .appmsg_mask is the "编辑" (edit) button.
    std::thread::sleep(Duration::from_secs(4));
    wait_and_execute(
        &tab,
        "var cards=document.querySelectorAll('.appmsg_card_wrp,.appmsg_card');\
         if(!cards.length)return false;\
         cards[0].dispatchEvent(new MouseEvent('mouseover',{bubbles:true}));\
         var btns=cards[0].querySelectorAll('.appmsg_mask a');\
         if(btns.length>=2){btns[1].click();return true;}\
         var links=cards[0].querySelectorAll('a');\
         for(var i=0;i<links.length;i++){\
           if(links[i].querySelector('i')&&links[i].offsetHeight>0){links[i].click();return true;}\
         }\
         return false;",
        20,
    )?;
    println!("  已进入草稿编辑...");
    std::thread::sleep(Duration::from_secs(5));

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
        if url.contains("cgi-bin/home") {
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
