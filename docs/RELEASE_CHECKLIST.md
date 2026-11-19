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
- [x] PR CI 已覆盖 Windows 源码构建二进制的无凭证 smoke。
- [x] release workflow 已覆盖打包后的 Windows zip 资产 smoke。

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

## 发版流程（实际生效机制）

release-please 自动开「`chore(main): release X.Y.Z`」PR，但合并后**不会自动构建资产**——根因是 release-please 用默认 `GITHUB_TOKEN` 创建 release/tag，而 GitHub 规定 GITHUB_TOKEN 产生的事件不会触发其它 workflow。所以 `release: published` 和 `push: tags` 都点不着火（v0.4.8/v0.4.9 都曾因此 0 资产）。

每一步：

1. 往 `main` 推 conventional 提交（`feat:`/`fix:` 会进 changelog，`style:`/`docs:` 默认不进）。
2. release-please 自动开/更新发版 PR。
3. **先确认这个 PR 只改版本元数据**：`gh pr diff <num> --name-only` 应只有 `.release-please-manifest.json`、`CHANGELOG.md`、`Cargo.toml`、`Cargo.lock`、`ROADMAP.md`。出现 `src/` 就停下，说明它不是纯粹的 release PR。

   合并（`gh pr merge <num> --squash`）。release-please 通过 API 建 release + tag，配置里 `"draft": true`，所以此刻它是**草稿**——草稿不计入 `latest`，`releases/latest/download/...` 仍指向上一版，安装链接不会因为正在构建而 404。

   发版 PR 会抬 `Cargo.toml` 的版本，`ROADMAP.md` 的「当前源码版本」由 release-please 的 `extra-files` generic 更新器自动改写（行尾带 `x-release-please-version` 标记）。若 CI 在发版 PR 上报 `roadmap_reports_the_crate_version` 失败，说明标记或 `extra-files` 配置丢了——**先修这个再合并**，否则 tag 指向的提交本身过不了 CI。
4. **把草稿发布出去，用它生成 tag**（关键，否则 0 资产）：

   ```bash
   gh release list --limit 3                 # 等到 vX.Y.Z 以 Draft 出现
   gh release edit vX.Y.Z --draft=false      # gh 用你的 token 建 tag，于是会点火
   git ls-remote --tags origin | grep vX.Y.Z # 确认 tag 已建、指向 main 上的合并提交
   ```

   这一步会同时触发**两个** `release.yml` run：`event=release`（publish 事件）和 `event=push`（gh 建 tag 的 push 事件）。两者都会完整构建 5 个平台，必须取消重复的那个：

   ```bash
   gh run list --workflow release.yml --limit 6 \
     --json databaseId,status,event,headBranch --jq '.[] | "\(.databaseId) \(.status) \(.event)"'
   gh run cancel <event=push 的那个 id>
   gh run watch <event=release 的那个 id> --exit-status
   ```

   保留 `event=release` 的那个即可；重复跑不会让资产变多，只会让 Actions 里多一条 cancelled 记录。

5. 终检（**必须是 11 个资产**）：

   ```bash
   gh release view vX.Y.Z \
     --json tagName,isDraft,isPrerelease,assets \
     --jq '{tag:.tagName,draft:.isDraft,pre:.isPrerelease,n:(.assets|length)}'
   ```

   预期 `draft=false`、`pre=false`、`n=11`（5 平台 × 二进制 + `.sha256`，外加 Obsidian 插件 zip）。`macos-15-intel` 是 x86 runner，排队经常比其他平台晚 5–10 分钟才起步，期间 `gh api .../actions/jobs/<id>/logs` 返回 404 是正常的（job 还没落盘），不要误判卡死。

6. **下载资产跑一次真实 smoke，不要只信 CI success**：见下面「发布后回归」。

### 构建失败：release 已发布但 0 资产

