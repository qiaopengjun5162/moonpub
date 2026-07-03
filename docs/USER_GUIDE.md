# MoonPub 用户使用说明书

> **MoonPub——写完文章跑一下，先本地渲染预览；配置微信凭证后，可以把文章推到公众号草稿箱，并尝试自动配置原创、赞赏、留言和创作来源。发表前仍建议打开微信后台检查草稿。**

---

## MoonPub 是什么

MoonPub 是一个小工具，帮你把 Markdown 文章变成微信公众号草稿。它可以本地完成排版、封面和预览；配置微信凭证后，可以调用微信 API 推送草稿，并通过 Chrome 自动配置部分后台选项。

当前状态：**Beta / 技术用户可试用**。本地渲染不需要任何凭证；真实推送会触达微信 API；AI 写作相关命令需要你自己的 AI provider key，默认 DeepSeek，也支持 OpenAI。

如果你现在最想先解决“我到底该走哪条路径”，先看 [RECOMMENDED_WORKFLOWS_ZH.md](RECOMMENDED_WORKFLOWS_ZH.md)。那份文档把当前正式主推的三条用户路径单独拆出来了：

- 已有 Markdown 文章 → 本地预览 → 微信草稿
- 飞书秒记 → 草稿 → 预览 → 微信草稿
- 照片素材 → 草稿 → 预览 → 微信草稿

如果你现在更想先解决“MoonPub 到底应该被理解成一个什么产品，而不是一堆命令”，再看 [PRODUCT_WRAP_ZH.md](PRODUCT_WRAP_ZH.md)。那份文档把本地发布内核、输入工作流层和用户入口层拆开了。

如果你现在其实更在意“第一次到底怎么走最不容易迷路”，直接看 [FIRST_RUN_WALKTHROUGH_ZH.md](FIRST_RUN_WALKTHROUGH_ZH.md)。那份文档把插件首页、飞书、照片和当前文章三种首次入口收成了一条最短体验路径。

如果你现在更在意“这些首次入口到底哪条已经真实打通、哪条证据还不够强”，再看 [FIRST_RUN_AUDIT_ZH.md](FIRST_RUN_AUDIT_ZH.md)。

如果你下一步准备补真实截图、录屏或样例验证，再看 [FIRST_RUN_EVIDENCE_CHECKLIST_ZH.md](FIRST_RUN_EVIDENCE_CHECKLIST_ZH.md)。仓库里也已经补了统一归档位和记录模板：`docs/first-run-evidence/README.md`、`docs/first-run-evidence/NOTES.md`，以及 3 个固定归档目录：`docs/first-run-evidence/homepage/`、`docs/first-run-evidence/feishu/`、`docs/first-run-evidence/photos/`。

---

## 命令会做什么

| 类型 | 命令 | 是否触达外部服务 | 说明 |
|------|------|------------------|------|
| 本地体验 | `init` / `new` / `render` / `preview` / `cover` | 否 | 只读写本地文件；`preview` 打开本机浏览器；`cover --screenshot` 需要本机 Chrome |
| 微信推送 | `push` / `update-draft` / `list-drafts` / `delete-draft` | 是，微信 API | 需要 `WECHAT_APPID` / `WECHAT_SECRET`，本机 IP 需在微信白名单 |
| 浏览器自动化 | `login` / `configure` / `ship` | 是，微信后台 + Chrome | 依赖微信后台页面；UI 变化时可能软失败 |
| 博客导出 | `export` | 否 | 写入本地 Zola 博客目录 |
| AI 写作 | `write` / `expand` / `polish` / `ship --ai` | 是，按配置调用 DeepSeek / OpenAI | 默认需要 `DEEPSEEK_API_KEY`；切到 OpenAI 时需要 `OPENAI_API_KEY` |

建议第一次使用先跑本地体验路径：

