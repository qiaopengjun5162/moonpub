# MoonPub Obsidian Plugin

> 完整用户上手指南：中文 [docs/USER_ONBOARDING_ZH.md](../docs/USER_ONBOARDING_ZH.md) / English [docs/USER_ONBOARDING.md](../docs/USER_ONBOARDING.md)。
>
> 第一次不知道先点哪里：中文 [docs/FIRST_RUN_WALKTHROUGH_ZH.md](../docs/FIRST_RUN_WALKTHROUGH_ZH.md) / English [docs/FIRST_RUN_WALKTHROUGH.md](../docs/FIRST_RUN_WALKTHROUGH.md)。

这个插件不是独立发布器，而是 **Obsidian 里的 MoonPub 入口**。

它的作用是：

- 在 Obsidian 里直接调用本地 `moonpub`
- 对当前打开的 Markdown 文件执行常用操作
- 减少你在终端里手敲命令的次数

它当前仍然是实验性入口，不是一个全自动发布机器人。

## 当前支持的命令

安装后，左侧 Ribbon 会多一个 MoonPub 图标，点击后会直接打开 `MoonPub 首页工作台`。

也可以在 Obsidian 里按 `Cmd/Ctrl + P`，看到这些命令：

- `打开 MoonPub 首页`
- `查看整体文章池状态`
- `检查当前文章状态`
- `导入最近一条飞书妙记并生成草稿预览`
- `导入最近一条飞书妙记并推进到微信草稿`
- `导入当前图片所在目录并生成照片草稿预览`
- `视觉分析当前图片目录并生成照片草稿预览`
- `发布到微信公众号`
- `预览文章`
- `AI 润色后发布到公众号`

这些命令实际调用的是本地 `moonpub`：

- `打开 MoonPub 首页` -> `moonpub --json doctor` + `moonpub --json workflow-registry` + `moonpub --json layout-recipes` + `moonpub --json evidence-status` + `moonpub --json release-check` + `moonpub --json workspace`
- `查看整体文章池状态` -> `moonpub --json doctor` + `moonpub --json workflow-registry` + `moonpub --json layout-recipes` + `moonpub --json evidence-status` + `moonpub --json release-check` + `moonpub --json workspace`
- `检查当前文章状态` -> `moonpub --json check <当前文件>`
- `导入最近一条飞书妙记并生成草稿预览` -> `moonpub --articles <path> --json intake feishu --latest --draft --preview`
- `导入最近一条飞书妙记并推进到微信草稿` -> `moonpub --articles <path> --json intake feishu --latest --draft --push`
- `导入当前图片所在目录并生成照片草稿预览` -> `moonpub --articles <path> --json intake photos <当前图片所在目录> --draft --preview`
- `视觉分析当前图片目录并生成照片草稿预览` -> `moonpub --articles <path> --json intake photos <当前图片所在目录> --analyze-images --draft --preview`
- `预览文章` -> `moonpub preview <当前文件>`
- `发布到微信公众号` -> `moonpub ship <当前文件>`（推草稿 + 自动配置后台 + 发送手机预览）
- `AI 润色后发布到公众号` -> `moonpub ship --ai <当前文件>`

如果你在插件设置里配置了 `Articles 根目录`，插件会自动按下面这种形式调用：

```bash
moonpub --articles <path> preview <当前文件>
moonpub --articles <path> ship <当前文件>
```

注意：

