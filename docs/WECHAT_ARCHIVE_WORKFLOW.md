# WeChat Official Account Archive Input Workflow

This document records a direction that is worth doing later, but should not be rushed into the main publishing pipeline right now:

**Safely archive already published WeChat Official Account articles into MoonPub / Obsidian, then continue organizing, rewriting, migrating, or republishing them through the unified Inbox model.**

Reference projects:

- [wechat-mp-batch-exporter](https://github.com/mcncarl/yichen-skills/tree/main/wechat-mp-batch-exporter)
- [moore-wechat-article-downloader](https://github.com/Moore-developers/moore-wechat-article-downloader)
- [mp-weixin-to-md](https://github.com/Noisepoint/mp-weixin-to-md)

MoonPub should learn from their workflow layering, output boundaries, and privacy principles, not copy external code.

## Why this is worth doing

MoonPub currently mainly solves:

- Entering drafts from Obsidian / Markdown / Feishu / photos
- Rendering WeChat-compatible HTML
- Moving content to WeChat drafts
- Assisting backend configuration

Many authors also have another real need:

- Back up articles they have already published
- Migrate old WeChat content back into Obsidian
- Extract topics, titles, summaries, and structures from historical articles
- Organize old content into collections, edit it again, and redistribute it to blogs

This line is not "publishing"; it is an "archive input source". It is closer to Feishu / photos as a peer input workflow.

## Suggested product positioning

In the short term, do not make it a black-box "bulk crawl WeChat history" tool.

A better positioning is:

**After the user explicitly provides article URLs or self-owned account authorization, MoonPub archives accessible public articles as Inbox Items.**

This means:

- By default, only process public `mp.weixin.qq.com` URLs provided by the user
- Do not bypass login, permissions, paid content, deleted content, verification, or platform risk controls
- Do not promise that reads, likes, comments, or other metrics are always available
- Do not write cookies, `pass_ticket`, `uin`, tokens, or QR-code login data into the repository
- Do not commit downloaded article content to the open-source repository by default

## Four-phase route

### Phase 1: Known URL archive

This is the best first scope.

Input:

- One or more WeChat Official Account article URLs explicitly provided by the user

Output:

- `Inbox/WechatArchive/*.md`
- Optional `raw.html`
- Optional `metadata.json`

Suggested Inbox frontmatter:

```yaml
---
source: wechat-mp-article
status: inbox
created: 2026-07-10
type: archived-article
external_id: "mp:<stable-url-or-hash>"
source_url: "https://mp.weixin.qq.com/s/..."
source_title: "Original article title"
source_author: "Official Account name"
captured_at: "2026-07-10T12:00:00+08:00"
---
```

The body should preserve as much as possible:

- Title
- Author / official account name
- Publish time, if available from the page
- Body text or Markdown
- Original URL

This phase may reuse the existing `fetch <url>` direction, but should not treat raw `fetch` output as a formal input source. The formal approach should be a new `intake wechat-url ...` or equivalent entry that aligns with the unified Inbox model.

The lesson from `moore-wechat-article-downloader` for this phase is: known URL download should remain a minimal, low-risk entry. It does not need WeChat backend login, does not need a proxy, and should not mix in history lists, comments, read counts, or browsing enhancement logic.

The lesson from `mp-weixin-to-md` is more concrete: the main line should be "article link -> standard Markdown"; local image download should be an explicit option; local HTML should only be a fallback input for validation pages or network failures; cookies should not be built in; login or verification pages should not be bypassed.

If MoonPub later implements `intake wechat-url`, keep these defaults:

- Save Markdown and source metadata by default.
- Keep remote image URLs by default; do not download images.
- Download WeChat image assets only with explicit `--download-assets`.
- Allow asset download only from common WeChat image domains, so arbitrary outbound links do not become a local crawler.
- Support `--from-html <file>` or an equivalent fallback entry, but label it as a manually provided HTML fallback, not an automatic way to bypass verification pages.

### Phase 2: Historical list indexing

This phase handles the case "I own an official account and want to list historical article URLs."

The boundary must be stricter:

- Only process accounts the user has permission to access
- Any QR scan, login, proxy, certificate trust, or WeChat Desktop operation must be confirmed by the user
- Agents should not operate the WeChat UI on behalf of the user
- Do not modify system proxy; if a local helper is ever needed, require explicit `--dry-run` / `--confirm`

Suggested outputs:

- `history.summary.json`
- `history.summary.md`
- `history.dedup.csv`
- `urls.all.txt`
- `urls.original.txt`

These files belong in a local working directory or private Obsidian directory, and should not be committed by default.

Historical lists should remain layered:

- Exporter / officially visible list: requires the user to scan or authorize, but does not touch system proxy.
- Subscription delta: only records local subscription state and new URLs; does not automatically download or republish.
- Proxy history: only a fallback when Exporter is unavailable, must be explicitly confirmed first, and can only list articles actually loaded.

MoonPub should design only the first two in the short term, and should not implement proxy history by default.

### Phase 3: Enhanced metric collection

Read counts, likes, wow, comments, and replies are higher-risk capabilities.

Consider this only when all conditions are met:

- The user confirms this is their own official account or they have legal permission
- Credentials are fresh and held locally by the user
- Sensitive credentials are not printed, stored, or committed
- Output clearly marks collection time and confidence

This should not be the main line for v0.4.x / v0.5.

If a future mode references "collect articles and comments while browsing", it must be separately labeled as browsing enhancement, not normal archive capability. It depends on the actual page load state, comments and metrics may be incomplete, and output must explicitly mark `missing` / `observed_at` rather than guessing.

### Phase 4: Browsing enhancement and proxy capability

This phase is recorded only; it is not part of MoonPub's near-term route.

Typical capabilities include:

- Modifying system proxy.
- Observing pages through the built-in browser in WeChat Desktop.
- Saving loaded comments, interaction data, and page snapshots.
- Injecting page buttons or resetting WebView processes.

These capabilities have much higher risk and maintenance cost than MoonPub's current main line. If they are ever built, they must satisfy:

- Disabled by default.
- Explain before each run that system proxy will be modified.
- Provide a mandatory cleanup step that restores proxy settings.
- Do not save cookies, auth keys, `pass_ticket`, or tokens in chats, logs, or the repository.
- Stay separate from the low-risk public URL archive entry such as `intake wechat-url`.

## Relationship to current MoonPub

This should not replace existing formal input workflows.

Current formal input workflows are still:

- Current article
- Feishu Minutes
- Photo material

The WeChat archive input source is better marked as:

- future workflow
- local archive workflow
- user-owned content workflow

If it later enters `workflow-registry`, it should first be marked `planned` or `experimental`, so it is not mistaken as already verified like Feishu / photos.

## Safety red lines

Implementation must obey:

- Do not commit archived article body, account data, cookies, QR-code secrets, `pass_ticket`, `uin`, or tokens
- Do not automatically operate WeChat Desktop
- Do not bypass login, paid content, deleted content, verification, or platform permissions
- Do not promise to fetch all historical articles
- Do not modify system proxy by default
- Do not enable proxy enhancement, WebView injection, or page snapshot collection by default
- Do not treat comments, read counts, likes, or wow counts as stable fields
- Do not build in cookies, and do not pretend success on captcha, shell, or verification pages
- Do not make image download the default; localizing images must be an explicit user choice
- Do not treat output from third-party exporters as republishable content
- Do not package "back up your own articles" as "copy other people's content"

## Suggested MoonPub landing point

The first step should not be a large feature, but a small closed loop:

```bash
moonpub intake wechat-url <url> --draft --preview --no-open
```

Suggested capabilities:

- Input a public article URL
- Fetch title, author, and body
- Write to `Inbox/WechatArchive/`
- Generate standard Markdown by default; Obsidian image syntax only as an optional format
- Keep remote image URLs by default, and download to local assets only when explicitly requested
- Reuse `draft-from-inbox`
- Stop at draft and local preview first
- Advance to WeChat draft only with explicit `--push`

Do not do yet:

- Bulk historical list crawling
- Comment / read count collection
- Proxy configuration
- Subscription delta sync
- Proxy history or browsing enhancement
- Automatic bypass of verification, shell, or risk-control pages
- WeChat Desktop automation
- Automatic republish of old articles

## Acceptance criteria

Before it becomes a formal input workflow, at least require:

- One Inbox artifact from a local public URL sample
- One Inbox artifact from a local HTML fallback sample
- `--json` output includes `command`, `action`, `inbox_path`, `draft_path`, `html_path`, `next_command`
- Images are not downloaded by default; when explicitly downloaded, only WeChat image domains are allowed, and local assets paths are written back to Markdown
- App-level regression tests at the same level as Feishu / photos
- Documentation clearly states copyright and permission boundaries
- Plugin homepage does not show high-risk bulk history capability by default
- Any historical list / subscription / proxy capability must not reuse the low-risk `intake wechat-url` entry name

## Current conclusion

This direction is worth doing, but should not become a "bulk WeChat Official Account exporter" too quickly.

The better first step is:

**Make a single public WeChat Official Account article URL into a safe MoonPub input source, archive it to Inbox, then reuse the existing draft and preview pipeline.**