（v0.4.14 就栽在打包 smoke 的 `init` → `render` 上）此时该 release 往往 **0 资产**，而 `latest` 已经指向它，所有 `releases/latest/download/...` 安装链接立刻 404。处置顺序：先用 `gh release delete vX.Y.Z --yes --cleanup-tag` 撤掉空壳（`latest` 会自动回退到上一个有资产的版本），修好根因后以**新版本号**重发——不要给同一个 tag 重新上传资产，那会让 tag 与代码版本对不上。

> 想让发版全自动：给 `release-please.yml` 的 `token:` 换成 PAT（仓库 secret，非 `GITHUB_TOKEN`），这样 `release: published` 才会自动点火。当前仓库无可用 PAT，故走手动 publish 草稿。另一条等效路径是 `gh workflow run release.yml --ref vX.Y.Z`，但它绕过了 tag 与 draft 的对应关系，且不会留下 `event=release` 的记录，排查时更难判断「这次构建属于哪次发布」，因此不作为默认做法。

### 陷阱：合并发版 PR 后立刻冒出一个「下一版」PR

症状：合并 `0.4.18` 的 release PR 后，release-please 又开了 `chore(main): release 0.4.19`，且它的 `CHANGELOG.md` diff 有上百行，把 `#150`、`#164`、`#207` 这些旧 PR 全部重新列了一遍（正常应该只有本次合并的那几条，例如 0.4.18 段只有 7 行）。

根因：那一跑发生在 `vX.Y.Z` tag **建好之前**，release-please 找不到「上次发布锚点」，于是回溯到一个很早的提交；仓库用 squash-merge，release PR 的 head SHA 与合并后 main 上的提交 SHA 不同，也无法用作锚点。

处置：**关掉，不要合并**。那个分支上只比 main 多一个 release commit，没有代码；一旦合并就会发布一份大量重复的 changelog。tag 已经存在且是 main 的祖先，下一次 push 到 main 时 release-please 会以它为起点重算。

**已从源头堵住（v0.4.22 之后）**：`.github/workflows/release-please.yml` 的 job 带了一个 `if:`，跳过 `head_commit.message` 以 `chore(main): release` 开头的那次 push——也就是「合并发版 PR、tag 还没建」的窗口。跳过不丢功能：manifest 已由发版 PR 写好，下一次真实提交会以新 tag 为锚点正常开下一版 PR。这条闸上线前，同一现象已连续复现五次（`#226` / `#232` / `#234` / `#236` / `#238`）。若仍然冒出这种 PR，先确认 `if:` 没被改掉，再按下面的判据处置。

判据：看到 release PR 的版本号跳变、但 main 上并没有新的 `feat`/`fix` 时，第一件事是看 CHANGELOG diff 有多长——几行是正常增量，上百行就是锚点丢了。

## 对外材料

- [x] README / README_zh 可直接作为项目首页说明。
- [x] `docs/GETTING_STARTED.md` 能引导新用户先跑本地体验。
- [x] `docs/USER_GUIDE.md` 有完整工作流。
- [x] `docs/LAUNCH_ARTICLE_ZH.md` 有中文发布文章发布稿，并已补 v0.4.1 release 与首跑 smoke test 口径。
- [x] `docs/LAUNCH_PLAN_ZH.md` 有面向队友和早期用户的目标、进度条、下一步计划。
- [x] `docs/LAUNCH_DEMO_ASSETS_ZH.md` 记录 v0.4.1 release 二进制生成的本地预览 HTML、封面 HTML、`check` 和 `status` 输出。
- [x] `docs/LAUNCH_SCREENSHOT_CHECKLIST_ZH.md` 有截图交付物清单。
- [x] `docs/WECHAT_REGRESSION_CHECKLIST_ZH.md` 有真实微信草稿回归清单。
- [x] 截图/录屏清单已准备：本地预览、封面和 `status` 已完成；真实微信草稿、`configure --headed` 等待真实回归。

## 发布后回归

