# MoonPub CLI

MoonPub CLI is the deterministic Rust core for the MoonPub publishing workflow.

Current scope:

- initialize a sample config;
- inspect the article pipeline state;
- check whether an article bundle has the expected files.
- store and list manually collected platform trend samples.

It does not call WeChat, generate AI content, upload cover images, export the blog, or distribute to other platforms yet.

## Commands

```bash
cargo run -- init
cargo run -- status
cargo run -- --vault /path/to/ObsidianMain status
cargo run -- --vault /path/to/ObsidianMain check Articles/published/<slug>.md
cargo run -- --vault /path/to/ObsidianMain radar add --platform xiaohongshu --keyword "AI写作" --title "一个热门标题" --likes 120 --collects 80
cargo run -- --vault /path/to/ObsidianMain radar list --platform xiaohongshu
```

Pass `--vault` when the current directory is not the Obsidian vault root.

## Article Bundle Contract

For a publishable draft:

```text
Articles/drafts/<slug>.md
Articles/drafts/<slug>.html
Articles/drafts/<slug>.draft.json
```

After WeChat draft creation:

```text
Articles/published/<slug>.media_id
```

`media_id` is useful for updating existing WeChat drafts but is not required for first-time publishability checks.

## Trend Radar Store

`radar add` stores samples in:

```text
.moonpub/trends.jsonl
```

Each record keeps:

- platform;
- keyword;
- title;
- url;
- author;
- likes;
- collects;
- comments;
- source.

This is intentionally manual-first. It is meant for user-provided samples, CSV imports later, or authorized data adapters. It does not scrape platforms.

## Next Commands

Planned:

```bash
moonpub radar import trends.csv
moonpub radar analyze <article.md> --platform wechat
moonpub render <article.md>
moonpub validate <article.md>
moonpub ready <article.md>
moonpub export-blog <article.md>
moonpub push-wechat <article.md>
moonpub cover <article.md>
moonpub distribute <article.md> --platforms zhihu,juejin,csdn
```
