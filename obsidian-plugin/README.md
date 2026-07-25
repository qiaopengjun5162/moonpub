# MoonPub Obsidian Plugin

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

- `打开 MoonPub 首页` -> `moonpub --json doctor` + `moonpub --json workflow-registry` + `moonpub --json evidence-status` + `moonpub --json release-check` + `moonpub --json workspace`
- `查看整体文章池状态` -> `moonpub --json doctor` + `moonpub --json workflow-registry` + `moonpub --json evidence-status` + `moonpub --json release-check` + `moonpub --json workspace`
- `检查当前文章状态` -> `moonpub check <当前文件>`
- `导入最近一条飞书妙记并生成草稿预览` -> `moonpub --articles <path> --json intake feishu --latest --draft --preview`
- `导入最近一条飞书妙记并推进到微信草稿` -> `moonpub --articles <path> --json intake feishu --latest --draft --push`
- `导入当前图片所在目录并生成照片草稿预览` -> `moonpub --articles <path> --json intake photos <当前图片所在目录> --draft --preview`
- `视觉分析当前图片目录并生成照片草稿预览` -> `moonpub --articles <path> --json intake photos <当前图片所在目录> --analyze-images --draft --preview`
- `预览文章` -> `moonpub preview <当前文件>`
- `发布到微信公众号` -> `moonpub ship <当前文件>`（推草稿 + 自动配置后台 + 发送手机预览）
- `AI 润色后发布到公众号` -> `moonpub ship --ai <当前文件>`

如果你在插件设置里配置了 `Articles 根目录`，插件会自动按下面这种形式调用：

注意：

- `预览文章` 是本地 HTML 预览，不需要任何微信凭证。
- `发布到微信公众号` 会调用 `moonpub ship`，自动完成：推草稿 → 配置后台设置 → 发送手机预览。第一次点击时插件会弹出阻塞式引导窗口，让你直接填写接收预览的个人微信号（保存到插件设置，并通过 `WECHAT_PREVIEW_TO` 传给 CLI，不需要回终端）；也可以选择"跳过预览直接发布"。之后可以在插件设置的"微信预览接收人"里修改或清空。如果你在 `moonpub.toml` 里设置 `[wechat].auth_method = "cookie"`（或环境变量 `WECHAT_AUTH_METHOD=cookie`），则不需要 `WECHAT_APPID` / `WECHAT_SECRET`，`ship` 会复用 `moonpub login` 保存的浏览器会话完成推送和后台配置。
- `configure` 默认会发送手机预览。第一次时需要提供接收微信号（`--to <你的微信号>`、`WECHAT_PREVIEW_TO` 环境变量，或先跑 `moonpub test-yulan --to <你的微信号>`）；成功一次后自动记住，以后不再需要输入。未配置接收人时，预览步骤会提示并跳过，configure 的其它步骤仍继续完成。

- `moonpub --articles <path> preview <当前文件>`
- `moonpub --articles <path> ship <当前文件>`

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
4. BRAT 会自动下载最新 Release 中的 `moonpub-obsidian-plugin-vX.Y.Z.zip`，解压到 `.obsidian/plugins/moonpub/`。
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

左侧 Ribbon 图标、`打开 MoonPub 首页` 和 `查看整体文章池状态` 都会先调用 `moonpub --json doctor` 做本地可用性诊断，再读取 `moonpub --json workflow-registry` 展示正式工作流契约、用户价值、安全起点和风险边界，读取 `moonpub --json evidence-status` 展示 v0.4.2 证据缺口，读取 `moonpub --json release-check` 展示 v0.4.2 发布门禁，最后调用 `moonpub --json workspace` 判断整个工作区该走哪条入口、文章池里现在有什么、下一步该先做什么。

插件会把 `moonpub --json doctor` 作为最低兼容性检查。也就是说，仅能运行 `moonpub --help` 的旧 CLI 不会被当成可用版本；如果你在本仓库开发，建议在插件设置里把 `MoonPub 可执行文件路径` 填成当前项目的 `target/debug/moonpub` 绝对路径，或升级到正式 v0.4.2+ 二进制。

飞书入口同样依赖 `Articles 根目录`，因为插件需要基于它来调用 `intake feishu --latest` 这条工作流。

如果 CLI 没找到，或者飞书 / 照片入口缺少 `Articles 根目录`，插件现在也会打开一个简短的修复工作台，把需要安装 CLI、填写可执行文件路径或补 Articles 根目录这些步骤列出来，而不是只弹一条容易错过的 Notice。

现在这个命令不只是弹一条状态提示，而是会继续打开一个简短的“首页工作台”弹窗，把下面几类信息分开展示：

- CLI 是否可用
- Articles 根目录和配置状态
- 正式工作流、安全起点和风险边界
- v0.4.2 证据状态、缺失数量和部分缺失路径
- v0.4.2 发布门禁状态、未完成 gate 和下一步命令
- 当前推荐入口
- drafts / ready / published 阶段数量
- 推荐下一步动作
- 哪些能力会联网或打开 Chrome

正式工作流区域不只是展示命令，也会给可执行的“安全开始”按钮：

- `current-article` 会引导你先检查或预览当前文章
- `feishu-minutes` 会走“导入最近飞书妙记并生成草稿预览”
- `photo-memory` 会走“导入当前图片目录并生成照片草稿预览”
- `wechat-draft` 只展示边界提示，不会从首页直接触发微信草稿推进

而且它现在开始更像插件首页了：在同一个工作台里，你也可以直接继续点下面这些入口动作：

- `检查当前文章`
- `预览当前文章`
- `导入最近飞书妙记`
- `导入当前图片目录`

它现在还会根据你当前打开的内容给出更贴近上下文的推荐：

