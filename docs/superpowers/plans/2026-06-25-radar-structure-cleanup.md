# Radar Structure Cleanup Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Split the remaining CSV import and title suggestion implementation out of `src/radar.rs` so Radar keeps the same behavior while the root module becomes a thin command entrypoint.

**Architecture:** Keep `src/radar.rs` as the public Radar boundary with `RadarCommand`, `run_radar`, and re-exports. Move CSV import helpers into `src/radar/import.rs`, move title suggestion helpers into `src/radar/suggest.rs`, keep the current test surface stable, and only update `PROGRESS.md` for this refactor.

**Tech Stack:** Rust 2024 edition, existing `radar` submodules, `cargo fmt`, `cargo clippy`, `cargo nextest`.

---

### Task 1: Extract CSV Import Module

**Files:**
- Create: `src/radar/import.rs`
- Modify: `src/radar.rs`
- Test: `src/radar.rs`

- [ ] **Step 1: Create a failing compile state by wiring a new module in `src/radar.rs`**

Add a new module declaration and import re-export in `src/radar.rs`, then remove the inline `import_csv` / `parse_csv_row` bodies without changing `run_radar` call sites.

```rust
mod import;

pub use import::import_csv;
#[cfg(test)]
pub(crate) use import::parse_csv_row;
```

Expected intermediate state: `cargo check` would fail until `src/radar/import.rs` exists and defines those functions.

- [ ] **Step 2: Move CSV constants and implementation into `src/radar/import.rs`**

Create `src/radar/import.rs` with the current CSV header constants plus the existing `import_csv` and `parse_csv_row` logic.

```rust
use std::fs;
use std::path::Path;

use crate::error::AppError;

use super::{TrendSample, add_trend_sample};

const COL_PLATFORM: &[&str] = &["platform", "平台"];
const COL_KEYWORD: &[&str] = &["keyword", "关键词", "keywords"];
const COL_TITLE: &[&str] = &["title", "标题"];
const COL_URL: &[&str] = &["url", "链接"];
const COL_AUTHOR: &[&str] = &["author", "作者"];
const COL_LIKES: &[&str] = &["likes", "点赞", "like_count"];
const COL_COLLECTS: &[&str] = &["collects", "收藏", "collect_count", "favorites"];
const COL_COMMENTS: &[&str] = &["comments", "评论", "comment_count"];
const COL_SOURCE: &[&str] = &["source", "来源"];

pub fn import_csv(
    articles_dir: &Path,
    csv_path: &Path,
    default_platform: Option<&str>,
) -> Result<String, AppError> {
    // move existing body unchanged
}

pub(crate) fn parse_csv_row(line: &str) -> Vec<String> {
    // move existing body unchanged
}
```

- [ ] **Step 3: Run focused Radar tests for CSV behavior**

Run:

```bash
cargo nextest run --all-features radar:: --status-level pass
```

Expected: existing Radar tests covering CSV import and CSV row parsing still pass.

- [ ] **Step 4: Keep test imports stable in `src/radar.rs`**

Adjust the `#[cfg(test)]` re-export block in `src/radar.rs` so existing tests can continue importing `parse_csv_row` from `crate::radar`.

```rust
#[cfg(test)]
pub(crate) use import::parse_csv_row;
```

- [ ] **Step 5: Commit the import extraction**

```bash
git add src/radar.rs src/radar/import.rs
git commit -m "refactor: extract radar csv import module"
```

### Task 2: Extract Title Suggestion Module

**Files:**
- Create: `src/radar/suggest.rs`
- Modify: `src/radar.rs`
- Test: `src/radar.rs`

- [ ] **Step 1: Create the new suggest module boundary**

Wire a new module in `src/radar.rs` and re-export `suggest_titles`.

```rust
mod suggest;

pub use suggest::suggest_titles;
```

Expected intermediate state: `cargo check` would fail until `src/radar/suggest.rs` is created.

- [ ] **Step 2: Move `suggest_titles` and its local helpers into `src/radar/suggest.rs`**

Create `src/radar/suggest.rs` and move the suggestion flow plus helper constants/functions that only serve title suggestion.

