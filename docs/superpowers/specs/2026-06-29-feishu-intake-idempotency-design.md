# Feishu Intake Idempotency Design

**Goal:** 让 `moonpub intake feishu` 对同一条飞书秒记可安全重跑，不会重复生成越来越多的 Inbox 文件和草稿文件。

## 背景

当前飞书秒记主链路已经能跑通：

`latest/query/minute-token -> Inbox/Feishu/*.md -> draft-from-inbox -> render/preview`

但这条链路还不具备稳定的重跑能力：

- 同一条飞书秒记重复导入时，当前实现只按标题 slug 和当天日期生成 Inbox 文件。
- 对 `--minute-token`、`--latest`、`--query` 来说，真正稳定的身份标识是 `minute_token`，不是标题。
- 一旦用户或后续 Agent 重复跑同一条秒记，就容易生成多份近似内容，后续再生成草稿时也会继续扩散。

这会直接阻碍“人只做确认，其它流程自动化”的目标，因为自动化系统必须能够安全重试。

## 需求范围

本轮只解决飞书秒记导入与草稿生成的幂等问题，不扩展到其它输入源。

### 包含

- `intake feishu --minute-token <token>`
- `intake feishu --latest`
- `intake feishu --query <keyword>`
- 这些命令在带 `--draft` 时触发的后续草稿生成

### 不包含

- `intake feishu <local-file>` 的模糊去重
- 基于标题或正文相似度的去重
- 微信推送幂等
- 浏览器自动化步骤幂等

## 设计原则

### 1. `minute_token` 是唯一稳定主键

对飞书秒记 API/CLI 拉取到的内容，`minute_token` 是唯一可依赖的身份标识。

因此：

- 如果输入源能提供 `minute_token`，就只按 `minute_token` 判断是否是同一条秒记
- 不使用标题、slug、日期去推断“可能是同一条”

### 2. 本地文本导入保持现状

`intake feishu <file>` 目前没有稳定外部主键。

这一轮不尝试对本地文本做内容 hash 或标题相似度匹配，原因是：

- 容易误把两篇不同内容合并
- 会显著扩大需求和测试面
- 不利于先把飞书官方链路做稳

### 3. 重跑默认更新，不报冲突

对同一 `minute_token` 重复执行导入时，系统应：

- 复用既有 Inbox 文件路径
- 更新 frontmatter 与正文内容
- 返回明确的 `updated` 语义

而不是：

- 再建一个新文件
- 或因为“文件已存在”直接失败

### 4. 草稿生成优先复用已存在草稿

从同一份 Inbox 素材继续执行 `--draft` 时，如果该 Inbox 之前已经生成过草稿，应尽量复用原草稿路径，而不是每次重新按标题创建新草稿。

这里的目标不是做复杂的草稿版本管理，而是把“同一份素材 -> 同一份可编辑草稿”稳定下来。

## 方案

## A. Inbox 文件幂等

### 现状

`src/intake.rs` 当前直接根据：

- 当天日期
- 标题 slug

生成 `Inbox/Feishu/<date>-<slug>.md`

这会导致同一 `minute_token` 在不同时间、不同标题变化或同标题重跑时缺少稳定映射。

### 新行为

新增“按 `minute_token` 查找既有 Inbox 文件”的路径：

1. 当 `FeishuMinutes.minute_token` 存在时，先扫描 `Inbox/Feishu/*.md`
2. 读取 frontmatter 里的 `minute_token`
3. 如果存在相同 token：
   - 复用该文件路径
   - 覆写 frontmatter 和正文
   - 返回 `updated`
4. 如果不存在：
   - 按当前命名策略新建文件
   - 返回 `created`

### 输出语义

`IntakeOutput` 增加结构化状态字段：

- `path`
- `message`
- `action`，值为 `created` 或 `updated`

文本输出示例：

- `intake created`
- `intake updated`

`--json` 场景后续也可以直接消费这个状态。

## B. 草稿生成幂等

### 现状

`draft-from-inbox` 当前调用 `write_article_file(...)` 生成新草稿。已有同名草稿时会报错。