**每次发版都要做（不要只信 CI success）**：

```bash
# 1. 下载本机平台的资产，与 .sha256 逐字节比对
TAG=<本版 tag>            # 只有这一步需要填具体版本；安装链接本身仍写 releases/latest
BASE="https://github.com/qiaopengjun5162/moonpub/releases/download/$TAG"
curl -fsSL "$BASE/moonpub-macos-arm64.tar.gz.sha256" -o sha
curl -fsSL "$BASE/moonpub-macos-arm64.tar.gz" -o bin.tar.gz
[ "$(awk '{print $1}' sha)" = "$(shasum -a 256 bin.tar.gz | awk '{print $1}')" ] && echo CHECKSUM_OK

# 2. 用这个二进制（不是 cargo install 的那个）跑一次真实渲染
./moonpub --version
./moonpub --articles <ws> render <article.md>
```

CI 绿只说明源码能编译、测试能过；它不证明**发出去的那个二进制**可用（v0.4.14 的 0 资产、旧版仍带 `white-space:pre-wrap` 都属于这一类）。本轮安全修复的验证方式可作为模板：用 release 二进制跑 `export`，再把生成的 frontmatter 交给 TOML 解析器，断言注入载荷没有变成独立字段。

历史记录：

- [x] 下载 v0.4.0 macOS amd64 release 资产，确认 sha256 通过。
- [x] v0.4.0 macOS amd64 release 资产可在 Apple Silicon / Rosetta 环境运行 `moonpub --help`。
- [x] 下载 v0.4.1 macOS ARM64 release 资产，确认 sha256 通过。
- [x] v0.4.1 macOS ARM64 release 资产可运行 `moonpub --help`。
- [x] v0.4.1 macOS ARM64 release 资产可运行 `moonpub --version`，输出 `moonpub 0.4.1`。
- [x] 用 v0.4.1 macOS ARM64 release 二进制跑通本地无凭证体验：`init` → `new` → `render` → `cover` → `check`。
- [x] release workflow 在 `windows-latest` 上解压 `moonpub-windows-amd64.zip` 并跑通 `--version` → `--help` → `init` → `new` → `render` → `check`。
- [x] v0.4.1 tag 初次触发 release workflow，但 macOS ARM64 build 因 `ring` + `target-cpu=native` 失败，未产出 release 资产。
- [x] 如果更新了 README 中的版本号，确认链接可下载。
- [x] 记录真实验证结果到 `PROGRESS.md`，不要把本地测试说成真实微信验证。

备注：当前开发机是 Apple Silicon。v0.4.0 macOS 资产只有 `macos-amd64`，已验证可下载、sha256 通过、`--help` 可运行，但 `--version` 不存在，且非交互 `init` 写入 `/path/to/ObsidianMain` 导致本地首跑失败；v0.4.1 已重新触发 release workflow 并成功产出 macOS ARM64/AMD64、Linux ARM64/AMD64、Windows AMD64 资产。macOS ARM64 资产已完成本地 smoke test，Windows 侧现在同时有源码构建 PR smoke 和打包 zip release smoke，可作为当前对外主推版本 + Windows 更强信心证据。

## v0.4.1 首发剩余工作

- [x] 用 release 二进制生成本地预览 HTML。
- [x] 用 release 二进制生成封面 HTML。
- [x] 记录 `moonpub status` / `moonpub check` 的安全输出。
- [x] 用普通系统浏览器或专门截图流程导出本地预览截图。
- [x] 用普通系统浏览器或专门截图流程导出封面截图或 PNG。
- [ ] 在有真实凭证且不泄露 secret 的环境中完成微信草稿回归。
- [x] 发布文章配图后再对外发布。

备注：2026-06-24 尝试用 Codex 内置浏览器打开本地 `file://` HTML 导出截图时，被 Browser Use URL policy 阻止；这属于当前工具安全边界，不代表 MoonPub 生成的 HTML 失败。
