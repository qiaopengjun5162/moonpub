---
title: 用 Rust 写公众号发布工具（第 5 篇）：8 篇已发布文章，正文顶部印着构建 tag
theme: geek-black
cover_subtitle: 正文顶部印着构建 tag
cover_tag: MOONPUB · 05 / 07
digest: 8篇已发文章正文第一屏露出moonpub render裸tag。根因:封面HTML拼进正文,微信剥CSS露裸DOM,本地预览看不出。08-13只中性化治标,08-25把封面移出正文。
wechat_author: Paxon Qiao
---

前四篇讲的都是"我在开发时踩的坑"。这一篇不一样——坑是我踩的，但**读者替我发现了**，而且是在 8 篇已经发出去的文章上。

2026-08-13，用户从一篇已发布公众号文章的链接点进去复查，回来跟我说：你文章正文第一屏写着 `$ BUILD NOTES` 和 `moonpub render Paxon Qiao`。

我当时愣了一下。那是我封面模板里的装饰文案——一个仿终端风格的卡片，顶部一行小字、一行构建标签。我在本地预览里看过无数次，它从来都是一张漂亮的封面图。怎么会出现在正文里？

答案藏在我一直忽略的一行代码里。

## 根因：封面 HTML 被整段塞进了正文开头

`render_article`（src/render.rs）早年有一行看起来很合理的逻辑：

```rust
let body_with_cover = match cover_html {
    Some(cover) => format!("{cover}\n{html_body}"),
    None => html_body,
};
```

封面 HTML 整段拼到正文开头。结构是这样的：

```html
<main class="cover" data-cover-style="geek-black">
  <div class="tag">WEB3 · DEV</div>
  <h1>文章标题</h1>
  <p>文章摘要</p>
  <p>Paxon Qiao</p>
</main>
<!-- 下面是真正的正文 -->
```

本地预览时，封面单独渲染成 `.cover.html` / `.cover.png`，我看的从来都是那张图，**从来没看过"拼进正文"之后的样子**。

但微信后台不是浏览器。它拿到这段 HTML 后，会把大部分 `<style>` 样式表剥掉，只保留裸 DOM。于是读者在手机上看到的，是没了样式的封面骨架——一行绿油油的 `WEB3 · DEV`、一个没排版的标题、一行 `Paxon Qiao`，然后才是正文。

**本地好看，线上裸奔。** 从 2026-08-06 第一批文章（arb-layers）到 08-12（ai-judgment / guide-first-flow），全部 8 篇已发布文章都带着这行裸 tag 上线了。本地永远发现不了，因为本地根本不走"拼进正文"那条渲染路径。

## 我错了第一次：把文案中性化，就当修好了

08-13 那天我发现后，做了第一次修复——把文案从工具内部信息换成中性技术语汇：

```text
moonpub render    →  WEB3 · DEV
$ BUILD NOTES     →  $ TECH NOTES
```

还顺手更新了单元测试断言，确认新文案在位、旧文案不在。

现在回头看，这就是典型的**治标不治本**。我把露出来的字换成了不尴尬的说法，却没有追问一个更根本的问题：

> 为什么封面 HTML 会出现在正文第一屏？

只要封面还拼在正文里，微信就一定会剥 CSS、露裸 DOM。我今天写 `WEB3 · DEV`，明天改 `CLOUD · NATIVE`，它照样裸露给读者。我那次修复，只是把"尴尬的裸 tag"换成了"体面的裸 tag"。

## 我错了第二次：把封面当成"内部产物"

更深的一层错：我长期把封面 HTML 当成发布流水线的内部实现，觉得"那是给工具自己看的预览，不是给用户的内容"。

`ENGINEERING_LESSONS_ZH.md` 后来把这条写成了铁律（第 225 条）：

> 封面 HTML 属于「用户可见内容」，评审视角与正文一致，不能当内部实现看待。

一旦你接受"封面会被送进读者眼睛"，很多决策都会变：装饰文案不能用工具名、作者品牌、版本号；每次 ship 之后不能只看本地预览，得去线上抽查正文第一屏。

## 真正的修复：封面从正文里彻底消失

