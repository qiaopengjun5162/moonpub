# 我用 Rust 写了一个公众号发布副驾驶：从 Obsidian 到微信后台，少点十几次鼠标

> 这是一篇发布文章初稿。对外发布前建议补 3 张图：本地预览截图、封面截图、微信后台 ready draft 截图。

## 开头

我一直有一个很具体的痛点：文章已经在 Obsidian 里写好了，但真正发到微信公众号，还是要重复很多机械动作。

复制正文，打开后台，调整排版，上传图片，设置封面，打开原创声明，设置赞赏，设置留言，选择创作来源，发送预览，再同步到博客。

这些动作本身不难，但每次都要点。写得越多，越觉得累。

所以我写了 MoonPub。

它不是一个“无人值守自动发文机器人”，而是一个本地运行的公众号发布副驾驶：把能确定、可复现、重复性的工作交给程序，把最终确认和发布留给人。

## MoonPub 是什么

MoonPub 是一个纯 Rust 写的 CLI 工具，用来把 Obsidian / Markdown 文章推进到微信公众号后台“可人工确认发布”的状态。

它做的事情大概是：

```text
Markdown 文章
  -> 生成微信友好的 HTML
  -> 生成封面
  -> 上传本地图片
  -> 调微信官方 API 创建草稿
  -> 用本地 Chrome 辅助配置后台选项
  -> 导出到 Zola 博客
  -> 人工检查并发布
```

一条命令就是：

```bash
moonpub ship article.md --style literary
```

如果你没有微信凭证，也可以先只体验本地渲染：

```bash
moonpub init
moonpub new "我的第一篇 MoonPub 文章"
moonpub render Articles/drafts/我的第一篇 MoonPub 文章.md
moonpub preview Articles/drafts/我的第一篇 MoonPub 文章.md
moonpub cover Articles/drafts/我的第一篇 MoonPub 文章.md --style literary
```

这条路径不会调用微信 API，也不会控制浏览器。

## 为什么不是只做微信 API

只调用微信 API 创建草稿，其实不算太有意思。

真正麻烦的是草稿之后的那些后台设置：原创声明、赞赏、留言、创作来源、预览、合集、封面确认。

这些地方很多没有稳定 API，只能在微信后台里点。

所以 MoonPub 的核心价值不是“我又封了一层 API”，而是把一条完整工作流串起来：

- Markdown 本地写作
- 微信排版渲染
- 封面生成
- 图片上传
- 草稿创建
- 后台配置辅助
- 博客导出

它解决的是“写完之后的重复劳动”。

## 浏览器自动化是不是有风险

有。

所以我没有把 MoonPub 定位成群控工具，也没有做绕过验证码、绕过扫码、绕过审核、自动点击最终发表。

MoonPub 的浏览器自动化只做一件事：在用户自己登录的本地 Chrome 会话里，辅助完成重复配置步骤。

边界很清楚：

- 用户自己扫码登录。
- 不绕过验证码。
- 不绕过平台审核。
- 不绕过账号权限。
- 不默认代替用户最终发布。
- 微信后台 UI 变化时，自动化步骤允许软失败。

这也是为什么我更愿意叫它“发布副驾驶”，而不是“自动发文机器人”。

## 当前能做什么

目前 MoonPub 已经支持：

- `render`：Markdown 转微信公众号 HTML。
- `preview`：本地浏览器预览。
- `cover`：生成 10 种风格封面。
- `push`：通过微信官方 API 创建草稿。
- `configure`：辅助配置原创、赞赏、留言、创作来源和预览。
- `ship`：串起封面、渲染、推送、配置和博客导出。
- `export`：导出到 Zola 博客。
- `humanize`：本地规则去 AI 味。
- `radar`：记录热点样本，辅助标题分析。

目前项目状态是 Beta。适合技术用户试用，不适合把它当成完全稳定的生产系统。

## 当前还没有完成什么

我现在不会把它说成“生产级稳定”。

还差这些：

- 微信后台 UI 变化时，需要持续维护自动化步骤。
- 合集选择还没有默认启用。
- 最终发表按钮不会默认自动点击。
- 浏览器自动化缺少真实 UI 回归测试。
- Homebrew tap 还没有正式发布。
- 需要更多真实文章样本来打磨排版。

但对我自己的工作流来说，它已经明显省力了。

## 为什么用 Rust

主要是三个原因：

第一，我希望它是一个本地工具，不依赖一堆外部运行时。

第二，Rust 写 CLI、文件处理、HTTP 客户端和测试都很舒服。

第三，这个工具会处理文章、配置和凭证，我希望它尽量可审计、可复现、少魔法。

MoonPub 的核心渲染不依赖 AI，也不需要第三方 SaaS。AI 写作、润色这些能力是可选的，只有用户显式使用 `write` / `expand` / `polish` / `ship --ai` 时才会调用 DeepSeek。

## 怎么开始

项目地址：

```text
https://github.com/qiaopengjun5162/moonpub
```

安装：

```bash
cargo install --git https://github.com/qiaopengjun5162/moonpub
```

或者从 GitHub Releases 下载二进制。

建议第一次这样试：

```bash
moonpub init
moonpub new "我的第一篇 MoonPub 文章"
moonpub render Articles/drafts/我的第一篇 MoonPub 文章.md
moonpub preview Articles/drafts/我的第一篇 MoonPub 文章.md
```

确认本地渲染没问题后，再配置微信凭证：

```bash
export WECHAT_APPID=wx***
export WECHAT_SECRET=your_secret
moonpub login
moonpub push Articles/drafts/我的第一篇 MoonPub 文章.md --render
```

推送成功后，文章会进入本地 `Articles/ready/`，表示它是“可人工确认发布”的状态。

## 结尾

我做 MoonPub 的初衷很简单：写文章已经够难了，发布不应该再消耗那么多机械注意力。

它现在还不完美，但已经能把很多重复动作从“每次手点”变成“本地可复现的流程”。

如果你也在用 Markdown / Obsidian 写公众号文章，欢迎试用，也欢迎提 issue。

我会继续把它打磨成一个稳定、诚实、好用的公众号发布副驾驶。
