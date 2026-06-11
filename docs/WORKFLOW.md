# MoonPub 发布工作流

从写文章到微信公众号发布的完整流程。

## 前置条件

`moonpub.toml`（放在 Obsidian vault 根目录）：

```toml
vault_root = "/path/to/ObsidianMain"
wechat_appid = "wxxxxxxxxxxx"
wechat_author = "你的名字"
wechat_thumb_media_id = "<封面图 media_id>"
wechat_theme = "default"   # default | warm | dark
```

环境变量（不写入 toml）：

```bash
export WECHAT_APPID=wxxxxxxxxxxx
export WECHAT_SECRET=xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx
```

**注意：每次 push 前确认本机 IP 在微信后台 IP 白名单内。**

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
- `文章名.html` — 微信兼容 HTML（inline CSS，按 wechat_theme 着色）
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
moonpub cover Articles/drafts/文章名.md --style clean
```

生成 `文章名.cover.html`（900×500px），用浏览器截图上传到微信素材库作封面图。

### 6. Push 到微信草稿

```bash
moonpub push Articles/drafts/文章名.md
```

push 做了什么：
1. 扫描 HTML 里的本地 `src="..."` 图片，逐个上传微信永久素材库，替换为 CDN URL
2. 重建 draft.json（含更新后的 HTML）
3. 调用 `draft/add` 创建草稿，写入 `.media_id` 文件
4. 将文章包移动到 `Articles/published/`

输出示例：
```
pushed
  media_id: xxxxx
  moved to .../Articles/published
  images: 2 uploaded to WeChat CDN
```

如果已有 draft.json 可跳过 render 直接 push：

```bash
moonpub push Articles/drafts/文章名.md --render   # 先 render 再 push
```

### 7. 微信后台操作

1. 打开公众号后台 → 草稿箱
2. 检查排版、封面图、摘要
3. 手动选择合集（API 暂不支持）
4. 点击「发表」

### 8. 标记已发布

```bash
moonpub mark-published Articles/published/文章名.md
```

写入 `.moonpub/status.jsonl` 状态记录。

---

## 状态查看

```bash
moonpub status          # 列出 drafts/ready/published 目录下所有文章
moonpub check 文章名.md  # 检查 md/html/draft.json 是否齐全
```

## 更新已有草稿

改完文章后重新 render，再：

```bash
moonpub update-draft Articles/published/文章名.md
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
