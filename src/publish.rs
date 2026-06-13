//! Pure Rust CDP automation via chromiumoxide — dedicated profile, no re-login.
//!
//! Key design: all clicks use JS evaluate so XPath selectors work correctly.
//! chromiumoxide's find_element() only accepts CSS — XPath must go through JS.

use chromiumoxide::Page;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::network::{Cookie, CookieParam};
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

pub fn auto_configure(_mid: &str, collection: &str, steps: &[String]) -> Result<String, String> {
    let _collection = if collection.is_empty() {
        "书"
    } else {
        collection
    };
    let steps = steps.to_vec();
    run(async move {
        let run_step = |name: &str| steps.is_empty() || steps.iter().any(|s| s == name);
        let (browser, page) = setup_editor().await?;

        if run_step(STEP_YUANZHUANG) {
            step_yuanzhuang(&page).await;
        }
        if run_step(STEP_ZANSHANG) {
            step_zanshang(&page).await;
        }
        println!("▶ 合集... (skipped)");
        if run_step(STEP_LIUYAN) {
            step_liuyan(&page).await;
        }
        if run_step(STEP_CHUANGZUO) {
            step_chuangzuo(&page).await;
        }
        if run_step(STEP_YULAN) {
            step_yulan(&page).await;
        }

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
    let params = chromiumoxide::page::ScreenshotParams::builder()
        .full_page(true)
        .build();
    page.save_screenshot(params, path).await.ok();
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

        // ── Step 6b: 搜索 + 选中卡片 ──
        s += 1;
        println!("\n══ Step {s}b: 搜索并选中「寻月隐君」 ══");
        wait_enter();
        // 与 auto_configure 完全相同的搜索逻辑
        let typed = page.evaluate(format!(
            r#"(() => {{
                var name = {0};
                var dialog = document.querySelector('mp-insert-profile-dialog');
                var scope = dialog ? (dialog.shadowRoot || dialog) : document;
                var inputs = scope.querySelectorAll('input[type="text"], input:not([type])');
                for (var i = 0; i < inputs.length; i++) {{
                    var inp = inputs[i];
                    if (inp.offsetParent !== null) {{
                        inp.focus();
                        inp.value = name;
                        inp.dispatchEvent(new Event('input', {{bubbles:true}}));
                        inp.dispatchEvent(new Event('change', {{bubbles:true}}));
                        return 'typed in dialog';
                    }}
                }}
                var allInputs = document.querySelectorAll('input[type="text"], input:not([type])');
                for (var i = 0; i < allInputs.length; i++) {{
                    var inp2 = allInputs[i];
                    if (inp2.offsetParent !== null && inp2.placeholder && inp2.placeholder.includes('账号')) {{
                        inp2.focus();
                        inp2.value = name;
                        inp2.dispatchEvent(new Event('input', {{bubbles:true}}));
                        inp2.dispatchEvent(new Event('change', {{bubbles:true}}));
                        return 'typed via placeholder fallback';
                    }}
                }}
                return 'no dialog input found';
            }})()"#,
            js_str("寻月隐君")
        )).await.ok().and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned()))).unwrap_or_default();
        println!("    搜索: {typed}");
        sleep_ms(3_000).await;
        // 点击选中卡片
        let ok_card = retry_click(
            &page,
            &[
                "//div[contains(@class, 'wx_profile_card') and .//em[contains(text(), '寻月隐君')]]",
                "//div[contains(@class, 'wx_profile_card') and contains(., '寻月隐君')]",
            ],
            8,
            400,
        ).await;
        println!("    选中卡片: {ok_card}");
        sleep_ms(1_000).await;
        shot(&page, &dir.join(format!("step{s:02}b.png"))).await;
        if !ask_ok("选中寻月隐君了？(应有绿色边框)") {
            return Err("取消".into());
        }

        // ── Step 6c: 点击插入 ──
        s += 1;
        println!("\n══ Step {s}c: 点击插入 ══");
        wait_enter();
        let ok4 = retry_click(
            &page,
            &[
                "//mp-image-product-dialog//button[contains(text(), '插入')]",
                "//div[contains(@class, 'weui-desktop-dialog')]//button[contains(text(), '插入')]",
                "//button[normalize-space(text())='插入']",
            ],
            10,
            400,
        )
        .await;
        println!("    插入: {ok4}");
        sleep_ms(1_000).await;
        shot(&page, &dir.join(format!("step{s:02}c.png"))).await;
        if !ask_ok("账号名片插入成功？") {
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
                var click = function(n) {{ n.scrollIntoView({{block:'center'}}); var o = {{bubbles:true, cancelable:true, view:window}}; n.dispatchEvent(new MouseEvent('mousedown', o)); n.dispatchEvent(new MouseEvent('mouseup', o)); n.click(); return true; }};
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

#[allow(dead_code)]
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

#[allow(dead_code)]
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

/// CDP coordinate click a button whose textContent equals `text` exactly.
/// Searches main document + all iframes + shadow DOMs.
async fn cdp_click_text(page: &Page, text: &str) -> bool {
    let rect_json = page
        .evaluate(format!(
            r#"(() => {{
                var t = {0};
                var search = function(root) {{
                    var btns = root.querySelectorAll('button');
                    for (var i = 0; i < btns.length; i++) {{
                        var b = btns[i];
                        if (b.offsetParent !== null && b.textContent && b.textContent.trim() === t) {{
                            b.scrollIntoView({{block:'center'}});
                            var r = b.getBoundingClientRect();
                            return JSON.stringify({{found:true, x: r.x + r.width/2, y: r.y + r.height/2}});
                        }}
                    }}
                    var all = root.querySelectorAll('*');
                    for (var j = 0; j < all.length; j++) {{
                        if (all[j].shadowRoot) {{
                            var sr = search(all[j].shadowRoot);
                            if (sr) return sr;
                        }}
                    }}
                    return null;
                }};
                var r = search(document);
                if (r) return r;
                var frames = document.querySelectorAll('iframe');
                for (var f = 0; f < frames.length; f++) {{
                    try {{
                        var d = frames[f].contentDocument;
                        if (d) {{ r = search(d); if (r) return r; }}
                    }} catch(e) {{}}
                }}
                return JSON.stringify({{found:false}});
            }})()"#,
            js_str(text)
        ))
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&rect_json)
        && v["found"].as_bool() == Some(true)
    {
        let x = v["x"].as_f64().unwrap_or(0.0);
        let y = v["y"].as_f64().unwrap_or(0.0);
        page.click(chromiumoxide::layout::Point { x, y }).await.ok();
        true
    } else {
        false
    }
}

