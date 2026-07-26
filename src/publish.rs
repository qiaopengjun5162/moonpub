//! Pure Rust CDP automation via chromiumoxide — dedicated profile, no re-login.
//!
//! Key design: all clicks use JS evaluate so XPath selectors work correctly.
//! chromiumoxide's find_element() only accepts CSS — XPath must go through JS.
//!
//! This module only orchestrates the workflow. CDP primitives live in `crate::cdp`,
//! and individual editor steps live in `crate::publish_steps`.

use std::path::Path;

use chromiumoxide::Page;

use crate::cdp::{
    BrowserProfileMode, ask_ok, check_wechat_health, open_browser, readline, retry_click, run,
    save_session, setup_editor, setup_editor_for_title, shot, sleep_ms, wait_enter, wait_url,
    with_retained_resource,
};
use crate::protocol::{wechat_health_json, wechat_health_text};
use crate::publish_steps::{
    step_chuangzuo, step_liuyan, step_moban, step_yuanzhuang, step_yulan, step_zanshang,
};

// ── Step name constants ──────────────────────────────────────────────────────
const STEP_YUANZHUANG: &str = "yuanzhuang";
const STEP_ZANSHANG: &str = "zanshang";
const STEP_LIUYAN: &str = "liuyan";
const STEP_CHUANGZUO: &str = "chuangzuo";
const STEP_MOBAN: &str = "moban";
const STEP_YULAN: &str = "yulan";

fn browser_profile_mode(temporary_profile: bool) -> BrowserProfileMode {
    BrowserProfileMode::from_temporary_flag(temporary_profile)
}

/// Open WeChat MP, wait for QR scan, and keep the browser open.
pub fn login(temporary_profile: bool) -> Result<String, String> {
    run(async {
        let mode = browser_profile_mode(temporary_profile);
        let session = open_browser(false, &mode).await?;
        let page = session.page.clone();
        with_retained_resource(session, |session| {
            Box::pin(async move {
                page.goto("https://mp.weixin.qq.com")
                    .await
                    .map_err(|e| e.to_string())?;
                println!("Scan QR once. Waiting for WeChat backend login...");
                let login_url = wait_url(&page, "cgi-bin/home").await;
                if login_url.is_empty() {
                    return Err("login timeout: QR code not scanned within 120s".into());
                }
                save_session(&session.browser, &mode).await;
                if temporary_profile {
                    println!("Login complete. Temporary browser session will be discarded after this run.");
                } else {
                    println!("Login complete. Browser session saved for later CDP automation.");
                }
                Ok("done".to_owned())
            })
        })
        .await
    })
}

pub fn health(headed: bool, temporary_profile: bool, json: bool) -> Result<String, String> {
    run(async move {
        let mode = browser_profile_mode(temporary_profile);
        let report = check_wechat_health(headed, &mode).await?;
        if json {
            Ok(wechat_health_json(&report))
        } else {
            Ok(wechat_health_text(&report))
        }
    })
}

/// Configure a draft after it has been pushed via the WeChat API.
///
/// Steps are soft-fail: if a button is not found, we print a warning and continue.
/// WeChat's editor is a live web app; UI changes should not break the whole flow.
#[allow(clippy::too_many_arguments)]
pub fn auto_configure(
    _mid: &str,
    _collection: &str,
    steps: &[String],
    headed: bool,
    temporary_profile: bool,
    template_name: Option<&str>,
    evidence_dir: Option<&Path>,
    draft_title: Option<&str>,
) -> Result<String, String> {
    let steps = steps.to_vec();
    let evidence_dir = evidence_dir.map(|path| path.to_path_buf());
    run(async move {
        let run_step = |name: &str| steps.is_empty() || steps.iter().any(|s| s == name);
        let mode = browser_profile_mode(temporary_profile);
        let session = setup_editor_for_title(headed, &mode, draft_title).await?;
        let browser = session.browser;
        let page = session.page;
        if let Some(dir) = &evidence_dir {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            shot(&page, &dir.join("wechat-draft-created.png")).await;
        }

        if run_step(STEP_YUANZHUANG) {
            step_yuanzhuang(&page).await;
        }
        if run_step(STEP_ZANSHANG) {
            step_zanshang(&page).await;
        }
        // 合集 step is intentionally disabled; do not add automation here without
        // re-checking the current WeChat UI.
        println!("▶ 合集... (skipped)");
        if run_step(STEP_LIUYAN) {
            step_liuyan(&page).await;
        }
        if run_step(STEP_CHUANGZUO) {
            step_chuangzuo(&page).await;
        }
        if run_step(STEP_MOBAN) {
            if let Some(name) = template_name {
                step_moban(&page, name).await;
            } else {
                println!("▶ 模板插入... (skipped: [template].name not set)");
            }
        }
        if let Some(dir) = &evidence_dir {
            shot(&page, &dir.join("configure-headed.png")).await;
        }
        if run_step(STEP_YULAN) {
            // Preview-send is part of the default configure flow. The WeChat backend
            // preview endpoint needs a preusername_list; step_yulan resolves the
            // recipient as --to > WECHAT_PREVIEW_TO > .moonpub/preview_to > page
            // auto-detect. If nothing resolves, it prints a one-time setup hint
            // and returns without failing the whole run.
            step_yulan(&page, None).await;
            if let Some(dir) = &evidence_dir {
                shot(&page, &dir.join("preview-sent.png")).await;
            }
        }

        if headed {
            println!("Done! Press Enter to close...");
            readline();
        }
        drop(browser);
        Ok("done".to_owned())
    })
}

