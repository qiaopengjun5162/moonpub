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
    BrowserProfileMode, ask_ok, check_wechat_health, login_timeout_message, login_wait_secs,
    open_browser, readline, retry_click, run, save_session, setup_editor, setup_editor_for_draft,
    setup_editor_for_title, shot, sleep_ms, wait_enter, wait_url, with_retained_resource,
};
use crate::protocol::{wechat_health_json, wechat_health_text};
use crate::publish_steps::{
    step_baocun, step_chuangzuo, step_fucha, step_liuyan, step_moban, step_yuanzhuang, step_yulan,
    step_zanshang,
};

// ── Step name constants ──────────────────────────────────────────────────────
const STEP_YUANZHUANG: &str = "yuanzhuang";
const STEP_ZANSHANG: &str = "zanshang";
const STEP_LIUYAN: &str = "liuyan";
const STEP_CHUANGZUO: &str = "chuangzuo";
const STEP_MOBAN: &str = "moban";
const STEP_YULAN: &str = "yulan";

/// Whether a token is a `configure <step>..` name that `run_step` will match.
///
/// This is the single source of truth shared with the CLI parser: `configure`
/// validates positional tokens through here so a typo (`zanshangg`) is rejected
/// instead of being collected as a step that never runs while `configure`
/// still reports success and saves the draft. `configure_plan` and this
/// validator match on the same constants, so adding/renaming a step cannot
/// drift the two.
pub fn is_valid_configure_step(name: &str) -> bool {
    matches!(
        name,
        STEP_YUANZHUANG | STEP_ZANSHANG | STEP_LIUYAN | STEP_CHUANGZUO | STEP_MOBAN | STEP_YULAN
    )
}

/// `configure` 流水线里的一步。`configure_plan()` 返回的向量顺序就是执行顺序。
///
/// 把「跑哪些步骤」从 `if` 嵌套里拎出来变成数据，是为了让这段编排能被断言：
/// 以前这段分支只有真 Chrome + 真草稿才能走一遍，「保存和复查是无条件的」
/// 「空步骤 = 全跑」这类约定因此只能靠人记。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigureAction<'a> {
    Yuanzhuang,
    Zanshang,
    /// 合集 step 故意保持禁用：微信这套 UI 没复核之前不加自动化。
    CollectionSkipped,
    Liuyan,
    Chuangzuo,
    Moban(&'a str),
    MobanSkippedNoTemplate,
    Baocun,
    Fucha,
    Yulan,
}

/// `configure [<step>..]` 的选择语义：不给步骤名 = 全跑。
fn step_enabled(steps: &[String], name: &str) -> bool {
    steps.is_empty() || steps.iter().any(|step| step == name)
}

/// 把「用户点了哪些步骤」翻译成一条确定的执行序列。
///
/// 两条不变量写在函数体里而不是注释里：① `baocun` / `fucha` **无条件**执行，
/// 前面任何一步被跳过，草稿也得落库；② 合集步骤即使被点名也不跑（它不在
/// `is_valid_configure_step` 的枚举里，CLI 也会先拦掉）。
fn configure_plan<'a>(
    steps: &[String],
    template_name: Option<&'a str>,
) -> Vec<ConfigureAction<'a>> {
    let mut plan = Vec::new();
    if step_enabled(steps, STEP_YUANZHUANG) {
        plan.push(ConfigureAction::Yuanzhuang);
    }
    if step_enabled(steps, STEP_ZANSHANG) {
        plan.push(ConfigureAction::Zanshang);
    }
    plan.push(ConfigureAction::CollectionSkipped);
    if step_enabled(steps, STEP_LIUYAN) {
        plan.push(ConfigureAction::Liuyan);
    }
    if step_enabled(steps, STEP_CHUANGZUO) {
        plan.push(ConfigureAction::Chuangzuo);
    }
    if step_enabled(steps, STEP_MOBAN) {
        match template_name {
            Some(name) => plan.push(ConfigureAction::Moban(name)),
            None => plan.push(ConfigureAction::MobanSkippedNoTemplate),
        }
    }
    plan.push(ConfigureAction::Baocun);
    plan.push(ConfigureAction::Fucha);
    if step_enabled(steps, STEP_YULAN) {
        plan.push(ConfigureAction::Yulan);
    }
    plan
}

