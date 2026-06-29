# Feishu Draft Auto Push Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add an explicit `--push` option so Feishu-driven draft generation can directly continue into `push --render`.

**Architecture:** Keep the change narrow: extend CLI parsing for `draft-from-inbox` and `intake feishu`, add a small app-layer auto-push branch that reuses `push_article(..., true, ...)`, and expose the extra push result in the existing structured JSON output only when `--push` is requested.

**Tech Stack:** Rust std, existing CLI parser, existing app JSON builders, cargo-nextest

---

### Task 1: Add failing CLI and JSON tests

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/app.rs`
- Test: `src/cli.rs`
- Test: `src/app.rs`

- [ ] **Step 1: Add a failing CLI test for `draft-from-inbox --push`**
- [ ] **Step 2: Add a failing CLI test for `intake feishu --draft --push`**
- [ ] **Step 3: Add failing CLI validation tests for `--push` requiring `--draft` and conflicting with `--preview`**
- [ ] **Step 4: Add failing app JSON builder tests for push metadata fields**
- [ ] **Step 5: Run focused tests and confirm they fail for the expected reasons**

### Task 2: Extend CLI parsing with explicit auto-push options

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/error.rs`
- Test: `src/cli.rs`

- [ ] **Step 1: Extend `Command::DraftFromInbox` with an `auto_push` flag**
- [ ] **Step 2: Extend `Command::IntakeFeishu` with an `auto_push` flag**
- [ ] **Step 3: Parse `--push` and enforce the new validation rules**
- [ ] **Step 4: Update help text**
- [ ] **Step 5: Re-run focused CLI tests**

### Task 3: Add app-layer auto-push orchestration

**Files:**
- Modify: `src/app.rs`
- Test: `src/app.rs`

- [ ] **Step 1: After draft generation, call `push_article(..., true, ...)` when `auto_push` is enabled**
- [ ] **Step 2: Keep plain-text output as concatenated intake/draft/push messages**
- [ ] **Step 3: Extend structured JSON with `pushed`, `media_id`, `stage`, and `next_step` when auto-push is enabled**
- [ ] **Step 4: Add/adjust tests so failure-before-network behavior is still protected**
- [ ] **Step 5: Re-run focused app tests**

### Task 4: Sync docs and run full verification

**Files:**
- Modify: `README.md`
- Modify: `README_zh.md`
- Modify: `docs/USER_GUIDE.md`
- Modify: `PROGRESS.md`
- Modify: `AGENTS.md`

- [ ] **Step 1: Document the new `--push` shortcut and its constraints**
- [ ] **Step 2: Run full verification**

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run --all-features
```

- [ ] **Step 3: Review diff to ensure this change does not spill into `ship`, browser automation, or WeChat push internals**
