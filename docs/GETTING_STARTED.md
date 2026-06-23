# MoonPub 新手上手指南

跟着这个指南，先完成不需要微信凭证的本地预览，再按需进入真实微信草稿推送。

## 第一步：安装

选一种方式：

**macOS Apple Silicon（推荐）**：
```bash
curl -L https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.1/moonpub-macos-arm64.tar.gz | tar xz
sudo mv moonpub /usr/local/bin/
```

macOS Intel 用户把文件名换成 `moonpub-macos-amd64.tar.gz`，Linux x86_64 用户换成 `moonpub-linux-amd64.tar.gz`，Linux ARM64 用户换成 `moonpub-linux-arm64.tar.gz`。

**Cargo（需要 Rust）**：
```bash
cargo install --git https://github.com/qiaopengjun5162/moonpub
```

Homebrew 支持还在准备中，当前推荐直接下载 release 或使用 Cargo。

验证安装：
```bash
moonpub --help
```

---

## 第二步：本地体验（不需要微信凭证）

先创建一篇示例文章，确认 MoonPub 能生成 HTML 和封面。

```bash
moonpub init
moonpub new "我为什么开始每天读书"
moonpub render Articles/drafts/我为什么开始每天读书.md
moonpub preview Articles/drafts/我为什么开始每天读书.md
moonpub cover Articles/drafts/我为什么开始每天读书.md --style literary
```

这一步只读写本地文件，不会调用微信 API，也不会发布任何内容。

---

## 第三步：获取微信凭证（真实推送才需要）

需要两个东西：AppID 和 AppSecret。

1. 打开 [微信公众平台](https://mp.weixin.qq.com)，扫码登录
2. 左侧菜单 → **设置与开发** → **基本配置**
3. 复制 **AppID**（以 `wx` 开头）和 **AppSecret**（点击"重置"获取）

设置环境变量：
```bash
export WECHAT_APPID=wx***
export WECHAT_SECRET=your_secret
```

> 💡 可以写入 `~/.zshrc` 或 `~/.bashrc`，免得每次开终端都要重新设。

---

## 第四步：扫码登录（仅首次）

```bash
moonpub login
```

会打开 Chrome 浏览器，微信扫码登录。此后 `cookie` 持久化，不需要重复登录。

---

## 第五步：创建或检查配置文件

在你的文章目录下：

```bash
cd /path/to/your/articles
moonpub init
```

这会生成 `moonpub.toml`。编辑它：

```toml
[articles]
root = "/path/to/your/articles"

[wechat]
appid = "wxa..."
author = "你的公众号作者名"
theme = "default"        # default | warm | dark | geek

[blog]
# 没博客就删掉这几行，不影响微信发布
```

---

## 第六步：创建一篇文章

**从零开始**（手写）：
```bash
moonpub new "我为什么开始每天读书"
```
会在 `Articles/drafts/` 下生成 `我为什么开始每天读书.md`，带好模板。
如果标题里有空格，空格会转成 `-`；后续命令以 `moonpub new` 打印出的路径为准。

**AI 生成**（需要 DeepSeek API Key）：
```bash
export DEEPSEEK_API_KEY=sk-***
moonpub write "写一篇关于《活着》的读书笔记"
```

编辑文章内容，替换 frontmatter 里的 `digest:` 和正文。

打开文件看一下结构：
```markdown
---
title: 我为什么开始每天读书
digest: 一个普通人的阅读实验
date:
tags: []
---

:::intro
开场导语，抓人眼球
:::

正文……

:::summary
结尾
:::
```

---

## 第七步：预览

渲染成 HTML，浏览器里看效果：

```bash
moonpub render Articles/drafts/我为什么开始每天读书.md
moonpub preview Articles/drafts/我为什么开始每天读书.md
```

浏览器会打开渲染后的文章。确认排版没问题，继续下一步。

---

## 第八步：发布副驾驶

```bash
moonpub ship Articles/drafts/我为什么开始每天读书.md
```

`ship` 会把文章推进到“微信后台可人工确认发布”的状态：

| 步骤 | 做什么 |
|------|--------|
| 📸 封面 | 生成封面卡片 → Chrome 截图 → 上传微信素材库 |
| 🎨 渲染 | Markdown → 带排版的 WeChat HTML |
| 📤 推送 | 调微信 API 推送到草稿箱 |
| ⚙️ 配置 | headless Chrome 辅助设置：原创声明、赞赏、留言、创作来源、预览 |
| 📝 导出 | 如果配了博客，自动导出 Zola 格式 |

推送成功后，本地文章包会进入 `Articles/ready/`。最终发表仍需要你在微信后台检查后手动确认。加 `--headed` 可以看到浏览器操作过程：

```bash
moonpub configure --headed
```

---

## 第九步：发表

打开 [微信公众平台草稿箱](https://mp.weixin.qq.com)，刚才的文章已经在里面了。

原创声明 ✅、赞赏 ✅、留言 ✅、创作来源 ✅ 全部配置好了。检查一下内容，点 **发表**。

---

## 常见问题

### IP 不在白名单

如果 push 报错 `errcode=40164: invalid ip`，说明当前 IP 不在微信白名单里。

去 [微信公众平台 → 基本配置 → IP 白名单](https://mp.weixin.qq.com) 添加报错信息里的 IP。

### DeepSeek API 报错

`DEEPSEEK_API_KEY` 没设。去 [DeepSeek 开放平台](https://platform.deepseek.com) 注册获取。

### Chrome 找不到

确保系统装了 Chrome 或 Chromium。macOS 默认有，Linux 用 `apt install chromium-browser`，Windows 会自动搜 Program Files。

### 文章渲染后样式不对

- 不要用 `<style>` 标签（微信会剥离）
- 用 `moonpub render` 渲染，样式都写进 inline CSS
- `:::blockname` 拼写正确，区分大小写

### 浏览器自动化某一步失败

微信编辑器是 live web app，UI 偶尔会变。某一步软失败（⚠）不影响文章推送，只是该项未自动配置。加 `--headed` 看看具体哪步挂了：
```bash
moonpub configure --headed
```

---

## AI 功能一览

所有 AI 功能需要 `DEEPSEEK_API_KEY`：

```bash
# 从想法生成文章
moonpub write "写一篇关于《局外人》的书评"

# 读书笔记展开成文章
moonpub expand Articles/drafts/局外人.md

# 润色已有文章
moonpub polish Articles/drafts/局外人.md

# 润色 + 发布
moonpub ship Articles/drafts/局外人.md --ai
```

---

## 更多

- [完整命令列表](../README_zh.md)
- [GitHub](https://github.com/qiaopengjun5162/moonpub)
- [问题反馈](https://github.com/qiaopengjun5162/moonpub/issues)