2026-08-25，提交 `3d67d91` 把根因拔掉了。改动只有几行，但方向对了：

```rust
// 2026-08-25: 封面 HTML 不再嵌入正文。
//
// 此前 `cover_html` 会整段拼到正文开头（<main class="cover"
// data-cover-style=…> 标题 + digest + tag + 作者）。微信编辑器会剥离
// 封面样式表，只留下裸 DOM，导致已发布文章正文顶部出现
// "WEB3 · DEV Paxon Qiao" 这类构建 tag 文字（D17/D18/D19 线上文章实测）。
// 封面作为 thumb_media_id 图片已单独设置，本地预览看 .cover.html/.cover.png，
// 正文不需要再嵌一份封面模板。
let _ = cover_html;
let body_with_cover = html_body;
```

要点：

1. **封面不再拼进正文**。微信封面本来就是单独的 `thumb_media_id` 图片字段，根本不需要塞进正文 HTML。
2. **本地预览照旧**。`.cover.html` / `.cover.png` 还在，作者设计封面时看的就是那份，不受影响。
3. **正文干干净净**。不管封面模板里写了什么，都不会再泄漏到读者屏幕上。

配套加了一条回归测试 `cover_html_not_embedded_into_body`（src/render.rs:351），它故意塞一份带 `WEB3 · DEV` 的封面 HTML 进去，然后断言本地 `.html` 和微信 `draft.json` **都不含** `data-cover-style`、也**都不含** `WEB3 · DEV`：

```rust
assert!(!html.contains("data-cover-style"), "本地预览不应嵌入封面 HTML");
assert!(!json_str.contains("data-cover-style"), "微信推送正文不应嵌入封面 HTML");
assert!(!json_str.contains("WEB3 · DEV"), "tag 文字不得泄漏进微信正文");
```

这条测试把"封面不得进正文"钉死，以后谁再把 `cover_html` 拼回去，CI 立刻红。

## 防复发：三条规则

从这两轮事故里沉淀出三条，现在锁在 `AGENTS.md` / `ENGINEERING_LESSONS_ZH.md`：

1. **封面装饰文案必须通用技术语汇，禁止出现工具名、作者品牌、版本信息**。加了测试断言 `!contains("moonpub")`，新封面风格谁写了 `moonpub` 谁过不了测试。
2. **每次 ship 后抽查已发布文章正文第一屏**。浏览器打开 `mp.weixin.qq.com` 链接，`js_content` 开头应直接是正文标题，不该有任何封面残骸。
3. **封面 HTML 是用户可见内容，评审标准和正文一致**。

## 时间线：一个 bug，两次修复

```text
08-06 → 08-12   8 篇已发布文章带着封面裸 tag 上线（本地预览看不出）
08-13           用户从已发布 URL 复查发现；第一次修复：文案中性化（治标）
08-25           3d67d91：封面 HTML 不再嵌入正文（治本）
```

08-13 那次改的 `TECH NOTES` / `WEB3 · DEV` 字样现在还留在 `geek-black` 封面模板里（src/cover.rs 的 `render_geek_black_cover`），但它们只用于封面图本身，再也不会进正文——所以那次"中性化"不是白做，它让封面图本身更体面；只是它从来不是真正的修复。

## 验证：代码还在，你可以对着看

- 根因注释 + 当前实现：`sed -n '48,60p' src/render.rs`
- 回归测试：`sed -n '351,390p' src/render.rs`
- 事故记录：`sed -n '225,238p' docs/ENGINEERING_LESSONS_ZH.md`

确认本地没有遗留的嵌入封面（应为空）：

```bash
grep -rl "data-cover-style" ready/ drafts/ output/ 2>/dev/null
```

提交本人可查：

```bash
git show 3d67d91 --stat
```

## 带走的一句话

> 本地预览和线上环境不是同一个渲染器。凡是"只在本地看过"的产物，第一次发布后都要去线上亲眼复查一遍——尤其是那些你以为"只是内部实现"的东西。

下一篇讲另一个数字陷阱：覆盖率 71.99% 看着很美，但核心发布链路只有 1.57%——因为微信 API 的域名被写死在代码里，根本没法注入测试。
