//! Individual WeChat editor automation steps.
//!
//! Each function corresponds to one configuration action in the WeChat backend.
//! They are intentionally small and sequential: open dialog, click option, confirm.
//! All CDP primitives come from `crate::cdp`.

use chromiumoxide::Page;

use crate::cdp::{close_dialog, sleep_ms};

async fn eval_json(page: &Page, script: &str) -> serde_json::Value {
    let out = page
        .evaluate(script)
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();
    serde_json::from_str(&out).unwrap_or(serde_json::Value::Null)
}

fn original_script(template: &str) -> String {
    template.replace("__FIND_DIALOG__", ORIGINAL_FIND_DIALOG_JS)
}

const ORIGINAL_FIND_DIALOG_JS: &str = r#"
    var findDlg = function() {
        var vis = function(el) { return !!el && el.offsetParent !== null; };
        var cands = document.querySelectorAll('.claim__original-dialog');
        for (var i = 0; i < cands.length; i++) {
            if (vis(cands[i])) return cands[i];
        }
        var dlgs = document.querySelectorAll('.weui-desktop-dialog');
        for (var j = 0; j < dlgs.length; j++) {
            if (!vis(dlgs[j])) continue;
            var t = dlgs[j].textContent || '';
            if (t.indexOf('原创') >= 0 && t.indexOf('我已阅读') >= 0) return dlgs[j];
        }
        return null;
    };
"#;

const ORIGINAL_CONFIGURE_SCRIPT: &str = r#"(() => {
    __FIND_DIALOG__
    var dlg = findDlg();
    if (!dlg) return JSON.stringify({dialog: false});
    // 协议勾选框和确定按钮都在弹窗底部 __ft（.original_agreement），
    // 不在正文 #js_original_edit_box 内 — 容器选错会永远找不到。
    var agreed = null;
    var ag = dlg.querySelector('.original_agreement input[type="checkbox"]');
    if (!ag) {
        var cbs = dlg.querySelectorAll('input[type="checkbox"]');
        for (var i = 0; i < cbs.length; i++) {
            var ctx = ((cbs[i].closest('label') || cbs[i].parentElement || cbs[i]).textContent || '');
            if (ctx.indexOf('我已阅读') >= 0) { ag = cbs[i]; break; }
        }
    }
    if (ag) { if (!ag.checked) ag.click(); agreed = ag.checked; }
    var radio = dlg.querySelector('input.js_original_type_radio[value="0"]');
    if (radio && !radio.checked) radio.click();
    var confirm = false;
    var scope = dlg.querySelector('.weui-desktop-dialog__ft') || dlg;
    var btns = scope.querySelectorAll('button');
    for (var j = 0; j < btns.length; j++) {
        var t = (btns[j].textContent || '').replace(/\s+/g, '');
        if (t === '确定' && btns[j].offsetParent !== null) { btns[j].click(); confirm = true; break; }
    }
    return JSON.stringify({dialog: true, agreed: agreed, confirm: confirm});
})()"#;

const ORIGINAL_VERIFY_SCRIPT: &str = r#"(() => {
    __FIND_DIALOG__
    var dlg = findDlg();
    // 已声明块 #js_original_open 的可见性才是真实状态；未声明块永远藏在
    // DOM 里（display:none），读 textContent 会把隐藏模板文字当成状态。
    var open = document.getElementById('js_original_open');
    var declared = !!open && open.offsetParent !== null;
    var err = '';
    if (dlg) {
        var e = dlg.querySelector('.js_author_error');
        if (e && e.offsetParent !== null) err = (e.textContent || '').trim();
    }
    return JSON.stringify({dialogOpen: !!dlg, declared: declared, error: err});
})()"#;

const OPEN_ORIGINAL_DIALOG_SCRIPT: &str = r#"(() => {
    var vis = function(el) { return !!el && el.offsetParent !== null; };
    var open = document.getElementById('js_original_open');
    if (vis(open)) return 'already-declared';
    var blocks = document.querySelectorAll('#js_original .js_original_type');
    for (var i = 0; i < blocks.length; i++) {
        if (!vis(blocks[i])) continue;
        var trigger = blocks[i].querySelector('.js_edit_ori') || blocks[i];
        trigger.scrollIntoView({block: 'center'});
        var o = {bubbles: true, cancelable: true, view: window};
        trigger.dispatchEvent(new MouseEvent('mousedown', o));
        trigger.dispatchEvent(new MouseEvent('mouseup', o));
        trigger.click();
        return 'clicked';
    }
    return 'no-trigger';
})()"#;