- `预览文章` 是本地 HTML 预览，不需要任何微信凭证。
- `发布到微信公众号` 会调用 `moonpub ship`，自动完成：推草稿 → 配置后台设置 → 发送手机预览。第一次点击时插件会弹出阻塞式引导窗口，让你直接填写接收预览的个人微信号（保存到插件设置，并通过 `WECHAT_PREVIEW_TO` 传给 CLI，不需要回终端）；也可以选择"跳过预览直接发布"。之后可以在插件设置的"微信预览接收人"里修改或清空。如果你在 `moonpub.toml` 里设置 `[wechat].auth_method = "cookie"`（或环境变量 `WECHAT_AUTH_METHOD=cookie`），则不需要 `WECHAT_APPID` / `WECHAT_SECRET`，`ship` 会复用 `moonpub login` 保存的浏览器会话完成推送和后台配置。
- `configure` 默认会发送手机预览。第一次时需要提供接收微信号（`--to <你的微信号>`、`WECHAT_PREVIEW_TO` 环境变量，或先跑 `moonpub test-yulan --to <你的微信号>`）；成功一次后自动记住，以后不再需要输入。未配置接收人时，预览步骤会提示并跳过，configure 的其它步骤仍继续完成。

## 产品边界

这个插件遵守和 MoonPub CLI 一样的边界：

- 本地预览不需要微信凭证
- 真正推送会触达微信 API
- 真正发布前仍然需要用户自己检查微信草稿
- 不绕过扫码、验证码、平台审核或最终人工确认

所以更准确地说，它是：

**Obsidian 中的本地发布副驾驶入口。**

## 安装方式

推荐普通用户通过 **BRAT** 安装，以获得自动更新；技术用户也可以手动复制目录。

### 方式一：BRAT 安装（推荐）

1. 在 Obsidian 的第三方插件市场里安装并启用 **BRAT**。
2. 打开 BRAT 设置，点击 `Add Beta plugin with frozen version` 或 `Add Beta plugin`。
3. 填入仓库地址：

   ```
   https://github.com/qiaopengjun5162/moonpub
   ```

4. BRAT 会自动下载最新 Release 中的插件包 `moonpub-obsidian-plugin-vX.Y.Z.zip`（里面包含 `main.js`、`manifest.json`、`styles.css`），解压到 `.obsidian/plugins/moonpub/`。
5. 进入 Obsidian `设置 → 第三方插件`，启用 `MoonPub`。
6. （首次使用）按下面“首次配置”步骤填写 `Articles 根目录` 和 `MoonPub 可执行文件路径`。

### 方式二：手动复制（开发/测试）

1. 先安装 MoonPub CLI，确保终端里能运行：

   ```bash
   moonpub --help
   ```

2. 把本仓库中的 `obsidian-plugin/` 复制到你的 vault：

   ```text
   .obsidian/plugins/moonpub/
   ```

3. 在插件目录中运行：

   ```bash
   npm ci
   npm test
   npm run build
   ```

4. 在 Obsidian 中启用第三方插件里的 `MoonPub`。

### 首次配置

当前插件已经有最基本的设置页，可以配置：

- `MoonPub 可执行文件路径`
- `Articles 根目录`
- `微信预览接收人`（首次点“发布到微信公众号”时也会弹出引导填写）

推荐你在这两种情况下手动设置：

- `moonpub` 不在常见安装路径里
- 你的当前 Vault 不是文章根目录，需要显式传 `--articles`
- PATH 里有旧版 `moonpub`，但它还不支持 `moonpub --json doctor` / `workspace`

左侧 Ribbon 图标、`打开 MoonPub 首页` 和 `查看整体文章池状态` 都会先调用 `moonpub --json doctor` 做本地可用性诊断，再读取 `moonpub --json workflow-registry` 展示正式工作流契约、用户价值、安全起点和风险边界，读取 `moonpub --json layout-recipes` 展示主题分组、重点主题和排版配方，读取 `moonpub --json evidence-status` 展示 v0.4.2 证据缺口，读取 `moonpub --json release-check` 展示 v0.4.2 发布门禁，最后调用 `moonpub --json workspace` 判断整个工作区该走哪条入口、文章池里现在有什么、下一步该先做什么。

插件会把 `moonpub --json doctor` 作为最低兼容性检查。也就是说，仅能运行 `moonpub --help` 的旧 CLI 不会被当成可用版本；如果你在本仓库开发，建议在插件设置里把 `MoonPub 可执行文件路径` 填成当前项目的 `target/debug/moonpub` 绝对路径，或升级到正式 v0.4.2+ 二进制。

