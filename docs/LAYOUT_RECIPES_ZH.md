# 微信文章排版配方

这份文档不是完整语法手册，而是给写作时快速选版式用的。优先按文章类型选一套配方，再按内容删减 Block。

## 先选主题

如果你只是想快速决定“这篇文章看起来应该像什么”，先按下面选，不用先读完整配方：

| 文章类型 | 推荐主题 | 封面 style | 适合场景 |
|----------|----------|------------|----------|
| 技术教程 / 工程复盘 | `geek` / `notebook` / `ocean` | `dark` | 稳妥清晰，代码块突出，适合大多数技术文 |
| 极客黑工程文章 | `geek-black` | `geek-black` | AI / Rust / Web3 / 终端感项目复盘 |
| 系统设计 / 协议说明 | `blueprint` | `blueprint` | 架构边界、模块拆分、协议设计、方案评审 |
| AI 工程实验 | `ai-lab` | `ai-lab` | Agent 工作流、模型评测、提示词系统、AI 产品工程 |
| 生活随笔 / 慢读 | `mist` / `letter` / `porcelain` | `literary` | 日常记录、散步口述、私人表达 |
| 照片留档 | `gallery` / `fieldnote` | `forest` | 多张照片、旅行碎片、相册记忆 |
| 日更贴图 / 图文贴片 | `gallery` / `fieldnote` / `newsletter` | `forest` | 每天一组图、平台贴图流、轻量更新 |
| 读书笔记 / 研究 | `paper` / `classic` / `academic` | `serif` | 书摘、结构化论证、长文阅读 |
| 日报周报 / 信息流 | `newsletter` / `notebook` / `editorial` | `workflow` | AI/Web3 日报、资料索引、链接合集 |
| 透明信源简报 | `newsletter` / `notebook` / `academic` | `workflow` | 官方 release 汇总、多源候选精编、每条消息保留来源 |

命令行里也能看同一份选型数据：

```bash
moonpub layout-recipes
moonpub --json layout-recipes
```

## 生活随笔

适合「闲月隐林」这类日常、散步、跑步、心绪记录。

推荐主题：`mist` / `letter` / `forest`

推荐结构：

```markdown
---
theme: mist
---

:::meta-strip
date: 2026-07-03
place: 河边小路
weather: 晚风
mood: 安静
今天只记一个真实的小片段。
:::

:::intro
开篇用 1-3 句话交代这篇文章为什么写。
:::

## 当时看到的

正文段落。

:::scene-card
label: 路上
place: 月下林边
这里放一段真实场景，不要过度修饰。
:::

## 后来想到的

正文段落。

:::closing-card
label: 慢慢来
给文章一个温柔收束。
:::
```

## 照片记录

适合同一天多张照片、跑步风景、旅行碎片、生活留档。

推荐主题：`gallery` / `mist` / `warm`

推荐结构：

```markdown
---
theme: gallery
---

:::intro
这一组照片记录的是哪一天、哪件事、为什么想留下来。
:::

:::photo-grid
- /photos/day-1.jpg | 雨后的树影
- /photos/day-2.jpg | 回家的路
- /photos/day-3.jpg | 天色暗下来的时候
:::

:::scene-card
label: 现场
place: 河边小路
把照片里确实发生的事说清楚。
:::

## 这一天

正文段落。
```

## 静谧开篇

适合「闲月隐林」这类合集第一篇，重点是交代名字、边界、心境和以后怎么写。它比 `collection-opener` 更克制，颜色也更低饱和。

推荐主题：`moonlit` / `porcelain` / `letter`

推荐结构：

```markdown
---
theme: moonlit
---

:::meta-strip
mood: 月下栖林、七分明说、三分自留
闲月隐林是一片安静的小林子，用来慢慢记录日常和所思所感。
:::

:::intro
欢迎来到这片林子。这里不追求热闹，只想把可以公开说的部分认真留下来。
:::

:::letter-card
title: 写在开篇
date: 2026-07-07
有些话明说，有些话留白。这个合集会在两者之间慢慢找到自己的节奏。
:::

:::scene-card
label: 起念
place: 月下林边
放一段真实起因：是什么契机让你想把这个地方建起来。
:::

:::closing-card
label: 入林
以后就在这里，慢慢写，慢慢沉淀。
:::
```

## 记忆留档

适合同一天的几张照片、散步或跑步后的生活片段。原则是实事求是，不替照片编故事，也不把日常过度拔高。

推荐主题：`fieldnote` / `gallery` / `porcelain`

推荐结构：

```markdown
---
theme: fieldnote
---

:::meta-strip
date: 2026-07-07
place: 河边小路
mood: 想留住这一天
这是一组从手机相册里捡回来的生活片段，只记录真实发生过的事。
:::

:::intro
这天没有特别大的事情，只是几张照片刚好把当时的路、风和心情留下来了。
:::

:::photo-grid
- /photos/river-1.jpg | 树影落在路边
- /photos/river-2.jpg | 天色慢慢暗下来
- /photos/river-3.jpg | 回家前拍的一张
:::

:::scene-card
label: 现场
place: 河边小路
照片里能确认的是：路边有风，天色在变暗，我那时候正慢慢往回走。
:::

:::closing-card
label: 留档
先把这一天留在这里，免得以后只剩下一堆快要删掉的照片。
:::
```

