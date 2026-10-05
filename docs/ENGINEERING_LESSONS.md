# MoonPub Engineering Lessons

This document is MoonPub's long-term, searchable record of solved engineering problems.

It serves two purposes: check whether a conclusion already exists before debugging, and only cite verified real problems and trade-offs when writing public articles. It does not record tokens, QR codes, accounts, private articles, photos, or full logs.

## Usage Rules

Before debugging, search this document and `AGENTS.md` by keyword. After a problem is fixed and verified, add or update a record whenever any of the following applies:

- It affects a real user path, data safety, privacy, or publishing result.
- The root cause is not obvious and is likely to recur.
- The fix introduces a new boundary, command, configuration convention, or regression test.

Each record should keep at least: symptom and impact scope, confirmed root cause, final fix, prevention constraints, publicly reviewable evidence, and an optional article angle. If a fix is not verified yet, keep it in an issue, PR, or temporary debugging note instead of writing it here as an established conclusion.

## High-priority Lessons

### Backend configuration must verify persisted state, not click logs

**Symptom:** `moonpub ship` logs showed original / reward / comments as successful, but the actual WeChat backend draft remained "not declared / disabled / comments disabled". All settings were not saved, and multiple runs falsely reported success.

**Root cause:** Four layers stacked together: the steps only verified that clicks were issued, not that settings took effect; WeChat editor setting rows contain both inactive and active blocks, where the inactive block is `display:none` but still present in the DOM, so reading `textContent` treated hidden template text as real state; global text-based confirmation clicks such as `cdp_click_exact_last("OK")` could hit another unclosed dialog; the browser closed without explicitly saving the draft, so settings only lived in page memory.

**Fix:** All steps now follow "state first + scoped container + explicit save + reload verification". Original statement uses `.claim__original-dialog`; author input readiness is verified before confirming, and validation failures retry. Reward is enabled from the visible `.js_reward_open` setting row after original statement. Comments use `input.js_interaction_setting.checked` as real state. `step_baocun` explicitly clicks "save as draft"; `step_fucha` reloads the editor and verifies persisted state by visibility and selected values.

**Prevention:** Any WeChat editor state read must check visibility (`offsetParent`, and `getClientRects().length` for fixed elements). Confirmation clicks must be scoped to a concrete dialog. Picker dialogs must use coordinates gathered by script and trusted CDP mouse events, because synthetic JS clicks (`isTrusted=false`) are ignored. Close residual dialogs before saving. Reloaded page state is the only success standard. New editor automation steps need script tests with visibility assertions.

**Evidence:** `src/publish_steps.rs` steps and `ORIGINAL_*` / `ZANSHANG_*` / `COMMENT_*` / `FUCHA_SCRIPT` tests; 2026-07-26 real Walden `ship` run for draft `100011146` verified original, reward, creation source, comments, and mobile preview `ret=0`.

**Article angle:** Between "clicked" and "done" sit three verification layers. In automation, the expensive failure is not missing a click; it is clicking the wrong thing and believing it worked.

### Cookie-mode cover must be uploaded explicitly and passed to draft creation

**Symptom:** In cookie mode, `moonpub ship` created WeChat drafts with an empty cover. A book-review article with remote `cover:` frontmatter failed cover upload with `ret=200002`.

**Root cause:** `download_cover` saved JPEG bytes as `.cover.png`, so the declared MIME did not match the actual bytes and WeChat rejected it. `create_draft_cookie` ignored the upload result and always picked the first `https://` image in the body. After the footer QR code moved to a data URI, the body no longer had any `https://` images, so cover fallback disappeared.

**Fix:** Downloaded covers now choose the real extension from Content-Type / magic bytes (`.jpg` / `.png`) and clear stale `<slug>.cover.*` files before writing. `push_article_cookie` uploads the cover and passes the CDN URL and file id explicitly into `create_draft_cookie` as `cdn_url0` / `fileid0`; only when empty does it fall back to the first body `https://` image. Cookie mode also decodes embedded data URI images, uploads them to CDN, and replaces them so the WeChat editor does not strip them.

