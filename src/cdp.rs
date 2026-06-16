//! CDP (Chrome DevTools Protocol) helpers for browser automation.
//!
//! This module isolates all chromiumoxide usage and JS evaluation tricks.
//! Keeping it separate from `publish.rs` makes the automation steps easier to
//! read and test independently.

use std::path::PathBuf;
use std::time::Duration;

use chromiumoxide::Page;
use chromiumoxide::browser::{Browser, BrowserConfig};
use chromiumoxide::cdp::browser_protocol::network::{Cookie, CookieParam};
use futures::StreamExt;

/// Run an async closure on a fresh Tokio runtime.
///
/// WeChat automation is not hot-path code, so creating a runtime per call keeps
/// the public API synchronous and avoids forcing async all the way up to main().
pub fn run<F>(f: F) -> Result<String, String>
where
    F: std::future::Future<Output = Result<String, String>>,
{
    tokio::runtime::Runtime::new()
        .map_err(|e| e.to_string())?
        .block_on(f)
}

pub fn readline() {
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).ok();
}

pub fn wait_enter() {
    print!("  → 按 Enter 继续...");
    std::io::Write::flush(&mut std::io::stdout()).ok();
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).ok();
}

pub fn ask_ok(prompt: &str) -> bool {
    print!("  → {prompt} (y/n): ");
    std::io::Write::flush(&mut std::io::stdout()).ok();
    let mut buf = String::new();
    std::io::stdin().read_line(&mut buf).ok();
    matches!(buf.trim().to_lowercase().as_str(), "y" | "yes" | "")
}

pub async fn sleep_ms(ms: u64) {
    tokio::time::sleep(Duration::from_millis(ms)).await;
}

pub async fn shot(page: &Page, path: &std::path::Path) {
    let params = chromiumoxide::page::ScreenshotParams::builder()
        .full_page(true)
        .build();
    page.save_screenshot(params, path).await.ok();
}

// ── click helpers ─────────────────────────────────────────────────────────────

