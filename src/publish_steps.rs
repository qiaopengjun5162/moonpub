//! Individual WeChat editor automation steps.
//!
//! Each function corresponds to one configuration action in the WeChat backend.
//! They are intentionally small and sequential: open dialog, click option, confirm.
//!
//! 浏览器操作全部经由 [`StepIo`]：线上实现是 [`Page`] 本身（CDP），测试实现是内存假体。
//! 这样「拿到页面状态之后怎么决策」可以脱离真 Chrome 验证——微信后台真正会出问题的地方
//! 就在这里（弹窗没关、校验拦截、按钮找不到），而不是在 JS 字符串里。

use chromiumoxide::Page;

// 别名：`impl StepIo for Page` 里也有同名的 trait 方法，不加前缀会被读成递归调用。
use crate::cdp::{close_dialog as cdp_close_dialog, sleep_ms as cdp_sleep_ms};

/// 一步设置操作的结果。
///
/// 与日志解耦：`println!` 是给人看的，这个枚举是给调用方和测试看的。
/// 括号里是稳定的原因码，测试按它断言走的是哪条分支。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum StepOutcome {
    /// 目标状态本来就成立，没有发起写操作。
    AlreadyDone(&'static str),
    /// 发起了写操作，且随后的自检通过。
    Applied(&'static str),
    /// 找不到入口或控件，主动跳过（不算失败）。
    Skipped(&'static str),
    /// 点了，但自检没通过——需要人工到后台核对。
    Unverified(&'static str),
}

/// 编排层与浏览器的唯一接缝。
///
/// 方法集合是「`step_*` 真正需要的能力」，不是「`Page` 能做什么」：
/// 执行 JS 拿结果、按坐标点击、关弹窗、等待。
///
/// 为什么点击必须是坐标：微信编辑器的 Vue 模型忽略 JS 合成点击
/// （`isTrusted === false`），视觉上变了但不会写进草稿，所以真实路径是
/// 先取元素中心坐标，再发可信鼠标事件。
pub(crate) trait StepIo {
    /// 执行 JS，取原始字符串结果。脚本返回非字符串时按空串处理。
    async fn eval_text(&self, script: &str) -> String;

    /// 执行 JS，取布尔结果。
    async fn eval_bool(&self, script: &str) -> bool;

    /// 执行 JS，取无符号整数结果。
    async fn eval_u64(&self, script: &str) -> u64;

    /// 在视口坐标发一次可信鼠标点击。
    async fn click_at(&self, x: f64, y: f64);

    /// 关掉最上层可见弹窗，返回是否关掉了。
    async fn close_dialog(&self) -> bool;

    /// 等待。
    async fn sleep_ms(&self, ms: u64);

    /// 执行 JS，把结果按 JSON 解析；解析失败给 `Null`（与线上容错一致）。
    ///
    /// 默认实现建立在 [`StepIo::eval_text`] 上，假体只需实现文本读取。
    async fn eval_json(&self, script: &str) -> serde_json::Value {
        let raw = self.eval_text(script).await;
        serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null)
    }
}

/// 执行一段 JS 并取出返回值。CDP 失败或脚本没有返回值时给 `None`。
async fn evaluate_value(page: &Page, script: &str) -> Option<serde_json::Value> {
    page.evaluate(script)
        .await
        .ok()
        .and_then(|v| v.value().cloned())
}

impl StepIo for Page {
    async fn eval_text(&self, script: &str) -> String {
        evaluate_value(self, script)
            .await
            .as_ref()
            .and_then(|v| v.as_str())
            .unwrap_or_default()
            .to_owned()
    }

    async fn eval_bool(&self, script: &str) -> bool {
        evaluate_value(self, script)
            .await
            .as_ref()
            .and_then(|v| v.as_bool())
            .unwrap_or(false)
    }

    async fn eval_u64(&self, script: &str) -> u64 {
        evaluate_value(self, script)
            .await
            .as_ref()
            .and_then(|v| v.as_u64())
            .unwrap_or(0)
    }

    async fn click_at(&self, x: f64, y: f64) {
        self.click(chromiumoxide::layout::Point { x, y }).await.ok();
    }

    async fn close_dialog(&self) -> bool {
        cdp_close_dialog(self).await
    }

    async fn sleep_ms(&self, ms: u64) {
        cdp_sleep_ms(ms).await
    }
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

pub async fn step_yuanzhuang(io: &impl StepIo) -> StepOutcome {
    println!("▶ 原创声明...");
    let opened = io.eval_text(OPEN_ORIGINAL_DIALOG_SCRIPT).await;
    println!("    open 原创弹窗: {opened}");
    if opened == "already-declared" {
        println!("  ✅ 原创已声明（无需重复操作）");
        return StepOutcome::AlreadyDone("yuanzhuang-already-declared");
    }
    if opened != "clicked" {
        println!("  ⚠ 未找到原创声明入口 — skipping");
        return StepOutcome::Skipped("yuanzhuang-entry-not-found");
    }

    // 微信原创弹窗的作者信息是异步加载的；就绪检测在新声明弹窗里不可靠
    // （作者字段结构不同，永远误报"未就绪"），所以直接确认，被"作者不能为
    // 空且不超过8个字"拦截时等待后重试 — 真实失败路径只有这一条。
    io.sleep_ms(2_000).await;
    let mut declared = false;
    for attempt in 1..=3 {
        let cfg = io
            .eval_json(&original_script(ORIGINAL_CONFIGURE_SCRIPT))
            .await;
        if attempt == 1 {
            println!(
                "    协议勾选: {:?}, 点击确定: {:?}",
                cfg["agreed"].as_bool(),
                cfg["confirm"].as_bool()
            );
        }
        io.sleep_ms(1_200).await;
        let verify = io.eval_json(&original_script(ORIGINAL_VERIFY_SCRIPT)).await;
        let dialog_open = verify["dialogOpen"].as_bool() == Some(true);
        let error = verify["error"].as_str().unwrap_or("");
        if dialog_open && !error.is_empty() {
            println!("    第 {attempt} 次确认被微信校验拦截: {error}");
            io.sleep_ms(3_000).await;
            continue;
        }
        if !dialog_open {
            declared = true;
            break;
        }
        println!("    第 {attempt} 次确认后弹窗仍未关闭");
    }

    let verify = io.eval_json(&original_script(ORIGINAL_VERIFY_SCRIPT)).await;
    declared = declared && verify["declared"].as_bool() == Some(true);
    if declared {
        println!("  ✅ 原创已声明");
        StepOutcome::Applied("yuanzhuang-declared")
    } else {
        println!("  ⚠ 原创声明未生效 — 行内仍显示'未声明'");
        StepOutcome::Unverified("yuanzhuang-still-undeclared")
    }
}

pub async fn step_zanshang(io: &impl StepIo) -> StepOutcome {
    println!("▶ 赞赏...");
    let state = io.eval_json(REWARD_STATE_SCRIPT).await;
    let declared = state["declared"].as_bool() == Some(true);
    let enabled = state["enabled"].as_bool() == Some(true);
    let tips = state["tips"].as_str().unwrap_or("");
    if enabled {
        println!("  ✅ 赞赏已开启（无需重复操作, state='{tips}'）");
        return StepOutcome::AlreadyDone("zanshang-already-on");
    }
    if !declared {
        println!("  ⚠ 赞赏未开启 — 需先声明原创（声明原创后才可开启赞赏）");
        return StepOutcome::Skipped("zanshang-needs-original");
    }
    // 真实产品路径（实测）：原创弹窗内的赞赏区域默认隐藏，正确入口是设置行
    // .js_reward_open 开关 → 赞赏设置弹窗 → 确定；落库靠后续保存草稿。
    let opened = io.eval_json(ZANSHANG_OPEN_SCRIPT).await;
    if opened["clicked"].as_bool() != Some(true) {
        println!("  ⚠ 赞赏开关不可点击 — skipping");
        return StepOutcome::Skipped("zanshang-switch-not-clickable");
    }
    println!("    点击设置行赞赏开关: true");
    io.sleep_ms(2_000).await;
    let confirm = io.eval_json(ZANSHANG_CONFIRM_SCRIPT).await;
    println!(
        "    弹窗操作: {}",
        confirm["actions"].as_str().unwrap_or("(none)")
    );
    io.sleep_ms(1_500).await;
    let after = io.eval_json(REWARD_STATE_SCRIPT).await;
    let after_tips = after["tips"].as_str().unwrap_or("");
    if after["enabled"].as_bool() == Some(true) {
        println!("  ✅ 赞赏已开启 (state='{after_tips}')");
        StepOutcome::Applied("zanshang-enabled")
    } else {
        println!("  ⚠ 赞赏未开启 (state='{after_tips}') — 请到后台人工核对");
        StepOutcome::Unverified("zanshang-still-off")
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

pub async fn step_liuyan(io: &impl StepIo) -> StepOutcome {
    println!("▶ 留言...");
    // 状态优先：input.js_interaction_setting 的 checked 是真实状态。
    // 旧版盲点"确定"会命中其它未关闭弹窗（曾把原创弹窗的确定当留言确认），
    // 所以确认按钮必须限定在含"留言"的可见弹窗内。
    let enable = io.eval_json(COMMENT_ENABLE_SCRIPT).await;
    match enable["state"].as_str() {
        Some("already-on") => {
            println!("  ✅ 留言已开启（无需重复操作）");
            return StepOutcome::AlreadyDone("liuyan-already-on");
        }
        Some("clicked") => {
            println!("    点击留言开关: true");
        }
        _ => {
            println!("  ⚠ 留言开关未找到 — skipping");
            return StepOutcome::Skipped("liuyan-switch-not-found");
        }
    }
    io.sleep_ms(1_500).await;
    let confirm = io.eval_json(COMMENT_CONFIRM_SCRIPT).await;
    println!("    弹窗确认: {:?}", confirm["confirm"].as_bool());
    io.sleep_ms(800).await;
    let state = io.eval_json(COMMENT_STATE_SCRIPT).await;
    if state["on"].as_bool() == Some(true) {
        println!("  ✅ 留言已开启");
        StepOutcome::Applied("liuyan-enabled")
    } else {
        println!("  ⚠ 留言未开启 — 请到后台人工核对");
        StepOutcome::Unverified("liuyan-still-off")
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

pub async fn step_chuangzuo(io: &impl StepIo) -> StepOutcome {
    println!("▶ 创作来源...");
    let _ = io
        .eval_text("window.scrollTo(0, document.body.scrollHeight)")
        .await;
    io.sleep_ms(500).await;

    // Open the 创作来源 picker. The live editor DOM shows a checkbox label
    // with a dedicated clickable wrapper .js_claim_source_desc inside it.
    // We must click that exact wrapper; searching for any "未添加" text hits
    // other rows (e.g. 合集) because their labels share the same structure.
    let ok = io.eval_bool(
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
        ).await;
    println!("    click '创作来源' 未添加: {ok}");
    if !ok {
        println!("  ⚠ '创作来源' picker not found — skipping");
        return StepOutcome::Skipped("chuangzuo-picker-not-found");
    }
    io.sleep_ms(2_000).await;

    // Select option value="4" (个人观点，仅供参考). The picker is a radio
    // group; using the input value is more reliable than text matching.
    let selected = io.eval_text(
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
        ).await;

    if selected.is_empty() {
        let _ = io.close_dialog().await;
        println!("  ⚠ 创作来源 — 未找到可选项，跳过");
        return StepOutcome::Skipped("chuangzuo-no-option");
    }
    println!("    select '{selected}': true");
    io.sleep_ms(500).await;

    // 确认按钮必须限定在创作来源 picker 容器内（以 radio 定位），
    // 全局盲点"确认/确定"会命中其它未关闭弹窗。
    // 微信编辑器的 Vue 模型忽略 JS 合成点击（isTrusted=false），脚本只取坐标，
    // 真正点击走 CDP 可信鼠标事件。
    let confirm_rect = io.eval_json(CHUANGZUO_CONFIRM_SCRIPT).await;
    let ok3 = if confirm_rect["found"].as_bool() == Some(true) {
        let x = confirm_rect["x"].as_f64().unwrap_or(0.0);
        let y = confirm_rect["y"].as_f64().unwrap_or(0.0);
        io.click_at(x, y).await;
        true
    } else {
        false
    };
    println!("    click '确认': {ok3}");
    io.sleep_ms(1_500).await;

    // Verify the setting stuck.
    let state = io.eval_text(
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
        ).await;
    println!("    创作来源 state: '{state}'");

    if ok3 && state.contains("个人观点") {
        println!("  ✅ 创作来源");
        StepOutcome::Applied("chuangzuo-applied")
    } else if ok3 {
        println!("  ⚠ 创作来源 已确认但状态未识别 (state='{state}')");
        StepOutcome::Unverified("chuangzuo-state-unreadable")
    } else {
        println!("  ⚠ 创作来源 '确认' not found");
        StepOutcome::Unverified("chuangzuo-confirm-not-found")
    }
}

/// 一次复核尝试的结局。
enum FuchaRead {
    /// 编辑器外壳都没出来（重载失败／超时）。
    PageNotLoaded,
    /// 设置区始终没渲染，这时读到的空串没有意义。
    NotRendered,
    /// 读到一份可用快照。
    State(serde_json::Value),
}

/// 重载编辑器 → 等设置区 → 读到两次一致的快照。
///
/// 两道门分开是有原因的：`#js_original`（标题框）比右侧设置区早得多，只等它就等于
/// 在设置还没画完时就读。`ready` 由 `FUCHA_SCRIPT` 一起返回，未见 `ready` 的读数直接作废。
async fn read_fucha_once(io: &impl StepIo) -> FuchaRead {
    let _ = io.eval_text("location.reload()").await;
    let mut loaded = false;
    for _ in 0..30 {
        io.sleep_ms(500).await;
        let ok = io
            .eval_bool("document.querySelector('#js_original')!==null")
            .await;
        if ok {
            loaded = true;
            break;
        }
    }
    if !loaded {
        return FuchaRead::PageNotLoaded;
    }
    let mut prev = String::new();
    let mut rendered = false;
    for read in 0..20 {
        io.sleep_ms(700).await;
        let raw = io.eval_text(FUCHA_SCRIPT).await;
        let parsed: serde_json::Value =
            serde_json::from_str(&raw).unwrap_or(serde_json::Value::Null);
        // 旧页面没有 `ready` 字段时不拦（保持原有行为），有就必须为真。
        if !parsed["ready"].as_bool().unwrap_or(true) {
            continue;
        }
        rendered = true;
        if read > 0 && raw == prev {
            return FuchaRead::State(parsed);
        }
        prev = raw;
    }
    if rendered {
        FuchaRead::State(serde_json::from_str(&prev).unwrap_or(serde_json::Value::Null))
    } else {
        FuchaRead::NotRendered
    }
}

/// 快照里已经确认落库的项数（0~4）。0 就意味着「四条一起读空」，
/// 这是要再读一遍的信号，而不是「都没保存」的结论。
fn landed_settings(state: &serde_json::Value) -> usize {
    let original = state["original"].as_str().unwrap_or("");
    let reward = state["reward"].as_str().unwrap_or("");
    let source = state["source"].as_str().unwrap_or("");
    let comment = state["comment"].as_str().unwrap_or("");
    let mut landed = 0;
    if original.starts_with("declared") {
        landed += 1;
    }
    if !reward.is_empty() && !reward.contains("不开启") {
        landed += 1;
    }
    if !source.is_empty() {
        landed += 1;
    }
    if comment == "on" {
        landed += 1;
    }
    landed
}

pub async fn step_fucha(io: &impl StepIo, expected_title: Option<&str>) -> StepOutcome {
    println!("▶ 复核后台设置（重新加载编辑器）...");
    // 只信保存后的服务器状态：重载编辑器，读设置行的真实文案。
    // 点击日志和内存态都可能是假成功（弹窗遮挡、校验拦截），重载后的
    // 页面状态才是微信后台真正落库的结果。
    //
    // `expected_title` 只用于**诊断**，不参与「是否落库」的判定：保存后页面未必落在
    // 刚编辑那一篇上，四项会整齐地一起读空。把重载后的标题和期望标题比一下，就能把
    // 「真没保存」和「复核页跑错篇」分开说，而不是让人对着四条 ⚠ 猜。读不到标题时不作判断。
    //
    // 2026-11-26 连推两次都「四项全 ⚠」，而同一篇草稿紧接着 `configure` 却四项全绿：
    // 后台早存上了，是这一读发生得太早。所以四项整齐读空时**再读一遍**（重载 + 重新等），
    // 不为一次读数下「没保存」的结论。
    let state;
    let mut attempt = 1;
    loop {
        match read_fucha_once(io).await {
            FuchaRead::PageNotLoaded => {
                println!("  ⚠ 复核页面未加载 — 请到后台人工核对");
                return StepOutcome::Unverified("fucha-page-not-loaded");
            }
            FuchaRead::NotRendered => {
                if attempt >= 2 {
                    println!("  ⚠ 复核页面没渲染出设置区（不是「没保存」）— 请到后台人工核对");
                    return StepOutcome::Unverified("fucha-settings-not-rendered");
                }
                println!("    设置区还没渲染出来，3 秒后再读一遍");
                io.sleep_ms(3_000).await;
            }
            FuchaRead::State(read) => {
                if landed_settings(&read) > 0 || attempt >= 2 {
                    state = read;
                    break;
                }
                println!("    四项都读空，5 秒后重载再读一遍（保存可能刚提交）");
                io.sleep_ms(5_000).await;
            }
        }
        attempt += 1;
    }
    let original = state["original"].as_str().unwrap_or("");
    let reward = state["reward"].as_str().unwrap_or("");
    let source = state["source"].as_str().unwrap_or("");
    let comment = state["comment"].as_str().unwrap_or("");
    let title = state["title"].as_str().unwrap_or("");
    let ok_original = original.starts_with("declared");
    let ok_reward = !reward.is_empty() && !reward.contains("不开启");
    let ok_source = !source.is_empty();
    let ok_comment = comment == "on";
    if ok_original {
        println!("  ✅ 原创已落库");
    } else {
        println!("  ⚠ 原创未保存到后台");
    }
    if ok_reward {
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
    if ok_source {
        println!("  ✅ 创作来源已落库 ({source})");
    } else {
        println!("  ⚠ 创作来源未保存到后台");
    }
    match comment {
        "on" => println!("  ✅ 留言已落库"),
        "off" => println!("  ⚠ 留言未保存到后台"),
        _ => println!("  ℹ 留言状态未识别"),
    }
    if ok_original && ok_reward && ok_source && ok_comment {
        StepOutcome::Applied("fucha-all-persisted")
    } else if let Some(expected) = expected_title {
        // 四项一起读空 + 标题对不上 = 重载落到了另一篇（或空白草稿）上。
        // 这是「复核页跑错篇」，不是「没保存」，两者下一步动作完全不同。
        let nothing_read = !ok_original && !ok_reward && !ok_source && comment != "on";
        if nothing_read && !same_title(title, expected) {
            println!(
                "  ⚠ 重载后的编辑器不是刚编辑的那篇（标题「{}」，期望「{}」）",
                if title.is_empty() { "空" } else { title },
                expected
            );
            println!("     └ 这份页面的设置不必当真；到后台或跑 configure 才能确认落库");
            return StepOutcome::Unverified("fucha-reloaded-different-draft");
        }
        StepOutcome::Unverified("fucha-some-missing")
    } else {
        StepOutcome::Unverified("fucha-some-missing")
    }
}

/// 标题比较只吃「非空白字符」：微信会把标题里的空格渲染成不换行空格，
/// 直接 `==` 必然不等。
fn same_title(actual: &str, expected: &str) -> bool {
    let norm = |s: &str| -> String { s.chars().filter(|c| !c.is_whitespace()).collect() };
    !actual.is_empty() && norm(actual) == norm(expected)
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
    // 标题只用于「重载后是不是同一篇」的诊断，读不到就给空串。
    var titleEl = document.getElementById('js_title');
    var title = titleEl ? norm(titleEl.textContent) : '';
    // 设置区是否已经渲染出来：复核要读的三块（赞赏行 / 创作来源 / 留言开关）任一存在即算。
    // 保存后立刻 reload，标题框（#js_original）会先出现，设置行还在等 XHR；这时读到的
    // 全是空串，会被下游当成「四项都没保存」（2026-11-26 实测：同一篇草稿紧接
    // `configure` 却四项全绿，说明后台早就存上了，只是复核读得太早）。
    var ready = !!(rewardEl || sourceEl || commentCb);
    return JSON.stringify({ready: ready, original: original, reward: reward, source: source, comment: comment, title: title});
})()"#;

pub async fn step_baocun(io: &impl StepIo) -> StepOutcome {
    println!("▶ 保存草稿...");
    // 任何残留弹窗都会挡住保存按钮的点击（picker 未关曾导致整轮设置不落库），
    // 保存前先关掉所有可见弹窗。
    for _ in 0..3 {
        let closed = io.eval_u64(CLOSE_VISIBLE_DIALOGS_SCRIPT).await;
        if closed == 0 {
            break;
        }
        println!("    关闭残留弹窗: {closed} 个");
        io.sleep_ms(500).await;
    }
    let rect = io.eval_json(SAVE_DRAFT_RECT_SCRIPT).await;
    let clicked = match rect {
        v if v["found"].as_bool() == Some(true) => {
            let x = v["x"].as_f64().unwrap_or(0.0);
            let y = v["y"].as_f64().unwrap_or(0.0);
            let label = v["label"].as_str().unwrap_or("保存");
            println!("    click '{label}': true");
            io.click_at(x, y).await;
            true
        }
        _ => {
            println!("    click '保存为草稿': false");
            false
        }
    };
    if !clicked {
        println!("  ⚠ 保存按钮未找到 — 后台设置可能仍未持久化");
        return StepOutcome::Skipped("baocun-save-button-not-found");
    }

    let mut saved = false;
    for _ in 0..12 {
        io.sleep_ms(500).await;
        saved = io.eval_bool(SAVE_DRAFT_STATE_SCRIPT).await;
        if saved {
            break;
        }
    }
    if saved {
        println!("  ✅ 草稿已保存");
        StepOutcome::Applied("baocun-saved")
    } else {
        println!("  ⚠ 已点击保存，但未识别到保存成功提示 — 请到后台人工核对");
        StepOutcome::Unverified("baocun-no-confirmation")
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
async fn autodetect_preview_wxname(io: &impl StepIo) -> Option<String> {
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
    let j = io.eval_json(js).await;
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

/// Build the `preusername_list` payload with proper JSON encoding.
/// Hand-rolled `format!` silently drops quotes from the recipient name;
/// serde_json escapes them so WeChat receives the literal value.
fn preview_recipient_payload(to_wxname: &str) -> String {
    serde_json::json!({ "preusername": [to_wxname] }).to_string()
}

pub async fn step_yulan(io: &impl StepIo, to_wxname: Option<&str>) -> StepOutcome {
    println!("▶ 预览 (cookie 接口)...");
    // Pull token + appmsgid straight from the editor URL — no UI click needed.
    // WeChat drops synthetic clicks (isTrusted===false), so instead we call the
    // real backend endpoint with our live session cookie (same-origin fetch).
    let info = io
        .eval_text(
            r#"(() => {
            var u = new URL(location.href);
            return JSON.stringify({
                token: u.searchParams.get('token') || '',
                appmsgid: u.searchParams.get('appmsgid') || u.searchParams.get('id') || ''
            });
        })()"#,
        )
        .await;
    let v: serde_json::Value = serde_json::from_str(&info).unwrap_or(serde_json::Value::Null);
    let token = v["token"].as_str().unwrap_or("").to_owned();
    let appmsgid = v["appmsgid"].as_str().unwrap_or("").to_owned();
    if token.is_empty() || appmsgid.is_empty() {
        println!("  ⚠ 无法从编辑器页获取 token/appmsgid (token='{token}' appmsgid='{appmsgid}')");
        return StepOutcome::Skipped("yulan-no-token");
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
        None => match autodetect_preview_wxname(io).await {
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
                return StepOutcome::Skipped("yulan-no-recipient");
            }
        },
    };
    let preusername_list = preview_recipient_payload(&to_wxname);
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
    let resp = io.eval_text(&js).await;
    println!("    preview resp: {resp}");
    let Ok(j) = serde_json::from_str::<serde_json::Value>(&resp) else {
        return StepOutcome::Unverified("yulan-unparsable-response");
    };
    let status = j["status"].as_i64().unwrap_or(-1);
    let body = j["body"].as_str().unwrap_or("").to_owned();
    println!("    http status: {status}");
    let Ok(b) = serde_json::from_str::<serde_json::Value>(&body) else {
        println!("    (non-json body): {body}");
        return StepOutcome::Unverified("yulan-non-json-body");
    };
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
        StepOutcome::Applied("yulan-sent")
    } else {
        println!("  ⚠ 预览返回 ret={ret} err={err}");
        StepOutcome::Unverified("yulan-ret-nonzero")
    }
}

/// Insert a WeChat article template by name via CDP automation.
///
/// This function focuses the editor, opens the template menu, searches for
/// the given template name, and clicks to insert it.
pub async fn step_moban(io: &impl StepIo, template_name: &str) -> StepOutcome {
    println!("▶ 模板插入 ({template_name})...");
    let js = moban_script(template_name);
    if io.eval_bool(&js).await {
        io.sleep_ms(1_500).await;
        println!("  ✅ 模板插入");
        StepOutcome::Applied("moban-inserted")
    } else {
        println!("  ⚠ 模板插入失败或模板未找到 — skipping");
        StepOutcome::Skipped("moban-not-found")
    }
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
        SAVE_DRAFT_STATE_SCRIPT, StepIo, StepOutcome, ZANSHANG_CONFIRM_SCRIPT,
        ZANSHANG_OPEN_SCRIPT, moban_script, original_script, preview_recipient_payload,
        step_baocun, step_chuangzuo, step_fucha, step_liuyan, step_moban, step_yuanzhuang,
        step_yulan, step_zanshang,
    };
    use std::cell::{Cell, RefCell};

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

    #[test]
    fn preview_recipient_payload_uses_json_encoding() {
        let payload = preview_recipient_payload("wx\"name\\id");
        assert_eq!(payload, r#"{"preusername":["wx\"name\\id"]}"#);
    }

    // ───────────────────────── 编排分支（假浏览器） ─────────────────────────
    //
    // 上面那些测试断言的是「JS 字符串里有没有某段代码」，只能证明脚本被改过，
    // 证明不了「后台返回 X 时会走哪条分支」。线上真正翻车的是后者：弹窗没关、
    // 校验拦截、按钮找不到。下面用内存假体把这些分支逐一钉死。

    /// 一次脚本调用的应答。
    enum Reply {
        /// 脚本返回的原始文本（JSON 也用这个，走 `eval_text` 再解析）。
        Text(&'static str),
        Bool(bool),
        U64(u64),
    }

    struct Rule {
        key: String,
        /// 按调用次数依次取用；用尽后重复最后一条。
        replies: Vec<Reply>,
        cursor: Cell<usize>,
    }

    /// 内存假体：按脚本内容匹配应答，记录每次调用、等待与点击。
    ///
    /// 匹配规则是「脚本包含 key」。多个规则同时命中会 panic——微信后台有几个脚本
    /// 读同一块 DOM（赞赏状态脚本和开关脚本都碰 `.js_reward_open`），静默串味比
    /// 直接失败更难查。
    #[derive(Default)]
    struct FakeIo {
        rules: Vec<Rule>,
        calls: RefCell<Vec<String>>,
        slept: RefCell<Vec<u64>>,
        clicks: RefCell<Vec<(f64, f64)>>,
        closes: Cell<u32>,
    }

    impl FakeIo {
        fn new() -> Self {
            Self::default()
        }

        /// 该脚本永远返回同一条应答。
        fn with(mut self, key: &str, reply: Reply) -> Self {
            self.rules.push(Rule {
                key: key.to_owned(),
                replies: vec![reply],
                cursor: Cell::new(0),
            });
            self
        }

        /// 该脚本按调用次数依次返回不同应答，用尽后重复最后一条。
        fn with_seq(mut self, key: &str, replies: Vec<Reply>) -> Self {
            assert!(!replies.is_empty(), "with_seq 至少要一条应答");
            self.rules.push(Rule {
                key: key.to_owned(),
                replies,
                cursor: Cell::new(0),
            });
            self
        }

        fn next_reply(&self, script: &str, want: &str) -> &Reply {
            let hits: Vec<&Rule> = self
                .rules
                .iter()
                .filter(|r| script.contains(&r.key))
                .collect();
            assert!(
                hits.len() <= 1,
                "{want}: {} 条规则同时命中同一个脚本，假体无法判断",
                hits.len()
            );
            let rule = match hits.first() {
                Some(r) => *r,
                None => panic!(
                    "{want} 收到未预期的脚本: {}",
                    script.chars().take(80).collect::<String>()
                ),
            };
            let i = rule.cursor.get().min(rule.replies.len() - 1);
            rule.cursor.set(i + 1);
            &rule.replies[i]
        }

        fn call_count(&self) -> usize {
            self.calls.borrow().len()
        }

        fn slept(&self) -> Vec<u64> {
            self.slept.borrow().clone()
        }

        fn clicks(&self) -> Vec<(f64, f64)> {
            self.clicks.borrow().clone()
        }
    }

    impl StepIo for FakeIo {
        async fn eval_text(&self, script: &str) -> String {
            match self.next_reply(script, "eval_text") {
                Reply::Text(s) => {
                    self.calls.borrow_mut().push("text".to_owned());
                    (*s).to_owned()
                }
                Reply::Bool(_) | Reply::U64(_) => {
                    panic!("该脚本配的是布尔/数字应答，不能按文本读")
                }
            }
        }

        async fn eval_bool(&self, script: &str) -> bool {
            match self.next_reply(script, "eval_bool") {
                Reply::Bool(b) => {
                    self.calls.borrow_mut().push("bool".to_owned());
                    *b
                }
                _ => panic!("该脚本配的不是布尔应答"),
            }
        }

        async fn eval_u64(&self, script: &str) -> u64 {
            match self.next_reply(script, "eval_u64") {
                Reply::U64(n) => {
                    self.calls.borrow_mut().push("u64".to_owned());
                    *n
                }
                _ => panic!("该脚本配的不是数字应答"),
            }
        }

        async fn click_at(&self, x: f64, y: f64) {
            self.clicks.borrow_mut().push((x, y));
        }

        async fn close_dialog(&self) -> bool {
            self.closes.set(self.closes.get() + 1);
            true
        }

        async fn sleep_ms(&self, ms: u64) {
            self.slept.borrow_mut().push(ms);
        }
    }

    /// 创作来源：先把编辑区滚到底部，让设置区可见。
    const SCROLL_MARK: &str = "window.scrollTo(0, document.body.scrollHeight)";
    /// 创作来源 picker 打开脚本的特征串（脚本是内联字面量，没有常量名）。
    const CHUANGZUO_OPEN_MARK: &str = ".js_claim_source_desc";
    /// 选项脚本的特征串。
    const CHUANGZUO_SELECT_MARK: &str = "个人观点，仅供参考";
    /// 校验脚本的特征串。
    const CHUANGZUO_STATE_MARK: &str = ".js_claim_source_selected";
    /// 预览：取 token/appmsgid 的脚本。
    const YULAN_INFO_MARK: &str = "searchParams.get('token')";
    /// 预览：真正发请求的脚本。
    const YULAN_SEND_MARK: &str = "operate_appmsg";
    /// 复核：编辑器重载完成的判定脚本。
    const RELOADED_MARK: &str = "js_original')!==null";

    // ── 赞赏 ──

    #[tokio::test]
    async fn zanshang_already_on_makes_no_write_calls() {
        let io = FakeIo::new().with(
            REWARD_STATE_SCRIPT,
            Reply::Text(r#"{"declared":true,"enabled":true,"tips":"已开启"}"#),
        );
        let outcome = step_zanshang(&io).await;
        assert_eq!(outcome, StepOutcome::AlreadyDone("zanshang-already-on"));
        // 已经开着就只读一次状态：不点开关、不开弹窗、不等待。
        assert_eq!(io.call_count(), 1);
        assert!(io.slept().is_empty());
    }

    #[tokio::test]
    async fn zanshang_requires_original_first() {
        let io = FakeIo::new().with(
            REWARD_STATE_SCRIPT,
            Reply::Text(r#"{"declared":false,"enabled":false}"#),
        );
        let outcome = step_zanshang(&io).await;
        assert_eq!(outcome, StepOutcome::Skipped("zanshang-needs-original"));
        assert_eq!(io.call_count(), 1);
        assert!(io.slept().is_empty());
    }

    #[tokio::test]
    async fn zanshang_skips_when_switch_not_clickable() {
        let io = FakeIo::new()
            .with(
                REWARD_STATE_SCRIPT,
                Reply::Text(r#"{"declared":true,"enabled":false}"#),
            )
            .with(ZANSHANG_OPEN_SCRIPT, Reply::Text(r#"{"clicked":false}"#));
        let outcome = step_zanshang(&io).await;
        assert_eq!(
            outcome,
            StepOutcome::Skipped("zanshang-switch-not-clickable")
        );
        assert_eq!(io.call_count(), 2);
        // 开关都没点上，不该白等。
        assert!(io.slept().is_empty());
    }

    #[tokio::test]
    async fn zanshang_reports_applied_only_after_state_self_check() {
        let io = FakeIo::new()
            .with_seq(
                REWARD_STATE_SCRIPT,
                vec![
                    Reply::Text(r#"{"declared":true,"enabled":false}"#),
                    Reply::Text(r#"{"enabled":true,"tips":"已开启"}"#),
                ],
            )
            .with(ZANSHANG_OPEN_SCRIPT, Reply::Text(r#"{"clicked":true}"#))
            .with(
                ZANSHANG_CONFIRM_SCRIPT,
                Reply::Text(r#"{"actions":"confirm"}"#),
            );
        let outcome = step_zanshang(&io).await;
        assert_eq!(outcome, StepOutcome::Applied("zanshang-enabled"));
        // 开弹窗前等 2s，确认后等 1.5s 再自检。
        assert_eq!(io.slept(), vec![2_000, 1_500]);
    }

    #[tokio::test]
    async fn zanshang_does_not_claim_success_when_state_stays_off() {
        let io = FakeIo::new()
            .with_seq(
                REWARD_STATE_SCRIPT,
                vec![
                    Reply::Text(r#"{"declared":true,"enabled":false}"#),
                    Reply::Text(r#"{"enabled":false,"tips":"未开启"}"#),
                ],
            )
            .with(ZANSHANG_OPEN_SCRIPT, Reply::Text(r#"{"clicked":true}"#))
            .with(
                ZANSHANG_CONFIRM_SCRIPT,
                Reply::Text(r#"{"actions":"none"}"#),
            );
        let outcome = step_zanshang(&io).await;
        assert_eq!(outcome, StepOutcome::Unverified("zanshang-still-off"));
    }

    // ── 留言 ──

    #[tokio::test]
    async fn liuyan_already_on_makes_no_write_calls() {
        let io = FakeIo::new().with(
            COMMENT_ENABLE_SCRIPT,
            Reply::Text(r#"{"state":"already-on"}"#),
        );
        assert_eq!(
            step_liuyan(&io).await,
            StepOutcome::AlreadyDone("liuyan-already-on")
        );
        assert_eq!(io.call_count(), 1);
        assert!(io.slept().is_empty());
    }

    #[tokio::test]
    async fn liuyan_skips_when_switch_missing() {
        let io = FakeIo::new().with(
            COMMENT_ENABLE_SCRIPT,
            Reply::Text(r#"{"state":"no-switch"}"#),
        );
        assert_eq!(
            step_liuyan(&io).await,
            StepOutcome::Skipped("liuyan-switch-not-found")
        );
        assert_eq!(io.call_count(), 1);
    }

    #[tokio::test]
    async fn liuyan_confirms_then_verifies() {
        let io = FakeIo::new()
            .with(COMMENT_ENABLE_SCRIPT, Reply::Text(r#"{"state":"clicked"}"#))
            .with(COMMENT_CONFIRM_SCRIPT, Reply::Text(r#"{"confirm":true}"#))
            .with(COMMENT_STATE_SCRIPT, Reply::Text(r#"{"on":true}"#));
        assert_eq!(
            step_liuyan(&io).await,
            StepOutcome::Applied("liuyan-enabled")
        );
        assert_eq!(io.slept(), vec![1_500, 800]);
    }

    // ── 原创声明 ──

    #[tokio::test]
    async fn yuanzhuang_already_declared_stops_immediately() {
        let io = FakeIo::new().with(
            super::OPEN_ORIGINAL_DIALOG_SCRIPT,
            Reply::Text("already-declared"),
        );
        assert_eq!(
            step_yuanzhuang(&io).await,
            StepOutcome::AlreadyDone("yuanzhuang-already-declared")
        );
        assert_eq!(io.call_count(), 1);
        assert!(io.slept().is_empty());
    }

    #[tokio::test]
    async fn yuanzhuang_skips_without_entry() {
        let io = FakeIo::new().with(super::OPEN_ORIGINAL_DIALOG_SCRIPT, Reply::Text(""));
        assert_eq!(
            step_yuanzhuang(&io).await,
            StepOutcome::Skipped("yuanzhuang-entry-not-found")
        );
    }

    #[tokio::test]
    async fn yuanzhuang_confirms_and_verifies() {
        let configure = original_script(ORIGINAL_CONFIGURE_SCRIPT);
        let verify = original_script(ORIGINAL_VERIFY_SCRIPT);
        let io = FakeIo::new()
            .with(super::OPEN_ORIGINAL_DIALOG_SCRIPT, Reply::Text("clicked"))
            .with(
                configure.as_str(),
                Reply::Text(r#"{"agreed":true,"confirm":true}"#),
            )
            .with(
                verify.as_str(),
                Reply::Text(r#"{"dialogOpen":false,"declared":true}"#),
            );
        assert_eq!(
            step_yuanzhuang(&io).await,
            StepOutcome::Applied("yuanzhuang-declared")
        );
        // 等作者信息加载 2s，确认后 1.2s 再验证。
        assert_eq!(io.slept(), vec![2_000, 1_200]);
    }

    #[tokio::test]
    async fn yuanzhuang_retries_when_wechat_rejects_the_author_field() {
        let configure = original_script(ORIGINAL_CONFIGURE_SCRIPT);
        let verify = original_script(ORIGINAL_VERIFY_SCRIPT);
        let io = FakeIo::new()
            .with(super::OPEN_ORIGINAL_DIALOG_SCRIPT, Reply::Text("clicked"))
            .with(
                configure.as_str(),
                Reply::Text(r#"{"agreed":true,"confirm":true}"#),
            )
            .with_seq(
                verify.as_str(),
                vec![
                    // 第一次被「作者不能为空且不超过8个字」拦下，弹窗还开着。
                    Reply::Text(r#"{"dialogOpen":true,"error":"作者不能为空且不超过8个字"}"#),
                    // 重试后弹窗关闭。
                    Reply::Text(r#"{"dialogOpen":false,"declared":true}"#),
                ],
            );
        assert_eq!(
            step_yuanzhuang(&io).await,
            StepOutcome::Applied("yuanzhuang-declared")
        );
        // 2s（等加载） → 1.2s（首次确认） → 3s（被拦后等重试） → 1.2s（重试确认）
        assert_eq!(io.slept(), vec![2_000, 1_200, 3_000, 1_200]);
    }

    #[tokio::test]
    async fn yuanzhuang_never_claims_success_without_final_state() {
        let configure = original_script(ORIGINAL_CONFIGURE_SCRIPT);
        let verify = original_script(ORIGINAL_VERIFY_SCRIPT);
        let io = FakeIo::new()
            .with(super::OPEN_ORIGINAL_DIALOG_SCRIPT, Reply::Text("clicked"))
            .with(
                configure.as_str(),
                Reply::Text(r#"{"agreed":true,"confirm":true}"#),
            )
            // 弹窗始终不关，最终验证也说没声明。
            .with(verify.as_str(), Reply::Text(r#"{"dialogOpen":true}"#));
        assert_eq!(
            step_yuanzhuang(&io).await,
            StepOutcome::Unverified("yuanzhuang-still-undeclared")
        );
    }

    // ── 创作来源 ──

    #[tokio::test]
    async fn chuangzuo_skips_when_picker_missing() {
        let io = FakeIo::new()
            .with(SCROLL_MARK, Reply::Text(""))
            .with(CHUANGZUO_OPEN_MARK, Reply::Bool(false));
        let outcome = step_chuangzuo(&io).await;
        assert_eq!(outcome, StepOutcome::Skipped("chuangzuo-picker-not-found"));
        assert_eq!(io.slept(), vec![500]);
    }

    #[tokio::test]
    async fn chuangzuo_closes_picker_when_no_option_matches() {
        let io = FakeIo::new()
            .with(SCROLL_MARK, Reply::Text(""))
            .with(CHUANGZUO_OPEN_MARK, Reply::Bool(true))
            .with(CHUANGZUO_SELECT_MARK, Reply::Text(""));
        let outcome = step_chuangzuo(&io).await;
        assert_eq!(outcome, StepOutcome::Skipped("chuangzuo-no-option"));
        assert_eq!(
            io.closes.get(),
            1,
            "选不到选项时必须把 picker 关掉，否则挡住后续按钮"
        );
    }

    #[tokio::test]
    async fn chuangzuo_clicks_confirm_by_coordinates() {
        let io = FakeIo::new()
            .with(SCROLL_MARK, Reply::Text(""))
            .with(CHUANGZUO_OPEN_MARK, Reply::Bool(true))
            .with(CHUANGZUO_SELECT_MARK, Reply::Text("个人观点，仅供参考"))
            .with(
                CHUANGZUO_CONFIRM_SCRIPT,
                Reply::Text(r#"{"found":true,"x":120.5,"y":340.0}"#),
            )
            .with(CHUANGZUO_STATE_MARK, Reply::Text("个人观点，仅供参考"));
        assert_eq!(
            step_chuangzuo(&io).await,
            StepOutcome::Applied("chuangzuo-applied")
        );
        // 微信编辑器忽略 JS 合成点击，必须走坐标可信点击。
        assert_eq!(io.clicks(), vec![(120.5, 340.0)]);
    }

    // ── 保存草稿 ──

    #[tokio::test]
    async fn baocun_reports_skip_when_save_button_missing() {
        let io = FakeIo::new()
            .with(CLOSE_VISIBLE_DIALOGS_SCRIPT, Reply::U64(0))
            .with(SAVE_DRAFT_RECT_SCRIPT, Reply::Text(r#"{"found":false}"#));
        assert_eq!(
            step_baocun(&io).await,
            StepOutcome::Skipped("baocun-save-button-not-found")
        );
        assert!(io.clicks().is_empty());
    }

    #[tokio::test]
    async fn baocun_closes_leftover_dialogs_before_clicking() {
        let io = FakeIo::new()
            .with_seq(
                CLOSE_VISIBLE_DIALOGS_SCRIPT,
                vec![Reply::U64(2), Reply::U64(0)],
            )
            .with(
                SAVE_DRAFT_RECT_SCRIPT,
                Reply::Text(r#"{"found":true,"x":8.0,"y":9.0,"label":"保存"}"#),
            )
            .with(SAVE_DRAFT_STATE_SCRIPT, Reply::Bool(true));
        assert_eq!(step_baocun(&io).await, StepOutcome::Applied("baocun-saved"));
        assert_eq!(io.clicks(), vec![(8.0, 9.0)]);
        // 关掉一轮弹窗后等 0.5s；点保存后第一轮 0.5s 就看到了成功提示。
        assert_eq!(io.slept(), vec![500, 500]);
    }

    #[tokio::test]
    async fn baocun_does_not_claim_success_without_toast() {
        let io = FakeIo::new()
            .with(CLOSE_VISIBLE_DIALOGS_SCRIPT, Reply::U64(0))
            .with(
                SAVE_DRAFT_RECT_SCRIPT,
                Reply::Text(r#"{"found":true,"x":1.0,"y":2.0}"#),
            )
            .with(SAVE_DRAFT_STATE_SCRIPT, Reply::Bool(false));
        assert_eq!(
            step_baocun(&io).await,
            StepOutcome::Unverified("baocun-no-confirmation")
        );
        // 轮询 12 次都失败后放弃，不无限等。
        assert_eq!(io.slept().len(), 12);
    }

    // ── 复核 ──

    #[tokio::test]
    async fn fucha_unverified_when_page_never_reloads() {
        let io = FakeIo::new()
            .with("location.reload()", Reply::Text(""))
            .with(RELOADED_MARK, Reply::Bool(false));
        assert_eq!(
            step_fucha(&io, None).await,
            StepOutcome::Unverified("fucha-page-not-loaded")
        );
        // 30 次 × 0.5s 之后就放弃。
        assert_eq!(io.slept().len(), 30);
    }

    #[tokio::test]
    async fn fucha_applied_only_when_every_setting_persisted() {
        let io = FakeIo::new()
            .with("location.reload()", Reply::Text(""))
            .with(RELOADED_MARK, Reply::Bool(true))
            .with(
                FUCHA_SCRIPT,
                Reply::Text(
                    r#"{"original":"declared","reward":"已开启","source":"个人观点，仅供参考","comment":"on"}"#,
                ),
            );
        assert_eq!(
            step_fucha(&io, None).await,
            StepOutcome::Applied("fucha-all-persisted")
        );
    }

    #[tokio::test]
    async fn fucha_flags_partial_persistence() {
        let io = FakeIo::new()
            .with("location.reload()", Reply::Text(""))
            .with(RELOADED_MARK, Reply::Bool(true))
            .with(
                FUCHA_SCRIPT,
                Reply::Text(
                    r#"{"original":"declared","reward":"","source":"个人观点，仅供参考","comment":"on"}"#,
                ),
            );
        assert_eq!(
            step_fucha(&io, None).await,
            StepOutcome::Unverified("fucha-some-missing")
        );
    }

    /// 2026-11-25 的真实形态：保存后重载落到空白草稿上，四项一起读空。
    /// 有期望标题时要能说破「跑错篇了」，而不是让人对着四条 ⚠ 猜。
    #[tokio::test]
    async fn fucha_names_the_wrong_draft_when_the_title_does_not_match() {
        let io = FakeIo::new()
            .with("location.reload()", Reply::Text(""))
            .with(RELOADED_MARK, Reply::Bool(true))
            .with(
                FUCHA_SCRIPT,
                Reply::Text(
                    r#"{"original":"undeclared","reward":"","source":"","comment":"unknown","title":""}"#,
                ),
            );
        assert_eq!(
            step_fucha(&io, Some("我为什么开始每天写点东西")).await,
            StepOutcome::Unverified("fucha-reloaded-different-draft")
        );
    }

    /// 标题对得上（微信把空格渲染成不换行空格也算对得上）就不许乱指「跑错篇」。
    #[tokio::test]
    async fn fucha_keeps_the_generic_reason_when_the_title_matches() {
        let io = FakeIo::new()
            .with("location.reload()", Reply::Text(""))
            .with(RELOADED_MARK, Reply::Bool(true))
            .with(
                FUCHA_SCRIPT,
                Reply::Text(
                    r#"{"original":"undeclared","reward":"","source":"","comment":"off","title":"我为什么\u00a0开始每天写点东西"}"#,
                ),
            );
        assert_eq!(
            step_fucha(&io, Some("我为什么开始每天写点东西")).await,
            StepOutcome::Unverified("fucha-some-missing")
        );
    }

    /// 没给期望标题（`configure` 之外的调用）时保持原行为，不得凭空判断。
    #[tokio::test]
    async fn fucha_without_expected_title_never_blames_the_wrong_draft() {
        let io = FakeIo::new()
            .with("location.reload()", Reply::Text(""))
            .with(RELOADED_MARK, Reply::Bool(true))
            .with(
                FUCHA_SCRIPT,
                Reply::Text(
                    r#"{"original":"undeclared","reward":"","source":"","comment":"unknown","title":"别的标题"}"#,
                ),
            );
        assert_eq!(
            step_fucha(&io, None).await,
            StepOutcome::Unverified("fucha-some-missing")
        );
    }

    /// 2026-11-26 真实形态：保存后立刻 reload，设置区还在等 XHR，四项整齐读空。
    /// 这种情况不能报「没保存」——同一篇草稿紧接着 `configure` 四项全绿。
    #[tokio::test]
    async fn fucha_never_calls_an_unrendered_panel_unpersisted() {
        let io = FakeIo::new()
            .with("location.reload()", Reply::Text(""))
            .with(RELOADED_MARK, Reply::Bool(true))
            .with(
                FUCHA_SCRIPT,
                Reply::Text(
                    r#"{"ready":false,"original":"undeclared","reward":"","source":"","comment":"unknown","title":""}"#,
                ),
            );
        assert_eq!(
            step_fucha(&io, Some("我为什么开始每天写点东西")).await,
            StepOutcome::Unverified("fucha-settings-not-rendered")
        );
    }

    /// 设置区晚到时继续读，读到为止就算通过（不能因为第一次为空就判死）。
    #[tokio::test]
    async fn fucha_keeps_reading_until_the_settings_panel_shows_up() {
        let io = FakeIo::new()
            .with("location.reload()", Reply::Text(""))
            .with(RELOADED_MARK, Reply::Bool(true))
            .with_seq(
                FUCHA_SCRIPT,
                vec![
                    Reply::Text(
                        r#"{"ready":false,"original":"undeclared","reward":"","source":"","comment":"unknown"}"#,
                    ),
                    Reply::Text(
                        r#"{"ready":true,"original":"declared","reward":"已开启","source":"个人观点，仅供参考","comment":"on"}"#,
                    ),
                ],
            );
        assert_eq!(
            step_fucha(&io, None).await,
            StepOutcome::Applied("fucha-all-persisted")
        );
    }

    /// 半渲染的读数（字段陆续到达）不能当最终结论：必须等两次读数一致。
    /// 这里第一次已是 `ready` 但内容为空，第二次才全绿；判定必须取后者。
    #[tokio::test]
    async fn fucha_does_not_judge_a_half_rendered_read() {
        let io = FakeIo::new()
            .with("location.reload()", Reply::Text(""))
            .with(RELOADED_MARK, Reply::Bool(true))
            .with_seq(
                FUCHA_SCRIPT,
                vec![
                    Reply::Text(
                        r#"{"ready":true,"original":"undeclared","reward":"","source":"","comment":"unknown"}"#,
                    ),
                    Reply::Text(
                        r#"{"ready":true,"original":"declared","reward":"已开启","source":"个人观点，仅供参考","comment":"on"}"#,
                    ),
                ],
            );
        assert_eq!(
            step_fucha(&io, None).await,
            StepOutcome::Applied("fucha-all-persisted")
        );
    }

    /// 2026-11-26 两次真推的形态：四项整齐读空，紧接着 `configure` 却全绿。
    /// 一次读空不能立刻判「没保存」，要重载再读一遍；第二遍读到就按读到算。
    #[tokio::test]
    async fn fucha_rereads_when_every_setting_reads_empty() {
        let io = FakeIo::new()
            .with("location.reload()", Reply::Text(""))
            .with(RELOADED_MARK, Reply::Bool(true))
            .with_seq(
                FUCHA_SCRIPT,
                vec![
                    Reply::Text(
                        r#"{"ready":true,"original":"undeclared","reward":"","source":"","comment":"off","title":"推草稿"}"#,
                    ),
                    Reply::Text(
                        r#"{"ready":true,"original":"undeclared","reward":"","source":"","comment":"off","title":"推草稿"}"#,
                    ),
                    Reply::Text(
                        r#"{"ready":true,"original":"declared","reward":"已开启","source":"个人观点，仅供参考","comment":"on","title":"推草稿"}"#,
                    ),
                    Reply::Text(
                        r#"{"ready":true,"original":"declared","reward":"已开启","source":"个人观点，仅供参考","comment":"on","title":"推草稿"}"#,
                    ),
                ],
            );
        assert_eq!(
            step_fucha(&io, Some("推草稿")).await,
            StepOutcome::Applied("fucha-all-persisted")
        );
        // 读空 → 等 5 秒 → 重载再读，这个 5 秒是本条的主角。
        assert!(io.slept().contains(&5_000), "{:?}", io.slept());
    }

    // ── 预览 ──

    #[tokio::test]
    async fn yulan_skips_without_token() {
        let io = FakeIo::new().with(
            YULAN_INFO_MARK,
            Reply::Text(r#"{"token":"","appmsgid":"123"}"#),
        );
        assert_eq!(
            step_yulan(&io, Some("wxid_demo")).await,
            StepOutcome::Skipped("yulan-no-token")
        );
        assert_eq!(io.call_count(), 1);
    }

    #[tokio::test]
    async fn yulan_reports_unverified_on_nonzero_ret() {
        let io = FakeIo::new()
            .with(
                YULAN_INFO_MARK,
                Reply::Text(r#"{"token":"tk","appmsgid":"42"}"#),
            )
            .with(
                YULAN_SEND_MARK,
                Reply::Text(r#"{"status":200,"body":"{\"ret\":\"-1\",\"err_msg\":\"bad\"}"}"#),
            );
        assert_eq!(
            step_yulan(&io, Some("wxid_demo")).await,
            StepOutcome::Unverified("yulan-ret-nonzero")
        );
    }

    // ── 模板插入 ──

    #[tokio::test]
    async fn moban_reports_applied_and_waits_for_menu() {
        let io = FakeIo::new().with(&moban_script("moonpub 卡片"), Reply::Bool(true));
        assert_eq!(
            step_moban(&io, "moonpub 卡片").await,
            StepOutcome::Applied("moban-inserted")
        );
        assert_eq!(io.slept(), vec![1_500]);
    }

    #[tokio::test]
    async fn moban_reports_skip_when_template_missing() {
        let io = FakeIo::new().with(&moban_script("不存在的模板"), Reply::Bool(false));
        assert_eq!(
            step_moban(&io, "不存在的模板").await,
            StepOutcome::Skipped("moban-not-found")
        );
        assert!(io.slept().is_empty());
    }
}
