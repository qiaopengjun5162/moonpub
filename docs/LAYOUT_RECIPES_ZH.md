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

## 选择原则

- 如果文章偏生活、安静、慢读，先试 `mist`。
- 如果文章照片多，先试 `gallery`，再配 `photo-grid`。
- 如果文章像写给读者的一封信，先试 `letter`。
- 如果文章是读书笔记，优先 `paper`，不要一上来用太花的块。
- 如果只是普通技术教程，优先 `geek` 或 `notebook`，少用装饰块。
- Block 不要堆满整篇文章。通常一篇文章用 2-4 个视觉块就够了。
