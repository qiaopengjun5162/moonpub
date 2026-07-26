//! CDP (Chrome DevTools Protocol) helpers for browser automation.
//!
//! This module isolates all chromiumoxide usage and JS evaluation tricks.
//! Keeping it separate from `publish.rs` makes the automation steps easier to
//! read and test independently.

use std::path::PathBuf;
use std::time::Duration;
use std::{future::Future, pin::Pin};

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

/// Keep `resource` alive until the async closure completes.
///
/// Browser automation futures often depend on side effects driven by a live
/// `Browser` handle. Dropping the handle before awaiting the rest of the flow
/// can cancel the CDP session in the middle of an otherwise valid login step.
pub async fn with_retained_resource<T, R>(
    resource: T,
    f: impl for<'a> FnOnce(&'a T) -> Pin<Box<dyn Future<Output = R> + 'a>>,
) -> R {
    f(&resource).await
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

/// Poll the page (including shadow roots and iframes) for `needle` to appear.
/// Used to wait for lazy-rendered dialogs after a button click.
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
///
/// The click is a trusted CDP mouse event at the element's coordinates, NOT a
/// synthetic JS .click(). WeChat's editor ignores untrusted events for its Vue
/// model updates: synthetic clicks change the visual state but the change
/// never reaches the saved draft (settings silently revert on reload).
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
                var rect = function(n) {{
                    n.scrollIntoView({{block:'center'}});
                    var r = n.getBoundingClientRect();
                    if (r.width === 0 && r.height === 0) {{
                        // Hidden inputs (e.g. weui switches) have no box; click their visible parent/label instead.
                        var p = n.closest('label') || n.parentElement;
                        if (!p) return null;
                        p.scrollIntoView({{block:'center'}});
                        r = p.getBoundingClientRect();
                    }}
                    return JSON.stringify({{found:true, x: r.x + r.width/2, y: r.y + r.height/2}});
                }};
                var mainFr = document.querySelector('iframe[name="main"]');
                if (mainFr) {{
                    try {{
                        var doc = mainFr.contentDocument;
                        if (doc) {{ var m = {qf}; if (m) return rect(m); }}
                    }} catch(e) {{}}
                }}
                var n = {qd};
                if (n) return rect(n);
                return JSON.stringify({{found:false}});
            }} catch(e) {{ return JSON.stringify({{found:false}}); }}
        }})()"#,
        qf = query_frame,
        qd = query_doc,
    );

    let rect_json = page
        .evaluate(js.as_str())
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
    false
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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BrowserProfileMode {
    Persistent,
    Temporary { dir: PathBuf },
}

impl BrowserProfileMode {
    pub fn persistent() -> Self {
        Self::Persistent
    }

    pub fn temporary() -> Self {
        let unique = format!(
            "moonpub-chrome-profile-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_nanos())
                .unwrap_or(0)
        );
        Self::Temporary {
            dir: std::env::temp_dir().join(unique),
        }
    }

    pub fn from_temporary_flag(enabled: bool) -> Self {
        if enabled {
            Self::temporary()
        } else {
            Self::persistent()
        }
    }
}

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

pub fn profile_dir_for(mode: &BrowserProfileMode) -> PathBuf {
    match mode {
        BrowserProfileMode::Persistent => profile_dir(),
        BrowserProfileMode::Temporary { dir } => {
            std::fs::create_dir_all(dir).ok();
            dir.clone()
        }
    }
}

pub fn session_file_for(mode: &BrowserProfileMode) -> Option<PathBuf> {
    match mode {
        BrowserProfileMode::Persistent => Some(session_file()),
        BrowserProfileMode::Temporary { .. } => None,
    }
}

struct TemporaryProfileGuard {
    dir: PathBuf,
}

impl Drop for TemporaryProfileGuard {
    fn drop(&mut self) {
        std::fs::remove_dir_all(&self.dir).ok();
    }
}