/// CDP coordinate click the first visible element matching any of the XPath selectors.
/// Searches main document, then all accessible iframes.
#[allow(dead_code)]
async fn cdp_click_xpath(page: &Page, selectors: &[&str], attempts: u32, delay_ms: u64) -> bool {
    for _ in 0..attempts {
        for &sel in selectors {
            let rect_json = page
                .evaluate(format!(
                    r#"(() => {{
                        var sel = {0};
                        var find = function(doc) {{
                            try {{
                                return doc.evaluate(sel, doc, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null).singleNodeValue;
                            }} catch(e) {{ return null; }}
                        }};
                        var node = find(document);
                        if (!node) {{
                            var frames = document.querySelectorAll('iframe');
                            for (var f = 0; f < frames.length && !node; f++) {{
                                try {{
                                    var d = frames[f].contentDocument;
                                    if (d) node = find(d);
                                }} catch(e) {{}}
                            }}
                        }}
                        if (node && node.offsetParent !== null) {{
                            node.scrollIntoView({{block:'center'}});
                            var r = node.getBoundingClientRect();
                            return JSON.stringify({{found:true, x: r.x + r.width/2, y: r.y + r.height/2}});
                        }}
                        return JSON.stringify({{found:false}});
                    }})()"#,
                    js_str(sel)
                ))
                .await
                .ok()
                .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
                .unwrap_or_default();
            if let Ok(v) = serde_json::from_str::<serde_json::Value>(&rect_json)
                && v["found"].as_bool() == Some(true)
            {
                let x = v["x"].as_f64().unwrap_or(0.0);
                let y = v["y"].as_f64().unwrap_or(0.0);
                page.click(chromiumoxide::layout::Point { x, y }).await.ok();
                return true;
            }
        }
        sleep_ms(delay_ms).await;
    }
    false
}

