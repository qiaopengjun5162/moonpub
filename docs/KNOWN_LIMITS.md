# MoonPub Known Limits and Boundaries

This document is not a disclaimer. It writes down the real boundaries so that "thought it was a bug, but it was actually a design choice" feedback is reduced.

## WeChat-side limits

- No automatic final publish: `ship` / `push` / `configure` only push the article to a WeChat draft and assist with backend configuration. The final publish button must be clicked by the user.
- Does not bypass QR scan, captcha, or review: `moonpub login` saves a browser session, but sessions expire and WeChat may require re-scanning or verification.
- Browser automation may soft-fail when the WeChat backend UI changes: original statement, reward, comments, creation source, and preview-send steps are designed to soft-fail without blocking the main draft-creation flow.
- IP whitelist (AppID / AppSecret mode): the current public egress IP must be added to the WeChat Official Account IP whitelist; proxy / mobile network egress drift causes `invalid ip` errors.
- Cover / body image format: WeChat is sensitive to image MIME, dimensions, and content; non-real images, oversized or undersized files may be rejected.

## Plugin-side limits

- The Obsidian plugin is only a CLI entry point, not an independent publisher; the MoonPub CLI must be installed locally and its executable path configured.
- The plugin does not automatically sync Obsidian settings to the CLI config; `Articles root directory` and `WeChat preview recipient` must be filled in the plugin settings separately.
- The Community Marketplace release is pending; BRAT or manual install is recommended for now.

## Local-side limits

- AI writing is not the core path: `write` / `expand` / `polish` / `ship --ai` require a DeepSeek / OpenAI API key, and the result must be reviewed by a human.
- Photo visual analysis (`--analyze-images`) needs human verification: image analysis results are written back to Inbox with a "needs human verification" label and do not automatically advance to a WeChat draft.
- Markdown rendering is constrained by the WeChat editor: WeChat strips some HTML tags and styles; `layout-audit` helps check but cannot cover all editor behaviors.

## Feedback

When you hit a problem:

1. Run `moonpub doctor` and `moonpub preflight <article.md>` first (local, does not reach WeChat).
2. Check `docs/ENGINEERING_LESSONS.md` (English) / `docs/ENGINEERING_LESSONS_ZH.md` (Chinese) for a recorded similar issue.
3. Still have questions? Open a GitHub Discussion or Issue.
