# yichen-skills 参考融合地图

这份文档记录 MoonPub 对 [mcncarl/yichen-skills](https://github.com/mcncarl/yichen-skills) 的参考结论。

结论先说清楚：

**MoonPub 可以吸收它的产品化工作流思想，但不复制代码，不直接引入高风险平台抓取、微信本地库解密或第三方桌面端自动化能力。**

## 参考库的核心启发

`yichen-skills` 更像一个内容创作者技能集合，覆盖：

- 对话精华沉淀到 Obsidian
- Markdown 上传到 X Articles 草稿
- 微信本地数据沉淀到私有 vault
- 抖音 / 小红书素材抓取
- ASR 转写和视频粗剪
- ChatGPT 官网调研报告
- Codex Memory 记忆库维护
- 微信公众号文章归档导出

它对 MoonPub 有价值的地方不是“某个脚本”，而是这几条产品原则：

- 默认先生成草稿或本地报告，不默认最终发布。
- 高风险动作前做 dry-run / 预检 / 显式确认。
- 私密素材先进入本地 vault / Inbox，再进入可编辑草稿。
- 机器可读 JSON 和人工可读 Markdown 同时存在。
- 任务结束后做 closeout / audit，把状态和证据沉淀下来。
- 公开仓库不携带真实 cookie、token、数据库、聊天记录和个人路径。

## 与 MoonPub 的映射

| yichen-skills 模块 | 可吸收点 | MoonPub 对应落点 | 当前判断 |
|---|---|---|---|
| `summary` | 对话精华结构化沉淀 | 未来 `intake conversation` 或手动 Inbox 模板 | 值得研究，但不进 v0.4.x |
| `x-article-draft-uploader` | `dry-run`、封面前置校验、独立浏览器、只存草稿 | `check` / `layout-audit` / `cover` / `push` 的发布前质量门 | 原则已部分具备，后续可增强 dry-run 聚合 |
| `wechat-local-vault` | 私有 vault、统一查询入口、摘要素材包 | 未来“聊天/收藏夹/朋友圈 -> Inbox”设计参考 | 高风险，只做长期研究，不碰真实解密 |
| `codex-memory` | prewrite reconcile、closeout、audit、泄漏检查 | release gate、证据归档、项目记忆/进度收口 | 很适合吸收为工程流程 |
| `chatgpt-web-research` | 等待完整答案、保存 raw + readable report、禁止读 cookie | `reports/` 或 `docs/first-run-evidence/` 证据保存 | 可吸收为证据和调研报告规范 |
| `wechat-mp-batch-exporter` | 已知 URL 归档、安全边界、指标采集分级 | `docs/WECHAT_ARCHIVE_WORKFLOW_ZH.md` | 已完成设计收口 |
| `douyin-fetcher` / `xiaohongshu-fetch` | 素材先落地，支持 metadata-only | 未来自媒体素材输入源 | 暂不纳入主线 |
| `volc-asr` | 音视频转写后再进入草稿 | 飞书之外的本地语音/视频输入源 | 中期可做，但低于飞书/照片稳定性 |
| `jianying-editor` | 视频剪辑最终交给专业工具 | MoonPub 不做视频编辑器 | 不进入 MoonPub |
| `mac-wechat-dual-open` | 工具化很强，但平台风险高 | 无直接落点 | 不进入 MoonPub |

## 最值得立刻吸收的三件事

### 1. 发布前 dry-run 聚合

MoonPub 已经有分散的质量门：

- `doctor --json`
- `check --json`
- `layout-audit`
- `wechat-health`
- `preview --no-open`

后续可以考虑新增一个更产品化的聚合入口：

```bash
moonpub preflight <article.md> --json
```

它只做本地和只读检查，不触发微信 API，不打开 Chrome，不上传草稿。

建议聚合：

- 文章 bundle 是否完整
- HTML 是否已生成
- 封面是否存在
- 排版审计是否通过
- 配置是否足够进入下一步
- 下一条推荐命令

这比继续往 `push` 里塞更多隐式检查更清晰。

### 2. 私有素材库与公开项目彻底隔离

`wechat-local-vault` 的重要启发是：私密原始素材不应该进入项目仓库。

MoonPub 后续如果做微信聊天、公众号归档、读书摘录或照片 EXIF 深解析，应保持：

- 原始素材进入用户的 `Articles/Inbox/` 或私有 vault。
- repo 只保存代码、模板、假样例和脱敏文档。
- 真实导出正文、聊天记录、cookie、token、数据库和截图证据不提交。
- 插件首页只暴露安全入口，不默认展示高风险批量导入。

### 3. closeout / audit 变成 release gate

`codex-memory` 的 closeout/audit 思路可以迁移到 MoonPub 的工程流程：

- 每个 PR 合并前记录真实验证命令。
- 每个 release 前跑证据清单，而不是只看测试绿。
- `PROGRESS.md` 只记录真实完成项，不把“设计完成”写成“用户已跑通”。
- 敏感信息泄漏检查成为 release gate 的一部分。

MoonPub 现在已经在做这件事，后续可以把它固化到 `docs/RELEASE_GATE_v0.4.2_ZH.md` 或后续 release gate 文档中。

## 不建议吸收的部分

短期不要把下面能力做进 MoonPub 主线：

- 微信 Mac 本地数据库解密。
- 微信桌面端 UI 操作。
- 抓取朋友圈、聊天记录、联系人或收藏夹。
- 平台历史列表批量抓取作为默认入口。
- 阅读数、评论、点赞等指标采集作为默认能力。
- 多平台社媒抓取和视频剪辑工具。

原因：

- 风险边界高，容易偏离“公众号发布内核”定位。
- 依赖本地 App、加密数据库、代理、证书或网页结构，维护成本高。
- 会让 v0.4.x 最重要的用户首次体验被稀释。

## 对 MoonPub 路线的建议

近期仍按这个顺序推进：

1. 先把 Obsidian 插件首页、飞书、照片和当前文章路径做到证据充分。
2. 再补 `preflight` 这类只读聚合质量门，让用户发布前知道缺什么。
3. 公众号归档只从单篇公开 URL 开始，不碰批量历史和指标。
4. 其它平台抓取、微信本地库、视频工作流只做研究，不进入 v0.4.x / v0.5 主线。

一句话：

**MoonPub 要学它的“本地优先、草稿优先、证据优先、安全边界优先”，不要学成一个无边界的万能抓取器。**
