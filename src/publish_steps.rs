//! Individual WeChat editor automation steps.
//!
//! Each function corresponds to one configuration action in the WeChat backend.
//! They are intentionally small and sequential: open dialog, click option, confirm.
//! All CDP primitives come from `crate::cdp`.

use chromiumoxide::Page;

use crate::cdp::{
    cdp_click_css, cdp_click_exact_last, cdp_click_text, check_agreement, close_dialog,
    retry_click, shot, sleep_ms,
};

pub async fn step_yuanzhuang(page: &Page) {
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
        // Vue checkbox needs the actual input element to be clicked; label text click is unreliable.
        let ok2 = check_agreement(page).await;
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

pub async fn step_zanshang(page: &Page) {
    println!("▶ 赞赏...");
    let _ = page
        .evaluate("window.scrollTo(0, document.body.scrollHeight)")
        .await;
    sleep_ms(800).await;
    let ok = cdp_click_exact_last(page, "赞赏").await;
    println!("    click '赞赏': {ok}");
    if !ok {
        println!("  ⚠ 赞赏 trigger not found — skipping");
        return;
    }
    sleep_ms(1_000).await;
    // Check if a dialog opened with the reward toggle
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
    if !dialog_open {
        println!("  ⚠ 赞赏 dialog did not open — skipping");
        return;
    }
    // Use direct JS click to bypass offsetParent visibility check (element may be in a collapsed section).
    // Try multiple selectors because WeChat's reward toggle has changed between a standalone tip
    // element and a weui-switch inside a label.
    let toggled = page
        .evaluate(
            r#"(function(){
        var selectors = ['.js_reward_setting_tips','.js_reward_setting_tips .weui-switch','label[for*="reward"] input','.reward_setting_switch'];
        var search=function(root){
            for(var i=0;i<selectors.length;i++){
                var el=root.querySelector(selectors[i]);
                if(el){el.click();return true;}
            }
            var frames=root.querySelectorAll('iframe');
            for(var f=0;f<frames.length;f++){
                try{var d=frames[f].contentDocument;if(d&&search(d))return true;}catch(e){}
            }
            return false;
        };
        return search(document);
    })()"#,
        )
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_bool()))
        .unwrap_or(false);
    println!("    click toggle: {toggled}");
    sleep_ms(1_500).await;
    // Re-check agreement if needed, then confirm. The agreement checkbox inside
    // the reward dialog is separate from the originality declaration one.
    let _ = check_agreement(page).await;
    sleep_ms(300).await;
    let mut ok3 = cdp_click_css(page, ".weui-desktop-btn_primary").await;
    if !ok3 {
        ok3 = cdp_click_text(page, "确定").await;
    }
    println!("    click '确定': {ok3}");
    sleep_ms(1_500).await;
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
    if !toggled {
        println!("  ⚠ 赞赏 toggle 不可点击 (账号限制或声明原创后才可开启)");
    } else if zs_state.contains("不开启") || zs_state.is_empty() {
        println!("  ⚠ 赞赏 未开启 — toggle 点击后未切换 (state='{zs_state}')");
    } else {
        println!("  ✅ 赞赏 已开启 (state='{zs_state}')");
    }
}

pub async fn step_liuyan(page: &Page) {
    println!("▶ 留言...");
    let ok = retry_click(page, &["//*[text()='留言']"], 8, 400).await;
    println!("    click '留言': {ok}");
    if ok {
        sleep_ms(1_600).await;
        let ok2 = cdp_click_exact_last(page, "确定").await;
        println!("    click '确定': {ok2}");
        let _ = close_dialog(page).await;
        sleep_ms(500).await;
        println!("  ✅");
    } else {
        println!("  ⚠ '留言' not found — skipping");
    }
}

