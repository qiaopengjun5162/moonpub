# MoonPub Agent / App Entry Protocol

This document answers one question only:

**If you are not exposing the CLI directly to humans, but wiring MoonPub into an Obsidian plugin, local app, agent, or automation script, which commands should you read first and in what layers?**

The goal is not to relist all commands, but to converge the currently stable high-level entries into a clearer protocol.

## One-sentence principle

Priority from high to low:

1. `doctor --json`
2. `workflow-registry --json`
3. `evidence-status --json`
4. `release-check --json`
5. `workspace --json`
6. `layout-recipes --json`
7. `status --json`
8. `check <article.md> --json`
9. `preflight <article.md> --json`
10. Concrete action commands: `preview` / `push` / `draft-from-inbox` / `intake feishu ... --draft`
11. `capabilities --json`

That means:

- Check whether the local environment can start, especially on plugin homepage or first open
- Read MoonPub's built-in workflow contracts before guessing paths from README or terminal text
- Read release / first-run evidence gaps before writing "implemented" as "verified"
- Judge which entry the whole workspace should take
- Read layout recipes and theme groups before duplicating theme metadata in upper layers
- Judge what is in the current pool
- Judge what a specific article is missing
- Run a read-only pre-publish quality gate
- Finally execute concrete actions

## Layer 1: Local diagnostics entry

### `moonpub doctor --json`

This is the local diagnostics entry for first use and the plugin homepage.

It answers:

- What is the current MoonPub CLI version?
- What is the current Articles root directory?
- Can the local config be found?
- What warnings remain for first use?
- Should the next step be initialize, create an article, or enter the workspace homepage?

Suitable for:

- "Can we start?" area at the top of the Obsidian plugin homepage
- Local app first-launch checks
- Agent confirming local environment before actions

Current key fields:

- `moonpub_version`
- `articles_root`
- `config_status`
- `capabilities_summary`
- `warnings`
- `next_command`
- `next_step`

Constraints:

- Does not trigger the WeChat API
- Does not open or control Chrome
- Does not read, print, or return real secrets

## Layer 2: Workspace entry

### `moonpub workspace --json`

This is the workspace-level entry.

It answers:

- What kind of workspace is this?
- Which entry path fits best right now?
- How many articles are in drafts / ready / published stages?
- What risk boundaries do built-in targets have?
- What is the most recommended next command now?

Suitable for:

- Obsidian plugin homepage
- Local app dashboard
- Agent first taking over a vault
- Automation script "look at the big picture first"

Current key fields:

- `workspace_kind`
- `entry_path`
- `entry_path_label`
- `total_articles`
- `stage_counts`
- `stages`
- `capabilities`
- `next_command`
- `next_step`

Recommended practice:

- Show `entry_path_label` to the user
- Treat `next_command` as the smallest next action
- Use `requires_network` / `requires_browser` from `capabilities` as risk hints

## Layer 3: Formal workflow catalog

### `moonpub workflow-registry --json`

This is MoonPub's built-in workflow contract catalog.

It answers:

- Which main paths are officially supported
- Which package / owner each path belongs to
- Which command is the safe starting point
- Which command moves to the next stage
- Whether a path needs network or browser
- What the current evidence status and docs entry are

Suitable for:

- Path selection area of the Obsidian plugin homepage
- Workflow picker in a local app
- Agent capability discovery before taking a task
- Avoiding reverse-parsing commands from README text

Current key fields:

- `id`
- `package`
- `status`
- `owner`
- `entry_command`
- `safe_start_command`
- `next_command`
- `requires_network`
- `requires_browser`
- `production_boundary`
- `evidence_status`
- `docs`

Constraints:

- Does not trigger the WeChat API
- Does not open or control Chrome
- Does not read, print, or return real secrets
- Built-in static contracts only; does not download content from an external registry

## Layer 4: Evidence status

### `moonpub evidence-status --json`

This is the read-only local interface for release gates and first-run evidence.

It answers:

- Whether the required evidence files for v0.4.2 are already in the fixed directory
- How many files are required
- How many are archived
- How many are missing and which paths
- Whether the next step is to add evidence or proceed to manual de-sensitization review

Suitable for:

- Release evidence hint area on the Obsidian plugin homepage
- Release gate panel in a local app
- Agent closeout confirming "code complete" and "real user evidence" are not conflated

For release scripts or CI gates, use `moonpub evidence-status --strict`. The default mode only reports status; `--strict` exits non-zero when required evidence files are missing. Neither mode opens images, reads image content, or replaces manual de-sensitization review.

Current key fields:

- `base_dir`
- `passed`
- `required_count`
- `present_count`
- `missing_count`
- `missing_paths`

## Layer 5: Release gate status

### `moonpub release-check --json`

This is the read-only overall release gate interface before v0.4.2 release.

It answers:

- Whether release gate docs exist
- Whether local release smoke, CI / Windows smoke are recorded as done
- Whether real WeChat regression, evidence files, doc consistency, and privacy review are still incomplete
- Which gate should be filled next

Suitable for:

- Release scripts or CI gates
- Agent closeout judging the gap between "code is done" and "v0.4.2 is releasable"
- Plugin / app pre-release status panel