**Additional verified detail (2026-07-25):** `filetransfer?action=upload_material&type=image` response shape depends on query parameters. Without `writetype=doublewrite&groupid=1`, it only returns numeric file id; with them, it returns a real `cdn_url`. Synthesizing CDN URLs from file ids is rejected by `operate_appmsg`. After successful cover upload, use both channels: `cdn_url` for `cdn_url0`, numeric file id for `fileid0`.

**Prevention:** Image file extensions must reflect real bytes. Draft cover source must be passed explicitly, not depend on the body accidentally having an `https://` image. New image embedding methods (data URI, local paths) need CDN upload regression tests. Do not guess WeChat API return values from field names; inspect real responses first.

**Evidence:** `src/ship.rs` `download_cover` / `cover_extension`, `src/push_browser.rs` `UploadOutcome` / `decode_data_uri` regression tests, and a 2026-07-25 real Walden `ship` run with cover upload, QR code CDN replacement, draft `100011016`, and mobile preview `ret=0`.

**Article angle:** "Works" and "works correctly" are separated by implicit assumptions. "Cover comes from the first body image" silently fails once body structure changes.

### Browser handle must stay alive until QR scan and session save finish

**Symptom:** `moonpub login` opened the browser and quickly failed with `oneshot canceled`, so QR scan could not complete.

**Root cause:** The login flow released the underlying `Browser` handle too early, canceling the CDP session.

**Fix:** The login path now keeps an active `Browser` alive until QR scan completes and session is saved.

**Prevention:** Login and QR recovery paths must not pass only `Page`. Browser lifecycle changes must be covered by resource-lifetime regression.

**Evidence:** `src/cdp.rs` and `PROGRESS.md` 2026-06-30 record.

**Article angle:** Browser automation failures are not always about QR scan. Sometimes they are client-side resource lifetime bugs.

### Headless mode must not wait for invisible QR codes

**Symptom:** After login state expired, background automation waited for a QR code in headless mode and timed out; the user had no visible window to operate.

**Root cause:** Reusable-session background flows and interactive QR login flows were mixed into one path.

**Fix:** In headless mode, failed session restore now fails fast and asks the user to run `moonpub login` or `moonpub configure --headed`. Temporary profile mode explicitly states that it cannot reuse persistent session.

**Prevention:** Invisible flows must not wait for human input. New browser entries must clearly define `headed`, persistent profile, and temporary profile login semantics.

**Evidence:** `src/cdp.rs` `headless_login_required_message` and `docs/USER_GUIDE.md`.

### A locked persistent Chrome profile is not token expiration

**Symptom:** Chrome launch errors containing `SingletonLock` or `ProcessSingleton` were easy to misread as expired WeChat login state.

**Root cause:** Another MoonPub automation Chrome window was using the persistent profile.

**Fix:** Low-level errors are now collapsed into a readable message: close the existing automation window, or use `--temporary-profile` for an isolated one-off check.

**Prevention:** Browser launch failures must first distinguish profile locks, login-state expiration, and backend page changes. Do not ask users to re-login for every browser failure.

**Evidence:** `src/cdp.rs` `browser_launch_error_message` regression test and `PROGRESS.md` 2026-07-04 record.

**Additional detail:** A 2026-07-14 real regression confirmed that when another `moonpub test-yulan --headed` process occupies the same profile, Chrome may first appear as a WebSocket address timeout instead of returning `SingletonLock`. Check for existing MoonPub automation processes and Chrome using the same profile before blaming WeChat login state.

### WeChat editor selectors should prefer DOM structure, not visible text

**Symptom:** Creation-source text could be split by icons and whitespace, so `textContent` option matching failed or misread state intermittently.

**Root cause:** The WeChat editor is a dynamic page; visible copy is not a stable DOM interface.

**Fix:** Open the entry with `.js_claim_source_desc`, select the stable source with `input[type="radio"][value="4"]`, and verify through `.js_claim_source_selected`.

**Prevention:** Automation should prefer DOM structure, class names, and input values. Setting steps should soft-fail without blocking WeChat API draft creation.

