# MoonPub v0.4.4 Release Gate

这份文档只回答一个问题：

**v0.4.4 什么时候可以发布。**

v0.4.4 不是功能版本，而是一次维护收口：把并行在途改动合并、替换过期群二维码、并修复 `release-check` 指向的发布门文档版本漂移。它没有引入新的平台或用户可见功能。

## 发布判断

当前源码版本为 v0.4.4（`Cargo.toml` 的 `version`）。相比 v0.4.3，v0.4.4 的主要变化是：

- 群二维码过期，已用新图覆盖 `moonpub-data/qrcode.png`（旧图备份为 `qrcode.backup-20261018.png`），`moonpub.toml` 的 `[footer] qrcode` 配置无需改动。
- 并行 codex 会话的在途改动已整体收口（约 68 个文件，提交于 `a022fdb`），main 测试套件 563/563 通过。
- `illustrate` 代码块调色板固定为深色 Xcode 风格，相关测试已对齐到该设计。
- `release-check` 原先硬编码指向 `docs/RELEASE_GATE_v0.4.2_ZH.md`，现已改为从 `CARGO_PKG_VERSION` 动态派生，永久消除版本漂移；散落的 `v0.4.2` 显示文案也已去版本化。

但 v0.4.4 不能只因为源码测试通过就发布。发布前仍需补齐下面这些证据。

## 必须完成

### 1. 无凭证 smoke

至少覆盖：

```bash
cargo build --release --all-features
target/release/moonpub --version
target/release/moonpub init /private/tmp/moonpub-smoke
```

如果验证 release 资产，必须用下载或打包后的二进制，不用源码构建结果替代。

当前记录：

- 2026-10-22：本机源码 release build（`cargo build --release --all-features`）通过；`--version` 输出 `moonpub 0.4.4`。这只证明当前源码构建出的 release 二进制可跑通无凭证初始化路径，不替代正式 release 资产下载验证。

### 2. CI 与 Windows smoke

发布前确认：

- PR build 通过
- `windows-smoke` 通过
- release workflow 会验证 Windows zip 内的 `moonpub.exe`

当前记录：

- 2026-10-22：v0.4.4 的版本漂移修复尚未合并进发版 PR，因此尚未单独触发 PR CI / Windows smoke。v0.4.3 的 PR #93 / #94 曾通过 `test` 与 `windows-smoke`；v0.4.4 的对应 CI 需在发版 PR 上重新确认。

### 3. 真实微信人工回归

至少完成一次真实账号路径：

```bash
moonpub wechat-health
moonpub push <article.md> --render
moonpub configure --headed
```

必须确认：

- 微信草稿创建成功
- 原创声明 / 赞赏 / 留言 / 创作来源至少能尝试配置
- 微信公众号后台预览发送成功或失败原因被记录
- 最终发表仍由人工确认，不自动点击发表

当前记录：

- 2026-10-22：v0.4.4 尚未执行独立的真实微信路径人工回归。v0.4.3 已于 2026-07-10 完成真实公众号草稿回归（草稿创建、后台配置、预览发送成功，未点击最终发表），该证据已归档于 `docs/first-run-evidence/wechat/`。v0.4.4 作为维护版本，建议发版前至少复跑一次上述命令确认无回退，尤其是群二维码替换是否仍然正确落进正文。

### 4. 真实证据归档

至少补下面这些截图或录屏，并裁掉敏感信息。这个清单必须和 `moonpub evidence-status` 的 11 个必需文件保持一致：

- `docs/first-run-evidence/homepage/homepage-workspace.png`
- `docs/first-run-evidence/homepage/homepage-context.png`
- `docs/first-run-evidence/feishu/feishu-home-entry.png`
- `docs/first-run-evidence/feishu/feishu-result-modal.png`
- `docs/first-run-evidence/feishu/feishu-draft-opened.png`
- `docs/first-run-evidence/photos/photos-image-opened.png`
- `docs/first-run-evidence/photos/photos-result-modal.png`
- `docs/first-run-evidence/photos/photos-draft-opened.png`
- `docs/first-run-evidence/wechat/wechat-draft-created.png`
- `docs/first-run-evidence/wechat/configure-headed.png`
- `docs/first-run-evidence/wechat/preview-sent.png`

