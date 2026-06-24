# Capabilities Command Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a `moonpub capabilities` command that reports built-in target capabilities for Obsidian plugins and future local apps.

**Architecture:** `src/plugin.rs` owns capability data and JSON/text rendering. `src/app.rs` exposes the command without loading user credentials. `src/cli.rs` parses both `moonpub --json capabilities` and `moonpub capabilities --json` so plugin callers have a stable machine-readable path.

**Tech Stack:** Rust 2024, existing hand-written JSON helpers, `cargo nextest`.

---

### Task 1: Capability Model

**Files:**
- Modify: `src/plugin.rs`

- [x] **Step 1: Write failing tests**

Add tests that expect `capabilities_json()` to include `wechat-draft`, `requires_network`, `requires_browser`, and `manual final confirmation`.

- [x] **Step 2: Verify RED**

Run: `cargo nextest run plugin::tests::capabilities`

Expected: compile failure because capability functions do not exist.

- [x] **Step 3: Implement model and renderers**

Add `TargetCapability`, `builtin_capabilities()`, `capabilities_text()`, and `capabilities_json()`.

- [x] **Step 4: Verify GREEN**

Run: `cargo nextest run plugin::tests::capabilities`

Expected: capability tests pass.

### Task 2: CLI Command

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/app.rs`
- Modify: `src/error.rs`

- [x] **Step 1: Write failing parser/app tests**

Add tests for `Options::parse(["capabilities"])`, `Options::parse(["capabilities", "--json"])`, and `run` output.

- [x] **Step 2: Verify RED**

Run: `cargo nextest run cli::tests::parses_capabilities app::tests::capabilities`

Expected: parser/app tests fail before command implementation.

- [x] **Step 3: Implement command**

Add `Command::Capabilities`, parse command-local `--json`, route in `app.rs`, and add help text.

- [x] **Step 4: Verify GREEN**

Run: `cargo nextest run cli::tests::parses_capabilities app::tests::capabilities`

Expected: parser/app tests pass.

### Task 3: Docs And Verification

**Files:**
- Modify: `README.md`
- Modify: `README_zh.md`
- Modify: `AGENTS.md`
- Modify: `PROGRESS.md`

- [x] **Step 1: Update docs**

Document `moonpub capabilities --json` as the stable plugin/App discovery command.

- [x] **Step 2: Run full verification**

Run:

```bash
git diff --check
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run --all-features
pre-commit run --all-files
```

Expected: all pass. If sandboxed `taplo` panics, rerun `pre-commit run --all-files` outside sandbox and record both facts in the PR.