```rust
use std::fs;
use std::path::Path;

use crate::{
    article::{parse_frontmatter, resolve_article_path, strip_frontmatter},
    error::AppError,
};

use super::{TrendSample, load_all_samples, tokenize, trend_store_path};

pub fn suggest_titles(
    articles_dir: &Path,
    article: &Path,
    platform: &str,
    top: usize,
) -> Result<String, AppError> {
    // move existing body unchanged
}

fn push_trend_ref(output: &mut String, trend: Option<&TrendSample>) {
    // move existing body unchanged
}

pub(crate) fn short_phrase(s: &str, max_chars: usize) -> String {
    // move existing body unchanged
}

pub(crate) fn extract_pain_point(body: &str) -> Option<&str> {
    // move existing body unchanged
}

pub(crate) fn first_paragraph_hook(body: &str) -> Option<&str> {
    // move existing body unchanged
}
```

- [ ] **Step 3: Preserve test access for helper functions**

If current tests in `src/radar.rs` import helper functions from `crate::radar`, keep matching `#[cfg(test)]` re-exports in `src/radar.rs`.

```rust
#[cfg(test)]
pub(crate) use suggest::{
    extract_pain_point,
    first_paragraph_hook,
    short_phrase,
};
```

- [ ] **Step 4: Run focused Radar tests again**

Run:

```bash
cargo nextest run --all-features radar:: --status-level pass
```

Expected: title suggestion and helper behavior stays green with no assertion text changes.

- [ ] **Step 5: Commit the suggestion extraction**

```bash
git add src/radar.rs src/radar/suggest.rs
git commit -m "refactor: extract radar suggestion module"
```

### Task 3: Thin `src/radar.rs` Down To Entry Layer

**Files:**
- Modify: `src/radar.rs`
- Test: `src/radar.rs`

- [ ] **Step 1: Remove now-duplicated inline implementations from `src/radar.rs`**

After the module extractions, `src/radar.rs` should keep:

- `use` statements needed for `run_radar`
- `mod analyze; mod cli; mod import; mod scrape; mod store; mod suggest;`
- `pub use` and `pub(crate) use` lines
- `RadarCommand`
- `run_radar`
- existing tests

It should no longer keep the inline bodies for:

- `import_csv`
- `parse_csv_row`
- `suggest_titles`
- helper functions moved to `suggest.rs`

- [ ] **Step 2: Run formatting and lint checks**

Run:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
```

Expected: both commands pass without introducing new warnings.

- [ ] **Step 3: Run full project verification**

Run:

```bash
cargo nextest run --all-features
```

Expected: full suite passes, not just Radar-focused tests.

- [ ] **Step 4: Commit the root module cleanup**

```bash
git add src/radar.rs
git commit -m "refactor: thin radar root module"
```

### Task 4: Sync Project Progress Record

**Files:**
- Modify: `PROGRESS.md`

- [ ] **Step 1: Add a dated progress entry for this cleanup**

Append a version log entry that records:

- `radar import` moved into `src/radar/import.rs`
- `radar suggest` moved into `src/radar/suggest.rs`
- the exact verification commands that actually passed

Use the existing style in `PROGRESS.md`, for example:

```markdown
- 2026-06-25: **Radar import/suggest 拆分** — `import_csv` / `parse_csv_row` 移入 `src/radar/import.rs`，`suggest_titles` 与其辅助逻辑移入 `src/radar/suggest.rs`；`cargo nextest run --all-features radar::` passed
```

- [ ] **Step 2: Verify only directly related docs changed**

Run:

```bash
git diff --stat
```

Expected: only `src/radar.rs`, new Radar submodule files, and `PROGRESS.md` are in scope.

- [ ] **Step 3: Commit the progress sync**

```bash
git add PROGRESS.md
git commit -m "docs: record radar structure cleanup"
```

### Task 5: Final Review And Delivery

**Files:**
- No new source files expected beyond prior tasks.

- [ ] **Step 1: Run whitespace and patch hygiene check**

Run:

```bash
git diff --check
```

Expected: no whitespace or merge-marker issues.

- [ ] **Step 2: Review commit stack and final status**

Run:

```bash
git status --short
git log --oneline -5
```

Expected: working tree clean except for anything intentionally left unstaged; recent commits clearly reflect import extraction, suggestion extraction, root thinning, and docs sync.

- [ ] **Step 3: Push for review**

```bash
git push origin codex/release-windows-zip-smoke
```

Expected: branch updates successfully with the Radar cleanup commits.
