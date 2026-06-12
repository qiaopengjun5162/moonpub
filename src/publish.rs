//! Pure Rust CDP automation via chromiumoxide — dedicated profile, no re-login.
//!
//! Key design: all clicks use JS evaluate so XPath selectors work correctly.
//! chromiumoxide's find_element() only accepts CSS — XPath must go through JS.

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
            .map_err(|e| e.to_string())?;
        println!("Scan QR once. This session is saved forever.");
        tokio::time::sleep(Duration::from_secs(120)).await;
        Ok("done".to_owned())
    })
}

pub fn auto_configure(_mid: &str) -> Result<String, String> {
    run(async {
        let (browser, page) = open_browser().await?;

        // ── Login ─────────────────────────────────────────────────────────────
        println!("▶ Login...");
        page.goto("https://mp.weixin.qq.com")
            .await
            .map_err(|e| format!("nav: {e}"))?;
        let url = wait_url(&page, "cgi-bin/home").await;
        println!("  ✅ {url}");
        let token = url
            .split("token=")
            .nth(1)
            .and_then(|s| s.split('&').next())
            .unwrap_or("");

        // ── Draft list ────────────────────────────────────────────────────────
        println!("▶ Draft list...");
        let list_url = format!(
            "https://mp.weixin.qq.com/cgi-bin/appmsg?begin=0&count=10&type=77&action=list_card&token={token}&lang=zh_CN"
        );
        page.goto(&list_url)
            .await
            .map_err(|e| format!("list: {e}"))?;
        if !wait_css(&page, ".weui-desktop-card__action", 15_000).await {
            return Err("draft list did not render".into());
        }

        // ── Enter editor ──────────────────────────────────────────────────────
        println!("▶ Entering editor...");
        // Click edit once; WeChat opens the editor in a new tab
        if let Ok(btns) = page
            .find_elements(".weui-desktop-card__action a.weui-desktop-icon-btn")
            .await
            && btns.len() >= 2
        {
            btns[1].click().await.ok();
            println!("    clicked edit btn (expecting new tab)");
        }

        // Wait up to 20s for the editor tab to appear
        let mut editor_opt: Option<Page> = None;
        for _ in 0..25 {
            sleep_ms(800).await;
            if let Ok(all) = browser.pages().await {
                for p in all {
                    let u = p.url().await.unwrap_or(None).unwrap_or_default();
                    if u.contains("appmsg_edit") {
                        println!("    found editor tab: {u}");
                        editor_opt = Some(p);
                        break;
                    }
                }
            }
            if editor_opt.is_some() {
                break;
            }
            // Fallback: same-tab navigation
            let cur = page.url().await.unwrap_or(None).unwrap_or_default();
            if cur.contains("appmsg_edit") {
                println!("    editor on same tab");
                break;
            }
        }

        let page = if let Some(ep) = editor_opt {
            ep
        } else {
            let cur = page.url().await.unwrap_or(None).unwrap_or_default();
            if !cur.contains("appmsg_edit") {
                println!("  ⚠ Editor not detected — navigate manually, then Enter...");
                readline();
            }
            page
        };
        println!("  ✅ In editor");
        sleep_ms(3_000).await;

        // ── 账号名片 ──────────────────────────────────────────────────────────
        println!("▶ 账号名片...");
        // Diagnose iframe structure on first attempt
        if let Ok(v) = page.evaluate(r#"
            (() => {
                var result = [];
                var frames = document.querySelectorAll('iframe');
                for (var i = 0; i < frames.length; i++) {
                    var f = frames[i];
                    var has_toolbar = false;
                    try {
                        var doc = f.contentDocument;
                        if (doc) has_toolbar = !!doc.querySelector('.js_editor_insert_more, [class*="insert_more"]');
                    } catch(e) {}
                    result.push('iframe[' + i + '] name=' + (f.name||'-') + ' id=' + (f.id||'-') + ' toolbar=' + has_toolbar);
                }
                return result.join('\n');
            })()
        "#).await
            && let Some(s) = v.value().and_then(|v| v.as_str().map(|s| s.to_owned()))
        {
            println!("    iframe map:\n{s}");
        }
        let ok = retry_click_editor(
            &page,
            &[
                ".js_editor_insert_more",
                "#js_insert_more",
                "//*[contains(@class,'js_editor_insert_more')]",
                "//*[contains(@class,'insert_more')]",
            ],
            8,
            400,
        )
        .await;
        println!("    click toolbar '...': {ok}");
        if ok {
            sleep_ms(800).await;
            let ok2 = retry_click(
                &page,
                &["//*[text()='账号名片']", "//*[contains(text(),'账号名片')]"],
                6,
                300,
            )
            .await;
            println!("    click '账号名片' menu: {ok2}");
            if ok2 {
                sleep_ms(1_500).await;
                let ok3 = retry_click(
                    &page,
                    &[
                        "//*[normalize-space(text())='寻月隐君']",
                        "//*[contains(text(),'寻月隐君')]",
                    ],
                    10,
                    300,
                )
                .await;
                println!("    click '寻月隐君': {ok3}");
                sleep_ms(500).await;
                let ok4 = retry_click(
                    &page,
                    &[
                        "//button[normalize-space(text())='插入']",
                        "//button[contains(text(),'插入')]",
                        "//button[normalize-space(text())='确定']",
                    ],
                    6,
                    300,
                )
                .await;
                println!("    click 插入: {ok4}");
                sleep_ms(500).await;
            }
            println!("  ✅");
        } else {
            println!("  ⚠ toolbar '...' not found — skipping");
        }

        // ── 原创声明 ──────────────────────────────────────────────────────────
        println!("▶ 原创声明...");
        let ok = retry_click(
            &page,
            &[
                "//span[text()='未声明']/..",
                "//*[contains(text(),'未声明') and not(self::script)]",
            ],
            15,
            400,
        )
        .await;
        println!("    click '未声明': {ok}");
        if ok {
            sleep_ms(1_200).await;
            // WeChat uses a custom styled checkbox — must click the wrapping label
            let ok2 = retry_click(
                &page,
                &[
                    "//*[contains(text(),'我已阅读并同意')]/ancestor::label",
                    "//*[contains(text(),'我已阅读并同意')]/preceding-sibling::span",
                    "//*[contains(text(),'我已阅读并同意')]",
                    "//span[contains(@class,'checkbox')]",
                    "//label[contains(.,'已阅读')]",
                ],
                12,
                400,
            )
            .await;
            println!("    check '已阅读': {ok2}");
            sleep_ms(500).await;
            let ok3 = retry_click(
                &page,
                &[
                    "//div[contains(@class,'popover') or contains(@class,'dialog')]//button[contains(.,'确定')]",
                    "//div[contains(@class,'btn_wrp')]//button[text()='确定']",
                    "//button[contains(@class,'primary') and text()='确定']",
                    "//button[normalize-space(text())='确定']",
                ],
                10,
                400,
            )
            .await;
            println!("    click '确定': {ok3}");
            // Wait for dialog to close (modal gone = 声明类型 text disappears)
            for i in 1u32..=20 {
                let still_open = page
                    .evaluate(
                        r#"(() => {
                    var fr = document.querySelector('iframe[name="main"]');
                    var doc = fr ? fr.contentDocument : document;
                    return doc ? doc.body.innerText.includes('声明类型') : false;
                })()"#,
                    )
                    .await
                    .ok()
                    .and_then(|v| v.value().and_then(|v| v.as_bool()))
                    .unwrap_or(false);
                if !still_open {
                    println!("    dialog closed (step {i})");
                    break;
                }
                if i == 20 {
                    println!("    ⚠ dialog still open — checkbox may not have been checked");
                }
                sleep_ms(400).await;
            }
            println!("  ✅");
        } else {
            println!("  ⚠ '未声明' not found — skipping");
        }

        // ── 赞赏 ──────────────────────────────────────────────────────────────
        println!("▶ 赞赏...");
        let ok = retry_click(
            &page,
            &[
                "//*[text()='赞赏']",
                "//*[contains(text(),'赞赏功能')]",
                "//span[contains(.,'赞赏')]",
            ],
            8,
            400,
        )
        .await;
        println!("    click '赞赏': {ok}");
        if ok {
            sleep_ms(800).await;
            let ok2 = retry_click(
                &page,
                &["//*[text()='开启赞赏']", "//*[contains(text(),'开启赞赏')]"],
                8,
                300,
            )
            .await;
            println!("    click '开启赞赏': {ok2}");
            sleep_ms(500).await;
            println!("  ✅");
        } else {
            println!("  ⚠ '赞赏' not found — skipping");
        }

        // ── 合集 ──────────────────────────────────────────────────────────────
        println!("▶ 合集...");
        let ok = retry_click(&page, &["//*[text()='合集']"], 8, 400).await;
        println!("    click '合集': {ok}");
        if ok {
            sleep_ms(1_000).await;
            let ok2 = retry_click(
                &page,
                &[
                    "//li[contains(@class,'collection')]",
                    "//*[contains(@class,'collect_item')]",
                ],
                6,
                300,
            )
            .await;
            println!("    select collection: {ok2}");
            sleep_ms(400).await;
            if ok2 {
                let ok3 = xclick(&page, "//button[normalize-space(text())='确定']").await;
                println!("    click '确定': {ok3}");
                sleep_ms(800).await;
            }
            println!("  ✅");
        } else {
            println!("  ⚠ '合集' not found — skipping");
        }

        // ── 留言 ──────────────────────────────────────────────────────────────
        println!("▶ 留言...");
        let ok = retry_click(&page, &["//*[text()='留言']"], 8, 400).await;
        println!("    click '留言': {ok}");
        if ok {
            sleep_ms(1_000).await;
            sleep_ms(600).await;
            let ok2 = retry_click(
                &page,
                &[
                    "//div[contains(@class,'desktop-dialog')]//button[contains(.,'确定')]",
                    "//div[contains(@class,'dialog__ft')]//button",
                    "//button[contains(@class,'primary') and contains(.,'确定')]",
                    "//button[normalize-space(text())='确定']",
                ],
                10,
                400,
            )
            .await;
            println!("    click '确定': {ok2}");
            sleep_ms(500).await;
            println!("  ✅");
        } else {
            println!("  ⚠ '留言' not found — skipping");
        }

        // ── 创作来源 ──────────────────────────────────────────────────────────
        println!("▶ 创作来源...");
        let ok = retry_click(
            &page,
            &["//*[text()='创作来源']", "//*[contains(text(),'创作来源')]"],
            8,
            400,
        )
        .await;
        println!("    click '创作来源': {ok}");
        if ok {
            sleep_ms(2_000).await;
            let ok2 = retry_click(
                &page,
                &[
                    "//*[contains(text(),'个人观点，仅供参考')]",
                    "//label[contains(.,'个人观点')]",
                ],
                8,
                300,
            )
            .await;
            println!("    select '个人观点': {ok2}");
            sleep_ms(400).await;
            let ok3 = retry_click(
                &page,
                &[
                    "//div[contains(@class,'desktop-dialog')]//button[contains(.,'确认')]",
                    "//div[contains(@class,'dialog__ft')]//button[contains(.,'确认')]",
                    "//button[contains(@class,'primary') and contains(.,'确认')]",
                    "//button[normalize-space(text())='确认']",
                ],
                10,
                400,
            )
            .await;
            println!("    click '确认': {ok3}");
            sleep_ms(1_000).await;
            println!("  ✅");
        } else {
            println!("  ⚠ '创作来源' not found — skipping");
        }

        // ── 保存为草稿 ────────────────────────────────────────────────────────
        println!("▶ Save draft...");
        let ok = retry_click(
            &page,
            &[
                "//button[normalize-space(text())='保存为草稿']",
                "//button[contains(text(),'保存')]",
            ],
            5,
            500,
        )
        .await;
        println!("    save: {ok}");
        sleep_ms(1_500).await;
        println!("  ✅");

        // ── 预览 ──────────────────────────────────────────────────────────────
        println!("▶ Preview...");
        let ok = retry_click(&page, &["//button[normalize-space(text())='预览']"], 5, 500).await;
        println!("    preview btn: {ok}");
        sleep_ms(2_000).await;
        xclick(&page, "//*[contains(text(),'公众号列表预览')]").await;
        sleep_ms(500).await;
        xclick(&page, "//button[normalize-space(text())='确定']").await;
        println!("  ✅");

        println!("Done! Press Enter to close...");
        readline();
        std::mem::forget(browser);
        Ok("done".to_owned())
    })
}