/// CDP coordinate click any visible element (span/label/div/a/button/li) whose
/// textContent contains `text`. Searches main document + all iframes + shadow DOMs.
async fn cdp_click_any_text(page: &Page, text: &str) -> bool {
    let rect_json = page
        .evaluate(format!(
            r#"(() => {{
                var t = {0};
                var sel = 'span, label, div, a, button, li';
                var search = function(root) {{
                    var els = root.querySelectorAll(sel);
                    for (var i = 0; i < els.length; i++) {{
                        var el = els[i];
                        if (el.offsetParent !== null && el.textContent && el.textContent.trim().indexOf(t) >= 0) {{
                            el.scrollIntoView({{block:'center'}});
                            var r = el.getBoundingClientRect();
                            return JSON.stringify({{found:true, x: r.x + r.width/2, y: r.y + r.height/2, tag: el.tagName, txt: el.textContent.trim().substring(0,30)}});
                        }}
                    }}
                    // recurse into shadow DOMs
                    var all = root.querySelectorAll('*');
                    for (var j = 0; j < all.length; j++) {{
                        if (all[j].shadowRoot) {{
                            var sr = search(all[j].shadowRoot);
                            if (sr) return sr;
                        }}
                    }}
                    return null;
                }};
                // main document
                var r = search(document);
                if (r) return r;
                // all iframes
                var frames = document.querySelectorAll('iframe');
                for (var f = 0; f < frames.length; f++) {{
                    try {{
                        var d = frames[f].contentDocument;
                        if (d) {{ r = search(d); if (r) return r; }}
                    }} catch(e) {{}}
                }}
                return JSON.stringify({{found:false}});
            }})()"#,
            js_str(text)
        ))
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&rect_json)
        && v["found"].as_bool() == Some(true)
    {
        let x = v["x"].as_f64().unwrap_or(0.0);
        let y = v["y"].as_f64().unwrap_or(0.0);
        page.click(chromiumoxide::layout::Point { x, y }).await.ok();
        true
    } else {
        false
    }
}

async fn cdp_click_css(page: &Page, selector: &str) -> bool {
    let js = format!(
        r#"(function(){{
    var sel = "{}";
    var search = function(root) {{
        var el = root.querySelector(sel);
        if (el && el.offsetParent !== null) {{
            var r = el.getBoundingClientRect();
            return JSON.stringify({{found:true,x:r.left+r.width/2,y:r.top+r.height/2}});
        }}
        var all = root.querySelectorAll('*');
        for (var i=0;i<all.length;i++) if(all[i].shadowRoot){{
            var res=search(all[i].shadowRoot);
            if(res) return res;
        }}
        return null;
    }};
    var res = search(document);
    if (res) return res;
    var frames = document.querySelectorAll('iframe');
    for (var f=0;f<frames.length;f++){{
        try{{ var d=frames[f].contentDocument; if(d){{ res=search(d); if(res) return res; }} }}catch(e){{}}
    }}
    return JSON.stringify({{found:false}});
}})()"#,
        selector.replace('"', "\\\"")
    );
    let rect_json = page
        .evaluate(js)
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&rect_json)
        && v["found"].as_bool() == Some(true)
    {
        let x = v["x"].as_f64().unwrap_or(0.0);
        let y = v["y"].as_f64().unwrap_or(0.0);
        page.click(chromiumoxide::layout::Point { x, y }).await.ok();
        true
    } else {
        false
    }
}

async fn cdp_click_exact_last(page: &Page, text: &str) -> bool {
    let rect_json = page
        .evaluate(format!(
            r#"(function(){{
    var target = {};
    var last = null;
    var search = function(root) {{
        var els = root.querySelectorAll('span, label, div, a, button, li');
        for (var i=0;i<els.length;i++) {{
            var e = els[i];
            if (e.offsetParent !== null) {{
                var txt = e.textContent.trim().replace(/\s+/g,' ');
                if (txt === target) last = e;
            }}
        }}
        var all = root.querySelectorAll('*');
        for (var j=0;j<all.length;j++) if(all[j].shadowRoot) search(all[j].shadowRoot);
    }};
    search(document);
    var frames = document.querySelectorAll('iframe');
    for (var f=0;f<frames.length;f++) {{
        try{{ var d=frames[f].contentDocument; if(d) search(d); }}catch(e){{}}
    }}
    if (last) {{
        var r = last.getBoundingClientRect();
        return JSON.stringify({{found:true,x:r.left+r.width/2,y:r.top+r.height/2}});
    }}
    return JSON.stringify({{found:false}});
}})()"#,
            js_str(text)
        ))
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&rect_json)
        && v["found"].as_bool() == Some(true)
    {
        let x = v["x"].as_f64().unwrap_or(0.0);
        let y = v["y"].as_f64().unwrap_or(0.0);
        page.click(chromiumoxide::layout::Point { x, y }).await.ok();
        true
    } else {
        false
    }
}

