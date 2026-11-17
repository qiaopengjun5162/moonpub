# MoonPub Business Model: Free Core, Optional Support

This document answers one question: **how is MoonPub sustained, and will money change what it does or where its boundaries are.**

It is not a fundraising plan or a price increase notice. It records what is already true, so users, contributors, and the maintainers themselves can judge whether this project is "selling out".

In one sentence: **the core stays free forever under MIT; support is optional appreciation that buys no features, priority, or service; and revenue will never justify hosting credentials, cloud auto-publishing, or bypassing platform rules.**

## 1. What stays free

The following are free today and will not become paid because of any business-model change:

- Every core command: render, cover, preview, check, layout-audit, preflight, draft push, blog export.
- Every input workflow: existing Markdown, Feishu Minutes, photos, and any future input source built to the same boundary.
- Every CLI / Obsidian plugin entry point and the structured JSON protocol layer.
- Every layout theme, cover style, Block template, and layout recipe.
- All documentation, onboarding guides, troubleshooting notes, and demo assets.

MoonPub is released under the MIT license. Anyone can download, modify, use, and redistribute it without permission or payment.

Core rule: **once a capability is in the free core, it cannot be moved behind a paywall.** If something might be monetized, it must be built as an add-on outside the core, and the free core must not get worse because of it.

## 2. What we will not do for money

Users hand this local tool their articles, credentials, and accounts because they trust it. These are hard boundaries and do not soften because of sponsorship or business discussions:

- **No credential hosting.** AppID / AppSecret / tokens / cookies stay on the user's machine by default. We do not upload them, because there is no such server.
- **No cloud auto-publishing.** No "hand us your account and we publish for you" service. Final publishing is always confirmed by the user in the WeChat backend.
- **No platform bypass.** Bypassing QR scans, captchas, review, account permissions, or risk controls is never a feature or a paid perk.
- **No SLA.** This is an open-source tool; uptime, response time, and dedicated support are not promised. Sponsorship does not change that.
- **No monetizing user content.** Articles, Feishu material, photos, and drafts are never collected, uploaded, or used for any data business.

These boundaries are also part of the product identity: MoonPub is a local publishing copilot, not a managed publishing platform.

## 3. What support is, and is not

If MoonPub is useful to you and you want to help it continue, support it through the channel published in the repository README / Releases.

What support is:

- **Is** voluntary appreciation and fuel for continued work, like buying a maintainer a coffee.
- **Is** one source of motivation to keep improving local rendering, WeChat compatibility, docs, and troubleshooting.
- **Is not** a purchase of any feature, theme, template, or priority.
- **Is not** a subscription, membership, or service contract; no SLA, response promise, or private support channel comes with it.
- **Does not** change anyone's right to use the core. Sponsors and non-sponsors get exactly the same experience.

In short: **support buys nothing; it just keeps the work going.** The moment support starts looking like "pay to unlock", this document has been violated.

The current support channel is whatever the repository actually publishes. Until then, no payment channel is invented, and no personal payment QR code or account is committed to the repository.

## 4. Possible optional paid layers (none exist today)

The following are **direction drafts, not promises, not a roadmap, not a countdown**. Any of them would be considered only after the core is stable long-term and none of the boundaries above are violated:

| Direction | Possible form | Boundary |
| --- | --- | --- |
| Premium templates / themes | Extra layout themes, cover styles, article structure templates | Free themes keep being maintained; paid packs are new options, not crippled free versions |
| Batch / team tools | Multi-account config, batch publish queue, review records | Still local, credentials stay local, final publish stays manual; not a cloud SaaS |
| Workflow services | Help individuals and teams set up Obsidian + MoonPub + blog + WeChat | Charge setup hours, not "ghostwriting services" or "guaranteed reach" |

Explicitly not doing:

- Turning already-free capabilities into paid ones.
- Hosting AppSecrets or running users' accounts for them.
- Selling auto-publishing, review bypass, or platform-risk evasion.
- Turning support into tiers, points, or privileges.

If any of the above is ever introduced, it will first be explained publicly in an issue — what is paid, what stays free, how the boundary is kept — and the conclusion written back into this document before any code is written. State the boundary first, then build; never the reverse.

## 5. What helps more than money

What MoonPub needs most is not money, but real usage feedback and sustainable maintenance effort. Any of the following is more valuable than a donation:

- Run it on a real article and file an issue for broken layout, awkward commands, or confusing docs.
- Add a regression test, fix a real bug, or improve a Chinese / English document.
- Contribute a theme, cover style, Block template, or layout recipe, with its use case explained.
- Share your own workflow: how Feishu notes, photos, or reading notes become articles.
- Add redacted real screenshots / recordings to first-run evidence so new users guess less.

## 6. Related documents

- For what the project actually is today: [PRODUCT_WRAP.md](PRODUCT_WRAP.md).
- For stage goals and ordering: [EXECUTION_PLAN_ZH.md](EXECUTION_PLAN_ZH.md) and [ROADMAP.md](../ROADMAP.md).
- For the Feishu route, AI Agent direction, and product split: [PRODUCT_EVALUATION_ZH.md](PRODUCT_EVALUATION_ZH.md).
- For how to start: [FIRST_RUN_WALKTHROUGH.md](FIRST_RUN_WALKTHROUGH.md).

## One-line external summary

**MoonPub is an MIT-licensed local publishing copilot. The core is free forever; support is optional and buys nothing; no credential hosting, no cloud auto-publishing, no platform bypass.**
