# MoonPub 用户使用说明书

> **一句话：一部手机看书做笔记，一条命令发公众号。**

---

## MoonPub 是什么

MoonPub 是一个命令行工具。你把 Markdown 文章写好（或者用 AI 生成），运行一条命令，文章就推送到微信公众号草稿箱——封面、排版、原创声明、赞赏、留言、创作来源全部自动配好。你只需要去后台点"发表"。

**不需要任何第三方服务，纯本地运行。**

---

## 安装

**macOS / Linux**（推荐）：
```bash
curl -L https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.0/moonpub-macos-amd64.tar.gz | tar xz
sudo mv moonpub /usr/local/bin/
```

**Homebrew**：
```bash
brew tap qiaopengjun5162/moonpub && brew install moonpub
```

**Windows**：从 [Releases](https://github.com/qiaopengjun5162/moonpub/releases) 下载 zip，解压 `moonpub.exe`，加入 PATH。

---

## 配置

在你的文章目录下运行：

```bash
moonpub init   # 生成 moonpub.toml
```

编辑 `moonpub.toml`：

```toml
[articles]
root = "/你的文章目录路径"

[wechat]
appid = "wx..."
author = "你的公众号作者名"
theme = "geek"        # default | warm | dark | geek

[blog]
# 如果没有博客，删掉这几行即可
kind = "zola"
root = "/你的博客路径"
```

设置微信凭证（二选一）：

```bash
# 方式1：环境变量
export WECHAT_APPID=wx***
export WECHAT_SECRET=你的secret

# 方式2：.env 文件（在文章目录下）
echo 'WECHAT_APPID=wx***' > .env
echo 'WECHAT_SECRET=你的secret' >> .env
```

首次使用需要扫码登录一次：

```bash
moonpub login
```

---

## 核心流程

### 流程一：已有完整文章 → 直接发布

```
写 Markdown → ship → 微信草稿箱
```

```bash
moonpub new "我的文章标题"                    # 创建文章模板
# 编辑 Articles/drafts/我的文章标题.md
moonpub render Articles/drafts/我的文章标题.md  # 预览
moonpub preview Articles/drafts/我的文章标题.md # 浏览器看效果
moonpub ship Articles/drafts/我的文章标题.md    # 发布
```

### 流程二：微信读书笔记 → AI 展开 → 发布

```
微信读书划线 → 导入 Obsidian → expand → ship → 微信草稿箱
```

```bash
# 1. 把笔记复制到 Articles/drafts/
# 2. AI 展开（需要 DEEPSEEK_API_KEY）
moonpub expand Articles/drafts/且听风吟.md
# 3. 预览
moonpub render Articles/drafts/且听风吟.md
moonpub preview Articles/drafts/且听风吟.md
# 4. 发布
moonpub ship Articles/drafts/且听风吟.md
```

### 流程三：一个想法 → AI 写作 → 发布

```bash
moonpub write "写一篇关于《活着》的读书笔记"
# AI 生成文章到 Articles/drafts/
moonpub ship Articles/drafts/写一篇关于活着-的读书笔记.md
```

---

## 命令速查

### 日常使用

| 命令 | 说明 |
|------|------|
| `moonpub new "标题"` | 创建文章模板 |
| `moonpub write "想法"` | AI 从想法生成文章 |
| `moonpub expand notes.md` | AI 展开读书笔记 |
| `moonpub polish draft.md` | AI 润色文章 |
| `moonpub ship article.md` | 一键发布全流程 |
| `moonpub ship article.md --ai` | 润色 + 发布 |
| `moonpub render article.md` | 渲染 HTML |
| `moonpub preview article.md` | 浏览器预览 |
| `moonpub cover article.md --style ink` | 生成封面 |

### 一次性配置

| 命令 | 说明 |
|------|------|
| `moonpub init` | 创建 moonpub.toml |
| `moonpub login` | 微信扫码登录 |

### AI 功能

| 命令 | 说明 |
|------|------|
| `moonpub write "想法"` | 从零生成（写文章） |
| `moonpub expand notes.md` | 笔记展开（重组内容） |
| `moonpub polish draft.md` | 润色优化（改进表达） |

### `ship` 做了什么

```
cover → render → push API → configure 浏览器 → export 博客
  ↓        ↓         ↓            ↓              ↓
封面截图  排版渲染  推送草稿  自动设置配置项   导出Zola
```

**configure 具体配置项**：

| 步骤 | 状态 |
|------|:--:|
| 原创声明 | ✅ |
| 赞赏 | ✅ |
| 留言 | ✅ |
| 创作来源 | ✅ 个人观点，仅供参考 |
| 预览 | ✅ 发送到手机 |
| 合集 | ⏸ 跳过（手动选） |

---

## AI 命令区别

`expand` 和 `polish` 和 `ship --ai` 不是一个东西：

| 命令 | 输入 | 输出 | 用途 |
|------|------|------|------|
| `expand` | 读书笔记碎片 | 完整文章 | 结构性重组 |
| `polish` | 完整文章 | 润色后文章 | 改进措辞 |
| `ship --ai` | 完整文章 | 润色 + 发布 | 一键搞定 |

**使用建议**：
- 微信读书笔记 → `expand`
- 自己写的草稿 → `polish` 或 `ship --ai`

---

## 封面风格

10 种可选：

```bash
moonpub ship article.md --style clean      # 白底简洁
moonpub ship article.md --style warm       # 暖色
moonpub ship article.md --style ink        # 水墨留白
moonpub ship article.md --style sunset     # 日落暖橙
moonpub ship article.md --style forest     # 森林绿
moonpub ship article.md --style literary   # 深色文学风（默认）
moonpub ship article.md --style dark       # 深蓝黑
moonpub ship article.md --style minimal    # 极简
moonpub ship article.md --style serif      # 衬线典雅
moonpub ship article.md --style gradient   # 紫粉渐变
```

**书名笔记有书封图**：微读导入的笔记 frontmatter 里自带 `cover: https://...`，moonpub 会自动下载上传微信作为封面。

**自己指定封面**：在 frontmatter 里加一行：
```yaml
cover: /path/to/your-image.png
```

---

## 文章格式

最小可用文章：

```markdown
---
title: 文章标题
digest: 120字以内的摘要（可选，不填则微信自动抓取）
tags: [标签1, 标签2]
---

正文内容，标准 Markdown 语法。

## 二级标题

- 列表
- **加粗** *斜体*

> 引用文字

:::intro
开场导语，抓读者注意力
:::

:::summary
结尾总结
:::
```

**Block 模板**（可选）：`:::intro` / `:::callout` / `:::steps` / `:::summary` / `:::book-info` / `:::figure` / `:::checklist` / `:::cover` / `:::quote-card` / `:::divider` / `:::concept-card` / `:::emotion-card`

---

## 常见问题

### IP 不在白名单
```
errcode=40164: invalid ip
```
→ 去 [微信公众平台 → 基本配置 → IP 白名单](https://mp.weixin.qq.com) 添加 IP。

### DeepSeek 报错
→ 去 [platform.deepseek.com](https://platform.deepseek.com) 注册获取 key，写入 `.env`：
```
DEEPSEEK_API_KEY=sk-***
```

### Chrome 找不到
→ macOS 自带 Chrome。Linux：`apt install chromium-browser`。Windows：自动搜 Program Files。

### 某一步软失败（⚠）
→ 不影响发布。微信编辑器偶尔 UI 变化，加 `--headed` 看具体问题：
```bash
moonpub configure --headed
```

---

## Obsidian 插件

在 Obsidian 里按 `Cmd+P`，输入"发布"，回车即发。

安装：把 `obsidian-plugin/` 目录复制到 vault 的 `.obsidian/plugins/moonpub/`，`npm install && npm run build`，启用。

---

## 更多

- [GitHub](https://github.com/qiaopengjun5162/moonpub)
- [上手教程](GETTING_STARTED.md)
- [首页](https://paxonqiao.com/moonpub/)
- [问题反馈](https://github.com/qiaopengjun5162/moonpub/issues)

---

> MoonPub — Markdown → 微信公众号，全自动发布。