const REWARD_STATE_SCRIPT: &str = r#"(() => {
    var vis = function(el) { return !!el && el.offsetParent !== null; };
    var open = document.getElementById('js_original_open');
    var declared = vis(open);
    var sw = document.querySelector('.js_reward_open');
    var tips = vis(sw) ? (sw.textContent || '').replace(/\s+/g, ' ').trim() : '';
    var enabled = tips !== '' && tips.indexOf('不开启') < 0;
    return JSON.stringify({declared: declared, tips: tips, enabled: enabled});
})()"#;

pub async fn step_yuanzhuang(page: &Page) {
    println!("▶ 原创声明...");
    let opened = page
        .evaluate(OPEN_ORIGINAL_DIALOG_SCRIPT)
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();
    println!("    open 原创弹窗: {opened}");
    if opened == "already-declared" {
        println!("  ✅ 原创已声明（无需重复操作）");
        return;
    }
    if opened != "clicked" {
        println!("  ⚠ 未找到原创声明入口 — skipping");
        return;
    }

    // 微信原创弹窗的作者信息是异步加载的；就绪检测在新声明弹窗里不可靠
    // （作者字段结构不同，永远误报"未就绪"），所以直接确认，被"作者不能为
    // 空且不超过8个字"拦截时等待后重试 — 真实失败路径只有这一条。
    sleep_ms(2_000).await;
    let mut declared = false;
    for attempt in 1..=3 {
        let cfg = eval_json(page, &original_script(ORIGINAL_CONFIGURE_SCRIPT)).await;
        if attempt == 1 {
            println!(
                "    协议勾选: {:?}, 点击确定: {:?}",
                cfg["agreed"].as_bool(),
                cfg["confirm"].as_bool()
            );
        }
        sleep_ms(1_200).await;
        let verify = eval_json(page, &original_script(ORIGINAL_VERIFY_SCRIPT)).await;
        let dialog_open = verify["dialogOpen"].as_bool() == Some(true);
        let error = verify["error"].as_str().unwrap_or("");
        if dialog_open && !error.is_empty() {
            println!("    第 {attempt} 次确认被微信校验拦截: {error}");
            sleep_ms(3_000).await;
            continue;
        }
        if !dialog_open {
            declared = true;
            break;
        }
        println!("    第 {attempt} 次确认后弹窗仍未关闭");
    }

    let verify = eval_json(page, &original_script(ORIGINAL_VERIFY_SCRIPT)).await;
    declared = declared && verify["declared"].as_bool() == Some(true);
    if declared {
        println!("  ✅ 原创已声明");
    } else {
        println!("  ⚠ 原创声明未生效 — 行内仍显示'未声明'");
    }
}

pub async fn step_zanshang(page: &Page) {
    println!("▶ 赞赏...");
    let state = eval_json(page, REWARD_STATE_SCRIPT).await;
    let declared = state["declared"].as_bool() == Some(true);
    let enabled = state["enabled"].as_bool() == Some(true);
    let tips = state["tips"].as_str().unwrap_or("");
    if enabled {
        println!("  ✅ 赞赏已开启（无需重复操作, state='{tips}'）");
        return;
    }
    if !declared {
        println!("  ⚠ 赞赏未开启 — 需先声明原创（声明原创后才可开启赞赏）");
        return;
    }
    // 真实产品路径（实测）：原创弹窗内的赞赏区域默认隐藏，正确入口是设置行
    // .js_reward_open 开关 → 赞赏设置弹窗 → 确定；落库靠后续保存草稿。
    let opened = eval_json(page, ZANSHANG_OPEN_SCRIPT).await;
    if opened["clicked"].as_bool() != Some(true) {
        println!("  ⚠ 赞赏开关不可点击 — skipping");
        return;
    }
    println!("    点击设置行赞赏开关: true");
    sleep_ms(2_000).await;
    let confirm = eval_json(page, ZANSHANG_CONFIRM_SCRIPT).await;
    println!(
        "    弹窗操作: {}",
        confirm["actions"].as_str().unwrap_or("(none)")
    );
    sleep_ms(1_500).await;
    let after = eval_json(page, REWARD_STATE_SCRIPT).await;
    let after_tips = after["tips"].as_str().unwrap_or("");
    if after["enabled"].as_bool() == Some(true) {
        println!("  ✅ 赞赏已开启 (state='{after_tips}')");
    } else {
        println!("  ⚠ 赞赏未开启 (state='{after_tips}') — 请到后台人工核对");
    }
}