## 日更贴图

适合每天一组图文贴片、平台贴图流、轻量更新。原则是“先留住当天的图和一句话”，不要为了日更把照片硬拔成宏大文章。

推荐主题：`gallery` / `fieldnote` / `newsletter`

推荐结构：

```markdown
---
theme: gallery
---

:::meta-strip
date: 2026-08-13
source: 今日贴图
mood: 先留一组
这是一条轻量图文更新，只记录今天最想留下来的几张图。
:::

:::intro
今天这组图的主题是一句话：把最真实的现场先贴出来。
:::

:::photo-grid
- /photos/today-1.jpg | 第一张图的真实说明
- /photos/today-2.jpg | 第二张图的真实说明
:::

:::compact-links
- 原始素材：本地相册 / 今日截图
- 下一步：如果这组图值得展开，再改成长文或照片记录
:::
```

## 口述随记

适合飞书妙记、散步录音、跑步后随口复盘、碎片想法整理成一篇可读文章。

推荐主题：`letter` / `mist` / `notebook`

推荐结构：

```markdown
---
theme: letter
---

:::meta-strip
date: 2026-07-05
place: 散步路上
mood: 边走边想
这篇来自一次口述记录，只保留当时真正说到的线索。
:::

:::intro
先用 2-3 句话交代这段口述的起因，不要把它包装成宏大结论。
:::

:::letter-card
title: 当时想说的是
date: 2026-07-05
把口述里最核心、最像“原话”的一段放在这里，保留一点呼吸感。
:::

## 慢慢整理

正文段落。

:::summary
- 一个确定发生过的事实
- 一个当时冒出来的判断
- 一个可以以后再展开的问题
:::

:::closing-card
label: 先记到这里
这次先不急着下结论，留给下一次继续想。
:::
```

## 合集开篇

适合栏目第一篇、付费合集序章、个人小专栏开场，例如「闲月隐林」这种需要交代名字、边界、心境和以后怎么写的文章。

推荐主题：`editorial` / `mist` / `letter`

推荐结构：

```markdown
---
theme: editorial
---

:::meta-strip
mood: 松弛、克制、慢慢写
闲月隐林：七分明说，三分自留。这是一篇给新合集立边界的开篇文。
:::

:::intro
先用 2-4 句话欢迎读者进入这里，也说明这片地方为什么存在。
:::

:::letter-card
title: 给读者的一封短笺
date: 2026-07-04
这里可以写“这个合集准备记录什么、不准备追求什么、希望读者以什么心态来看”。
:::

:::scene-card
label: 起点
place: 月下林边
放一段真实起因：是什么契机让你想把这个地方建起来。
:::

## 慢慢开始

正文段落。

:::closing-card
label: 欢迎进来
以后就在这里，慢慢写，慢慢聊。
:::
```

## 读书笔记

适合书摘、微信读书导入、阅读后的结构化思考。

推荐主题：`paper` / `classic` / `academic`

推荐结构：

```markdown
---
theme: paper
---

:::book-info
title: 书名
author: 作者
publisher: 出版社
rating: 8.5
:::

:::intro
这本书最打动你的地方是什么。
:::

:::key-points
- 一个核心观点
- 一个关键例子
- 一个自己的反思
:::

## 让我停下来的地方

正文段落。

:::pull-quote
source: 《书名》

值得慢下来读的一句话。
:::
```

## 技术文章

适合教程、踩坑记录、项目复盘、工程说明。

推荐主题：`geek` / `geek-black` / `blueprint` / `notebook` / `ocean`

推荐结构：

````markdown
---
theme: geek-black
---

:::intro
这篇文章解决什么问题，读者看完能得到什么。
:::

:::callout
label: 结论
先把最终判断放出来。
:::

## 背景

正文段落。

:::steps
1. 先确认现象
2. 再定位根因
3. 最后给出修复
:::

## 关键实现

```rust
fn main() {
    println!("hello");
}
```

:::summary
最后总结边界、风险和下一步。
:::
````

## AI 工程笔记

适合 Agent 工作流、模型评测、提示词系统、AI 产品工程复盘。

推荐主题：`ai-lab` / `geek-black` / `cyber` / `blueprint`

推荐结构：

```markdown
---
theme: ai-lab
---

:::intro
这次记录一个 AI 工程实验：它解决什么问题，失败边界在哪里。
:::

:::callout
label: 结论
先把最重要的判断写出来，不要让读者等到最后才知道结果。
:::

:::concept-card
number: 1
工作流契约
把可执行命令、风险边界和下一步建议都显式暴露给插件和 Agent。
:::

:::steps
1. 先确认输入和约束
2. 再跑最小可验证路径
3. 最后补回归和文档
:::

:::compact-links
- 01 | 协议说明 | MoonPub｜本地文档 | docs/AGENT_PROTOCOL_ZH.md
:::

:::summary
- 这次实验已经验证的事实
- 还不能承诺的边界
- 下一次最值得补的证据
:::
```