/// Diagnostic: dump all visible button texts plus elements matching a text search.
async fn dump_buttons(page: &Page, label: &str) {
    let txt = page
        .evaluate(
            r#"(() => {{
                var out = [];
                var dump = function(el, tag, prefix) {{
                    var t = el.textContent.trim().replace(/\s+/g,' ');
                    if (t.length > 0 && t.length < 32) out.push(prefix + tag + ':' + t);
                }};
                // buttons
                var btns = document.querySelectorAll('button');
                for (var i = 0; i < btns.length; i++) if (btns[i].offsetParent !== null) dump(btns[i], 'btn', '');
                // any element with 开启赞赏/赞赏/确定/确认/关闭
                var all = document.querySelectorAll('span, label, div, a, button');
                var kw = ['开启赞赏','赞赏','确定','确认','关闭','取消'];
                for (var i = 0; i < all.length; i++) {{
                    var el = all[i];
                    if (el.offsetParent !== null) {{
                        var t = el.textContent.trim().replace(/\s+/g,' ');
                        for (var j = 0; j < kw.length; j++) {{
                            if (t.startsWith(kw[j])) dump(el, el.tagName.toLowerCase(), 'txt:');
                        }}
                    }}
                }}
                // iframe[name="main"]
                var fr = document.querySelector('iframe[name="main"]');
                if (fr) {{
                    try {{
                        var doc = fr.contentDocument;
                        if (doc) {{
                            var fbtns = doc.querySelectorAll('button');
                            for (var k = 0; k < fbtns.length; k++) dump(fbtns[k], 'btn', 'iframe:');
                        }}
                    }} catch(e) {{ out.push('iframe err'); }}
                }}
                // shadow DOM: mp-insert-profile-dialog and other custom elements
                var customs = document.querySelectorAll('mp-*, weui-*, [class*="dialog"]');
                for (var m = 0; m < customs.length; m++) {{
                    if (customs[m].shadowRoot) {{
                        var sbtns = customs[m].shadowRoot.querySelectorAll('button, span, label');
                        for (var n = 0; n < sbtns.length; n++) {{
                            if (sbtns[n].offsetParent !== null) dump(sbtns[n], sbtns[n].tagName.toLowerCase(), 'shadow:');
                        }}
                    }}
                }}
                return out.join(' | ') || '(empty)';
            }})()"#,
        )
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();
    println!("    [diag {label}]: {txt}");
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

pub fn test_chuangzuo() -> Result<String, String> {
    run(async {
        let (browser, page) = setup_editor().await?;
        step_yuanzhuang(&page).await;
        step_chuangzuo(&page).await;
        println!("\n── 创作来源测试完成，按 Enter 关闭浏览器...");
        sleep_ms(3_000).await;
        readline();
        drop(browser);
        Ok("done".to_owned())
    })
}

pub fn test_zanshang() -> Result<String, String> {
    run(async {
        let (browser, page) = setup_editor().await?;
        step_yuanzhuang(&page).await;
        step_zanshang(&page).await;
        println!("\n── 赞赏测试完成，按 Enter 关闭浏览器...");
        sleep_ms(3_000).await;
        readline();
        drop(browser);
        Ok("done".to_owned())
    })
}

pub fn test_yulan() -> Result<String, String> {
    run(async {
        let (browser, page) = setup_editor().await?;
        step_yuanzhuang(&page).await;
        step_yulan(&page).await;
        println!("\n── 预览测试完成，按 Enter 关闭浏览器...");
        sleep_ms(3_000).await;
        readline();
        drop(browser);
        Ok("done".to_owned())
    })
}

