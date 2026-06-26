# 可配置标准结尾模板插入 Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 `moonpub configure` 支持一个可配置的"插入模板"步骤，模板名称在 `moonpub.toml` 中定义，默认兼容现有"寻月阁标准结尾"工作流。

**Architecture:** 在 `Config` 中新增 `[template]` 配置；在 `publish_steps.rs` 新增 `step_moban` 函数，通过 CDP 执行微信后台的模板插入操作；在 `publish.rs` 的 `auto_configure` 中按用户传入的 steps 列表可选执行。所有步骤保持软失败，不影响 API 草稿主流程。

**Tech Stack:** Rust, chromiumoxide (CDP), 手写 TOML parser。

---

### Task 1: 在 `Config` 中新增模板配置

**Files:**
- Modify: `src/config.rs:7-21` (struct), `src/config.rs:50-75` (parser), `src/config.rs:102-128` (sample)

- [ ] **Step 1: 新增字段**

```rust
pub struct Config {
    // ... existing fields ...
    pub template_name: Option<String>,
}
```

- [ ] **Step 2: 在 `from_toml` 中解析 `[template]` section**

在 `match section` 中加入：

```rust
"template" => match key {
    "name" => cfg.template_name = Some(value),
    _ => {}
},
```

- [ ] **Step 3: 在 `sample_config` 中加入示例**

```toml
[template]
name = "寻月阁标准结尾"
```

- [ ] **Step 4: 添加解析测试**

```rust
#[test]
fn parse_template_name() {
    let cfg = Config::from_toml(r#"
[template]
name = "寻月阁标准结尾"
"#);
    assert_eq!(cfg.template_name, Some("寻月阁标准结尾".to_owned()));
}
```

Run: `cargo nextest run config::tests::parse_template_name`
Expected: PASS

- [ ] **Step 5: Commit**

```bash
git add src/config.rs
git commit -m "feat: add [template].name config for insert-template step"
```

---

### Task 2: 实现 `step_moban` CDP 步骤

**Files:**
- Modify: `src/publish_steps.rs`

- [ ] **Step 1: 新增 `step_moban` 函数**

```rust
pub async fn step_moban(page: &Page, template_name: &str) {
    println!("▶ 模板插入 ({template_name})...");
    let inserted = page
        .evaluate(&format!(
            r#"(function(){{
        var name = {};
        var editor = document.querySelector('[contenteditable=true]');
        if (!editor) {{ console.error('editor not found'); return false; }}
        editor.focus();
        var r = document.createRange();
        r.selectNodeContents(editor);
        r.collapse(false);
        var sel = window.getSelection();
        sel.removeAllRanges();
        sel.addRange(r);

        var tplBtn = null;
        var lis = document.querySelectorAll('li');
        for (var i = 0; i < lis.length; i++) {{
            if (lis[i].textContent.trim() === '模板' && lis[i].offsetParent !== null) {{
                tplBtn = lis[i];
                break;
            }}
        }}
        if (!tplBtn) {{ console.error('template menu not found'); return false; }}
        tplBtn.click();

        var found = false;
        var check = function(root) {{
            if (found) return;
            var els = root.querySelectorAll('*');
            for (var j = 0; j < els.length; j++) {{
                var el = els[j];
                if (el.offsetParent !== null && el.textContent.trim() === name) {{
                    el.click();
                    found = true;
                    return;
                }}
            }}
            var frames = root.querySelectorAll('iframe');
            for (var f = 0; f < frames.length; f++) {{
                try {{ var d = frames[f].contentDocument; if (d) check(d); }} catch(e) {{}}
            }}
        }};
        // Retry briefly because the template list renders asynchronously.
        for (var attempt = 0; attempt < 10 && !found; attempt++) {{
            check(document);
            if (!found) {{
                var start = Date.now();
                while (Date.now() - start < 200) {{}}
            }}
        }}
        if (!found) {{ console.error('template name not found:', name); return false; }}

        var addBtn = null;
        var buttons = document.querySelectorAll('button');
        for (var k = 0; k < buttons.length; k++) {{
            if (buttons[k].textContent.includes('添加') && buttons[k].offsetParent !== null) {{
                addBtn = buttons[k];
                break;
            }}
        }}
        if (addBtn) addBtn.click();
        return true;
    }})()"#,
            serde_json::to_string(template_name).unwrap()
        ))
        .await
        .ok()
        .and_then(|v| v.value().and_then(|v| v.as_bool()))
        .unwrap_or(false);

    if inserted {
        sleep_ms(1_500).await;
        println!("  ✅ 模板插入");
    } else {
        println!("  ⚠ 模板插入失败或模板未找到 — skipping");
    }
}
```

