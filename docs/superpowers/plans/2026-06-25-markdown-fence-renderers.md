# Markdown Fence Renderer Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move the fence block rendering path out of `src/markdown.rs` into `src/markdown/blocks.rs` so the root Markdown module becomes a thin parser-and-dispatch entrypoint without changing any rendered HTML behavior.

**Architecture:** Keep `src/markdown.rs` responsible for parsing `MdBlock`s, rendering plain Markdown segments, and hosting inline helpers such as `inline_md`. Introduce `src/markdown/blocks.rs` to own `render_fence_block` and the fence-only renderer functions, while continuing to call into `illustrate` and `theme` exactly as before.

**Tech Stack:** Rust 2024 edition, existing `markdown` module, `illustrate`, `theme`, `cargo fmt`, `cargo clippy`, `cargo nextest`.

---

### Task 1: Create `blocks.rs` And Move Fence Dispatch

**Files:**
- Create: `src/markdown/blocks.rs`
- Modify: `src/markdown.rs`
- Test: `src/markdown.rs`

- [ ] **Step 1: Wire the new submodule in `src/markdown.rs`**

Add a `blocks` submodule declaration and update `md_to_wechat_html` to call `blocks::render_fence_block(...)`.

```rust
mod blocks;
mod parser;

use parser::{MdBlock, parse_blocks};

pub fn md_to_wechat_html(md: &str, theme: &theme::Theme) -> String {
    let blocks = parse_blocks(md);
    let mut out = String::new();

    for block in &blocks {
        match block {
            MdBlock::Fence(name, props, body) => {
                out.push_str(&blocks::render_fence_block(name, props, body, theme))
            }
            MdBlock::Markdown(text) => out.push_str(&render_markdown_segment(text, theme)),
        }
    }

    out
}
```

Expected intermediate state: `cargo check` would fail until `src/markdown/blocks.rs` exists and exports `render_fence_block`.

- [ ] **Step 2: Create `src/markdown/blocks.rs` with the fence router**

Create the new file and move `render_fence_block` into it. Keep the same `match name { ... }` shape and continue to call `illustrate` helpers exactly as before.

```rust
use crate::illustrate;
use crate::theme;

use super::inline_md;

pub(crate) fn render_fence_block(
    name: &str,
    props: &[(&str, &str)],
    body: &str,
    theme: &theme::Theme,
) -> String {
    match name {
        "book-info" => render_book_info(props, theme),
        "intro" => render_intro(body, theme),
        "callout" => render_callout(props, body, theme),
        "steps" => render_steps(body, theme),
        "summary" => render_summary(body, theme),
        "figure" => render_figure(props, theme),
        "checklist" => render_checklist(body, theme),
        "cover" => render_cover(props, theme),
        // ... keep existing remaining cases unchanged
        _ => render_generic_fence(name, body, theme),
    }
}
```

- [ ] **Step 3: Run Markdown-focused tests to catch integration mistakes early**

Run:

```bash
cargo nextest run --all-features markdown:: --status-level pass
```

Expected: existing Markdown tests still pass, even before every fence-specific helper is moved.

- [ ] **Step 4: Commit the new module boundary**

```bash
git add src/markdown.rs src/markdown/blocks.rs
git commit -m "refactor: add markdown fence blocks module"
```

### Task 2: Move Fence-Specific Renderers Into `blocks.rs`

**Files:**
- Modify: `src/markdown.rs`
- Modify: `src/markdown/blocks.rs`
- Test: `src/markdown.rs`

- [ ] **Step 1: Move the fence-only renderer functions**

Move these functions out of `src/markdown.rs` into `src/markdown/blocks.rs` without changing their bodies:

- `render_book_info`
- `render_intro`
- `render_callout`
- `render_steps`
- `render_summary`
- `render_figure`
- `render_checklist`
- `render_cover`
- `render_generic_fence`

Keep their signatures unchanged except for module path adjustments.

- [ ] **Step 2: Expose only the minimal helper needed from `markdown.rs`**

If `blocks.rs` needs `inline_md`, expose it with the smallest practical visibility from `src/markdown.rs`.

```rust
pub(crate) fn inline_md(text: &str, theme: &theme::Theme) -> String {
    // existing body unchanged
}
```

Do not widen any other helper visibility unless the compiler proves it is necessary.

- [ ] **Step 3: Run Markdown-focused tests again**

Run:

```bash
cargo nextest run --all-features markdown:: --status-level pass
```

Expected: all Markdown-focused tests remain green with no assertion text changes.

- [ ] **Step 4: Commit the renderer extraction**

```bash
git add src/markdown.rs src/markdown/blocks.rs
git commit -m "refactor: extract markdown fence renderers"
```

### Task 3: Thin `src/markdown.rs` To Entry Layer

**Files:**
- Modify: `src/markdown.rs`
- Test: `src/markdown.rs`

- [ ] **Step 1: Remove now-duplicated fence implementation from `src/markdown.rs`**

After Task 2, `src/markdown.rs` should keep:

- module declarations
- `md_to_wechat_html`
- `render_markdown_segment`
- inline markdown helpers such as `inline_md`
- ordinary Markdown rendering logic
- tests

It should no longer contain:

- `render_fence_block`
- any fence-only renderer moved to `blocks.rs`

- [ ] **Step 2: Fix imports and visibility after thinning**

Clean up now-unused imports in `src/markdown.rs`, and make sure `src/markdown/blocks.rs` imports exactly what it needs (`illustrate`, `theme`, and the minimal helpers from `markdown.rs`).

- [ ] **Step 3: Run formatting and lint checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
```

Expected: both commands pass without introducing new warnings.

- [ ] **Step 4: Commit the thin root module state**

```bash
git add src/markdown.rs src/markdown/blocks.rs
git commit -m "refactor: thin markdown root module"
```

### Task 4: Full Verification And Progress Sync

**Files:**
- Modify: `PROGRESS.md`

- [ ] **Step 1: Run full project verification**

Run:

```bash
cargo nextest run --all-features
```

Expected: the full suite passes, not just Markdown-focused tests.

- [ ] **Step 2: Record the refactor in `PROGRESS.md`**

Append a version log entry in the existing style, for example:

```markdown
- 2026-06-25: **Markdown fence renderer 拆分** — `render_fence_block` 与 fence 专属 renderer 移入 `src/markdown/blocks.rs`；`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 通过
```

- [ ] **Step 3: Verify doc scope stayed minimal**

Run:

```bash
git diff --stat
```

Expected: only `src/markdown.rs`, `src/markdown/blocks.rs`, and `PROGRESS.md` changed for this task, ignoring any unrelated pre-existing worktree changes.

- [ ] **Step 4: Commit the progress sync**

```bash
git add PROGRESS.md
git commit -m "docs: record markdown fence cleanup"
```

### Task 5: Final Hygiene Check

**Files:**
- No new source files expected beyond prior tasks.

- [ ] **Step 1: Run patch hygiene**

Run:

```bash
git diff --check
```

Expected: no whitespace or merge-marker issues.

- [ ] **Step 2: Review final status**

Run:

```bash
git status --short
git log --oneline -5
```

Expected: the Markdown cleanup files are in the expected state, and recent commits clearly reflect the module creation, renderer extraction, root thinning, and docs sync.
