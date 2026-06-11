//! WeChat backend automation via headless_chrome (CDP).

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

/// Full automation after API push.
/// Steps: login → home → recent draft → original → reward → source →
///        collection → AI cover → save → preview → Enter to close.
pub fn auto_configure(_media_id: &str) -> Result<String, String> {
    let browser = launch_headed()?;
    let tab = browser.new_tab().map_err(|e| format!("tab: {e}"))?;

    // ── Step 1: Root → Cookie-based 302 redirect to home ──
    println!("▶ 检查登录状态...");
    tab.navigate_to("https://mp.weixin.qq.com")
        .map_err(|e| format!("navigate: {e}"))?;
    std::thread::sleep(Duration::from_secs(3));

    let mut current_url = tab.get_url();

    // ── Step 2: If not redirected to home, Cookie expired → scan QR ──
    if !current_url.contains("cgi-bin/home") {
        println!("⚠️ 凭证过期，请扫码登录...");
        wait_for_login(&tab, 120)?;
        current_url = tab.get_url();
    }
    println!("  ✅ 已登录");

    // ── Step 3: Wait for "近期草稿" cards on home page ──
    println!("  ▶ 等待首页「近期草稿」加载...");
    // Debug: print page text to find correct selectors
    if let Ok(r) = tab.evaluate("return document.body.innerText.substring(0,300)", false) {
        if let Some(v) = r.value.and_then(|v| v.as_str().map(String::from)) {
            println!("  Page: {v}");
        }
    }
    // Try finding any clickable draft link by text content ("更新于" marker)
    let loaded = (0..20).any(|_| {
        if let Ok(res) = tab.evaluate(
            "var a=document.querySelectorAll('a');for(var i=0;i<a.length;i++){if(a[i].offsetHeight>0&&a[i].textContent.includes('更新于')){return true;}}return false;",
            false,
        ) {
            if res.value.and_then(|v| v.as_bool()).unwrap_or(false) {
                return true;
            }
        }
        std::thread::sleep(Duration::from_millis(500));
        false
    });
    if !loaded {
        return Err("首页近期草稿加载超时".to_string());
    }

    // ── Step 4: Click first draft card (3 strategies) ──
    println!("  ▶ 点击第一篇草稿...");
    wait_and_execute(
        &tab,
        "var cards=document.querySelectorAll('.appmsg_item,[class*=\"draft_item\"],.recent_draft_item');\
         if(!cards.length){var d=document.querySelectorAll('div');for(var i=0;i<d.length;i++){if(d[i].offsetHeight>0&&d[i].textContent.includes('更新于')){var c=d[i].closest('div');if(c){cards=[c];break;}}}}\
         if(!cards.length)return false;\
         var card=cards[0];\
         /* Strategy A: full-depth hover to reveal hidden buttons */\
         ['mouseover','mouseenter','mousemove'].forEach(function(e){card.dispatchEvent(new MouseEvent(e,{bubbles:true,cancelable:true,view:window}));var ch=card.querySelectorAll('*');for(var k=0;k<ch.length;k++)ch[k].dispatchEvent(new MouseEvent(e,{bubbles:true}));});\
         var btns=card.querySelectorAll('a,button,[class*=\"btn\"],[class*=\"item\"]');\
         var found=[];\
         for(var j=0;j<btns.length;j++){if(btns[j].title==='编辑'||btns[j].textContent.includes('编辑')||btns[j].querySelector('.weui-desktop-icon-edit')||btns[j].getAttribute('href')==='javascript:;')found.push(btns[j]);}\
         if(found.length>=2&&found[1].offsetHeight>0){found[1].click();return true;}\
         /* Strategy B: extract real href and redirect */\
         var raw=card.getAttribute('href')||card.querySelector('a')?.getAttribute('href');\
         if(raw&&raw!=='javascript:;'){window.location.href=raw;return true;}\
         /* Strategy C: blind click on title/thumb area */\
         var ct=card.querySelector('.appmsg_title a,.appmsg_thumb,[class*=\"title\"]');\
         if(ct){ct.click();return true;}\
         card.click();return true;",
        20,
    )?;

    // ── Step 5: Wait for editor to fully load ──
    tab.wait_for_element("div#edui1_iframeholder")
        .map_err(|_| "编辑器加载超时".to_string())?;
    println!("  ✅ 正文编辑器已加载");
    std::thread::sleep(Duration::from_secs(3));

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
    wait_and_execute(&tab, "var items=document.querySelectorAll('[class*=dropdown] li,[class*=dropdown] span,[class*=menu] li,[class*=menu] span,[class*=list] li');for(var i=0;i<items.length;i++){if(items[i].offsetHeight>0&&items[i].textContent.trim().length>0){items[i].click();return true;}}return false;", 10).ok();
    println!("  ✅ 合集");

    // ── AI Cover ──
    println!("▶ AI 封面...");
    wait_and_execute(&tab, "document.querySelector('.js_cover_btn_area')?.dispatchEvent(new MouseEvent('mouseover',{bubbles:true}));return true;", 5).ok();
    std::thread::sleep(Duration::from_millis(500));
    wait_and_execute(&tab, "var a=document.querySelectorAll('*');for(var i=0;i<a.length;i++){if(a[i].textContent.includes('AI')&&a[i].textContent.includes('配图')){a[i].click();return true;}}return false;", 10).ok();
    std::thread::sleep(Duration::from_secs(3));
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

    println!("🎉 全部完成！请确认后点「发表」。");
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
        if tab.get_url().contains("cgi-bin/home") {
            return Ok(());
        }
    }
    Err("Login timeout".to_owned())
}

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
