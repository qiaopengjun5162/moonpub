# MoonPub First-Run Walkthrough

This document answers one question only:

**If I am using MoonPub for the first time, what is the least-effort, least-confusing order to try?**

The goal is not to cover all commands, but to give first-time users a short path they can follow.

## Remember one principle first

MoonPub is currently best understood not as an "auto-publishing robot", but as:

**A local-first content publishing copilot.**

Its first value is not "fewer clicks to publish", but:

**Turn scattered material into local Markdown assets first, then move into editable drafts and safe publishing.**

So the three entry points — existing articles, Feishu, and photos — all follow the same rhythm:

1. Raw material enters Inbox or an existing Markdown file first
2. AI only helps organize it into an editable draft; it does not finalize the draft for you
3. Review the local HTML preview first, then decide whether to push to a WeChat draft

Therefore, on first use, the most important thing is not publishing directly, but confirming three things:

1. Can you smoothly enter a workflow?
2. Can you understand the generated draft and preview?
3. Can you distinguish "local preview" from "WeChat draft publishing"?

## Recommended first-run order

For the first run, follow this order:

1. Open the plugin homepage to see the overall workspace state
2. Pick a material entry point: Feishu, photos, or existing Markdown
3. Stop at the draft and local preview first; do not push to WeChat immediately
4. After understanding the result page and next actions, enter the real publishing pipeline

## Step 0: Prepare the environment

Prepare at least these three things:

1. Your machine can run `moonpub --help`
2. The Obsidian plugin is installed and enabled
3. The plugin settings have the `Articles root directory` filled in

If you have not installed the plugin yet, read:

- [../obsidian-plugin/README.md](../obsidian-plugin/README.md)

## Step 1: Open the plugin homepage first

In Obsidian run:

- `Open MoonPub Homepage`

This step is now more than a status query; it is starting to play the role of the plugin homepage.

You will see cards from top to bottom:

- **Current file**: what you currently have open in Obsidian, which entry fits it, and primary actions
- **Workspace overview**: CLI / config status, total article count, `drafts / ready / published` stage counts
- **Recommended next step + first-run suggestions**: next command and ordering advice based on current context
- **Available workflows**: entry buttons for current article, Feishu, photos, and WeChat draft boundary
- **Layout themes**: highlighted themes such as geek-black, blueprint, and AI lab, plus theme groups, the full recipe guide, and copy actions for article `theme` frontmatter / cover commands
- **v0.4.2 evidence / gates**: release evidence and release gate status
- **Action entries**: import Feishu, import photos, view WeChat draft boundary
- **Reach WeChat**: reminders about networking / controlling Chrome
- **Not sure where to start?**: permanent help prompt for first-time users

Current quick-entry buttons include:

- `Check current article`
- `Preview current article`
- `Import latest Feishu Minutes`
- `Import current image directory`
- `View WeChat draft boundary`

If `doctor` reports warnings or configuration is incomplete, the `Current file` card also shows `Open plugin settings`, which jumps directly to fill in `MoonPub executable path`, `Articles root directory`, and `WeChat preview recipient`.

If you are using MoonPub for the first time, always start here instead of memorizing commands first.

## Step 2: Choose an entry point based on your content source

### Entry A: You already have an existing Markdown article

This fits you if you have already written the article and mainly want to check typography and publish.

Recommended order:

1. Open that article
2. Click `Check current article`
3. Click `Preview current article`
4. After understanding the current-article workspace, decide whether to push to WeChat

### Entry B: You start from Feishu Minutes

This fits you if what you have right now is a transcript, voice record, or fragmented material.

Recommended order:

1. Click `Import latest Feishu Minutes` on the plugin homepage
2. In the confirmation dialog, verify that "the full transcript will be sent to the currently configured AI provider", then confirm
3. Wait for the plugin to generate the draft and the result workspace
4. Look at the result workspace:
   - Inbox
   - Draft
   - HTML preview
   - Recommended next step
5. First click `Open draft` or `Preview draft`
6. After confirming the content is fine, consider `Push to WeChat draft`

On first use, it is not recommended to take the Feishu fast-push path.

### Entry C: You start from life photos

This fits you if you want to turn a set of photos into a draft instead of leaving them in your phone.

Recommended order:

1. Open one image in Obsidian first
2. Click `Import current image directory` on the plugin homepage
3. In the confirmation dialog, verify the directory and data boundary: only file path, file name, size, and modification time are sent to AI; image pixels are not uploaded
4. Wait for the plugin to generate the photo draft and the result workspace
5. First click `Open draft` or `Preview draft`
6. After confirming the material is organized as expected, continue to the next step

If you need visible information from the photos themselves, not just file information, use the command-palette command `Analyze current image directory and generate photo draft preview` instead. It pops a second confirmation dialog clearly stating that jpg/jpeg/png/webp images will be sent to OpenAI; the generated image descriptions still need to be manually verified in the draft.

## Step 3: Stop locally first, do not rush to WeChat

On first use, set your goal here:

**Successfully see the draft and the local HTML preview.**

Once you reach this point, you have completed the most important part of the first-run experience.

Because it means:

- The entry point works
- Material can enter the system
- A draft can be generated
- Preview can be opened
- You are starting to understand MoonPub's rhythm

## Step 4: Enter real publishing on the second run

After you are used to the "material -> draft -> preview" rhythm on the first run, then start entering:

- `push --render`
- `ship`
- `configure`
- WeChat backend preview

In other words:

- First-run goal: understand the flow
- Second-run goal: real publishing

## If you only remember one shortest suggestion

On first use, just remember these four steps:

1. Open `Open MoonPub Homepage`
2. From the homepage pick Feishu, photos, or current article
3. Stop at the draft and local preview first
4. After understanding the result page, enter WeChat publishing

## What to read next

After finishing the first-run experience, continue as needed:

- Formal workflow paths: [RECOMMENDED_WORKFLOWS.md](RECOMMENDED_WORKFLOWS.md) (English) / [RECOMMENDED_WORKFLOWS_ZH.md](RECOMMENDED_WORKFLOWS_ZH.md) (Chinese)
- Product framing: [PRODUCT_WRAP.md](PRODUCT_WRAP.md) (English) / [PRODUCT_WRAP_ZH.md](PRODUCT_WRAP_ZH.md) (Chinese)
- Stage plan: [EXECUTION_PLAN.md](EXECUTION_PLAN.md) (English) / [EXECUTION_PLAN_ZH.md](EXECUTION_PLAN_ZH.md) (Chinese)
- Plugin details: [../obsidian-plugin/README.md](../obsidian-plugin/README.md)
- Full user guide: [USER_GUIDE.md](USER_GUIDE.md)
