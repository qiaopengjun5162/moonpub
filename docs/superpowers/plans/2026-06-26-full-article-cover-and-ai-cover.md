# Full Article Cover And AI Cover Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** 让 MoonPub 先从全文提炼本地封面文案，再把微信后台 AI 配图作为可选兜底步骤接入 `configure` / `ship`。

**Architecture:** 复用现有 `article.rs` 清洗正文、`cover.rs` 渲染 HTML、`ai_workflow.rs` 调用 provider、`publish_steps.rs` 执行浏览器自动化。新增能力优先保持为小函数和可选步骤，避免把封面逻辑塞回 `app.rs` 或 `publish.rs`。

**Tech Stack:** Rust CLI、chromiumoxide、现有 AI provider（DeepSeek / OpenAI）、cargo nextest

---

### Task 1: 全文封面文案提炼

**Files:**
- Modify: `src/article.rs`
- Modify: `src/cover.rs`
- Test: `src/article.rs`
- Test: `src/cover.rs`

- [ ] Step 1: 先写失败测试，覆盖“无 frontmatter 标题摘要时，从全文提炼封面文案”
- [ ] Step 2: 运行定向 nextest，确认测试先失败
- [ ] Step 3: 在 `src/article.rs` 增加正文候选提取 helper，在 `src/cover.rs` 增加封面文本组装 helper
- [ ] Step 4: 重新运行定向 nextest，确认转绿

### Task 2: AI 封面文案提炼编排

**Files:**
- Modify: `src/ai.rs`
- Modify: `src/ai_workflow.rs`
- Modify: `src/cli.rs`
- Modify: `src/app.rs`
- Test: `src/cli.rs`
- Test: `src/ai.rs` or `src/ai_workflow.rs`

- [ ] Step 1: 写失败测试，覆盖 `cover --ai` 命令解析和封面 prompt 关键字段
- [ ] Step 2: 运行定向测试确认失败
- [ ] Step 3: 新增 AI cover prompt 和封面文案提炼函数，接入 `cover` 命令
- [ ] Step 4: 重新运行定向测试确认通过

### Task 3: 微信 AI 配图自动化步骤

**Files:**
- Modify: `src/publish_steps.rs`
- Modify: `src/publish.rs`
- Modify: `src/cli.rs`
- Modify: `src/error.rs`
- Test: `src/publish_steps.rs`
- Test: `src/cli.rs`

- [ ] Step 1: 写失败测试，覆盖 `aicover` 步骤脚本构造和 `configure aicover --headed` 解析
- [ ] Step 2: 运行定向测试确认失败
- [ ] Step 3: 实现 `step_aicover` 和流程挂载，保持软失败
- [ ] Step 4: 重新运行定向测试确认通过

### Task 4: ship 接入与文档同步

**Files:**
- Modify: `src/ship.rs`
- Modify: `README.md`
- Modify: `README_zh.md`
- Modify: `PROGRESS.md`
- Modify: `CLAUDE.md`

- [ ] Step 1: 把 `ship` 接到新的 AI cover 选项或默认配置开关
- [ ] Step 2: 更新中英文文档和项目经验记录
- [ ] Step 3: 跑格式化、clippy、全量 nextest
- [ ] Step 4: 根据真实输出修正文档措辞
