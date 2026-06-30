# Feishu Draft Auto Push Design

**Goal:** 让 `draft-from-inbox` 和 `intake feishu --draft` 可选地直接推进到微信草稿推送，减少“生成草稿后再手动切 `moonpub push ... --render`”这一步。

## 背景

目前飞书秒记主链路已经具备两层能力：

1. `intake feishu --minute-token/--latest/--query`
   可以把官方飞书秒记稳定导入到 `Inbox/Feishu/`
2. `draft-from-inbox` / `intake feishu --draft`
   可以基于 Inbox 素材生成可编辑草稿

并且上一轮已经补上了幂等重跑：

- 同一条飞书秒记会复用同一个 Inbox 文件
- 重复生成草稿会复用同一个草稿文件

但用户当前还要手动执行下一条命令：

`moonpub push <draft.md> --render`

这一步本质上是固定后续动作，尤其在飞书自动化链路里显得重复。

## 范围

本轮只给“草稿生成后自动推微信草稿”补一个明确开关，不改变默认行为。

### 包含

- `moonpub draft-from-inbox <inbox.md> --push`
- `moonpub intake feishu ... --draft --push`

### 不包含

- 自动执行 `configure`
- 自动执行 `ship`
- 自动触发浏览器预览
- 自动发表

## 设计原则

### 1. 默认行为不变

不带 `--push` 时，现有行为完全保持：

- 生成草稿
- 输出 `next: moonpub push ... --render`
- 用户自己决定是否继续

### 2. `--push` 是显式推进

只有用户明确写了 `--push`，才会在草稿生成成功后继续执行微信草稿推送。

这能避免：

- 本地只想看草稿，却误触网络调用
- AI 草稿还没确认，就被直接推到微信后台

### 3. `--push` 与 `--preview` 互斥

`--preview` 的含义是“生成后先人工看一眼”，`--push` 的含义是“生成后直接推进到下一步”。

两者代表不同的节奏，因此这一轮直接定义为互斥，避免出现含义模糊的组合：

- `--preview --push`
- `--preview --no-open --push`

如果用户要先看再推，仍然走现有两步：

1. `--preview`
2. `push --render`

### 4. 自动推送时始终用 `push --render` 语义

因为草稿内容刚刚由 AI 生成，后续自动推送时应该强制确保：

- HTML 是新渲染的
- draft.json 是最新的

因此实现上直接调用已有的：

```rust
push_article(..., true, ...)
```

也就是等价于 CLI 的 `moonpub push <draft.md> --render`。

## CLI 方案

## A. `draft-from-inbox`

新增：

```bash
moonpub draft-from-inbox Inbox/Feishu/demo.md --push
```

行为：

1. 生成或更新草稿
2. 直接渲染并推送到微信草稿
3. 返回组合输出

### 约束

- `--push` 与 `--preview` 互斥
- `--push` 与 `--no-open` 也不需要组合，因为它不走 preview

## B. `intake feishu ... --draft`

新增：

```bash
moonpub intake feishu --latest --draft --push
moonpub intake feishu --minute-token <token> --draft --push
moonpub intake feishu --query <text> --draft --push
```

行为：

1. 导入或更新 Inbox
2. 生成或更新草稿
3. 直接渲染并推送到微信草稿
4. 返回组合输出

### 约束

- `--push` 只能和 `--draft` 一起用
- `--push` 与 `--preview` 互斥

## App 层方案

`src/app.rs` 继续作为编排层，只新增一个明确分支：

- 草稿生成成功后，如果 `auto_push == true`
  - 加载 config
  - 调用 `push_article(&options.articles, &draft_output.path, true, &cfg)`

### 文本输出

保留组合式输出，顺序如下：

1. intake 输出
2. draft 输出
3. push 输出

### JSON 输出

已有结构化对象继续保留原字段，并在 `--push` 时补充一组 push 结果：

- `pushed: true`
- `media_id`
- `stage`
- `next_step`

这样自动化调用方不需要再单独跑一次 `push --json`。

## 错误与边界

### 保持已有失败行为

如果自动 `push` 阶段失败：

- 整个命令返回失败
- 但前面的 Inbox / draft 文件更新已经落地

这和现有的命令式流水线一致：前一步成功并不因为后一步失败而回滚。

### 不做事务回滚

这一轮不尝试：

- push 失败时回滚草稿文件
- push 失败时回滚 Inbox 文件

因为这些文件本来就是用户可继续编辑和重试的中间产物，保留它们比强行回滚更安全。

## 文件范围

### 主要修改

- `src/cli.rs`
  给 `draft-from-inbox` 和 `intake feishu` 增加 `--push`
- `src/app.rs`
  增加草稿后自动推送编排与 JSON 输出扩展
- `src/error.rs`
  同步 help text

### 测试

- `src/cli.rs`
  新增 flag 解析和互斥约束测试
- `src/app.rs`
  新增 builder 级 JSON 测试，以及“失败前置边界不触网”的测试

### 文档

- `README.md`
- `README_zh.md`
- `docs/USER_GUIDE.md`
- `PROGRESS.md`
- `AGENTS.md`

## 成功标准

满足以下条件即视为本轮完成：

1. `draft-from-inbox --push` 能直接推进到微信草稿推送
2. `intake feishu ... --draft --push` 能直接推进到微信草稿推送
3. `--push` 与 `--preview` 的互斥规则明确且有测试保护
4. 默认不带 `--push` 的行为完全不变