不要提交 token、cookie、二维码、AppSecret、手机号、账号隐私或不可公开照片。

可以在仓库根目录运行下面的本地只读命令，快速查看这些证据文件是否已经归档：

```bash
moonpub evidence-status
moonpub --json evidence-status
moonpub evidence-status --strict
```

这个命令会显示已归档 / 必需 / 缺失数量和缺失路径清单；它只检查文件是否存在，不打开截图、不读取图片内容，也不替代人工脱敏审查。默认模式只报告状态，适合插件首页和人工查看；`--strict` 在缺少必需证据时会非零退出，适合 release 脚本或 CI gate。

如果要看整个 v0.4.4 release gate，而不只是证据文件，可以运行：

```bash
moonpub release-check
moonpub --json release-check
moonpub release-check --strict
```

`release-check` 会聚合本文件里的通过标准勾选状态和 `docs/first-run-evidence/` 文件存在检查；它仍然只做本地只读检查，不会触发微信 API、浏览器自动化或图片内容扫描。`--strict` 在任一 gate 未完成时会非零退出。

当前记录：

- 2026-10-22：`docs/first-run-evidence/` 下 11 个必需文件均已归档（本机 `cargo run -- --json evidence-status --strict` 显示 11/11 present）。这些证据继承自 v0.4.3 的验证，未因 v0.4.4 维护改动失效。

## v0.4.4 范围说明

v0.4.4 明确是维护收口，不新增：

- 不自动最终发表
- 不新增平台
- 不拆飞书为独立项目
- 不把 Obsidian 插件包装成独立发布器
- 不宣称普通用户已完全无门槛使用
- AI 封面真实图片生成尚未在此版本验证（封面 HTML 生成可用，真实 AI 出图路径需真实 OpenAI key + 联网，待单独验证）

## 发布文案口径

推荐口径：

> MoonPub v0.4.4 是一次维护收口版本：合并并行改动、替换过期群二维码、修复 release 版本漂移。它仍然是 Beta，适合能配置微信公众号凭证、愿意人工检查草稿的技术用户。

不要写：

> 全自动公众号发布。

也不要写：

> 无需扫码、无需人工确认。

注意：当前公开可下载的 GitHub Release 仍是 `v0.4.2`（v0.4.4 尚未 cut / 发布）。对外下载与安装指引继续指向 v0.4.2 是准确的；本文件仅在描述"源码版本"时使用 v0.4.4。

## 通过标准

满足下面条件后，可以准备 v0.4.4：

- [x] 本地 release build smoke 通过
- [ ] CI / Windows smoke 通过（v0.4.4 提交需在发版 PR 上重新确认）
- [ ] 真实微信路径人工回归通过或失败原因已记录（v0.4.4 尚未独立复跑；v0.4.3 证据已归档）
- [x] 首次体验证据目录已补齐首页、飞书、照片和微信回归四类核心截图，并通过 `moonpub evidence-status --strict` 文件存在检查
- [x] README / README_zh / USER_GUIDE / PROGRESS 与 release 事实一致（源码版本已同步到 0.4.4；公开下载指引仍指向 v0.4.2，属快照事实，按 AGENTS.md 政策保留）
- [x] 没有真实凭据、token、二维码或隐私截图被提交

可以用 `moonpub release-check --strict` 在准备发版前做一次总门禁检查。

当前记录：

- 2026-10-22：版本漂移修复完成，`release-check` 现已动态指向 `docs/RELEASE_GATE_v0.4.4_ZH.md`；本机 `cargo build --release --all-features` 与 `cargo test` 563/563 通过；`docs/first-run-evidence` 11/11 归档。真实微信回归与 CI / Windows smoke 尚未为 v0.4.4 单独验证，故上述两项门禁暂未勾选；v0.4.4 尚未从源码 cut 为公开 GitHub Release。