/// Truncates a browser URL for the step log at a UTF-8 character boundary.
///
/// CDP `page.url()` is not guaranteed to be ASCII-only (query strings, titles
/// in the path), so slicing at a fixed byte offset like `&url[..80]` can panic
/// with `byte index 80 is not a char boundary`. `char_indices().nth(80)` is
/// always a valid boundary. The crate's MSRV (1.88) predates the stable
/// `floor_char_boundary` API (1.91), hence the explicit iterator.
fn log_url_preview(url: &str) -> &str {
    match url.char_indices().nth(80) {
        Some((byte_idx, _)) => &url[..byte_idx],
        None => url,
    }
}

/// Escape a string for use as an XPath string literal.
/// WeChat preview recipient names rarely contain quotes; this handles the
/// common single-quote case and falls back to double quotes when needed.
fn xpath_literal(s: &str) -> String {
    if !s.contains('\'') {
        format!("'{s}'")
    } else if !s.contains('"') {
        format!("\"{s}\"")
    } else {
        // Both quote types present: build concat('a', "'", 'b"c').
        let parts: Vec<String> = s.split('\'').map(|part| format!("'{part}'")).collect();
        parts.join(", \"'\", ")
    }
}

fn browser_profile_mode(temporary_profile: bool) -> BrowserProfileMode {
    BrowserProfileMode::from_temporary_flag(temporary_profile)
}

/// 从 `cgi-bin/home?token=…` 之类的 URL 里取 token。
///
/// `step-test` 用它决定接下来打开哪个草稿列表页。取不到时返回空串而不是
/// panic：URL 里没有 `token=`、`token=` 后没有 `&`、`token=` 后面直接是空值，
/// 三种都要能安全落到「空 token」这一条路上。
fn token_from_home_url(url: &str) -> &str {
    url.split("token=")
        .nth(1)
        .and_then(|rest| rest.split('&').next())
        .unwrap_or("")
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
                println!(
                    "Scan QR once. Waiting up to {}s for WeChat backend login...",
                    login_wait_secs()
                );
                let login_url = wait_url(&page, "cgi-bin/home").await;
                if login_url.is_empty() {
                    return Err(login_timeout_message(login_wait_secs()));
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
    mid: &str,
    _collection: &str,
    steps: &[String],
    headed: bool,
    temporary_profile: bool,
    template_name: Option<&str>,
    evidence_dir: Option<&Path>,
    draft_title: Option<&str>,
) -> Result<ConfigureReport, String> {
    let steps = steps.to_vec();
    let evidence_dir = evidence_dir.map(|path| path.to_path_buf());
    run(async move {
        let mode = browser_profile_mode(temporary_profile);
        let session = setup_editor_for_draft(headed, &mode, Some(mid), draft_title).await?;
        let browser = session.browser;
        let page = session.page;
        if let Some(dir) = &evidence_dir {
            std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
            shot(&page, &dir.join("wechat-draft-created.png")).await;
        }

        let report = run_configure_plan(
            &page,
            evidence_dir.as_deref(),
            &steps,
            template_name,
            draft_title,
        )
        .await;

        if headed {
            println!("Done! Press Enter to close...");
            readline();
        }
        drop(browser);
        Ok(report)
    })
}

/// 一次后台配置的结果。`fucha` 是复核那一步的结局，`None` 表示这次没跑复核。
///
/// 为什么要把结论传出来：`configure` 只负责执行，`push` / `ship` 打印的最后一行
/// 是「✓ done」。2026-11-25 实测 `push` 的复核四项全部报「⚠ 未保存到后台」，
/// 紧跟着依然打「✓ done」——同一个终端里自相矛盾，用户只会记住最后一个 ✅。
/// 复核失败的分两种：真没保存（要去后台补），或复核页没读到（跑一次
/// `moonpub configure` 就能确认）。两种都不该被一句 `✓ done` 盖掉。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigureReport {
    pub fucha: Option<crate::publish_steps::StepOutcome>,
}

