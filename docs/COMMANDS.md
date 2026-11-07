# MoonPub Command Reference

Flag-level reference for every `moonpub` command. The compact one-line-per-command
table lives in the [README](../README.md); per-command pages are available in the
CLI itself:

```bash
moonpub help             # grouped command list
moonpub help <command>   # one command's usage and notes
moonpub --json help      # machine-readable command registry (help/v1)
```

The command set below mirrors `src/help.rs` (the single source of truth). A test
keeps this file, both READMEs and the registry from drifting apart.

## Global options (before the command)

```
--articles <path>    Workspace root that holds Articles/ (default: current dir)
--config <path>      Config file (default: <root>/moonpub.toml)
--json               Machine-readable output
```

## Getting started

```
moonpub init [moonpub.toml] [--yes]   Create a sample moonpub.toml
moonpub doctor                        Check local readiness for first use
moonpub workspace                     Summarize the workspace and suggest the next command
moonpub status                        List articles in drafts, ready and published
moonpub help [<command>]              Show the command list, or one command's usage
moonpub version                       Print the moonpub version
```

`init` runs an interactive wizard on a terminal; `--yes` (or a non-interactive
stdin) writes the default sample config without prompting. It never overwrites an
existing `moonpub.toml`.

## Writing

```
moonpub new <title>                   Scaffold an article with frontmatter and body template
moonpub write <idea>                  Generate an article from an idea (AI)
moonpub expand <article.md>           Expand reading notes into a full article (AI)
moonpub polish <article.md>           AI polish and de-AI-ify an existing article
moonpub humanize <article.md>         Strip AI writing patterns offline
moonpub draft-from-inbox <inbox.md> [--preview] [--no-open] [--push]
                                      Turn Inbox source material into an editable draft
```

`write` / `expand` / `polish` / `draft-from-inbox` need an AI API key
(`DEEPSEEK_API_KEY` or `OPENAI_API_KEY`). `humanize` is fully offline.
For `draft-from-inbox`, `--preview` is the recommended conservative path;
`--push` is the explicit fast-forward into WeChat draft push.

## Render & review

```
moonpub render <article.md> [--author <name>] [--thumb <media_id>]
                                      Generate <slug>.html and <slug>.draft.json
moonpub check <article.md>            Check that a bundle has md / html / draft.json
moonpub preflight <article.md> [--strict]
                                      Read-only publish gate: bundle, layout audit, next action
moonpub layout-audit <html>           Flag WeChat editor compatibility risks in rendered HTML
moonpub layout-recipes                List layout recipes and recommended themes
moonpub preview <article.md> [--no-open]
                                      Open the rendered HTML in your browser
moonpub cover <article.md> [--style <name>] [--screenshot]
                                      Generate a cover from article frontmatter
moonpub cards <article.md>            Split an article into Xiaohongshu-style vertical cards
```

Behavior notes:

- `preflight` never calls the WeChat API and never opens Chrome. It checks the
  Markdown / HTML / `draft.json` bundle, runs `layout-audit` when HTML exists,
  treats a missing `.media_id` as a warning (that only means "not pushed yet"),
  and returns the next safe command.
- `preflight` exits 0 by default **even when `passed` is false**: the report is
  advisory, so plugin and agent callers must read `passed` themselves. Add
  `--strict` to turn it into a process-level gate for shell/CI chaining
  (`moonpub preflight a.md --strict && moonpub push a.md`). On that path a failed
  check exits non-zero and prints every failing check plus the suggested next
  command to stderr; no report is written to stdout. Use the plain `--json`
  form (without `--strict`) when you need the full `checks[]` payload.
- `layout-audit` checks rendered WeChat HTML for forbidden tags, forbidden
  attributes, full-page shells and risky CSS before it reaches the editor.
- `layout-recipes` covers life essays, quiet openings, spoken notes, collection
  openers, memory notes, photo stories, daily image cards, book notes, technical
  posts, AI engineering notes, system design reviews, daily reports with source
  indexes and transparent briefings. It also returns grouped theme suggestions so
  a CLI, plugin, app or agent can present a compact theme picker.
