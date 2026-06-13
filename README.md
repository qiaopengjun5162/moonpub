# MoonPub

[![All Contributors](https://img.shields.io/badge/all_contributors-1-orange.svg?style=flat-square)](#contributors-)
![Rust Version](https://img.shields.io/badge/rust-%3E%3D1.85-blue)
![License](https://img.shields.io/badge/license-MIT-green)

Pure Rust CLI: Markdown → WeChat Official Account, fully automated.

MoonPub is built around stable boundaries:

- `render`: Markdown → WeChat HTML + draft.json (Block template system + de-AI)
- `push`: Native WeChat API client (zero md2wechat dependency, direct draft/add)
- `export`: Zola blog export (YAML → TOML frontmatter)
- `radar`: Platform trend sample management + title suggestions

## Why Not md2wechat

Most WeChat publishing tools rely on paid APIs or third-party CLIs. MoonPub is built from scratch:

- WeChat API client (Rust + ureq), zero external tool dependencies
- 40+ layout templates → built-in Block template system, free and customizable
- All transformations are fully offline (no network except WeChat API calls)

## Quick Start

```bash
cargo install --git https://github.com/qiaopengjun5162/moonpub
moonpub init                    # Create moonpub.toml
moonpub status                  # View article pipeline
moonpub render article.md       # Generate HTML + draft.json
moonpub push article.md         # Push draft to WeChat
moonpub export article.md       # Export to Zola blog
```

Pushing requires WeChat credentials:

```bash
export WECHAT_APPID=wx***
export WECHAT_SECRET=your_secret
```

## CDP Browser Automation

Post-render draft configuration can be fully automated via Chrome DevTools Protocol:

```bash
moonpub login              # First time: scan QR code (headed browser), saves cookies
moonpub configure          # Headless: auto-configure draft settings, no browser window
moonpub configure --headed # Debug mode: visible browser + screenshots
moonpub test-zanshang --headed  # Debug a single step in isolation
```

Steps automated: 原创声明 (originality), 赞赏 (tips), 留言 (comments), 创作来源 (source), 预览 (preview).

## Configuration

```bash
moonpub init    # Create default moonpub.toml
```

```toml
[vault]
root = "/path/to/ObsidianMain"

[wechat]
appid = "wx..."
author = "Your Name"
account_type = "personal"    # personal | verified | service | wecom
auto_publish = false          # Set true for verified accounts (API direct publish)
theme = "default"             # default | warm | dark
thumb_media_id = ""           # Default cover image media_id

[blog]
kind = "zola"
root = "/path/to/blog"
```

## Block Template System

Use `:::blockname` syntax in Markdown:

```markdown
:::book-info
title: Book Title
author: Author Name
cover: https://...
publisher: Publisher
rating: 8.1
:::

:::intro
A 1-3 sentence hook to grab the reader
:::

:::callout
label: Key Takeaway
The one thing you want the reader to remember
:::

:::steps
1. Step one
2. Step two
3. Step three
:::

:::summary
Closing summary
:::
```

12 blocks supported: `book-info` / `intro` / `callout` / `steps` / `summary` / `figure` / `checklist` / `cover` / `quote-card` / `divider` / `concept-card` / `emotion-card`

## De-AI (humanize)

```bash
moonpub humanize article.md              # Run standalone
moonpub render --humanize article.md     # Combined with render
```

6-stage rule pipeline: filler phrases → AI vocabulary → parallelism breaking → modifier simplification → generic conclusions → em-dash cleanup

## All Commands

```bash
moonpub init [path]            # Create config
moonpub status                 # Article pipeline status
moonpub check <article.md>     # Check article bundle integrity
moonpub render <article.md>    # Markdown → HTML + draft.json
moonpub preview <article.md>   # Open in browser
moonpub push <article.md>      # Push to WeChat drafts (auto-uploads local images)
moonpub update-draft <article.md>  # Update existing draft
moonpub export <article.md>    # Export to Zola blog
moonpub humanize <article.md>  # Strip AI patterns
moonpub cover <article.md> [--style dark|clean|minimal|warm|serif|gradient] [--screenshot]
moonpub ship <article.md> [--style ...]  # One-shot: cover + render + push + export
moonpub mark-ready <article.md>    # Mark preview confirmed
moonpub mark-published <article.md>  # Mark as published

moonpub radar add --platform <name> --keyword <kw> --title <title>
moonpub radar list [--platform <name>]
moonpub radar import <file.csv>
moonpub radar analyze <article.md> --platform <name>
moonpub radar suggest <article.md> --platform <name>
moonpub radar scrape --platform <name> --keyword <kw>
```

Global flags: `--vault <path>` / `--config <moonpub.toml>` / `--json`

## Development

```bash
cargo fmt
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run
```

Use `cargo nextest`, not `cargo test`.

## Architecture

- Business logic in pure Rust, minimal dependencies (`ureq` for HTTP)
- Block template system: `// ── Block renderers` section in `src/lib.rs`
- WeChat API client: `src/wechat.rs`
- De-AI: `src/humanize.rs`
- CDP automation: `src/publish.rs`
- All styles inline CSS, WeChat-compatible

## Contributing

PR-first workflow. Create a `codex/<short-topic>` branch, keep changes focused, run `cargo clippy` and `cargo nextest`, push, and open a PR against `main`. See [CONTRIBUTING.md](CONTRIBUTING.md).

## License

MIT

## Contributors

<!-- ALL-CONTRIBUTORS-LIST:START - Do not remove or modify this section -->
<!-- prettier-ignore-start -->
<!-- markdownlint-disable -->
<table>
  <tbody>
    <tr>
      <td align="center" valign="top" width="14.28%"><a href="https://github.com/qiaopengjun5162"><img src="https://avatars.githubusercontent.com/u/124650229?v=4?s=100" width="100px;" alt="Paxon Qiao"/><br /><sub><b>Paxon Qiao</b></sub></a><br /><a href="https://github.com/qiaopengjun5162/moonpub/commits?author=qiaopengjun5162" title="Code">💻</a> <a href="#doc-qiaopengjun5162" title="Documentation">📖</a> <a href="#ideas-qiaopengjun5162" title="Ideas">🤔</a> <a href="#projectManagement-qiaopengjun5162" title="Project Management">📆</a></td>
    </tr>
  </tbody>
</table>
<!-- markdownlint-restore -->
<!-- prettier-ignore-end -->
<!-- ALL-CONTRIBUTORS-LIST:END -->
