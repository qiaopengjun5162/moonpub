# MoonPub Engineering Agent Guardrails

This document is the meta-rule set for coding agents such as Codex and Claude Code. It complements the project-specific boundaries in `AGENTS.md` with general collaboration rules for branches, commits, documentation, PRs, and production boundaries.

## Start Protocol

Before touching code, read:

1. `README.md` / `README_zh.md`
2. `AGENTS.md`
3. `docs/PRODUCT_WRAP_ZH.md`
4. `docs/EXECUTION_PLAN_ZH.md`
5. The README or documentation of the target module

If the change touches the WeChat API, browser automation, AI providers, file write-back, or a new input source, also read `docs/ENGINEERING_LESSONS.md` / `docs/ENGINEERING_LESSONS_ZH.md` and `docs/KNOWN_LIMITS.md` / `docs/KNOWN_LIMITS_ZH.md`.

## Branch Rules

- Do not develop directly on `main`.
- Each branch should focus on one module, one workflow, or one recorded plan item.
- Use Conventional Commits for commit messages:
  - `feat`: new feature
  - `fix`: bug fix
  - `docs`: documentation update
  - `refactor`: behavior-preserving refactor
  - `test`: test additions or fixes
  - `build`: build/dependency/toolchain changes
  - `ci`: CI/gate changes
  - `chore`: routine maintenance

## Documentation-First Rule

For new features, behavior changes, migrations, input source extensions, or production-adjacent changes, write or update the documentation first, then the code. Document:

- Target module and boundaries
- Verification commands
- Whether it touches the WeChat API / browser automation / AI provider / file write-back
- Rollback or downgrade path

Pure typos, mechanical formatting, or single-line bug fixes do not need extra design docs.

## Architecture Boundary Rules

Follow the module responsibilities defined in `AGENTS.md`:

- `src/main.rs`: load environment, parse arguments, emit results
- `src/cli.rs`: CLI parsing and command-level protocol declarations
- `src/app.rs`: command routing and use-case orchestration
- `src/app_*_commands.rs`: concrete command wrappers
- `src/ai_workflow.rs`: AI invocation and file write-back orchestration
- `src/push.rs` / `src/wechat.rs`: WeChat API
- `src/publish.rs` / `src/cdp.rs`: browser automation
- `src/intake.rs` / `src/intake/*.rs`: input sources
- `src/protocol.rs`: structured JSON payload convergence

Do not couple platform-specific logic (WeChat, Feishu, photos, AI) into `app.rs` or `app_support.rs`.

## Structured Output Rules

When adding machine-readable commands (`doctor`, `workspace`, `check`, `preflight`, `push`, etc.), prefer to keep JSON builders in `src/protocol.rs` and serialize `serde::Serialize` payloads instead of expanding hand-written JSON strings. Do not move JSON builders back into `src/app.rs`.

## Production Boundary Rules

If a change touches any of the following boundaries, the PR must explicitly state the risk points and rollback plan:

- WeChat API (`push` / `ship` / `configure` / `login`)
- Browser automation (CDP / headless Chrome / WeChat backend DOM manipulation)
- AI provider (`write` / `expand` / `polish` / `ship --ai` / `intake --analyze-images`)
- File write-back (Inbox / drafts / ready / published / `.env` / `moonpub.toml` / `.moonpub/`)
- Plugin/browser-side behavior (`obsidian-plugin/`)

Never commit real credentials (`WECHAT_SECRET`, `DEEPSEEK_API_KEY`, `.env`, `moonpub.toml`, sessions, cookies, `pass_ticket`, `uin`, tokens) to git.

## Commit Message Convention

This repository uses [Conventional Commits](https://www.conventionalcommits.org/) so Release Please can generate the changelog and version automatically.

Allowed types (matching `.release-please-config.json`):

- `feat`: new feature
- `fix`: bug fix
- `docs`: documentation update
- `refactor`: behavior-preserving refactor
- `test`: test additions or fixes
- `build`: build/dependency/toolchain changes
- `ci`: CI/gate changes
- `chore`: routine maintenance
- `perf`: performance improvement
- `revert`: revert a change
- `style`: pure formatting changes

Examples:

```text
feat: add wechat-health command
fix(push): retry cover upload on timeout
docs: update onboarding guide for cookie auth
```

### Local git hook

```bash
./scripts/install-git-hooks.sh
```

After installation, the `commit-msg` hook checks that the first line of every commit follows Conventional Commits and blocks the commit if it does not.

### CI check

`build.yml` runs `./scripts/check-commits.sh` on PRs, checking all non-merge commits in the `origin/main..HEAD` range.

## Verification Rules

At minimum, run these checks for code changes:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run --all-features
```

When modifying `obsidian-plugin/`, also run:

```bash
cd obsidian-plugin && npm ci && npm test && npm run build
```

For documentation changes, run locally:

```bash
# Check all Markdown files
npx markdownlint-cli2

# Check files changed on the current branch (also runs in CI)
./scripts/lint-docs.sh --changed
```

## Stop Conditions

Stop and confirm with the user/owner before proceeding when:

- Introducing a new programming language or build system
- Enabling real WeChat backend send or auto-publish
- Relaxing Feishu/WeChat/AI permissions or data boundaries
- Replacing or deleting `AGENTS.md`, core architecture documents, or release gates
- Deleting historical archives, rollback materials, or evidence files

## PR Rules

- Use the repository PR template (`.github/PULL_REQUEST_TEMPLATE.md`).
- State the change type, verification commands, documentation updates, production boundaries, and credential checks.
- Treat PR-Agent comments as suggestions; they do not replace CI, CODEOWNERS, or owner review.

## Related Files

- `.pr_agent.toml`: PR-Agent Chinese review instructions
- `.markdownlint-cli2.yaml`: Markdown lint rules
- `.github/PULL_REQUEST_TEMPLATE.md`: PR template
- `.github/CODEOWNERS`: code owner rules
- `AGENTS.md`: project-specific architecture and command boundaries

## Release Process

This repository uses Release Please to manage release PRs and draft releases:

1. After a feature/fix PR is merged into `main`, the Release Please action maintains a release PR automatically.
2. The release PR contains a `CHANGELOG.md`, version bumps (updating `Cargo.toml`, `Cargo.lock`, and `.release-please-manifest.json`) based on Conventional Commits.
3. The owner reviews and merges the release PR.
4. Release Please creates a draft GitHub Release.
5. The owner manually publishes the draft release, which creates a tag.
6. The existing `.github/workflows/release.yml` listens for tag pushes (`v*`), builds multi-platform binaries and the Obsidian plugin package, and uploads them to the release assets.

Do not manually create tags or edit the top release entries in `CHANGELOG.md`; Release Please maintains those. Engineering notes in `docs/ENGINEERING_LESSONS.md` / `docs/ENGINEERING_LESSONS_ZH.md` are not subject to this restriction.