pub struct BrowserSession {
    pub browser: Browser,
    pub page: Page,
    _temporary_profile: Option<TemporaryProfileGuard>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WechatHealthStatus {
    Ready,
    NeedsLogin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WechatHealthReport {
    pub status: WechatHealthStatus,
    pub profile_mode: &'static str,
    pub session_file: Option<PathBuf>,
    pub session_file_exists: bool,
    pub current_url: String,
    pub next_command: &'static str,
    pub next_step: &'static str,
}

pub async fn save_session(browser: &Browser, mode: &BrowserProfileMode) {
    let Some(path) = session_file_for(mode) else {
        println!("  [session: temporary profile, skip save]");
        return;
    };
    let Ok(cookies) = browser.get_cookies().await else {
        return;
    };
    let wx: Vec<&Cookie> = cookies
        .iter()
        .filter(|c| c.domain.contains("weixin.qq.com"))
        .collect();
    if let Ok(json) = serde_json::to_string_pretty(&wx) {
        std::fs::write(path, &json).ok();
        println!("  [session: {} cookies saved]", wx.len());
    }
}

/// Inject saved cookies, navigate to WeChat MP, return true if already logged in.
pub async fn try_restore_session(
    browser: &Browser,
    page: &Page,
    mode: &BrowserProfileMode,
) -> bool {
    let Some(path) = session_file_for(mode) else {
        return false;
    };
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

pub async fn check_wechat_health(
    headed: bool,
    mode: &BrowserProfileMode,
) -> Result<WechatHealthReport, String> {
    let session_file = session_file_for(mode);
    let session_file_exists = session_file.as_ref().is_some_and(|path| path.exists());
    let profile_mode = match mode {
        BrowserProfileMode::Persistent => "persistent",
        BrowserProfileMode::Temporary { .. } => "temporary",
    };
    let BrowserSession {
        browser,
        page,
        _temporary_profile,
    } = open_browser(!headed, mode).await?;
    let restored = try_restore_session(&browser, &page, mode).await;
    if !restored {
        page.goto("https://mp.weixin.qq.com")
            .await
            .map_err(|e| format!("nav: {e}"))?;
    }
    sleep_ms(500).await;
    let current_url = sanitize_wechat_url(&page.url().await.unwrap_or(None).unwrap_or_default());
    let (status, next_command, next_step) = if restored || current_url.contains("cgi-bin/home") {
        (
            WechatHealthStatus::Ready,
            "moonpub configure --headed",
            "browser automation login is reusable; continue with WeChat backend preview/configure",
        )
    } else {
        (
            WechatHealthStatus::NeedsLogin,
            "moonpub login",
            "scan the WeChat QR code once, then rerun wechat-health or configure",
        )
    };
    drop(browser);
    Ok(WechatHealthReport {
        status,
        profile_mode,
        session_file,
        session_file_exists,
        current_url,
        next_command,
        next_step,
    })
}

pub fn headless_login_required_message(mode: &BrowserProfileMode) -> String {
    match mode {
        BrowserProfileMode::Persistent => "saved WeChat browser session is not reusable in headless mode. Run `moonpub login` once to refresh the saved session, or rerun `moonpub configure --headed` if you want to scan the QR code in the visible browser.".to_owned(),
        BrowserProfileMode::Temporary { .. } => "temporary profile cannot reuse the saved WeChat browser session in headless mode. Rerun with --headed to scan the QR code for this one-off browser session, or remove --temporary-profile to use the persistent saved session.".to_owned(),
    }
}

pub fn sanitize_wechat_url(url: &str) -> String {
    url.split('?').next().unwrap_or(url).to_owned()
}

pub fn browser_launch_error_message(error: &str, mode: &BrowserProfileMode) -> String {
    if matches!(mode, BrowserProfileMode::Persistent)
        && (error.contains("SingletonLock") || error.contains("ProcessSingleton"))
    {
        return "launch: MoonPub persistent Chrome profile is already in use. Close the existing MoonPub automation Chrome window, or rerun with --temporary-profile for a one-off isolated browser session.".to_owned();
    }
    format!("launch: {error}")
}

pub async fn open_browser(
    headless: bool,
    mode: &BrowserProfileMode,
) -> Result<BrowserSession, String> {
    let profile_dir = profile_dir_for(mode);
    let mut config = BrowserConfig::builder()
        .no_sandbox()
        .user_data_dir(profile_dir);
    if headless {
        config = config.new_headless_mode();
        config = config.window_size(1920, 1080);
    } else {
        config = config.with_head();
        config = config.arg("--start-maximized");
    }
    let (browser, mut handler) = Browser::launch(config.build().map_err(|e| e.to_string())?)
        .await
        .map_err(|e| browser_launch_error_message(&e.to_string(), mode))?;
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
    let temporary_profile = match mode {
        BrowserProfileMode::Persistent => None,
        BrowserProfileMode::Temporary { dir } => Some(TemporaryProfileGuard { dir: dir.clone() }),
    };
    Ok(BrowserSession {
        browser,
        page,
        _temporary_profile: temporary_profile,
    })
}

/// Login → draft list → enter editor → scroll to settings area.
///
/// This is the main entry point for all WeChat editor automation. It restores
/// the saved session if possible, navigates to the draft list, and opens the
/// requested draft editor when a title is supplied.
pub async fn setup_editor(
    headed: bool,
    mode: &BrowserProfileMode,
) -> Result<BrowserSession, String> {
    setup_editor_for_title(headed, mode, None).await
}

/// Open the editor for a named draft, or retain the legacy first-draft behavior.
pub async fn setup_editor_for_title(
    headed: bool,
    mode: &BrowserProfileMode,
    draft_title: Option<&str>,
) -> Result<BrowserSession, String> {
    setup_editor_for_draft(headed, mode, None, draft_title).await
}

/// Open the editor by appmsgid when available, falling back to title selection.
pub async fn setup_editor_for_draft(
    headed: bool,
    mode: &BrowserProfileMode,
    appmsgid: Option<&str>,
    draft_title: Option<&str>,
) -> Result<BrowserSession, String> {
    let BrowserSession {
        browser,
        page,
        _temporary_profile,
    } = open_browser(!headed, mode).await?;

    if try_restore_session(&browser, &page, mode).await {
        println!("  ✅ Session 已恢复，无需扫码");
    } else {
        if !headed {
            return Err(headless_login_required_message(mode));
        }
        println!("▶ 请扫描二维码登录...");
        page.goto("https://mp.weixin.qq.com")
            .await
            .map_err(|e| format!("nav: {e}"))?;
        let login_url = wait_url(&page, "cgi-bin/home").await;
        if login_url.is_empty() {
            return Err("login timeout: QR code not scanned within 120s".into());
        }
        save_session(&browser, mode).await;
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

    if let Some(appmsgid) =
        appmsgid.filter(|id| !id.is_empty() && id.chars().all(|c| c.is_ascii_digit()))
    {
        println!("▶ Opening editor by appmsgid...");
        let edit_url = editor_url(token, appmsgid);
        page.goto(&edit_url)
            .await
            .map_err(|e| format!("editor: {e}"))?;
        let current = page.url().await.unwrap_or(None).unwrap_or_default();
        if !current.contains("appmsg_edit") || !current.contains(&format!("appmsgid={appmsgid}")) {
            return Err("editor page did not open for the requested appmsgid".into());
        }
        println!("  ✅ In editor");
        settle_editor_page(&page).await;
        return Ok(BrowserSession {
            browser,
            page,
            _temporary_profile,
        });
    }

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
    let target_title = draft_title.map(js_str).unwrap_or_else(|| "null".to_owned());
    // Whitespace must be normalized on both sides before matching: WeChat renders
    // card titles with non-breaking/multiple spaces (e.g. "ship 自动化验证" shows
    // up with   in innerText), so a raw includes() misses drafts whose titles
    // contain ASCII spaces.
    let selection_script = r#"(() => {
                var norm = function(s) { return (s || '').replace(/\s+/g, ' ').trim(); };
                var targetTitle = __MOONPUB_DRAFT_TITLE__;
                var normTitle = norm(targetTitle);
                var targetDate = targetTitle && targetTitle.match(/\d{4}-\d{2}-\d{2}/);
                var btns = document.querySelectorAll('.weui-desktop-card__action a.weui-desktop-icon-btn');
                if (targetTitle) {
                    var cards = document.querySelectorAll('.weui-desktop-card');
                    for (var c = 0; c < cards.length; c++) {
                        var card = cards[c];
                        var cardText = norm(card.innerText);
                        if (!cardText.includes(normTitle) && !(targetDate && cardText.includes(targetDate[0]))) continue;
                        var cardBtns = card.querySelectorAll('.weui-desktop-card__action a.weui-desktop-icon-btn');
                        if (cardBtns.length === 0) return JSON.stringify({found: false, reason: 'matching draft has no edit button'});
                        var cardBtn = cardBtns.length >= 2 ? cardBtns[1] : cardBtns[0];
                        cardBtn.scrollIntoView({block: 'center'});
                        var cardRect = cardBtn.getBoundingClientRect();
                        return JSON.stringify({found: true, x: cardRect.x + cardRect.width/2, y: cardRect.y + cardRect.height/2, idx: c, count: cards.length, title: targetTitle});
                    }
                    return JSON.stringify({found: false, reason: 'draft title not found', title: targetTitle, count: cards.length, cards: Array.from(cards).map(function(card) { return card.innerText.trim().replace(/\s+/g, ' ').slice(0, 160); })});
                }
                var idx = btns.length >= 2 ? 1 : 0;
                if (btns.length === 0) return JSON.stringify({found: false, count: 0});
                var btn = btns[idx];
                btn.scrollIntoView({block: 'center'});
                var r = btn.getBoundingClientRect();
                return JSON.stringify({found: true, x: r.x + r.width/2, y: r.y + r.height/2, idx: idx, count: btns.length});
            })()"#
        .replace("__MOONPUB_DRAFT_TITLE__", &target_title);
    let rect_json: String = page
        .evaluate(selection_script)
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
        let selected_title = v["title"].as_str().unwrap_or("first draft");
        println!("  click {selected_title} btn[{idx}] of {count} at ({x:.0},{y:.0})");
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
    settle_editor_page(&page).await;
    Ok(BrowserSession {
        browser,
        page,
        _temporary_profile,
    })
}

fn editor_url(token: &str, appmsgid: &str) -> String {
    format!(
        "https://mp.weixin.qq.com/cgi-bin/appmsg?t=media/appmsg_edit&action=edit&type=77&appmsgid={appmsgid}&isMul=1&replaceScene=0&isSend=0&isFreePublish=0&token={token}&lang=zh_CN"
    )
}

async fn settle_editor_page(page: &Page) {
    sleep_ms(3_000).await;
    let _ = page
        .evaluate("window.scrollTo(0, document.body.scrollHeight)")
        .await;
    sleep_ms(2_000).await;
    let _ = page
        .evaluate("window.scrollTo(0, document.body.scrollHeight - 500)")
        .await;
    sleep_ms(1_000).await;
}

#[cfg(test)]
mod tests {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };

    use super::{
        BrowserProfileMode, browser_launch_error_message, editor_url, js_str, profile_dir_for,
        sanitize_wechat_url, session_file_for, with_retained_resource,
    };

    struct DropSpy {
        dropped: Arc<AtomicBool>,
    }

    impl Drop for DropSpy {
        fn drop(&mut self) {
            self.dropped.store(true, Ordering::SeqCst);
        }
    }

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
    fn editor_url_targets_numeric_appmsgid_directly() {
        let url = editor_url("token123", "100011067");

        assert!(url.contains("t=media/appmsg_edit"));
        assert!(url.contains("appmsgid=100011067"));
        assert!(url.contains("token=token123"));
    }

    #[test]
    fn retained_resource_stays_alive_until_async_work_finishes() {
        let dropped = Arc::new(AtomicBool::new(false));
        let inside = Arc::clone(&dropped);
        let outside = Arc::clone(&dropped);

        let spy = DropSpy {
            dropped: Arc::clone(&dropped),
        };

        let result = tokio::runtime::Runtime::new()
            .unwrap()
            .block_on(with_retained_resource(spy, |_| {
                Box::pin(async move {
                    assert!(
                        !inside.load(Ordering::SeqCst),
                        "resource was dropped before async work finished"
                    );
                    "done"
                })
            }));

        assert_eq!(result, "done");
        assert!(
            outside.load(Ordering::SeqCst),
            "resource should be dropped after async work completes"
        );
    }

    #[test]
    fn persistent_profile_uses_config_directory() {
        let mode = BrowserProfileMode::persistent();
        let profile = profile_dir_for(&mode);
        let session = session_file_for(&mode);

        assert!(profile.to_string_lossy().contains(".config/moonpub"));
        assert_eq!(session, Some(super::session_file()));
    }

    #[test]
    fn temporary_profile_uses_temp_directory_and_no_session_file() {
        let mode = BrowserProfileMode::temporary();
        let profile = profile_dir_for(&mode);

        assert!(profile.starts_with(std::env::temp_dir()));
        assert_eq!(session_file_for(&mode), None);
    }

    #[test]
    fn sanitize_wechat_url_removes_query_token() {
        let url = "https://mp.weixin.qq.com/cgi-bin/home?t=home/index&token=secret";

        assert_eq!(
            sanitize_wechat_url(url),
            "https://mp.weixin.qq.com/cgi-bin/home"
        );
    }

    #[test]
    fn browser_launch_error_explains_persistent_profile_lock() {
        let message = browser_launch_error_message(
            "Failed to create chrome-profile/SingletonLock: File exists. Failed to create a ProcessSingleton",
            &BrowserProfileMode::persistent(),
        );

        assert!(message.contains("persistent Chrome profile is already in use"));
        assert!(message.contains("--temporary-profile"));
    }

    #[test]
    fn browser_launch_error_keeps_generic_temporary_profile_error() {
        let mode = BrowserProfileMode::temporary();
        let message = browser_launch_error_message("Failed to create SingletonLock", &mode);

        assert_eq!(message, "launch: Failed to create SingletonLock");
    }

    #[test]
    fn headless_login_required_message_points_persistent_users_to_saved_session_refresh() {
        let message = super::headless_login_required_message(&BrowserProfileMode::persistent());

        assert!(message.contains("headless mode"));
        assert!(message.contains("moonpub login"));
        assert!(message.contains("configure --headed"));
    }

    #[test]
    fn headless_login_required_message_explains_temporary_profile_tradeoff() {
        let mode = BrowserProfileMode::temporary();
        let message = super::headless_login_required_message(&mode);

        assert!(message.contains("temporary profile"));
        assert!(message.contains("--headed"));
        assert!(message.contains("remove --temporary-profile"));
    }
}