- `cover` style order: `--style` > frontmatter `cover_style` > `theme` >
  `editorial`. `ship` resolves the style through the same chain.
- `preview` is a local HTML preview, not the WeChat backend preview-send.

## Publish & WeChat backend

```
moonpub login [--temporary-profile]   One-time WeChat backend login by QR scan
moonpub wechat-health [--headed] [--temporary-profile]
                                      Check whether the saved browser session is reusable
moonpub push <article.md> [--render] [--temporary-profile]
                                      Push a draft, write .media_id, move the bundle to ready/
moonpub publish <article.md> --target wechat-draft [--render] [--temporary-profile]
                                      Generic publish entrypoint (only wechat-draft today)
moonpub update-draft <article.md> [--media-id <id>]
                                      Re-push updated HTML to an existing WeChat draft
moonpub ship <article.md> [--style <name>] [--ai]
                                      Cover + render + push + configure + export in one go
moonpub configure [<step>..] [--headed] [--temporary-profile] [--evidence-dir <dir>]
                                      Configure 原创 / 赞赏 / 留言 / 创作来源 and preview-send
moonpub step-test [--headed] [--temporary-profile]
                                      Interactive browser automation test
moonpub test-zanshang [--headed] [--temporary-profile]
                                      Test the 赞赏 (reward) step only
moonpub test-chuangzuo [--headed] [--temporary-profile]
                                      Test the 创作来源 (creation source) step only
moonpub test-yulan [--to <wxid>] [--title <text>] [--headed] [--temporary-profile]
                                      Test the backend preview-send step only
moonpub list-drafts                   List all drafts with media_id and title
moonpub delete-draft <media_id>       Delete a draft by media_id
moonpub mark-ready <article.md>       Move a bundle to Articles/ready
moonpub mark-published <article.md>   Move a bundle to Articles/published
moonpub export <article.md> [--target zola]
                                      Generic export entrypoint (only zola today)
```

Behavior notes:

- The first backend preview-send needs a recipient: `--to <wxid>`,
  `WECHAT_PREVIEW_TO`, or a saved `.moonpub/preview_to`. After the first
  successful send later runs are automatic; without a recipient the step prints a
  one-time setup hint and is skipped while the rest of the flow completes.
- `[wechat].auth_method = "cookie"` (or `WECHAT_AUTH_METHOD=cookie`) lets `ship`
  bypass the WeChat IP whitelist by reusing the browser session from
  `moonpub login`; in cookie mode `ship` needs no `WECHAT_APPID` / `WECHAT_SECRET`.
- When `push` creates a new draft for a bundle that already has a local
  `.media_id`, MoonPub updates `.media_id` and best-effort deletes the previous
  WeChat draft keyed by the recorded `media_id` — never by title — so same-title
  drafts are not removed accidentally.
- `--temporary-profile` isolates post-push backend automation in a throwaway
  Chrome profile and usually requires scanning the QR code again.
- Final WeChat publication stays manual by design.

## Import & research

```
moonpub fetch <url>                   Fetch a WeChat article and extract title + body (needs Chrome)
moonpub intake feishu <file> | --latest | --query <kw> | --minute-token <t>
                                      Import Feishu minutes into the Obsidian Inbox
moonpub intake photos <file-or-dir>... [--analyze-images] [--draft] [--preview] [--no-open] [--push]
                                      Import a batch of real photo files into Inbox/Photos
moonpub radar <add|list|import|analyze|suggest|scrape> ...
                                      Store and analyze platform trend samples
```

Intake behavior notes:

- The conservative default for both intake sources is `--draft --preview`; only
  an explicit `--push` continues into WeChat draft push. There are two distinct
  preview stages: local HTML preview (`moonpub preview`, no credentials) and the
  WeChat backend preview-send (part of `configure`, or `test-yulan` in isolation).
- Feishu official Minutes paths (`--minute-token` / `--latest` / `--query`) are
  idempotent on the shared `external_id` metadata field: rerunning the same
  source reuses the same Inbox file, and repeated draft generation reuses the
  same draft path with `action: "created" | "updated"`. Feishu keeps
  `minute_token` as a source-specific compatibility field.
