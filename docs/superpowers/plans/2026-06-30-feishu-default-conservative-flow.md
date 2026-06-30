# Feishu Default Conservative Flow Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 固化“飞书内容默认先停在草稿和本地预览，只有显式 `--push` 才继续推进到微信草稿”的项目规则，并同步所有用户可见入口文案。

**Architecture:** 这一轮只做文档与帮助文本层面的规则固化，不引入新的自动判断逻辑，也不改变现有命令行为。核心做法是在 README、中文说明书、help text、AGENTS、PROGRESS 中统一表达默认保守模式、显式直发模式以及“本地预览 vs 微信公众号后台预览”的区别。

**Tech Stack:** Markdown docs, Rust help text strings, cargo fmt, cargo nextest

---

### Task 1: 同步用户可见的默认规则表达

**Files:**
- Modify: `README.md`
- Modify: `README_zh.md`
- Modify: `docs/USER_GUIDE.md`

- [ ] **Step 1: 写出要新增的规则文案**

在三份用户文档里统一加入以下表达要点：

```text
默认模式：飞书内容先到“可编辑草稿 + 本地预览”
快速模式：只有显式加 --push 才继续推进到微信草稿
本地预览不等于微信公众号后台预览
进入微信草稿后，后半段和其它文章统一走 configure / ship / 微信后台预览 / 发布
```

- [ ] **Step 2: 更新英文 README 的命令说明**

把飞书相关命令说明统一成下面的语义：

```text
--preview is the default conservative path for local HTML review
--push is the explicit fast-forward path to WeChat draft push
local preview and WeChat backend preview-send are different stages
```

- [ ] **Step 3: 更新中文 README 的命令说明**

把飞书相关命令说明统一成下面的语义：

```text
默认推荐先走 --preview
只有显式 --push 才直发到微信草稿
本地预览和微信公众号后台预览不是一回事
```

- [ ] **Step 4: 更新用户手册中的标准流程**

把飞书链路的推荐顺序写成明确步骤：

```text
1. intake feishu ... --draft --preview
2. 人工修改 / polish / humanize（可选）
3. push --render
4. configure / ship
5. 微信公众号后台预览
6. 发布
```

- [ ] **Step 5: 自查三份文档是否仍有“自动直发是默认行为”的歧义**

Run:

```bash
rg -n "默认|preview|--push|直发|backend preview|后台预览" README.md README_zh.md docs/USER_GUIDE.md
```

Expected: 命中内容全部符合“默认保守、显式 `--push` 才直发”的口径。

### Task 2: 同步 CLI help text 与内部约定

**Files:**
- Modify: `src/error.rs`
- Modify: `AGENTS.md`
- Modify: `PROGRESS.md`

- [ ] **Step 1: 更新 `src/error.rs` 的帮助文本**

将相关命令说明明确为：

```text
preview = local preview
configure/test-yulan = WeChat backend preview-send
--push = explicit continuation into WeChat draft push
```

- [ ] **Step 2: 更新 `AGENTS.md` 中的流程约定**

加入以下约束：

```text
飞书链路默认推荐 `--draft --preview`
只有显式 `--push` 才表示继续推进到微信草稿
不要把本地 preview 和微信公众号后台 preview-send 混为一谈
```

- [ ] **Step 3: 更新 `PROGRESS.md` 中的规则总结**

补一条进展记录，明确：

```text
飞书默认保守模式已固化
`--push` 是显式直发开关
本地 preview / 微信后台 preview-send 已在文档中分层
```

- [ ] **Step 4: 自查帮助文本和内部约定**

Run:

```bash
rg -n "preview|--push|后台预览|local preview|WeChat backend" src/error.rs AGENTS.md PROGRESS.md
```

Expected: 命中内容和 spec 一致，没有相互矛盾的描述。

### Task 3: 验证并提交规则固化

**Files:**
- Modify: `README.md`
- Modify: `README_zh.md`
- Modify: `docs/USER_GUIDE.md`
- Modify: `src/error.rs`
- Modify: `AGENTS.md`
- Modify: `PROGRESS.md`

- [ ] **Step 1: 跑格式检查**

Run:

```bash
cargo fmt --all -- --check
```

Expected: PASS

- [ ] **Step 2: 跑回归测试**

Run:

```bash
cargo nextest run --all-features
```

Expected: PASS，全量测试通过。

- [ ] **Step 3: 检查工作区 diff 是否只包含规则固化相关改动**

Run:

```bash
git diff -- README.md README_zh.md docs/USER_GUIDE.md src/error.rs AGENTS.md PROGRESS.md
```

Expected: 只出现“默认保守模式 / 显式 `--push` / 两种预览分离”相关文案变化，不引入新的代码行为。

- [ ] **Step 4: 提交改动**

Run:

```bash
git add README.md README_zh.md docs/USER_GUIDE.md src/error.rs AGENTS.md PROGRESS.md
git commit -m "Document conservative Feishu publishing flow"
```

Expected: commit 成功，工作区干净。

- [ ] **Step 5: 推送并更新 PR**

Run:

```bash
git push -u origin codex/feishu-intake-idempotency
```

Expected: 远端分支更新，现有 PR 获得新提交。
