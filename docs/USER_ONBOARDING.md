# MoonPub User Onboarding Guide

> Goal: understand what MoonPub does in 10 minutes, then walk through your first path in 30 minutes.
>
> If you want to know "where do I click first", see [FIRST_RUN_WALKTHROUGH.md](FIRST_RUN_WALKTHROUGH.md) (English) / [FIRST_RUN_WALKTHROUGH_ZH.md](FIRST_RUN_WALKTHROUGH_ZH.md) (Chinese). If you want the product framing, see [PRODUCT_WRAP.md](PRODUCT_WRAP.md) (English) / [PRODUCT_WRAP_ZH.md](PRODUCT_WRAP_ZH.md) (Chinese).

## In one sentence

MoonPub is a **local-first publishing copilot**. It turns scattered material into editable drafts, lets you preview and refine locally, and only then explicitly pushes to a WeChat Official Account draft.

It is not an unattended publishing bot, and it does not confirm the final publish step for you.

## Step 1: Install the CLI

Download the `v0.4.2` release binary:

**macOS Apple Silicon:**

```bash
curl -L https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.2/moonpub-macos-arm64.tar.gz | tar xz
sudo mv moonpub /usr/local/bin/
```

**macOS Intel:** use `moonpub-macos-amd64.tar.gz`.
**Linux:** use `moonpub-linux-amd64.tar.gz` or `moonpub-linux-arm64.tar.gz`.
**Windows:** download the `.zip` from [Releases](https://github.com/qiaopengjun5162/moonpub/releases), extract `moonpub.exe`, and add it to PATH.

Or install with Cargo:

```bash
cargo install --git https://github.com/qiaopengjun5162/moonpub
```

Verify:

```bash
moonpub --version
```

## Step 2: Initialize your articles directory

Enter your Obsidian vault or any directory where you want to keep articles:

```bash
moonpub init
```

This creates a sample `moonpub.toml`. Open it and set your real articles root:

```toml
[articles]
root = "/Users/yourname/Obsidian/Articles"

[wechat]
appid = "wx..."
author = "Your Official Account author name"
theme = "geek"

[ai]
provider = "deepseek"
```

If you do not have a WeChat Official Account yet, leave `appid` blank; local rendering and preview still work.
For technical posts, keep `geek`, set `geek-black` for terminal-like notes, `blueprint` for system design, or `ai-lab` for AI experiments.
If you are unsure which theme to choose, run `moonpub layout-recipes`; it lists theme groups, highlighted themes, and copyable layout recipes.

## Step 3: Create and render your first article

```bash
moonpub new "My First Article"
```

This creates `My First Article.md` under `Articles/drafts/` with a frontmatter template. Edit it, write some body text, then render:

```bash
moonpub render Articles/drafts/My-First-Article.md
moonpub preview Articles/drafts/My-First-Article.md
```

`render` generates two files:

- `Articles/drafts/My-First-Article.html` for local preview
- `Articles/drafts/My-First-Article.draft.json` for WeChat draft push

`preview` opens the rendered HTML in your browser so you can check layout before touching WeChat.

## Step 4: Generate a cover

```bash
moonpub cover Articles/drafts/My-First-Article.md --style literary
```

This creates `Articles/drafts/My-First-Article.cover.html`. Add `--screenshot` to capture a cover image with Chrome.

## Step 5: Push to a WeChat draft (optional)

If you have WeChat credentials:

```bash
export WECHAT_APPID=wx***
export WECHAT_SECRET=your_secret
moonpub push Articles/drafts/My-First-Article.md --render
```

If you want to avoid the IP whitelist, set `auth_method = "cookie"` in `moonpub.toml` or `WECHAT_AUTH_METHOD=cookie`. Then `push` / `ship` will reuse the browser session from `moonpub login` instead of requiring `WECHAT_APPID` / `WECHAT_SECRET` and an IP whitelist.

After a successful push, the article bundle moves to `Articles/ready/` and a `.media_id` file is written.

## Step 6: Assisted backend configuration (optional)

Once the draft is in the WeChat backend, configure original statement, reward, comments, and creation source:

```bash
moonpub login       # one-time QR scan
moonpub configure   # auto-configure backend and send mobile preview
```

`configure` sends a mobile preview by default. For the first run, provide the recipient WeChat ID:

```bash
moonpub configure --to your_wxid
```

After the first success, the recipient is remembered.

## One-shot full flow

Once you are comfortable:

```bash
moonpub ship Articles/drafts/My-First-Article.md --style literary
```

`ship` does: cover → render → push to WeChat draft → backend configuration → mobile preview send. Final publishing still requires you to confirm in the WeChat backend.

## Start from Feishu Minutes

If you only have a Feishu Minutes transcript, not a full article:

```bash
moonpub --articles "/Users/yourname/Obsidian" intake feishu --latest --draft --preview
```

This will:

1. Pull the latest Feishu Minutes record
2. Save it to `Inbox/Feishu/`
3. Generate an editable draft
4. Render a local HTML preview
5. Stop and wait for you to review

After reviewing the draft, continue manually:

```bash
moonpub push Articles/drafts/<draft-filename>.md --render
moonpub configure
```

If you are already familiar with the flow and want to fast-forward:

```bash
moonpub --articles "/Users/yourname/Obsidian" intake feishu --latest --draft --push
```

## Start from photos

If you have opened an image in Obsidian and want to turn a directory of photos into a draft:

```bash
moonpub --articles "/Users/yourname/Obsidian" intake photos photos/day1 --draft --preview
```

This will:

1. Archive the photos into `Inbox/Photos/`
2. Generate a factual material draft based on file metadata
3. Render a local HTML preview
4. Stop and wait for you to review

If you need visible information from the photos themselves (text, scenes), explicitly opt in:

```bash
moonpub --articles "/Users/yourname/Obsidian" intake photos photos/day1 --analyze-images --draft --preview
```

Limits: OpenAI provider only, up to 5 jpg/jpeg/png/webp images, 8 MiB per image, 20 MiB total. Analysis results are marked "needs human verification" and are not treated as fact.

## Use the Obsidian plugin

1. Install and enable BRAT in Obsidian.
2. Add the beta plugin: `https://github.com/qiaopengjun5162/moonpub`
3. Enable the `MoonPub` plugin.
4. Fill in plugin settings:
   - `MoonPub executable path`
   - `Articles root directory`
   - `WeChat preview recipient` (optional)

Click the MoonPub ribbon icon to open the homepage workspace. It shows:

- Current file context and recommended entry path
- Workspace article counts
- Recommended next step
- Entry points for Feishu, photos, and current article

Common commands:

- `Check current article status`: runs `check` on the current Markdown
- `Preview article`: runs `preview` on the current file
- `Import latest Feishu Minutes and draft preview`: stops at draft + local preview
- `Publish to WeChat Official Account`: runs `ship` on the current file

## Local checks before publishing

Before touching the WeChat API, run these local-only commands:

```bash
moonpub doctor                          # local environment diagnostics
moonpub check Articles/drafts/article.md # article bundle status
moonpub preflight Articles/drafts/article.md # publish quality gate
moonpub layout-audit article.html       # HTML layout risk check
```

None of these call the WeChat API or control Chrome.

## Safety boundaries

- **No automatic final publish**: `ship` only brings the article to a state where you can manually confirm in the WeChat backend.
- **No login bypass**: the browser session saved by `moonpub login` expires; you may need to scan the QR code again.
- **Browser automation may soft-fail**: when the WeChat backend UI changes, original / reward / comment settings may fail, but draft push is not blocked.
- **IP whitelist**: in AppID/AppSecret mode, your current public IP must be on the WeChat IP whitelist. If you get a new IP error after adding one yesterday, fix your proxy or network egress first, then add the `current IP` from the error.
- **AI output must be reviewed**: content generated by `write` / `expand` / `polish` / `ship --ai` must be checked by a human.
- **Photo analysis must be verified**: `--analyze-images` results are written back to Inbox with a "needs human verification" label.

## Common command cheat sheet

| Command | Purpose |
|--------|---------|
| `moonpub init` | Initialize config |
| `moonpub new "Title"` | Scaffold article template |
| `moonpub render article.md` | Render WeChat HTML |
| `moonpub preview article.md` | Open local preview |
| `moonpub cover article.md --style literary` | Generate cover |
| `moonpub check article.md` | Check article status |
| `moonpub preflight article.md` | Pre-publish quality gate |
| `moonpub push article.md --render` | Push to WeChat draft |
| `moonpub ship article.md` | One-shot full flow |
| `moonpub login` | QR login |
| `moonpub configure` | Configure backend and send preview |
| `moonpub wechat-health` | Check browser automation health |
| `moonpub doctor` | Local diagnostics |

## AI Features and Common Questions

### AI Features

The following commands need your own AI provider key. By default they read `DEEPSEEK_API_KEY`. Set `provider = "openai"` in `moonpub.toml` to read `OPENAI_API_KEY` instead:

```bash
moonpub write "Reading notes on To Live"          # turn an idea into an article
moonpub expand Articles/drafts/notes.md            # expand notes into an article
moonpub polish Articles/drafts/draft.md            # polish the draft
moonpub ship Articles/drafts/article.md --ai      # polish then publish
```

AI-generated content must be reviewed by a human before publishing.

### Common Questions

**Q: I get `invalid ip` when pushing.**

Go to the WeChat Official Account backend → Basic Configuration → IP whitelist and add the IP shown in the error message. If you added an IP yesterday and see a new one today, first fix a stable network egress or disable a proxy that rotates nodes, then add the `current IP` from this error. Do not keep accumulating historical addresses.

**Q: A browser automation step failed.**

The WeChat editor is a live web app and the UI changes occasionally. Steps such as original statement, reward, or comments are designed to soft-fail and do not block the main draft push. Add `--headed` to see which step is stuck:

```bash
moonpub configure --headed
```

**Q: Chrome is not found.**

Make sure Chrome or Chromium is installed. macOS usually has it by default. On Linux install `chromium-browser`, on Windows MoonPub searches Program Files automatically.

**Q: The rendered article style looks wrong.**

- Do not use `<style>` tags; the WeChat editor strips them.
- Use `moonpub render`, which writes styles as inline CSS.
- Check that `:::blockname` is spelled correctly and matches the case in the template.

## Next steps

- Three workflow paths: [RECOMMENDED_WORKFLOWS.md](RECOMMENDED_WORKFLOWS.md) (English) / [RECOMMENDED_WORKFLOWS_ZH.md](RECOMMENDED_WORKFLOWS_ZH.md) (Chinese)
- First-run plugin experience: [FIRST_RUN_WALKTHROUGH.md](FIRST_RUN_WALKTHROUGH.md) (English) / [FIRST_RUN_WALKTHROUGH_ZH.md](FIRST_RUN_WALKTHROUGH_ZH.md) (Chinese)
- Product framing: [PRODUCT_WRAP.md](PRODUCT_WRAP.md) (English) / [PRODUCT_WRAP_ZH.md](PRODUCT_WRAP_ZH.md) (Chinese)
- Stage plan: [EXECUTION_PLAN.md](EXECUTION_PLAN.md) (English) / [EXECUTION_PLAN_ZH.md](EXECUTION_PLAN_ZH.md) (Chinese)
- Full command reference: [USER_GUIDE.md](USER_GUIDE.md)
- Troubleshooting: [ENGINEERING_LESSONS.md](ENGINEERING_LESSONS.md) (English) / [ENGINEERING_LESSONS_ZH.md](ENGINEERING_LESSONS_ZH.md) (Chinese)
- Known limits: [KNOWN_LIMITS.md](KNOWN_LIMITS.md) (English) / [KNOWN_LIMITS_ZH.md](KNOWN_LIMITS_ZH.md) (Chinese)
