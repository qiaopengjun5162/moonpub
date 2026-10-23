# Contributing

Thanks for helping improve `MoonPub`.

## Development

Read [docs/ENGINEERING_AGENT_GUARDAILS_ZH.md](docs/ENGINEERING_AGENT_GUARDAILS_ZH.md) first. It defines the collaboration rules: start protocol, branch rules, documentation-first, architecture boundaries, production boundaries, and stop conditions.

```bash
# Install the local git hook for Conventional Commits
./scripts/install-git-hooks.sh

# Run checks
./scripts/lint-docs.sh
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run --all-features
```

Use `cargo nextest`, not `cargo test`, for project test runs. If you change `obsidian-plugin/`, also run `cd obsidian-plugin && npm ci && npm test && npm run build`.

## Code Style

- Keep business logic in Rust. Do not introduce new language/toolchain stacks without an owner-approved decision.
- Architecture boundaries (see `AGENTS.md`):
  - CLI parsing: `src/cli.rs`
  - Command routing: `src/app.rs`
  - Local article commands: `src/app_article_commands.rs`
  - WeChat publish commands: `src/app_publish_commands.rs`
  - Draft follow-up: `src/app_draft_follow_up.rs`
  - AI workflow: `src/ai_workflow.rs`
  - WeChat API: `src/wechat.rs` / `src/push.rs`
  - CDP automation: `src/cdp.rs` / `src/publish.rs` / `src/publish_steps.rs`
  - Markdown rendering: `src/markdown.rs` / `src/markdown/*.rs`
  - Cover: `src/cover.rs`
  - Bundle / stage management: `src/bundle.rs`
  - Intake: `src/intake.rs` / `src/intake/*.rs`
- Write source comments in English for code logic, Chinese for domain-specific notes.
- Do not commit secrets, real credentials, or personal `.moonpub/` state.

## Block Template System

When adding a new block type:

1. Add the block name to `render_fence_block()` match in `src/illustrate.rs`
2. Implement the render function (e.g., `render_book_info()`)
3. All CSS must be inline — no `<style>`, `<script>`, or `class` attributes
4. Use `<table>` layouts for complex blocks (WeChat compatible)
5. Update `PROGRESS.md` with the new block

## JSONL Serialization

Trend samples and status tracking use hand-rolled JSONL (no `serde`):

- `escape_json()` — string escaping
- `extract_json_string()` / `extract_json_number()` — parsing
- `to_json_line()` / `from_json_line()` — round-trip (TrendSample)

When modifying these, ensure round-trip tests pass for all edge cases (quotes, newlines, Unicode).

## Pull Requests

- Create a `codex/<short-topic>` branch for each focused change.
- Keep changes small enough to review in one pull request.
- Use the PR template in `.github/PULL_REQUEST_TEMPLATE.md` and fill every section: change type, description, modules, validation commands, documentation sync, production boundary, and credentials check.
- Update `PROGRESS.md` and `docs/` when architecture or workflow changes.
- Add or update tests for behavior changes.
- Before investigating a recurring failure, search [docs/ENGINEERING_LESSONS_ZH.md](docs/ENGINEERING_LESSONS_ZH.md). When a fix changes a user path, privacy boundary, release path, or non-obvious invariant, record its root cause, prevention, and verification evidence there.
- Run all validation commands before opening a PR.
- Use Conventional Commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`, `ci:`, `build:`, `perf:`, `revert:`.
- Commit messages in English or Chinese; the PR body should be in Chinese unless the change is documentation-only in English.

## Self-PR Workflow

```bash
git switch -c codex/your-change
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run --all-features
git add .
git commit -m "feat: describe your change"
git push origin codex/your-change
gh pr create --base main --head codex/your-change
```

After CI passes, review the diff on GitHub, merge the PR, and delete the branch.

## References

See [docs/REFERENCES.md](docs/REFERENCES.md) for the full list of reference projects, articles, and tools.