pub async fn step_chuangzuo(page: &Page) {
    println!("▶ 创作来源...");
    let _ = page
        .evaluate("window.scrollTo(0, document.body.scrollHeight)")
        .await;
    sleep_ms(500).await;

    // Open the 创作来源 picker. The live editor DOM shows a checkbox label
    // with a dedicated clickable wrapper .js_claim_source_desc inside it.
    // We must click that exact wrapper; searching for any "未添加" text hits
    // other rows (e.g. 合集) because their labels share the same structure.
    let ok = page
        .evaluate(
            r#"(function(){
        var forceClick = function(el){
            el.scrollIntoView({block:'center'});
            var o = {bubbles:true, cancelable:true, view:window};
            el.dispatchEvent(new MouseEvent('mousedown', o));
            el.dispatchEvent(new MouseEvent('mouseup', o));
            el.click();
            return true;
        };
        var search=function(root){
            // 1. Direct: the unique 创作来源 desc wrapper.
            var wrap=root.querySelector('.js_claim_source_desc');
            if(wrap && wrap.offsetParent!==null) return forceClick(wrap);
            // 2. Inside the label whose primary text is exactly "创作来源".
            var labels=root.querySelectorAll('label');
            for(var i=0;i<labels.length;i++){
                var lbl=labels[i];
                if(lbl.offsetParent===null) continue;
                var main=lbl.querySelector('.lbl_content');
                if(!main) main=lbl;
                var t=main.textContent.trim().replace(/\s+/g,' ');
                if(t==='创作来源' || t.indexOf('创作来源')===0){
                    var w=lbl.querySelector('.js_claim_source_desc, .allow_click_opr');
                    if(w) return forceClick(w);
                    return forceClick(lbl);
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
        .unwrap_or(false);
    println!("    click '创作来源' 未添加: {ok}");
    if !ok {
        println!("  ⚠ '创作来源' picker not found — skipping");
        return;
    }
    sleep_ms(2_000).await;

    // Select option value="4" (个人观点，仅供参考). The picker is a radio
    // group; using the input value is more reliable than text matching.
    let selected = page
        .evaluate(
            r#"(function(){
        var forceClick = function(el){
            el.scrollIntoView({block:'center'});
            var o = {bubbles:true, cancelable:true, view:window};
            el.dispatchEvent(new MouseEvent('mousedown', o));
            el.dispatchEvent(new MouseEvent('mouseup', o));
            el.click();
            return true;
        };
        var search=function(root){
            var radios=root.querySelectorAll('input[type="radio"][value="4"]');
            for(var i=0;i<radios.length;i++){
                var r=radios[i];
                if(r.offsetParent===null) continue;
                forceClick(r);
                return '个人观点，仅供参考';
            }
            var labels=root.querySelectorAll('label');
            for(var j=0;j<labels.length;j++){
                var lbl=labels[j];
                if(lbl.offsetParent===null) continue;
                var txt=lbl.textContent.trim().replace(/\s+/g,' ');
                if(txt.indexOf('个人观点，仅供参考')>=0){
                    var input=lbl.querySelector('input[type="radio"]');
                    if(input) forceClick(input);
                    else forceClick(lbl);
                    return '个人观点，仅供参考';
                }
            }
            return null;
        };
        var r=search(document);
        if(r) return r;
        var frames=document.querySelectorAll('iframe');
        for(var f=0;f<frames.length;f++){try{var d=frames[f].contentDocument;if(d){var r2=search(d);if(r2) return r2;}}catch(e){}}
        return null;
    })()"#,
        )
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();

    if selected.is_empty() {
        let _ = close_dialog(page).await;
        println!("  ⚠ 创作来源 — 未找到可选项，跳过");
        return;
    }
    println!("    select '{selected}': true");
    sleep_ms(500).await;

    // Confirm the selection.
    let mut ok3 = cdp_click_text(page, "确认").await;
    if !ok3 {
        ok3 = cdp_click_text(page, "确定").await;
    }
    if !ok3 {
        ok3 = cdp_click_css(page, ".weui-desktop-dialog__ft .weui-desktop-btn_primary").await;
    }
    if !ok3 {
        ok3 = cdp_click_css(page, ".weui-desktop-btn_primary").await;
    }
    println!("    click '确认': {ok3}");
    sleep_ms(1_500).await;

    // Verify the setting stuck.
    let state = page
        .evaluate(
            r#"(function(){
        var search=function(root){
            var selected=root.querySelector('.js_claim_source_selected');
            if(selected && selected.offsetParent!==null) return selected.textContent.trim();
            var labels=root.querySelectorAll('label');
            for(var i=0;i<labels.length;i++){
                var t=labels[i].textContent.trim().replace(/\s+/g,' ');
                if(t==='创作来源' || t.indexOf('创作来源')===0){
                    var span=labels[i].querySelector('.js_claim_source_selected');
                    if(span) return span.textContent.trim();
                }
            }
            return null;
        };
        var r=search(document);
        if(r) return r;
        var frames=document.querySelectorAll('iframe');
        for(var f=0;f<frames.length;f++){try{var d=frames[f].contentDocument;if(d){var r2=search(d);if(r2) return r2;}}catch(e){}}
        return '(not found)';
    })()"#,
        )
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();
    println!("    创作来源 state: '{state}'");

    if ok3 && state.contains("个人观点") {
        println!("  ✅ 创作来源");
    } else if ok3 {
        println!("  ⚠ 创作来源 已确认但状态未识别 (state='{state}')");
    } else {
        println!("  ⚠ 创作来源 '确认' not found");
    }
}