/// Click an element by XPath (starts with '/') or CSS selector.
/// Searches iframe[name="main"] first (WeChat editor settings sandbox),
/// then falls back to the main document.
/// Returns true if the element was found and clicked.
pub async fn xclick(page: &Page, selector: &str) -> bool {
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

/// Try each selector in sequence, retrying up to `attempts` times with `delay_ms` between rounds.
pub async fn retry_click(page: &Page, selectors: &[&str], attempts: u32, delay_ms: u64) -> bool {
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
pub async fn cdp_click_text(page: &Page, text: &str) -> bool {
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

/// CDP coordinate click any visible element (span/label/div/a/button/li) whose
/// textContent contains `text`. Searches main document + all iframes + shadow DOMs.
pub async fn cdp_click_any_text(page: &Page, text: &str) -> bool {
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

pub async fn cdp_click_css(page: &Page, selector: &str) -> bool {
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

pub async fn cdp_click_exact_last(page: &Page, text: &str) -> bool {
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

// ── dialog helpers ────────────────────────────────────────────────────────────

/// Build a JS snippet that returns true if any of `texts` is visible in the page
/// (main document, iframes, or shadow DOMs).
pub fn has_visible_text_js(texts: &[&str]) -> String {
    let targets = texts
        .iter()
        .map(|t| js_str(t))
        .collect::<Vec<_>>()
        .join(",");
    format!(
        r#"(function(){{
    var targets = [{}];
    var search = function(root) {{
        var els = root.querySelectorAll('span, label, div, p, h1, h2, h3, h4, button');
        for (var i=0;i<els.length;i++) {{
            if (els[i].offsetParent === null) continue;
            var txt = els[i].textContent.trim();
            for (var j=0;j<targets.length;j++) {{
                if (txt === targets[j]) return true;
            }}
        }}
        var all = root.querySelectorAll('*');
        for (var k=0;k<all.length;k++) if(all[k].shadowRoot && search(all[k].shadowRoot)) return true;
        return false;
    }};
    if (search(document)) return true;
    var frames = document.querySelectorAll('iframe');
    for (var f=0;f<frames.length;f++){{
        try{{ var d=frames[f].contentDocument; if(d && search(d)) return true; }}catch(e){{}}
    }}
    return false;
}})()"#,
        targets
    )
}

/// Evaluate JS to check whether any of `texts` is currently visible.
pub async fn has_visible_text(page: &Page, texts: &[&str]) -> bool {
    page.evaluate(has_visible_text_js(texts))
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_bool()))
        .unwrap_or(false)
}

/// Close the top-most dialog by pressing Escape, then clicking cancel/close if still open.
pub async fn close_dialog(page: &Page) -> bool {
    let _ = page
        .evaluate("document.dispatchEvent(new KeyboardEvent('keydown',{key:'Escape',code:'Escape',bubbles:true}))")
        .await;
    sleep_ms(300).await;
    let ok = cdp_click_exact_last(page, "取消").await
        || cdp_click_exact_last(page, "关闭").await
        || cdp_click_text(page, "取消").await
        || cdp_click_text(page, "关闭").await;
    sleep_ms(300).await;
    ok
}

/// Click the agreement checkbox in a WeChat dialog.
///
/// WeChat's Vue-based checkbox often ignores clicks on the label text; the
/// actual `input[type="checkbox"]` element must be clicked. This helper tries
/// text first, then falls back to checkbox input lookup by nearby label text.
pub async fn check_agreement(page: &Page) -> bool {
    if cdp_click_any_text(page, "我已阅读并同意").await {
        return true;
    }
    page.evaluate(
        r#"(function(){
        var search=function(root){
            // 1. Find visible checkbox whose label contains agreement text
            var inputs=root.querySelectorAll('input[type="checkbox"]');
            for(var i=0;i<inputs.length;i++){
                var inp=inputs[i];
                if(inp.offsetParent===null) continue;
                var label=inp.closest('label');
                var txt='';
                if(label) txt=label.textContent;
                else txt=(inp.getAttribute('aria-label')||'');
                txt=txt.trim();
                if(txt.indexOf('我已阅读并同意')>=0||txt.indexOf('已阅读')>=0||txt.indexOf('同意')>=0){
                    inp.click();
                    return true;
                }
            }
            // 2. Find label by text and click its checkbox or the label itself
            var labels=root.querySelectorAll('label');
            for(var j=0;j<labels.length;j++){
                var t=labels[j].textContent.trim();
                if(t.indexOf('我已阅读并同意')>=0||t.indexOf('已阅读')>=0){
                    var cb=labels[j].querySelector('input[type="checkbox"]');
                    if(cb){cb.click();return true;}
                    labels[j].click();
                    return true;
                }
            }
            return false;
        };
        if(search(document))return true;
        var frames=document.querySelectorAll('iframe');
        for(var f=0;f<frames.length;f++){try{var d=frames[f].contentDocument;if(d&&search(d))return true;}catch(e){}}
        return false;
    })()"#,
    )
    .await
    .ok()
    .and_then(|v| v.value().and_then(|v| v.as_bool()))
    .unwrap_or(false)
}

// ── wait helpers ──────────────────────────────────────────────────────────────

/// Wait until the page URL contains `needle`; returns the full URL, or empty string on timeout.
pub async fn wait_url(page: &Page, needle: &str) -> String {
    let deadline = std::time::Instant::now() + Duration::from_secs(120);
    loop {
        if let Some(u) = page.url().await.unwrap_or(None)
            && u.contains(needle)
        {
            return u;
        }
        if std::time::Instant::now() >= deadline {
            return String::new();
        }
        sleep_ms(500).await;
    }
}

/// Wait up to `timeout_ms` for a CSS selector to match something in the DOM.
pub async fn wait_css(page: &Page, css: &str, timeout_ms: u64) -> bool {
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

// ── JS string quoting ─────────────────────────────────────────────────────────

/// Wrap `s` in double quotes with minimal JS-safe escaping.
pub fn js_str(s: &str) -> String {
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

pub fn profile_dir() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    let p = PathBuf::from(format!("{home}/.config/moonpub/chrome-profile"));
    std::fs::create_dir_all(&p).ok();
    p
}

pub fn session_file() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_default();
    let dir = PathBuf::from(format!("{home}/.config/moonpub"));
    std::fs::create_dir_all(&dir).ok();
    dir.join("session.json")
}

