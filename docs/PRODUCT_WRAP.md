# MoonPub Product Wrap

This document answers one question only:

**How should MoonPub be understood right now, so it is not seen as a pile of unrelated commands?**

It is not a vision doc and not an implementation spec. The goal is to wrap the currently existing capabilities into a product shape that users, plugins, apps, and agents can all reuse.

## One-sentence positioning

**MoonPub is a local-first content publishing kernel.**

It ingests various source materials, turns them into drafts, completes local preview, WeChat draft push, and assisted backend configuration, and leaves the "what to do next" decision to the upper-level entry point.

## What it is not yet

Do not currently understand MoonPub as any of the following:

- Not an unattended auto-publishing bot
- Not a single command that only pushes WeChat drafts
- Not a one-off script only for Feishu Minutes
- Not a fully productized multi-platform SaaS

A more accurate description is:

- A local kernel that can actually run the main content publishing pipeline
- A workflow system that has already started to support multiple input sources
- A low-level runtime suitable for further wrapping by plugins, apps, and agents

## Three-layer structure

To avoid continuing to mix all capabilities into "many commands", MoonPub is now understood in three layers.

### Layer 1: MoonPub Core

Role: local publishing kernel.

Responsible for:

- Markdown / Obsidian article rendering
- WeChat-compatible HTML and draft JSON generation
- Cover image generation
- WeChat API draft push
- WeChat backend browser automation assistance
- Article stage management
- Blog export
- Structured JSON protocol

This layer solves:

**How an article reliably moves from a local file to a publishable state.**

### Layer 2: Input Workflows

Role: input-source workflow layer.

Responsible for bringing content that is "not yet an article" into the system, then advancing it to the draft, preview, and publish main line.

Two input workflows are currently formal:

1. Feishu Minutes
   `Feishu Minutes -> Inbox -> Draft -> Preview -> WeChat Draft`
2. Photos
   `Photos -> Inbox -> Draft -> Preview -> WeChat Draft`

An input workflow under evaluation but not yet a formal entry:

- WeChat Official Account archive
  `Known WeChat URL -> Inbox -> Draft -> Preview`

This line should follow the safety boundaries in [WECHAT_ARCHIVE_WORKFLOW.md](WECHAT_ARCHIVE_WORKFLOW.md) (English) / [WECHAT_ARCHIVE_WORKFLOW_ZH.md](WECHAT_ARCHIVE_WORKFLOW_ZH.md) (Chinese): by default only process public URLs explicitly provided by the user, do not automatically scrape historical lists, and do not save sensitive credentials.

This layer solves:

**How raw material from different sources is first organized into an editable draft.**

### Layer 3: User Surfaces

Role: user entry-point layer.

Responsible for exposing MoonPub's capabilities to real users or upper-level systems through different entry points.

Two formal entry points currently exist:

1. CLI
2. Obsidian plugin

Two entry points are forming:

1. Local app
2. Agent wrapper

This layer solves:

**Where users or systems should enter, what they see first, and what to do next.**

### Optional future layer: Knowledge Assistant

Role: local read-only knowledge assistant layer.

This is not a current formal layer and not the main line for v0.4.x / v0.5. It is kept only as a later direction:

- Local read-only search
  `Articles / Inbox -> Search Results with Sources`
- Question-and-answer with source citations
  `Articles / Inbox -> Answer with File Citations`

This line should follow the safety boundaries in [KHOJ_REFERENCE_ZH.md](KHOJ_REFERENCE_ZH.md): read-only by default, returns sources, does not trigger the WeChat API, does not open a browser, and does not write back to files.

## Current formal capability map

In product terms, MoonPub is no longer "just a push command". It now has at least the following formal parts:

### Formal input workflows

- `Existing Markdown article`
- `Feishu Minutes`
- `Photo material`

### Recorded but not yet enabled input workflows

- `WeChat Official Account archive URL`

### Formal entry points

- `moonpub` CLI
- `obsidian-plugin/`

### Protocol layer stable enough for upper layers to reuse

