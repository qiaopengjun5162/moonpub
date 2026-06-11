//! Browser automation via chromiumoxide (modern CDP, Chrome 149+).

use chromiumoxide::Page;
use chromiumoxide::browser::{Browser, BrowserConfig};
use futures::StreamExt;
use std::time::Duration;

pub fn login() -> Result<String, String> {
    run_async(async {
        let (_, page) = open_browser().await?;
        page.goto("https://mp.weixin.qq.com")
            .await
            .map_err(|e| format!("{e}"))?;
        println!("Scan QR. 120s...");
        tokio::time::sleep(Duration::from_secs(120)).await;
        Ok("done".to_owned())
    })
}

pub fn auto_configure(_mid: &str) -> Result<String, String> {
    run_async(async {
        let (browser, page) = open_browser().await?;

        // Login
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
        tokio::time::sleep(Duration::from_secs(5)).await;

        // Click edit — find_elements for 2nd icon button in first card
        println!("▶ Click edit...");
        let mut entered = false;
        for _ in 0..20 {
            if let Ok(btns) = page
                .find_elements(".weui-desktop-card__action a.weui-desktop-icon-btn")
                .await
            {
                if btns.len() >= 2 {
                    btns[1].click().await.ok();
                }
                tokio::time::sleep(Duration::from_secs(4)).await;
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
            tokio::time::sleep(Duration::from_millis(500)).await;
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
            if let Ok(el) = page.find_element("//*[text()='未声明']").await {
                el.click().await.ok();
                break;
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        tokio::time::sleep(Duration::from_secs(2)).await;
        xclick(&page, "//*[contains(text(),'已阅读')]").await;
        xclick(&page, "//button[text()='确定']").await;
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
        xclick(&page, "#editor_showmore").await;
        tokio::time::sleep(Duration::from_secs(1)).await;
        xclick(&page, "#js_editor_insertProfile").await;
        tokio::time::sleep(Duration::from_secs(2)).await;
        xclick(&page, "//button[text()='确定']").await;
        println!("  ✅");

        // Save
        println!("▶ Save...");
        xclick(&page, "//button[text()='保存为草稿']").await;
        tokio::time::sleep(Duration::from_secs(3)).await;
        println!("  ✅");

        // Preview
        println!("▶ Preview...");
        xclick(&page, "//button[text()='预览']").await;
        tokio::time::sleep(Duration::from_secs(2)).await;
        xclick(&page, "//*[contains(text(),'公众号列表预览')]").await;
        xclick(&page, "//button[text()='确定']").await;
        println!("  ✅");

        println!("Done! Browser open. Enter to close...");
        let mut buf = String::new();
        std::io::stdin().read_line(&mut buf).ok();
        std::mem::forget(browser);
        Ok("done".to_owned())
    })
}

fn run_async<F>(f: F) -> Result<String, String>
where
    F: std::future::Future<Output = Result<String, String>>,
{
    let rt = tokio::runtime::Runtime::new().map_err(|e| format!("{e}"))?;
    rt.block_on(f)
}

async fn xclick(page: &Page, selector: &str) {
    if let Ok(el) = page.find_element(selector).await {
        el.click().await.ok();
    }
}

async fn open_browser() -> Result<(Browser, Page), String> {
    let (browser, mut handler) = Browser::launch(
        BrowserConfig::builder()
            .with_head()
            .no_sandbox()
            .user_data_dir("/tmp/moonpub-chrome-profile")
            .build()
            .map_err(|e| format!("config: {e}"))?,
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