fn wait_enter() {
    print!("  → 按 Enter 继续...");
    std::io::Write::flush(&mut std::io::stdout()).ok();
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).ok();
}

fn ask_ok(prompt: &str) -> bool {
    print!("  → {prompt} (y/n): ");
    std::io::Write::flush(&mut std::io::stdout()).ok();
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).ok();
    matches!(buf.trim().to_lowercase().as_str(), "y" | "yes" | "")
}

async fn shot(page: &Page, path: &std::path::Path) {
    page.save_screenshot(chromiumoxide::page::ScreenshotParams::default(), path)
        .await
        .ok();
}

pub fn step_test() -> Result<String, String> {
    run(async {
        let (browser, page) = open_browser().await?;
        let dir = std::path::PathBuf::from("/tmp/moonpub-test");
        std::fs::create_dir_all(&dir).ok();
        let mut s = 0u32;

        // ── Step 1: 导航 ──
        s += 1;
        println!("\n══ Step {s}: 导航到 mp.weixin.qq.com ══");
        wait_enter();
        page.goto("https://mp.weixin.qq.com")
            .await
            .map_err(|e| format!("nav: {e}"))?;
        let url = page.url().await.unwrap_or(None).unwrap_or_default();
        println!("  URL: {url}");
        shot(&page, &dir.join(format!("step{s:02}.png"))).await;
        if !ask_ok("页面打开？") {
            return Err("取消".into());
        }

        // ── Step 2: 登录 ──
        s += 1;
        println!("\n══ Step {s}: 登录 ══");
        println!("  等待跳转到 cgi-bin/home...");
        let url = wait_url(&page, "cgi-bin/home").await;
        let token = url
            .split("token=")
            .nth(1)
            .and_then(|t| t.split('&').next())
            .unwrap_or("");
        println!("  token: {token}");
        shot(&page, &dir.join(format!("step{s:02}.png"))).await;
        if !ask_ok("登录成功？") {
            return Err("取消".into());
        }

        // ── Step 3: 草稿列表 ──
        s += 1;
        println!("\n══ Step {s}: 草稿列表 ══");
        wait_enter();
        page.goto(&format!(
            "https://mp.weixin.qq.com/cgi-bin/appmsg?begin=0&count=10&type=77&action=list_card&token={token}&lang=zh_CN"
        )).await.map_err(|e| format!("list: {e}"))?;
        let ok = wait_css(&page, ".weui-desktop-card__action", 15_000).await;
        println!("  列表: {}", if ok { "✅" } else { "❌" });
        shot(&page, &dir.join(format!("step{s:02}.png"))).await;
        if !ask_ok("草稿列表？") {
            return Err("取消".into());
        }

        // ── Step 4: 进入编辑器 ──
        s += 1;
        println!("\n══ Step {s}: 进入编辑器 ══");
        wait_enter();
        if let Ok(btns) = page
            .find_elements(".weui-desktop-card__action a.weui-desktop-icon-btn")
            .await
            && btns.len() >= 2
        {
            btns[1].click().await.ok();
            println!("  点击编辑按钮");
        }
        let mut edit_page: Option<Page> = None;
        for _i in 0..30 {
            sleep_ms(800).await;
            if let Ok(all) = browser.pages().await {
                for p in all {
                    if p.url()
                        .await
                        .unwrap_or(None)
                        .unwrap_or_default()
                        .contains("appmsg_edit")
                    {
                        edit_page = Some(p);
                        break;
                    }
                }
            }
            if edit_page.is_some() {
                break;
            }
        }
        let page = edit_page.unwrap_or(page);
        let cur = page.url().await.unwrap_or(None).unwrap_or_default();
        println!(
            "  编辑器: {}",
            if cur.len() > 80 { &cur[..80] } else { &cur }
        );
        shot(&page, &dir.join(format!("step{s:02}.png"))).await;
        sleep_ms(3_000).await;
        if !ask_ok("编辑器打开？") {
            return Err("取消".into());
        }

        // ── Step 5: 工具栏 "..." ──
        s += 1;
        println!("\n══ Step {s}: 点击工具栏 ... ══");
        wait_enter();
        let ok = retry_click(
            &page,
            &[
                "#editor_showmore",
                "//li[@id='editor_showmore']",
                ".jsInsertIcon",
            ],
            8,
            400,
        )
        .await;
        println!("  ... 按钮: {ok}");
        shot(&page, &dir.join(format!("step{s:02}.png"))).await;
        if !ask_ok("下拉菜单打开了？") {
            return Err("取消".into());
        }

        // ── Step 6a: 选择账号名片 ──
        s += 1;
        println!("\n══ Step {s}a: 选择账号名片 ══");
        wait_enter();
        let ok = retry_click(
            &page,
            &[
                "#js_editor_insertProfile",
                "//li[@id='js_editor_insertProfile']",
            ],
            6,
            300,
        )
        .await;
        println!("  账号名片菜单: {ok}");
        sleep_ms(2_000).await;
        shot(&page, &dir.join(format!("step{s:02}a_dialog.png"))).await;
        if !ask_ok("账号名片对话框打开了？") {
            return Err("取消".into());
        }

        // ── Step 6b: 选公众号 ──
        s += 1;
        println!("\n══ Step {s}b: 选择公众号「寻月隐君」 ══");
        wait_enter();
        // First: diagnose what's on the page
        let diag = page.evaluate(
            r#"(() => {
                var info = [];
                // List all dialogs and their visible state
                var dialogs = document.querySelectorAll('[class*="dialog"], [class*="Dialog"], [class*="modal"], mp-image-product-dialog, [role="dialog"]');
                info.push('dialogs: ' + dialogs.length);
                for (var i = 0; i < dialogs.length; i++) {
                    var d = dialogs[i];
                    info.push('  ['+i+'] ' + (d.tagName||'?') + ' visible=' + (d.offsetParent!==null) + ' class=' + (d.className||d.getAttribute('class')||''));
                }
                // Find all elements with 寻月隐君 text
                var all = document.querySelectorAll('*');
                var matches = [];
                for (var i = 0; i < all.length; i++) {
                    var t = all[i].childNodes;
                    var hasText = false;
                    for (var j = 0; j < t.length; j++) {
                        if (t[j].nodeType === 3 && t[j].textContent.includes('寻月隐君')) {
                            hasText = true; break;
                        }
                    }
                    if (hasText) {
                        matches.push(all[i].tagName + '.' + (all[i].className||'') + ' visible=' + (all[i].offsetParent!==null));
                    }
                }
                info.push('text matches: ' + matches.length);
                for (var i = 0; i < Math.min(matches.length, 10); i++) {
                    info.push('  ' + matches[i]);
                }
                return info.join('\n');
            })()"#,
        ).await.ok().and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned()))).unwrap_or_default();
        println!("  页面诊断:\n{diag}");

        let ok = page.evaluate(
            r#"(() => {
                var clickEl = function(el) {
                    el.scrollIntoView({block:'center'});
                    el.click();
                    var o = {bubbles:true, cancelable:true, view:window};
                    el.dispatchEvent(new MouseEvent('mousedown', o));
                    el.dispatchEvent(new MouseEvent('mouseup', o));
                    el.dispatchEvent(new MouseEvent('click', o));
                };
                var xpath = "//mp-image-product-dialog//div[contains(@class,'weui-desktop-grid__col')][.//*[contains(text(),'寻月隐君')]]";
                var result = document.evaluate(xpath, document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null);
                if (result.singleNodeValue) {
                    clickEl(result.singleNodeValue);
                    return 'xpath: ' + (result.singleNodeValue.className||result.singleNodeValue.tagName);
                }
                var items = document.querySelectorAll('.weui-desktop-grid__col, .appmsg_card_context, .wx_profile_card, .profile_history_item');
                for (var i = 0; i < items.length; i++) {
                    if (items[i].textContent && items[i].textContent.includes('寻月隐君')) {
                        var outer = items[i].closest('.weui-desktop-grid__col') || items[i].closest('.appmsg_card_context') || items[i];
                        clickEl(outer);
                        return 'fallback: ' + (outer.className||outer.tagName);
                    }
                }
                return 'not found';
            })()"#,
        ).await.ok().and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned()))).unwrap_or_default();
        println!("  选择公众号: {ok}");
        sleep_ms(500).await;
        shot(&page, &dir.join(format!("step{s:02}b_select.png"))).await;
        if !ask_ok("公众号选中了？") {
            return Err("取消".into());
        }

        // ── Step 6c: 点击插入 ──
        s += 1;
        println!("\n══ Step {s}c: 点击插入 ══");
        wait_enter();
        let ok = page
            .evaluate(
                r#"(() => {
                var btns = document.querySelectorAll('button');
                for (var i = 0; i < btns.length; i++) {
                    if (btns[i].textContent && btns[i].textContent.trim() === '插入') {
                        btns[i].scrollIntoView({block:'center'});
                        btns[i].click();
                        var o = {bubbles:true, cancelable:true, view:window};
                        btns[i].dispatchEvent(new MouseEvent('mousedown', o));
                        btns[i].dispatchEvent(new MouseEvent('mouseup', o));
                        btns[i].dispatchEvent(new MouseEvent('click', o));
                        return 'clicked';
                    }
                }
                return 'not found';
            })()"#,
            )
            .await
            .ok()
            .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
            .unwrap_or_default();
        println!("  插入: {ok}");
        sleep_ms(1_000).await;
        let cur = page.url().await.unwrap_or(None).unwrap_or_default();
        println!(
            "  当前URL: {}",
            if cur.len() > 80 { &cur[..80] } else { &cur }
        );
        shot(&page, &dir.join(format!("step{s:02}c_insert.png"))).await;
        if !ask_ok("账号名片插入成功？(应该还在编辑器)") {
            return Err("取消".into());
        }

        println!("\n══ 🛑 Step 6 账号名片流程结束 ══\n按 Enter 关闭浏览器...");
        readline();
        std::mem::forget(browser);
        Ok("done".into())
    })
}

