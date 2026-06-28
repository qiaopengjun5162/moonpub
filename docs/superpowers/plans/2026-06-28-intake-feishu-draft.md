# Intake Feishu Draft Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add `--draft` to `moonpub intake feishu` so imported Feishu Minutes material can immediately produce an editable article draft.

**Architecture:** Keep intake responsible for writing source material into `Inbox/Feishu`, but return a structured output containing the created Inbox path. Let `app.rs` decide whether `--draft` should call the existing `draft_from_inbox` AI workflow after intake succeeds.

**Tech Stack:** Rust CLI, existing handwritten parser, existing AI provider configuration.

---

### Task 1: CLI And Intake Output

**Files:**
- Modify: `src/cli.rs`
- Modify: `src/intake.rs`
- Modify: `src/app.rs`
- Modify: `src/error.rs`
- Modify: `README.md`
- Modify: `README_zh.md`
- Modify: `docs/USER_GUIDE.md`
- Modify: `PROGRESS.md`
- Modify: `AGENTS.md`

- [ ] Add parser tests for `intake feishu <source> --draft`, including file, `--latest`, and `--query`.
- [ ] Run the parser tests and verify they fail because `--draft` is not yet supported.
- [ ] Add a `draft: bool` field to `Command::IntakeFeishu`.
- [ ] Change Feishu intake functions to return a structured output with the created Inbox path plus display message.
- [ ] In `app.rs`, when `draft` is true, load AI config and call `draft_from_inbox` using the structured Inbox path.
- [ ] Update help text and docs to mention `--draft`.
- [ ] Run focused tests, then full format, clippy, and nextest.
