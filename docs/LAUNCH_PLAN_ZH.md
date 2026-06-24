# MoonPub 首版对外发布计划

这份计划给队友、早期用户和准备写发布文章时使用。它回答三个问题：最终目标是什么，现在处在哪一步，下一步具体做什么。

最终可发布状态见 [LAUNCH_READY_ZH.md](LAUNCH_READY_ZH.md)。

长期插件化、多平台、App 和商业化路线见 [../ROADMAP.md](../ROADMAP.md)。

## 最终目标

MoonPub 要成为一个本地公众号发布副驾驶：作者在 Obsidian / Markdown 中完成写作后，可以用可审计、可复现的 Rust CLI 完成本地渲染、封面生成、微信 API 草稿推送、后台重复配置辅助和博客导出。

它不是无人值守自动发布机器人。最终发表、平台审核、账号权限、扫码登录和验证码仍由用户自己处理。

## 当前位置

整体进度：`█████████░` 87%

| 阶段 | 状态 | 说明 |
|------|------|------|
| 本地写作闭环 | `█████████░` 90% | `init` / `new` / `render` / `preview` / `cover` / `check` 已能完成无凭证首跑 |
| 微信草稿推送 | `████████░░` 85% | API 草稿创建、更新、本地图片上传可用；仍需真实微信回归 |
| 浏览器辅助配置 | `███████░░░` 70% | 原创、赞赏、留言、创作来源、预览已有自动化；微信 UI 变化时可能软失败 |
| 对外安装体验 | `█████████░` 88% | v0.4.1 已发布五个平台资产，macOS ARM64 release 二进制通过 smoke test |
| 对外材料 | `██████████` 96% | README、上手指南、发布文章、演示素材记录、本地截图、截图清单和回归清单已有；还缺真实微信截图/短录屏 |

## v0.4.1 能给别人用吗

可以给技术用户试用，但要诚实说明边界。

适合：

- 能配置微信公众号 AppID / AppSecret 的用户。
- 愿意先跑本地预览，再检查微信草稿的人。
- 接受 Beta 工具需要反馈问题、偶尔更新的人。

暂不适合：

- 期待完全无人值守自动发布的人。
- 不愿意处理微信 IP 白名单、扫码登录和草稿检查的人。
- 需要稳定企业级 SLA 的团队。

## 下一组小目标

1. 做真实微信草稿回归：回归清单已补齐；下一步用真实凭证验证 `login`、`push --render`、`configure --headed`，但不打印或提交凭据。
2. 补真实微信截图/短录屏：本地预览、封面、`status` / `check` 已完成；只剩微信草稿和 `configure --headed` 证据。
3. 收集首批用户反馈：在 README 和文章中明确 issue、适用人群、已知限制。
4. 进入 v0.4.2：优先修复真实微信回归发现的问题，而不是盲目新增功能。
5. 准备 v0.5 设计：插件化核心先行，Obsidian 插件正式化和 WordPress / Ghost 多平台发布随后推进。

## 下一步

下一步先做真实微信草稿回归，并补微信后台截图/短录屏。

推荐顺序：

```bash
moonpub --version
moonpub init
moonpub new "我的第一篇 MoonPub 文章"
moonpub render "Articles/drafts/我的第一篇-MoonPub-文章.md"
moonpub cover "Articles/drafts/我的第一篇-MoonPub-文章.md" --style literary
moonpub check "Articles/drafts/我的第一篇-MoonPub-文章.md"
```

确认本地体验后，再在有凭证的环境中执行：

```bash
moonpub login
moonpub push Articles/drafts/文章名.md --render
moonpub configure --headed
```

真实微信回归完成前，对外文案只写“可试用 / Beta”，不写“生产稳定”。

演示素材生成记录见 [LAUNCH_DEMO_ASSETS_ZH.md](LAUNCH_DEMO_ASSETS_ZH.md)，本地预览和封面 PNG 已进入 `docs/assets/launch/`。

截图清单见 [LAUNCH_SCREENSHOT_CHECKLIST_ZH.md](LAUNCH_SCREENSHOT_CHECKLIST_ZH.md)，真实微信回归清单见 [WECHAT_REGRESSION_CHECKLIST_ZH.md](WECHAT_REGRESSION_CHECKLIST_ZH.md)。
