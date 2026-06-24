# Generic Target Commands Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Expose the existing publish/export target core through user-facing `publish --target` and `export --target` CLI commands.

**Architecture:** Keep existing `push` and `export <article.md>` behavior compatible. Add a generic `publish` command that dispatches the built-in `wechat-draft` target, and extend `export` with an optional target that defaults to `zola`.

**Tech Stack:** Rust CLI, existing `Options::parse`, `app::run`, built-in `PublishTarget` / `ExportTarget`, cargo nextest.

---

### Task 1: CLI Parsing

**Files:**
- Modify: `src/cli.rs`

- [ ] Add `Command::Publish { article: PathBuf, target: String, auto_render: bool }`.
- [ ] Extend `Command::Export` with `target: Option<String>`.
- [ ] Add parser tests for `publish Articles/ready/demo.md --target wechat-draft --render`.
- [ ] Add parser tests for `export Articles/published/demo.md --target zola`.
- [ ] Keep `push <article.md> [--render]` and `export <article.md>` parsing compatible.
- [ ] Run focused CLI tests with `cargo nextest run --all-features cli::tests`.

### Task 2: App Routing

**Files:**
- Modify: `src/app.rs`

- [ ] Route `Command::Publish` with target `wechat-draft` to `push_article`.
- [ ] Route `Command::Export` with no target or `zola` to `export_article`.
- [ ] Return a clear `UnknownCommand` error for unknown publish/export targets.
- [ ] Add app tests for unknown target errors that avoid real network/API calls.
- [ ] Run focused app tests with `cargo nextest run --all-features app::tests`.

### Task 3: Docs And Help

**Files:**
- Modify: `src/error.rs`
- Modify: `README.md`
- Modify: `README_zh.md`
- Modify: `PROGRESS.md`

- [ ] Add `publish <article.md> --target wechat-draft [--render]` to CLI help and README command tables.
- [ ] Update `export` help and README command tables to document `--target zola`.
- [ ] Update progress notes with the new command bridge and real test count after verification.

### Task 4: Verification And PR

**Files:**
- No code files expected beyond Tasks 1-3.

- [ ] Run `git diff --check`.
- [ ] Run `cargo fmt --all -- --check`.
- [ ] Run `cargo clippy --all-targets --all-features --tests --benches -- -D warnings`.
- [ ] Run `cargo nextest run --all-features`.
- [ ] Run `cargo run --quiet -- capabilities --json`.
- [ ] Run `pre-commit run --all-files`; if taplo fails due macOS sandbox system-configuration, rerun with escalation.
- [ ] Commit as `feat: add generic target commands`.
- [ ] Push branch and open a self-PR.