pub async fn step_yulan(page: &Page) {
    println!("▶ 预览...");
    let _ = page
        .evaluate("window.scrollTo(0, document.body.scrollHeight)")
        .await;
    sleep_ms(500).await;
    let ok = cdp_click_text(page, "预览").await;
    println!("    click '预览': {ok}");
    sleep_ms(2_000).await; // wait for dialog to render
    shot(page, std::path::Path::new("/tmp/yulan-1-dialog.png")).await;
    // Dump visible text to diagnose radio button label
    let diag = page
        .evaluate(
            r#"(() => {
        var out = [];
        var search = function(root) {
            var els = root.querySelectorAll('label, .weui-desktop-form-ctrl__radio, input[type=radio]');
            for (var i = 0; i < els.length; i++) {
                var t = els[i].textContent.trim().replace(/\s+/g,' ');
                if (t) out.push(els[i].tagName + '[' + t.substring(0,40) + ']');
            }
            var all = root.querySelectorAll('*');
            for (var j=0;j<all.length;j++) if(all[j].shadowRoot) search(all[j].shadowRoot);
        };
        search(document);
        var frames = document.querySelectorAll('iframe');
        for (var f=0;f<frames.length;f++){try{var d=frames[f].contentDocument;if(d)search(d);}catch(e){}}
        return out.join(' | ') || '(none)';
    })()"#,
        )
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();
    println!("    [diag radio]: {diag}");

    let ok2 = cdp_click_exact_last(page, "通过公众号列表预览").await;
    println!("    select mode: {ok2}");
    sleep_ms(1_000).await;
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

/// Insert a WeChat article template by name via CDP automation.
///
/// This function focuses the editor, opens the template menu, searches for
/// the given template name, and clicks to insert it. It returns true if
/// the template was found and the add button was clicked.
pub async fn step_moban(page: &Page, template_name: &str) -> bool {
    println!("▶ 模板插入 ({template_name})...");
    let js = moban_script(template_name);
    let ok = page
        .evaluate(js)
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_bool()))
        .unwrap_or(false);
    if ok {
        sleep_ms(1_500).await;
        println!("  ✅ 模板插入");
    } else {
        println!("  ⚠ 模板插入失败或模板未找到 — skipping");
    }
    ok
}

pub async fn step_aicover(page: &Page, prompt: &str) -> bool {
    println!("▶ AI 配图封面...");
    let js = aicover_script(prompt);
    let ok = page
        .evaluate(js)
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_bool()))
        .unwrap_or(false);
    if ok {
        sleep_ms(2_000).await;
        println!("  ✅ AI 配图已触发");
    } else {
        println!("  ⚠ AI 配图入口不可用或生成失败 — skipping");
    }
    ok
}

