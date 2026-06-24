# Capabilities Schema Version Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add stable top-level schema/version metadata to `moonpub capabilities --json` for plugin and app compatibility checks.

**Architecture:** Keep the existing `targets` array unchanged. Add `schema_version` and `moonpub_version` at the JSON root so callers can identify both the metadata contract and CLI binary version before invoking target commands.

**Tech Stack:** Rust CLI metadata in `src/plugin.rs`, existing manual JSON serialization, cargo nextest.

---

### Task 1: JSON Metadata

**Files:**
- Modify: `src/plugin.rs`

- [ ] Add a failing test that asserts `capabilities_json()` starts with `{"schema_version":"capabilities/v1","moonpub_version":"`.
- [ ] Add a failing test that asserts the JSON still contains the existing `targets` array and command templates.
- [ ] Define a `CAPABILITIES_SCHEMA_VERSION` constant.
- [ ] Serialize `schema_version` and `moonpub_version` before `targets`.
- [ ] Run `cargo nextest run --all-features plugin::tests`.

### Task 2: Docs And Progress

**Files:**
- Modify: `README.md`
- Modify: `README_zh.md`
- Modify: `docs/PLUGIN_ARCHITECTURE_ZH.md`
- Modify: `PROGRESS.md`

- [ ] Document that plugin/app callers can check `schema_version` and `moonpub_version`.
- [ ] Update progress notes with the new metadata contract and real test count after full verification.

### Task 3: Verification And PR

**Files:**
- No additional code files expected.

- [ ] Run `git diff --check`.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --all-targets --all-features --tests --benches -- -D warnings`.
- [ ] Run `cargo nextest run --all-features`.
- [ ] Run `cargo run --quiet -- capabilities --json`.
- [ ] Run `pre-commit run --all-files`.
- [ ] Commit as `feat: version capabilities schema`.
- [ ] Push branch and open a self-PR.
