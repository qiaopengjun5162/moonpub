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
    // Click the "未添加" span that sits inside the "创作来源" row.
    // The editor has many "未添加" labels (original statement, source link, etc.);
    // we must scope the click to the row whose text contains "创作来源".
    let ok = page
        .evaluate(
            r#"(function(){
        var search=function(root){
            var rows=root.querySelectorAll('label, div, li, .weui-desktop-setting__item');
            for(var i=0;i<rows.length;i++){
                var row=rows[i];
                if(row.textContent.trim().indexOf('创作来源')<0) continue;
                // Prefer the right-hand "未添加" span (lbl_content_desc)
                var spans=row.querySelectorAll('span, a, div');
                for(var j=0;j<spans.length;j++){
                    var s=spans[j];
                    var cls=s.className||'';
                    var txt=s.textContent.trim();
                    if(txt==='未添加'&&(cls.indexOf('lbl_content')>=0||cls.indexOf('desc')>=0||s.tagName.toLowerCase()==='a')){
                        s.scrollIntoView({block:'center'});
                        s.click();
                        return true;
                    }
                }
                // Fallback: click any child that says 未添加
                for(var k=0;k<spans.length;k++){
                    if(spans[k].textContent.trim()==='未添加'){
                        spans[k].scrollIntoView({block:'center'});
                        spans[k].click();
                        return true;
                    }
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
        println!("  ⚠ '创作来源' row not found — skipping");
        return;
    }
    sleep_ms(1_500).await;
    // Select one of the known source options. WeChat may open a standalone
    // dropdown or merge this into the 原创声明 dialog; either way we look for
    // the option text itself rather than detecting a specific dialog type.
    let selected = page
        .evaluate(
            r#"(function(){
        var options=['个人观点，仅供参考','个人观点','转载','翻译','综合整理','其他'];
        var search=function(root){
            var els=root.querySelectorAll('span, label, div, li, a, button, p');
            for(var i=0;i<els.length;i++){
                var e=els[i];
                if(e.offsetParent===null) continue;
                var txt=e.textContent.trim().replace(/\s+/g,' ');
                for(var j=0;j<options.length;j++){
                    if(txt===options[j]){
                        e.scrollIntoView({block:'center'});
                        e.click();
                        return options[j];
                    }
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
        // No recognizable option — WeChat may have merged the entry into
        // another dialog. Close any open dialog and continue.
        let _ = close_dialog(page).await;
        println!("  ⚠ 创作来源 — 未找到可选项，跳过");
        return;
    }
    println!("    select '{selected}': true");
    sleep_ms(1_000).await;
    let mut ok3 = cdp_click_css(page, ".weui-desktop-dialog__ft .weui-desktop-btn_primary").await;
    if !ok3 {
        ok3 = cdp_click_css(page, ".weui-desktop-btn_primary").await;
    }
    if !ok3 {
        ok3 = cdp_click_text(page, "确认").await;
    }
    if !ok3 {
        ok3 = cdp_click_text(page, "确定").await;
    }
    println!("    click '确认': {ok3}");
    sleep_ms(1_000).await;
    if ok3 {
        println!("  ✅ 创作来源");
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
