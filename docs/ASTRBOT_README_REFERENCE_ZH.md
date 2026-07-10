# AstrBot README 参考融合地图

这份文档记录 MoonPub 对 [AstrBot 中文 README](https://github.com/AstrBotDevs/AstrBot/blob/master/README_zh.md) 的参考结论。

结论先说清楚：

**AstrBot README 对 MoonPub 的价值在于“开源项目第一屏如何让用户快速判断、安装、进入文档和参与社区”，不是让 MoonPub 近期变成聊天机器人框架或多平台 Bot 系统。**

## 参考 README 的核心启发

AstrBot README 面向的是一个多平台 AI 聊天机器人框架。它的内容组织里有几件事值得 MoonPub 学：

- 第一屏直接给出项目定位、官网、文档、博客、路线图、问题反馈和 QQ 群等入口。
- 用中英双语入口减少用户迷路成本。
- 把“适合谁用”和“怎么部署”尽量前置。
- 用支持矩阵说明平台、模型服务商、部署方式和生态能力。
- 用插件市场 / 社区 / Star History 让项目看起来不是孤立工具。
- 用清晰贡献入口承接技术用户和开发者。

对 MoonPub 来说，这些启发不在于功能形态，而在于 README 的产品化表达纪律：

- 先帮助用户判断“我该不该用它”。
- 再给用户一条最短开始路径。
- 再把高级能力、插件、路线和社区入口分层展开。
- 不要让用户读完整个 README 才知道第一步是什么。

## 与 MoonPub 的映射

| AstrBot README 做法 | 可吸收点 | MoonPub 对应落点 | 当前判断 |
|---|---|---|---|
| 顶部入口聚合 | 官网 / 文档 / 路线图 / 问题反馈一屏可见 | README / README_zh 第一屏 | 值得吸收 |
| 多语言入口 | 中文用户和国际用户分流 | README 与 README_zh 互链 | 已有，可更明显 |
| 部署方式分层 | 新手、Docker、源码、面板分别说明 | CLI 安装、Obsidian 插件、源码构建、release smoke | 值得优化 |
| 支持矩阵 | 平台 / 模型 / 插件能力一眼看清 | 输入源、发布目标、主题、质量门、浏览器自动化 | 值得吸收 |
| 插件市场展示 | 生态入口降低扩展理解成本 | v0.5 插件化核心、Obsidian 插件入口 | 中期可用 |
| Roadmap 可见 | 用户知道项目还在推进什么 | ROADMAP / RELEASE_GATE / PROGRESS | 已有，可强化首屏链接 |
| 社区与贡献入口 | 给试用者和开发者不同入口 | Issues / Discussions / PR 模板 / 贡献指南 | 可补强 |

## 最值得吸收的四件事

### 1. README 第一屏要先帮用户做选择

MoonPub 现在能力很多：当前文章、飞书、照片、封面、排版主题、微信公众号草稿、Obsidian 插件、Zola 导出、浏览器自动化。能力多是好事，但第一屏不能变成命令清单。

更好的第一屏结构应是：

- MoonPub 是什么。
- 当前 Beta 适合谁。
- 一条推荐主线是什么。
- 哪些动作只在本地，哪些会触达微信。
- 新用户先从 Obsidian 插件首页还是 CLI 开始。
- 如果只是想试用，不需要微信凭证的路径是什么。

这比继续堆更多命令更能降低首次使用焦虑。

### 2. 支持矩阵比散落段落更清楚

AstrBot 用支持入口和生态说明减少认知负担。MoonPub 也适合把能力做成矩阵，而不是散落在段落里：

- 输入源：Markdown / Obsidian 当前文章 / 飞书秒记 / 照片 / 未来公众号 URL。
- 输出目标：本地 HTML / 微信公众号草稿 / Zola / 未来 WordPress / Ghost。
- 用户入口：CLI / Obsidian 插件 / 未来本地 App / Agent。
- 质量门：doctor / check / layout-audit / wechat-health / 未来 preflight。
- 风险等级：本地安全 / 生成草稿 / 触达微信 API / 浏览器辅助 / 人工最终发布。

这能让用户快速知道“我现在能用哪一块”，也能防止文档把未实现能力说得太满。

### 3. 安装路径要按用户类型分层

MoonPub 目前最容易卡用户的是安装和第一次配置。可以参考 AstrBot 的表达方式，把路径按用户类型拆清楚：

- 只想体验本地渲染：下载 release 或源码构建，跑 `init -> new -> preview`。
- Obsidian 用户：安装 CLI，启用插件，从 MoonPub 首页开始。
- 微信公众号作者：先跑 `doctor` / `wechat-health`，再推微信草稿。
- 开发者：源码构建、测试、PR-first、贡献指南。

这样用户不用先理解所有功能，就能找到自己的入口。

### 4. 社区入口和反馈路径要更明确

AstrBot README 把社区、问题反馈、路线图和插件生态放在用户能看到的位置。MoonPub 现在更像工程项目，未来对外 Beta 时也需要更清楚：

- 遇到安装失败去哪看。
- 遇到微信登录 / IP 白名单 / 草稿失败去哪排障。
- 想提交排版主题、封面样式或平台插件怎么做。
- 哪些功能欢迎提 issue，哪些明确不做。

这会让技术用户更愿意试用，也能减少重复解释。

## 不建议近期吸收的部分

短期不要把下面能力做进 MoonPub 主线：

- 多聊天平台适配。
- Bot 管理面板。
- 消息事件系统。
- 插件市场前端。
- 模型服务商聚合路由。
- Web UI 后台和用户账号体系。
- 把 MoonPub 说成通用 AI Agent 平台。

原因：

- MoonPub 的核心是本地发布内核，不是实时对话框架。
- 当前最重要的是 Obsidian / 飞书 / 照片 -> 草稿 -> 预览 -> 微信草稿这条链路可用。
- 过早引入面板、市场和社区生态，会稀释 v0.4.2 / v0.5 的收口目标。

## 对 MoonPub 路线的建议

近期仍按这个顺序推进：

1. 先把 README / README_zh 第一屏收成“用户快速判断 + 推荐入口 + 风险边界”。
2. 把输入源、输出目标、用户入口和质量门整理成支持矩阵。
3. 把安装路径按“本地试用 / Obsidian 用户 / 微信公众号作者 / 开发者”分层。
4. 把 `docs/FIRST_RUN_WALKTHROUGH_ZH.md` 和 `docs/RELEASE_GATE_v0.4.2_ZH.md` 作为第一屏可见入口，而不是深埋在长文档里。
5. 插件市场和生态展示等到 v0.5 插件化核心稳定后再考虑，不提前包装。

一句话：

**MoonPub 可以学习 AstrBot README 的“入口聚合、支持矩阵、用户分层和社区承接”，但不能因为 README 好看就提前把产品边界扩成 AI 平台。**

## 安全与边界备注

MoonPub 后续如果参考 AstrBot 做更产品化的 README，应保持：

- 不夸大生产可用度。
- 不把未实现平台写成已支持。
- 不把浏览器自动化描述成全自动发布。
- 不隐藏微信凭证、扫码、审核和最终人工确认边界。
- 不把社区入口包装成商业承诺。
- 不复制 AstrBot README 的文案、图标、素材或站点结构。

## 参考来源

- [AstrBot](https://github.com/AstrBotDevs/AstrBot)
- [AstrBot 中文 README](https://github.com/AstrBotDevs/AstrBot/blob/master/README_zh.md)