**Evidence:** `src/publish_steps.rs`, `docs/BROWSER_AUTOMATION.md`, and `PROGRESS.md` 2026-07-03 real regression.

**Article angle:** Why MoonPub does not promise browser automation will be fully automatic forever.

### Config assets and article-relative assets need different resolution bases

**Symptom:** Footer QR code existed in local HTML, but failed to upload or display after pushing to WeChat.

**Root cause:** Article-relative images should resolve from the article directory, while config assets such as `qrcode` / `cover` should resolve from the articles root. These rules were mixed.

**Fix:** Rendering now resolves config assets to absolute paths using articles root before passing them to upload logic.

**Prevention:** Config paths should be normalized at the config boundary. New config assets need articles-root resolution tests and must not reuse article-directory rules.

**Evidence:** `AGENTS.md` config-asset constraint and historical notes in `CLAUDE.md`.

### WeChat cover media_id is not permanent cache

**Symptom:** A once-valid permanent material was deleted, and the stale `thumb_media_id` caused `ship` push failure.

**Root cause:** WeChat material lifecycle is not controlled by local config; cached media ids can expire.

**Fix:** `ship` now generates and uploads a cover PNG each run and uses the returned media id. Config value is only a final fallback.

**Prevention:** External platform resource identifiers are invalidatable cache. Write down refresh strategy and failure recovery.

**Evidence:** `src/ship.rs` cover upload behavior, regression tests, and historical notes in `CLAUDE.md`.

### WeChat IP whitelist failures can come from public egress drift

**Symptom:** The user added the previous `40164 invalid ip` address to the whitelist, but a later retry still failed and the error's `current IP` had changed.

**Root cause:** WeChat validates the actual public egress IP of each API request. Home broadband, mobile networks, and rotating proxies can change egress. A historical IP being whitelisted does not mean the current request still uses it.

**Fix:** `40164` errors now print the current IP and clearly suggest disabling rotating proxies or using stable egress before updating the whitelist. User docs explain that accumulating historical IPs does not fix drift.

**Prevention:** During debugging, verify both default environment and no-proxy requests. If both have the same IP but it differs from history, treat it as public egress change. Do not conflate browser login state, token expiration, and API IP whitelist.

**Evidence:** On 2026-07-15 the same `update-draft` returned `1.80.191.120`, `1.80.191.168`, and `117.35.173.2`; current default and direct egress were both `117.35.173.2`. `src/wechat.rs` `errcode_detected` regression covers stable-egress hint.

### Cover screenshot success cannot be inferred from stale PNG existence

**Symptom:** After changing the cover template and rerunning `cover --screenshot`, the command reported success but the visible cover was still the previous PNG.

**Root cause:** Chrome headless did not reliably overwrite an existing screenshot file. The screenshot flow did not remove the old PNG first and only checked whether the path existed after running, so stale output was misread as the current result.

**Fix:** Chrome now writes to a temporary screenshot in the same directory first; after confirming the temp file was generated, it replaces the official PNG. When Chrome fails, the previous usable cover is preserved.

**Prevention:** Fixed-path generated artifacts must not prove current success only by "file exists after command". Overwrite generation should write an independent temporary artifact first, then replace the final file.

**Evidence:** `src/cover.rs` `completed_capture_replaces_stale_png_and_removes_temp_file` regression test and 2026-07-15 real `workflow` cover PNG verification.

### A complete cover source does not mean WeChat crop is usable

**Symptom:** The 900x500 original image had complete flow cards and phone mockup, but center-cropping to WeChat landscape ratio cut off bottom material entries and the phone confirmation button. Square thumbnails also cut off the left-aligned title.

**Root cause:** Cover design was laid out only for the generated canvas, without a shared safe area for WeChat list landscape image, share square, and small thumbnails.

**Fix:** The `workflow` cover title became centered, flow component height was compressed, and the three input types, MoonPub core, and phone confirmation were moved into the central landscape safe area.

**Prevention:** After changing covers, check at least the 900x500 original, central 900x383 crop, 500x500 square crop, and around 360px thumbnail. Do not use original screenshot as the only visual acceptance evidence.