impl ConfigureReport {
    /// 复核跑过且没有「未验证」结局，才算真的确认落库。
    pub fn is_verified(&self) -> bool {
        !matches!(
            self.fucha,
            Some(crate::publish_steps::StepOutcome::Unverified(_))
        )
    }

    /// 收尾那一行该说的话。已验证就只是 `done`；未验证要说清两种可能和下一步。
    pub fn summary(&self) -> String {
        if self.is_verified() {
            return "done".to_owned();
        }
        let reason = match self.fucha {
            Some(crate::publish_steps::StepOutcome::Unverified(code)) => code,
            _ => "fucha-some-missing",
        };
        // 「重载跑错篇」不是「没保存」：这一句要说得比通用文案更准，
        // 否则用户会跑去后台手工补四项设置，白做一遍。
        if reason == "fucha-reloaded-different-draft" {
            return format!(
                "done, but backend settings were not verified ({reason})\n  \
                 ⚠ 保存后重载的编辑器不是刚编辑的那篇（微信可能换了 appmsgid），四项无法判断\n  \
                 next: moonpub configure   # 它从草稿列表重开这篇再读，读得到就说明已经落库"
            );
        }
        // 「设置区还没渲染出来」也不是「没保存」：页面自己没画完，读到的空串没有意义。
        // 2026-11-26 实测这种读法会把已经落库的四项谎报成「未保存」。
        if reason == "fucha-settings-not-rendered" {
            return format!(
                "done, but backend settings were not verified ({reason})\n  \
                 ⚠ 复核页面没渲染出设置区（等了 14 秒），这不代表没保存，只是没法读\n  \
                 next: moonpub configure   # 重开这篇再读一次，读得到就说明已经落库"
            );
        }
        format!(
            "done, but backend settings were not verified ({reason})\n  \
             ⚠ 原创 / 赞赏 / 留言 / 创作来源 四项未能复核：可能没保存成功，也可能只是复核页没读到\n  \
             next: moonpub configure   # 重做这四步并复核，它会直接告诉你结果"
        )
    }
}

