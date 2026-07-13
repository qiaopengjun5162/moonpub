# MoonPub v0.4.2

MoonPub v0.4.2 is a Beta release focused on making the local publishing workflow easier to verify before it reaches WeChat.

It is for technical users who can configure their own WeChat Official Account credentials and are willing to review every draft manually.

## Highlights

- Added a local readiness check with `moonpub doctor` and structured `doctor --json` output.
- Added workflow discovery, workspace status, evidence status, and release gate JSON contracts for the Obsidian plugin and future Agent integrations.
- Added formal Feishu Minutes and photo metadata intake flows that stop at editable local drafts and HTML previews by default.
- Added explicit confirmation before the plugin sends Feishu transcripts or photo metadata to the configured AI provider.
- Added optional OpenAI-only photo vision intake behind a separate confirmation, with fixed image limits and an "needs human review" boundary.
- Added `layout-audit` and `preflight` so WeChat-compatible HTML and local article bundles can be checked before any WeChat API call.
- Added `moonlit`, `porcelain`, and `fieldnote` reading themes plus life-writing layout recipes.
- Improved browser automation recovery: persistent and temporary profiles are explicit, headless login failures fail fast, and `wechat-health` reports whether a saved session is reusable.
- Added an Obsidian homepage workspace with current-article, Feishu, and photo entry paths; repeated homepage opens now replace the prior modal instead of stacking over result workbenches.

## Verified

- Rust formatting, strict Clippy, and the full `cargo nextest run --all-features` suite passed during the v0.4.2 closeout.
- The Obsidian plugin build passed.
- `moonpub evidence-status --strict` reports `11/11` required first-run evidence files present.
- `moonpub release-check --strict` passes.
- Real Feishu and photo metadata flows produced local Inbox items, editable drafts, and HTML previews without pushing WeChat drafts.
- A real WeChat regression created a draft, completed the supported backend configuration steps, and sent a backend preview without clicking final publish.
- The public macOS ARM64 asset was downloaded from GitHub Releases, verified against its SHA-256 file, and passed the no-credential `--version -> init -> new -> render -> check` smoke.

## Safety Boundaries

- MoonPub does not automatically publish final WeChat articles.
- WeChat QR login, verification, platform review, and final publish remain manual.
- Default photo intake sends metadata only. Sending image pixels requires the separate `--analyze-images` flow and OpenAI confirmation.
- Feishu transcripts and photo materials remain local until the user explicitly confirms the AI drafting step.

## Upgrade Notes

The recommended starting point is the Obsidian plugin homepage. It checks `moonpub --json doctor`, shows the recommended safe path for the active file, and keeps WeChat draft handoff as an explicit later action.

For CLI users, start with local-only checks:

```bash
moonpub doctor
moonpub workflow-registry
moonpub workspace
moonpub preflight Articles/drafts/article.md
```

See `docs/RELEASE_GATE_v0.4.2_ZH.md` for the Chinese release gate record and `docs/FIRST_RUN_WALKTHROUGH_ZH.md` for the first-run path.