- `intake photos` groups real image files into `Inbox/Photos/`, writes shared
  Inbox metadata (`source: photos`, `type: photo-note`, `external_id`,
  `captured_at`) and generates a factual source note from file paths, sizes and
  timestamps. `--analyze-images` is an explicit OpenAI-only opt-in that sends up
  to 5 jpg/jpeg/png/webp images (8 MiB each, 20 MiB total) for a cautious visual
  description, writes the result back to Inbox as requiring human review, and
  never pushes a WeChat draft by itself.

Radar subcommand signatures:

```
moonpub radar add --platform <name> --keyword <text> --title <text> [--url <url>] [--author <name>] [--likes/--collects/--comments <n>] [--source <text>]
moonpub radar list [--platform <name>] [--keyword <text>]
moonpub radar import <file.csv> [--platform <name>]
moonpub radar analyze <article.md> --platform <name> [--top <n>]
moonpub radar suggest <article.md> --platform <name> [--top <n>]
moonpub radar scrape --platform <name> --keyword <text> [--count <n>] [--url <url>]
```

## Integration & release

```
moonpub capabilities                  List built-in targets with risk and capability metadata
moonpub workflow-registry             List workflow contracts for plugins, apps and agents
moonpub wechat-checklist              Read-only checklist for positioning, title and read-through
moonpub evidence-status [--strict]    Check required release evidence files
moonpub release-check [--strict]      Aggregate release gate status from docs and evidence
```

`evidence-status` only checks that files exist; it does not open or inspect them.
`--strict` exits non-zero when anything is missing. `release-check --strict`
exits non-zero when any recorded gate is incomplete.

## JSON output contracts

`--json` is primarily intended for automation. Prefer `moonpub --json <command>`
in plugins and scripts; the structured workflow/discovery commands below also
accept a trailing `moonpub <command> --json` for CLI compatibility. Commands
outside this set fall back to `{"output":"..."}`.

- `moonpub --json help` → `schema_version: "help/v1"`, `moonpub_version`, `global_options[]`, `groups[]`, `commands[]` (the full command registry)
- `moonpub help <command> --json` → one command's page from the same registry
- `moonpub doctor --json` → `command`, `moonpub_version`, `articles_root`, `config_status`, `capabilities_summary[]`, `warnings[]`, `next_step`, `next_command`
- `moonpub workspace --json` → `command`, `workspace_kind`, `entry_path`, `entry_path_label`, `total_articles`, `stage_counts`, `stages[]`, `capabilities[]`, `next_command`, `next_step`
- `moonpub workflow-registry --json` → `command`, `source`, `workflows[]`; each workflow includes `id`, `package`, `status`, `owner`, `safe_start_command`, `next_command`, `user_value`, risk flags, boundary text, evidence status, and docs
- `moonpub evidence-status --json` → `command`, `base_dir`, `passed`, `required_count`, `present_count`, `missing_count`, `missing_paths[]`, `sections[]`, `next_step`, `next_command`
- `moonpub release-check --json` → `command`, `release_version`, `repo_root`, `passed`, `checks[]`, `next_step`, `next_command`
- `moonpub wechat-checklist --json` → `command`, `source`, `boundary`, `sections[]`, `next_command`, `next_step`
- `moonpub layout-recipes --json` → `command`, `guide`, `theme_groups[]`, `theme_spotlights[]`, `recipes[]`; each recipe includes `id`, `title`, `best_for`, `themes[]`, `blocks[]`
- `moonpub --json layout-audit <html>` → `command`, `html_path`, `passed`, `errors[]`, `warnings[]`, `next_step`
- `moonpub wechat-health --json` → `command`, `status`, `profile_mode`, `session_file`, `session_file_exists`, sanitized `current_url`, `next_command`, `next_step`
- `moonpub status --json` → `command`, `stages[]`, `next_command`, `next_step`; per stage `stage`, `count`, `files[]` with `file`, `slug`, `latest_status`, `latest_detail`
- `moonpub check <article.md> --json` → `command`, `article_path`, `html_path`, `draft_json_path`, `media_id_path`, `has_markdown`, `has_html`, `has_draft_json`, `has_media_id`, `publishable`, `theme`, `effective_theme`, `theme_source`, `next_command`, `next_step`
- `moonpub preflight <article.md> --json` → `command`, `article_path`, `html_path`, `draft_json_path`, `media_id_path`, `passed`, `checks[]`, `next_command`, `next_step`
- `moonpub preview <article.md> --json` → `command`, `article_path`, `html_path`, `opened_browser`, `next_command`
- `moonpub push <article.md> --json` → `command`, `article_path`, `media_id`, `stage`, `next_step`
- `moonpub draft-from-inbox <inbox.md> --json` → `command`, `input_path`, `draft_path`, optional `html_path`, `action`, `next_command`; with `--push`, also `pushed`, `media_id`, `stage`, `next_step`
- `moonpub intake feishu ... --draft --json` → `command`, `inbox_path`, `draft_path`, optional `html_path`, `action`, `next_command`; with `--push`, also `pushed`, `media_id`, `stage`, `next_step`
- `moonpub intake photos ... --draft --json` → `command: "intake-photos"`, `inbox_path`, `draft_path`, optional `html_path`, `action`, `next_command`; with `--push`, also `pushed`, `media_id`, `stage`, `next_step`
- `moonpub capabilities` (always structured) → top-level `schema_version` / `moonpub_version` plus each target's risk metadata, prerequisites and argv-style `command` template. Callers should check the schema, show missing `required_env` / `required_config`, replace the `"{article}"` placeholder and pass the array directly to the process runner instead of building a shell string.