飞书入口同样依赖 `Articles 根目录`，因为插件需要基于它来调用 `intake feishu --latest` 这条工作流。

如果 CLI 没找到，或者飞书 / 照片入口缺少 `Articles 根目录`，插件现在也会打开一个简短的修复工作台，把需要安装 CLI、填写可执行文件路径或补 Articles 根目录这些步骤列出来，而不是只弹一条容易错过的 Notice。

## 首页工作台与结果工作台

左侧 Ribbon 图标、`打开 MoonPub 首页` 和 `查看整体文章池状态` 都会打开卡片化首页工作台，而不是只弹一条 Notice。首页从上到下展示：

- **当前文件**：当前 Obsidian 文件、适合走哪条入口、主动作按钮
- **工作区概览**：CLI / Articles 根目录 / 配置状态、文章总数、`drafts / ready / published` 阶段数量
- **推荐下一步 + 首次建议**：根据当前上下文给出的下一步命令和第一次使用顺序
- **正式工作流**：当前文章、飞书、照片、微信草稿边界的安全起点和风险提示
- **排版主题**：`geek-black` / `blueprint` / `ai-lab` 等重点主题、主题分组、完整配方入口，以及一键复制正文 `theme` / 封面命令
- **证据 / 门禁 / 操作入口**：v0.4.2 证据状态、发布门禁、导入飞书、导入图片、查看微信草稿边界

首页会根据当前文件给出更贴近上下文的推荐：打开 Markdown 时优先检查或预览当前文章；打开图片时优先导入当前图片目录；没有合适文件时，引导你回到首页入口。排版主题卡片只读本地元数据，不触发渲染、微信 API 或 Chrome；其中“复制 theme”会复制类似 `theme: geek-black` 的 frontmatter 片段，“复制封面命令”会复制 `moonpub cover article.md --style geek-black --screenshot` 这类命令，当前打开 Markdown 时会自动带上当前文章路径。

首页工作台、当前文章工作台、飞书/照片结果工作台、发布前检查和排版审计工作台使用同一套 `moonpub-card` 容器、`moonpub-card-title` 标题和 `moonpub-action-row` 按钮行。

### 当前文章工作台

`检查当前文章状态` 会打开当前文章工作台，展示：

- 当前是否可继续发布
- Markdown / HTML / `draft.json` / `media_id` 是否齐全
- 对应产物路径
- 当前文章是否显式配置正文 `theme`，以及最终实际使用哪套 `effective_theme`
- 当前最推荐的下一步命令
- 后续动作：复制下一步命令、复制当前 `theme`、复制匹配封面命令、复制极客黑 `theme`、预览当前文章、发布前检查、排版审计、推进到微信草稿

`发布前检查` 调用 `moonpub --json preflight <article.md>`，只读本地产物，不触发微信 API，不打开或控制 Chrome。缺 `.media_id` 只表示还没推到微信草稿，不代表本地产物失败。

`排版审计` 调用 `moonpub --json layout-audit <html>`，只检查本地 HTML 的公众号编辑器兼容风险。审计结果弹窗里可以再手动打开 HTML 预览。

### 飞书和照片结果工作台

飞书入口有两条路径：

- `导入最近一条飞书妙记并生成草稿预览`：默认保守路径，停在“草稿 + 本地预览”
- `导入最近一条飞书妙记并推进到微信草稿`：显式快速路径，继续推进到微信草稿

飞书入口在读取素材前会打开确认窗口，明确说明完整转写会发送到当前配置的 AI provider。默认“草稿 + 本地预览”路径不会触达微信公众号或 Chrome；只有明确选择快速 push 才会创建微信草稿。

照片入口适合当前打开一张图片，并把“这张图片所在目录”当成一组生活照片素材。默认路径只发送文件路径、文件名、大小和修改时间，不上传图片像素；`视觉分析当前图片目录并生成照片草稿预览` 会额外确认图片像素将发送给 OpenAI，最多 5 张 jpg/jpeg/png/webp，单张 8 MiB、合计 20 MiB。

