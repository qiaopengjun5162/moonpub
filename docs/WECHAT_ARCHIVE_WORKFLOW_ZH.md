# 微信公众号归档输入工作流

这份文档记录一个后续可做、但当前不应贸然塞进主发布链路的方向：

**把已发布的微信公众号文章安全归档到 MoonPub / Obsidian，再按统一 Inbox 模型继续整理、重写、迁移或二次发布。**

参考项目：

- [wechat-mp-batch-exporter](https://github.com/mcncarl/yichen-skills/tree/main/wechat-mp-batch-exporter)
- [moore-wechat-article-downloader](https://github.com/Moore-developers/moore-wechat-article-downloader)
- [mp-weixin-to-md](https://github.com/Noisepoint/mp-weixin-to-md)

这里吸收的是它们的工作流分层、输出边界和隐私原则，不复制外部代码。

## 为什么值得做

MoonPub 现在主要解决的是：

- 从 Obsidian / Markdown / 飞书 / 照片进入草稿
- 渲染公众号 HTML
- 推进到微信草稿
- 辅助后台配置

但很多作者还有另一个真实需求：

- 备份自己已经发过的公众号文章
- 把旧公众号内容迁移回 Obsidian
- 从历史文章里提取选题、标题、摘要和结构
- 对旧内容做合集整理、再编辑、再分发到博客

这条线不是“发布”，而是“归档输入源”。它更像飞书 / 照片的同级输入工作流。

## 建议产品定位

短期不要把它做成“批量爬公众号历史”的黑盒工具。

更合适的定位是：

**用户显式提供文章 URL 或自有账号授权后，MoonPub 把可访问的公开文章归档为 Inbox Item。**

这意味着：

- 默认只处理用户提供的公开 `mp.weixin.qq.com` URL
- 不绕过登录、权限、付费、删除、验证或平台风控
- 不承诺阅读数、点赞、评论等指标一定可得
- 不把 cookie、pass_ticket、uin、token、二维码登录信息写入仓库
- 不把下载下来的文章内容默认提交到开源仓库

## 三阶段路线

### Phase 1：已知 URL 归档

这是最适合作为第一步的范围。

输入：

- 一个或多个用户显式提供的公众号文章 URL

输出：

- `Inbox/WechatArchive/*.md`
- 可选 `raw.html`
- 可选 `metadata.json`

建议 Inbox frontmatter：

```yaml
---
source: wechat-mp-article
status: inbox
created: 2026-07-10
type: archived-article
external_id: "mp:<stable-url-or-hash>"
source_url: "https://mp.weixin.qq.com/s/..."
source_title: "原文章标题"
source_author: "公众号名称"
captured_at: "2026-07-10T12:00:00+08:00"
---
```

正文应尽量保留：

- 标题
- 作者 / 公众号名
- 发布时间，如果页面可获得
- 正文纯文本或 Markdown
- 原文 URL

这个阶段可以复用当前已经存在的 `fetch <url>` 方向，但不应该直接把 `fetch` 输出当作正式输入源。正式做法应是新增 `intake wechat-url ...` 或等价入口，让它对齐统一 Inbox 模型。

`moore-wechat-article-downloader` 对这个阶段的启发是：已知 URL 下载应保持为最小、低风险入口。它不需要公众号后台登录，不需要代理，也不应该混入历史列表、评论、阅读数或浏览增强逻辑。

`mp-weixin-to-md` 对这个阶段的启发更具体：主线应该是“文章链接 -> 标准 Markdown”，图片本地下载应是显式选项，本地 HTML 只作为验证页或网络失败后的备用输入，不内置 Cookie，也不绕过登录或验证页。

如果 MoonPub 后续实现 `intake wechat-url`，建议保持这组默认：

- 默认只保存 Markdown 和来源元数据。
- 默认保留远程图片 URL，不下载图片。
- 只有显式 `--download-assets` 才下载微信图片资源。
- 资源下载只允许常见微信图片域名，避免把任意外链变成本地抓取器。
- 支持 `--from-html <file>` 或等价备用入口，但它必须被标成手动提供 HTML 的 fallback，而不是绕过验证页的自动能力。

### Phase 2：历史列表索引

这一步用于处理“我有一个公众号，希望列出历史文章 URL”。

边界必须更谨慎：

- 只处理用户自己有权限访问的账号
- 任何扫码、登录、代理、证书信任、WeChat Desktop 操作都必须由用户确认
- Agent 不应代替用户操作微信 UI
- 不修改系统代理；如果未来确实需要本地 helper，也必须显式 `--dry-run` / `--confirm`

输出建议：

- `history.summary.json`
- `history.summary.md`
- `history.dedup.csv`
- `urls.all.txt`
- `urls.original.txt`

这些文件适合放在本地工作目录或 Obsidian 私有目录，不应默认提交。

历史列表应继续分层：

- Exporter / 官方可见列表：需要用户自行扫码登录或授权，但不碰系统代理。
- 订阅增量：只记录本地订阅状态和新 URL，不自动下载或重发。
- 代理历史：只能作为 Exporter 不可用时的备用方案，必须先显式确认，且只能列出实际加载到的文章。

MoonPub 短期只应设计前两类，不应默认实现代理历史。

### Phase 3：增强指标采集

阅读数、点赞、在看、评论、回复等数据属于更高风险能力。

只有在同时满足下面条件时才考虑：

- 用户确认这是自己的公众号或自己有合法权限
- 凭证新鲜且由用户本地持有
- 不打印、不存储、不提交敏感凭据
- 输出明确标注采集时间和可信度

这一步不应成为 v0.4.x / v0.5 的主线。

如果未来要参考“浏览时收藏文章和评论”的模式，必须单独标成浏览增强能力，而不是普通归档能力。它依赖页面实际加载状态，评论和指标都可能不完整，输出必须显式标注 `missing` / `observed_at`，不能猜测。

### Phase 4：浏览增强和代理能力

这一步目前只记录，不进入 MoonPub 近期路线。

典型能力包括：

- 修改系统代理。
- 通过微信桌面客户端内置浏览器观察页面。
- 保存已加载评论、互动数据和页面快照。
- 注入页面按钮或重置 WebView 进程。

这些能力的风险和维护成本都明显高于 MoonPub 当前主线。若未来确实要做，必须满足：

- 默认关闭。
- 每次启用前明确说明会修改系统代理。
- 提供恢复代理的强制收尾步骤。
- 不在对话、日志或仓库里保存 cookie、auth-key、pass_ticket、token。
- 与 `intake wechat-url` 这种低风险公开 URL 归档入口分离。

## 和当前 MoonPub 的关系

它不应该替代现有正式输入工作流。

当前正式输入工作流仍然是：

- 当前文章
- 飞书妙记
- 照片素材

公众号归档输入源更适合被标成：

- future workflow
- local archive workflow
- user-owned content workflow

后续如果进入 `workflow-registry`，也应先标为 `planned` 或 `experimental`，不能和飞书 / 照片一样被误认为已打通。

## 安全红线

实现时必须遵守：

- 不提交归档正文、账号数据、cookies、二维码 secret、pass_ticket、uin、token
- 不自动操作微信桌面端
- 不绕过登录、付费、删除、验证和平台权限
- 不承诺能抓取所有历史文章
- 不默认修改系统代理
- 不默认开启代理增强、WebView 注入或页面快照采集
- 不把评论、阅读数、点赞数、在看数当成稳定可得字段
- 不内置 Cookie，也不在验证码、空壳页或验证页场景下继续伪装成功
- 不把图片下载做成默认行为；本地化图片必须是用户显式选择
- 不把第三方导出工具的输出当成可再发布内容
- 不把“备份自己的文章”包装成“搬运别人内容”

## 对 MoonPub 的建议落点

第一步不写大功能，先做小闭环：

```bash
moonpub intake wechat-url <url> --draft --preview --no-open
```

建议能力：

- 输入一个公开文章 URL
- 抓取标题、作者、正文
- 写入 `Inbox/WechatArchive/`
- 默认生成标准 Markdown，Obsidian 图片语法只作为可选格式
- 图片默认保留远程 URL，显式选择后才下载到本地 assets 目录
- 复用 `draft-from-inbox`
- 先停在草稿和本地预览
- 只有显式 `--push` 才推进微信草稿

暂时不做：

- 批量历史列表抓取
- 评论 / 阅读数采集
- 代理配置
- 订阅增量同步
- 代理历史和浏览增强
- 自动绕过验证页、空壳页或风控页
- 微信桌面端自动操作
- 自动重发旧文章

## 验收标准

进入正式输入工作流前，至少需要：

- 一个本地公开 URL 样例的 Inbox 产物
- 一个本地 HTML fallback 样例的 Inbox 产物
- `--json` 输出包含 `command`、`action`、`inbox_path`、`draft_path`、`html_path`、`next_command`
- 默认不下载图片；显式下载时只允许微信图片域名，并把本地 assets 路径写回 Markdown
- 和飞书 / 照片同等级的 app 级回归测试
- 文档明确版权与权限边界
- 插件首页不默认展示高风险批量历史能力
- 任何历史列表 / 订阅 / 代理能力都不得复用 `intake wechat-url` 的低风险入口名

## 当前结论

这个方向值得做，但不要急着做成“批量公众号导出器”。

更好的第一步是：

**把单篇公开公众号文章 URL 做成 MoonPub 的一个安全输入源，归档到 Inbox，再复用现有草稿和预览链路。**
