# Article Typography Polish Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Improve article typography choices and default rendered article polish without changing the publishing workflow.

**Architecture:** Keep `src/markdown.rs` as the dispatcher. Add typography behavior in `src/markdown/plain.rs`, keep theme presets in `src/theme.rs`, and update docs/progress after verification.

**Tech Stack:** Rust, inline CSS for WeChat-compatible HTML, `cargo nextest`, `cargo llvm-cov`.

---

### Task 1: Add Focused Typography Regression Tests

**Files:**
- Modify: `src/markdown.rs`
- Modify: `src/theme.rs`

- [x] Add tests for `editorial` and `zen` theme discovery and lookup.
- [x] Add tests that verify lead paragraph rendering, `####` subhead rendering, and image caption figure rendering.
- [x] Run `cargo nextest run --all-features markdown:: theme:: --status-level fail` and confirm the new tests fail before implementation.

### Task 2: Implement Typography Improvements

**Files:**
- Modify: `src/theme.rs`
- Modify: `src/markdown/plain.rs`

- [x] Add `Theme::editorial()` and `Theme::zen()`.
- [x] Register both names in `Theme::names()` and `Theme::from_name()`.
- [x] Track the first normal paragraph in `render_markdown_segment()` and render it as a non-indented lead paragraph.
- [x] Render Markdown images as a figure-style block with optional alt-text caption.
- [x] Render `####` as a compact subhead style.
- [x] Run focused tests and keep existing markdown behavior green.

### Task 3: Docs, Coverage, Verification, Commit

**Files:**
- Modify: `README.md`
- Modify: `README_zh.md`
- Modify: `AGENTS.md`
- Modify: `PROGRESS.md`

- [x] Update the theme count and theme tables from 12 to 14.
- [x] Update progress with real test and coverage numbers from fresh commands.
- [x] Run `cargo fmt --all -- --check`.
- [x] Run `cargo clippy --all-targets --all-features --tests --benches -- -D warnings`.
- [x] Run `cargo nextest run --all-features`.
- [x] Run `cargo llvm-cov nextest --all-features --summary-only`.
- [ ] Commit with a focused Conventional Commit message.