/// Build the JS string used by `step_moban`.
///
/// Extracted into a helper so unit tests can assert on the generated JS
/// without needing a `Page`.
fn moban_script(template_name: &str) -> String {
    let escaped = crate::cdp::js_str(template_name);
    format!(
        r#"(function(){{
    var templateName = {escaped};
    var forceClick = function(el){{
        el.scrollIntoView({{block:'center'}});
        var o = {{bubbles:true, cancelable:true, view:window}};
        el.dispatchEvent(new MouseEvent('mousedown', o));
        el.dispatchEvent(new MouseEvent('mouseup', o));
        el.click();
        return true;
    }};
    var search = function(root){{
        // 1. Focus the editor and move cursor to end
        var editor = root.querySelector('#js_editor');
        if(editor){{
            editor.focus();
            var sel = root.getSelection ? root.getSelection() : window.getSelection();
            if(sel && sel.rangeCount > 0){{
                var range = sel.getRangeAt(0);
                range.selectNodeContents(editor);
                range.collapse(false);
                sel.removeAllRanges();
                sel.addRange(range);
            }}
        }}
        // 2. Click the "模板" toolbar menu item
        var lis = root.querySelectorAll('li');
        for(var i=0;i<lis.length;i++){{
            var li = lis[i];
            if(li.offsetParent !== null && li.textContent.trim() === '模板'){{
                forceClick(li);
                return true;
            }}
        }}
        return false;
    }};
    if(!search(document)){{
        var frames = document.querySelectorAll('iframe');
        for(var f=0;f<frames.length;f++){{try{{var d=frames[f].contentDocument;if(d&&search(d))return true;}}catch(e){{}}}}
        return false;
    }}
    // 3. Wait briefly for menu to open, then search for template by name
    var found = false;
    var start = Date.now();
    while(Date.now() - start < 3000){{
        var allEls = document.querySelectorAll('*');
        for(var i=0;i<allEls.length;i++){{
            var el = allEls[i];
            if(el.offsetParent !== null && el.textContent.trim() === templateName){{
                forceClick(el);
                found = true;
                break;
            }}
        }}
        if(found) break;
        var frames = document.querySelectorAll('iframe');
        for(var f=0;f<frames.length;f++){{
            try{{
                var d = frames[f].contentDocument;
                if(!d) continue;
                var all = d.querySelectorAll('*');
                for(var j=0;j<all.length;j++){{
                    var el = all[j];
                    if(el.offsetParent !== null && el.textContent.trim() === templateName){{
                        forceClick(el);
                        found = true;
                        break;
                    }}
                }}
                if(found) break;
            }}catch(e){{}}
        }}
        if(found) break;
        // Sleep ~200ms between attempts to avoid CPU busy-wait
        var t0 = Date.now();
        while(Date.now() - t0 < 200) {{}}
    }}
    if(!found) return false;
    // 4. Click the visible button whose text is exactly "添加到正文" if found;
    //    otherwise fall back to the first visible button containing "添加".
    var exactBtn = null;
    var fallbackBtn = null;
    var btns = document.querySelectorAll('button');
    for(var i=0;i<btns.length;i++){{
        var btn = btns[i];
        if(btn.offsetParent === null) continue;
        var txt = btn.textContent.trim();
        if(txt === '添加到正文'){{
            exactBtn = btn;
            break;
        }}
        if(fallbackBtn === null && txt.indexOf('添加') >= 0){{
            fallbackBtn = btn;
        }}
    }}
    if(exactBtn){{
        forceClick(exactBtn);
        return true;
    }}
    if(fallbackBtn){{
        forceClick(fallbackBtn);
        return true;
    }}
    var frames = document.querySelectorAll('iframe');
    for(var f=0;f<frames.length;f++){{
        try{{
            var d = frames[f].contentDocument;
            if(!d) continue;
            exactBtn = null;
            fallbackBtn = null;
            var btns2 = d.querySelectorAll('button');
            for(var i=0;i<btns2.length;i++){{
                var btn = btns2[i];
                if(btn.offsetParent === null) continue;
                var txt = btn.textContent.trim();
                if(txt === '添加到正文'){{
                    exactBtn = btn;
                    break;
                }}
                if(fallbackBtn === null && txt.indexOf('添加') >= 0){{
                    fallbackBtn = btn;
                }}
            }}
            if(exactBtn){{
                forceClick(exactBtn);
                return true;
            }}
            if(fallbackBtn){{
                forceClick(fallbackBtn);
                return true;
            }}
        }}catch(e){{}}
    }}
    return false;
}})()"#
    )
}

