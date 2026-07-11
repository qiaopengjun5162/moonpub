# ai-content-kb 参考结论

这份文档记录 MoonPub 对 [mrbear1024/ai-content-kb](https://github.com/mrbear1024/ai-content-kb) 的参考结论。

## 它解决什么问题

`ai-content-kb` 是一套 review-first 的本地 Markdown 知识库模板。它把作者原始材料、外部来源、已审核产品和 AI 未审核候选分开，要求 AI 输出先进入 staging，再由人工决定是否提升为正式内容。

这与 MoonPub 的素材到公众号工作流直接相关：飞书逐字稿和照片不应因 AI 整理而失去来源，草稿不应因生成成功就被当成可发布内容。

## 可以吸收

### 1. 保持现有目录，但明确角色

MoonPub 不需要复制 `raw/`、`sources/`、`products/`、`wiki/` 的目录树。现有角色已经足够：

| MoonPub 位置 | 角色 | 是否可直接推送 |
|---|---|---:|
| `Inbox/Feishu/`、`Inbox/Photos/` | 原始素材与来源记录 | 否 |
| `Articles/drafts/` | AI 或人工整理后的可编辑候选稿 | 否 |
| `Articles/ready/` | 已创建微信草稿、等待后台人工检查的文章包 | 否 |
| `Articles/published/` | 已确认发表的本地归档 | 否，归档状态 |

后续实现必须维护这些角色，不能让 AI 直接覆盖 Inbox，也不能把 `ready` 误写成“已经发表”。

### 2. 来源优先于生成文本

现有 `InboxMetadata` 的 `source`、`external_id`、`source_title`、`source_url`、`captured_at` 是正确基础。后续草稿处理应能追溯到其 Inbox 素材，而不是只保留 AI 重写后的文章。

### 3. AI 输出先停在可审查阶段

飞书和照片默认路径已经是 `--draft --preview`。继续保持：AI 生成草稿和本地 HTML 预览不是发布；只有显式 `--push` 才创建微信草稿，最终发表仍只由微信公众号后台人工确认。

### 4. 质量门保持本地只读

`layout-audit` 和 `preflight` 已符合 review-first 原则。后续如增加 `draft-audit`，应只读取草稿及来源元数据，报告来源缺失、未确认 AI 改写或产物缺失；不得调用微信 API、浏览器或自动发表。

## 不应吸收

- 不复制其完整知识库目录、wiki、typed graph sidecar 或迁移系统。MoonPub 是发布内核，不是通用第二大脑。
- 不在近期加入向量数据库、嵌入模型或图数据库。当前产品缺口是首次体验证据和发布可信度。
- 不让 AI 自动把候选稿提升为 `ready` 或 `published`。

## 后续优先级

完成 v0.4.2 的真实首次体验证据后，优先补最小草稿来源追溯与只读 `draft-audit`，而不是扩展新的知识库子系统。
