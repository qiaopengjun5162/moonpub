# MoonPub CLI Progress

## Status

Core publish pipeline complete. All commands implemented and tested.

## Completed

- `init` / `status` / `check` — 基础脚手架
- `render` — Markdown → WeChat HTML + draft.json（内联 CSS，微信合规）
- `push` — 调 md2wechat 推送草稿，写 .media_id，ready→published 自动移目录
- `export` — Zola 博客导出（YAML→TOML frontmatter，剥离微信 footer，替换 CDN 图片）
- `preview` — 用系统浏览器打开 .html 预览
- `radar add/list/import/analyze` — 热点样本管理与标题建议
- `radar scrape` — 抓取平台搜索页（playwright-cli 优先，fallback curl），自动入库
- `--json` / `--config` 全局 flag
- 43 unit tests，0 clippy warnings，零外部依赖

## Full Workflow

```bash
# 1. 生成 HTML + draft.json
moonpub --config moonpub.toml render Articles/drafts/demo.md

# 2. 预览效果（可选）
moonpub --vault ~/vault preview Articles/drafts/demo.md

# 3. 推送微信草稿，自动移到 published/
export WECHAT_SECRET="your_secret"
moonpub --config moonpub.toml push Articles/drafts/demo.md

# 4. 导出 Zola 博客
moonpub --config moonpub.toml export Articles/published/demo.md

# 5. 抓取平台热点（需 playwright-cli 或 curl）
moonpub --vault ~/vault radar scrape --platform wechat --keyword "AI写作" --count 10

# 6. 分析文章标题建议
moonpub --vault ~/vault radar analyze Articles/drafts/demo.md --platform wechat
```

## Not Implemented

- WeChat HTML 验证（validate_html.py 已有，未移植到 Rust）
- 封面图生成/上传
- 多平台分发适配器
- Obsidian 插件 / 桌面端
- 模板系统（当前样式硬编码）

## Verification

```bash
cargo fmt --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run   # 43 tests, 0 skipped
```


## Completed

- Rust CLI scaffold under `/Users/qiaopengjun/Code/Rust/moonpub`.
- Zero external dependencies (pure std).
- `init` command creates a sample `moonpub.toml`.
- `status` command lists `Articles/drafts`, `Articles/ready`, and `Articles/published`.
- `check` command inspects an article bundle and reports missing `md/html/draft.json/media_id` files.
- `radar add` command stores manual platform trend samples in `.moonpub/trends.jsonl`.
- `radar list` command lists stored trend samples with optional platform/keyword filters.
- **`--json` flag** — all commands wrap their output in `{"output":"..."}` when `--json` is passed.
- **`--config <moonpub.toml>`** — loads config file, overrides `--vault` with `[vault] root`.
  - `Config::from_toml()` is a minimal hand-rolled parser (no external deps).
- **`radar import <file.csv>`** — imports trend samples from a CSV file.
  - Supports quoted fields (RFC-4180 style), header name aliases (CN/EN), optional `--platform` default.
- **`radar analyze <article.md> --platform <name> [--top <n>]`** — scores radar samples by
  engagement (likes + collects×2 + comments×3) + keyword overlap with article text, outputs ranked title suggestions.
- **`render <article.md> [--author <name>] [--thumb <media_id>]`** — Markdown → WeChat HTML + draft.json
  - Parses YAML frontmatter (`title`, `digest`)
  - Renders `p / h2 / h3 / blockquote / hr / strong / em / code / img` with inline CSS matching published article style
  - Auto-strips old banner+CTA footer from Markdown if already present (避免重复)
  - `author` and `thumb_media_id` fall back to `[wechat]` section in `--config` file
  - draft.json schema matches `md2wechat create_draft` exactly
- **`push <article.md> [--render]`** — 调 `md2wechat create_draft`，写 `.media_id`，移目录
  - `WECHAT_APPID` / `WECHAT_SECRET` 从环境变量读（config `appid` 作兜底）
  - `--render` flag：draft.json 缺失时自动先执行 render
  - 失败时从错误信息里提取当前 IP，提示加白名单
  - 成功后把 `drafts/` 或 `ready/` 下的三件套自动移到 `published/`
- 32 unit tests; all passing.

## Not Implemented

- WeChat HTML validation adapter.
- WeChat draft push/update (HTTP API client).
- Zola blog export.
- Cover generation or upload.
- Multi-platform distribution adapter.
- Obsidian plugin or desktop app.

## Verification

Last verified (2026-06-10):

```bash
cargo fmt --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run
```

All 20 tests pass, zero clippy warnings.

## Next Step

- WeChat draft API: `moonpub push Articles/ready/demo.md --platform wechat`
- Zola export: `moonpub export Articles/published/demo.md --blog zola`
- JSON output for every command already works via `--json`; MCP adapter can consume it.
