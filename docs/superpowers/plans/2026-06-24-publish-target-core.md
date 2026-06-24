# Publish Target Core Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Introduce the internal `PublishTarget` abstraction and route the existing WeChat draft push through the first built-in target without changing CLI behavior.

**Architecture:** `src/plugin.rs` owns generic publish target metadata, context, outcome, and dispatch helpers. `src/push.rs` keeps the existing WeChat implementation details but exposes `WechatDraftTarget` as the first `PublishTarget`, so future targets can follow the same shape.

**Tech Stack:** Rust 2024, existing `Config`, `AppError`, `cargo nextest`.

---

### Task 1: Add PublishTarget Core

**Files:**
- Create: `src/plugin.rs`
- Modify: `src/lib.rs`

- [x] **Step 1: Write failing tests in `src/plugin.rs`**

```rust
#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use crate::config::Config;
    use crate::error::AppError;
    use crate::plugin::{PublishContext, PublishOutcome, PublishTarget, run_publish_target};

    struct FakeTarget;

    impl PublishTarget for FakeTarget {
        fn id(&self) -> &'static str {
            "fake"
        }

        fn display_name(&self) -> &'static str {
            "Fake"
        }

        fn requires_network(&self) -> bool {
            false
        }

        fn requires_browser(&self) -> bool {
            false
        }

        fn publish(&self, ctx: PublishContext<'_>) -> Result<PublishOutcome, AppError> {
            Ok(PublishOutcome {
                message: format!(
                    "{}:{}:{}",
                    ctx.articles_dir.display(),
                    ctx.article.display(),
                    ctx.auto_render
                ),
            })
        }
    }

    #[test]
    fn publish_target_exposes_capability_metadata() {
        let target = FakeTarget;

        assert_eq!(target.id(), "fake");
        assert_eq!(target.display_name(), "Fake");
        assert!(!target.requires_network());
        assert!(!target.requires_browser());
    }

    #[test]
    fn run_publish_target_dispatches_context() -> Result<(), Box<dyn std::error::Error>> {
        let cfg = Config::default();
        let target = FakeTarget;
        let outcome = run_publish_target(
            &target,
            PublishContext {
                articles_dir: Path::new("/vault"),
                article: Path::new("Articles/drafts/demo.md"),
                auto_render: true,
                config: &cfg,
            },
        )?;

        assert_eq!(
            outcome.message,
            "/vault:Articles/drafts/demo.md:true"
        );
        Ok(())
    }
}
```

- [x] **Step 2: Run focused test and verify RED**

Run: `cargo nextest run plugin::tests::`

Expected: compile failure because `crate::plugin` does not exist.

- [x] **Step 3: Implement minimal plugin module**

Add `PublishContext<'a>`, `PublishOutcome`, `PublishTarget`, and `run_publish_target`.

- [x] **Step 4: Run focused test and verify GREEN**

Run: `cargo nextest run plugin::tests::`

Expected: plugin tests pass.

### Task 2: Add WeChat Draft Target

**Files:**
- Modify: `src/push.rs`
- Modify: `src/app.rs` if necessary

- [x] **Step 1: Write failing target metadata test in `src/push.rs`**

```rust
#[test]
fn wechat_draft_target_reports_capabilities() {
    use crate::plugin::PublishTarget;

    let target = WechatDraftTarget;

    assert_eq!(target.id(), "wechat-draft");
    assert_eq!(target.display_name(), "WeChat Draft");
    assert!(target.requires_network());
    assert!(target.requires_browser());
}
```

- [x] **Step 2: Run focused test and verify RED**

Run: `cargo nextest run push::tests::wechat_draft_target_reports_capabilities`

Expected: compile failure because `WechatDraftTarget` is not defined.

- [x] **Step 3: Implement target wrapper**

Add `pub struct WechatDraftTarget;` and implement `PublishTarget` by delegating to the current push implementation.

- [x] **Step 4: Route `push_article` through target dispatch**

Keep `push_article(articles_dir, article, auto_render, cfg) -> Result<String, AppError>` as the public compatibility wrapper, but have it call `run_publish_target(&WechatDraftTarget, PublishContext { ... })`.

- [x] **Step 5: Run focused regression tests**

Run:

```bash
cargo nextest run plugin::tests:: push::tests::wechat_draft_target_reports_capabilities push::tests::push_fails_without_draft_json push::tests::push_auto_render_creates_draft_json
```

Expected: selected tests pass without real WeChat credentials.

### Task 3: Sync Docs And Verify

**Files:**
- Modify: `AGENTS.md`
- Modify: `PROGRESS.md`

- [x] **Step 1: Update architecture docs**

Record that `src/plugin.rs` owns target traits and `src/push.rs` implements the first built-in WeChat target.

- [x] **Step 2: Run verification**

Run:

```bash
git diff --check
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run --all-features
pre-commit run --all-files
```

Expected: all pass. If sandboxed `taplo` panics, rerun `pre-commit run --all-files` outside sandbox and record both facts in the PR.