- `workspace --json`
- `workflow-registry --json`
- `layout-recipes --json`
- `status --json`
- `check --json`
- `preview --json`
- `push --json`
- `draft-from-inbox --json`
- `intake feishu ... --draft --json`
- `intake photos ... --draft --json`
- `capabilities --json`

## Why the Feishu workflow is not a separate project yet

The current conclusion is clear:

**Feishu Minutes should first be advanced as a formal module inside MoonPub, not split into a new project.**

The reason is not "never split", but that the strongest value still depends on the MoonPub main pipeline:

- The value of Feishu is not just importing text, but turning the import directly into a publishable draft
- What is actually missing now is user entry and product expression, not repository splitting
- Input sources and the publishing kernel still need to iterate together frequently

The same judgment applies to the photo workflow: first make it a formal input workflow, rather than rushing to make it an independent project.

## How agents should integrate with MoonPub

Currently it is also not recommended to understand an agent as "building another new product".

A more reasonable direction is:

**Wrap MoonPub as an agent-ready local publishing kernel.**

That means:

- The agent first calls `workspace --json` to judge the current workspace state
- Then reads `workflow-registry --json` to understand currently supported workflow contracts, risk boundaries, and safe starting points
- When helping users choose presentation, reads `layout-recipes --json` for theme groups, highlighted themes, and recipe combinations
- Then uses `status --json` / `check --json` to judge the overall pool or the current article
- Only then triggers `preview`, `draft-from-inbox`, `intake feishu`, `intake photos`, or `push`

This way the agent does not need to reinvent workflow semantics, but reuses the protocol layer MoonPub is already stabilizing.

## Recommended first-time experience

For a first-time user, the currently recommended order is not to read all commands at once, but:

1. Enter through the `MoonPub homepage workspace` in Obsidian
2. Choose the current article, Feishu, or photo entry based on context
3. Stop at the draft and local preview for the first round
4. After understanding the workflow rhythm, advance to a real WeChat draft

The reasons:

- The plugin homepage is starting to play the role of a unified entry layer
- The Feishu path best demonstrates workflow value
- The photo path shows that MoonPub already has a formal multi-source shape
- Stopping first at draft and local preview best fits MoonPub's current boundary of "copilot, not unattended robot"

## Current target users

MoonPub is not for everyone right now. It best fits people who:

- Already write in Obsidian / Markdown
- Accept the "preview first, confirm, then publish" rhythm
- Are technical enough to configure a local environment and WeChat Official Account credentials
- Need to turn Feishu Minutes, photos, and later voice memos into articles

## What not to do in the near term

To avoid further scattering, the following are explicitly not main-line priorities in the short term:

- Do not wrap MoonPub as an unattended auto-publishing bot first
- Do not split the Feishu workflow into a separate repository just because it has potential
- Do not expand to many new platforms while ignoring the current entry-point convergence
- Do not add many new commands while users still do not know which path to run first

## Relationship to other documents

If you are trying to solve a problem at a different level, continue here:

- Want to know which path to take first: [RECOMMENDED_WORKFLOWS_ZH.md](RECOMMENDED_WORKFLOWS_ZH.md)
- Want to know what to do at the current stage: [EXECUTION_PLAN_ZH.md](EXECUTION_PLAN_ZH.md)
- Want to know why the project is positioned this way: [PRODUCT_EVALUATION_ZH.md](PRODUCT_EVALUATION_ZH.md)
- Want to know how plugins / apps / agents should use the protocol: [AGENT_PROTOCOL_ZH.md](AGENT_PROTOCOL_ZH.md)
- Want to know how input sources should be modeled uniformly: [INPUT_MODEL.md](INPUT_MODEL.md) (English) / [INPUT_MODEL_ZH.md](INPUT_MODEL_ZH.md) (Chinese)

## Current simplest product conclusion

If only one sentence is kept:

**MoonPub should currently be understood as: a local publishing kernel that has started to grow formal input workflows and a formal user entry-point layer.**