// ── Step name constants ──────────────────────────────────────────────────────
const STEP_YUANZHUANG: &str = "yuanzhuang";
const STEP_ZANSHANG: &str = "zanshang";
const STEP_LIUYAN: &str = "liuyan";
const STEP_CHUANGZUO: &str = "chuangzuo";
const STEP_YULAN: &str = "yulan";

fn session_file() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    let dir = PathBuf::from(format!("{home}/.config/moonpub"));
    std::fs::create_dir_all(&dir).ok();
    dir.join("session.json")
}

async fn save_session(browser: &Browser) {
    let Ok(cookies) = browser.get_cookies().await else {
        return;
    };
    let wx: Vec<&Cookie> = cookies
        .iter()
        .filter(|c| c.domain.contains("weixin.qq.com"))
        .collect();
    if let Ok(json) = serde_json::to_string_pretty(&wx) {
        std::fs::write(session_file(), &json).ok();
        println!("  [session: {} cookies saved]", wx.len());
    }
}

/// Inject saved cookies, navigate to WeChat MP, return true if already logged in.
async fn try_restore_session(browser: &Browser, page: &Page) -> bool {
    let path = session_file();
    if !path.exists() {
        return false;
    }
    let Ok(json) = std::fs::read_to_string(&path) else {
        return false;
    };
    let Ok(saved) = serde_json::from_str::<Vec<Cookie>>(&json) else {
        return false;
    };
    let params: Vec<CookieParam> = saved
        .into_iter()
        .filter_map(|c| {
            serde_json::to_value(&c)
                .ok()
                .and_then(|v| serde_json::from_value(v).ok())
        })
        .collect();
    if browser.set_cookies(params).await.is_err() {
        return false;
    }
    if page.goto("https://mp.weixin.qq.com").await.is_err() {
        return false;
    }
    for _ in 0..10 {
        sleep_ms(500).await;
        let url = page.url().await.unwrap_or(None).unwrap_or_default();
        if url.contains("cgi-bin/home") {
            return true;
        }
        if url.contains("login") {
            return false;
        }
    }
    false
}

