# MoonPub 首发截图清单

这份清单用于准备 README、发布文章和社媒介绍图。截图可以人工完成，不要求自动化；但素材必须来自 v0.4.1 release 二进制生成的文件。

## 输入素材

先确认演示素材已经生成，记录见 [LAUNCH_DEMO_ASSETS_ZH.md](LAUNCH_DEMO_ASSETS_ZH.md)。

当前可截图文件：

- 本地预览 HTML：`/private/tmp/moonpub-launch-demo/Articles/drafts/我的第一篇-MoonPub-文章.html`
- 封面 HTML：`/private/tmp/moonpub-launch-demo/Articles/drafts/我的第一篇-MoonPub-文章.cover.html`

## 截图交付物

建议输出到 `/private/tmp/moonpub-launch-demo/screenshots/`：

- [ ] `01-preview.png`：本地文章预览，展示标题、导语和正文排版。
- [ ] `02-cover.png`：literary 风格封面，展示标题、摘要和作者。
- [ ] `03-check-output.png`：终端 `moonpub check` 输出，展示 `publishable: yes`。
- [ ] `04-status-output.png`：终端 `moonpub status` 输出，展示 drafts/ready/published 三段状态。
- [ ] `05-wechat-draft.png`：真实微信草稿截图，必须在真实微信回归后再补。
- [ ] `06-configure-headed.png`：`moonpub configure --headed` 可见模式截图，必须在真实微信回归后再补。

## 人工截图建议

本地预览和封面：

```bash
open "/private/tmp/moonpub-launch-demo/Articles/drafts/我的第一篇-MoonPub-文章.html"
open "/private/tmp/moonpub-launch-demo/Articles/drafts/我的第一篇-MoonPub-文章.cover.html"
```

终端输出：

```bash
cd /private/tmp/moonpub-launch-demo
/private/tmp/moonpub-release-smoke-v041/moonpub check "Articles/drafts/我的第一篇-MoonPub-文章.md"
/private/tmp/moonpub-release-smoke-v041/moonpub status
```

注意：

- 不要截图 `.env`、`moonpub.toml` 里的真实凭据或本地私密路径。
- 微信后台截图前先确认没有暴露 AppSecret、access token、手机号、未公开文章内容或账号隐私信息。
- 对外文章优先使用本地预览、封面和安全终端输出；微信后台截图只作为“真实草稿回归证据”，不必展示敏感区域。