**Evidence:** 2026-07-15 crop checks for `docs/MOONPUB_INTRO_ARTICLE_ZH.cover.png` and the cover visual QA constraint in `AGENTS.md`.

### Mobile WeChat typography must not depend on multi-column tables

**Symptom:** The project-introduction article looked correct in local preview, but mobile WeChat preview compressed two-column blocks such as `meta-strip` into narrow table columns. Dates, mood, and descriptive text squeezed together; reference links looked like ordinary body text.

**Root cause:** WeChat mobile readable width is small. Horizontal `<table>` works for light alignment, not body-level information blocks. Bare URLs were previously rendered as normal text with little visual weight.

**Fix:** `summary`, `callout`, `steps`, `checklist`, `key-points`, `photo-grid`, `meta-strip`, normal Markdown tables, and illustration blocks such as `comparison` / `concept-card` / `timeline` now render as vertical cards or information blocks. Explicit Markdown links and bare URLs render as bold highlighted links.

**Prevention:** Markdown / block / illustration regressions assert that mobile-sensitive blocks no longer output `<table>`. Link highlighting has regression coverage. New typography blocks should be checked with real mobile preview or at least narrow screenshots, not just desktop local HTML.

**Evidence:** Regression tests in `src/markdown/blocks.rs`, `src/markdown/plain.rs`, `src/markdown/inline.rs`, and `src/illustrate.rs`; 2026-07-15 real article rerender passed `layout-audit`, updated the same WeChat draft, and completed original / reward / comments / creation source / mobile preview.

### Input material is conservative by default; visual analysis needs explicit confirmation

**Symptom:** If photo organization uploaded pixels by default or treated model descriptions as facts, it would risk privacy leakage and content distortion.

**Root cause:** File metadata organization and visible image information recognition are two different data processing boundaries.

**Fix:** The default photo path only processes path, file name, size, and modification time. Only explicit `--analyze-images` sends a limited number of images to OpenAI. Results are written back to Inbox and marked as requiring human verification.

**Prevention:** Any capability expanding material data scope needs an independent confirmation step. AI photo descriptions must not automatically advance to WeChat draft as verified facts.

**Evidence:** `src/ai_workflow.rs`, `src/intake/photos.rs`, and `docs/FIRST_RUN_WALKTHROUGH_ZH.md`.

**Article angle:** How to turn photos into life records without letting tools invent the life.

### Release smoke must run in case-sensitive paths

**Symptom:** The first `v0.4.2` tag workflow's Linux ARM64 archive smoke failed because `Archive-Smoke.md` and `archive-smoke.md` did not match.

**Root cause:** The smoke title and later command path had different casing, relying on a case-insensitive environment.

**Fix:** The workflow now consistently uses lowercase kebab-case titles and runs credential-free smoke on the officially downloaded macOS ARM64 asset.

**Prevention:** After generating files, reuse actual paths or normalize names consistently. Treat case-sensitive environments as the release-smoke baseline.

**Evidence:** `.github/workflows/release.yml`, `docs/RELEASE_GATE_v0.4.2_ZH.md`, and `PROGRESS.md` 2026-07-13 record.

### Draft titles with spaces need normalized matching in automation

**Symptom:** In cookie mode, `ship` pushed successfully and media id was generated, but `auto_configure` reported `draft title not found`. Debug output showed the target draft first in the list.

**Root cause:** The ASCII space in the draft title was rendered by WeChat as non-breaking or multiple whitespace characters. Raw `card.innerText.includes(targetTitle)` failed. Debug output normalized `\s+` to a single space, so it looked like it should have matched.

**Fix:** `setup_editor_for_title` normalizes whitespace on both card text and target title using `replace(/\s+/g, ' ').trim()` before `includes` comparison (`src/cdp.rs`).

**Prevention:** Normalize DOM text whitespace before matching. Debug-output normalization must match decision logic, otherwise logs hide real mismatches.

**Evidence:** 2026-07-26 real `ship` full path: the title with a space normalized, selected correctly, and original / reward / comments / creation source / preview all configured successfully with media id `100011077`; stale draft was deleted automatically.