- 当前打开的是 Markdown：更推荐先检查或预览当前文章
- 当前打开的是图片：更推荐先导入当前图片目录
- 当前没有打开文件，或打开的是别的文件：会提示你先回到更合适的入口

现在这个首页还会把“第一次建议步骤”直接列出来。也就是说，它不只告诉你“该点哪个入口”，还会继续告诉你“点完之后建议按什么顺序走”。

`检查当前文章状态` 现在也会继续打开“当前文章工作台”弹窗，把这篇文章最关键的状态拆开展示：

- 当前是否已经可继续发布
- Markdown / HTML / `draft.json` / `media_id` 是否齐全
- 对应产物路径
- 当前最推荐的下一步命令
- 可继续操作按钮：复制下一步命令、预览当前文章、发布前检查；当 HTML 已存在时，可以执行排版审计；当 `draft.json` 已存在时，可以显式推进到微信草稿

其中“排版审计”实际调用 `moonpub --json layout-audit <html>`，只检查本地 HTML 的公众号编辑器兼容风险，不触发微信 API，也不会自动打开浏览器；审计结果弹窗里可以再手动点“打开 HTML 预览”查看页面。

其中“发布前检查”实际调用 `moonpub --json preflight <article.md>`，会聚合检查 Markdown / HTML / `draft.json`、排版审计结果和 `.media_id` 状态。这个动作同样只读本地产物，不触发微信 API，也不会打开或控制 Chrome；缺 `.media_id` 只会作为“还没推到微信草稿”的提醒。

如果你平时是从飞书妙记开始，而不是先自己写 Markdown，插件现在也补了两条更像“正式入口”的命令：

- `导入最近一条飞书妙记并生成草稿预览`
- `导入最近一条飞书妙记并推进到微信草稿`

它们分别对应：

- 默认保守路径：先停在“草稿 + 本地预览”
- 显式快速路径：继续推进到微信草稿

这样你不必先切回终端，也能在 Obsidian 里直接起飞书主工作流。

飞书入口在真正读取素材前会先打开确认窗口，明确说明完整转写会发送到当前配置的 AI provider 生成草稿。默认“草稿 + 本地预览”路径不会触达微信公众号或 Chrome；只有明确选择快速 push 才会继续创建微信草稿。

导入完成后会先展示结果工作台，再由你点击其中的“打开草稿”。这样路径、预览和推荐下一步会先完整可见，不会因切换标签页而丢掉结果工作台。

另外，飞书入口执行完成后，插件现在会继续打开一个“飞书结果工作台”弹窗，把下面这些信息分开展示：

- Inbox 路径
- 生成出来的草稿路径
- 本地 HTML 预览路径（如果这次走了 `--preview`）
- 是否已经推进到微信草稿，以及 `media_id`
- 推荐下一步动作和对应命令

这个结果工作台现在不只是展示信息，还会提供后续动作按钮。你可以直接从里面继续：

- `打开草稿`
- `检查草稿`
- `预览草稿`
- `复制下一步命令`
- `发布前检查`
- `排版审计`（仅当这次已经生成本地 HTML 预览）
- `推进到微信草稿`（仅当这次还没 push）

其中“排版审计”和当前文章工作台一致，调用 `moonpub --json layout-audit <html>`，只检查本地 HTML，不触发微信 API，也不会自动打开浏览器；审计结果弹窗里可以再手动打开 HTML 预览。这样你导入结束后，不会只剩下一条长 Notice，而是能直接看到这次飞书 / 照片链路到底产出了什么、下一步该怎么接。

“发布前检查”和当前文章工作台一致，调用 `moonpub --json preflight <draft.md>`。推荐在点击“推进到微信草稿”前先跑一次，让插件先告诉你本地产物是否齐全、排版是否有高风险结构。

照片链路现在也开始有了第一条正式插件入口：

- `导入当前图片所在目录并生成照片草稿预览`

它适合你当前正打开一张图片，希望把“这张图片所在目录”当成一组生活照片素材统一整理的场景。插件会把所在目录传给 `--json intake photos ... --draft --preview`，然后和飞书一样回到草稿与结果工作台。

照片入口同样要求明确确认。当前版本只把图片文件路径、文件名、大小和修改时间写入 Inbox 并发送这份素材清单给 AI provider，**不会上传图片像素**；确认窗口会在执行前展示当前目录与这条边界。

如果你需要从照片本身提取可见信息，可以显式运行 `视觉分析当前图片目录并生成照片草稿预览`。它会在第二个确认窗口里说明上传范围和限制：最多 5 张 jpg/jpeg/png/webp 图片，单张不超过 8 MiB、合计不超过 20 MiB，且只支持 `[ai] provider = "openai"`。分析结果会写入 Inbox 并标为“需人工核对”，不能直接当作事实发表。

另外，插件现在会在执行“发布到微信公众号”前先调用 `moonpub --json capabilities` 读取能力元数据，并给出一条简短提示，例如：

- 这次操作会不会触达外部服务
- 会不会拉起或控制 Chrome
- 通常需要哪些环境变量或配置项

这里的提示是“发布前说明”，不是硬性阻断。也就是说：

- 插件不会再仅凭 Obsidian 进程里的 `process.env.WECHAT_*` 就误判“你不能发布”
- 如果你的凭据是放在项目 `.env` 或 `~/.moonpub.env` 里，MoonPub CLI 仍会按自己的配置优先级继续加载。插件运行命令时，会在开发构建的仓库根目录执行；正式安装的二进制则在 Articles 根目录执行，因此两种场景都能让 CLI 找到对应的本地 `.env`，而不需要把密钥写进插件设置。
- 真正是否能成功推送，仍以 MoonPub CLI 的实际运行结果为准

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
