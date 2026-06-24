# MoonPub

![Rust Version](https://img.shields.io/badge/rust-%3E%3D1.85-blue)
![License](https://img.shields.io/badge/license-MIT-green)

纯 Rust CLI，本地公众号发布副驾驶：Markdown → 渲染文章 → 微信草稿 → 辅助配置后台 → 同步导出博客。

## 项目状态

MoonPub 当前处于 **Beta / 技术用户可试用** 阶段。

如果你能配置微信公众号 AppID / AppSecret，并愿意在发布前检查草稿，它已经可以用于真实工作流。没有微信凭证时，也可以先跑本地渲染和预览路径，确认排版、Block 模板和封面效果。

MoonPub 不是无人值守发布机器人，也不是群控工具。稳定核心是本地渲染和微信官方 API 草稿推送；浏览器自动化是辅助驾驶，用来减少微信后台里的重复点击，最终发布仍由用户自己确认。

当前限制：

- `push` / `ship` 会触达微信 API，`login` / `configure` 会打开或控制 Chrome。
- 浏览器自动化依赖微信后台实时页面，微信改 DOM 或文案时，部分配置步骤可能软失败。
- 浏览器自动化不绕过扫码、验证码、平台审核、账号权限或最终人工确认。
- Homebrew tap 尚未发布，当前推荐使用 release 二进制或 Cargo 安装。
- `write` / `expand` / `polish` / `ship --ai` 是可选 DeepSeek 功能；核心渲染和推送流程不依赖 AI。

```bash
moonpub render article.md
moonpub preview article.md
moonpub ship article.md --style literary
```

## v0.4.1 演示效果

下面两张图来自 v0.4.1 release 二进制生成的本地演示素材，没有读取或触达真实微信凭证。

![MoonPub 本地文章预览](docs/assets/launch/01-preview.png)

![MoonPub literary 风格封面](docs/assets/launch/02-cover.png)

## 快速开始

### 不需要微信凭证：先本地体验

```bash
moonpub init                         # 创建 moonpub.toml
moonpub new "我的第一篇文章"          # 创建文章（带 frontmatter 模板）
moonpub render Articles/drafts/我的第一篇文章.md
moonpub preview Articles/drafts/我的第一篇文章.md
moonpub cover Articles/drafts/我的第一篇文章.md --style literary
```

如果标题里有空格，文件名会把空格转成 `-`；后续命令以 `moonpub new` 打印出的路径为准。

### 需要微信凭证：推送到草稿

```bash
export WECHAT_APPID=wx***
export WECHAT_SECRET=your_secret
moonpub login
moonpub push Articles/drafts/我的第一篇文章.md --render
```

或者一键发布：

```bash
moonpub ship Articles/drafts/我的第一篇文章.md --style literary
```

## 配置

```bash
moonpub init    # 创建默认 moonpub.toml
```

```toml
[articles]
root = "/path/to/ObsidianMain"

[wechat]
appid = "wx..."
author = "寻月隐君"
theme = "geek"                 # default | warm | dark | geek
account_type = "personal"      # personal | verified | service | wecom
auto_publish = false            # 推荐保持 false，最终发布由人工确认
thumb_media_id = ""             # 默认封面图 media_id（ship 会自动上传刷新）
qrcode = "Context/assets/qrcode.jpg"

[footer]
enabled = true
title = "加入「我的社群」"
description = "欢迎每一位对技术保持热爱与好奇心的朋友。"
rules = "· 亮出身份，以诚会友\n· 专注技术，言之有物\n· 君子之交，和而不同\n· 广告勿扰，保持纯粹"
qrcode = "Context/assets/qrcode-group.jpg"
qrcode_note = "长按下方二维码即可入群。\n若二维码过期，请在公众号后台回复 加群 获取最新二维码。"
follow_image = ""
follow_text = "点个「赞」让我知道你喜欢，点个「推荐」让更多人看到。"
divider = "— · —"

[blog]
kind = "zola"
root = "/path/to/blog"
```

**优先级:** 环境变量 > .env 文件 > moonpub.toml

## 一键发布副驾驶 (ship)

`ship` 命令把文章推进到“微信后台可人工确认发布”的状态：

```bash
moonpub ship article.md --style literary
```

流程：封面截图 → 渲染 HTML → API 推送草稿 → 浏览器辅助配置 → 导出博客 → 人工检查并发布

支持的 style：`dark` / `clean` / `minimal` / `warm` / `serif` / `gradient` / `literary`（默认）/ `ink` / `sunset` / `forest`

首版发布前的验收清单见 [docs/RELEASE_CHECKLIST.md](docs/RELEASE_CHECKLIST.md)。如果你想对外介绍项目，先看 [docs/LAUNCH_READY_ZH.md](docs/LAUNCH_READY_ZH.md) 的最终可发布状态，再看 [docs/LAUNCH_PLAN_ZH.md](docs/LAUNCH_PLAN_ZH.md) 的目标和进度条，最后从 [docs/LAUNCH_ARTICLE_ZH.md](docs/LAUNCH_ARTICLE_ZH.md) 的发布稿开始改。长期插件化、多平台、App 和商业化路线见 [ROADMAP.md](ROADMAP.md)。

## 浏览器自动化 (CDP)

API 推送后，微信草稿还需手动配置：原创声明、赞赏、留言、创作来源、预览。MoonPub 通过 Chrome DevTools Protocol 辅助完成这些重复步骤。

这是本地辅助驾驶，不是绕过平台：

- 用户自己扫码登录，MoonPub 只复用本地浏览器会话。
- 不绕过验证码、审核、权限限制或账号风控。
- 最终发布仍由用户在微信后台人工确认。
- 微信后台页面变化时，自动化步骤应软失败，不能影响 API 草稿推送主流程。

首次需扫码登录一次（打开浏览器）：

```bash
moonpub login
```

之后完全静默 headless：

```bash
moonpub configure                    # 全部步骤
moonpub configure zanshang chuangzuo # 指定步骤
moonpub configure --headed           # 调试：可见浏览器 + 截图
```

当前自动化状态：

| 步骤 | 状态 |
|------|------|
| 原创声明 | ✅ |
| 赞赏 | ✅ |
| 留言 | ✅ |
| 创作来源 | ✅ |
| 预览 | ✅ |
| 合集 | ⏸ 已禁用 |

## Block 模板系统

在 Markdown 中使用 `:::blockname` 语法：

```markdown
:::book-info
title: 书名
author: 作者
cover: https://...
publisher: 出版社
rating: 8.1
:::

:::intro
1-3 句话导语，迅速抓住读者
:::

:::callout
label: 核心结论
这里写你最想让读者带走的一句话
:::

:::steps
1. 第一步说明
2. 第二步说明
3. 第三步说明
:::

:::summary
结尾总结
:::
```

支持的 12 种 Block：`book-info` / `intro` / `callout` / `steps` / `summary` / `figure` / `checklist` / `cover` / `quote-card` / `divider` / `concept-card` / `emotion-card`

## 去 AI 味

```bash
moonpub humanize article.md              # 单独处理
moonpub render --humanize article.md     # render 时一并处理
```

6 阶段规则处理：填充短语 → AI 词汇替换 → 排比打破 → 修饰简化 → 通用结论 → 破折号清理

## 封面生成

```bash
moonpub cover article.md                    # 默认 literary 风格
moonpub cover article.md --style dark       # 深色风格
moonpub cover article.md --style warm       # 暖色风格
moonpub cover article.md --screenshot       # 同时生成 PNG
```

## 全部命令

```bash
moonpub new <title>               # 创建新文章（带 frontmatter 模板）
moonpub --version                 # 显示版本号
moonpub write <idea>              # 从想法生成文章（DeepSeek）
moonpub expand <article.md>       # 读书笔记展开成文章（DeepSeek）
moonpub polish <article.md>       # AI 润色 + 去 AI 味（DeepSeek）
moonpub init [path]               # 创建配置
moonpub status                    # 查看文章流水线 + 状态追踪
moonpub capabilities              # 查看内置发布/导出 target 能力和风险提示
  --json                          # 输出给 Obsidian 插件 / 本地 App 读取的原始 JSON
moonpub check <article.md>        # 检查文章三件套
moonpub render <article.md>       # Markdown → HTML + draft.json
moonpub preview <article.md>      # 浏览器预览
moonpub push <article.md>         # 推送到微信草稿，并移动到 ready/
  --render                        # push 前自动 render
moonpub update-draft <article.md> # 更新已有草稿
moonpub export <article.md>       # 导出 Zola 博客
moonpub humanize <article.md>     # 去 AI 味
moonpub cover <article.md>        # 生成封面
  --style dark|clean|minimal|warm|serif|gradient|literary|ink|sunset|forest
  --screenshot                    # 导出 PNG
moonpub ship <article.md>         # 发布副驾驶：封面 + 渲染 + 推送 + 配置 + 导出
  --style dark|clean|minimal|warm|serif|gradient|literary|ink|sunset|forest

moonpub login                     # 扫码登录，保存 cookie
moonpub configure [<steps>] [--headed]  # 自动配置草稿设置
moonpub test-zanshang [--headed]  # 调试赞赏步骤
moonpub test-chuangzuo [--headed] # 调试创作来源步骤
moonpub test-yulan [--headed]     # 调试预览步骤
moonpub list-drafts               # 列出所有微信草稿
moonpub delete-draft <media_id>   # 删除草稿

moonpub radar add --platform <name> --keyword <kw> --title <title>
moonpub radar list [--platform <name>]
moonpub radar import <file.csv>
moonpub radar analyze <article.md> --platform <name>
moonpub radar suggest <article.md> --platform <name>
moonpub radar scrape --platform <name> --keyword <kw>
```

全局 flag：`--articles <path>` / `--config <moonpub.toml>` / `--json`

## 开发

```bash
cargo fmt
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run
```

使用 `cargo nextest`，不是 `cargo test`。

## 架构

- Zero AI dependencies — 所有转换完全离线、确定性的
- CLI 解析: `src/cli.rs`
- 配置: `src/config.rs`
- 文章 / frontmatter 工具: `src/article.rs`
- WeChat API 客户端: `src/wechat.rs` (ureq, 零 SDK)
- Markdown → HTML: `src/markdown.rs`
- Block 模板: `src/illustrate.rs`
- CDP 自动化原语: `src/cdp.rs`
- 编辑器自动化步骤: `src/publish_steps.rs`
- 浏览器自动化编排: `src/publish.rs`
- 去 AI 味: `src/humanize.rs`
- 封面生成: `src/cover.rs`
- 所有样式 inline CSS，微信兼容

## 贡献

使用 PR-first 工作流。创建 `codex/<short-topic>` 分支，保持改动聚焦，运行 `cargo clippy` 和 `cargo nextest`，推送分支，向 `main` 发起 PR。详见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## 参考资料

- [docs/REFERENCES.md](docs/REFERENCES.md) — 30+ 参考项目文档
- [docs/BROWSER_AUTOMATION.md](docs/BROWSER_AUTOMATION.md) — 浏览器自动化参考

## License

MIT

## 贡献者

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/qiaopengjun5162"><img src="https://avatars.githubusercontent.com/u/124650229?v=4?s=100" width="100px;" alt="Paxon Qiao 乔鹏军"/><br /><sub><b>Paxon Qiao 乔鹏军</b></sub></a><br /><a href="https://github.com/qiaopengjun5162/moonpub/commits?author=qiaopengjun5162" title="Code">💻</a> <a href="#doc-qiaopengjun5162" title="Documentation">📖</a> <a href="#ideas-qiaopengjun5162" title="Ideas">🤔</a> <a href="#projectManagement-qiaopengjun5162" title="Project Management">📆</a></td>
    </tr>
  </tbody>
</table>
<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->
<!-- ALL-CONTRIBUTORS-LIST:END -->