/// Login → draft list → enter editor → scroll to settings area.
async fn setup_editor() -> Result<(Browser, Page), String> {
    let (browser, page) = open_browser().await?;

    if try_restore_session(&browser, &page).await {
        println!("  ✅ Session 已恢复，无需扫码");
    } else {
        println!("▶ 请扫描二维码登录...");
        page.goto("https://mp.weixin.qq.com")
            .await
            .map_err(|e| format!("nav: {e}"))?;
        wait_url(&page, "cgi-bin/home").await;
        save_session(&browser).await;
        println!("  ✅ 登录成功");
    }

    let url = page.url().await.unwrap_or(None).unwrap_or_default();
    let token = url
        .split("token=")
        .nth(1)
        .and_then(|s| s.split('&').next())
        .unwrap_or("")
        .to_owned();
    let token = token.as_str();
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
    println!("▶ Entering editor...");
    if let Ok(btns) = page
        .find_elements(".weui-desktop-card__action a.weui-desktop-icon-btn")
        .await
        && btns.len() >= 2
    {
        btns[1].click().await.ok();
    }
    let mut editor_opt: Option<Page> = None;
    for _ in 0..25 {
        sleep_ms(800).await;
        if let Ok(all) = browser.pages().await {
            for p in all {
                let u = p.url().await.unwrap_or(None).unwrap_or_default();
                if u.contains("appmsg_edit") {
                    editor_opt = Some(p);
                    break;
                }
            }
        }
        if editor_opt.is_some() {
            break;
        }
        let cur = page.url().await.unwrap_or(None).unwrap_or_default();
        if cur.contains("appmsg_edit") {
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
    let _ = page
        .evaluate("window.scrollTo(0, document.body.scrollHeight)")
        .await;
    sleep_ms(2_000).await;
    let _ = page
        .evaluate("window.scrollTo(0, document.body.scrollHeight - 500)")
        .await;
    sleep_ms(1_000).await;
    Ok((browser, page))
}

async fn step_yuanzhuang(page: &Page) {
    println!("▶ 原创声明...");
    let ok = retry_click(
        page,
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
        // Vue checkbox needs physical CDP event, not synthetic JS click
        let ok2 = cdp_click_any_text(page, "我已阅读并同意").await;
        if !ok2 {
            let _ = retry_click(
                page,
                &[
                    "//label[contains(.,'已阅读')]",
                    "//span[contains(@class,'checkbox')]",
                ],
                4,
                200,
            )
            .await;
        }
        println!("    check '已阅读': {ok2}");
        sleep_ms(500).await;
        let ok3 = retry_click(
            page,
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
        sleep_ms(500).await;
        println!("  ✅");
    } else {
        println!("  ⚠ '未声明' not found — skipping");
    }
}

async fn step_zanshang(page: &Page) {
    println!("▶ 赞赏...");
    let _ = page
        .evaluate("window.scrollTo(0, document.body.scrollHeight)")
        .await;
    sleep_ms(800).await;
    let diag_trigger = page
        .evaluate(
            r#"(() => {
        var out = [];
        var search = function(root, label) {
            var els = root.querySelectorAll('span, label, div, a, button, li');
            for (var i = 0; i < els.length; i++) {
                var e = els[i];
                if (e.offsetParent !== null) {
                    var txt = e.textContent.trim().replace(/\s+/g,' ');
                    if (txt === '赞赏') {
                        var r = e.getBoundingClientRect();
                        out.push(label+e.tagName+'['+e.className.substring(0,30)+']@y='+Math.round(r.y));
                    }
                }
            }
            var all = root.querySelectorAll('*');
            for (var j=0;j<all.length;j++) if(all[j].shadowRoot) search(all[j].shadowRoot,label+'S:');
        };
        search(document,'');
        var frames=document.querySelectorAll('iframe');
        for(var f=0;f<frames.length;f++){try{var d=frames[f].contentDocument;if(d)search(d,'I:');}catch(e){}}
        return out.join(' | ') || '(none)';
    })()"#,
        )
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();
    println!("    [diag trigger]: {diag_trigger}");
    let ok = cdp_click_exact_last(page, "赞赏").await;
    println!("    click '赞赏': {ok}");
    if !ok {
        println!("  ❌ 赞赏 trigger not found");
        return;
    }
    sleep_ms(1_000).await;
    shot(page, std::path::Path::new("/tmp/zanshang-1-dialog.png")).await;
    println!("    [shot] /tmp/zanshang-1-dialog.png");
    let dialog_open = page
        .evaluate(
            r#"(function(){
        var check=function(root){return root.querySelector('.js_reward_setting_tips')!==null;};
        if(check(document))return true;
        var frames=document.querySelectorAll('iframe');
        for(var f=0;f<frames.length;f++){try{if(check(frames[f].contentDocument))return true;}catch(e){}}
        return false;
    })()"#,
        )
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_bool()))
        .unwrap_or(false);
    println!("    dialog open: {dialog_open}");
    if !dialog_open {
        println!("  ❌ 赞赏 dialog did not open");
        return;
    }
    let ok2 = cdp_click_css(page, ".js_reward_setting_tips").await;
    println!("    click toggle: {ok2}");
    sleep_ms(2_000).await;
    shot(
        page,
        std::path::Path::new("/tmp/zanshang-2-after-toggle.png"),
    )
    .await;
    println!("    [shot] /tmp/zanshang-2-after-toggle.png");
    let mut ok3 = cdp_click_css(page, ".weui-desktop-btn_primary").await;
    if !ok3 {
        ok3 = cdp_click_text(page, "确定").await;
    }
    println!("    click '确定': {ok3}");
    sleep_ms(1_000).await;
    shot(
        page,
        std::path::Path::new("/tmp/zanshang-3-after-confirm.png"),
    )
    .await;
    println!("    [shot] /tmp/zanshang-3-after-confirm.png");
    sleep_ms(1_500).await;
    let _ = page
        .evaluate("window.scrollTo(0, document.body.scrollHeight)")
        .await;
    sleep_ms(500).await;
    let zs_state = page
        .evaluate(
            r#"(function(){
        var search=function(root){
            var el=root.querySelector('.js_reward_setting_tips');
            if(el) return el.textContent.trim();
            var frames=root.querySelectorAll('iframe');
            for(var f=0;f<frames.length;f++){
                try{var d=frames[f].contentDocument;if(d){var r=search(d);if(r)return r;}}catch(e){}
            }
            return null;
        };
        return search(document)||'(not found)';
    })()"#,
        )
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();
    println!("    赞赏 state: '{zs_state}'");
    shot(page, std::path::Path::new("/tmp/zanshang-4-state.png")).await;
    println!("    [shot] /tmp/zanshang-4-state.png");
    if !ok3 {
        println!("  ❌ 赞赏 '确定' not found");
    } else if zs_state == "不开启" || zs_state.is_empty() {
        println!("  ❌ 赞赏 未开启 (state='{zs_state}') — 对话框内 toggle 未切换");
    } else {
        println!("  ✅ 赞赏 已开启 (state='{zs_state}')");
    }
}

