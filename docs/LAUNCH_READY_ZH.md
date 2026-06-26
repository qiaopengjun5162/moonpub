# MoonPub v0.4.1 最终可发布状态

## 结论

MoonPub v0.4.1 **可以对外发布给技术用户试用**。

不要宣传成“生产级稳定”或“无人值守自动发文机器人”。准确口径是：

> MoonPub 是本地公众号发布副驾驶。稳定核心是 Markdown 本地渲染、封面生成和微信官方 API 草稿推送；浏览器自动化负责减少微信后台重复点击，但最终发布、扫码、验证码、审核和账号权限仍由用户自己处理。

## 今天可以给用户的路径

### 无微信凭证体验

这条路径已用 v0.4.1 macOS ARM64 release 二进制跑通：

```bash
moonpub init
moonpub new "我的第一篇 MoonPub 文章"
moonpub render "Articles/drafts/我的第一篇-MoonPub-文章.md"
moonpub cover "Articles/drafts/我的第一篇-MoonPub-文章.md" --style literary
moonpub check "Articles/drafts/我的第一篇-MoonPub-文章.md"
moonpub status
```

已生成素材：

- 本地预览 HTML：`/private/tmp/moonpub-launch-demo/Articles/drafts/我的第一篇-MoonPub-文章.html`
- 封面 HTML：`/private/tmp/moonpub-launch-demo/Articles/drafts/我的第一篇-MoonPub-文章.cover.html`
- 本地预览 PNG：`docs/assets/launch/01-preview.png`
- 封面 PNG：`docs/assets/launch/02-cover.png`
- 安全文本输出：`/private/tmp/moonpub-launch-demo/screenshots/00-version-output.txt`
- 安全文本输出：`/private/tmp/moonpub-launch-demo/screenshots/03-check-output.txt`
- 安全文本输出：`/private/tmp/moonpub-launch-demo/screenshots/04-status-output.txt`

### 真实微信草稿路径

这条路径可以给技术用户试用，但需要用户自己准备凭证和扫码：

```bash
export WECHAT_APPID=wx***
export WECHAT_SECRET=your_secret

moonpub login
moonpub push "Articles/drafts/文章名.md" --render
moonpub configure --headed
```

成功创建微信草稿后，本地文章包进入 `Articles/ready/`，不是 `Articles/published/`。

## 对外发布前还差什么

这些不是代码阻塞，而是需要用户/人工参与：

- 微信草稿截图：需要真实公众号凭证、IP 白名单和扫码登录。
- `configure --headed` 截图：同样需要真实微信后台环境。

本地预览和封面 PNG 已通过本机 Chrome headless 从 release demo HTML 生成。真实微信截图仍需要真实公众号环境。

## 当前仓库状态

- 当前版本：`0.4.1`
- Release：`https://github.com/qiaopengjun5162/moonpub/releases/tag/v0.4.1`
- Release 状态：非 draft，非 prerelease。
- Release 资产：macOS ARM64、macOS x86_64、Linux ARM64、Linux x86_64、Windows x86_64 均已产出。
- macOS ARM64 release 二进制：sha256 已验证，`--help` / `--version` / 无凭证首跑已验证。
- Windows：PR CI 已通过源码构建二进制的无凭证 smoke test；release workflow 也会自动解压并 smoke 测试打包后的 Windows zip 资产。
- 中文发布文章：`docs/LAUNCH_ARTICLE_ZH.md` 已整理为可直接发布稿。
- 首发本地截图：`docs/assets/launch/01-preview.png` 和 `docs/assets/launch/02-cover.png` 已生成。
- 测试：`cargo nextest run --all-features` 168 个测试通过；最近包含 Windows smoke 的 PR CI 已通过。
- CI：main 分支最近 PR 均通过 GitHub Actions。

## 最短发布文案

> MoonPub v0.4.1 发布了。它是一个用 Rust 写的本地公众号发布副驾驶，可以把 Markdown / Obsidian 文章渲染成微信 HTML，生成封面，通过微信官方 API 创建草稿，并辅助完成后台重复配置。它不是无人值守发文机器人，最终发布仍由用户人工确认。当前适合技术用户试用，推荐先跑无凭证本地路径，再接入微信凭证。

## 下一步优先级

1. 用真实公众号完成 `login`、`push --render`、`configure --headed` 回归。
2. 补真实微信草稿和 `configure --headed` 截图，注意隐藏账号隐私。
3. 根据真实回归结果修 v0.4.2，不再盲目新增功能。
