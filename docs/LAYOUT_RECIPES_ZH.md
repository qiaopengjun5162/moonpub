# 微信文章排版配方

这份文档不是完整语法手册，而是给写作时快速选版式用的。优先按文章类型选一套配方，再按内容删减 Block。

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

推荐主题：`geek` / `notebook` / `ocean`

推荐结构：

````markdown
---
theme: geek
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

## 选择原则

- 如果文章偏生活、安静、慢读，先试 `mist`。
- 如果文章来自飞书妙记或散步口述，先试 `letter`，用 `letter-card` 保留原话感。
- 如果文章是合集第一篇，先试 `editorial` 或 `mist`，用 `letter-card` 交代边界和写作约定。
- 如果文章照片多，先试 `gallery`，再配 `photo-grid`。
- 如果文章像写给读者的一封信，先试 `letter`。
- 如果文章是读书笔记，优先 `paper`，不要一上来用太花的块。
- 如果只是普通技术教程，优先 `geek` 或 `notebook`，少用装饰块。
- 如果文章是日报/周报，优先 `notebook`，用 `compact-links` 把文末来源压成小字号索引。
- Block 不要堆满整篇文章。通常一篇文章用 2-4 个视觉块就够了。