// ── runtime ───────────────────────────────────────────────────────────────────

fn run<F>(f: F) -> Result<String, String>
where
    F: std::future::Future<Output = Result<String, String>>,
{
    tokio::runtime::Runtime::new()
        .map_err(|e| e.to_string())?
        .block_on(f)
}

fn readline() {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).ok();
}

// ── click helpers ─────────────────────────────────────────────────────────────

/// Click an element by XPath (starts with '/') or CSS selector.
/// Searches iframe[name="main"] first (WeChat editor settings sandbox),
/// then falls back to the main document.
/// Returns true if the element was found and clicked.
async fn xclick(page: &Page, selector: &str) -> bool {
    let (query_doc, query_frame) = if selector.starts_with('/') {
        let q = format!(
            "doc.evaluate({sel}, doc, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue",
            sel = js_str(selector)
        );
        let qd = format!(
            "document.evaluate({sel}, document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue",
            sel = js_str(selector)
        );
        (qd, q)
    } else {
        (
            format!("document.querySelector({sel})", sel = js_str(selector)),
            format!("doc.querySelector({sel})", sel = js_str(selector)),
        )
    };

    // Search order:
    // 1. iframe[name="main"] — settings panel and its dialogs
    // 2. Main document — save/preview buttons and any top-level overlays
    // Non-main iframes (article editor) intentionally excluded to avoid false matches.
    let js = format!(
        r#"(() => {{
            try {{
                var click = function(n) {{ n.scrollIntoView({{block:'center'}}); n.click(); return true; }};
                var mainFr = document.querySelector('iframe[name="main"]');
                if (mainFr) {{
                    try {{
                        var doc = mainFr.contentDocument;
                        if (doc) {{ var m = {qf}; if (m) return click(m); }}
                    }} catch(e) {{}}
                }}
                var n = {qd};
                if (n) return click(n);
                return false;
            }} catch(e) {{ return false; }}
        }})()"#,
        qf = query_frame,
        qd = query_doc,
    );

    page.evaluate(js.as_str())
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_bool()))
        .unwrap_or(false)
}