const ZANSHANG_OPEN_SCRIPT: &str = r#"(() => {
    var vis = function(el) { return !!el && el.offsetParent !== null; };
    var sw = document.querySelector('.js_reward_open');
    if (!vis(sw)) return JSON.stringify({clicked: false});
    sw.scrollIntoView({block: 'center'});
    var o = {bubbles: true, cancelable: true, view: window};
    sw.dispatchEvent(new MouseEvent('mousedown', o));
    sw.dispatchEvent(new MouseEvent('mouseup', o));
    sw.click();
    return JSON.stringify({clicked: true});
})()"#;

const ZANSHANG_CONFIRM_SCRIPT: &str = r#"(() => {
    var vis = function(el) { return !!el && el.offsetParent !== null; };
    var norm = function(s) { return (s || '').replace(/\s+/g, '').trim(); };
    var dlgs = document.querySelectorAll('.weui-desktop-dialog, [class*="dialog"]');
    for (var i = dlgs.length - 1; i >= 0; i--) {
        var d = dlgs[i];
        if (!vis(d)) continue;
        if ((d.textContent || '').indexOf('赞赏') < 0) continue;
        var actions = [];
        var cbs = d.querySelectorAll('input[type="checkbox"]');
        for (var c = 0; c < cbs.length; c++) {
            var ctx = ((cbs[c].closest('label') || cbs[c].parentElement || cbs[c]).textContent || '');
            if (ctx.indexOf('协议') >= 0 && !cbs[c].checked) {
                cbs[c].click();
                actions.push('agreement->' + cbs[c].checked);
            }
        }
        var btns = d.querySelectorAll('button');
        for (var b = 0; b < btns.length; b++) {
            if (norm(btns[b].textContent) === '确定' && vis(btns[b])) {
                btns[b].click();
                actions.push('confirm');
            }
        }
        return JSON.stringify({actions: actions.join(',') || 'nothing'});
    }
    return JSON.stringify({actions: 'no-dialog'});
})()"#;

pub async fn step_liuyan(page: &Page) {
    println!("▶ 留言...");
    // 状态优先：input.js_interaction_setting 的 checked 是真实状态。
    // 旧版盲点"确定"会命中其它未关闭弹窗（曾把原创弹窗的确定当留言确认），
    // 所以确认按钮必须限定在含"留言"的可见弹窗内。
    let enable = eval_json(page, COMMENT_ENABLE_SCRIPT).await;
    match enable["state"].as_str() {
        Some("already-on") => {
            println!("  ✅ 留言已开启（无需重复操作）");
            return;
        }
        Some("clicked") => {
            println!("    点击留言开关: true");
        }
        _ => {
            println!("  ⚠ 留言开关未找到 — skipping");
            return;
        }
    }
    sleep_ms(1_500).await;
    let confirm = eval_json(page, COMMENT_CONFIRM_SCRIPT).await;
    println!("    弹窗确认: {:?}", confirm["confirm"].as_bool());
    sleep_ms(800).await;
    let state = eval_json(page, COMMENT_STATE_SCRIPT).await;
    if state["on"].as_bool() == Some(true) {
        println!("  ✅ 留言已开启");
    } else {
        println!("  ⚠ 留言未开启 — 请到后台人工核对");
    }
}

const COMMENT_ENABLE_SCRIPT: &str = r#"(() => {
    var cb = document.querySelector('input.js_interaction_setting');
    if (!cb) return JSON.stringify({state: 'no-switch'});
    if (cb.checked) return JSON.stringify({state: 'already-on'});
    cb.scrollIntoView({block: 'center'});
    cb.click();
    return JSON.stringify({state: 'clicked', checked: cb.checked});
})()"#;

const COMMENT_CONFIRM_SCRIPT: &str = r#"(() => {
    var dlgs = document.querySelectorAll('.weui-desktop-dialog, [class*="dialog"]');
    for (var i = dlgs.length - 1; i >= 0; i--) {
        var d = dlgs[i];
        if (d.offsetParent === null) continue;
        if ((d.textContent || '').indexOf('留言') < 0) continue;
        var btns = d.querySelectorAll('button');
        for (var j = 0; j < btns.length; j++) {
            var t = (btns[j].textContent || '').replace(/\s+/g, '');
            if (t === '确定' && btns[j].offsetParent !== null) {
                btns[j].click();
                return JSON.stringify({confirm: true});
            }
        }
    }
    return JSON.stringify({confirm: false});
})()"#;

const COMMENT_STATE_SCRIPT: &str = r#"(() => {
    var cb = document.querySelector('input.js_interaction_setting');
    return JSON.stringify({on: cb ? cb.checked : null});
})()"#;

const CLOSE_VISIBLE_DIALOGS_SCRIPT: &str = r#"(() => {
    var vis = function(el) { return !!el && el.offsetParent !== null; };
    var closers = document.querySelectorAll('.weui-desktop-dialog__close-btn, .weui-desktop-dialog__wrp .weui-desktop-icon-btn');
    var n = 0;
    for (var i = 0; i < closers.length; i++) {
        if (!vis(closers[i])) continue;
        closers[i].click();
        n++;
    }
    return n;
})()"#;