pub async fn save_session(browser: &Browser) {
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
pub async fn try_restore_session(browser: &Browser, page: &Page) -> bool {
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

pub async fn open_browser(headless: bool) -> Result<(Browser, Page), String> {
    let mut config = BrowserConfig::builder()
        .no_sandbox()
        .user_data_dir(profile_dir());
    if headless {
        config = config.new_headless_mode();
        config = config.window_size(1920, 1080);
    } else {
        config = config.with_head();
        config = config.arg("--start-maximized");
    }
    let (browser, mut handler) = Browser::launch(config.build().map_err(|e| e.to_string())?)
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
    let page = if let Some(first) = pages.into_iter().next() {
        first
    } else {
        browser
            .new_page("about:blank")
            .await
            .map_err(|e| e.to_string())?
    };
    Ok((browser, page))
}

/// Login → draft list → enter editor → scroll to settings area.
///
/// This is the main entry point for all WeChat editor automation. It restores
/// the saved session if possible, navigates to the draft list, and clicks the
/// first draft's edit button to open the editor.
pub async fn setup_editor(headed: bool) -> Result<(Browser, Page), String> {
    let (browser, page) = open_browser(!headed).await?;

    if try_restore_session(&browser, &page).await {
        println!("  ✅ Session 已恢复，无需扫码");
    } else {
        println!("▶ 请扫描二维码登录...");
        page.goto("https://mp.weixin.qq.com")
            .await
            .map_err(|e| format!("nav: {e}"))?;
        let login_url = wait_url(&page, "cgi-bin/home").await;
        if login_url.is_empty() {
            return Err("login timeout: QR code not scanned within 120s".into());
        }
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
    // WeChat draft cards open the editor in a new tab via window.open().
    // CDP mouse click (page.click) is treated as a real user gesture by Chromium,
    // so window.open() is allowed. Extracting appmsgid from static DOM doesn't work
    // because WeChat uses Vue.js — the attribute is not in the static markup.
    let rect_json: String = page
        .evaluate(
            r#"(() => {
                var btns = document.querySelectorAll('.weui-desktop-card__action a.weui-desktop-icon-btn');
                var idx = btns.length >= 2 ? 1 : 0;
                if (btns.length === 0) return JSON.stringify({found: false, count: 0});
                var btn = btns[idx];
                btn.scrollIntoView({block: 'center'});
                var r = btn.getBoundingClientRect();
                return JSON.stringify({found: true, x: r.x + r.width/2, y: r.y + r.height/2, idx: idx, count: btns.length});
            })()"#,
        )
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();
    if let Ok(v) = serde_json::from_str::<serde_json::Value>(&rect_json)
        && v["found"].as_bool() == Some(true)
    {
        let x = v["x"].as_f64().unwrap_or(0.0);
        let y = v["y"].as_f64().unwrap_or(0.0);
        let idx = v["idx"].as_u64().unwrap_or(0);
        let count = v["count"].as_u64().unwrap_or(0);
        println!("  click btn[{idx}] of {count} at ({x:.0},{y:.0})");
        page.click(chromiumoxide::layout::Point { x, y }).await.ok();
    } else {
        return Err(format!("draft list edit button not found: {rect_json}"));
    }

    // Poll browser tabs for the new editor page opened by WeChat's window.open()
    let mut edit_page: Option<Page> = None;
    for _ in 0..60 {
        sleep_ms(1_000).await;
        if let Ok(all) = browser.pages().await {
            for p in all {
                let u = p.url().await.unwrap_or(None).unwrap_or_default();
                if u.contains("appmsg_edit") {
                    edit_page = Some(p);
                    break;
                }
            }
        }
        if edit_page.is_some() {
            break;
        }
    }
    let page = match edit_page {
        Some(p) => {
            println!("  ✅ In editor");
            p
        }
        None => return Err("editor page did not open within 60s — WeChat may have blocked popup or wrong button clicked".into()),
    };
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

#[cfg(test)]
mod tests {
    use super::js_str;

    #[test]
    fn js_str_wraps_plain_text() {
        assert_eq!(js_str("hello"), "\"hello\"");
    }

    #[test]
    fn js_str_escapes_double_quotes() {
        assert_eq!(js_str("a\"b"), "\"a\\\"b\"");
    }

    #[test]
    fn js_str_escapes_backslashes() {
        assert_eq!(js_str("a\\b"), "\"a\\\\b\"");
    }

    #[test]
    fn js_str_escapes_control_chars() {
        assert_eq!(js_str("a\nb\tc\r"), "\"a\\nb\\tc\\r\"");
    }

    #[test]
    fn js_str_empty_string() {
        assert_eq!(js_str(""), "\"\"");
    }

    #[test]
    fn has_visible_text_js_includes_targets() {
        let js = super::has_visible_text_js(&["声明类型", "文字原创"]);
        assert!(js.contains("声明类型"));
        assert!(js.contains("文字原创"));
        assert!(js.contains("span, label, div, p, h1, h2, h3, h4, button"));
    }
}