## 系统设计复盘

适合架构边界、模块拆分、协议设计、技术方案评审。

推荐主题：`blueprint` / `notebook` / `academic` / `geek`

推荐结构：

```markdown
---
theme: blueprint
---

:::intro
先把这次设计要解决的问题、非目标和真实约束讲清楚。
:::

:::concept-card
number: 1
核心边界
这个模块只负责状态发现，不触发发布、不打开浏览器、不写回文件。
:::

:::steps
1. 当前系统怎么跑
2. 问题出在哪个边界
3. 新方案如何收口
:::

:::key-points
- 哪些接口保持稳定
- 哪些行为必须显式确认
- 哪些风险留给人工复核
:::

:::summary
最后用 3-5 条把取舍、验证和下一步收住。
:::
```

## 日报周报

适合 AI / Web3 日报、资料索引、可追溯信息流和群聊素材沉淀。

推荐主题：`notebook` / `newsletter` / `editorial`

推荐结构：

```markdown
---
theme: notebook
---

:::intro
先用 2-3 句话告诉读者今天最值得看的主线。
:::

:::divider
label: 今日速览
:::

:::summary
- 第一条核心信号
- 第二条核心信号
- 第三条核心信号
:::

:::callout
label: 先读这条
这里放今天最重要的一件事：发生了什么、为什么重要、后续看什么。
:::

## 参考来源

:::compact-links
- 01 | 原文标题 | OpenAI｜官方公告 | https://example.com/source
- 02 | 研究文章 | Ethereum Research｜深读 | https://example.com/research
:::
```

## 透明信源简报

适合 AI 早报、官方 release 汇总、多源候选精编和需要“每条消息都能点回来源”的信息流。它的重点不是多塞信息，而是让读者看清每条消息来自哪里、可信度如何、为什么值得看。

推荐主题：`newsletter` / `notebook` / `academic`

推荐结构：

```markdown
---
theme: newsletter
---

:::meta-strip
date: 2026-08-13
scope: AI / Agent / 模型发布
source_policy: 官方优先；媒体和社区信号必须保留原文链接
本简报只做可追溯信息整理，不把单源传闻写成确定事实。
:::

:::intro
今天最值得看的主线：官方 release、开发者生态和开源项目都在围绕同一个变化收束。
:::

:::summary
- 官方确认：一条已经有官方公告或 release note 的更新
- 多源印证：一条被多个可信来源同时提到的趋势
- 单源信号：一条值得观察、但还不能下定论的线索
:::

:::callout
label: 今日必须看
先写发生了什么，再写为什么重要，最后写后续应该盯哪个官方链接或仓库。
:::

## 来源索引

:::compact-links
- 01 | 官方公告标题 | OpenAI｜官方确认 | https://example.com/openai-release
- 02 | Release Note | GitHub｜官方 release | https://example.com/release
- 03 | 研究论文 | arXiv｜单源信号 | https://example.com/paper
:::
```

这套配方只规定本地文章结构。它不负责后台抓取、不配置 API token，也不自动生成或发布早报；如果素材来自外部日报系统，先把 Markdown 留在本地，再用 `layout-audit` / `preflight` 进入 MoonPub 发布链路。

## 选择原则

- 如果文章偏生活、安静、慢读，先试 `mist` 或 `porcelain`。
- 如果文章是「闲月隐林」这类合集开篇，优先试 `moonlit`，它比绿色主题更克制。
- 如果文章来自飞书妙记或散步口述，先试 `letter`，用 `letter-card` 保留原话感。
- 如果文章是合集第一篇，先试 `editorial` 或 `mist`，用 `letter-card` 交代边界和写作约定。
- 如果文章照片多，先试 `gallery`；如果是相册留档和散步记录，先试 `fieldnote`，再配 `photo-grid`。
- 如果文章像写给读者的一封信，先试 `letter`。
- 如果文章是读书笔记，优先 `paper`，不要一上来用太花的块。
- 如果只是普通技术教程，优先 `geek` 或 `notebook`，少用装饰块；如果是 AI / Rust / Web3 工程复盘，想要终端感，可以试 `geek-black`。
- 如果文章是在解释系统架构或协议边界，优先试 `blueprint`；如果是在记录 AI 工程实验，优先试 `ai-lab`。
- 如果文章是普通日报/周报，优先 `notebook`，用 `compact-links` 把文末来源压成小字号索引；如果简报强调信源透明，优先 `newsletter`，每条信息都标清 `官方确认` / `多源印证` / `单源信号`。
- 如果文章偏技术、代码密集，或想要编辑器氛围，可以试 `monokai` / `dracula` / `nord` / `one-dark` / `gruvbox`；这几套是深色底，长文请先在手机上预览确认阅读舒适度。
- Block 不要堆满整篇文章。通常一篇文章用 2-4 个视觉块就够了。
