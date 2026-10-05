# MoonPub Recommended Workflows

This document answers one question only:

**If I am using MoonPub for the first time, which path should I take now?**

Do not start by reading all commands. The current recommendation is: if you work in Obsidian, open the `MoonPub homepage workspace` first, then choose one of the three main paths below based on your content source.

If you want not a full workflow description but "what exactly should I click or run first so I don't get lost", see [FIRST_RUN_WALKTHROUGH_ZH.md](FIRST_RUN_WALKTHROUGH_ZH.md) (Chinese).

MoonPub's workflow main line can be summarized in one sentence:

**Material enters local Markdown first, drafts are confirmed by you, and WeChat publishing is advanced last.**

This is why Feishu and photo flows both stop at `--draft --preview` by default, instead of pushing directly. This lets users keep the original material, see the draft path, revise and add images repeatedly, and only then explicitly move to a WeChat draft.

## Decide which kind of user you are

### Path A: You already have a Markdown article

This path fits you if:

- You already write in Obsidian / Markdown
- You already have a complete draft
- You mainly want to format the article and publish it to WeChat

### Path B: You start from Feishu Minutes / voice records

This path fits you if:

- Your content is not yet an article, just a transcript
- You want to organize the material into a draft first
- You want to review the content before publishing to WeChat

### Path C: You start from life photos

This path fits you if:

- You already have a set of photos on your phone you want to keep
- You want to organize the photos into a draft instead of leaving them deep in the album
- You want to confirm the recording style before deciding whether to publish to WeChat

---

## Path A: Existing article → Local preview → WeChat draft

This is the most basic, stable, and easy-to-understand main path.

### Step 1: Create or prepare the article

```bash
moonpub new "My first article"
```

Or use an existing Markdown file directly.

### Step 2: Render and preview locally first

```bash
moonpub render Articles/drafts/My-first-article.md
moonpub preview Articles/drafts/My-first-article.md
```

This step only processes local files and does not need WeChat credentials.

Check:

- Whether the title, summary, and paragraphs look right
- Whether the typography theme fits
- Whether images, blockquotes, and code blocks render correctly

### Step 3: Generate a cover if needed

```bash
moonpub cover Articles/drafts/My-first-article.md --style literary --screenshot
```

### Step 4: Push to a WeChat draft

```bash
moonpub push Articles/drafts/My-first-article.md --render
```

Prerequisites:

- `WECHAT_APPID` is configured
- `WECHAT_SECRET` is configured
- The current machine IP is on the WeChat whitelist

### Step 5: Assisted backend configuration and preview send

```bash
moonpub configure
```

It tries to complete:

- Original statement
- Reward
- Comments
- Creation source
- Mobile preview send

### Step 6: Manual confirmation and publish in the WeChat backend

MoonPub's product boundary is always:

- Help you push content to a "publishable state"
- Not replace your final publish confirmation

---

## Path B: Feishu Minutes → Draft → Preview → WeChat draft

This is currently the second main path worth promoting.

It is not "publish directly", but:

**Organize the material into a draft first, then let you decide whether to continue.**

### Recommended default: conservative path

```bash
moonpub --articles "<Obsidian path>" intake feishu --latest --draft --preview
```

Or:

```bash
moonpub --articles "<Obsidian path>" intake feishu --minute-token <token> --draft --preview
```

This path does four things:

1. Import the Feishu Minutes record into `Inbox/Feishu/`
2. Generate an editable article draft
3. Render a local HTML preview
4. Stop and wait for your review

### What to do next

After the draft is generated, prioritize these three follow-up actions:

- `edit_path`: continue editing, add images, reduce AI flavor
- `preview_command`: continue checking local typography
- `push_command`: push to WeChat draft after confirming

Recommended order:

1. Edit first
2. Check local preview
3. Push to WeChat draft last

### Fast mode: only when you clearly want to fast-forward

```bash
moonpub --articles "<Obsidian path>" intake feishu --latest --draft --push
```

Only use this when:

- You are already familiar with this pipeline
- You confirm this content does not need multiple rounds of revision
- You are willing to enter the WeChat draft and review there

### After the Feishu path enters the WeChat draft

The second half is exactly the same as a normal article:

```bash
moonpub configure
```

Then:

- Mobile preview
- Manual backend confirmation
- Publish

---

## Path C: Photos → Draft → Preview → WeChat draft

This path, like Feishu, is:

**Organize the material into a draft first, then let you decide whether to continue.**

### Recommended default: conservative path

```bash
moonpub --articles "<Obsidian path>" intake photos photos/day1 --draft --preview
```

Or:

```bash
moonpub --articles "<Obsidian path>" intake photos photos/day1 photos/day2 --draft --preview
```

If you really need to extract visible information from the photos themselves, not just organize file metadata, explicitly opt in after confirming the outbound scope:

```bash
moonpub --articles "<Obsidian path>" intake photos photos/day1 --analyze-images --draft --preview
```

`--analyze-images` only supports OpenAI, uploads up to 5 jpg/jpeg/png/webp images, 8 MiB per image, 20 MiB total. Results are treated only as "needs human verification" auxiliary information in Inbox and do not automatically advance to a WeChat draft.

This path does four things:

1. Archive a set of real photos into `Inbox/Photos/`
2. Generate an editable article draft
3. Render a local HTML preview
4. Stop and wait for your review

### What to do next

After the photo draft is generated, the recommended order is similar to Feishu:

1. First check whether the draft faithfully records the set of photos
2. Then check the local preview
3. Finally decide whether to push to a WeChat draft

### When this path fits

Typical scenarios:

- A set of photos after running, walking, or traveling
- Want to keep "what happened this day" as a draft first
- Don't want the photos to end up only in the album

---

## Which path to use when

### Use Path A if you already have a complete article

Typical scenarios:

- You wrote the article in Obsidian
- You already organized the structure locally
- You mainly care about typography and publishing

### Use Path B if you only have material right now

Typical scenarios:

- Voice thoughts after a run
- Feishu Minutes transcript
- Temporarily recorded ideas
- Meeting / walking / reading fragments

### Use Path C if you mainly want to organize photos

Typical scenarios:

- A set of life photos from the same day on your phone
- Want to first record the photos based on real information
- Add feelings and context later

---

## Currently recommended first-time order

If you are using MoonPub for the first time, try in this order:

1. Run the local-only part of Path A first to confirm you like the generated article and typography
2. If you installed the plugin, open the `MoonPub homepage workspace`
3. Then try the Feishu path to confirm you accept the "material → draft → preview → push" rhythm
4. If you have life photo material, try the photo path
5. Finally enter real WeChat official account publishing

Reasons:

- Path A is the easiest to understand
- Path B best demonstrates future product potential
- Path C shows that MoonPub already has a multi-source shape
- The plugin homepage is starting to play the unified entry role
- After all three are working, you can understand MoonPub's product shape more completely

---

## Remember this product principle

MoonPub is currently not an "auto-publishing robot".

The most reasonable way to understand it now is:

**A local-first content publishing copilot.**

It is good at:

- Organizing content for you
- Generating drafts
- Completing typography
- Pushing to WeChat drafts
- Reducing repeated backend clicks

But whether the content is suitable and whether to publish is still confirmed by you.
