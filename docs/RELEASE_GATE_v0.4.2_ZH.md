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
- 微信草稿创建截图
- `configure --headed` 截图
- 预览发送结果截图

不要提交 token、cookie、二维码、AppSecret、手机号、账号隐私或不可公开照片。

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
- [ ] 真实微信路径人工回归通过或失败原因已记录
- [ ] 首次体验证据目录至少补齐首页、飞书、照片三类核心截图
- [ ] README / README_zh / USER_GUIDE / PROGRESS 与 release 事实一致
- [ ] 没有真实凭据、token、二维码或隐私截图被提交
