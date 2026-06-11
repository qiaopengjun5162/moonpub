//! WeChat backend automation — headless_chrome, headed mode, no profile conflict.

use headless_chrome::{Browser, LaunchOptions};
use std::sync::Arc;
use std::time::Duration;

type HlcTab = Arc<headless_chrome::Tab>;

pub fn login() -> Result<String, String> {
    Ok("scan QR in browser".to_owned())
}

pub fn auto_configure(_mid: &str) -> Result<String, String> {
    let tab = Browser::new(
        LaunchOptions::default_builder()
            .headless(false)
            .build()
            .map_err(|e| format!("{e}"))?,
    )
    .map_err(|e| format!("{e}"))?
    .new_tab()
    .map_err(|e| format!("{e}"))?;

    // Login
    println!("▶ Login...");
    tab.navigate_to("https://mp.weixin.qq.com")
        .map_err(|e| format!("nav: {e}"))?;
    std::thread::sleep(Duration::from_secs(5));
    if !tab.get_url().contains("cgi-bin/home") {
        println!("  Scan QR. 120s...");
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

    // Click 2nd icon button in first card (=edit)
    println!("▶ Click edit...");
    let mut entered = false;
    for _ in 0..20 {
        // Use evaluate with the exact DOM selector from real page
        if let Ok(r) = tab.evaluate(
            "var a=document.querySelectorAll('.weui-desktop-card__action a.weui-desktop-icon-btn');if(a.length>=2){a[1].click();return true;}return false;", false
        ) {
            if r.value.and_then(|v| v.as_bool()).unwrap_or(false) {
                std::thread::sleep(Duration::from_secs(4));
                if tab.get_url().contains("appmsg_edit") { entered = true; break; }
            }
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    if !entered {
        println!("  ⚠ Click edit manually, then Enter...");
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).ok();
    }
    println!("  ✅ Editor");
    std::thread::sleep(Duration::from_secs(5));

    // Helper: eval in iframe context
    let ei = |tab: &HlcTab, js: &str| -> bool {
        if let Ok(r) = tab.evaluate(
            &format!("var d=document;var f=d.querySelector('iframe[src*=\"appmsg_edit\"]');if(f&&f.contentDocument)d=f.contentDocument;{js}"), false
        ) { r.value.and_then(|v| v.as_bool()).unwrap_or(false) } else { false }
    };

    // Original
    println!("▶ Original...");
    for _ in 0..40 {
        if ei(
            &tab,
            "var a=d.querySelectorAll('*');for(var i=0;i<a.length;i++){if(a[i].textContent.trim()==='未声明'){a[i].parentElement.click();return true;}}return false;",
        ) {
            break;
        }
        std::thread::sleep(Duration::from_millis(500));
    }
    std::thread::sleep(Duration::from_secs(2));
    ei(
        &tab,
        "var a=d.querySelectorAll('*');for(var i=0;i<a.length;i++){if(a[i].textContent.includes('已阅读'))a[i].click();}var b=d.querySelectorAll('button');for(var j=0;j<b.length;j++){if(b[j].textContent.trim()==='确定'){b[j].click();return true;}}return false;",
    );
    println!("  ✅");

    // Source
    println!("▶ Source...");
    ei(
        &tab,
        "var el=d.querySelector('#js_claim_source_area');if(el)el.click();return!!el;",
    );
    std::thread::sleep(Duration::from_secs(2));
    ei(
        &tab,
        "var a=d.querySelectorAll('*');for(var i=0;i<a.length;i++){if(a[i].textContent.trim()==='个人观点，仅供参考'){a[i].click();break;}}var b=d.querySelectorAll('button');for(var j=0;j<b.length;j++){if(b[j].textContent.trim()==='确认'){b[j].click();return true;}}return false;",
    );
    println!("  ✅");

    // Account card
    println!("▶ Account card...");
    ei(
        &tab,
        "var ed=d.querySelector('[contenteditable=\"true\"]');if(ed){ed.focus();var r=document.createRange();r.selectNodeContents(ed);r.collapse(false);window.getSelection().removeAllRanges();window.getSelection().addRange(r);}return true;",
    );
    std::thread::sleep(Duration::from_millis(500));
    ei(
        &tab,
        "var el=d.querySelector('#editor_showmore');if(el){el.click();return true;}return false;",
    );
    std::thread::sleep(Duration::from_secs(1));
    ei(
        &tab,
        "var el=d.querySelector('#js_editor_insertProfile');if(el){el.click();return true;}return false;",
    );
    std::thread::sleep(Duration::from_secs(2));
    ei(
        &tab,
        "var b=d.querySelectorAll('button');for(var i=0;i<b.length;i++){if(b[i].textContent.trim()==='确定'){b[i].click();return true;}}return false;",
    );
    println!("  ✅");

    // Save
    println!("▶ Save...");
    ei(
        &tab,
        "var b=d.querySelectorAll('button');for(var i=0;i<b.length;i++){if(b[i].textContent.trim()==='保存为草稿'){b[i].click();return true;}}return false;",
    );
    std::thread::sleep(Duration::from_secs(3));
    println!("  ✅");

    // Preview
    println!("▶ Preview...");
    ei(
        &tab,
        "var b=d.querySelectorAll('button');for(var i=0;i<b.length;i++){if(b[i].textContent.trim()==='预览'){b[i].click();return true;}}return false;",
    );
    std::thread::sleep(Duration::from_secs(2));
    ei(
        &tab,
        "var a=d.querySelectorAll('label');for(var i=0;i<a.length;i++){if(a[i].textContent.includes('公众号列表预览'))a[i].click();}var b=d.querySelectorAll('button');for(var j=0;j<b.length;j++){if(b[j].textContent.trim()==='确定'){b[j].click();return true;}}return false;",
    );
    println!("  ✅");

    println!("Done! Enter to close...");
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).ok();
    Ok("done".to_owned())
}