/// 按 `configure_plan()` 给出的顺序逐步操作编辑器。
///
/// 步骤函数本身是软失败（返回 `StepOutcome`，不中断），这里只负责顺序与
/// 证据截图：`configure-headed.png` 在保存+复查之后，`preview-sent.png` 在
/// 预览之后——截图位置跟以前一致，便于和旧日志逐张对照。
///
/// 返回值只关心**复核**那一步：它是唯一能证明「后台真的落库了」的环节，
/// 其余步骤的结局不改变调用方的收尾文案。
async fn run_configure_plan(
    page: &Page,
    evidence_dir: Option<&Path>,
    steps: &[String],
    template_name: Option<&str>,
    draft_title: Option<&str>,
) -> ConfigureReport {
    let mut report = ConfigureReport { fucha: None };
    for action in configure_plan(steps, template_name) {
        match action {
            ConfigureAction::Yuanzhuang => {
                step_yuanzhuang(page).await;
            }
            ConfigureAction::Zanshang => {
                step_zanshang(page).await;
            }
            ConfigureAction::CollectionSkipped => println!("▶ 合集... (skipped)"),
            ConfigureAction::Liuyan => {
                step_liuyan(page).await;
            }
            ConfigureAction::Chuangzuo => {
                step_chuangzuo(page).await;
            }
            ConfigureAction::Moban(name) => {
                step_moban(page, name).await;
            }
            ConfigureAction::MobanSkippedNoTemplate => {
                println!("▶ 模板插入... (skipped: [template].name not set)");
            }
            ConfigureAction::Baocun => {
                step_baocun(page).await;
            }
            ConfigureAction::Fucha => {
                report.fucha = Some(step_fucha(page, draft_title).await);
                if let Some(dir) = evidence_dir {
                    shot(page, &dir.join("configure-headed.png")).await;
                }
            }
            ConfigureAction::Yulan => {
                // Preview-send is part of the default configure flow. The WeChat backend
                // preview endpoint needs a preusername_list; step_yulan resolves the
                // recipient as --to > WECHAT_PREVIEW_TO > .moonpub/preview_to > page
                // auto-detect. If nothing resolves, it prints a one-time setup hint
                // and returns without failing the whole run.
                step_yulan(page, None).await;
                if let Some(dir) = evidence_dir {
                    shot(page, &dir.join("preview-sent.png")).await;
                }
            }
        }
    }
    report
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
        let token = token_from_home_url(&url);
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
        println!("  编辑器: {}", log_url_preview(&cur));
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
        let profile_name = std::env::var("WECHAT_PREVIEW_TO")
            .ok()
            .filter(|s| !s.trim().is_empty())
            .or_else(|| {
                let path = std::path::Path::new(".moonpub").join("preview_to");
                std::fs::read_to_string(&path)
                    .ok()
                    .map(|s| s.trim().to_owned())
                    .filter(|s| !s.is_empty())
            })
            .unwrap_or_else(|| "寻月隐君".to_owned());
        println!("\n══ Step {s}b: 搜索并选中「{profile_name}」 ══");
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
            crate::cdp::js_str(&profile_name)
        )).await.ok().and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned()))).unwrap_or_default();
        println!("    搜索: {typed}");
        sleep_ms(3_000).await;
        let ok_card = retry_click(
            &page,
            &[
                &format!(
                    "//div[contains(@class, 'wx_profile_card') and .//em[contains(text(), {})]]",
                    xpath_literal(&profile_name)
                ),
                &format!(
                    "//div[contains(@class, 'wx_profile_card') and contains(., {})]",
                    xpath_literal(&profile_name)
                ),
            ],
            8,
            400,
        )
        .await;
        println!("    选中卡片: {ok_card}");
        sleep_ms(1_000).await;
        shot(&page, &dir.join(format!("step{s:02}b.png"))).await;
        if !ask_ok(&format!("选中{profile_name}了？(应有绿色边框)")) {
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
        dump_zanshang_dom(&page, "before").await;
        step_zanshang(&page).await;
        dump_zanshang_dom(&page, "after").await;
        if headed {
            println!("\n── 赞赏测试完成，按 Enter 关闭浏览器...");
            sleep_ms(3_000).await;
            readline();
        }
        drop(browser);
        Ok("done".to_owned())
    })
}