const CHUANGZUO_CONFIRM_SCRIPT: &str = r#"(() => {
    // position:fixed 元素 offsetParent 为 null，必须用 getClientRects 判可见
    var vis = function(el) { return !!el && el.getClientRects().length > 0; };
    var norm = function(s) { return (s || '').replace(/\s+/g, '').trim(); };
    var radio = document.querySelector('input[type="radio"][value="4"]');
    if (!radio) return JSON.stringify({confirm: false});
    var findBtn = function(root) {
        var els = root.querySelectorAll('button, a, span, div, li, [role="button"]');
        for (var i = 0; i < els.length; i++) {
            var t = norm(els[i].textContent);
            if ((t === '确认' || t === '确定') && vis(els[i])) return els[i];
        }
        return null;
    };
    // picker 的确认按钮可能在 radio 所在盒子的兄弟 footer 里，
    // 从 radio 逐层向上，第一个包含可见确认按钮的祖先即为 picker 整体
    var node = radio;
    var btn = null;
    while (node && node !== document.body) {
        node = node.parentElement;
        if (!node) break;
        btn = findBtn(node);
        if (btn) break;
    }
    if (btn) {
        btn.scrollIntoView({block: 'center'});
        var r = btn.getBoundingClientRect();
        return JSON.stringify({found: true, x: r.left + r.width / 2, y: r.top + r.height / 2});
    }
    return JSON.stringify({found: false});
})()"#;

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

    // 确认按钮必须限定在创作来源 picker 容器内（以 radio 定位），
    // 全局盲点"确认/确定"会命中其它未关闭弹窗。
    // 微信编辑器的 Vue 模型忽略 JS 合成点击（isTrusted=false），脚本只取坐标，
    // 真正点击走 CDP 可信鼠标事件。
    let confirm_rect = eval_json(page, CHUANGZUO_CONFIRM_SCRIPT).await;
    let ok3 = if confirm_rect["found"].as_bool() == Some(true) {
        let x = confirm_rect["x"].as_f64().unwrap_or(0.0);
        let y = confirm_rect["y"].as_f64().unwrap_or(0.0);
        page.click(chromiumoxide::layout::Point { x, y }).await.ok();
        true
    } else {
        false
    };
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

pub async fn step_fucha(page: &Page) {
    println!("▶ 复核后台设置（重新加载编辑器）...");
    // 只信保存后的服务器状态：重载编辑器，读设置行的真实文案。
    // 点击日志和内存态都可能是假成功（弹窗遮挡、校验拦截），重载后的
    // 页面状态才是微信后台真正落库的结果。
    let _ = page.evaluate("location.reload()").await;
    let mut loaded = false;
    for _ in 0..30 {
        sleep_ms(500).await;
        let ok = page
            .evaluate("document.querySelector('#js_original')!==null")
            .await
            .ok()
            .and_then(|v| v.value().and_then(|v| v.as_bool()))
            .unwrap_or(false);
        if ok {
            loaded = true;
            break;
        }
    }
    if !loaded {
        println!("  ⚠ 复核页面未加载 — 请到后台人工核对");
        return;
    }
    sleep_ms(1_500).await;
    // 页面刚 reload 时设置区可能还没渲染完，轮询直到读到非空状态或超时。
    let mut state = serde_json::Value::Null;
    for _ in 0..10 {
        sleep_ms(800).await;
        state = eval_json(page, FUCHA_SCRIPT).await;
        if state
            .get("original")
            .and_then(|v| v.as_str())
            .map(|s| !s.is_empty())
            .unwrap_or(false)
        {
            break;
        }
    }
    let original = state["original"].as_str().unwrap_or("");
    let reward = state["reward"].as_str().unwrap_or("");
    let source = state["source"].as_str().unwrap_or("");
    let comment = state["comment"].as_str().unwrap_or("");
    if original.starts_with("declared") {
        println!("  ✅ 原创已落库");
    } else {
        println!("  ⚠ 原创未保存到后台");
    }
    if !reward.is_empty() && !reward.contains("不开启") {
        println!("  ✅ 赞赏已落库 ({reward})");
    } else {
        println!(
            "  ⚠ 赞赏未保存到后台 (state='{}')",
            if reward.is_empty() {
                "未识别"
            } else {
                reward
            }
        );
    }
    if source.is_empty() {
        println!("  ⚠ 创作来源未保存到后台");
    } else {
        println!("  ✅ 创作来源已落库 ({source})");
    }
    match comment {
        "on" => println!("  ✅ 留言已落库"),
        "off" => println!("  ⚠ 留言未保存到后台"),
        _ => println!("  ℹ 留言状态未识别"),
    }
}