`release-check` only reports status by default; `release-check --strict` exits non-zero if any gate is incomplete. It does not trigger the WeChat API, open a browser, scan image content, or replace manual de-sensitization review.

Current key fields:

- `release_version`
- `repo_root`
- `passed`
- `checks`
- `next_command`
- `next_step`

Constraints:

- Does not open images
- Does not read image content
- Does not trigger the WeChat API
- Does not replace manual de-sensitization review

## Layer 6: Layout and theme discovery

### `moonpub layout-recipes --json`

This is the read-only discovery interface for article layout and theme selection.

It answers:

- Which theme groups exist for a compact theme picker
- Which themes should be highlighted first, such as `geek-black`, `blueprint`, and `ai-lab`
- Which themes and blocks fit each article type
- Which full layout recipe guide the user should read

Suitable for:

- Theme selection areas in the Obsidian plugin
- Layout wizards in a local app
- Agents choosing the visual tone before drafting for a user
- Avoiding stale copied theme tables in plugins or apps

Current key fields:

- `guide`
- `theme_groups`
- `theme_spotlights`
- `recipes`

Constraints:

- Does not trigger the WeChat API
- Does not open or control Chrome
- Does not read article content
- Only returns built-in static layout metadata

## Layer 7: Article pool status

### `moonpub status --json`

This is the "article pool" status interface.

It answers:

- Which articles are in drafts / ready / published stages
- What the latest status record is for each article
- Which article is recommended to handle next if only looking at the pool

Suitable for:

- List views
- Stage filter pages
- Article pool statistics views

If you have already called `workspace --json`, usually only continue to `status --json` when you need to show a finer file list.

## Layer 8: Single-article status

### `moonpub check <article.md> --json`

This is the "single article" status interface.

It answers:

- Whether the article has markdown
- Whether it has html
- Whether it has draft.json
- Whether it has media_id
- Whether it is ready for the next step
- What command is most recommended next

Suitable for:

- Current-file detail panel
- Single-article status check button
- Pre-publish self-check

### `moonpub preflight <article.md> --json`

This is the "pre-publish local quality gate".

It answers:

- Whether the article bundle markdown / html / draft.json are complete
- Whether the rendered HTML passes the WeChat typography audit
- Whether `.media_id` already exists
- Whether it is okay to continue reaching the WeChat API
- Whether the next step should be render, fix HTML, push, or first check browser login state

Suitable for:

- "Pre-publish check" button in the current-article workspace
- Mandatory read-only check for an agent before `push`
- Local artifact quality gate in CI or scripts

Constraints:

- Does not trigger the WeChat API
- Does not open or control Chrome
- Missing `.media_id` is only a warning, because it means the draft has not been pushed to WeChat yet, not that local artifacts failed

## Layer 9: Action commands

These commands are for "actually moving the flow forward", not for "judging first".

### `moonpub preview <article.md> --json`

Suitable for:

- Local HTML preview
- Opening or emitting the local preview path

### `moonpub push <article.md> --json`

Suitable for:

- Pushing the article to a WeChat draft
- Returning `media_id`, `stage`, and next-step advice

### `moonpub draft-from-inbox <inbox.md> --json`

Suitable for:

- Generating an editable draft from an existing Inbox text

### `moonpub intake feishu ... --draft --json`

Suitable for:

- Importing from Feishu Minutes and continuing to generate a draft

Both "generate draft" commands support:

- `action: created | updated`
- Optional `html_path`
- `next_command`

When `--push` is explicitly added, they additionally return:

- `pushed`
- `media_id`
- `stage`
- `next_step`

## Layer 10: Capability metadata

### `moonpub capabilities --json`

This command does not tell you "how the current workspace is", but tells you:

- Which built-in targets exist
- Whether each target uses the network
- Whether it may open a browser
- Which env / config it depends on
- What risks and follow-up manual steps it has

Suitable for:

- Pre-publish hints
- App button permission descriptions
- Risk dialogs before executing an action

## Recommended integration order

If you are building a new user entry, wire in this order:

1. `doctor --json`
2. `workflow-registry --json`
3. `evidence-status --json`
4. `workspace --json`
5. `check --json`
6. `preflight --json`
7. `preview --json`
8. `push --json`

If you are building the Feishu flow:

1. `doctor --json`
2. `workflow-registry --json`
3. `evidence-status --json`
4. `workspace --json`
5. `intake feishu ... --draft --json`
6. Back to `check --json`
7. After confirming, `preflight --json`
8. Finally `push --json`

## Not recommended

Currently not recommended:

- Reverse-parsing state from plain text output
- The plugin inventing its own "current workspace entry semantics"
- Skipping `workspace` and directly assuming the user wants to publish
- Using `capabilities --json` as a substitute for workspace state judgment

Because these approaches make the entry layer diverge again and eventually return to "capabilities exist, but users don't know how to use them".

## What this protocol means now

This protocol shows that MoonPub is no longer just a "collection of commands".

It already has:

- A local publishing kernel
- Input workflows
- A user entry protocol

The next step, whether continuing the Obsidian plugin, the Feishu path, or future local app / agent, is not to reinvent logic, but to keep reusing and stabilizing this entry protocol.
