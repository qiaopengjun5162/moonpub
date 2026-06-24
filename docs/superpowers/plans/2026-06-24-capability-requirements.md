# Capability Requirements Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose static target prerequisites in `moonpub capabilities --json` so plugins and apps can warn users before invoking publish/export commands.

**Architecture:** Extend each built-in `TargetCapability` with `required_env` and `required_config` arrays. These are descriptive metadata only; command behavior and credential loading remain unchanged.

**Tech Stack:** Rust CLI metadata in `src/plugin.rs`, existing manual JSON serialization, cargo nextest.

---

### Task 1: Capability Metadata

**Files:**
- Modify: `src/plugin.rs`

- [ ] Add failing tests that assert `wechat-draft` exposes `required_env:["WECHAT_APPID","WECHAT_SECRET"]`.
- [ ] Add failing tests that assert `zola` exposes `required_config:["blog.root"]`.
- [ ] Add `required_env` and `required_config` fields to `TargetCapability`.
- [ ] Serialize both arrays in `capabilities_json()`.
- [ ] Add text output lines for human-readable capabilities.
- [ ] Run `cargo nextest run --all-features plugin::tests app::tests::capabilities_outputs_text`.

### Task 2: Docs And Progress

**Files:**
- Modify: `README.md`
- Modify: `README_zh.md`
- Modify: `docs/PLUGIN_ARCHITECTURE_ZH.md`
- Modify: `PROGRESS.md`

- [ ] Document that capabilities include prerequisite env/config metadata.
- [ ] Update progress notes with the new metadata and real test count after full verification.

### Task 3: Verification And PR

**Files:**
- No additional code files expected.

- [ ] Run `git diff --check`.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --all-targets --all-features --tests --benches -- -D warnings`.
- [ ] Run `cargo nextest run --all-features`.
- [ ] Run `cargo run --quiet -- capabilities --json`.
- [ ] Run `pre-commit run --all-files`.
- [ ] Commit as `feat: expose capability requirements`.
- [ ] Push branch and open a self-PR.
