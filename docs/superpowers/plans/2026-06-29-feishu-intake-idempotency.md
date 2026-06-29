# Feishu Intake Idempotency Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make Feishu Minutes intake and draft generation safe to rerun by reusing the same Inbox and draft files for the same source material.

**Architecture:** Keep the change narrowly scoped to the Feishu intake and draft-generation path. `src/intake.rs` will detect existing Inbox files by `minute_token` and update them in place, `src/ai_workflow.rs` will reuse an existing draft path instead of erroring, and `src/app.rs` will surface `created` / `updated` in both text and structured `--json` output.

**Tech Stack:** Rust std, existing serde_json parsing, existing `test_helpers`, cargo-nextest

---

### Task 1: Add failing tests for idempotent intake and draft reuse

**Files:**
- Modify: `src/intake.rs`
- Modify: `src/ai_workflow.rs`
- Modify: `src/app.rs`
- Test: `src/intake.rs`
- Test: `src/ai_workflow.rs`
- Test: `src/app.rs`

- [ ] **Step 1: Add a failing intake rerun test in `src/intake.rs`**

Add a test that imports Feishu material with a fixed `minute_token`, writes the same token a second time, and asserts the second run updates the same Inbox path instead of creating a new one.

- [ ] **Step 2: Run the focused intake test and verify it fails for the right reason**

Run:

```bash
cargo nextest run --all-features intake::tests::intake_feishu_minutes_with_same_token_updates_existing_inbox
```

Expected: FAIL because current code does not search for an existing Inbox file by `minute_token`.

- [ ] **Step 3: Add a failing draft reuse test in `src/ai_workflow.rs`**

Add a test that creates an existing draft file for a known title, reruns the draft write path, and asserts the content should be replaced with an `updated` result instead of returning `AlreadyExists`.

- [ ] **Step 4: Run the focused draft test and verify it fails for the right reason**

Run:

```bash
cargo nextest run --all-features ai_workflow::tests::writing_existing_draft_reuses_same_path
```

Expected: FAIL because current draft writing still rejects existing files.

- [ ] **Step 5: Add failing JSON builder tests in `src/app.rs`**

Add builder-level assertions that the structured `draft-from-inbox` and `intake-feishu` JSON outputs include an `action` field.

- [ ] **Step 6: Run the focused app tests and verify they fail**

Run:

```bash
cargo nextest run --all-features app::tests::draft_from_inbox_json_builder_includes_action app::tests::intake_draft_preview_json_builder_includes_action
```

Expected: FAIL because current JSON builders do not include `action`.

### Task 2: Implement idempotent Inbox file reuse by `minute_token`

**Files:**
- Modify: `src/intake.rs`
- Test: `src/intake.rs`

- [ ] **Step 1: Introduce an explicit intake action type**

Add a small enum in `src/intake.rs` to represent whether intake created or updated a file:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IntakeAction {
    Created,
    Updated,
}
```

Extend `IntakeOutput` with:

```rust
pub action: IntakeAction,
```

- [ ] **Step 2: Add a helper that finds an existing Inbox file by `minute_token`**

Implement a focused helper that scans `Inbox/Feishu/*.md`, reads frontmatter text, and returns the first file whose `minute_token: "<token>"` matches exactly.

- [ ] **Step 3: Reuse the existing Inbox path when a token match is found**

Update `write_feishu_minutes(...)` so that:

- token present + existing file found => overwrite existing path, return `IntakeAction::Updated`
- otherwise => create a new file path, return `IntakeAction::Created`

Keep local-file intake without token on the existing create-only behavior.

- [ ] **Step 4: Update the human-readable message to match the action**

Generate:

```rust
let verb = match action {
    IntakeAction::Created => "intake created",
    IntakeAction::Updated => "intake updated",
};
```

- [ ] **Step 5: Run the focused intake tests and verify they pass**

Run:

```bash
cargo nextest run --all-features intake::tests::intake_feishu_writes_raw_minutes_to_obsidian_inbox intake::tests::intake_feishu_minutes_with_same_token_updates_existing_inbox
```

Expected: PASS.

### Task 3: Implement draft file reuse and action reporting

**Files:**
- Modify: `src/draft.rs`
- Modify: `src/ai_workflow.rs`
- Test: `src/draft.rs`
- Test: `src/ai_workflow.rs`

- [ ] **Step 1: Introduce a shared draft write result in `src/draft.rs`**

Add:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DraftWriteAction {
    Created,
    Updated,
}

pub struct DraftWriteOutput {
    pub path: PathBuf,
    pub action: DraftWriteAction,
}
```

- [ ] **Step 2: Replace create-only draft writing with an overwrite-capable helper**

Keep `new_article(...)` behavior unchanged, but add a new helper that writes draft content to the normal draft path and returns `Created` or `Updated` depending on whether the file already existed.

- [ ] **Step 3: Update `draft_from_inbox(...)` to use the new helper**

Extend `DraftOutput` with:

```rust
pub action: crate::draft::DraftWriteAction,
```

and build the message from the action:

```rust
"draft created"
"draft updated"
```

- [ ] **Step 4: Run the focused draft tests and verify they pass**

Run:

```bash
cargo nextest run --all-features draft::tests::write_or_update_article_file_updates_existing_draft ai_workflow::tests::writing_existing_draft_reuses_same_path ai_workflow::tests::draft_from_inbox_message_includes_next_push_command
```

Expected: PASS.

### Task 4: Surface `action` through app JSON and text output

**Files:**
- Modify: `src/app.rs`
- Test: `src/app.rs`

- [ ] **Step 1: Add `action` to the JSON builders**

Update:

- `draft_from_inbox_json(...)`
- `intake_draft_preview_json(...)`

to include:

```rust
"action":"created"
```

or:

```rust
"action":"updated"
```

- [ ] **Step 2: Thread `action` through the command execution path**

Use `output.action` / `draft_output.action` when building structured JSON for:

- `Command::DraftFromInbox`
- `Command::IntakeFeishu { draft: true, .. }`

Plain-text output should already reflect the action through the updated messages from lower layers.

- [ ] **Step 3: Run the focused app tests and verify they pass**

Run:

```bash
cargo nextest run --all-features app::tests::draft_from_inbox_json_builder_includes_action app::tests::intake_draft_preview_json_builder_includes_action
```

Expected: PASS.

### Task 5: Sync docs and run full verification

**Files:**
- Modify: `README.md`
- Modify: `README_zh.md`
- Modify: `docs/USER_GUIDE.md`
- Modify: `PROGRESS.md`
- Modify: `AGENTS.md`

- [ ] **Step 1: Update docs for rerunnable Feishu intake**

Document that Feishu Minutes imports with `minute_token` now safely reuse the same Inbox and draft files, and that this applies to the official Feishu path only.

- [ ] **Step 2: Run full project verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run --all-features
```

Expected: PASS.

- [ ] **Step 3: Review diff for scope discipline**

Confirm the change stays within:

- `src/intake.rs`
- `src/draft.rs`
- `src/ai_workflow.rs`
- `src/app.rs`
- related docs/tests

and does not alter WeChat push or browser automation behavior.