## Debugging

- `MOONPUB_DEBUG_PROXY=1` prints which proxy MoonPub actually selected when the
  WeChat API path is unreachable. The log only shows sanitized URLs and never
  prints the `access_token` query.
- When `errcode=40164` shows a `current IP` that keeps changing, disable the
  rotating proxy or pin a stable public egress before updating the WeChat IP
  whitelist; do not whitelist a stream of historical IPs.

## Documented ideas that are not commands yet

These have reference documents and product boundaries but no CLI entry today:

- WeChat public-account archive intake: a safe first step would be
  user-provided public URL → `Inbox/WechatArchive/` → draft and local preview,
  never history crawling or storing cookies / `pass_ticket` / `uin` / tokens.
  See [WECHAT_ARCHIVE_WORKFLOW.md](WECHAT_ARCHIVE_WORKFLOW.md) / [WECHAT_ARCHIVE_WORKFLOW_ZH.md](WECHAT_ARCHIVE_WORKFLOW_ZH.md).
- Khoj-style local knowledge assistant: read-only search over `Articles/` /
  `Inbox/` with file citations and no WeChat API or browser side effects.
  See [KHOJ_REFERENCE_ZH.md](KHOJ_REFERENCE_ZH.md).
- Identity Skill-style reference-image-driven visual workflows: brief →
  reference image → asset split → visual QA loop for covers and page design.
  See [IDENTITY_SKILL_REFERENCE_ZH.md](IDENTITY_SKILL_REFERENCE_ZH.md).
- Hand-drawn explanation illustrations: read-only illustration plans confirmed
  by the user before any local asset generation; never a publish prerequisite.
  See [IAN_HANDDRAWN_PPT_REFERENCE_ZH.md](IAN_HANDDRAWN_PPT_REFERENCE_ZH.md).
- AstrBot-style product onboarding expression (first screen, support matrix,
  install paths, roadmap) — not a product pivot.
  See [ASTRBOT_README_REFERENCE_ZH.md](ASTRBOT_README_REFERENCE_ZH.md).
- Horizon-style radar / daily-digest pipelines: read-only candidate ranking with
  dedupe, source links and local draft output only. For AI briefings today use
  the `transparent-briefing` layout recipe. See [HORIZON_REFERENCE_ZH.md](HORIZON_REFERENCE_ZH.md).

For the agent / plugin integration protocol (layering, state vs action,
discovery), see [AGENT_PROTOCOL.md](AGENT_PROTOCOL.md) /
[AGENT_PROTOCOL_ZH.md](AGENT_PROTOCOL_ZH.md).