/// Searches non-main iframes only — for article editor toolbar buttons.
async fn xclick_editor(page: &Page, selector: &str) -> bool {
    let query_frame = if selector.starts_with('/') {
        format!(
            "doc.evaluate({sel}, doc, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue",
            sel = js_str(selector)
        )
    } else {
        format!("doc.querySelector({sel})", sel = js_str(selector))
    };

    let js = format!(
        r#"(() => {{
            try {{
                var frames = document.querySelectorAll('iframe:not([name="main"])');
                for (var i = 0; i < frames.length; i++) {{
                    try {{
                        var doc = frames[i].contentDocument;
                        if (!doc) continue;
                        var m = {qf};
                        if (m) {{ m.scrollIntoView({{block:'center'}}); m.click(); return true; }}
                    }} catch(e) {{}}
                }}
                return false;
            }} catch(e) {{ return false; }}
        }})()"#,
        qf = query_frame,
    );

    page.evaluate(js.as_str())
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_bool()))
        .unwrap_or(false)
}

async fn retry_click_editor(page: &Page, selectors: &[&str], attempts: u32, delay_ms: u64) -> bool {
    for _ in 0..attempts {
        for &sel in selectors {
            if xclick_editor(page, sel).await {
                return true;
            }
        }
        sleep_ms(delay_ms).await;
    }
    false
}