```bash
moonpub init
moonpub new "我的第一篇文章"
moonpub render Articles/drafts/我的第一篇文章.md
moonpub preview Articles/drafts/我的第一篇文章.md
moonpub cover Articles/drafts/我的第一篇文章.md --style literary
```

---

## 安装

当前公开可下载的 GitHub Release 是 `v0.4.1`；如果你看到仓库源码版本更高，那表示新改动还没正式打包发布。

**macOS / Linux**（推荐）：
```bash
curl -L https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.1/moonpub-macos-arm64.tar.gz | tar xz
sudo mv moonpub /usr/local/bin/
```

macOS Intel 用户把文件名换成 `moonpub-macos-amd64.tar.gz`；Linux 用户换成对应的 `moonpub-linux-amd64.tar.gz` 或 `moonpub-linux-arm64.tar.gz`。

**Homebrew**：
Homebrew 支持还在准备中，当前推荐直接下载 release 或使用 Cargo。

**Windows**：从 [Releases](https://github.com/qiaopengjun5162/moonpub/releases) 下载 zip，解压 `moonpub.exe`，加入 PATH。PR CI 已验证源码构建的 Windows 二进制无凭证路径；已发布 zip 仍建议先按 [WINDOWS_SMOKE_CHECKLIST_ZH.md](WINDOWS_SMOKE_CHECKLIST_ZH.md) 自检。

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
theme = "geek"        # default | warm | dark | geek | paper | magazine | notebook | classic | forest | sunset | ocean | mono | editorial | zen | newsletter | academic | cyber | letter | mist | gallery

[blog]
# 如果没有博客，删掉这几行即可
kind = "zola"
root = "/你的博客路径"

[template]
name = "寻月阁标准结尾"   # 可选；供 configure / ship 自动插入模板

[ai]
provider = "deepseek"      # deepseek | openai
model = "deepseek-chat"    # 可选，默认按 provider 推荐模型
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

如果你不想复用 MoonPub 默认保存的浏览器登录态，而是想用一次性的隔离环境，可以显式加上：

```bash
moonpub login --temporary-profile
```

这个模式会启用临时 Chrome profile，不会读取或写回持久 session，所以通常需要重新扫码。

---

## 核心流程

### 流程一：已有完整文章 → 推送草稿

```
写 Markdown → ship → 微信草稿箱
```

```bash
moonpub new "我的文章标题"                    # 创建文章模板
# 编辑 Articles/drafts/我的文章标题.md
moonpub render Articles/drafts/我的文章标题.md  # 预览
moonpub preview Articles/drafts/我的文章标题.md # 浏览器看效果
moonpub ship Articles/drafts/我的文章标题.md    # 推进到可人工确认发布
```

`ship` 会调用微信 API 并控制 Chrome，把文章推到草稿箱并移动到 `Articles/ready/`。第一次真实发布前，建议先用 `push --render` 推到草稿箱，再手动检查草稿。

### 流程二：微信读书笔记 → AI 展开 → 发布

```
微信读书划线 → 导入 Obsidian → expand → ship → 微信草稿箱
```

```bash
# 1. 把笔记复制到 Articles/drafts/
# 2. AI 展开（默认需要 DEEPSEEK_API_KEY；也支持改用 OpenAI）
moonpub expand Articles/drafts/且听风吟.md
# 3. 预览
moonpub render Articles/drafts/且听风吟.md
moonpub preview Articles/drafts/且听风吟.md
# 4. 推进到可人工确认发布
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
| `moonpub draft-from-inbox Inbox/Feishu/demo.md --preview --no-open` | 从 Inbox 素材生成可编辑草稿，只生成本地预览 HTML，并提示下一步 push 命令 |
| `moonpub draft-from-inbox Inbox/Feishu/demo.md --push` | 从 Inbox 素材生成可编辑草稿，并直接继续执行 `push --render` 推到微信草稿 |
| `moonpub expand notes.md` | AI 展开读书笔记 |
| `moonpub polish draft.md` | AI 润色文章 |
| `moonpub intake feishu minutes.txt --draft --preview --no-open` | 导入飞书秒记，继续生成草稿，并只生成本地预览 HTML |
| `moonpub intake feishu --latest --draft --push` | 导入最近一条飞书妙记，生成草稿后直接继续执行 `push --render` |
| `moonpub intake feishu --minute-token <token>` | 从飞书妙记拉取逐字稿到 `Inbox/Feishu/` |
| `moonpub intake feishu --latest --draft --preview` | 导入我拥有的最近一条飞书妙记，并继续生成草稿和本地预览 |
| `moonpub intake feishu --query <关键词> --draft --preview --no-open` | 搜索飞书妙记、导入第一条结果，并只生成本地预览 HTML |
| `moonpub intake photos photos/day1 photos/day2 --draft --preview --no-open` | 导入一组真实照片到 `Inbox/Photos/`，继续生成草稿，并只生成本地预览 HTML |
| `moonpub ship article.md` | 发布副驾驶全流程 |
| `moonpub ship article.md --ai` | 润色 + 发布副驾驶 |
| `moonpub render article.md` | 渲染 HTML |
| `moonpub preview article.md --no-open` | 只确认本地 HTML 预览文件路径，不打开浏览器 |
| `moonpub cover article.md --style ink` | 生成封面 |

飞书链路默认推荐保守模式：先到“可编辑草稿 + 本地预览”，也就是 `intake feishu ... --draft --preview`。只有你显式加 `--push` 时，才会继续执行等价于 `push --render` 的快速路径，把内容推进到微信草稿。

2026-07-01 已做过一次真实链路验证，不只是单元测试：

- `moonpub --articles "<Obsidian 路径>" --json intake feishu --latest --draft --preview --no-open`
  真实成功返回了 `inbox_path`、`draft_path`、`html_path`
- `moonpub --articles "<Obsidian 路径>" --json intake feishu --latest --draft --push`
  真实成功推进到微信草稿，并在后台自动完成原创/赞赏/留言/创作来源/预览发送，最终返回 `pushed: true`、`stage: "ready"` 和真实 `media_id`

这里也顺手记住两个 CLI 细节：

- 当前入口参数是 `--articles <path>`，不是 `--vault`
- `--json` 是全局参数，必须放在子命令前面

这 7 条工作流命令在全局 `--json` 下会返回结构化字段，方便脚本、插件和后续 Agent 直接接力，而不是再从纯文本里反解析：

- `workspace`：`command`、`workspace_kind`、`entry_path`、`entry_path_label`、`total_articles`、`stage_counts`、`stages`、`capabilities`、`next_command`、`next_step`
- `status`：`command`、`stages`、`next_command`、`next_step`；每个 stage 下会带 `stage`、`count`、`files`，而每个文件项会带 `file`、`slug`、`latest_status`、`latest_detail`
- `check`：`command`、`article_path`、`html_path`、`draft_json_path`、`media_id_path`、`has_markdown`、`has_html`、`has_draft_json`、`has_media_id`、`publishable`、`next_command`、`next_step`
- `preview`：`command`、`article_path`、`html_path`、`opened_browser`、`next_command`
- `push`：`command`、`article_path`、`media_id`、`stage`、`next_step`
- `draft-from-inbox`：`command`、`input_path`、`draft_path`、可选 `html_path`、`action`、`next_command`；加 `--push` 时还会带 `pushed`、`media_id`、`stage`、`next_step`
- `intake feishu ... --draft`：`command`、`inbox_path`、`draft_path`、可选 `html_path`、`action`、`next_command`；加 `--push` 时还会带 `pushed`、`media_id`、`stage`、`next_step`
- `intake photos ... --draft`：`command: "intake-photos"`、`inbox_path`、`draft_path`、可选 `html_path`、`action`、`next_command`；加 `--push` 时也会带 `pushed`、`media_id`、`stage`、`next_step`

如果你现在做的不是直接用 CLI，而是要做插件、App 或 Agent 入口，建议继续看 [AGENT_PROTOCOL_ZH.md](AGENT_PROTOCOL_ZH.md)。那份文档把 `workspace` / `status` / `check` / 动作命令 / `capabilities` 这几层的职责拆开了。

如果你现在做的是产品收口、入口排序或路线判断，也建议一起看 [PRODUCT_WRAP_ZH.md](PRODUCT_WRAP_ZH.md)。那份文档会先把 MoonPub 现在应该怎样被理解讲清楚，再决定入口层要怎么继续补。

除此之外，其它命令的 `--json` 仍是兼容模式的 `{"output":"..."}`。

这里有两种“预览”，不要混淆：

- `moonpub preview`、`moonpub intake feishu ... --draft --preview` 或 `moonpub draft-from-inbox ... --preview` 指的是本地 HTML 预览，用来先检查排版和内容。
- 微信公众号后台的“预览发送到手机”属于 `configure` / `ship` 阶段，是文章已经推入微信草稿后的后台操作。

`--push` 只在已经生成草稿的链路上生效：`draft-from-inbox --push` 或 `intake feishu ... --draft --push`。它会等价执行后续的 `moonpub push <draft.md> --render`，因此和本地 `--preview` 是互斥的；如果你想先人工看一眼、去 AI 味、补图或多改几轮，继续走默认的本地 `--preview` 路径更合适。推入微信草稿后，再和其它文章一样走 `configure` 里的微信公众号后台预览。

如果你的目标是和项目里其它文章完全一致的微信公众号发布节奏，推荐流程是：

1. `intake feishu ... --draft --preview`
2. 人工修改 / `polish` / `humanize`（可选）
3. 确认本地预览没问题后执行 `push --render`
4. 再像其它文章一样执行 `configure` / `ship`
5. 微信公众号后台预览发送到手机
6. 发布

如果你走的是飞书官方秒记链路，也就是 `--minute-token`、`--latest`、`--query` 这几种方式，那么重复导入同一条秒记时会按统一 Inbox 元数据里的 `external_id` 复用并更新原 `Inbox/Feishu/*.md`；飞书当前也会继续保留 `minute_token` 这个来源专属字段，兼容旧文件和来源追踪。后续重复生成草稿时也会复用同一份草稿文件，不再因为“已存在”直接中断。

照片链路现在也有第一版正式入口：`intake photos <文件或目录...>`。它会先把一组真实照片文件归档到 `Inbox/Photos/`，根据文件路径、文件大小和修改时间生成一份尽量实事求是的素材稿，再继续复用 `draft-from-inbox`、`--preview`、`--push` 这一整条后续工作流。当前这一步还没有做 EXIF 深解析或图片内容理解，但已经足够把“手机里的生活照片先稳定留进系统”这件事跑起来。

### 一次性配置

| 命令 | 说明 |
|------|------|
| `moonpub init` | 创建 moonpub.toml |
| `moonpub login` | 微信扫码登录 |
| `moonpub login --temporary-profile` | 用一次性隔离 profile 登录，不复用已保存 session |
| `moonpub configure moban --headed` | 单独调试微信模板插入 |
| `moonpub configure --temporary-profile --headed` | 用隔离 profile 调试后台自动化 |
| `moonpub step-test --temporary-profile --headed` | 用隔离 profile 跑完整交互测试链路 |

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
| `ship --ai` | 完整文章 | 润色 + 发布副驾驶 | 一键推进到可确认发布 |

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

**Block 模板**（可选）：`:::intro` / `:::callout` / `:::steps` / `:::summary` / `:::book-info` / `:::figure` / `:::checklist` / `:::key-points` / `:::pull-quote` / `:::cover` / `:::letter-card` / `:::scene-card` / `:::closing-card` / `:::photo-grid` / `:::meta-strip` / `:::quote-card` / `:::divider` / `:::concept-card` / `:::emotion-card`

**正文主题**：在 `moonpub.toml` 的 `[wechat].theme` 或文章 frontmatter 里设置 `theme`：

```yaml
theme: paper
```

当前可选：`default` / `warm` / `dark` / `geek` / `paper` / `magazine` / `notebook` / `classic` / `forest` / `sunset` / `ocean` / `mono` / `editorial` / `zen` / `newsletter` / `academic` / `cyber` / `letter` / `mist` / `gallery`。

生活合集、照片记录和私人随笔可以优先试 `mist` / `gallery` / `letter`：`mist` 更安静，`gallery` 更适合图文，`letter` 更像一封写给读者的短笺。照片多的时候，可以用 `:::meta-strip` 先交代日期地点，再用 `:::photo-grid` 放两列照片组。

普通 Markdown 的标题、首段导语、段落、行内高亮 / 删除线、引用、分割线、带 caption 的图片、表格、无序 / 有序 / 任务列表和三反引号代码块会自动渲染成微信兼容的 inline CSS 排版；需要更强视觉块时再使用上面的 `:::` Block 模板。

---

## 常见问题

### IP 不在白名单
```
errcode=40164: invalid ip
```
→ 去 [微信公众平台 → 基本配置 → IP 白名单](https://mp.weixin.qq.com) 添加 IP。

### AI provider 报错
→ 默认走 DeepSeek。去 [platform.deepseek.com](https://platform.deepseek.com) 注册获取 key，写入 `.env`：
```
DEEPSEEK_API_KEY=sk-***
```

→ 如果 `[ai].provider = "openai"`，则改为：
```
OPENAI_API_KEY=sk-***
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

当前仓库里已经有实验性的 Obsidian 插件入口，但它更适合被理解为：

**在 Obsidian 里调用本地 MoonPub CLI 的快捷入口。**

它目前支持 9 个命令：

- `打开 MoonPub 首页`
- `查看整体文章池状态`
- `检查当前文章状态`
- `导入最近一条飞书妙记并生成草稿预览`
- `导入最近一条飞书妙记并推进到微信草稿`
- `导入当前图片所在目录并生成照片草稿预览`
- `预览文章`
- `发布到微信公众号`
- `AI 润色后发布到公众号`

安装方式：

1. 先确保本机已经安装好 `moonpub`
2. 把 `obsidian-plugin/` 目录复制到 vault 的 `.obsidian/plugins/moonpub/`
3. 在插件目录里执行 `npm install && npm run build`
4. 回到 Obsidian 启用 `MoonPub`
5. 如有需要，在插件设置中补 `MoonPub 可执行文件路径` 和 `Articles 根目录`

其中 `打开 MoonPub 首页` 和 `查看整体文章池状态` 都依赖 `Articles 根目录`，因为它们现在都会调用 `moonpub workspace --json`，先判断整个工作区该走哪条入口、文章池里当前有哪些阶段、下一步推荐先做什么，而不只是查询当前打开文件。

插件现在还会把这份 `workspace --json` 结果继续展开成一个简短的工作台弹窗，而不是只留一条压缩 Notice。这样用户在 Obsidian 里能更直观看到：

- 当前推荐入口
- drafts / ready / published 的阶段数量
- 推荐下一步命令
- 哪些能力会联网或打开 Chrome

这个工作区工作台现在也开始更像插件首页，而不只是状态面板。你可以直接从里面继续点：

- 检查当前文章
- 预览当前文章
- 导入最近飞书妙记
- 导入当前图片目录

插件首页现在还会根据你当前打开的内容给出更贴近上下文的推荐：

- 打开的是 Markdown：更推荐先检查当前文章
- 打开的是图片：更推荐先导入当前图片目录
- 没有打开文件或打开的是别的文件：会提示你先回到更合适的入口

它现在还会把“第一次建议步骤”直接列出来，把“当前更适合走哪条路径”再往前推进一层，变成“现在建议你先做什么、再做什么、最后再做什么”。

如果你不是从现有 Markdown 开始，而是从飞书妙记起步，插件现在也开始提供两条素材入口：

- `导入最近一条飞书妙记并生成草稿预览`
- `导入最近一条飞书妙记并推进到微信草稿`

它们分别直连：

- `moonpub --articles <path> intake feishu --latest --draft --preview --json`
- `moonpub --articles <path> intake feishu --latest --draft --push --json`

前者更适合作为默认保守路径，后者更适合作为显式快速路径。这样你在 Obsidian 里不必先切回终端，也能直接起飞书主推工作流。

如果生成出来的草稿本来就落在当前 vault 里，插件还会尝试自动打开那篇草稿，尽量把“导入完还得自己去找文件”的一步省掉。

飞书入口执行完成后，插件现在还会继续打开一个“飞书结果工作台”弹窗，专门展示：

- Inbox 路径
- Draft 路径
- 本地 HTML 预览路径
- 是否已经推进到微信草稿
- 推荐下一步动作

而且这个结果工作台现在已经不是纯展示页了。你可以直接从里面继续：

- 打开草稿
- 检查草稿
- 预览草稿
- 继续推进到微信草稿

这样这条链路在插件里不再只是“跑完一条命令”，而是开始变成一个更像工作流结果页的入口。

照片链路现在也开始有第一条正式插件入口：`导入当前图片所在目录并生成照片草稿预览`。

它适合你当前已经在 Obsidian 里打开一张图片，希望直接把“这张图片所在目录”当成一批生活素材整理成草稿的场景。插件会继续调用：

- `moonpub --articles <path> intake photos <当前图片所在目录> --draft --preview --json`

然后和飞书一样自动打开草稿、展示结果工作台，并把后续的检查/预览动作继续串起来。

`检查当前文章状态` 现在也会把 `check --json` 结果展开成当前文章工作台，而不是只显示一行 `publishable / html / draft_json / media_id / next`。这样你在 Obsidian 里能直接看到：

- 当前是否已经可继续发布
- Markdown / HTML / `draft.json` / `media_id` 是否齐全
- 相关产物路径
- 当前最推荐的下一步动作

插件现在还会在执行“发布到微信公众号”前，先用 `moonpub capabilities --json` 做一层轻量提示：

- 这次操作会不会联网
- 会不会打开或控制 Chrome
- 通常依赖哪些环境变量或配置项

这个提示不会只因为 Obsidian 进程里没看到 `WECHAT_APPID` / `WECHAT_SECRET` 就直接拦住你。因为 MoonPub CLI 自己还会继续读取项目 `.env` 和 `~/.moonpub.env`，最终是否能发布成功，以 CLI 的真实运行结果为准。

推荐第一次先用：

- `打开 MoonPub 首页`
- 如果你从飞书素材开始：`导入最近一条飞书妙记并生成草稿预览`
- 如果你从照片素材开始：先打开一张图片，再执行 `导入当前图片所在目录并生成照片草稿预览`
- `检查当前文章状态`
- `预览文章`

确认本地排版没问题后，再执行：

- `发布到微信公众号`

现在第一步里的 `打开 MoonPub 首页` 已经开始承担“插件首页”的角色。实际使用时，你完全可以先点它，再从工作台里继续进入飞书、照片或当前文章路径。

插件详细说明和边界见：

- [../obsidian-plugin/README.md](../obsidian-plugin/README.md)

---

## 更多

- [GitHub](https://github.com/qiaopengjun5162/moonpub)
- [上手教程](GETTING_STARTED.md)
- [首页](https://paxonqiao.com/moonpub/)
- [问题反馈](https://github.com/qiaopengjun5162/moonpub/issues)

---

> MoonPub — Markdown → 微信公众号发布副驾驶。