fn aicover_script(prompt: &str) -> String {
    let escaped = crate::cdp::js_str(prompt);
    format!(
        r#"(async function(){{
    var promptText = {escaped};
    var forceClick = function(el){{
        if(!el) return false;
        el.scrollIntoView({{block:'center'}});
        var o = {{bubbles:true, cancelable:true, view:window}};
        el.dispatchEvent(new MouseEvent('mousedown', o));
        el.dispatchEvent(new MouseEvent('mouseup', o));
        el.click();
        return true;
    }};
    var wait = function(ms){{ return new Promise(function(resolve){{ setTimeout(resolve, ms); }}); }};
    var visible = function(el){{ return !!el && el.offsetParent !== null; }};

    var coverArea = document.querySelector('.js_cover_btn_area');
    if(!visible(coverArea)) return false;
    coverArea.dispatchEvent(new MouseEvent('mouseover', {{bubbles:true}}));
    await wait(300);

    var aiBtn = document.querySelector('.js_aiImage');
    if(!visible(aiBtn) || !forceClick(aiBtn)) return false;
    await wait(1200);

    var input = document.querySelector('textarea, input[type="text"]');
    if(!visible(input)) return false;
    input.focus();
    input.value = promptText;
    input.dispatchEvent(new Event('input', {{bubbles:true}}));
    input.dispatchEvent(new Event('change', {{bubbles:true}}));
    await wait(200);

    var sendBtn = document.querySelector('button.send-btn');
    if(!visible(sendBtn) || !forceClick(sendBtn)) return false;

    var found = null;
    for(var i=0;i<40;i++){{
        var imgs = document.querySelectorAll('img');
        for(var j=0;j<imgs.length;j++){{
            var img = imgs[j];
            if(img.src.includes('mpimageai') && img.naturalWidth > 500){{
                found = img;
                break;
            }}
        }}
        if(found) break;
        await wait(500);
    }}
    if(!found || !forceClick(found)) return false;
    await wait(300);

    var clickButton = function(label){{
        var buttons = document.querySelectorAll('button');
        for(var i=0;i<buttons.length;i++){{
            var btn = buttons[i];
            if(visible(btn) && btn.textContent.trim() === label) {{
                return forceClick(btn);
            }}
        }}
        return false;
    }};

    if(!clickButton('使用')) return false;
    await wait(300);
    return clickButton('确认');
}})()"#
    )
}

#[cfg(test)]
mod tests {
    use super::aicover_script;
    use super::moban_script;

    #[test]
    fn moban_script_contains_escaped_template_name() {
        let template_name = "test\"template\\name";
        let js = moban_script(template_name);
        // The escaped template name should appear in the JS string
        assert!(js.contains("test\\\"template\\\\name"));
        // Verify the variable assignment line is present
        assert!(js.contains("var templateName = "));
        // Verify the polling loop is present
        assert!(js.contains("Date.now() - start < 3000"));
        // Verify the exact button match logic
        assert!(js.contains("txt === '添加到正文'"));
        // Verify the sleep between polling attempts
        assert!(js.contains("Date.now() - t0 < 200"));
        // Verify the early return when template not found
        assert!(js.contains("if(!found) return false;"));
    }

    #[test]
    fn moban_script_with_plain_name() {
        let js = moban_script("My Template");
        assert!(js.contains("var templateName = \"My Template\""));
    }

    #[test]
    fn aicover_script_contains_prompt_and_selectors() {
        let js = aicover_script("一张安静书桌上的读书思考");

        assert!(js.contains(".js_cover_btn_area"));
        assert!(js.contains(".js_aiImage"));
        assert!(js.contains("button.send-btn"));
        assert!(js.contains("mpimageai"));
        assert!(js.contains("一张安静书桌上的读书思考"));
    }
}
