# MoonPub Execution Plan

This document is not a vision statement and not a scattered TODO list.

It answers only 3 questions:

1. **What should this project focus on now?**
2. **What counts as "done" for the current stage?**
3. **What order should the next steps follow?**

The goal is to converge the overall evaluation into an executable plan, so the project does not drift back into "continuous optimization with an increasingly scattered main line".

If you are still judging "what product MoonPub is right now, rather than a pile of capabilities and commands", read [PRODUCT_WRAP.md](PRODUCT_WRAP.md) first. Then this execution plan will be easier to align on.

## Overall goal

The current-stage goal of MoonPub is not "as many capabilities as possible", but:

**Make MoonPub a local publishing kernel that users can understand, run, and that plugins / agents can reuse.**

More specifically:

- When users first see the project, they know which path to take
- At least three main paths are in a formally closed state:
  - Normal Markdown article path
  - Feishu Minutes path
  - Photo material path
- CLI, Obsidian plugin, and future agents no longer invent their own process, but reuse the same entry protocol layer

## Current product judgments

### 1. Feishu workflow

The current conclusion is clear:

- **Do not split it into a new project now**
- **Advance it first as a formal module inside MoonPub**

Reasons:

- The strongest current value depends on the MoonPub main pipeline
- It has not yet formed an independent product boundary
- The current bottleneck is "users don't know how to use it", not "the repository is not split enough"

### 2. AI Agent direction

The more accurate statement now is not "build another AI Agent project", but:

- **Wrap MoonPub as an agent-ready local publishing kernel**

That means:

- Input sources: Feishu, Obsidian, future photos / voice / excerpts
- State layer: workspace / status / check
- Action layer: preview / push / draft-from-inbox / intake feishu
- Risk layer: capabilities

## Three-layer structure

### MoonPub Core

Responsible for:

- Rendering
- Cover generation
- Push
- WeChat backend automation
- Export
- Stage tracking

### Input Workflows

Responsible for:

- Feishu Minutes
- Photo organization
- Future voice notes
- Future reading excerpts

### User Surfaces

Responsible for:

- CLI
- Obsidian plugin
- Future local app
- Future agent wrapper

## Current-stage milestones

### M1: Users understand the entry points

Goal:

- When users first enter the project, they know which path fits them

Done criteria:

- [x] README first screen clearly shows user entry points
- [x] Recommended workflows document exists
- [x] Feishu path appears alongside the normal article path
- [x] Obsidian plugin is described as one of the formal entry points

Current evidence:

- `docs/RECOMMENDED_WORKFLOWS.md` / `docs/RECOMMENDED_WORKFLOWS_ZH.md`
- `README.md`
- `README_zh.md`
- `docs/PRODUCT_EVALUATION_ZH.md`

### M2: Users can run the main paths

Goal:

- Users not only understand, but can actually run at least one main path

Done criteria:

- [x] Normal article path is clearly visible
- [x] Feishu Minutes path documentation is closed
- [x] Feishu Minutes real loop is verified through WeChat backend preview send
- [x] Obsidian plugin can serve as a non-confusing entry
- [x] Real WeChat regression screenshots / recordings are complete
- [x] First-timer perspective walkthrough is archived for homepage, Feishu, photo, and current article paths

Current evidence:

- `PROGRESS.md` records the real Feishu closed-loop verification on 2026-07-01
- `obsidian-plugin/README.md`
- `docs/USER_GUIDE.md`
- `docs/first-run-evidence/`
- `docs/FIRST_RUN_WALKTHROUGH_ZH.md`
- `docs/RELEASE_GATE_v0.4.2_ZH.md`

### M3: Entry protocol is stable

Goal:

- Plugins / apps / agents no longer piece together workspace semantics on their own

Done criteria:

- [x] `workspace --json`
- [x] `status --json`
- [x] `check --json`
- [x] `capabilities --json`
- [x] Obsidian plugin consumes `workspace --json`
- [ ] One more real entry reuses this protocol layer

Current evidence:

- `src/app.rs`
- `obsidian-plugin/main.ts`
- `docs/AGENT_PROTOCOL_ZH.md`

### M4: Feishu is formally modularized

Goal:

- The Feishu workflow no longer feels like an add-on, but a core input workflow

Done criteria:

- [x] Feishu workflow documentation is a first-class entry
- [x] Default conservative mode and explicit fast mode are defined
- [x] `--draft` / `--preview` / `--push` semantics are stable
- [x] Idempotent updates and `action: created | updated` are stable
- [x] A unified input model for future photo / voice sources is drafted (`docs/INPUT_MODEL_ZH.md` exists, Feishu Inbox has started using generic `external_id`)
- [ ] Feishu entry is further converged into a more complete flow description or UI entry

Current evidence:

- `src/intake.rs`
- `src/ai_workflow.rs`
- `docs/RECOMMENDED_WORKFLOWS.md` / `docs/RECOMMENDED_WORKFLOWS_ZH.md`
- `docs/AGENT_PROTOCOL_ZH.md`
- `docs/INPUT_MODEL.md` / `docs/INPUT_MODEL_ZH.md`

## Next 2-4 week execution order

### P1: Close real-user evidence (completed)

Priority: highest

Completed:

1. Add real WeChat regression screenshots / short recordings
2. Walk the main path once more from a first-trial user's perspective
3. Write lessons back into documentation

Completion evidence:

- De-screenshoted evidence of real official-account draft creation, editor configuration, and backend preview send is archived
- 11 first-run evidence files for homepage, Feishu, and photo paths are archived and pass `moonpub evidence-status --strict`
- `moonpub release-check --strict` passes; final public release still requires manual PR review, merge, tag, and release asset smoke

### P2: Make the plugin entry feel more like a homepage

Priority: high

What to do:

1. Build clearer prompts based on `workspace --json`
2. Distinguish "workspace entry" from "current article entry"
3. Present risk targets more naturally

Done criteria:

- The plugin does not just pop a Notice
- Users can more naturally judge the next step in Obsidian

Current status:

- Homepage workspace first version exists
- Real evidence is complete; current focus shifts to details, consistency, confirmation dialogs, and result-workspace interaction regression

### P3: Define a unified input model

Priority: high

What to do:

1. Summarize the common input shape of Feishu, photos, and voice
2. Clarify which fields belong to raw material and which belong to draft metadata
3. Leave interfaces for future input sources

Done criteria:

- At least one input model specification document
- Feishu no longer feels like a special case

### P4: Continue clarifying agent-ready boundaries

Priority: medium-high

What to do:

1. Close another round based on `docs/AGENT_PROTOCOL_ZH.md`
2. Clarify which commands belong to the state layer and which to the action layer
3. If needed, add more `workspace` upper-layer integration examples

Done criteria:

- Future new entries do not need to guess MoonPub's flow boundaries

## What is not a priority now

These are not the most important things to do first:

- Split Feishu into a separate repository immediately
- Promise a "complete AI Agent product" now
- Keep expanding horizontally to many platforms
- Host AppSecret in the cloud
- Market "auto-final-publish" as a feature

There is only one reason:

**The current main problem is still "make users know how to use it", not "let capabilities keep diverging".**

## Current next steps

If looking only at the 3 most worthwhile things right now:

1. Complete real WeChat regression evidence
2. Complete first-run evidence for plugin homepage / Feishu / photo paths
3. Organize a unified input model for Feishu / photo / voice

## How to use this document

After each significant push, return to this document and update:

- Which milestone was advanced
- Which done criteria now have evidence
- Whether the current next step has changed

If subsequent judgments change, update this document first rather than letting the plan scatter across README, ROADMAP, PROGRESS, and chat logs.
