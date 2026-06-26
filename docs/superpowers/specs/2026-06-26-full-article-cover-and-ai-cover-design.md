# 全文封面提炼与 AI 配图兜底设计

## 背景

当前 `moonpub cover` / `moonpub ship` 的内置封面仍以 frontmatter `title` / `digest` 为主，缺少“根据整篇文章内容自动提炼封面文案”的能力。对没有手填标题摘要、或标题质量一般的文章，用户仍需要手工调整。与此同时，微信后台已经提供 AI 配图入口，仓库中也已有对应的浏览器自动化探测经验，但还没有接入正式流程。

## 目标

1. 本地封面生成优先基于整篇文章内容提炼更像“封面文案”的标题与副标题，而不是只依赖 frontmatter。
2. 微信后台自动化增加 AI 配图封面步骤，作为本地封面不满意时的兜底方案。
3. `ship` / `configure` 能显式控制是否执行 AI 配图，默认保持可控、软失败。

## 方案

### 1. 全文提炼封面文案

新增一个面向封面的文案提炼流程：

- 输入：frontmatter + 去掉 frontmatter / 微信尾部后的全文正文
- 输出：`title` / `subtitle`
- 优先级：
  - frontmatter 有显式 `title` / `digest` 时优先保留
  - 否则尝试从全文提炼
  - 再兜底到现有规则

提炼策略分两层：

- 无 AI：纯本地启发式
  - 取正文首段、H1/H2、重复出现的关键词句
  - 过滤 CTA、图片、分隔线、模板标记
  - 输出较短标题和一句副标题
- 有 AI：复用现有 `ai.rs` provider
  - 用统一 prompt 让模型从全文提炼“适合公众号封面的标题 + 副标题”
  - 仅在用户显式要求 AI 或配置允许时启用

### 2. 微信 AI 配图步骤

在 `publish_steps.rs` 增加独立 `step_aicover`：

- hover 封面区域
- 点击 `.js_aiImage`
- 输入基于全文/封面提炼结果生成的 prompt
- 点击发送
- 选择首张合格结果图
- 点击“使用”与“确认”

这一步必须软失败：

- 找不到按钮、生成失败、微信 UI 变化时，只打印 warning，不中断主流程

### 3. 命令与配置边界

- `cover`：
  - 增加可选 `--ai`，表示使用 AI 提炼封面文案
- `ship` / `ship --ai`：
  - 继续负责本地封面生成
  - 增加可选 `--ai-cover` 或配置化步骤，使推送草稿后可继续走微信后台 AI 配图
- `configure`：
  - 新增步骤名 `aicover`
  - 可单独调试：`moonpub configure aicover --headed`
- 配置：
  - 在 `[wechat]` 或独立 `[cover]` 增加一个简单布尔开关，控制 ship 后是否默认尝试 AI 配图

## 架构边界

- `src/article.rs`
  - 保持正文清洗、标题/摘要候选提取
- `src/cover.rs`
  - 保持 HTML 封面模板与最终文本渲染
- `src/ai_workflow.rs`
  - 新增封面文案 AI 提炼编排，避免把 AI 调用塞回 `app.rs` / `cover.rs`
- `src/publish_steps.rs`
  - 新增 `step_aicover`
- `src/publish.rs`
  - 只负责把 `aicover` 挂到自动化流程里
- `src/cli.rs` / `src/error.rs`
  - 负责新 flag / help text / 命令帮助

## 验证

- 单元测试：
  - 全文提炼在无 title/digest 时能产出合理封面文案
  - AI prompt 生成稳定，关键字段存在
  - `aicover` 的脚本字符串包含关键 selector 与 prompt 注入
- 命令解析测试：
  - `cover --ai`
  - `configure aicover --headed`
- 全量验证：
  - `cargo fmt --all -- --check`
  - `cargo clippy --all-targets --all-features --tests --benches -- -D warnings`
  - `cargo nextest run --all-features`
