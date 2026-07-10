# MoonPub v0.4.2 Release Gate

这份文档只回答一个问题：

**v0.4.2 什么时候可以发布。**

v0.4.2 的目标不是继续加功能，而是让真实微信路径、首次体验路径和 release 资产更可信。

## 发布判断

当前源码已经具备 v0.4.2 的主要能力：

- 公众号草稿 API 路径可用
- `wechat-health` 可做浏览器自动化预检
- headless 登录态失效会快速失败，不再等待不可见二维码
- `layout-audit` 可检查公众号 HTML 兼容风险
- `preflight` 可在触达微信 API 前聚合检查文章包、排版审计和下一步动作
- `moonlit` / `porcelain` / `fieldnote` 已补齐生活合集排版选择
- 插件首页、飞书、照片、当前文章四类入口已形成

但 v0.4.2 不能只因为源码测试通过就发布。发布前还需要补齐下面这些证据。

## 必须完成

### 1. 无凭证 smoke

至少覆盖：

```bash
cargo build --release --all-features
target/release/moonpub --version
target/release/moonpub init /private/tmp/moonpub-smoke
```

如果验证 release 资产，必须用下载或打包后的二进制，不用源码构建结果替代。

当前记录：

- 2026-07-09：本机源码 release build smoke 通过。已运行 `cargo build --release --all-features`、`target/release/moonpub --version`（输出 `moonpub 0.4.2`）和 `target/release/moonpub init /private/tmp/moonpub-smoke-v042`。这只证明当前源码构建出的 release 二进制可跑通无凭证初始化路径，不能替代正式 release 资产下载验证。

### 2. CI 与 Windows smoke

发布前确认：

- PR build 通过
- `windows-smoke` 通过
- release workflow 会验证 Windows zip 内的 `moonpub.exe`

当前记录：

- 2026-07-09：PR #93 / #94 均已通过 `test` 与 `windows-smoke`。其中 #94 的 GitHub Actions run `29009548275` 显示 `test` pass、`windows-smoke` pass，Windows smoke 覆盖 release binary build 和无凭证 smoke workflow。

### 3. 真实微信人工回归

至少完成一次真实账号路径：

```bash
moonpub wechat-health
moonpub push <article.md> --render
moonpub configure --headed
```

必须确认：

- 微信草稿创建成功
- 原创声明 / 赞赏 / 留言 / 创作来源至少能尝试配置
- 微信公众号后台预览发送成功或失败原因被记录
- 最终发表仍由人工确认，不自动点击发表

### 4. 真实证据归档

至少补下面这些截图或录屏，并裁掉敏感信息：

- `docs/first-run-evidence/homepage/homepage-workspace.png`
- `docs/first-run-evidence/homepage/homepage-context.png`
- `docs/first-run-evidence/feishu/feishu-result-modal.png`
- `docs/first-run-evidence/photos/photos-result-modal.png`
- `docs/first-run-evidence/wechat/wechat-draft-created.png`
- `docs/first-run-evidence/wechat/configure-headed.png`
- `docs/first-run-evidence/wechat/preview-sent.png`

不要提交 token、cookie、二维码、AppSecret、手机号、账号隐私或不可公开照片。

可以在仓库根目录运行下面的本地只读命令，快速查看这些证据文件是否已经归档：

```bash
moonpub evidence-status
moonpub --json evidence-status
moonpub evidence-status --strict
```

这个命令会显示已归档 / 必需 / 缺失数量和缺失路径清单；它只检查文件是否存在，不打开截图、不读取图片内容，也不替代人工脱敏审查。默认模式只报告状态，适合插件首页和人工查看；`--strict` 在缺少必需证据时会非零退出，适合 release 脚本或 CI gate。

如果要看整个 v0.4.2 release gate，而不只是证据文件，可以运行：

```bash
moonpub release-check
moonpub --json release-check
moonpub release-check --strict
```

`release-check` 会聚合本文件里的通过标准勾选状态和 `docs/first-run-evidence/` 文件存在检查；它仍然只做本地只读检查，不会触发微信 API、浏览器自动化或图片内容扫描。`--strict` 在任一 gate 未完成时会非零退出。

## 明确不做

v0.4.2 不做这些事：

- 不自动最终发表
- 不新增平台
- 不拆飞书为独立项目
- 不把 Obsidian 插件包装成独立发布器
- 不宣称普通用户已完全无门槛使用

## 发布文案口径

推荐口径：

> MoonPub v0.4.2 是一次真实微信路径和首次体验收口版本。它仍然是 Beta，适合能配置微信公众号凭证、愿意人工检查草稿的技术用户。

不要写：

> 全自动公众号发布。

也不要写：

> 无需扫码、无需人工确认。

## 通过标准

满足下面条件后，可以准备 v0.4.2：

- [x] 本地 release build smoke 通过
- [x] CI / Windows smoke 通过
- [x] 真实微信路径人工回归通过或失败原因已记录
- [ ] 首次体验证据目录至少补齐首页、飞书、照片和微信回归四类核心截图，并通过 `moonpub evidence-status --strict` 文件存在检查
- [x] README / README_zh / USER_GUIDE / PROGRESS 与 release 事实一致
- [ ] 没有真实凭据、token、二维码或隐私截图被提交

可以用 `moonpub release-check --strict` 在准备发版前做一次总门禁检查。

当前记录：

- 2026-07-10：README / README_zh / USER_GUIDE / PROGRESS / AGENTS / Obsidian 插件 README 已同步 `evidence-status`、`release-check`、插件首页入口和当前 v0.4.2 release gate 事实；`cargo run -- --json release-check` 仍显示真实微信回归、证据文件和隐私审查未完成。
- 2026-07-10：`git ls-files docs/first-run-evidence/**/*.png docs/first-run-evidence/**/*.jpg docs/first-run-evidence/**/*.jpeg docs/first-run-evidence/**/*.webp` 无输出，说明首次体验证据目录尚未提交截图图片；但仓库中已有 `Context/assets/qrcode.png` 这类历史二维码资产，隐私 / 二维码审查仍需人工确认后再勾选。
- 2026-07-10：`cargo run -- --json wechat-health` 返回 `status: ready`、`profile_mode: persistent`、`session_file_exists: true`，脱敏后的 `current_url` 为 `https://mp.weixin.qq.com/cgi-bin/home`。这说明当前浏览器自动化登录态可复用，可以继续跑真实微信草稿回归。
- 2026-07-10：使用 `/private/tmp/moonpub-wechat-regression/Articles/drafts/moonpub-v042-wechat-regression.md` 公开测试文运行 `cargo run -- --articles /private/tmp/moonpub-wechat-regression/Articles push drafts/moonpub-v042-wechat-regression.md --render`。结果：微信草稿创建成功，文章包移动到 `/private/tmp/moonpub-wechat-regression/Articles/ready`，`.media_id` 文件存在；浏览器自动化恢复 session 后进入编辑器，原创声明、赞赏、留言、创作来源均配置成功，`[template].name` 未配置时模板插入按设计跳过，微信公众号后台预览发送成功；未点击最终发表。`media_id` 已在公开文档中脱敏不记录。截图证据和隐私 / 二维码人工审查仍未完成，因此 release 总门禁仍不能通过。
