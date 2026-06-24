# Export Target Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Introduce the internal `ExportTarget` abstraction and route existing Zola export through the first built-in export target.

**Architecture:** `src/plugin.rs` owns export target traits and dispatch helpers alongside publish targets. `src/export.rs` keeps the existing Zola file generation details but exposes `ZolaExportTarget`, while `export_article` remains the compatibility wrapper used by current CLI commands.

**Tech Stack:** Rust 2024, standard library filesystem APIs, existing `AppError`, `cargo nextest`.

---

### Task 1: Add ExportTarget Core

**Files:**
- Modify: `src/plugin.rs`

- [x] **Step 1: Write failing trait tests**

Add a fake export target test that expects `ExportContext`, `ExportOutcome`, `ExportTarget`, and `run_export_target` to dispatch context.

- [x] **Step 2: Verify RED**

Run: `cargo nextest run plugin::tests::export_target`

Expected: compile failure because export target interfaces do not exist.

- [x] **Step 3: Implement minimal export target interfaces**

Add `ExportContext<'a>`, `ExportOutcome`, `ExportTarget`, and `run_export_target`.

- [x] **Step 4: Verify GREEN**

Run: `cargo nextest run plugin::tests::export_target`

Expected: export target tests pass.

### Task 2: Add Zola Target And Capabilities

**Files:**
- Modify: `src/export.rs`
- Modify: `src/plugin.rs`

- [x] **Step 1: Write failing Zola target/capability tests**

Add tests expecting `ZolaExportTarget` metadata and `capabilities_json()` to include `zola` with no network/browser requirement.

- [x] **Step 2: Verify RED**

Run: `cargo nextest run export::tests::zola_export_target_reports_capabilities plugin::tests::capabilities_json_exposes_zola_export`

Expected: compile/test failure before implementation.

- [x] **Step 3: Implement Zola target wrapper**

Add `pub struct ZolaExportTarget;`, implement `ExportTarget`, move existing function body into an internal helper, and have `export_article` call `run_export_target`.

- [x] **Step 4: Update capabilities**

Add `zola` as `kind = "export"`, `requires_network = false`, `requires_browser = false`.

- [x] **Step 5: Verify GREEN**

Run:

```bash
cargo nextest run plugin::tests::export_target plugin::tests::capabilities export::tests::
```

Expected: selected tests pass.

### Task 3: Docs And Verification

**Files:**
- Modify: `AGENTS.md`
- Modify: `PROGRESS.md`

- [x] **Step 1: Update architecture docs**

Record that `src/plugin.rs` owns publish/export target traits and that Zola export is the first `ExportTarget`.

- [x] **Step 2: Run full verification**

Run:

```bash
git diff --check
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run --all-features
cargo run --quiet -- capabilities --json
pre-commit run --all-files
```

Expected: all pass. If sandboxed `taplo` panics, rerun `pre-commit run --all-files` outside sandbox and record both facts in the PR.
