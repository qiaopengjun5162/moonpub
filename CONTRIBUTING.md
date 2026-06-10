# Contributing

Thanks for helping improve `MoonPub`.

## Development

```bash
cargo fmt
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run
```

Use `cargo nextest`, not `cargo test`, for project test runs.

## Code Style

- Keep business logic in Rust. Zero external dependencies (only `ureq` for HTTP/TLS).
- `src/lib.rs` — CLI core, Block template system, Markdown renderer
- `src/wechat.rs` — WeChat API client (access_token, draft/add, draft/update, upload_image)
- `src/humanize.rs` — Rule-based Chinese de-AI-ification (6 phases)
- `src/main.rs` — Entry point
- Write source comments in English for code logic, Chinese for domain-specific notes.
- Do not commit secrets or `moonpub.toml` with real credentials.

## Block Template System

When adding a new block type:

1. Add the block name to `render_fence_block()` match in `src/lib.rs`
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
- Update `PROGRESS.md` and `docs/` when architecture or workflow changes.
- Add or update tests for behavior changes.
- Run `cargo clippy --all-targets --all-features --tests --benches -- -D warnings` and `cargo nextest run` before opening a PR.
- Use Conventional Commits: `feat:`, `fix:`, `docs:`, `refactor:`, `test:`, `chore:`.
- Commit messages in English.

## Self-PR Workflow

```bash
git switch -c codex/your-change
cargo fmt
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run
git add .
git commit -m "feat: describe your change"
git push origin codex/your-change
gh pr create --base main --head codex/your-change
```

After CI passes, review the diff on GitHub, merge the PR, and delete the branch.

## References

See [docs/REFERENCES.md](docs/REFERENCES.md) for the full list of reference projects, articles, and tools.