飞书 / 照片导入完成后会打开结果工作台，展示 Inbox、草稿、本地 HTML 预览、是否已推进到微信草稿、`media_id` 和推荐下一步。你可以从结果工作台继续打开草稿、检查草稿、预览草稿、复制下一步命令、发布前检查、排版审计，或在确认后推进到微信草稿。

### 发布前风险提示

插件执行 `发布到微信公众号` 前会调用 `moonpub --json capabilities` 读取能力元数据，提示这次操作是否会触达外部服务、是否会拉起或控制 Chrome、通常需要哪些环境变量或配置项。

这个提示不是硬性阻断。插件不会只凭 Obsidian 进程里的 `process.env.WECHAT_*` 判断能否发布；如果凭据放在项目 `.env` 或 `~/.moonpub.env`，MoonPub CLI 仍会按自己的配置优先级加载。真正是否能成功推送，以 CLI 的实际运行结果为准。

## 推荐使用顺序

第一次使用插件时，推荐这样走：

1. 先在设置里填好 `Articles 根目录`
2. 先点击左侧 MoonPub 图标，或执行 `打开 MoonPub 首页`
3. 如果你从飞书素材开始，先执行 `导入最近一条飞书妙记并生成草稿预览`
4. 如果你从照片素材开始，先打开一张图片，再执行 `导入当前图片所在目录并生成照片草稿预览`
5. 如果你从现有文章开始，再打开一篇 Markdown 文章并执行 `检查当前文章状态`
6. 再执行 `预览文章`
7. 确认本地排版没问题后，再执行 `发布到微信公众号`

现在第 2 步里的左侧 MoonPub 图标和 `打开 MoonPub 首页`，本身就已经开始承担这个首页角色。也就是说，你很多时候不必先记住所有命令，而是先打开它，再从里面继续点下一步。

不要一上来就直接发。

如果你要给插件首页、飞书入口或照片入口补真实截图 / 录屏证据，按仓库里的取证 runbook 走：

- `docs/first-run-evidence/RUNBOOK_ZH.md`

这些证据是 v0.4.2 release gate 的一部分。没有真实截图时，不要用示意图冒充真实体验证据。

## 当前限制

- 目前只有桌面版可用：`manifest.json` 已设置 `isDesktopOnly: true`
- 当前设置项还很少，只解决最关键的路径问题
- 主要依赖系统里已经装好的 `moonpub`
- 预览走本地 CLI；发布仍然依赖微信凭证、IP 白名单和 Chrome 自动化链路

## 后续方向

这个插件后面更适合往这些方向走：

- 继续把 workspace / 当前文件 两层状态展示做得更清楚
- 继续扩展 `moonpub --json capabilities` 的使用，把风险提示做得更细
- 区分本地命令与会触达微信的命令
- 明确展示“当前命令会不会联网、会不会打开 Chrome”

但当前阶段，最重要的是：

**先把它做成一个可靠、好理解的 Obsidian 入口。**

## 社区市场上架状态

- [x] 插件 manifest、main.js、styles.css 已随 Release 发布
- [ ] 已提交 PR 到 [obsidianmd/obsidian-releases](https://github.com/obsidianmd/obsidian-releases)
- [ ] 已通过社区市场审核

在通过社区市场审核前，请先用 **BRAT** 安装。
也可以直接下载 Release 中的 `main.js`、`manifest.json`、`styles.css` 手动放到 `.obsidian/plugins/moonpub/`。
**上架进度：**fork 已创建（`qiaopengjun5162/obsidian-releases`），分支 `add-moonpub-plugin` 已推送，包含 `community-plugins.json` 修改。由于 GitHub token 权限限制，PR 未能自动创建，请访问 <https://github.com/qiaopengjun5162/obsidian-releases/pull/new/add-moonpub-plugin> 手动提交 PR。
