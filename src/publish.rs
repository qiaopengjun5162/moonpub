//! Pure Rust CDP automation via chromiumoxide — dedicated profile, no re-login.

use chromiumoxide::Page;
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use std::path::PathBuf;
use std::time::Duration;

pub fn login() -> Result<String, String> {
    run(async {
        let (_, page) = open_browser().await?;
        page.goto("https://mp.weixin.qq.com")
            .await
            .map_err(|e| format!("{e}"))?;
        println!("Scan QR once. This session is saved forever.");
        tokio::time::sleep(Duration::from_secs(120)).await;
        Ok("done".to_owned())
    })
}

pub fn auto_configure(_mid: &str) -> Result<String, String> {
    run(async {
        let (browser, page) = open_browser().await?;

        // Login — dedicated profile remembers cookies
        println!("▶ Login...");
        page.goto("https://mp.weixin.qq.com")
            .await
            .map_err(|e| format!("nav: {e}"))?;
        let mut url = String::new();
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            if let Some(u) = page.url().await.unwrap_or(None) {
                if u.contains("cgi-bin/home") {
                    url = u;
                    break;
                }
            }
        }
        println!("  ✅ Logged in");
        let token = url
            .split("token=")
            .nth(1)
            .and_then(|s| s.split('&').next())
            .unwrap_or("");

        // Drafts list
        let list_url = format!(
            "https://mp.weixin.qq.com/cgi-bin/appmsg?begin=0&count=10&type=77&action=list_card&token={token}&lang=zh_CN"
        );
        page.goto(&list_url)
            .await
            .map_err(|e| format!("list: {e}"))?;
        // Wait for cards to render
        loop {
            tokio::time::sleep(Duration::from_millis(500)).await;
            if page
                .find_element(".weui-desktop-card__action")
                .await
                .is_ok()
            {
                break;
            }
        }

        // Click edit — explicit retry with URL verification
        println!("▶ Click edit...");
        let mut entered = false;
        for _ in 0..30 {
            if let Ok(btns) = page
                .find_elements(".weui-desktop-card__action a.weui-desktop-icon-btn")
                .await
            {
                if btns.len() >= 2 {
                    btns[1].click().await.ok();
                }
            }
            tokio::time::sleep(Duration::from_millis(800)).await;
            if page
                .url()
                .await
                .unwrap_or(None)
                .unwrap_or_default()
                .contains("appmsg_edit")
            {
                entered = true;
                break;
            }
        }
        if !entered {
            println!("  ⚠ Click edit manually, then Enter...");
            let mut buf = String::new();
            std::io::stdin().read_line(&mut buf).ok();
        }
        println!("  ✅ Editor");
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Original
        println!("▶ Original...");
        for _ in 0..40 {
            if let Ok(el) = page.find_element("//span[text()='未声明']/..").await {
                el.click().await.ok();
                break;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
        xclick(&page, "//span[contains(text(),'已阅读')]").await;
        xclick(&page, "//button[text()='确定']").await;
        println!("  ✅");

        // Reward
        println!("▶ Reward...");
        xclick(&page, "//*[text()='赞赏']").await;
        tokio::time::sleep(Duration::from_millis(800)).await;
        xclick(&page, "//*[text()='开启赞赏']").await;
        println!("  ✅");

        // Source
        println!("▶ Source...");
        xclick(&page, "//*[contains(text(),'创作来源')]").await;
        tokio::time::sleep(Duration::from_secs(2)).await;
        xclick(&page, "//*[contains(text(),'个人观点，仅供参考')]").await;
        xclick(&page, "//button[text()='确认']").await;
        println!("  ✅");

        // Account card
        println!("▶ Account card...");
        if let Ok(el) = page.find_element("[contenteditable='true']").await {
            el.focus().await.ok();
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
        xclick(&page, ".js_editor_insert_more, i[class*='more']").await;
        tokio::time::sleep(Duration::from_millis(800)).await;
        xclick(&page, "//*[text()='账号名片']").await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        xclick(&page, "//button[text()='确定']").await;
        println!("  ✅");

        // Save
        println!("▶ Save...");
        xclick(&page, "//button[text()='保存为草稿']").await;
        println!("  ✅");

        // Preview
        println!("▶ Preview...");
        xclick(&page, "//button[text()='预览']").await;
        tokio::time::sleep(Duration::from_secs(2)).await;
        xclick(&page, "//*[contains(text(),'公众号列表预览')]").await;
        xclick(&page, "//button[text()='确定']").await;
        println!("  ✅");

        println!("Done! Enter to close...");
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).ok();
        std::mem::forget(browser);
        Ok("done".to_owned())
    })
}

fn run<F>(f: F) -> Result<String, String>
where
    F: std::future::Future<Output = Result<String, String>>,
{
    tokio::runtime::Runtime::new()
        .map_err(|e| format!("{e}"))?
        .block_on(f)
}

async fn xclick(page: &Page, s: &str) {
    if let Ok(el) = page.find_element(s).await {
        el.click().await.ok();
    }
}

fn profile_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    let mut p = PathBuf::from(format!(
        "{home}/Library/Mobile Documents/com~apple~CloudDocs/ObsidianMain/.moonpub"
    ));
    p.push("dedicated-chrome-profile");
    std::fs::create_dir_all(&p).ok();
    p
}

async fn open_browser() -> Result<(Browser, Page), String> {
    let (browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .with_head()
            .no_sandbox()
            .user_data_dir(profile_dir())
            .window_size(1280, 1024)
            .build()
            .map_err(|e| format!("{e}"))?,
    )
    .await
    .map_err(|e| format!("launch: {e}"))?;
    tokio::task::spawn(async move {
        while let Some(h) = handler.next().await {
            if h.is_err() {
                break;
            }
        }
    });
    let pages = browser.pages().await.map_err(|e| format!("{e}"))?;
    let page = if !pages.is_empty() {
        pages.into_iter().next().unwrap()
    } else {
        browser
            .new_page("about:blank")
            .await
            .map_err(|e| format!("{e}"))?
    };
    Ok((browser, page))
}