/// Interactive end-to-end test of the browser automation path.
///
/// Used to verify that login, draft list, editor entry, and the account-card
/// insertion flow still work after WeChat UI changes.
pub fn step_test(headed: bool, temporary_profile: bool) -> Result<String, String> {
    run(async {
        let mode = browser_profile_mode(temporary_profile);
        let session = open_browser(!headed, &mode).await?;
        let browser = session.browser;
        let page = session.page;
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
        ))
        .await
        .map_err(|e| format!("list: {e}"))?;
        let ok = crate::cdp::wait_css(&page, ".weui-desktop-card__action", 15_000).await;
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
            crate::cdp::js_str("寻月隐君")
        )).await.ok().and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned()))).unwrap_or_default();
        println!("    搜索: {typed}");
        sleep_ms(3_000).await;
        let ok_card = retry_click(
            &page,
            &[
                "//div[contains(@class, 'wx_profile_card') and .//em[contains(text(), '寻月隐君')]]",
                "//div[contains(@class, 'wx_profile_card') and contains(., '寻月隐君')]",
            ],
            8,
            400,
        )
        .await;
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

pub fn test_chuangzuo(headed: bool, temporary_profile: bool) -> Result<String, String> {
    run(async {
        let mode = browser_profile_mode(temporary_profile);
        let session = setup_editor(headed, &mode).await?;
        let browser = session.browser;
        let page = session.page;
        step_yuanzhuang(&page).await;
        step_chuangzuo(&page).await;
        if headed {
            println!("\n── 创作来源测试完成，按 Enter 关闭浏览器...");
            sleep_ms(3_000).await;
            readline();
        }
        drop(browser);
        Ok("done".to_owned())
    })
}

pub fn test_zanshang(headed: bool, temporary_profile: bool) -> Result<String, String> {
    run(async {
        let mode = browser_profile_mode(temporary_profile);
        let session = setup_editor(headed, &mode).await?;
        let browser = session.browser;
        let page = session.page;
        step_yuanzhuang(&page).await;
        step_zanshang(&page).await;
        if headed {
            println!("\n── 赞赏测试完成，按 Enter 关闭浏览器...");
            sleep_ms(3_000).await;
            readline();
        }
        drop(browser);
        Ok("done".to_owned())
    })
}

pub fn test_yulan_for_title(
    headed: bool,
    temporary_profile: bool,
    draft_title: Option<&str>,
    to_wxname: Option<&str>,
) -> Result<String, String> {
    run(async {
        let mode = browser_profile_mode(temporary_profile);
        let session = setup_editor_for_title(headed, &mode, draft_title).await?;
        let browser = session.browser;
        let page = session.page;
        step_yuanzhuang(&page).await;
        // Recipient resolution is handled inside step_yulan:
        // --to > WECHAT_PREVIEW_TO > auto-detect from the editor page.
        // Passing None here keeps test-yulan usable with no args (like the
        // manual preview flow the user already relies on).
        step_yulan(&page, to_wxname).await;
        if headed {
            println!("\n── 预览测试完成，按 Enter 关闭浏览器...");
            sleep_ms(3_000).await;
            readline();
        }
        drop(browser);
        Ok("done".to_owned())
    })
}

#[cfg(test)]
mod tests {
    use crate::cdp::BrowserProfileMode;

    use super::browser_profile_mode;

    #[test]
    fn step_test_uses_temporary_profile_mode_when_requested() {
        assert!(matches!(
            browser_profile_mode(true),
            BrowserProfileMode::Temporary { .. }
        ));
    }
}
