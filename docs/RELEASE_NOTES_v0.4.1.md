# MoonPub v0.4.1

MoonPub is a local publishing copilot for WeChat Official Accounts:

```text
Markdown / Obsidian
  -> local WeChat-style HTML preview
  -> cover generation
  -> WeChat draft creation through the official API
  -> assisted backend configuration in local Chrome
  -> optional Zola blog export
```

This release is **Beta / early adopter ready**. It is suitable for technical users who can configure WeChat Official Account credentials and are comfortable reviewing generated drafts before publishing.

MoonPub is **not** an unattended publishing bot. It does not bypass QR login, captcha, platform review, account permissions, or final human confirmation.

## Install

macOS Apple Silicon:

```bash
curl -L https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.1/moonpub-macos-arm64.tar.gz | tar xz
sudo mv moonpub /usr/local/bin/
moonpub --version
```

macOS x86_64:

```bash
curl -L https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.1/moonpub-macos-amd64.tar.gz | tar xz
sudo mv moonpub /usr/local/bin/
moonpub --version
```

Linux x86_64:

```bash
curl -L https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.1/moonpub-linux-amd64.tar.gz | tar xz
sudo mv moonpub /usr/local/bin/
moonpub --version
```

Linux ARM64:

```bash
curl -L https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.1/moonpub-linux-arm64.tar.gz | tar xz
sudo mv moonpub /usr/local/bin/
moonpub --version
```

Windows users can download `moonpub-windows-amd64.zip`, unzip `moonpub.exe`, and add it to `PATH`.

Cargo install is also supported:

```bash
cargo install --git https://github.com/qiaopengjun5162/moonpub
```

Homebrew tap is not published yet.

## Try Locally First

This path does not require WeChat credentials:

```bash
moonpub init
moonpub new "My First MoonPub Article"
moonpub render "Articles/drafts/My-First-MoonPub-Article.md"
moonpub preview "Articles/drafts/My-First-MoonPub-Article.md"
moonpub cover "Articles/drafts/My-First-MoonPub-Article.md" --style literary
moonpub check "Articles/drafts/My-First-MoonPub-Article.md"
```

Use the exact path printed by `moonpub new` if your title contains spaces or non-ASCII characters.

## Demo Screenshots

The README and launch article include screenshots generated from the v0.4.1 release binary without WeChat credentials:

- Rendered article preview: `docs/assets/launch/01-preview.png`
- Literary cover card: `docs/assets/launch/02-cover.png`

For the full launch narrative, see `docs/LAUNCH_ARTICLE_ZH.md`.

## Push To WeChat Drafts

After configuring credentials and IP allowlist:

```bash
export WECHAT_APPID=wx***
export WECHAT_SECRET=your_secret

moonpub login
moonpub push "Articles/drafts/article.md" --render
```

Successful draft creation moves the local article bundle to `Articles/ready/`. It is not marked as `published` until you manually confirm publication or run `moonpub mark-published`.

## What Is Included

- `init`, `new`, `status`, `check`
- Markdown rendering to WeChat-friendly HTML and draft JSON
- Local preview and 10 built-in cover styles
- Official WeChat API draft create/update/delete/list
- Local image upload and replacement during push
- Assisted Chrome/CDP backend configuration for supported steps
- Zola blog export
- Optional DeepSeek-powered `write`, `expand`, `polish`, and `ship --ai`
- `radar` title/trend helper commands

## Verified

- GitHub release assets exist for macOS ARM64, macOS x86_64, Linux ARM64, Linux x86_64, and Windows x86_64.
- macOS ARM64 release binary passed sha256 verification.
- macOS ARM64 release binary passed `--help`, `--version`, and a no-credential first-run smoke test:

```text
init -> new -> render -> cover -> check
```

- Local preview and cover screenshots were generated from the release demo HTML with Chrome headless and committed under `docs/assets/launch/`.

## Known Limits

- Live WeChat regression still requires real credentials, IP allowlist, and QR-code login.
- Browser automation depends on WeChat's live backend UI and may soft-fail when WeChat changes DOM or wording.
- Collection selection, backend cover setting, and final publish button are not enabled as unattended actions.
- Real WeChat draft and `configure --headed` screenshots still require a live WeChat account environment; do not capture AppSecret, access tokens, phone numbers, or private account data.

## Useful Docs

- Quick start: `docs/GETTING_STARTED.md`
- Full user guide: `docs/USER_GUIDE.md`
- Launch plan and progress: `docs/LAUNCH_PLAN_ZH.md`
- Real WeChat regression checklist: `docs/WECHAT_REGRESSION_CHECKLIST_ZH.md`
