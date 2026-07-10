# Khoj 参考融合地图

这份文档记录 MoonPub 对 [khoj-ai/khoj](https://github.com/khoj-ai/khoj) 的参考结论。

结论先说清楚：

**Khoj 对 MoonPub 的价值在于“本地优先知识层”和“多入口 AI 助手”的产品结构参考，不是让 MoonPub 近期变成完整二脑、搜索引擎或聊天 SaaS。**

## Khoj 的核心启发

Khoj 把自己定位成个人 AI second brain。它的公开 README 和文档里有几个高信号点：

- 支持本地或云端 LLM，对用户文档和互联网内容进行问答。
- 支持 Markdown、PDF、Word、Notion、GitHub 等多类数据源。
- 支持 Web、Obsidian、Emacs、Desktop、Phone、Whatsapp 等多入口。
- 支持语义搜索、聊天、Agent、自动化通知和知识库检索。
- 支持开源、自托管和云端形态。

对 MoonPub 来说，最值得吸收的不是某个具体实现，而是这几条产品原则：

- 用户内容先形成可索引的本地知识库，再由上层入口读取。
- 搜索 / 问答 / Agent 要引用来源，不能把生成内容伪装成原始事实。
- 多端入口共享同一个后端语义，而不是每个入口重新实现工作流。
- 本地隐私模式和云端便利模式要明确分层，不能混在一起讲。
- 知识助手能力应该是可选增强，不应该成为基础发布链路的前置条件。

## 与 MoonPub 的映射

| Khoj 能力 | 可吸收点 | MoonPub 对应落点 | 当前判断 |
|---|---|---|---|
| 本地 / 自托管知识库 | 用户资料留在本机，可选索引 | `Articles/`、`Inbox/`、Obsidian vault | 适合长期方向 |
| 语义搜索 | 自然语言找笔记和素材 | 未来 `moonpub search` 或插件搜索工作台 | 不进 v0.4.x / v0.5 |
| Chat over docs | 基于来源回答问题 | 未来 `moonpub ask`，必须返回引用文件 | 先设计，不实现 |
| Obsidian 插件 | 在用户写作工具内提供入口 | 当前 MoonPub 插件首页 | 已经方向一致 |
| 多客户端 | Web / Desktop / Mobile 共享后端 | 未来本地 App / Agent 包装 | 中长期产品化方向 |
| Agent | 自定义 persona / knowledge / tools | `workflow-registry --json` + 发布副驾驶 | 只吸收契约思想 |
| Automations | 定时研究、通知、newsletter | 未来可做发布提醒 / 证据提醒 | 不能自动最终发表 |
| 多数据源 | 文档、Notion、GitHub 等接入 | 飞书、照片、公众号 URL、读书摘录 | 只按输入工作流逐步接 |

## 最值得未来吸收的三件事

### 1. 只读知识索引层

MoonPub 现在已经有稳定的内容目录：

- `Articles/drafts/`
- `Articles/ready/`
- `Articles/published/`
- `Articles/Inbox/`

未来如果做知识助手，第一步不应该直接做聊天机器人，而应该先做只读索引：

```bash
moonpub index --articles <path>
moonpub search "上次写闲月隐林时提到的自动化工作流"
```

这层必须保持只读：

- 不改原文。
- 不移动文章包。
- 不触发微信 API。
- 不打开浏览器。
- 搜索结果返回文件路径、标题、阶段和片段。

### 2. 带来源引用的问答层

如果未来做 `moonpub ask`，它应该和普通 AI 写作严格区分。

推荐边界：

- 回答必须列出引用来源。
- 来源优先指向本地文件路径和 frontmatter 元数据。
- 不确定时明确说“不确定”，不能自动补事实。
- 默认不写回文件；只有显式 `--write-draft` 才能生成草稿。
- 生成草稿也应先进入 `Inbox -> Draft -> Preview`，不直接 push。

这能让 MoonPub 的 AI 能力更像“写作副驾驶”，而不是会悄悄改库的黑箱助手。

### 3. 多入口共享同一套协议

Khoj 的多入口形态提醒我们：MoonPub 后续如果有 CLI、Obsidian 插件、本地 App、Agent 包装，不应该让每个入口自己理解项目状态。

MoonPub 已经开始具备这层基础：

- `doctor --json`
- `workspace --json`
- `workflow-registry --json`
- `status --json`
- `check --json`
- `layout-recipes --json`
- `layout-audit --json`
- `wechat-health --json`

未来如果新增知识助手，也应继续走结构化协议，而不是只输出自然语言：

```bash
moonpub search "照片合集第一篇" --json
moonpub ask "这周有哪些素材适合整理成文章？" --json
```

## 不建议近期吸收的部分

短期不要把下面能力做进 MoonPub 主线：

- 完整语义向量库和重排模型。
- 多用户云端账号体系。
- 在线 App / SaaS 托管。
- Whatsapp / 手机端聊天入口。
- 自动 newsletter 或周期性主动发文。
- 通用文件上传知识库。
- Agent 自动调用发布命令并推进到微信草稿。

原因：

- v0.4.2 / v0.5 的核心瓶颈仍是首次体验证据、插件首页、真实微信回归和 release 收口。
- 完整搜索 / 问答系统会引入索引、模型、隐私、同步和多端状态管理，工程重量远超当前发布内核。
- 如果过早引入，会让用户更不清楚 MoonPub 到底是发布工具、知识库还是聊天助手。

## 对 MoonPub 路线的建议

近期仍按这个顺序推进：

1. 先把 Obsidian 插件首页、飞书、照片、当前文章和微信草稿路径做到真实证据充分。
2. 再做 v0.5 插件化核心，让现有发布能力更可维护。
3. 然后补只读 `preflight` 聚合质量门。
4. 再评估只读 `search`：只索引 MoonPub 管理的 Articles / Inbox，不扩全盘知识库。
5. 最后再评估带来源引用的 `ask`，并且默认只读、不写回。

如果未来要做得更像 Khoj，也应该先回答清楚：

- 这是 MoonPub Core 的能力，还是独立的 Knowledge Layer？
- 它是否会影响基础发布流程？
- 它是否需要额外模型、索引目录或长期后台进程？
- 它是否会读取用户 Obsidian vault 里 MoonPub 之外的隐私笔记？
- 它是否有明确的引用来源和撤销机制？

## 当前结论

Khoj 对 MoonPub 的长期启发很明确：

**MoonPub 可以在发布内核之上长出一个可选的本地知识助手层，但近期不能为了“二脑感”牺牲发布主链路的可用性。**

也就是说：

- v0.4.2：先让用户会用。
- v0.5：先让核心可扩展。
- v0.6：先让 Obsidian 入口正式化。
- 更后面：再考虑 `search` / `ask` / 本地知识层。

## 参考来源

- [khoj-ai/khoj](https://github.com/khoj-ai/khoj)
- [Khoj Features Overview](https://raw.githubusercontent.com/khoj-ai/khoj/master/documentation/docs/features/all-features.md)
- [Khoj Search](https://raw.githubusercontent.com/khoj-ai/khoj/master/documentation/docs/features/search.md)
- [Khoj Agents](https://raw.githubusercontent.com/khoj-ai/khoj/master/documentation/docs/features/agents.md)
- [Khoj Obsidian Client](https://raw.githubusercontent.com/khoj-ai/khoj/master/documentation/docs/clients/obsidian.md)
- [Khoj Data Sources](https://raw.githubusercontent.com/khoj-ai/khoj/master/documentation/docs/data-sources/share_your_data.md)
