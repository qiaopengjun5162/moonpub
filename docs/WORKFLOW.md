# MoonPub 发布工作流

从写文章到微信公众号发布的完整流程。

如果你现在还在判断“我应该走哪条路径”，先看 [RECOMMENDED_WORKFLOWS_ZH.md](RECOMMENDED_WORKFLOWS_ZH.md)。这份 `WORKFLOW.md` 更适合已经知道自己要走普通文章发布路径、并准备查看完整细节的人。

## 前置条件

`moonpub.toml`（放在文章根目录或通过 `--config` 指定）：

```toml
[articles]
root = "/path/to/ObsidianMain"

[wechat]
appid = "wxxxxxxxxxxx"
author = "你的名字"
thumb_media_id = "<封面图 media_id>"
theme = "default"   # default | warm | dark | geek | paper | magazine | notebook | classic | forest | sunset | ocean | mono | editorial | zen | newsletter | academic | cyber | letter | mist | gallery

[blog]
kind = "zola"
root = "/path/to/blog"
```

环境变量（不写入 toml）：

```bash
export WECHAT_APPID=wxxxxxxxxxxx
export WECHAT_SECRET=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

**注意：每次 push 前确认本机 IP 在微信后台 IP 白名单内。**

---

## 命令影响范围

| 阶段 | 命令 | 影响范围 | 凭证要求 |
|------|------|----------|----------|
| 本地创建 | `moonpub init` / `moonpub new` | 只写本地配置和 Markdown 文件 | 无 |
| 本地渲染 | `moonpub render` / `moonpub preview` / `moonpub cover` | 生成 HTML、draft JSON、封面 HTML；预览只打开本机浏览器 | 无 |
| 微信 API | `moonpub push` / `moonpub update-draft` | 上传图片、创建或更新微信草稿 | `WECHAT_APPID` / `WECHAT_SECRET`，IP 白名单 |
| 微信后台 | `moonpub login` / `moonpub configure` | 打开或控制 Chrome，操作微信后台草稿设置 | 微信扫码登录 |
| 全流程 | `moonpub ship` | cover → render → push → configure → export | 微信凭证 + Chrome；博客导出按配置可选 |
| 本地导出 | `moonpub export` | 写入本地 Zola 博客目录 | 无 |

当前建议：第一次给别人演示时，先跑 `init → new → render → preview → cover`；确认本地输出没问题，再进入 `login → push --render → configure`。

---

## 完整流程

### 1. 写文章

在 `Articles/drafts/` 下新建 Markdown 文件，frontmatter 格式：

```markdown
---
title: 文章标题
digest: 一句话摘要（微信草稿摘要字段）
date: 2026-06-11
tags: ["标签1", "标签2"]
---

正文内容…
```

本地图片直接用相对路径引用（`![](./img/foo.png)`），push 时会自动上传到微信素材库。

### 2. 去 AI 味（可选）

```bash
moonpub humanize Articles/drafts/文章名.md
```

in-place 修改，覆盖原文件。处理 6 类 AI 语言特征。

### 3. 渲染

```bash
moonpub render Articles/drafts/文章名.md
```

生成同目录下：
- `文章名.html` — 微信兼容 HTML（inline CSS，按 `[wechat].theme` 着色）
- `文章名.draft.json` — WeChat `draft/add` 接口格式

加 `--humanize` 可在渲染前自动去 AI 味：

```bash
moonpub render Articles/drafts/文章名.md --humanize
```

### 4. 本地预览

```bash
moonpub preview Articles/drafts/文章名.md
```

用系统浏览器打开 HTML，检查排版效果。

### 5. 生成封面（可选）

```bash
# 仅生成 HTML
moonpub cover Articles/drafts/文章名.md --style clean

# 同时生成 HTML + PNG 截图（需要 Chrome）
moonpub cover Articles/drafts/文章名.md --style gradient --screenshot
```

风格可选：`dark` / `clean` / `minimal` / `warm` / `serif` / `gradient` / `literary` / `ink` / `sunset` / `forest`

`--screenshot` 会在本地用 Chrome headless 将 HTML 截图为 `文章名.cover.png`（900×500px），可上传到微信素材库作封面图。

### 6. Push 到微信草稿

```bash
moonpub push Articles/drafts/文章名.md
```

push 做了什么：
1. 扫描 HTML 里的本地 `src="..."` 图片，逐个上传微信永久素材库，替换为 CDN URL
2. 重建 draft.json（含更新后的 HTML）
3. 调用 `draft/add` 创建草稿，写入 `.media_id` 文件
4. 将文章包移动到 `Articles/ready/`，等待人工检查和发表

输出示例：
```
pushed to WeChat draft
  media_id: xxxxx
  moved to .../Articles/ready
  images: 2 uploaded to WeChat CDN
  next: check in WeChat backend, then publish manually
```

如果已有 draft.json 可跳过 render 直接 push：

```bash
moonpub push Articles/drafts/文章名.md --render   # 先 render 再 push
```

### 7. 浏览器自动配置（可选）

```bash
moonpub configure
```

自动化会尝试配置原创声明、赞赏、留言、创作来源和预览。微信后台是 live web app，DOM 或文案变化时，某一步可能软失败；这不影响已经通过 API 创建的草稿。

### 8. 微信后台操作

1. 打开公众号后台 → 草稿箱
2. 检查排版、封面图、摘要
3. 手动选择合集（API 暂不支持）
4. 点击「发表」

### 9. 标记已发布

```bash
moonpub mark-published Articles/ready/文章名.md
```

写入 `.moonpub/status.jsonl` 状态记录，并把文章包移动到 `Articles/published/`。

---

## 状态查看

```bash
moonpub status          # 列出 drafts/ready/published 目录下所有文章
moonpub check 文章名.md  # 检查 md/html/draft.json 是否齐全
```

## 更新已有草稿

改完文章后重新 render，再：

```bash
moonpub update-draft Articles/ready/文章名.md
```

注意：update-draft 后微信后台部分设置（封面、摘要）会重置，需手动重新设置。

## Radar（热点参考）

```bash
# 手动录入热点标题
moonpub radar add --platform wechat --keyword 读书 --title "我读了100本书的感悟" --likes 5000

# 分析文章与热点的关键词重合度
moonpub radar analyze Articles/drafts/文章名.md --platform wechat

# 生成标题建议（4 种公式）
moonpub radar suggest Articles/drafts/文章名.md --platform wechat
```

## 导出到博客

```bash
moonpub export Articles/published/文章名.md
```

将 YAML frontmatter 转为 TOML，去掉微信 footer，输出到 Zola 博客目录。
