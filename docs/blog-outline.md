# Announcing MoonPub: Markdown → WeChat, Fully Automated

## Hook

You just finished writing an article in your favorite Markdown editor. Now you need to:
copy text → open WeChat backend → paste → reformat → upload images → set originality →
enable tips → configure comments → set creation source → preview → cross-post to blog.

That's 30+ clicks. Every. Single. Time.

MoonPub does it in one command: `moonpub ship article.md`

## What is MoonPub

A pure Rust CLI that automates the entire Markdown-to-WeChat publishing pipeline.
No AI dependencies. No third-party APIs (except WeChat's own). Open source, MIT.

Five stages, one command:

1. **cover** — Generate a cover card from your frontmatter (6 styles, or use your own image)
2. **render** — Convert Markdown to WeChat-compatible HTML with 12 built-in Block templates
3. **push** — Upload to WeChat drafts via the native API
4. **configure** — Headless Chrome automates: originality declaration, tips, comments, creation source, preview
5. **export** — Sync to your Zola blog

## Why I Built It

I write on WeChat Official Account. The editing experience is terrible:
- No Markdown support in the editor
- 30+ manual clicks per article just for settings
- No way to batch export to my blog

I looked at existing tools — they either charge money, call AI APIs I don't trust, or only solve one piece of the puzzle.

So I built the whole pipeline in Rust. Zero runtime dependencies except Chrome for CDP automation.

## Coolest Feature: Headless CDP

The hard part was WeChat's draft settings panel. There's no API for originality, tips, comments, or creation source. You have to click through a Vue.js web interface.

MoonPub uses Chrome DevTools Protocol to click the right buttons at the right time — fully headless. Scan a QR code once to save cookies, then everything runs in the background.

Debug mode (`--headed`) opens a visible browser with screenshots when things go wrong.

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

# Ship it
moonpub ship my-article.md --style gradient
```

## What's Next

- Pre-built binaries for macOS and Linux
- Homebrew formula
- More Block templates
- AI-assisted title suggestions (optional, user's own API key)

## Links

- GitHub: https://github.com/qiaopengjun5162/moonpub
- MIT License. Contributions welcome.
