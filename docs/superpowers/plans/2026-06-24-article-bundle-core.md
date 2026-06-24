# Article Bundle Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Move article bundle path, stage, report, and stage-transition logic into `src/bundle.rs` without changing CLI behavior.

**Architecture:** `src/bundle.rs` becomes the single owner of `ArticleBundle`, `ArticleStage`, bundle reporting, and bundle movement between `drafts`, `ready`, and `published`. `status.rs`, `push.rs`, and `app.rs` call that module instead of carrying their own bundle logic.

**Tech Stack:** Rust 2024, standard library filesystem APIs, existing `AppError`, `cargo nextest`.

---

### Task 1: Add Bundle Module With Tests

**Files:**
- Create: `src/bundle.rs`
- Modify: `src/lib.rs`

- [x] **Step 1: Write failing tests in `src/bundle.rs`**

```rust
#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::bundle::{ArticleBundle, ArticleStage, move_article_bundle};
    use crate::test_helpers::{create_file, temp_root};

    #[test]
    fn bundle_report_marks_missing_parts() -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("bundle-report")?;
        let article = root.join("Articles/drafts/demo.md");
        create_file(&article, "# demo")?;
        create_file(&root.join("Articles/drafts/demo.html"), "<p>demo</p>")?;

        let bundle = ArticleBundle::from_markdown(&article)?;
        let output = bundle.report();

        assert!(output.contains("markdown: ok"));
        assert!(output.contains("html: ok"));
        assert!(output.contains("draft_json: missing"));
        assert!(output.contains("publishable: no"));

        std::fs::remove_dir_all(root)?;
        Ok(())
    }

    #[test]
    fn bundle_stage_detects_article_stages() {
        assert_eq!(
            ArticleStage::from_dir(Path::new("/vault/Articles/drafts")),
            Some(ArticleStage::Drafts)
        );
        assert_eq!(
            ArticleStage::from_dir(Path::new("/vault/Articles/ready")),
            Some(ArticleStage::Ready)
        );
        assert_eq!(
            ArticleStage::from_dir(Path::new("/vault/Articles/published")),
            Some(ArticleStage::Published)
        );
        assert_eq!(ArticleStage::from_dir(Path::new("/vault/Articles")), None);
    }

    #[test]
    fn move_article_bundle_moves_known_files_to_target_stage()
    -> Result<(), Box<dyn std::error::Error>> {
        let root = temp_root("bundle-move")?;
        let drafts = root.join("Articles/drafts");
        create_file(&drafts.join("demo.md"), "# demo")?;
        create_file(&drafts.join("demo.html"), "<p>demo</p>")?;
        create_file(&drafts.join("demo.draft.json"), "{}")?;
        create_file(&drafts.join("demo.media_id"), "media_id")?;

        let target =
            move_article_bundle(&drafts, "demo", ArticleStage::Ready)?.expect("moved");

        assert_eq!(target, root.join("Articles/ready"));
        assert!(root.join("Articles/ready/demo.md").exists());
        assert!(root.join("Articles/ready/demo.html").exists());
        assert!(root.join("Articles/ready/demo.draft.json").exists());
        assert!(root.join("Articles/ready/demo.media_id").exists());
        assert!(!root.join("Articles/drafts/demo.md").exists());

        std::fs::remove_dir_all(root)?;
        Ok(())
    }
}
```

- [x] **Step 2: Run focused test and verify RED**

Run: `cargo nextest run bundle::tests::`

Expected: compile failure because `crate::bundle` does not exist.

- [x] **Step 3: Add minimal `src/bundle.rs` implementation and `pub mod bundle;`**

Implement `ArticleStage::from_dir`, `ArticleStage::as_str`, `ArticleBundle::from_markdown`, `ArticleBundle::report`, and `move_article_bundle`.

- [x] **Step 4: Run focused test and verify GREEN**

Run: `cargo nextest run bundle::tests::`

Expected: all bundle tests pass.

### Task 2: Replace Existing Call Sites

**Files:**
- Modify: `src/status.rs`
- Modify: `src/push.rs`
- Modify: `src/app.rs`

- [x] **Step 1: Remove local bundle logic from `status.rs`**

Use `ArticleBundle::from_markdown(&article)?.report()` in `check_article`.

- [x] **Step 2: Update movement call sites**

Use `crate::bundle::move_article_bundle` and `ArticleStage::{Ready, Published}` in `push.rs` and `app.rs`.

- [x] **Step 3: Keep compatibility helper only if existing tests need it**

If tests still import `dir_stage`, keep a thin wrapper in `status.rs` that delegates to `ArticleStage::from_dir(dir).map(ArticleStage::as_str)`.

- [x] **Step 4: Run focused regression tests**

Run:

```bash
cargo nextest run status::tests:: push::tests::pushed_bundle_moves_to_ready_not_published app::tests::mark_published_moves_ready_bundle_to_published
```

Expected: all selected tests pass.

### Task 3: Sync Docs And Full Verification

**Files:**
- Modify: `PROGRESS.md`
- Modify: `AGENTS.md` if a durable architecture boundary changed.

- [x] **Step 1: Update progress**

Add a dated progress entry that `ArticleBundle` moved into `src/bundle.rs` and existing behavior remains compatible.

- [x] **Step 2: Run verification**

Run:

```bash
git diff --check
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run --all-features
pre-commit run --all-files
```

Expected: all pass. If `taplo` panics in the sandbox, rerun the taplo/pre-commit command outside the sandbox and record both facts in the PR.