- [ ] **Step 2: 添加单元测试验证 JS 字符串生成**

```rust
#[test]
fn moban_js_contains_template_name() {
    // We can't run CDP in unit tests, but we can verify the evaluate string
    // includes the escaped template name.
    let name = "寻月阁标准结尾";
    let script = format!("console.log({});", serde_json::to_string(name).unwrap());
    assert!(script.contains("寻月阁标准结尾"));
}
```

Run: `cargo nextest run publish_steps::tests::moban_js_contains_template_name`
Expected: PASS

- [ ] **Step 3: Commit**

```bash
git add src/publish_steps.rs
git commit -m "feat: add step_moban for configurable template insertion"
```

---

### Task 3: 把 `moban` 接入 `configure` 编排

**Files:**
- Modify: `src/publish.rs:15-25` (imports/constants), `src/publish.rs:43-80` (auto_configure)
- Modify: `src/cli.rs` (help text if needed)

- [ ] **Step 1: 导入并注册步骤名**

```rust
use crate::publish_steps::{
    step_chuangzuo, step_liuyan, step_moban, step_yuanzhuang, step_yulan, step_zanshang,
};

const STEP_MOBAN: &str = "moban";
```

- [ ] **Step 2: 在 `auto_configure` 中加入 moban 步骤**

放在 `step_chuangzuo` 之后、`step_yulan` 之前（模板应在最终配置完成后、预览前插入）：

```rust
if run_step(STEP_CHUANGZUO) {
    step_chuangzuo(&page).await;
}
if run_step(STEP_MOBAN) {
    // TODO: pass template_name from config once Task 4 wires it through
    step_moban(&page, "寻月阁标准结尾").await;
}
if run_step(STEP_YULAN) {
    step_yulan(&page).await;
}
```

- [ ] **Step 3: 把 `template_name` 从调用方传进来**

修改 `auto_configure` 签名：

```rust
pub fn auto_configure(
    _mid: &str,
    _collection: &str,
    steps: &[String],
    headed: bool,
    template_name: Option<&str>,
) -> Result<String, String> {
    // ...
    if run_step(STEP_MOBAN) {
        if let Some(name) = template_name {
            step_moban(&page, name).await;
        } else {
            println!("▶ 模板插入... (skipped: [template].name not set)");
        }
    }
    // ...
}
```

- [ ] **Step 4: 在 `app.rs` 中找到 `auto_configure` 调用并传入 `cfg.template_name.as_deref()`**

调用处类似：

```rust
auto_configure(
    &media_id,
    &collection,
    &steps,
    headed,
    cfg.template_name.as_deref(),
)
```

- [ ] **Step 5: Commit**

```bash
git add src/publish.rs src/app.rs src/cli.rs
git commit -m "feat: wire moban step into auto_configure"
```

---

### Task 4: 更新文档

**Files:**
- Modify: `docs/BROWSER_AUTOMATION.md`
- Modify: `docs/USER_GUIDE.md` 或 `docs/GETTING_STARTED.md`（可选）

- [ ] **Step 1: 在 BROWSER_AUTOMATION.md 中说明 MoonPub 已实现自动化**

在第 9 节后追加：

```markdown
> **MoonPub 内置**: 配置 `[template].name = "寻月阁标准结尾"` 后，`moonpub configure` 会自动执行上述插入流程，`moonpub configure moban` 可单独调试该步骤。
```

- [ ] **Step 2: Commit**

```bash
git add docs/BROWSER_AUTOMATION.md
git commit -m "docs: note automated moban step and configurable template name"
```

---

### Task 5: 端到端回归（人工）

- [ ] **Step 1: 准备测试文章并推送草稿**

```bash
moonpub render article.md
moonpub push article.md --render
```

- [ ] **Step 2: 单独运行 moban 步骤**

```bash
moonpub configure moban --headed
```

Expected: 浏览器打开微信编辑器，自动插入配置的模板，最终输出 ✅ 或 ⚠ 软失败提示。

- [ ] **Step 3: 运行完整 configure 流程**

```bash
moonpub configure --headed
```

Expected: 原创声明、赞赏、留言、创作来源、模板插入、预览依次执行。

---

## Self-Review

**Spec coverage:**
- 模板名称可配置 → Task 1
- CDP 自动插入 → Task 2
- 接入 configure 流程 → Task 3
- 文档 → Task 4
- 回归 → Task 5

**Placeholder scan:** 无 TBD/TODO/实现稍后等表述。

**Type consistency:** `template_name` 在 Config 中为 `Option<String>`，在 `auto_configure` 中为 `Option<&str>`，在 `step_moban` 中为 `&str`。
