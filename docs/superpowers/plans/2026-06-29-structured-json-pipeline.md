# Structured JSON Pipeline Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Make the Feishu Minutes to WeChat draft pipeline machine-readable when `--json` is used, without changing default human-readable output.

**Architecture:** Keep existing plain-text command output unchanged by default. Add small structured result builders for the four pipeline commands in `app.rs`, using a shared JSON helper module so `--json` returns stable fields instead of wrapping plain text in `{"output":"..."}`. Limit scope to `draft-from-inbox`, `intake feishu ... --draft`, `preview`, and `push`.

**Tech Stack:** Rust std, existing zero-dependency JSON string escaping helpers, cargo-nextest

---

### Task 1: Add failing tests for command-specific JSON output

**Files:**
- Modify: `src/app.rs`
- Modify: `src/lib.rs`
- Test: `src/app.rs`

- [ ] **Step 1: Write failing JSON behavior tests in `src/app.rs`**

Add tests that call `run()` with `json: true` and assert command-specific fields exist:

```rust
    #[test]
    fn preview_json_includes_paths_and_next_command() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("preview-json")?;
        let md = root.join("Articles/drafts/demo.md");
        let html = root.join("Articles/drafts/demo.html");
        create_file(&md, "---\ntitle: Demo\n---\n正文\n")?;
        create_file(&html, "<p>正文</p>")?;

        let output = run(&Options {
            articles: root.clone(),
            command: Command::Preview {
                article: std::path::PathBuf::from("Articles/drafts/demo.md"),
                open: false,
            },
            json: true,
            config: None,
        })?;

        assert!(output.contains(r#""command":"preview""#), "{output}");
        assert!(output.contains(r#""article_path":"#), "{output}");
        assert!(output.contains(r#""html_path":"#), "{output}");
        assert!(output.contains(r#""next_command":"moonpub push "#), "{output}");

        std::fs::remove_dir_all(root)?;
        Ok(())
    }
```

Also add focused tests for:
- `draft-from-inbox` JSON includes `draft_path` and `next_command`
- `intake feishu --draft --preview --no-open` JSON includes `inbox_path`, `draft_path`, `html_path`
- `push` JSON still includes `media_id` and `stage`

- [ ] **Step 2: Run targeted tests to verify they fail**

Run:

```bash
cargo nextest run --all-features app::tests::preview_json_includes_paths_and_next_command
```

Expected: FAIL because current `--json` output is only `{"output":"..."}`.

- [ ] **Step 3: Keep the generic wrapping test only for fallback commands**

Update `src/lib.rs` so the current `json_output_wraps_text` test still checks legacy fallback behavior on `status`, not the new structured pipeline commands.

- [ ] **Step 4: Run fallback JSON test**

Run:

```bash
cargo nextest run --all-features tests::json_output_wraps_text
```

Expected: PASS or still green after the new failing command-specific tests are added.

### Task 2: Add shared structured JSON helpers

**Files:**
- Modify: `src/json_util.rs`
- Modify: `src/app.rs`
- Test: `src/json_util.rs`

- [ ] **Step 1: Add field helper functions in `src/json_util.rs`**

Add small zero-dependency helpers for composing JSON objects:

```rust
pub fn json_string_field(name: &str, value: &str) -> String {
    format!("\"{name}\":\"{}\"", escape_json(value))
}

pub fn json_bool_field(name: &str, value: bool) -> String {
    format!("\"{name}\":{value}")
}

pub fn json_optional_string_field(name: &str, value: Option<&str>) -> String {
    value.map_or_else(
        || format!("\"{name}\":null"),
        |value| json_string_field(name, value),
    )
}

pub fn json_object(fields: &[String]) -> String {
    format!("{{{}}}", fields.join(","))
}
```

- [ ] **Step 2: Add helper tests in `src/json_util.rs`**

Add tests for escaping and null handling:

```rust
#[test]
fn json_optional_string_field_emits_null() {
    assert_eq!(json_optional_string_field("draft_path", None), "\"draft_path\":null");
}
```

- [ ] **Step 3: Run targeted helper tests**

Run:

```bash
cargo nextest run --all-features json_util::
```

Expected: PASS after helper implementation.

### Task 3: Return structured JSON for the four pipeline commands