/// Try each selector in sequence, retrying up to `attempts` times with `delay_ms` between rounds.
async fn retry_click(page: &Page, selectors: &[&str], attempts: u32, delay_ms: u64) -> bool {
    for _ in 0..attempts {
        for &sel in selectors {
            if xclick(page, sel).await {
                return true;
            }
        }
        sleep_ms(delay_ms).await;
    }
    false
}

// ── wait helpers ──────────────────────────────────────────────────────────────

/// Wait until the page URL contains `needle`; returns the full URL.
async fn wait_url(page: &Page, needle: &str) -> String {
    loop {
        if let Some(u) = page.url().await.unwrap_or(None)
            && u.contains(needle)
        {
            return u;
        }
        sleep_ms(500).await;
    }
}

/// Wait up to `timeout_ms` for a CSS selector to match something in the DOM.
async fn wait_css(page: &Page, css: &str, timeout_ms: u64) -> bool {
    let deadline = std::time::Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        if page.find_element(css).await.is_ok() {
            return true;
        }
        if std::time::Instant::now() >= deadline {
            return false;
        }
        sleep_ms(300).await;
    }
}

async fn sleep_ms(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
}

// ── JS string quoting ─────────────────────────────────────────────────────────

/// Wrap `s` in double quotes with minimal JS-safe escaping.
fn js_str(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

// ── browser / profile ─────────────────────────────────────────────────────────

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
            .arg("--start-maximized")
            .build()
            .map_err(|e| e.to_string())?,
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
    let pages = browser.pages().await.map_err(|e| e.to_string())?;
    let page = if !pages.is_empty() {
        pages.into_iter().next().unwrap()
    } else {
        browser
            .new_page("about:blank")
            .await
            .map_err(|e| e.to_string())?
    };
    Ok((browser, page))
}