/// Diagnostic: dump the 赞赏 settings row and any open dialog's switch DOM so
/// selector changes are based on the real WeChat DOM, not guesses.
async fn dump_zanshang_dom(page: &Page, tag: &str) {
    let dump = page
        .evaluate(
            r#"(function(){
        var out = {row: null, dialogs: []};
        var tips = document.querySelector('.js_reward_setting_tips');
        if (tips) {
            var row = tips.closest('div');
            out.row = row ? row.outerHTML.slice(0, 1200) : tips.outerHTML.slice(0, 1200);
        }
        var dialogs = document.querySelectorAll('.weui-desktop-dialog, [class*="dialog"], [class*="popover"]');
        for (var i = 0; i < dialogs.length; i++) {
            var d = dialogs[i];
            if (d.offsetParent === null) continue;
            var html = d.outerHTML;
            if (html.indexOf('赞赏') < 0 && html.indexOf('reward') < 0) continue;
            out.dialogs.push(html.slice(0, 2000));
        }
        return JSON.stringify(out);
    })()"#,
        )
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();
    println!("── zanshang DOM ({tag}): {dump}");
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

    use super::{
        ConfigureAction, ConfigureReport, browser_profile_mode, configure_plan,
        is_valid_configure_step, log_url_preview, step_enabled, token_from_home_url, xpath_literal,
    };

    fn steps(names: &[&str]) -> Vec<String> {
        names.iter().map(|name| (*name).to_owned()).collect()
    }

    #[test]
    fn step_test_uses_temporary_profile_mode_when_requested() {
        assert!(matches!(
            browser_profile_mode(true),
            BrowserProfileMode::Temporary { .. }
        ));
    }

    // ── configure 执行序列 ──────────────────────────────────────────────────

    #[test]
    fn an_empty_step_list_means_run_everything() {
        // 裸跑 `moonpub configure` 是默认路径：不点名 = 全跑，含预览。
        assert_eq!(
            configure_plan(&[], Some("模板A")),
            vec![
                ConfigureAction::Yuanzhuang,
                ConfigureAction::Zanshang,
                ConfigureAction::CollectionSkipped,
                ConfigureAction::Liuyan,
                ConfigureAction::Chuangzuo,
                ConfigureAction::Moban("模板A"),
                ConfigureAction::Baocun,
                ConfigureAction::Fucha,
                ConfigureAction::Yulan,
            ]
        );
        assert!(step_enabled(&[], "anything"));
    }

    #[test]
    fn naming_one_step_keeps_saves_and_verify_unconditional() {
        // 保存和复查不是「步骤」，它们无条件执行：否则 `configure zanshang`
        // 会把改动留在 DOM 里不落库，用户看到的却是命令成功。
        assert_eq!(
            configure_plan(&steps(&["zanshang"]), Some("模板A")),
            vec![
                ConfigureAction::Zanshang,
                ConfigureAction::CollectionSkipped,
                ConfigureAction::Baocun,
                ConfigureAction::Fucha,
            ]
        );
    }

    #[test]
    fn template_step_without_a_template_name_says_so_instead_of_vanishing() {
        // `[template].name` 没配时以前会静默跳过，日志里看不出「模板没插」。
        assert_eq!(
            configure_plan(&steps(&["moban"]), None),
            vec![
                ConfigureAction::CollectionSkipped,
                ConfigureAction::MobanSkippedNoTemplate,
                ConfigureAction::Baocun,
                ConfigureAction::Fucha,
            ]
        );
    }

    #[test]
    fn preview_runs_last_when_it_is_asked_for() {
        assert_eq!(
            configure_plan(&steps(&["yulan"]), None),
            vec![
                ConfigureAction::CollectionSkipped,
                ConfigureAction::Baocun,
                ConfigureAction::Fucha,
                ConfigureAction::Yulan,
            ]
        );
    }

    #[test]
    fn the_collection_step_is_always_reported_but_never_automated() {
        // 合集自动化被刻意关掉（微信 UI 未复核）。它必须以「跳过」出现在
        // 计划里，否则用户会以为是被自己的参数过滤掉的。
        for plan in [
            configure_plan(&[], None),
            configure_plan(&steps(&["yuanzhuang"]), None),
        ] {
            assert_eq!(
                plan.iter()
                    .filter(|action| **action == ConfigureAction::CollectionSkipped)
                    .count(),
                1
            );
        }
    }

    #[test]
    fn a_misspelled_step_name_runs_nothing_extra() {
        // CLI 已经拦掉拼错的步骤名；这一层再锁一次，防止将来有人放宽校验后
        // 「拼错 = 什么都不做，但命令照样报成功」重新出现。
        assert!(!is_valid_configure_step("zanshangg"));
        assert_eq!(
            configure_plan(&steps(&["zanshangg"]), None),
            vec![
                ConfigureAction::CollectionSkipped,
                ConfigureAction::Baocun,
                ConfigureAction::Fucha,
            ]
        );
    }

    #[test]
    fn every_name_the_cli_accepts_actually_reaches_a_step() {
        // `is_valid_configure_step` 与 `configure_plan` 共用同一组 STEP_* 常量。
        // 只改一处时这条会红：校验器放行的名字，计划里必须真有一个动作。
        const ALWAYS_ON: [ConfigureAction<'static>; 3] = [
            ConfigureAction::CollectionSkipped,
            ConfigureAction::Baocun,
            ConfigureAction::Fucha,
        ];
        for name in [
            "yuanzhuang",
            "zanshang",
            "liuyan",
            "chuangzuo",
            "moban",
            "yulan",
        ] {
            assert!(is_valid_configure_step(name), "{name} should be valid");
            let plan = configure_plan(&steps(&[name]), Some("模板A"));
            assert!(
                plan.iter().any(|action| !ALWAYS_ON.contains(action)),
                "{name} produced no step: {plan:?}"
            );
        }
    }

    // ── 首页 URL 里的 token ────────────────────────────────────────────────

    #[test]
    fn token_is_taken_from_the_middle_of_a_home_url() {
        assert_eq!(
            token_from_home_url(
                "https://mp.weixin.qq.com/cgi-bin/home?t=home/index&token=123456&lang=zh_CN"
            ),
            "123456"
        );
    }

    #[test]
    fn token_at_the_end_of_the_url_still_works() {
        assert_eq!(
            token_from_home_url("https://mp.weixin.qq.com/cgi-bin/home?token=123456"),
            "123456"
        );
    }

    #[test]
    fn a_url_without_a_token_yields_an_empty_string_instead_of_panicking() {
        assert_eq!(token_from_home_url("https://mp.weixin.qq.com/"), "");
        assert_eq!(token_from_home_url(""), "");
    }

    #[test]
    fn an_empty_token_value_stays_empty() {
        assert_eq!(
            token_from_home_url("https://mp.weixin.qq.com/cgi-bin/home?token=&lang=zh_CN"),
            ""
        );
    }

    #[test]
    fn xpath_literal_uses_single_quotes_for_plain_names() {
        assert_eq!(xpath_literal("寻月隐君"), "'寻月隐君'");
    }

    #[test]
    fn xpath_literal_switches_to_double_quotes_when_name_has_apostrophe() {
        assert_eq!(xpath_literal("O'Neil"), "\"O'Neil\"");
    }

    #[test]
    fn xpath_literal_concatenates_names_with_both_quote_types() {
        // XPath has no escape character: `a'b"c` must become
        // concat-style `'a', "'", 'b"c'`.
        assert_eq!(xpath_literal("a'b\"c"), "'a', \"'\", 'b\"c'");
    }

    #[test]
    fn xpath_literal_handles_lone_quote_characters() {
        assert_eq!(xpath_literal("'"), "\"'\"");
        assert_eq!(xpath_literal("\""), "'\"'");
    }

    #[test]
    fn xpath_literal_keeps_empty_name_quotable() {
        assert_eq!(xpath_literal(""), "''");
    }

    #[test]
    fn log_url_preview_truncates_long_ascii_urls() {
        let url = format!("https://mp.weixin.qq.com/{}", "a".repeat(120));
        let preview = log_url_preview(&url);
        assert_eq!(preview.chars().count(), 80);
        assert!(url.starts_with(preview));
    }

    #[test]
    fn log_url_preview_never_splits_a_multibyte_character() {
        // 27 Chinese chars = 81 bytes; the old `&url[..80]` would panic
        // because byte 80 lands inside the last 3-byte character.
        let url = "一二三四五六七八九".repeat(3);
        let preview = log_url_preview(&url);
        assert!(url.starts_with(preview));
        assert!(preview.chars().count() <= 80);
        // Every returned slice is a valid &str, i.e. ends on a char boundary.
        assert!(preview.is_char_boundary(preview.len()));
    }

    #[test]
    fn verified_configure_report_says_plain_done() {
        for fucha in [
            None,
            Some(crate::publish_steps::StepOutcome::Applied(
                "fucha-all-persisted",
            )),
            Some(crate::publish_steps::StepOutcome::AlreadyDone(
                "fucha-all-persisted",
            )),
        ] {
            let report = ConfigureReport { fucha };
            assert!(report.is_verified(), "{fucha:?}");
            assert_eq!(report.summary(), "done");
        }
    }

    /// 2026-11-25 实测：`push` 的复核四项全部报「⚠ 未保存到后台」，紧跟着照样打
    /// `✓ done`——同一个终端里自相矛盾，用户只会记住最后那个对勾。复核没过时
    /// 收尾那一行必须说清「有两种可能」并给出唯一确定的下一步。
    #[test]
    fn unverified_configure_report_never_reads_as_plain_success() {
        let report = ConfigureReport {
            fucha: Some(crate::publish_steps::StepOutcome::Unverified(
                "fucha-some-missing",
            )),
        };
        assert!(!report.is_verified());

        let summary = report.summary();
        assert!(summary.starts_with("done, but"), "{summary}");
        // 四项要一件件数出来，用户才知道该去后台看什么。
        for setting in ["原创", "赞赏", "留言", "创作来源"] {
            assert!(summary.contains(setting), "缺 `{setting}`：{summary}");
        }
        // 两种可能都要说：可能是真没保存，也可能只是复核页没读到。
        assert!(summary.contains("可能没保存成功"), "{summary}");
        assert!(summary.contains("复核页没读到"), "{summary}");
        // 并且必须给一条能直接跑的命令，而不是「请人工核对」。
        assert!(summary.contains("moonpub configure"), "{summary}");
    }

    /// 页面没画完 ≠ 没保存。2026-11-26 实测：`push` 报四项 ⚠，同一篇草稿紧接着
    /// `configure` 四项全绿——后台早存上了，是复核读得太早。文案必须把这件事说破，
    /// 否则用户会跑去后台手工补一遍设置。
    #[test]
    fn unrendered_settings_panel_is_not_reported_as_unpersisted() {
        let report = ConfigureReport {
            fucha: Some(crate::publish_steps::StepOutcome::Unverified(
                "fucha-settings-not-rendered",
            )),
        };
        let summary = report.summary();
        assert!(summary.starts_with("done, but"), "{summary}");
        assert!(summary.contains("fucha-settings-not-rendered"), "{summary}");
        assert!(summary.contains("不代表没保存"), "{summary}");
        // 不许复用「可能没保存成功」那句——它会把读者推去做无用的返工。
        assert!(!summary.contains("可能没保存成功"), "{summary}");
        assert!(summary.contains("moonpub configure"), "{summary}");
    }

    /// 「重载跑错篇」是另一类未验证：四项不是「没存上」，而是根本没读到。
    /// 收尾文案不能让人去后台白补一遍设置。
    #[test]
    fn wrong_draft_diagnosis_points_at_configure_not_at_redoing_the_settings() {
        let report = ConfigureReport {
            fucha: Some(crate::publish_steps::StepOutcome::Unverified(
                "fucha-reloaded-different-draft",
            )),
        };
        assert!(!report.is_verified());
        let summary = report.summary();
        assert!(summary.starts_with("done, but"), "{summary}");
        assert!(summary.contains("不是刚编辑的那篇"), "{summary}");
        assert!(summary.contains("moonpub configure"), "{summary}");
        // 不能沿用通用那句「可能没保存成功」，否则用户会跑去后台重设四项。
        assert!(!summary.contains("可能没保存成功"), "{summary}");
    }

    #[test]
    fn skipped_fucha_counts_as_verified_because_nothing_was_claimed() {
        // 没跑复核（`fucha: None`）等于没有「未验证」的断言，不该报警。
        // 但 Skipped / 其它结局也不该被当成 Unverified——那是另一类语义。
        let report = ConfigureReport {
            fucha: Some(crate::publish_steps::StepOutcome::Skipped("fucha-skipped")),
        };
        assert!(report.is_verified());
    }

    #[test]
    fn log_url_preview_keeps_short_and_multibyte_urls_intact() {
        let short = "https://mp.weixin.qq.com/cgi-bin/appmsg?t=media/appmsg_edit";
        assert_eq!(log_url_preview(short), short);
        let chinese = "https://example.com/搜索?q=微信公众号标题";
        assert_eq!(log_url_preview(chinese), chinese);
    }
}