const FUCHA_SCRIPT: &str = r#"(() => {
    var vis = function(el) { return !!el && el.offsetParent !== null; };
    var norm = function(s) { return (s || '').replace(/\s+/g, ' ').trim(); };
    // #js_original_open 可见 = 已声明；未声明块永远藏在 DOM 里（display:none），
    // 不能靠 textContent 里的"未声明"字样判断。
    var open = document.getElementById('js_original_open');
    var original = vis(open) ? 'declared' : 'undeclared';
    var rewardEl = document.querySelector('.js_reward_open');
    var reward = vis(rewardEl) ? norm(rewardEl.textContent) : '';
    var sourceEl = document.querySelector('.js_claim_source_selected');
    var source = sourceEl ? norm(sourceEl.textContent) : '';
    var commentCb = document.querySelector('input.js_interaction_setting');
    var comment = commentCb ? (commentCb.checked ? 'on' : 'off') : 'unknown';
    return JSON.stringify({original: original, reward: reward, source: source, comment: comment});
})()"#;

pub async fn step_baocun(page: &Page) {
    println!("▶ 保存草稿...");
    // 任何残留弹窗都会挡住保存按钮的点击（picker 未关曾导致整轮设置不落库），
    // 保存前先关掉所有可见弹窗。
    for _ in 0..3 {
        let closed = page
            .evaluate(CLOSE_VISIBLE_DIALOGS_SCRIPT)
            .await
            .ok()
            .and_then(|v| v.value().and_then(|v| v.as_u64()))
            .unwrap_or(0);
        if closed == 0 {
            break;
        }
        println!("    关闭残留弹窗: {closed} 个");
        sleep_ms(500).await;
    }
    let rect_json = page
        .evaluate(SAVE_DRAFT_RECT_SCRIPT)
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();
    let clicked = match serde_json::from_str::<serde_json::Value>(&rect_json) {
        Ok(v) if v["found"].as_bool() == Some(true) => {
            let x = v["x"].as_f64().unwrap_or(0.0);
            let y = v["y"].as_f64().unwrap_or(0.0);
            let label = v["label"].as_str().unwrap_or("保存");
            println!("    click '{label}': true");
            page.click(chromiumoxide::layout::Point { x, y }).await.ok();
            true
        }
        _ => {
            println!("    click '保存为草稿': false");
            false
        }
    };
    if !clicked {
        println!("  ⚠ 保存按钮未找到 — 后台设置可能仍未持久化");
        return;
    }

    let mut saved = false;
    for _ in 0..30 {
        sleep_ms(500).await;
        saved = page
            .evaluate(SAVE_DRAFT_STATE_SCRIPT)
            .await
            .ok()
            .and_then(|v| v.value().and_then(|v| v.as_bool()))
            .unwrap_or(false);
        if saved {
            break;
        }
    }
    if saved {
        println!("  ✅ 草稿已保存");
    } else {
        println!(
            "  ⚠ 已点击保存，但 15 秒内未识别到保存成功提示 — 请到后台人工核对；如果后台实际已保存，可忽略本警告"
        );
    }
}

const SAVE_DRAFT_RECT_SCRIPT: &str = r#"(() => {
    var norm = function(s) { return (s || '').replace(/\s+/g, ' ').trim(); };
    var isSaveButton = function(el) {
        var text = norm(el.innerText || el.textContent);
        return text === '保存为草稿' || text === '保存';
    };
    var search = function(root) {
        var preferred = ['.js_editor_save_draft', '#js_editor_save_draft', 'button[data-action="save"]'];
        for (var s = 0; s < preferred.length; s++) {
            var el = root.querySelector(preferred[s]);
            if (el && el.offsetParent !== null) {
                var rect = el.getBoundingClientRect();
                return {found: true, x: rect.left + rect.width / 2, y: rect.top + rect.height / 2, label: norm(el.innerText || el.textContent) || '保存'};
            }
        }
        var els = root.querySelectorAll('button, a, [role="button"]');
        for (var i = 0; i < els.length; i++) {
            if (els[i].offsetParent !== null && isSaveButton(els[i])) {
                var r = els[i].getBoundingClientRect();
                return {found: true, x: r.left + r.width / 2, y: r.top + r.height / 2, label: norm(els[i].innerText || els[i].textContent)};
            }
        }
        var all = root.querySelectorAll('*');
        for (var j = 0; j < all.length; j++) {
            if (all[j].shadowRoot) {
                var nested = search(all[j].shadowRoot);
                if (nested) return nested;
            }
        }
        return null;
    };
    var found = search(document);
    if (!found) {
        var frames = document.querySelectorAll('iframe');
        for (var f = 0; f < frames.length; f++) {
            try {
                var doc = frames[f].contentDocument;
                if (doc) {
                    found = search(doc);
                    if (found) break;
                }
            } catch (e) {}
        }
    }
    return JSON.stringify(found || {found: false});
})()"#;

