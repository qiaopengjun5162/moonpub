# Capability Command Templates Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add machine-readable CLI invocation templates to `moonpub capabilities --json` so Obsidian plugins and local apps can invoke targets without hard-coding command shapes.

**Architecture:** Extend `TargetCapability` with an argv-style `command` template and an `article_arg` placeholder. Keep text output human-focused, and keep existing target execution behavior unchanged.

**Tech Stack:** Rust CLI metadata in `src/plugin.rs`, existing JSON string escaping helper, cargo nextest.

---

### Task 1: Metadata Shape

**Files:**
- Modify: `src/plugin.rs`

- [ ] Add failing tests that assert `capabilities_json()` includes `command` arrays for `wechat-draft` and `zola`.
- [ ] Add failing tests that assert the command template uses `"{article}"` instead of a real path.
- [ ] Add `command: &'static [&'static str]` and `article_arg: &'static str` to `TargetCapability`.
- [ ] Populate built-in target metadata:
  - `wechat-draft`: `["publish", "{article}", "--target", "wechat-draft"]`
  - `zola`: `["export", "{article}", "--target", "zola"]`
- [ ] Serialize `command` and `article_arg` in `capabilities_json()`.
- [ ] Run `cargo nextest run --all-features plugin::tests`.

### Task 2: Docs And Progress

**Files:**
- Modify: `README.md`
- Modify: `README_zh.md`
- Modify: `docs/PLUGIN_ARCHITECTURE_ZH.md`
- Modify: `PROGRESS.md`

- [ ] Document that `capabilities --json` includes argv-style command templates for app/plugin callers.
- [ ] Update progress notes with the new metadata and test count after full verification.

### Task 3: Verification And PR

**Files:**
- No additional code files expected.

- [ ] Run `git diff --check`.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --all-targets --all-features --tests --benches -- -D warnings`.
- [ ] Run `cargo nextest run --all-features`.
- [ ] Run `cargo run --quiet -- capabilities --json`.
- [ ] Run `pre-commit run --all-files`; if taplo fails in sandbox, rerun with escalation.
- [ ] Commit as `feat: expose capability command templates`.
- [ ] Push branch and open a self-PR.
