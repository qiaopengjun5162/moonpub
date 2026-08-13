---
name: moonpub-wechat-publish
description:
  This skill should be used when publishing a WeChat Official Account article through
  the moonpub Rust CLI — turning a source link (X/Twitter, web article, or pasted text)
  into a drafted, covered, rendered, pushed, and phone-previewed WeChat post. It also
  covers evaluating external GitHub repos for good practices worth porting into moonpub
  (e.g. validation checks, CI gates). Trigger on requests like "把这条链接转成公众号草稿",
  "推送并手机预览", "发到公众号", or "看看这个仓库能不能借鉴".
agent_created: true
---

# MoonPub → WeChat 公众号发布

Two reusable workflows bundled here:

1. **Publish from a source link** — full moonpub pipeline: draft → cover → render → push → phone preview → user publishes.
2. **Evaluate & port external repo practices** — research a GitHub repo, keep only genuinely transferable generic bits, port them into moonpub correctly.

Detailed command catalog and exact error codes live in `references/command-reference.md`. Read it before running commands.

**Execution layer (MCP):** the same commands are exposed as 21 callable MCP tools by `mcp/server.py` (fastmcp over stdio). Wire it into Claude Desktop / Cursor per `mcp/README.md` — each tool maps 1:1 to a command below, all calls auto-append `--json`. This skill is the *knowledge* layer; the MCP server is the *execution* layer. Prefer the MCP tools when an agent/client can call them directly; fall back to the bash commands here otherwise.

---

## Part A — Publish from a source link

### Step 1: Capture the source
Use `WebFetch` on the link to extract text/author/date. If the user pastes text directly, skip this.

### Step 2: Confirm intent
Do **not** assume "publish to WeChat". Ask (one `AskUserQuestion`):
- 转成公众号草稿 (draft, stop at local editable + HTML preview, no push)
- 收进 Inbox 素材 (material only)
- 仅整理成笔记
- 就看看内容

For the common "链接 → 草稿 → 推送 → 手机预览 → 群发" path, proceed through Steps 3–7.

### Step 3: Write the draft
Check for AI keys first: `echo "${DEEPSEEK_API_KEY:+yes}${OPENAI_API_KEY:+yes}"`. If empty, `moonpub write` (AI generation) cannot run — write the Markdown draft manually with moonpub frontmatter:

```markdown
---
title: <标题，≤64 字符>
theme: <见下方主题选择>
digest: <摘要，≤120 字符；留空微信自动提取>
tags: [工具, 效率]
date: 2026-08-12
author: <作者>
cover: <slug>.cover.png
---

正文用 intro / :::card / :::summary 等块组织。
```

Theme choices: future/tech/anime feel → `geek-black` (dark + neon green) or `cyber`; bright/clean → `blueprint`; minimal → `minimal`; warm/editorial → `newsletter` / `magazine`. Avoid `clean` (orange accent) if the user dislikes yellow.

Standard footer (固定结尾模板) is configured globally via `[footer]` in `moonpub.toml` (auto-read from the `--articles` root). If missing, copy it from `moonpub-data/moonpub.toml` (e.g. the 寻月阁 template + `qrcode.png`) — do not hand-inject footer into body.

### Step 4: Render + preflight
```bash
moonpub --articles . render <slug>.md
moonpub --articles . preflight <slug>.md
```
Use **relative** paths (relative to `--articles`). Preflight validates WeChat hard limits (title/digest length, H1↔title match) and image integrity (broken refs → fail, orphan images → warn).

### Step 5: Cover
```bash
moonpub --articles . cover <slug>.md --style geek-black --screenshot
```
`--screenshot` is **required** to emit a PNG (without it only HTML is produced). Chrome is found at `/Applications/Google Chrome.app`.

**CRITICAL:** the `cover` command does **not** write `cover:` back to frontmatter. After generating, ensure `cover: <slug>.cover.png` is in the frontmatter, or `push` fails with `40007 invalid media_id`.

### Step 6: Push
```bash
moonpub --articles . push <slug>.md --render
```
Needs `WECHAT_APPID` / `WECHAT_SECRET` (or `[wechat].auth_method = "cookie"` reusing a `moonpub login` session).

Known failures:
- `40164 invalid ip` — machine egress IP not in mp.weixin.qq.com IP whitelist. Fix egress (disable proxy so IP is stable), then add the **exact** `current IP` WeChat reports to 公众号后台 → 开发 → 基本配置 → IP白名单. (Proxy rotation makes a single added IP useless — stabilize egress first.)
- `40007 invalid media_id` — cover not linked (see Step 5).

After a successful push the article moves `drafts/` → `ready/`. **All subsequent commands must use `ready/<slug>.md`**, not the old `drafts/` path.

### Step 7: Phone preview (optional, before user publishes)
```bash
moonpub --articles . test-yulan --title "<标题>"
```
Sends to the wxid in `.moonpub/preview_to`. Requires a **persistent** Chrome session from `moonpub login` (headed QR scan on the Mac). After scanning, **close the automation Chrome window** or you get "persistent Chrome profile is already in use". `--temporary-profile` does NOT reuse the saved session and will demand a rescan (unusable headless). Note: `push --render` already auto-sends the preview (ret=0); `test-yulan` reads whatever draft is already in WeChat, so it can show a *stale* version if push failed — treat `push` media_id return, not `test-yulan` success, as the source of truth.

The user publishes manually from 公众号后台 → 草稿箱. Nothing auto-sends publicly.

---

## Part B — Evaluate & port external repo practices

### Step 1: Map the repo
```bash
gh api repos/<owner>/<repo>/git/trees/<branch>?recursive=1   # full tree
gh api repos/<owner>/<repo>/contents/<path> --jq '.content' | base64 -d   # read file
```
Read README + engineering/CI docs first.

### Step 2: Classify portability
Decide if the repo shares moonpub's domain (a Rust publishing CLI). If it is divergent infra (agent orchestration, content-workflow monorepos), **most code is not portable** — only look for *generic* transferable practices: validation checks, CI gates, hygiene rules.

### Step 3: Port only generic bits, correctly adapted
- wx-cli `check-public-draft-style.mjs` WeChat hard limits → added as `preflight` checks (title/digest length by **char count**, not UTF-8 bytes; use `wechat_title()` for the real sent title).
- qintopia-agent-os secret-scan gate → added `secret-scan` job (`gitleaks/gitleaks-action@v2`) to `build.yml`.
Do **not** copy coupling-specific code (manifest/local_id structures, server deploy scripts).

### Step 4: Verify before commit
```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run --all-features
```
If a tool can't be installed locally (this environment kills network egress), substitute a **targeted static grep** for secret patterns instead of claiming unverified success. Then sync `AGENTS.md` / `docs/ENGINEERING_AGENT_GUARDAILS_ZH.md`.

---

## Key gotchas (read before every run)
- **Path mode:** `moonpub --articles <dir> <cmd> <file.md>` — `<file.md>` is relative to `--articles`. Absolute paths cause "path duplication" errors.
- **Post-push relocation:** article moves `drafts/` → `ready/`; update the path in every later command.
- **Cover must be linked in frontmatter** or push returns `40007`.
- **IP whitelist must be the exact IP WeChat reports**, after stabilizing egress (proxy off).
- **`test-yulan` ≠ push success** — it previews the existing WeChat draft, possibly stale.
- **`present_files` is not a valid tool** — show rendered HTML via the `Read` tool on the `.html` file, not a dedicated present command.