async fn step_liuyan(page: &Page) {
    println!("▶ 留言...");
    let ok = retry_click(page, &["//*[text()='留言']"], 8, 400).await;
    println!("    click '留言': {ok}");
    if ok {
        sleep_ms(1_000).await;
        sleep_ms(600).await;
        dump_buttons(page, "留言 dialog").await;
        let ok2 = cdp_click_text(page, "确定").await;
        println!("    click '确定': {ok2}");
        sleep_ms(500).await;
        println!("  ✅");
    } else {
        println!("  ⚠ '留言' not found — skipping");
    }
}

async fn step_chuangzuo(page: &Page) {
    println!("▶ 创作来源...");
    let _ = page
        .evaluate("window.scrollTo(0, document.body.scrollHeight)")
        .await;
    sleep_ms(500).await;
    let ok = cdp_click_exact_last(page, "未添加").await;
    println!("    click '未添加': {ok}");
    if !ok {
        println!("  ⚠ '未添加' not found — skipping 创作来源");
        return;
    }
    sleep_ms(1_500).await;
    let ok2 = cdp_click_exact_last(page, "个人观点，仅供参考").await;
    println!("    select '个人观点': {ok2}");
    sleep_ms(1_000).await;
    let mut ok3 = cdp_click_css(page, ".weui-desktop-dialog__ft .weui-desktop-btn_primary").await;
    if !ok3 {
        ok3 = cdp_click_css(page, ".weui-desktop-btn_primary").await;
    }
    if !ok3 {
        ok3 = cdp_click_text(page, "确认").await;
    }
    println!("    click '确认': {ok3}");
    sleep_ms(1_500).await;
    let czly_state = page
        .evaluate(
            r#"(function(){
        var search=function(root){
            var els=root.querySelectorAll('span,div');
            for(var i=0;i<els.length;i++){
                var t=els[i].textContent.trim().replace(/\s+/g,' ');
                if(t==='个人观点，仅供参考'||t==='个人观点') return t;
            }
            var frames=root.querySelectorAll('iframe');
            for(var f=0;f<frames.length;f++){
                try{var d=frames[f].contentDocument;if(d){var r=search(d);if(r)return r;}}catch(e){}
            }
            return null;
        };
        return search(document)||'(not found)';
    })()"#,
        )
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();
    println!("    创作来源 state: '{czly_state}'");
    if ok3 {
        println!("  ✅ 创作来源");
    } else {
        println!("  ❌ 创作来源 '确认' not found");
    }
}

async fn step_yulan(page: &Page) {
    println!("▶ 预览...");
    let _ = page
        .evaluate("window.scrollTo(0, document.body.scrollHeight)")
        .await;
    sleep_ms(500).await;
    let ok = cdp_click_text(page, "预览").await;
    println!("    click '预览': {ok}");
    sleep_ms(1_000).await;
    let ok2 = cdp_click_exact_last(page, "通过公众号列表预览").await;
    println!("    select mode: {ok2}");
    sleep_ms(500).await;
    let mut ok3 = cdp_click_css(page, ".weui-desktop-dialog__ft .weui-desktop-btn_primary").await;
    if !ok3 {
        ok3 = cdp_click_text(page, "确定").await;
    }
    println!("    click '确定': {ok3}");
    sleep_ms(1_000).await;
    if ok3 {
        println!("  ✅ 预览发送成功");
    } else {
        println!("  ⚠ 预览确定点击失败");
    }
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