## New Record Template

```markdown
### <Symptom or failure message>

**Symptom:** <What the user saw and which path was affected.>

**Root cause:** <Verified cause.>

**Fix:** <Final code or process fix.>

**Prevention:** <Test, constraint, monitoring, or documentation entry.>

**Evidence:** <Source / test / PR / docs, without sensitive data.>

**Article angle:** <Optional, suitable public article angle.>
```

## Related Records

- `AGENTS.md`: current constraints and module boundaries to follow during development.
- `CLAUDE.md`: older detailed debugging notes kept as background; this document wins for new conclusions.
- `PROGRESS.md`: chronological record of completed features, verification, and release facts.
- `docs/WECHAT_REGRESSION_CHECKLIST_ZH.md`: real WeChat regression checklist; does not replace root-cause records.

### Standard template footer visual structure must be stable and replaceable

**Symptom:** The user reported an unwanted gray background frame, repeated personal-brand name, and the word "Official Account" in the standard template footer. The community QR code area also rendered unstably when text existed but no image was configured.

**Root cause:** The brand card used `<table>` + gray background + `border-radius`, which appeared as a visible frame in the WeChat editor. The brand description repeated the name. The `follow_image` alt text contained "Official Account". The community section display condition included only `description` / `rules` / `qrcode`, not `title` and `qrcode_note`, so text-only configuration could make the whole section disappear.

**Fix:** The brand card keeps table layout because WeChat strips flex, but removes gray background and rounded corners. The description no longer repeats the name. `follow_image` alt changed to a neutral follow label. The community section displays when any of `title` / `description` / `rules` / `qrcode_note` / `qrcode` is non-empty. `render` now warns in the terminal when the local qrcode path is unreadable.

**Prevention:** Footer visual changes must check whether WeChat strips critical style, whether brand name repeats, whether any alt/copy contains unwanted product naming, whether text-only config renders stably, and whether local QR code path readability is reported during render rather than discovered only after pushing to WeChat.

**Evidence:** `src/footer.rs` brand card and community section rendering, `src/render.rs` qrcode readability check, footer tests, and user confirmation on 2026-07-26 that the structure can be fixed and only the group QR image needs replacement later.

**Article angle:** A template footer is not better when it is richer. Stable structure, replaceable assets, and no redundant copy let users care only about the one image they need to swap.

### Obsidian plugin homepage must evolve from status string to card workspace

**Symptom:** After opening the plugin homepage, users saw a chain of `ul` / `li` and multiple `h3` sections and did not know what to click first. First-time users were likely to get lost between workspace status, current context, recommended next step, and risk boundaries.

**Root cause:** The early homepage flattened fields from `workspace --json` without information hierarchy or primary buttons. Current file and workspace overview were mixed together, and action buttons were scattered.

**Fix:** The homepage is now split into 8 card layers: current file with context kind / path / recommendation / primary action; workspace overview with CLI status and stage counts; recommended next step and first-run suggestions; available workflows; v0.4.2 evidence / gates; action entries; WeChat reach reminder; permanent help prompt. Current article, Feishu/photo results, preflight, and layout-audit workspaces now share the same `moonpub-card` + `moonpub-action-row` styling. Auxiliary dialogs also share the `moonpub-homepage` class.

**Prevention:** New plugin workspaces should reuse `moonpub-card` / `moonpub-card-title` / `moonpub-action-row`. Homepage information order should follow current file -> workspace -> next step -> workflow -> gates -> actions -> reminder -> help. Any action button must close the current modal before triggering the next step so notices are not obscured.

**Evidence:** `obsidian-plugin/main.ts` `MoonPubWorkspaceModal` / `MoonPubArticleModal` / `MoonPubIntakeResultModal` / `MoonPubPreflightModal` / `MoonPubLayoutAuditModal`, `obsidian-plugin/styles.css` card styles, and plugin docs in `obsidian-plugin/README.md` and `docs/USER_GUIDE.md`.

**Article angle:** The distance from CLI tool to regular user is often a layer of "what should I click now?"