这对于自动化重跑不友好，因为：

- 同一份 Inbox 素材再次生成草稿时会直接中断
- 用户必须手动清理已有草稿后才能继续

### 新行为

为 `draft-from-inbox` 增加“优先复用既有草稿路径”的最小策略：

1. 根据 Inbox 文件名 stem 推导草稿标题和默认草稿路径
2. 如果默认草稿文件已存在：
   - 直接覆写该草稿内容
   - 返回 `updated`
3. 如果不存在：
   - 创建新草稿
   - 返回 `created`

这里仍然只处理“同一路径同名草稿”的复用，不做跨文件搜索或复杂映射。

### 输出语义

`DraftOutput` 增加：

- `action`，值为 `created` 或 `updated`

文本输出示例：

- `draft created`
- `draft updated`

后续 `next: moonpub push ... --render` 保持不变。

## C. App 层集成

`src/app.rs` 负责把 intake + draft + preview 串起来，因此需要同步两类变化：

### 文本输出

- `intake feishu --draft`
- `draft-from-inbox`

在成功信息中保留 `created/updated` 语义。

### JSON 输出

对这两条已有结构化输出的链路补充状态字段：

- `action: "created" | "updated"`

这样上层 Agent 能区分：

- 首次成功
- 重跑更新成功

而不必继续从 message 文本反推。

## D. 错误与边界

### 保留现有失败场景

以下错误仍然保持失败：

- `lark-cli` 调用失败
- 秒记详情缺少 transcript
- AI 调用失败
- preview 依赖的 HTML 不存在

### 不引入的新行为

这一轮不做：

- 草稿历史版本
- 三方合并
- 冲突提示 UI
- 按正文 hash 检测“内容是否真的变化”

即使重复运行而内容完全一致，也允许直接覆写。因为目标是“稳定可重跑”，不是“最少写盘”。

## 文件变更范围

### 主要修改

- `src/intake.rs`
  新增按 `minute_token` 查找既有 Inbox 文件与 `created/updated` 输出
- `src/ai_workflow.rs`
  新增草稿覆写复用策略与 `created/updated` 输出
- `src/app.rs`
  把 `action` 状态接到文本和 JSON 输出

### 测试

- `src/intake.rs`
  补同 token 重跑应更新原 Inbox 文件的测试
- `src/ai_workflow.rs`
  补已有草稿时应复用并返回 `updated` 的测试
- `src/app.rs`
  补 `--json` 输出含 `action` 字段的测试

### 文档

- `README.md`
- `README_zh.md`
- `docs/USER_GUIDE.md`
- `PROGRESS.md`
- `AGENTS.md`

文档只描述“飞书秒记官方链路支持幂等重跑”，不夸大为“所有导入源都支持去重”。

## 测试策略

### 单元测试

- `intake`：同一 `minute_token` 第二次导入时，路径不变、内容更新、action 为 `updated`
- `draft-from-inbox`：同一路径草稿已存在时，内容被更新、action 为 `updated`
- `app --json`：结构化输出包含 `action`

### 回归验证

完整验证仍按项目约定：

- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features --tests --benches -- -D warnings`
- `cargo nextest run --all-features`

## 取舍

### 为什么不先做“一键 push/ship”

因为一键链路建立在“前面几步可安全重试”之上。

如果同一条秒记重复导入就产生多份 Inbox 或多份草稿，那么越自动化，数据会越乱。

所以先把幂等打稳，比先缩短命令条数更重要。

### 为什么不做标题相似度去重

因为标题相似度不可靠，会误合并不同内容；而 `minute_token` 是飞书原生稳定主键，先用它就够把官方主链路打通。

## 成功标准

满足以下条件即视为本轮完成：

1. 同一条飞书秒记用 `--minute-token` / `--latest` / `--query` 重复导入时，复用原 Inbox 文件而不是新建副本
2. 同一份 Inbox 素材重复生成草稿时，复用原草稿文件而不是报“已存在”
3. 文本和 `--json` 输出都能区分 `created` / `updated`
4. 不改变微信推送和浏览器自动化现有行为
