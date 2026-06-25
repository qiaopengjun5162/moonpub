# Windows Smoke CI Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a Windows CI smoke test that verifies `moonpub.exe` can run the no-credential local first-run path on pull requests and pushes to `main`.

**Architecture:** Keep the existing Ubuntu lint/test job unchanged. Add a separate `windows-smoke` job on `windows-latest` that builds the binary and runs `--version`, `--help`, `init`, `new`, `render`, and `check` in a temporary smoke directory.

**Tech Stack:** GitHub Actions, PowerShell, Rust stable toolchain, existing CLI commands.

---

### Task 1: CI Job

**Files:**
- Modify: `.github/workflows/build.yml`

- [ ] Add a `windows-smoke` job using `windows-latest`.
- [ ] Install stable Rust with `dtolnay/rust-toolchain@stable`.
- [ ] Cache Rust build outputs with `Swatinem/rust-cache@v2`.
- [ ] Run `cargo build --release`.
- [ ] Use PowerShell to create a temporary smoke directory.
- [ ] Run:
  - `moonpub.exe --version`
  - `moonpub.exe --help`
  - `moonpub.exe init moonpub.toml`
  - `moonpub.exe new "Windows Smoke"`
  - `moonpub.exe render Articles/drafts/windows-smoke.md`
  - `moonpub.exe check Articles/drafts/windows-smoke.md`

### Task 2: Docs And Progress

**Files:**
- Modify: `PROGRESS.md`
- Modify: `docs/LAUNCH_READY_ZH.md`
- Modify: `docs/RELEASE_NOTES_v0.4.1.md`

- [ ] Record that Windows release assets exist and PR CI now smoke-tests a source-built Windows binary.
- [ ] Keep the distinction clear: this does not yet prove the published Windows release asset has been manually downloaded and smoke-tested.

### Task 3: Verification And PR

**Files:**
- No additional code files expected.

- [ ] Run `git diff --check`.
- [ ] Run `pre-commit run --all-files`.
- [ ] Commit as `ci: add windows smoke test`.
- [ ] Push branch and open a self-PR.
- [ ] Wait for GitHub CI, especially the new `windows-smoke` job.
