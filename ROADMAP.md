# MoonPub Roadmap

MoonPub 的长期方向是本地优先的写作发布副驾驶，而不是云端代发 SaaS。

核心原则：

- 本地运行优先：文章、配置和凭证默认留在用户机器上。
- 官方 API 优先：能用稳定 API 的平台优先接入，网页自动化只做辅助。
- 人工确认优先：不绕过扫码、验证码、平台审核、账号权限或最终发布确认。
- 插件化优先：先把核心能力做成可扩展接口，再做完整 App。

## 现状

当前源码版本：v0.4.2。

最新已验证公开 release 资产：v0.4.1。

当前定位：Beta / 技术用户可试用。

已经完成：

- Markdown / Obsidian 到微信公众号 HTML 和 draft JSON。
- 10 种封面风格、本地预览截图素材，以及 23 套正文主题 / 20 种 Block。
- 微信官方 API 草稿创建、更新、图片上传。
- CDP 浏览器辅助配置原创、赞赏、留言、创作来源和预览；`wechat-health` 可提前检查持久登录态，headless 登录态失效会快速提示恢复。
- Zola 导出。
- macOS / Linux / Windows release 资产。
- Obsidian 插件首页雏形，已接入当前文章、飞书和照片三条首次入口。
- `layout-audit` 质量门和 `layout-recipes` 排版配方发现入口。

仍未完成：

- 真实微信账号回归截图和短录屏仍未归档。
- 插件首页、飞书、照片三条首次体验证据仍未补齐。
- 浏览器自动化真实 UI 回归覆盖仍偏人工。
- 插件化平台接口。
- Obsidian 插件正式发布流程。
- WordPress / Ghost 等多平台发布。
- 本地 App / Pro 版产品形态。

## v0.4.2: 真实微信回归

目标：把 v0.4.1 从“本地闭环已验证”推进到“真实微信公众号草稿路径更可信、证据可复查”。

范围：

- 保留 `moonpub login` 作为显式扫码刷新命令。
- 保留 `wechat-health` 作为发文前浏览器自动化预检。
- 保留 headless fail-fast 行为：不可见窗口不等待用户扫码。
- 用真实凭证和 IP 白名单完成微信草稿路径人工回归。
- 补微信草稿、`configure --headed`、后台预览发送截图，隐藏 AppSecret、access token、手机号和账号隐私。
- 补首次体验证据：插件首页、飞书结果工作台、照片结果工作台。
- 完成 macOS release 二进制、Windows zip、无凭证本地路径 smoke。

不做：

- 不默认点击最终发表。
- 不把文案升级为生产稳定。
- 不新增高维护平台。
- 不拆飞书为单独项目。

## v0.5: 插件化核心

目标：把 MoonPub 从单一 CLI 命令集合整理成可扩展的本地发布内核。

设计草案见 [docs/PLUGIN_ARCHITECTURE_ZH.md](docs/PLUGIN_ARCHITECTURE_ZH.md)。

优先扩展点：

- `PlatformPublisher`：发布到微信公众号、WordPress、Ghost、静态博客等平台。
- `Renderer`：微信 HTML、博客 Markdown、未来可能的富文本目标。
- `CoverTheme`：封面模板和品牌样式扩展。
- `ExportTarget`：Zola、Hugo、WordPress、Ghost。
- `Hook`：发布前检查、发布后导出、通知、归档。

设计约束：

- 插件默认不读取真实凭据，凭据仍通过环境变量、本地 `.env` 或本地配置提供。
- 插件失败必须可诊断，不能吞掉发布错误。
- 微信浏览器自动化不开放成“绕过平台”的扩展点。

## v0.6: Obsidian 插件正式化

目标：把已有 `obsidian-plugin/` 从实验入口打磨成正式写作入口。

优先体验：

- 从当前笔记直接运行 `render` / `preview` / `cover` / `check`。
- 清晰展示文章当前状态：drafts / ready / published。
- 调用本地 `moonpub` 二进制，不在插件内重新实现发布逻辑。
- 只在用户明确触发时调用 `push` / `ship`。
- 明确提示哪些命令会触达微信 API 或打开 Chrome。

不做：

- 不在插件里保存 AppSecret。
- 不把 Obsidian 插件描述成全自动发布机器人。
- 不在插件里绕过微信后台确认。

## v0.7: 多平台发布

目标：在微信之外接入低风险、官方 API 或文件系统友好的平台。

优先级：

1. Zola / Hugo：静态博客导出，维护成本低。
2. WordPress：官方 REST API，适合内容站和技术用户。
3. Ghost：官方 Admin API，适合独立博客和 newsletter。
4. 其它平台：知乎、掘金、小红书、B 站专栏等只做研究，不作为第一批承诺。

判断标准：

- 是否有稳定官方 API。
- 是否允许本地用户自带凭证。
- 是否能保留人工确认边界。
- 是否不会把维护成本推到不可控。

## v1.0: 产品化和商业化

目标：在核心流程真实稳定后，探索可持续商业模式。

可行方向：

- 开源核心：CLI、本地渲染、基础微信草稿推送继续免费。
- Pro 版：高级主题、封面模板、批量发布、多账号管理、平台插件。
- 模板市场：公众号排版主题、封面风格、文章结构模板。
- 工作室版：多账号、发布审核、协作记录、团队配置。
- 服务收入：为自媒体团队配置 Obsidian + MoonPub + 博客 + 公众号工作流。

暂不建议：

- 云端托管 AppSecret。
- 云端代发公众号文章。
- 以绕过平台审核或自动最终发表作为卖点。

## 当前下一步

1. 补齐 v0.4.2 release gate：真实微信截图 / 录屏、首次体验证据、release smoke。
2. 把回归发现的问题收敛到 v0.4.2，但不新增平台或自动最终发表。
3. 按 `docs/PLUGIN_ARCHITECTURE_ZH.md` 拆出 v0.5 插件化核心的第一批内部接口。
4. 梳理 `obsidian-plugin/` 的安全边界、配置体验和正式发布清单。

如果当前讨论的重点不是“版本号往前推多少”，而是“项目现在到底该怎么定位、飞书路线是否拆分、先做什么后做什么”，请先看 [docs/PRODUCT_EVALUATION_ZH.md](docs/PRODUCT_EVALUATION_ZH.md)。它是基于当前代码和文档现状写的整体评估与阶段计划。