**Files:**
- Modify: `src/app.rs`
- Modify: `src/preview.rs`
- Modify: `src/ai_workflow.rs`
- Modify: `src/intake.rs`
- Modify: `src/push.rs`
- Test: `src/app.rs`

- [ ] **Step 1: Add small command-specific JSON builders in `src/app.rs`**

Add focused builders instead of one large abstraction:

```rust
fn preview_json(article_path: &std::path::Path, html_path: &std::path::Path) -> String
fn draft_json(command: &str, draft_path: &std::path::Path) -> String
fn intake_draft_preview_json(
    inbox_path: &std::path::Path,
    draft_path: &std::path::Path,
    html_path: Option<&std::path::Path>,
) -> String
fn push_json(article_path: &std::path::Path, media_id: Option<&str>, stage: &str, output: &str) -> String
```

Keep them private to `app.rs`.

- [ ] **Step 2: Expose machine-usable paths from `preview.rs`**

Add a small helper that resolves article/html paths without changing the public plain-text API:

```rust
pub fn preview_paths(
    articles_dir: &Path,
    article: &Path,
) -> Result<(PathBuf, PathBuf), AppError>
```

Use this helper inside both `preview_article_with_open` and the JSON builder path.

- [ ] **Step 3: Reuse existing workflow structs instead of reparsing text**

Keep `DraftOutput` and `IntakeOutput`, and add the minimal extra data needed:

```rust
pub struct DraftOutput {
    pub path: PathBuf,
    pub message: String,
}

pub struct IntakeOutput {
    pub path: PathBuf,
    pub message: String,
}
```

Do not introduce a new generic pipeline trait. Compose JSON in `app.rs` using these concrete structs plus resolved preview paths.

- [ ] **Step 4: Add a push result struct in `src/push.rs`**

Introduce a compatibility wrapper:

```rust
pub struct PushOutput {
    pub media_id: String,
    pub stage: &'static str,
    pub message: String,
}
```

Keep `push_article(...) -> Result<String, AppError>` for compatibility, but route it through:

```rust
pub fn push_article_output(...) -> Result<PushOutput, AppError>
```

Set `stage` to `"ready"` for the current draft push path.

- [ ] **Step 5: Switch `app.rs` JSON handling from generic wrapper to command-specific builders**

Replace:

```rust
if options.json && !matches!(options.command, Command::Capabilities) {
    Ok(to_json_string(&raw))
}
```

with command-specific handling for only:
- `Command::DraftFromInbox`
- `Command::IntakeFeishu { draft: true, .. }`
- `Command::Preview`
- `Command::Push`

All other commands should still fall back to:

```rust
Ok(to_json_string(&raw))
```

- [ ] **Step 6: Run targeted tests for the new structured commands**

Run:

```bash
cargo nextest run --all-features app::tests::preview_json_includes_paths_and_next_command
```

Expected: PASS.

Then run the other new app JSON tests added in Step 1.

### Task 4: Document the new JSON contract and run full verification

**Files:**
- Modify: `README.md`
- Modify: `README_zh.md`
- Modify: `docs/USER_GUIDE.md`
- Modify: `PROGRESS.md`

- [ ] **Step 1: Update CLI docs for structured `--json` output**

Document that these commands now return structured JSON when `--json` is supplied:

```text
moonpub --json draft-from-inbox Inbox/Feishu/demo.md --preview --no-open
moonpub --json intake feishu --latest --draft --preview --no-open
moonpub --json preview Articles/drafts/demo.md --no-open
moonpub --json push Articles/drafts/demo.md --render
```

Mention representative fields: `draft_path`, `html_path`, `media_id`, `next_command`.

- [ ] **Step 2: Record the feature in `PROGRESS.md`**

Add a 2026-06-29 entry noting that the Feishu-to-WeChat pipeline now has machine-readable `--json` outputs for the four key commands.

- [ ] **Step 3: Run full project verification**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run --all-features
```

Expected: all pass.

- [ ] **Step 4: Commit**

```bash
git add src/app.rs src/preview.rs src/push.rs src/json_util.rs src/ai_workflow.rs src/intake.rs README.md README_zh.md docs/USER_GUIDE.md PROGRESS.md
git commit -m "feat: add structured json pipeline output"
```
