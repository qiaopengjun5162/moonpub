---
title: 用 Rust 写公众号发布工具（第 3 篇）：Vue 不认 JS 点击，改用 CDP 派发可信事件
theme: geek-black
digest: Vue 不认 JS 合成点击(isTrusted=false)。修法不是换库，是用 CDP 取坐标派发可信鼠标事件。本文讲坐标/iframe/shadow DOM/隐藏开关四个坑的真代码。
wechat_author: Paxon Qiao
---

上篇停在 6 月 12 日晚上 8 点 21 分，那条提交第一次把病根写进了消息里：

```text
20:21  fix: 原创 checkbox use CDP click for 我已阅读并同意 (Vue ignores JS events)
```

知道病根和把病治好，之间还隔着四个坑。这篇讲怎么从一句"Vue ignores JS events"变成能稳定跑通的代码——而且代码至今还在 `src/cdp.rs` 里，你可以打开对着看。

## 核心机制：为什么 `.click()` 没用

我用自动化"点"一个开关，一直是这么点的：

```js
document.querySelector('#origin-switch').click();
```

在普通网页上这没问题。但在微信编辑器这种 Vue 应用里，这段 JS 合成出来的点击事件，浏览器给它打的标记是 `isTrusted = false`。Vue 的模型层只认真人用鼠标产生的、内核派发的可信事件；`isTrusted=false` 的事件，它直接忽略。

所以现象是：脚本报"点成功了"，开关的视觉样式可能变了，但 Vue 的 `data` 没更新，你点"保存"时后台根本没收到这个改动。刷新页面，开关回到原样——就像你从没点过。

修法不是换库，是**换一种点击**：

> 先算出元素的屏幕坐标，再用 CDP 在那些坐标上派发真正的鼠标按下/抬起。Chromium 把它当成一次真人点击（`isTrusted = true`），Vue 才会认。

现在 `src/cdp.rs` 里的 `xclick` 函数，开头注释就是这么写的：

```rust
/// The click is a trusted CDP mouse event at the element's coordinates, NOT a
/// synthetic JS .click(). WeChat's editor ignores untrusted events for its Vue
/// model updates: synthetic clicks change the visual state but the change
/// never reaches the saved draft (settings silently revert on reload).
```

它的做法是：`getBoundingClientRect()` 取中心点坐标 → `page.click(Point { x, y })`。就这么简单，但每一个词都是那一晚试出来的。

## 坑一：开关在 iframe 里

我以为"取坐标点一下"就完了。第一个翻车点是：微信后台的设置面板，整个装在 `iframe[name="main"]` 里。我在顶层 `document` 上算坐标，算的是 iframe 外面的空气。

`xclick` 的查询顺序是先钻进这个 iframe 的 `contentDocument`，找不到才回退顶层：

```rust
var mainFr = document.querySelector('iframe[name="main"]');
if (mainFr) {
    var doc = mainFr.contentDocument;
    if (doc) { var m = /* 在 iframe 里查 */; if (m) return rect(m); }
}
var n = /* 在顶层 document 查 */;
```

更隐蔽的一点是：`setup_editor_for_draft` 里有一句注释——"Extracting appmsgid from static DOM doesn't work because WeChat uses Vue.js — the attribute is not in the static markup." 也就是说，连"草稿 ID 在哪个属性上"都不能靠静态 DOM 猜，因为 Vue 根本没把真实属性写进静态标记。这跟上一篇文章讲的 `vid ≠ appmsgid` 是同一个根：**微信后台是活的 Vue 应用，不是静态 HTML**。

## 坑二：按钮在 shadow DOM 里

好不容易钻进 iframe，有些按钮还是点不到。因为它们在 Web Component 的 **shadow DOM** 里——普通的 `querySelector` 摸不到 shadow root 内部。

6 月 12 日 19 点 54 分那次提交就是专门补这个的：

```text
19:54  fix: add shadow DOM traversal to cdp_click_text
```

现在 `cdp_click_text` 在遍历节点时，会递归下钻每一个 `shadowRoot`：

```rust
var all = root.querySelectorAll('*');
for (var j = 0; j < all.length; j++) {
    if (all[j].shadowRoot) {
        var sr = search(all[j].shadowRoot);
        if (sr) return sr;
    }
}
```