const SAVE_DRAFT_STATE_SCRIPT: &str = r#"(() => {
    var text = (document.body && document.body.innerText || '').replace(/\s+/g, '');
    return text.indexOf('保存成功') >= 0 || text.indexOf('已保存') >= 0 || text.indexOf('已存草稿') >= 0;
})()"#;

/// Best-effort discovery of the last-used preview recipient from the live
/// WeChat editor page. When the user has manually previewed before, WeChat
/// remembers the target WeChat id in the page's local/session storage, so we
/// can replay it without forcing the user to type `--to` every time.
/// Best-effort discovery of the last-used preview recipient from the live
/// WeChat editor page. WeChat does NOT persist the preview target in
/// localStorage/sessionStorage (verified: the editor page only keeps 8
/// unrelated keys), so this almost always returns None. It remains as a
/// last-resort heuristic in case a future WeChat build starts caching it.
async fn autodetect_preview_wxname(page: &Page) -> Option<String> {
    let js = r#"(() => {
        try {
            var cands = [];
            function scan(store) {
                for (var i = 0; i < store.length; i++) {
                    var k = store.key(i);
                    var val = '';
                    try { val = store.getItem(k) || ''; } catch (e) {}
                    var blob = (k + ' ' + val).toLowerCase();
                    if (blob.indexOf('preview') >= 0 || blob.indexOf('yulan') >= 0 ||
                        blob.indexOf('preuser') >= 0 || blob.indexOf('recent') >= 0 ||
                        blob.indexOf('预览') >= 0) {
                        var ms = val.match(/wxid_[a-zA-Z0-9_-]+/g);
                        if (ms) cands.push.apply(cands, ms);
                        var ms2 = val.match(/"?([a-zA-Z][a-zA-Z0-9_-]{5,19})"?/g);
                        if (ms2) cands.push.apply(cands, ms2);
                    }
                }
            }
            scan(localStorage);
            scan(sessionStorage);
            var seen = {}, uniq = [];
            for (var c of cands) { c = String(c).replace(/^"|"$/g, ''); if (!seen[c]) { seen[c] = 1; uniq.push(c); } }
            uniq.sort(function (a, b) {
                return (a.indexOf('wxid_') >= 0 ? 0 : 1) - (b.indexOf('wxid_') >= 0 ? 0 : 1);
            });
            return JSON.stringify({ candidates: uniq });
        } catch (e) { return JSON.stringify({ error: String(e) }); }
    })()"#;
    let out = page
        .evaluate(js)
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();
    let j: serde_json::Value = serde_json::from_str(&out).unwrap_or(serde_json::Value::Null);
    if let Some(err) = j["error"].as_str() {
        println!("    (自动读取预览接收人失败: {err})");
        return None;
    }
    j["candidates"]
        .as_array()
        .and_then(|arr| arr.first())
        .and_then(|v| v.as_str())
        .map(|s| s.to_owned())
}

/// Read a persisted preview recipient from `.moonpub/preview_to` (project
/// local). This lets the user supply their WeChat id once and have later
/// `test-yulan` runs work with no arguments.
fn preview_to_from_config() -> Option<String> {
    let path = std::path::Path::new(".moonpub").join("preview_to");
    std::fs::read_to_string(&path)
        .ok()
        .map(|s| s.trim().to_owned())
        .filter(|s| !s.is_empty())
}

/// Persist a successfully-used preview recipient so future runs are automatic.
fn persist_preview_to(wxname: &str) {
    let dir = std::path::Path::new(".moonpub");
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }
    let path = dir.join("preview_to");
    let _ = std::fs::write(&path, wxname);
}

