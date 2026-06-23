# Announcing MoonPub: A Local Publishing Copilot for WeChat

## Hook

You just finished writing an article in your favorite Markdown editor. Now you need to:
copy text → open WeChat backend → paste → reformat → upload images → set originality →
enable tips → configure comments → set creation source → preview → cross-post to blog.

That's 30+ clicks. Every. Single. Time.

MoonPub turns that into an assisted local workflow: `moonpub ship article.md`

## What is MoonPub

A pure Rust CLI that turns Markdown into a WeChat-ready draft, then helps with the repetitive backend settings through local browser automation. No required AI dependency. No third-party publishing SaaS. Open source, MIT.

Five stages, one command:

1. **cover** — Generate a cover card from your frontmatter (10 styles, or use your own image)
2. **render** — Convert Markdown to WeChat-compatible HTML with 12 built-in Block templates
3. **push** — Upload to WeChat drafts via the native API
4. **configure** — Headless Chrome automates: originality declaration, tips, comments, creation source, preview
5. **export** — Sync to your Zola blog

Final publishing remains a human confirmation step in the WeChat backend.

## Why I Built It

I write on WeChat Official Account. The editing experience is terrible:
- No Markdown support in the editor
- 30+ manual clicks per article just for settings
- No way to batch export to my blog

I looked at existing tools — they either charge money, call AI APIs I don't trust, or only solve one piece of the puzzle.

So I built the whole pipeline in Rust. Zero runtime dependencies except Chrome for CDP automation.

## Coolest Feature: Local CDP Assistance

The hard part was WeChat's draft settings panel. There's no API for originality, tips, comments, or creation source. You have to click through a Vue.js web interface.

MoonPub uses Chrome DevTools Protocol to click the repetitive controls at the right time. Scan a QR code yourself once to save the browser session, then let MoonPub assist with the boring parts.

Debug mode (`--headed`) opens a visible browser with screenshots when things go wrong.

What it does not do:

- It does not bypass QR login or captcha.
- It does not bypass WeChat review or account permissions.
- It does not replace the final human publish decision.

## How to Get Started

```bash
# Install
cargo install --git https://github.com/qiaopengjun5162/moonpub
# Or Docker
docker build -t moonpub https://github.com/qiaopengjun5162/moonpub.git

# Configure
export WECHAT_APPID=wx***
export WECHAT_SECRET=***
moonpub init

# First time: scan QR
moonpub login

# Ship it to a ready-to-review draft
moonpub ship my-article.md --style gradient
```

## What's Next

- Better first-run guided setup
- A safer assisted mode for browser configuration
- More Block templates
- More real article fixtures for layout regression checks

## Links

- GitHub: https://github.com/qiaopengjun5162/moonpub
- MIT License. Contributions welcome.