不递归这一层，那些藏在组件内部的"确定""确认"按钮就永远点不中。

## 坑三：那个开关是 0×0 的隐形元素

最阴的一个。微信的"原创"勾选框，真实的可点击控件是 `weui` 的开关，那个 `input` 的 `getBoundingClientRect()` 返回的是 **0 宽 0 高**——你算出来的中心点是它自己，点了等于点在空气上。真正能点的，是它外层的 `<label>`。

`xclick` 里专门处理了这种情况：

```rust
var r = n.getBoundingClientRect();
if (r.width === 0 && r.height === 0) {
    // Hidden inputs (e.g. weui switches) have no box; click their visible parent/label instead.
    var p = n.closest('label') || n.parentElement;
    if (!p) return null;
    p.scrollIntoView({block:'center'});
    r = p.getBoundingClientRect();
}
```

要先判断盒子是不是 0×0，不是就直接点；是，就回退到最近的 `<label>`。这个分支是我被一个永远点不亮的开关耗了快一小时才加上的。

## 坑四：不滚到底，Vue 根本不渲染

最后一个坑不在点击，在**渲染时机**。6 月 12 日 21 点 28 分：

```text
21:28  fix: scroll to bottom after editor load to trigger Vue settings rendering
```

微信编辑器的部分设置区，是滚动到可视范围才由 Vue 渲染出 DOM 的。你不滚到底，那些按钮在 DOM 里压根不存在，再怎么算坐标都是 NaN。

现在进编辑器后有一步固定操作 `settle_editor_page`，就是先滚到页面底部、再回滚一点，给 Vue 渲染留时间：

```rust
async fn settle_editor_page(page: &Page) {
    sleep_ms(3_000).await;
    page.evaluate("window.scrollTo(0, document.body.scrollHeight)").await.ok();
    sleep_ms(2_000).await;
    page.evaluate("window.scrollTo(0, document.body.scrollHeight - 500)").await.ok();
    sleep_ms(1_000).await;
}
```

## 我错在哪

这四个坑，对应的不是四个 bug，是一个**错误的底层假设**：我一直以为"点一个元素"等于"在那个元素上调用 `.click()`"。这个假设在原生 DOM 上成立，在"Vue + shadow DOM + iframe + 隐形控件 + 懒渲染"叠满的微信编辑器上，一步都不成立。

上一篇文章说"如果早点看 `isTrusted` 半小时就结束"。更准确地说，省下的是**换库那一百分钟**；但即使不换库，这四个坑照样要一个一个踩——因为它们不是"库不行"，是"我和这个页面之间的协议"从根上就理解错了。

`src/cdp.rs` 现在把这套认知固化成了四个函数：`xclick`（坐标 + iframe + 隐形 label 回退）、`cdp_click_text` / `cdp_click_exact_last`（含 shadow DOM 递归）、`settle_editor_page`（滚底渲染）。这四个加起来，就是那三个半小时加一整晚的代价。

## 你可以自己核对

这篇文章里的代码都来自当前 `src/cdp.rs`，提交哈希来自真实历史。复算：

```bash
# 当前实现
sed -n '85,153p' src/cdp.rs        # xclick：坐标点击 + iframe 穿透 + label 回退
sed -n '170,221p' src/cdp.rs       # cdp_click_text：含 shadow DOM 递归
sed -n '803,813p' src/cdp.rs       # settle_editor_page：滚底触发渲染

# 06-12 那一段的历史提交
git show e750b8c      # 20:21 原创 checkbox 用 CDP 点击，首次点出 Vue 根因
git show 12038c11     # 19:54 shadow DOM 遍历
git show c12aca72     # 21:28 滚底触发 Vue 渲染
```

## 下一篇

开关点了、Vue 认了、坐标对了，但还有一个更反直觉的坑：**点了"保存"，刷新页面设置就没了**。因为设置只活在页面内存里，不点"保存为草稿"不落库，而且保存前必须关掉残留弹窗，否则弹窗挡住保存按钮，整轮白做。下一篇写这场持久化拉锯战。

---

项目在这里：[github.com/qiaopengjun5162/moonpub](https://github.com/qiaopengjun5162/moonpub)