pub async fn step_yulan(page: &Page, to_wxname: Option<&str>) {
    println!("▶ 预览 (cookie 接口)...");
    // Pull token + appmsgid straight from the editor URL — no UI click needed.
    // WeChat drops synthetic clicks (isTrusted===false), so instead we call the
    // real backend endpoint with our live session cookie (same-origin fetch).
    let info = page
        .evaluate(
            r#"(() => {
            var u = new URL(location.href);
            return JSON.stringify({
                token: u.searchParams.get('token') || '',
                appmsgid: u.searchParams.get('appmsgid') || u.searchParams.get('id') || ''
            });
        })()"#,
        )
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();
    let v: serde_json::Value = serde_json::from_str(&info).unwrap_or(serde_json::Value::Null);
    let token = v["token"].as_str().unwrap_or("").to_owned();
    let appmsgid = v["appmsgid"].as_str().unwrap_or("").to_owned();
    if token.is_empty() || appmsgid.is_empty() {
        println!("  ⚠ 无法从编辑器页获取 token/appmsgid (token='{token}' appmsgid='{appmsgid}')");
        return;
    }
    // Resolve recipient: --to > WECHAT_PREVIEW_TO > .moonpub/preview_to > auto-detect.
    let explicit = match to_wxname {
        Some(s) if !s.trim().is_empty() => Some(s.trim().to_owned()),
        _ => std::env::var("WECHAT_PREVIEW_TO")
            .ok()
            .filter(|s| !s.trim().is_empty()),
    };
    let explicit = match explicit {
        Some(s) => Some(s),
        None => preview_to_from_config(),
    };
    let to_wxname = match explicit {
        Some(s) => s,
        None => match autodetect_preview_wxname(page).await {
            Some(s) => {
                println!("    从微信页面自动读取到预览接收人: {s}");
                s
            }
            None => {
                println!("  ⚠ 微信后台预览需要知道发给谁，但找不到已保存的接收人。");
                println!(
                    "    原因: 微信 preview 接口要求显式传入微信号；脚本无法读取微信对话框里记住的号码。"
                );
                println!("    解决方法（任选其一，只需配置一次，之后自动记住）：");
                println!("      1) 本次加参数: --to <你的微信号>");
                println!("      2) 设置环境变量: WECHAT_PREVIEW_TO=<你的微信号>");
                println!("      3) 运行一次: moonpub test-yulan --to <你的微信号>");
                println!("    本次跳过预览发送，configure 的其它步骤仍会继续完成。");
                return;
            }
        },
    };
    let preusername_list = format!("{{\"preusername\":[\"{}\"]}}", to_wxname.replace('"', ""));
    let js = format!(
        r#"(async () => {{
            var token = {tok};
            var appmsgid = {aid};
            var preusername_list = {pl};
            var body = 'appmsgid=' + encodeURIComponent(appmsgid)
                + '&AppMsgId=' + encodeURIComponent(appmsgid)
                + '&preusername_list=' + encodeURIComponent(preusername_list)
                + '&is_preview=1&preview_mode_type=0';
            try {{
                var r = await fetch('/cgi-bin/operate_appmsg?sub=preview&t=ajax-appmsg-preview&type=10&token=' + token, {{
                    method: 'POST',
                    headers: {{'Content-Type': 'application/x-www-form-urlencoded'}},
                    body: body
                }});
                return JSON.stringify({{status: r.status, body: (await r.text()).substring(0, 600)}});
            }} catch (e) {{ return JSON.stringify({{status: -1, body: String(e)}}); }}
        }})()"#,
        tok = serde_json::to_string(&token).unwrap_or_default(),
        aid = serde_json::to_string(&appmsgid).unwrap_or_default(),
        pl = serde_json::to_string(&preusername_list).unwrap_or_default(),
    );
    let resp = page
        .evaluate(js)
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_str().map(|s| s.to_owned())))
        .unwrap_or_default();
    println!("    preview resp: {resp}");
    if let Ok(j) = serde_json::from_str::<serde_json::Value>(&resp) {
        let status = j["status"].as_i64().unwrap_or(-1);
        let body = j["body"].as_str().unwrap_or("").to_owned();
        println!("    http status: {status}");
        if let Ok(b) = serde_json::from_str::<serde_json::Value>(&body) {
            let ret = b["ret"]
                .as_str()
                .or_else(|| b["base_resp"]["ret"].as_str())
                .unwrap_or("?");
            let err = b["err_msg"]
                .as_str()
                .or_else(|| b["base_resp"]["err_msg"].as_str())
                .unwrap_or("");
            if ret == "0" {
                println!("  ✅ 预览已发送到手机微信 (ret=0, 接收人: {to_wxname})");
                persist_preview_to(&to_wxname);
                println!("    (已记住该接收人，下次无需 --to)");
            } else {
                println!("  ⚠ 预览返回 ret={ret} err={err}");
            }
        } else {
            println!("    (non-json body): {body}");
        }
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

#[cfg(test)]
mod tests {
    use super::{
        CHUANGZUO_CONFIRM_SCRIPT, CLOSE_VISIBLE_DIALOGS_SCRIPT, COMMENT_CONFIRM_SCRIPT,
        COMMENT_ENABLE_SCRIPT, COMMENT_STATE_SCRIPT, FUCHA_SCRIPT, ORIGINAL_CONFIGURE_SCRIPT,
        ORIGINAL_VERIFY_SCRIPT, REWARD_STATE_SCRIPT, SAVE_DRAFT_RECT_SCRIPT,
        SAVE_DRAFT_STATE_SCRIPT, ZANSHANG_CONFIRM_SCRIPT, ZANSHANG_OPEN_SCRIPT, moban_script,
        original_script,
    };

    #[test]
    fn original_configure_script_scopes_actions_to_dialog() {
        let js = original_script(ORIGINAL_CONFIGURE_SCRIPT);
        assert!(js.contains("我已阅读"));
        // 协议勾选和确定按钮都在弹窗底部 __ft，不在正文 #js_original_edit_box
        assert!(js.contains(".original_agreement"));
        assert!(js.contains(".weui-desktop-dialog__ft"));
    }

    #[test]
    fn original_verify_script_reads_settings_row_not_click_log() {
        let js = original_script(ORIGINAL_VERIFY_SCRIPT);
        // #js_original_open 可见性才是真实状态；隐藏模板里的"未声明"字样会误判
        assert!(js.contains("getElementById('js_original_open')"));
        assert!(js.contains("dialogOpen"));
    }

    #[test]
    fn reward_state_script_is_read_only() {
        assert!(!REWARD_STATE_SCRIPT.contains(".click()"));
        assert!(REWARD_STATE_SCRIPT.contains("js_original_open"));
    }

    #[test]
    fn zanshang_scripts_scope_confirm_to_reward_dialog() {
        assert!(ZANSHANG_OPEN_SCRIPT.contains(".js_reward_open"));
        assert!(ZANSHANG_CONFIRM_SCRIPT.contains("赞赏"));
        assert!(ZANSHANG_CONFIRM_SCRIPT.contains("确定"));
    }

    #[test]
    fn comment_scripts_are_state_first() {
        assert!(COMMENT_ENABLE_SCRIPT.contains("input.js_interaction_setting"));
        assert!(COMMENT_STATE_SCRIPT.contains("input.js_interaction_setting"));
        // 确认按钮必须限定在含"留言"的可见弹窗内，不能全局盲点
        assert!(COMMENT_CONFIRM_SCRIPT.contains("留言"));
    }

    #[test]
    fn chuangzuo_confirm_is_scoped_to_picker() {
        // 从已选中的 radio 逐层向上定位 picker，禁止全局盲点"确认/确定"
        assert!(CHUANGZUO_CONFIRM_SCRIPT.contains("parentElement"));
        assert!(CHUANGZUO_CONFIRM_SCRIPT.contains("input[type=\"radio\"][value=\"4\"]"));
        // picker 的"确认"可能是 span/div 而不是 button
        assert!(CHUANGZUO_CONFIRM_SCRIPT.contains("span, div, li"));
        // position:fixed 弹窗 offsetParent 为 null，必须用 getClientRects 判可见
        assert!(CHUANGZUO_CONFIRM_SCRIPT.contains("getClientRects"));
        // Vue 模型忽略 JS 合成点击，脚本只返回坐标，真正点击走 CDP 可信鼠标事件
        assert!(!CHUANGZUO_CONFIRM_SCRIPT.contains(".click()"));
        assert!(CHUANGZUO_CONFIRM_SCRIPT.contains("getBoundingClientRect"));
    }

    #[test]
    fn save_closes_leftover_dialogs_first() {
        assert!(CLOSE_VISIBLE_DIALOGS_SCRIPT.contains("__close-btn"));
    }

    #[test]
    fn fucha_script_reads_persisted_rows() {
        assert!(FUCHA_SCRIPT.contains("#js_original"));
        assert!(FUCHA_SCRIPT.contains(".js_claim_source_selected"));
        assert!(!FUCHA_SCRIPT.contains(".click()"));
    }

    #[test]
    fn save_draft_script_targets_editor_save_button() {
        assert!(SAVE_DRAFT_RECT_SCRIPT.contains(".js_editor_save_draft"));
        assert!(SAVE_DRAFT_RECT_SCRIPT.contains("text === '保存为草稿'"));
        assert!(SAVE_DRAFT_RECT_SCRIPT.contains("text === '保存'"));
    }

    #[test]
    fn save_draft_state_script_requires_positive_confirmation() {
        assert!(SAVE_DRAFT_STATE_SCRIPT.contains("保存成功"));
        assert!(SAVE_DRAFT_STATE_SCRIPT.contains("已保存"));
        assert!(SAVE_DRAFT_STATE_SCRIPT.contains("已存草稿"));
    }

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
}
