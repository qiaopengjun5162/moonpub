# Markdown Fence Renderer Cleanup Design

## Goal

把 `src/markdown.rs` 里与 `:::` fence block 渲染直接相关的大段实现下沉到新的 `src/markdown/blocks.rs`，让 `markdown.rs` 回到“解析 Markdown block 并分发到对应渲染器”的入口层，同时保持当前 block 语法、输出 HTML、样式和测试结果不变。

## Why This Slice

`markdown.rs` 当前仍有 700+ 行，最明显的体量集中在 `render_fence_block` 和一串 block 专属 renderer 上。这里已经具备很自然的边界：

- `markdown.rs` 负责把正文拆成 `MdBlock`
- `render_fence_block` 根据 block name 路由
- 每个具体 renderer 只关心某一种 fence block 的 HTML 输出

这说明 fence renderer 已经是一个成型的子系统，先拆它，能在不触碰 inline markdown、普通段落渲染和 `render.rs` 的前提下，明显降低 `markdown.rs` 的复杂度。

## Scope

### In Scope

- 新增 `src/markdown/blocks.rs`
- 把 `render_fence_block` 从 `src/markdown.rs` 下沉到 `src/markdown/blocks.rs`
- 把只服务于 fence block 的 renderer 一并下沉，例如：
  - `render_book_info`
  - `render_intro`
  - `render_callout`
  - `render_steps`
  - `render_summary`
  - `render_figure`
  - `render_checklist`
  - `render_cover`
  - `render_generic_fence`
- 让 `src/markdown.rs` 保留：
  - `md_to_wechat_html`
  - `render_markdown_segment`
  - inline markdown 相关逻辑
  - `parser` 调用与 Markdown block 分发
- 保持现有 Markdown 测试和全量测试行为不变
- 在 `PROGRESS.md` 记录这次结构清理和真实验证结果

### Out of Scope

- 不改 `:::` block 语法
- 不改任何 block 的 HTML 样式或文案
- 不拆 inline markdown / emphasis / code span / list / paragraph 渲染
- 不改 `src/render.rs`
- 不改 `src/illustrate.rs`
- 不顺手优化 theme 结构或 block API

## Target Structure

本次完成后，Markdown 相关结构应变为：

```text
src/
  markdown.rs
  markdown/
    parser.rs
    blocks.rs
```

职责边界如下：

- `src/markdown.rs`
  - 保持 Markdown 转 HTML 的主入口
  - 遍历 `MdBlock`
  - 对普通 Markdown segment 继续直接渲染
  - 对 fence block 调用 `blocks::render_fence_block(...)`
- `src/markdown/blocks.rs`
  - 负责所有 fence block name 到具体 renderer 的映射
  - 负责 fence block 专属 HTML 拼装
  - 继续调用 `illustrate` 和 `theme`

## Design Decisions

### 1. 只拆 fence block，不碰 inline markdown

当前 `markdown.rs` 实际上混着两类复杂度：

- fence block 渲染
- 普通 Markdown / inline 语法渲染

这两类逻辑彼此独立，但第二类回归面更广，也更容易牵动普通段落输出。为了让这轮保持“小而稳”，只先拆第一类，不顺手碰第二类。

### 2. `render_fence_block` 和专属 renderer 一起下沉

如果只把 `render_fence_block` 下沉，而把 `render_book_info`、`render_callout` 等具体函数留在 `markdown.rs`，那只是“换了一个调度点”，不会真正减轻主文件复杂度。

所以这次要把 fence block 的路由和实现一起搬走，让 `markdown.rs` 对这部分只保留调用，而不再持有细节。

### 3. 保持 `illustrate` 依赖方向不变

现在一些 block 会调用 `illustrate::render_illustration`、`render_code_block`、`render_timeline`、`render_comparison`、`render_tip`。这次不重新设计这层边界，只是把这些调用从 `markdown.rs` 平移到 `blocks.rs`，避免把结构清理升级成架构重写。

### 4. 不为了“纯粹”新增过多抽象

这次目标是收薄文件，不是建立一套新的 block trait 或 registry 系统。`blocks.rs` 里继续保留直观的 `match name { ... }` 和若干专属函数就够了，后续真的有扩展压力，再讨论更重的抽象。

## Behavior Invariants

本次重构后，以下行为必须保持不变：

- `md_to_wechat_html` 的对外签名不变
- 所有已支持 block name 保持不变
- 未知 block 仍走 `render_generic_fence`
- `illustrate` 系列 block 的输出保持不变
- Markdown 测试断言文本不需要因结构调整而修改

如果实现过程中发现不得不改 block 输出内容，需要停下来单独确认，而不是把行为变化混进结构清理里。

## Testing Strategy

这轮是结构重构，验证重点是“输出没变”：

- 跑 `cargo fmt --all -- --check`
- 跑 `cargo clippy --all-targets --all-features --tests --benches -- -D warnings`
- 跑 `cargo nextest run --all-features markdown::`
- 再跑 `cargo nextest run --all-features`

若 Markdown 相关测试里没有直接覆盖某个关键 fence block 路径，可以补最小必要测试，但只能补“守住现有输出”的测试，不能借机扩行为。

## Documentation Impact

这次不需要改 README / README_zh，因为没有用户可见行为变化。

需要更新：

- `PROGRESS.md`
  - 记录 `markdown fence renderer` 已拆入 `src/markdown/blocks.rs`
  - 记录本次实际跑过的验证命令

若实现中发现新的长期结构约束明显且稳定，再补 `AGENTS.md`；否则不额外增加噪音。

## Risks And Mitigations

### 风险 1：函数依赖移动后出现循环或可见性问题

`blocks.rs` 会依赖 `illustrate`、`theme`，也可能需要使用 `inline_md` 或其他仍留在 `markdown.rs` 的函数。拆分时最容易出问题的是可见性和模块引用路径。

缓解方式：

- 先明确哪些函数继续留在 `markdown.rs`
- 只为 `blocks.rs` 暴露最小必要的 helper
- 避免在两个模块之间来回交叉调用过多函数

### 风险 2：结构拆了，但 `markdown.rs` 还是很厚

如果只搬走部分函数而保留大量 fence 细节在原文件，这轮收益会很有限。

缓解方式：

- 把 `render_fence_block` 和 fence 专属 renderer 一起迁走
- 实现后重新检查 `markdown.rs` 是否真的回到了“入口层”

### 风险 3：顺手碰到普通 Markdown 渲染逻辑

这会把本来清晰的结构重构任务，扩大成更高风险的渲染重构。

缓解方式：

- 不修改 `render_markdown_segment`
- 不改 inline markdown helper
- 不动 paragraph/list/code span 的行为

## Acceptance Criteria

满足以下条件即可视为这轮设计达成：

1. `src/markdown.rs` 不再持有 fence block 的大段实现细节，只负责主入口和分发。
2. 新增 `src/markdown/blocks.rs`，承接 `render_fence_block` 与 fence 专属 renderer。
3. Markdown 相关测试和全量验证通过，且没有输出行为回归。
4. `PROGRESS.md` 记录这次结构清理和真实验证结果。
