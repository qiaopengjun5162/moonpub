# MoonPub 首版发布检查清单

这份清单用于决定一个版本能否对外发布。目标不是证明 MoonPub 已经生产稳定，而是保证技术用户能安全试用、知道边界、遇到问题能排查。

v0.4.1 最终可发布状态见 [LAUNCH_READY_ZH.md](LAUNCH_READY_ZH.md)，GitHub Release 发布说明见 [RELEASE_NOTES_v0.4.1.md](RELEASE_NOTES_v0.4.1.md)。

## 发布定位

- [x] README 第一屏说明：MoonPub 是本地公众号发布副驾驶，不是无人值守发布机器人。
- [x] README / README_zh 明确 Beta 状态、适用人群和限制。
- [x] 浏览器自动化说明清楚：不绕过扫码、验证码、平台审核、账号权限或最终人工确认。
- [x] `push` / `ship` 文档说明草稿成功后进入 `Articles/ready/`，不是 `published/`。
- [x] Homebrew 未发布时，不出现可直接 `brew install` 的用户路径。

## 安装与版本

- [x] `Cargo.toml` 版本号已确认：`0.4.1`。
- [x] `CHANGELOG.md` 已包含本版本对用户有意义的变更。
- [x] GitHub release workflow 存在，tag 推送后应产出 macOS / Linux / Windows 资产。
- [x] README release 下载链接指向真实存在的版本和资产。
- [x] Windows 用户有 zip 下载说明。
- [x] release workflow 已补 macOS ARM64 资产目标。
- [x] release build 覆盖 `RUSTFLAGS`，避免继承本地 `target-cpu=native`。

## 本地无凭证体验

2026-06-23 已用源码构建二进制在 `/tmp/moonpub-local-check` 验证 `init` → `new` → `render` → `cover` → `check`；暂未自动打开 `preview`，避免发布检查时弹出 UI。

用一个空目录验证：

```bash
moonpub init
moonpub new "我的第一篇 MoonPub 文章"
moonpub render Articles/drafts/我的第一篇-MoonPub-文章.md
moonpub preview Articles/drafts/我的第一篇-MoonPub-文章.md
moonpub cover Articles/drafts/我的第一篇-MoonPub-文章.md --style literary
moonpub check Articles/drafts/我的第一篇-MoonPub-文章.md
```

期望结果：

- [x] 不需要微信凭证。
- [x] 生成 `.html` 和 `.draft.json`。
- [ ] `preview` 能打开本地 HTML。
- [x] `cover` 能生成 HTML；有 Chrome 时可截图。
- [x] `check` 能说明文章包是否完整。

## 微信凭证体验

详细步骤见 [WECHAT_REGRESSION_CHECKLIST_ZH.md](WECHAT_REGRESSION_CHECKLIST_ZH.md)。

真实触达微信前确认：

- [ ] `WECHAT_APPID` 已设置。
- [ ] `WECHAT_SECRET` 来自环境变量或本地 env 文件，未写进仓库。
- [ ] 本机 IP 已加入微信公众平台 IP 白名单。
- [ ] 已理解 `push` / `ship` 会调用微信 API。

建议验证顺序：

```bash
moonpub login
moonpub push Articles/drafts/文章名.md --render
moonpub configure --headed
```

期望结果：

- [ ] `login` 由用户自己扫码完成。
- [ ] `push` 创建微信草稿，写入 `.media_id`，本地文章包进入 `Articles/ready/`。
- [ ] `configure` 可见模式下能辅助配置已支持步骤。
- [ ] 微信后台仍由用户人工检查和点击发表。

## 自动化边界

- [ ] 自动化失败时不影响已经创建的微信草稿。
- [ ] 文档说明微信 UI 变化会导致步骤软失败。
- [ ] 最终发表动作默认不由 MoonPub 自动点击。
- [ ] 合集、封面图后台设置、发表按钮仍列为未完成或需人工确认。

## 质量门禁

每次发版前至少跑：

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run --all-features
pre-commit run --all-files
```

发布相关或依赖变更时加跑：

```bash
cargo audit
cargo deny check
```

## 对外材料

- [x] README / README_zh 可直接作为项目首页说明。
- [x] `docs/GETTING_STARTED.md` 能引导新用户先跑本地体验。
- [x] `docs/USER_GUIDE.md` 有完整工作流。
- [x] `docs/LAUNCH_ARTICLE_ZH.md` 有中文发布文章发布稿，并已补 v0.4.1 release 与首跑 smoke test 口径。
- [x] `docs/LAUNCH_PLAN_ZH.md` 有面向队友和早期用户的目标、进度条、下一步计划。
- [x] `docs/LAUNCH_DEMO_ASSETS_ZH.md` 记录 v0.4.1 release 二进制生成的本地预览 HTML、封面 HTML、`check` 和 `status` 输出。
- [x] `docs/LAUNCH_SCREENSHOT_CHECKLIST_ZH.md` 有截图交付物清单。
- [x] `docs/WECHAT_REGRESSION_CHECKLIST_ZH.md` 有真实微信草稿回归清单。
- [ ] 截图/录屏清单已准备：本地预览、封面、微信草稿、`configure --headed`、`status`。

## 发布后回归

- [x] 下载 v0.4.0 macOS amd64 release 资产，确认 sha256 通过。
- [x] v0.4.0 macOS amd64 release 资产可在 Apple Silicon / Rosetta 环境运行 `moonpub --help`。
- [x] 下载 v0.4.1 macOS ARM64 release 资产，确认 sha256 通过。
- [x] v0.4.1 macOS ARM64 release 资产可运行 `moonpub --help`。
- [x] v0.4.1 macOS ARM64 release 资产可运行 `moonpub --version`，输出 `moonpub 0.4.1`。
- [x] 用 v0.4.1 macOS ARM64 release 二进制跑通本地无凭证体验：`init` → `new` → `render` → `cover` → `check`。
- [x] v0.4.1 tag 初次触发 release workflow，但 macOS ARM64 build 因 `ring` + `target-cpu=native` 失败，未产出 release 资产。
- [x] 如果更新了 README 中的版本号，确认链接可下载。
- [x] 记录真实验证结果到 `PROGRESS.md`，不要把本地测试说成真实微信验证。

备注：当前开发机是 Apple Silicon。v0.4.0 macOS 资产只有 `macos-amd64`，已验证可下载、sha256 通过、`--help` 可运行，但 `--version` 不存在，且非交互 `init` 写入 `/path/to/ObsidianMain` 导致本地首跑失败；v0.4.1 已重新触发 release workflow 并成功产出 macOS ARM64/AMD64、Linux ARM64/AMD64、Windows AMD64 资产。macOS ARM64 资产已完成本地 smoke test，可作为对外主推版本。

## v0.4.1 首发剩余工作

- [x] 用 release 二进制生成本地预览 HTML。
- [x] 用 release 二进制生成封面 HTML。
- [x] 记录 `moonpub status` / `moonpub check` 的安全输出。
- [x] 用普通系统浏览器或专门截图流程导出本地预览截图。
- [x] 用普通系统浏览器或专门截图流程导出封面截图或 PNG。
- [ ] 在有真实凭证且不泄露 secret 的环境中完成微信草稿回归。
- [ ] 发布文章配图后再对外发布。

备注：2026-06-24 尝试用 Codex 内置浏览器打开本地 `file://` HTML 导出截图时，被 Browser Use URL policy 阻止；这属于当前工具安全边界，不代表 MoonPub 生成的 HTML 失败。
