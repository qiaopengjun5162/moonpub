# MoonPub CLI Progress

## Status

Beta / early adopter ready. MoonPub v0.4.2 is publicly released with Linux amd64/arm64, macOS amd64/arm64, and Windows amd64 assets. The release workflow passed plugin build, Rust checks, all five platform builds, Linux/macOS archive smoke, and Windows zip smoke; the downloaded macOS ARM64 asset also passed SHA-256 and no-credential `--version -> init -> new -> render -> check` smoke. It is usable by technical users who can configure WeChat credentials. The current source build completed a live WeChat backend preview/configure regression on 2026-07-03, and on 2026-07-10 it also created a real WeChat draft from a public temporary test article, moved the local bundle to `Articles/ready`, configured原创/赞赏/留言/创作来源, and sent the backend preview successfully without final publishing. Final WeChat publication remains manual by design.

## Final Goal

MoonPub 的最终目标：让作者从 Obsidian / Markdown 出发，用一个可审计、可复现、可本地运行的 Rust CLI，把文章稳定发布到微信公众号草稿，并同步导出到个人博客；对外使用时，用户应能按 README 完成安装、配置、预览、推送和故障排查。

长期路线见 [ROADMAP.md](ROADMAP.md)：先完成真实微信回归，再做插件化核心、Obsidian 插件正式化、WordPress / Ghost 等低风险多平台发布，最后探索本地 App 和 Pro 版。v0.5 插件化设计见 [docs/PLUGIN_ARCHITECTURE_ZH.md](docs/PLUGIN_ARCHITECTURE_ZH.md)。

## Progress Bar

整体进度：`█████████░` 89%（技术用户 Beta 完成度；不是 v1.0 产品化完成度）

补充判断：
- 技术用户可用度：88-90%
- 普通用户可顺利上手度：70-75%
- v1.0 产品化完成度：55-60%

| 领域 | 进度 | 当前判断 |
|------|------|----------|
| 核心 CLI / 配置 / 状态 | `█████████░` 90% | 常用命令完整，仍可继续改善错误提示和 dry-run |
| Markdown 渲染 / Block / Theme | `█████████░` 95% | 已能产出微信 HTML，解析、行内语法、普通段落与 fence block 渲染已拆分；正文主题增至 23 套，Block 模板为 20 种；新增 `layout-audit` 质量门和 `moonlit` / `porcelain` / `fieldnote` 三套生活合集主题 |
| WeChat API 推送 | `████████░░` 85% | draft add/update/image upload 可用，仍需更多错误场景文档 |
| CDP 浏览器自动化 | `█████████░` 89% | 2026-07-03 已用真实登录态跑通 `test-yulan --headed` 和 `configure --headed`；2026-07-10 已用公开临时测试文跑通 `push --render` 到真实微信草稿创建、进入 ready、原创/赞赏/留言/创作来源配置和后台预览发送；新增 `wechat-health` 发布前预检入口，`configure --evidence-dir` 可显式保存 release 取证截图；headless 下登录态失效会快速失败并提示恢复，不再等待不可见二维码；合集/发表仍未启用 |
| 对外安装 / Release | `██████████` 100% | v0.4.2 已公开发布五个平台资产；tag release workflow 通过 Linux / macOS `.tar.gz` 与 Windows `.zip` 打包资产 smoke，本机已从 GitHub Releases 下载 macOS ARM64 包、校验 SHA-256 并完成无凭证 smoke |
| 文档 / 教程 / 对外介绍 | `██████████` 100% | README、发布清单、发布说明、发布计划、演示素材记录、截图清单、微信回归清单、中文发布文章和本地预览/封面 PNG 已同步；真实微信回归与首次体验 11/11 脱敏证据均已归档，v0.4.2 仍定位为需要人工检查的技术用户 Beta |
| 测试 / CI / 审计 | `█████████░` 86% | PR CI 与 tag release workflow 都会执行 Obsidian 插件的 `npm ci && npm run build`；v0.4.2 release run 已通过五个平台构建与打包资产 smoke。本地 `cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` fresh 通过，当前 353 tests passed；`cargo llvm-cov nextest --all-features --summary-only` 上次测得总行覆盖 59.65%，浏览器自动化覆盖仍不足 |
| 代码结构 / 可维护性 | `█████████░` 92% | Radar 已完成首轮拆分，Markdown parser、inline、plain、blocks、AI workflow、init、draft、bundle、plugin、cover 辅助、intake 上游素材导入与 ship 编排模块已拆出；capabilities 提供插件/App 可直接调用的 target 命令模板和前置条件，AI provider 与 configure 模板插入已可配置 |

## Current Milestone

目标：把项目从“作者本人可用”推进到“技术用户可照文档试用”。

完成标准：
- [x] v0.4.0 release 有 Linux / macOS / Windows 资产，且已验证 macOS amd64 下载与 sha256
- [x] PR CI 通过：fmt / clippy / cargo audit / nextest / Windows 无凭证 smoke
- [x] README 不再指向过期 release 或不存在的 Homebrew tap
- [x] README / README_zh 第一屏明确 Beta 状态、适用人群和限制
- [x] 新手路径有一条已实测的 dry-run / preview-only 流程
- [x] `PROGRESS.md` 持续记录真实验证、覆盖率和未完成项

## Next Small Goals

1. 对外定位：更新 README / README_zh，明确当前是 Beta，适合技术用户试用；说明哪些步骤会触达微信 API，哪些只是本地渲染。
2. 新手闭环：补一条不需要真实微信凭证的本地体验路径：`init` → `new` → `render` → `preview` → `cover`。（源码构建二进制已实测 `init` → `new` → `render` → `cover` → `check`）
3. 文档一致性：把 `PROGRESS.md`、`docs/GETTING_STARTED.md`、`docs/USER_GUIDE.md` 的安装、状态和风险描述统一。（已完成首轮，后续随功能变化继续维护）
4. 结构清理：`src/radar.rs` 已完成首轮拆分，分出 `radar/cli.rs`、`radar/store.rs`、`radar/analyze.rs`、`radar/scrape.rs`；Markdown 已拆出 `markdown/parser.rs`、`markdown/inline.rs`、`markdown/plain.rs`、`markdown/blocks.rs`；AI 命令编排已拆到 `src/ai_workflow.rs`；初始化向导已拆到 `src/init.rs`；本地草稿创建/写入已拆出 `src/draft.rs`；文章包状态和移动已拆到 `src/bundle.rs`；内部 target trait 已拆到 `src/plugin.rs`，微信草稿发布已成为第一个 `PublishTarget`，Zola 导出已成为第一个 `ExportTarget`；封面 style/HTML/PNG 辅助已回收到 `src/cover.rs`；ship 一键发布编排已拆到 `src/ship.rs`；通用 `publish --target` / `export --target` 命令开始承接插件化核心。
5. 自动化风险：浏览器自动化已明确为本地辅助驾驶，不绕过扫码/验证码/审核/最终人工确认；后续继续补真实微信回归清单。
6. 长期产品化：路线已确定为 CLI 稳定核心 → 插件化扩展点 → Obsidian 插件正式化 → WordPress / Ghost 等平台 → 本地 App / Pro 版。
7. 产品收口：先让用户会用，再继续扩能力；项目整体评估、飞书路线判断和近期阶段计划见 `docs/PRODUCT_EVALUATION_ZH.md`
8. 主推路径：普通文章路径与飞书路径的推荐入口已收口到 `docs/RECOMMENDED_WORKFLOWS_ZH.md`
9. 插件入口：`obsidian-plugin/` 已经开始承担正式首页入口之一；下一步重点不再是“承认它存在”，而是继续补真实首次体验证据和入口细节
10. 执行计划：当前阶段的里程碑、完成标准和推进顺序已收口到 `docs/EXECUTION_PLAN_ZH.md`
11. 产品包装：当前产品形态、三层结构和正式输入/入口层已收口到 `docs/PRODUCT_WRAP_ZH.md`
12. 源码审计：基于当前真实源码的结构判断、风险点和近期优先级已收口到 `docs/CODEBASE_AUDIT_ZH.md`

## Immediate Next Step

v0.4.2 已公开发布：真实微信回归、插件首页、飞书和照片路径共 11 份脱敏证据已归档，release workflow 完成五平台构建与资产 smoke，官方下载的 macOS ARM64 包已通过 SHA-256 和无凭证 smoke。下一步是收集技术用户 Beta 反馈，优先处理真实首次使用和微信后台变动带来的问题；最终微信公众号发表仍由人工确认。

## Completed

### 基础
- `init` / `status` / `check` — 基础脚手架
- `--json` / `--config` 全局 flag
- `doctor` / `workspace` / `workflow-registry` / `evidence-status` / `release-check` / `layout-recipes` / `layout-audit` / `wechat-health` / `status` / `check` / `preflight` / `preview` / `push` / `draft-from-inbox` / `intake feishu ... --draft` / `intake photos ... --draft` 在 `--json` 下返回命令专属结构化对象，便于 Agent / 插件直接读取本地可用性、入口建议、正式工作流契约、证据文件状态、v0.4.2 release gate 状态、排版配方、排版审计结果、浏览器自动化登录态、阶段列表、产物状态、本地发布质量门、`media_id` 和下一步动作；`workflow-registry` 现在还为每条正式路径提供 `user_value`，让插件首页能解释“这条路径能帮用户保留什么素材、先确认什么”；插件和脚本推荐全局前置 `--json`，这些结构化入口也兼容后置 `--json` 以降低手工 CLI 使用摩擦；其中 `doctor` 只检查本地首次使用环境，不触发微信 API、不打开 Chrome，`workflow-registry` 只暴露内置只读契约，`evidence-status` 只检查证据文件是否存在、不打开图片、不替代人工脱敏审查，`release-check` 只聚合 release gate 文档勾选状态和证据文件状态、不触发微信 API 或浏览器自动化，`preflight` 只做本地只读聚合检查、不触发微信 API 或浏览器自动化，`draft-from-inbox --push` / `intake feishu ... --draft --push` 还会补充 `pushed`、`media_id`、`stage`、`next_step`；其余命令仍保持兼容的 `{"output":"..."}` 包装
- `intake feishu <file>` / `--minute-token <token>` / `--latest` / `--query <关键词>` — 飞书秒记导出文本、指定 token、最近妙记或关键词搜索结果导入 `Inbox/Feishu/`；官方秒记链路会按 `minute_token` 复用既有 Inbox 文件；加 `--draft` 后继续生成可编辑文章草稿，加 `--preview` 后本地渲染并打开 HTML 预览
- `draft-from-inbox ... --preview --no-open` / `intake feishu ... --draft --preview --no-open` — 自动化友好的预览路径：生成 HTML 和 draft JSON，但不拉起系统浏览器，适合 CI、脚本和后续 Agent 编排
- `draft-from-inbox ... --push` / `intake feishu ... --draft --push` — 生成草稿后直接继续执行 `push --render`；`--push` 与 `--preview` 互斥，且 `intake feishu` 下必须搭配 `--draft`
- 飞书默认保守模式已固化到 help text / AGENTS / 文档：推荐先走 `intake feishu ... --draft --preview`，只有显式 `--push` 才表示继续推进到微信草稿；本地 preview 与微信公众号后台 preview-send 现已明确分层

### 渲染与发布
- `render` — Markdown → WeChat HTML + draft.json（Block 模板系统 + inline CSS）
  - 支持 `--humanize` flag 在渲染时去 AI 味
  - 支持 `--author` / `--thumb` 覆盖
- 支持 theme 配置（default/warm/dark/geek/paper/magazine/notebook/classic/forest/sunset/ocean/mono/editorial/zen/newsletter/academic/cyber/letter/mist/gallery/moonlit/porcelain/fieldnote），通过 Theme 系统注入 inline CSS
- `preview` — 本地 HTML 预览；默认打开系统浏览器，`--no-open` 时只生成并输出 HTML 路径
- `push` — 原生 WeChat API 推送（无需 md2wechat）
  - **自动上传本地图片**：push 时扫描 HTML 里的本地 src，逐个上传微信素材库，替换为 CDN URL
- `update-draft` — 更新已有微信草稿
- `publish --target wechat-draft` — 通用发布 target 入口，当前复用微信草稿发布能力
- `export --target zola` — 通用导出 target 入口，默认仍兼容原 `export <article.md>`
- `humanize` — 独立去 AI 味命令，in-place 修改文章

### 一键发布 (ship)
- `ship` — cover + render + push + configure + export 全流程
- headless 模式端到端验证通过

### 浏览器自动化 (CDP)
- `login` — 首次扫码登录，保存 cookie
- `wechat-health` — 发布前检查持久 profile/session 是否能复用，并返回是否需要重新扫码
- `configure` — headless 自动配置草稿设置
- `configure moban` — 按 `[template].name` 自动插入微信后台模板；未配置时跳过
- 全部步骤稳定：

| 步骤 | 状态 |
|------|------|
| 原创声明 | ✅ |
| 赞赏 | ✅ |
| 留言 | ✅ |
| 创作来源 | ✅ |
| 预览 | ✅ |
| 合集 | ⏸ 已禁用 |

- 实现: `src/publish.rs` (编排) + `src/cdp.rs` (CDP 原语) + `src/publish_steps.rs` (步骤)

### Block 模板系统
`:::blockname` 语法，20 种 block：
`book-info` / `intro` / `callout` / `steps` / `summary` / `figure` / `checklist` / `cover`
`key-points` / `pull-quote` / `letter-card` / `scene-card` / `closing-card` / `compact-links` / `photo-grid` / `meta-strip` / `quote-card` / `divider` / `concept-card` / `emotion-card`

### Theme 系统
- `theme = "default"|"warm"|"dark"|"geek"|"paper"|"magazine"|"notebook"|"classic"|"forest"|"sunset"|"ocean"|"mono"|"editorial"|"zen"|"newsletter"|"academic"|"cyber"|"letter"|"mist"|"gallery"|"moonlit"|"porcelain"|"fieldnote"` in moonpub.toml
- `Theme::section_style()` 生成 section 级 inline CSS
- 普通 Markdown 标题、首段导语、段落、行内高亮/删除线、引用、分割线、带 caption 图片、表格、无序/有序/任务列表和三反引号代码块统一走微信兼容 inline CSS 排版

### 去 AI 味（Humanize）
- 6 阶段规则处理：填充短语 → AI词汇 → 排比 → 修饰 → 结论 → 破折号

### AI 写作
- `write` / `expand` / `polish` / `ship --ai` 支持按 `[ai]` 配置切换 provider / model / api key
- `draft-from-inbox <inbox.md>` 支持把飞书秒记等 Inbox 原始素材整理成可继续编辑的文章草稿
- 当前内置 provider：`deepseek` / `openai`

### 封面生成（Cover）
- `moonpub cover [--style ...] [--screenshot]`
- 10 套 HTML 模板：dark / clean / minimal / warm / serif / gradient / literary / ink / sunset / forest
- 封面模板会转义 frontmatter 中的标题、副标题和作者文本，避免特殊字符破坏 HTML
- `cover` / `ship` 共用封面标题回退：`title` → 正文 H1 → 第一行有效正文 → 文件名；标题为空时自动提摘要，避免出现 `无标题`

### Radar
- 热点样本管理与标题建议

### 状态追踪
- `.moonpub/status.jsonl` — render/push/ready/published 状态自动记录

### WeChat API 客户端
- `src/wechat.rs` — access_token / draft_add / draft_update / upload_image / upload_image_url
- 完全替换 md2wechat，零外部 CLI 依赖，仅依赖 `ureq`

## 项目结构

```
src/
  main.rs          # 入口
  app.rs           # 命令路由与用例编排
  ai_workflow.rs   # write/polish/expand/ship --ai 的 AI 调用和文章文件写回
  cli.rs           # CLI 解析
  config.rs        # TOML 配置
  init.rs          # init 交互/非交互配置生成和 .env 更新
  draft.rs         # 本地草稿创建、AI 文章写入、草稿路径和重复文件校验
  bundle.rs        # ArticleBundle、stage 判断、文章包移动
  plugin.rs        # PublishTarget trait、能力元数据、publish context/outcome
  error.rs         # 错误类型与工具函数
  article.rs       # frontmatter 解析
  render.rs        # 文章级渲染 → HTML + draft.json
  ship.rs          # 一键发布编排：cover → thumb upload → render → push → optional export
  markdown.rs      # Markdown → WeChat HTML 顶层分发
  markdown/parser.rs # ::: block 与属性解析
  markdown/inline.rs # 行内 Markdown 渲染
  markdown/plain.rs  # 普通 Markdown 段落/表格/列表/引用/代码块渲染
  markdown/blocks.rs # ::: fence block 渲染
  push.rs          # WeChat API 推送
  wechat.rs        # WeChat API 客户端
  publish.rs       # 浏览器自动化流程编排
  cdp.rs           # CDP 底层辅助（chromiumoxide）
  publish_steps.rs # 微信编辑器各配置步骤
  cover.rs         # 封面样式解析、HTML 模板、HTML 写入和 PNG 截图辅助
  theme.rs         # 渲染主题
  humanize.rs      # 去 AI 味
  illustrate.rs    # Block 模板渲染
  radar.rs         # 热点管理
  ...
docs/
  REFERENCES.md          # 参考项目
  BROWSER_AUTOMATION.md  # 浏览器自动化参考
  RELEASE_CHECKLIST.md   # 首版发布验收清单
  RELEASE_NOTES_v0.4.1.md # GitHub Release 发布说明
  LAUNCH_READY_ZH.md     # v0.4.1 最终可发布状态
  LAUNCH_ARTICLE_ZH.md   # 中文发布文章发布稿
  LAUNCH_PLAN_ZH.md      # 首版对外发布计划和进度条
  LAUNCH_DEMO_ASSETS_ZH.md # v0.4.1 首发演示素材记录
  LAUNCH_SCREENSHOT_CHECKLIST_ZH.md # 首发截图交付清单
  WECHAT_REGRESSION_CHECKLIST_ZH.md # 真实微信草稿回归清单
  PROGRESS.md            # 项目进度
  WORKFLOW.md            # 完整发布工作流
```

## 依赖

仅 `ureq`（HTTP + TLS）+ `chromiumoxide`（CDP 浏览器控制）。其他全部纯 Rust std。

## 待办

- [ ] 浏览器自动化：合集选择、封面图设置、发表按钮
- [ ] 降低浏览器自动化登录摩擦：继续补真实 session 过期场景、恢复提示和截图/录屏证据
- [ ] 文章排版优化（间距、配色、书封卡片）

## 版本日志

- 2026-06-23: **Radar CLI 拆分** — `parse_radar_command` 与子命令参数解析移入 `src/radar/cli.rs`，`cargo nextest run --all-features radar::` 17 tests passed
- 2026-06-23: **Radar store 拆分** — `TrendSample`、JSONL 编解码和趋势样本 add/list/load 移入 `src/radar/store.rs`，`cargo nextest run --all-features radar::` 17 tests passed
- 2026-06-23: **Radar analyze 拆分** — `analyze_article`、tokenize 和分析结果格式化移入 `src/radar/analyze.rs`，`cargo nextest run --all-features radar::` 17 tests passed
- 2026-06-23: **Radar scrape 拆分** — `scrape_radar`、页面抓取、标题提取和 URL 编码移入 `src/radar/scrape.rs`，`cargo nextest run --all-features radar::` 17 tests passed
- 2026-06-25: **Radar import/suggest 拆分** — `import_csv` / `parse_csv_row` 移入 `src/radar/import.rs`，`suggest_titles` 与其辅助逻辑移入 `src/radar/suggest.rs`，并新增 `suggest_titles_includes_formula_and_trend_reference` 直接回归测试；`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 通过
- 2026-06-26: **可配置 AI provider** — `[ai]` section 支持 `provider` / `model` / `api_key`，`write` / `expand` / `polish` / `ship --ai` 统一走 `AiProvider`，当前支持 `deepseek` / `openai`；`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 通过
- 2026-06-26: **可配置模板插入** — `[template].name` 接入 `moonpub configure` 的 `moban` 步骤，`step_moban` 通过 CDP 自动打开模板菜单、按名称选中并点击“添加到正文”；未配置模板名时软跳过；`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 通过
- 2026-06-27: **Markdown 渲染层收口拆分** — 行内 Markdown 渲染移入 `src/markdown/inline.rs`，普通段落/表格/列表/引用/代码块移入 `src/markdown/plain.rs`，`src/markdown.rs` 回到顶层 block 分发；新增 `markdown/blocks.rs` 直接单测覆盖 book-info、steps、summary、figure、checklist、generic fallback，并修复 checklist 正文残留 `]` 的排版问题；`cargo nextest run --all-features` 194 tests passed，`cargo llvm-cov nextest --all-features --summary-only` 总行覆盖 57.43%，`markdown/blocks.rs` 行覆盖 63.09%
- 2026-06-27: **文章排版精修** — 正文主题新增 `editorial` / `zen`，普通 Markdown 支持首段导语、`####` 小标题和带 caption 的图片 figure；README / README_zh / AGENTS 主题数量同步为 14；`cargo nextest run --all-features` 197 tests passed，`cargo llvm-cov nextest --all-features --summary-only` 总行覆盖 58.16%
- 2026-06-27: **更多正文主题与行内强调** — 正文主题新增 `newsletter` / `academic` / `cyber`，主题数量同步为 17；行内 Markdown 支持 Obsidian 常用 `==高亮==` 和 `~~删除线~~`，输出微信兼容 inline CSS；`cargo nextest run --all-features` 198 tests passed，`cargo llvm-cov nextest --all-features --summary-only` 总行覆盖 58.66%
- 2026-06-27: **清单与重点排版增强** — 普通 Markdown `- [x]` / `- [ ]` 任务列表渲染为微信兼容 checklist，新增 `key-points` 和 `pull-quote` 两种文章排版块；`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 通过，201 tests passed，`cargo llvm-cov nextest --all-features --summary-only` 总行覆盖 59.10%
- 2026-06-27: **本地排版预览 UTF-8 修复** — 用临时排版样例文章覆盖普通 Markdown、14 种 Block、任务清单、重点卡片和金句卡片，浏览器 QA 发现本地 `.html` 预览缺少 charset 会导致中文乱码；现已为本地预览 HTML 增加 doctype、UTF-8 meta 和 viewport，draft JSON 仍保留微信正文片段；桌面 1280px 与移动 390px 检查无横向溢出、无控制台错误；`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 通过，201 tests passed，`cargo llvm-cov nextest --all-features --summary-only` 总行覆盖 59.14%
- 2026-06-27: **本地预览阅读宽度模拟** — 本地 `.html` 预览增加浅色背景、居中 720px 阅读卡片和移动端自适应，用浏览器 QA 确认桌面 1280px 下正文居中、移动 390px 下无横向溢出；微信 draft JSON 继续不包含本地预览外壳；`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 通过，201 tests passed，`cargo llvm-cov nextest --all-features --summary-only` 总行覆盖 59.16%
- 2026-06-27: **本地预览实际宽度校准** — 为本地预览卡片增加 `box-sizing: border-box`，修正 padding 导致 720px 预览卡片实际渲染为 768px 的偏差；浏览器复查桌面 1280px 下 main 实际宽度为 720px、无横向溢出、无控制台错误；`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 通过，201 tests passed，`cargo llvm-cov nextest --all-features --summary-only` 总行覆盖 59.17%
- 2026-06-27: **同文章旧草稿安全清理** — `push` 创建新微信草稿前读取同文章包旧 `.media_id`，新草稿创建成功并更新本地 `.media_id` 后，按旧 `media_id` best-effort 删除旧草稿；不按标题批量删除，避免误删同名草稿；新增 `previous_media_id_trims_existing_bundle_id` 回归测试
- 2026-06-27: **主题与 Block 文档口径修正** — `moonpub init` 生成配置、Getting Started、User Guide 和 Workflow 的正文主题列表同步为 17 种，User Guide 的 Block 列表同步为 14 种，并新增 sample config 回归测试防止主题提示再次落后；`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 通过，202 tests passed，`cargo llvm-cov nextest --all-features --summary-only` 总行覆盖 59.65%
- 2026-06-28: **飞书草稿预览 no-open** — `draft-from-inbox` 与 `intake feishu --draft --preview` 新增 `--no-open`，可只生成本地预览 HTML / draft JSON 而不打开系统浏览器，便于 CI 和 Agent 自动化先跑通链路；同步 README / README_zh / User Guide / help text，新增 CLI 与 preview 单元测试；已用 `/private/tmp/moonpub-flow-check` 跑通 `intake feishu --latest --draft --preview --no-open`，真实飞书 latest → Inbox → AI 草稿 → render → no-open 预览成功
- 2026-06-28: **草稿后续动作提示** — `draft-from-inbox` / `intake feishu --draft` 生成草稿后输出 `next: moonpub push <draft.md> --render`，让用户预览确认后能直接进入微信草稿推送下一步；新增消息格式单元测试，并用临时 vault 验证真实输出
- 2026-06-28: **普通预览 no-open** — `moonpub preview <article.md> --no-open` 支持只校验并输出本地 HTML 路径，不拉起系统浏览器，方便服务端、CI 和手机确认流中的非交互预览检查
- 2026-06-29: **工作流命令结构化 JSON** — `preview`、`push`、`draft-from-inbox`、`intake feishu ... --draft` 在全局 `--json` 下改为返回命令专属字段，而不是统一包进 `{"output":"..."}`；新增 `preview_paths`、`PushOutput` 和对应 app/push/preview 回归测试，文档同步为自动化/插件用法；`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` fresh 通过，241 tests passed
- 2026-06-29: **飞书秒记幂等重跑** — 官方飞书秒记链路（`--minute-token` / `--latest` / `--query`）重复导入时会按 `minute_token` 复用并更新同一个 `Inbox/Feishu/*.md`；`draft-from-inbox` 与 `intake feishu --draft` 重复生成草稿时复用原草稿文件，不再因为已存在而失败；结构化 `--json` 额外返回 `action: "created" | "updated"` 供 Agent 判断首次生成还是重跑更新
- 2026-06-29: **飞书草稿自动继续 push** — `draft-from-inbox --push` 与 `intake feishu ... --draft --push` 会在草稿生成后直接复用 `push --render` 继续推到微信草稿；`--push` 与 `--preview` 互斥，`intake feishu --push` 必须显式搭配 `--draft`；对应的结构化 `--json` 额外返回 `pushed`、`media_id`、`stage`、`next_step`
- 2026-06-30: **飞书默认保守流规则固化** — CLI help text、`AGENTS.md` 和 `PROGRESS.md` 已统一为同一口径：飞书链路默认推荐 `--draft --preview`，只有显式 `--push` 才继续推进到微信草稿；本地 `preview` 与微信公众号后台 preview-send 已明确区分
- 2026-07-01: **飞书秒记真实闭环验证** — 使用真实 Obsidian articles 路径运行 `moonpub --articles "<path>" --json intake feishu --latest --draft --preview --no-open`，成功拿到真实 `inbox_path` / `draft_path` / `html_path`；继续运行 `moonpub --articles "<path>" --json intake feishu --latest --draft --push`，成功恢复微信会话、进入编辑器、自动完成原创/赞赏/留言/创作来源，并完成微信公众号后台“预览发送到手机”，最终返回 `pushed: true`、真实 `media_id` 和 `stage: ready`。同时确认当前 CLI 实际入口是 `--articles`，不是 `--vault`；当时结构化 JSON 只验证了全局前置 `--json` 写法。
- 2026-07-01: **整体评估与阶段计划** — 新增 `docs/PRODUCT_EVALUATION_ZH.md`，基于当前代码、README、ROADMAP 和 PROGRESS 现状，明确项目当前应定位为“本地发布内核”，飞书秒记应先作为内部正式模块而非立刻拆新项目，并把后续重点收口为“先让用户会用，再继续扩能力”；同步 README_zh / ROADMAP / PROGRESS 入口说明。
- 2026-07-01: **推荐工作流入口收口** — 新增 `docs/RECOMMENDED_WORKFLOWS_ZH.md`，把当前最主推的两条用户路径单独写清楚：`已有 Markdown 文章 → 本地预览 → 微信草稿` 与 `飞书秒记 → 草稿 → 预览 → 微信草稿`；同步 README_zh / USER_GUIDE / WORKFLOW 的入口提示，减少“用户拿到项目却不知道先跑哪条路径”的理解成本。
- 2026-07-02: **产品包装层文档收口** — 新增 `docs/PRODUCT_WRAP_ZH.md`，把 MoonPub 现在的产品形态明确收口为三层：`Core`、`Input Workflows`、`User Surfaces`，同时明确它当前不是无人值守机器人、不急着拆飞书新项目，而是先作为本地发布内核继续长正式输入工作流和正式入口层；README / README_zh / USER_GUIDE / AGENTS / PROGRESS 入口同步更新，减少“能力已经很多，但用户仍然不知道这项目到底是什么”的理解成本。
- 2026-07-01: **Obsidian 插件入口补全** — 为 `obsidian-plugin/` 新增独立 README，明确它当前是“在 Obsidian 里调用本地 MoonPub CLI 的实验性入口”，不是独立发布器；同时修正插件里“预览文章”不应强依赖微信凭证的问题，并同步 USER_GUIDE / README_zh 的插件入口说明，让第三个用户入口的边界更清楚。
- 2026-07-01: **README 首页入口收口** — README / README_zh 第一屏新增“你是哪类用户 / Pick Your Entry Path”，把当前主推入口明确拆成三条：已有 Markdown 文章、飞书秒记素材、Obsidian 插件入口；让用户在进入全部命令说明前，先知道自己该走哪条路径。
- 2026-07-01: **Obsidian 插件设置页补齐** — `obsidian-plugin/main.ts` 新增最小设置页，支持配置 `MoonPub 可执行文件路径` 和 `Articles 根目录`；命令执行从拼 shell 字符串改为 `execFile` 参数数组，减少路径和空格问题；插件 README / USER_GUIDE 同步补上设置说明，`npm run build` 重新验证通过。
- 2026-07-01: **Obsidian 插件发布前提示接入 capabilities** — 插件发布命令执行前会调用 `moonpub capabilities --json`，展示“是否联网 / 是否可能打开 Chrome / 常见前置条件”等轻量风险提示；同时去掉仅凭 Obsidian 进程 `process.env.WECHAT_*` 就硬阻断发布的误判逻辑，避免和 MoonPub 本身的 `.env` / `~/.moonpub.env` 配置优先级打架；插件 README / USER_GUIDE / AGENTS 说明同步更新，`npm run build` 与 Rust 全量检查重新通过。
- 2026-07-01: **Obsidian 插件补状态检查入口** — 插件新增“检查当前文章状态”命令，直接调用 `moonpub check <当前文件>`，把 `publishable`、`html`、`draft_json`、`media_id` 这些最关键信息提炼成 Notice，减少用户在 Obsidian 里来回切终端判断当前文件阶段的成本；插件 README / USER_GUIDE 同步更新，`npm run build` 与 Rust 全量检查重新通过。
- 2026-07-01: **check 命令结构化 JSON** — `moonpub check <article.md>` 在全局 `--json` 下改为返回 `command`、`article_path`、`html_path`、`draft_json_path`、`media_id_path`、`has_*` 和 `publishable` 字段，不再只剩 `{"output":"..."}` 文本包装；Obsidian 插件已改为优先消费这份 JSON，而不是脆弱地解析纯文本；README / README_zh / USER_GUIDE / AGENTS 同步更新，`npm run build` 与 Rust 全量检查重新通过。
- 2026-07-01: **check 命令补下一步建议** — `moonpub check --json` 进一步返回 `next_command` / `next_step`，让状态检查不只告诉你“缺什么”，还直接告诉你“下一步建议做什么”；Obsidian 插件的状态提示也同步展示下一步建议，继续把“用户不知道接下来该点哪一步”的问题往下压。
- 2026-07-01: **status 命令结构化 JSON** — `moonpub status` 在全局 `--json` 下改为返回 `command: "status"` 和按 `drafts` / `ready` / `published` 分组的 `stages` 数组，每个文件项都会带 `file`、`slug`、`latest_status`、`latest_detail`；这样插件 / App / Agent 不必再反解析终端文本，就能知道当前文章池的整体阶段分布。
- 2026-07-01: **status 命令补全局下一步建议** — `moonpub status --json` 进一步返回 `next_command` / `next_step`：优先指向第一篇 drafts，再到 ready、published，最后回退到 `moonpub new`；这样上层入口不只知道“当前池子里有什么”，还知道“现在最合理的第一步是什么”。
- 2026-07-01: **Obsidian 插件补整体状态入口** — 插件新增“查看整体文章池状态”命令，直接消费 `moonpub status --json`，在 Obsidian 里就能快速看到 `drafts` / `ready` / `published` 数量和推荐下一步，不必先回终端判断整个文章池；插件 README / USER_GUIDE / README_zh / AGENTS 同步更新，继续把插件收成真正可用的第三入口。
- 2026-07-01: **workspace 统一入口 JSON** — 新增 `moonpub workspace [--json]`，把 `workspace_kind`、推荐入口 `entry_path`、文章池阶段分布、内置 capability 摘要和推荐下一步一次性收口，作为 CLI / Obsidian / 后续 Agent 的高层入口对象；同步 CLI help、README / README_zh / USER_GUIDE / PRODUCT_EVALUATION / AGENTS，真实 `cargo run -- --json workspace` 已验证输出符合预期。
- 2026-07-02: **Obsidian 插件改接 workspace 入口** — 插件里的“查看整体文章池状态”不再只读 `status --json`，而是直接消费 `workspace --json`，把推荐入口、阶段数量、风险 target 和下一步建议一次性展示出来；这让插件开始真正复用高层协议，而不是自己拼装工作区语义。
- 2026-07-02: **统一 Inbox 元数据正式落代码结构** — `src/intake.rs` 不再手写拼接飞书 frontmatter，而是引入统一 `InboxMetadata` 结构负责读写 `source` / `status` / `created` / `type` / `external_id` 等字段；飞书仍保留 `minute_token` 兼容字段，但官方秒记链路的复用逻辑开始优先走通用 `external_id`，并补了“旧文件只有 minute_token 也能被复用和升级”的回归测试，为后续照片 / 语音输入源复用同一套 Inbox 模型打基础。
- 2026-07-02: **Obsidian 插件补工作区工作台弹窗** — “查看整体文章池状态” 现在不再只弹一条压缩 Notice，而是会在读取 `moonpub workspace --json` 后继续打开一个简短工作台弹窗，把推荐入口、drafts/ready/published 阶段数量、推荐下一步命令和会联网/打开 Chrome 的风险边界分开展示；目标是继续降低“用户拿到插件但不知道先点什么”的理解成本。`npm run build` 与 `cargo fmt --all -- --check` 已重新通过。
- 2026-07-02: **Obsidian 插件补当前文章工作台弹窗** — “检查当前文章状态” 现在也不再只留一条 `publishable / html / draft_json / media_id / next` 提示，而是会把 `check --json` 展开成当前文章工作台，分开展示可发布性、Markdown/HTML/`draft.json`/`media_id` 产物状态、对应路径和推荐下一步动作；继续把插件从“命令入口”往“可理解的首页/向导”推进。`npm run build` 已重新通过。
- 2026-07-02: **Obsidian 插件补飞书主推入口** — 插件新增“导入最近一条飞书妙记并生成草稿预览”和“导入最近一条飞书妙记并推进到微信草稿”两条命令，分别直连 `intake feishu --latest --draft --preview --json` 与 `intake feishu --latest --draft --push --json`；这样用户即使不先回终端，也能从插件里直接起飞书主工作流。`obsidian-plugin/README.md`、`docs/USER_GUIDE.md`、`README_zh.md` 已同步更新。
- 2026-07-02: **飞书导入后自动回到草稿** — Obsidian 插件在飞书入口成功生成草稿后，如果目标草稿位于当前 vault 内，会自动尝试打开那篇草稿，减少“导入完还要自己去目录里找文件”的割裂感；相关 README / USER_GUIDE / README_zh 说明已同步更新。
- 2026-07-02: **插件补飞书结果工作台** — 飞书入口执行完成后，插件会继续打开一个“飞书结果工作台”弹窗，集中展示 `inbox_path`、`draft_path`、可选 `html_path`、是否已推进微信草稿、`media_id` 和推荐下一步；这样飞书链路在插件里开始从“命令触发器”往“工作流结果页”演进。
- 2026-07-02: **飞书结果工作台补后续动作** — 飞书结果工作台现在不只是展示产物路径和状态，还提供“打开草稿 / 检查草稿 / 预览草稿 / 推进到微信草稿”等按钮，让用户可以直接从结果页继续下一步，而不必再回到命令面板重新找命令。
- 2026-07-02: **Obsidian 插件接入照片正式入口** — 插件新增“导入当前图片所在目录并生成照片草稿预览”，当用户当前打开一张图片时，可以直接把该图片所在目录作为一组照片素材传给 `intake photos ... --draft --preview --json`，并复用与飞书相同的草稿打开和结果工作台流程。这样 MoonPub 开始拥有第二条正式插件素材入口。
- 2026-07-02: **工作区工作台开始承担插件首页角色** — `查看整体文章池状态` 打开的工作区工作台现在不只展示 `workspace --json` 状态，还提供“检查当前文章 / 预览当前文章 / 导入最近飞书妙记 / 导入当前图片目录”等快捷动作。这样插件首页开始从“状态面板”往“统一入口页”收口。
- 2026-07-02: **插件首页补上下文感知推荐** — 工作区工作台现在会根据当前打开的是 Markdown、图片还是其他文件，提示更适合的入口动作；这让插件首页不再只是罗列按钮，而开始具备最基础的“此刻你更应该走哪条路径”的判断。
- 2026-07-02: **插件首页补首次建议步骤** — 工作区工作台现在会根据当前上下文直接列出“第一次建议步骤”，把推荐入口再继续展开成更具体的先后顺序，减少用户虽然知道该点哪个入口，但还不知道下一步先做什么的停顿。
- 2026-07-02: **插件首页命名显式化** — Obsidian 插件新增 `打开 MoonPub 首页` 命令，和已有的 `查看整体文章池状态` 一样都指向 `workspace --json`，但前者更明确承担首页语义；首页弹窗标题同步改为“MoonPub 首页工作台”，用户第一次上手时不再需要自己猜哪个命令才是首页。
- 2026-07-02: **首次使用向导收口** — 新增 `docs/FIRST_RUN_WALKTHROUGH_ZH.md`，把第一次体验推荐顺序单独写清楚：先进插件首页，再按飞书 / 照片 / 当前文章三类入口走到草稿和本地预览；README_zh / USER_GUIDE / RECOMMENDED_WORKFLOWS 已同步挂入口，减少用户第一次拿到项目时“看了很多文档，还是不知道先点哪里”的问题。
- 2026-07-02: **首次使用审计收口** — 新增 `docs/FIRST_RUN_AUDIT_ZH.md`，把当前文章、飞书、照片三条首次路径和插件首页按“真实证据 / 已通过 / 仍待补强”重新审计一遍，明确飞书是当前最成熟路径、照片入口已成形但真实用户证据仍弱于飞书；README_zh / USER_GUIDE / README 已同步挂入口。与首次体验直接相关的定向验证 `cargo nextest run --all-features app::tests::intake_photos_draft_preview_json_creates_inbox_draft_and_html app::tests::intake_feishu_draft_preview_json_creates_inbox_draft_and_html app::tests::ensure_preview_html_renders_html_before_returning_path ai::tests::call_ai_uses_test_override_when_present` 已通过。
- 2026-07-02: **首次体验取证清单收口** — 新增 `docs/FIRST_RUN_EVIDENCE_CHECKLIST_ZH.md`，把插件首页、飞书、照片三类首次体验证据需要补哪些截图 / 录屏 / 样例、按什么标准验收拆清楚；README_zh / USER_GUIDE 同步挂入口，后续可直接按同一清单补强真实用户证据。
- 2026-07-02: **源码审计基线收口** — 新增 `docs/CODEBASE_AUDIT_ZH.md`，基于 `src/cli.rs`、`src/app.rs`、`src/intake.rs`、`obsidian-plugin/main.ts` 和 CI workflow 的真实状态，补了一轮“当前代码到底长成了什么、风险点在哪里、下一步该先收产品还是先收工程”的源码级判断；明确当前最值得做的两件事是补首次体验真实证据，以及把 `app.rs` 里的协议输出继续模块化。
- 2026-07-02: **协议输出开始从 `app.rs` 抽层** — 新增 `src/protocol.rs`，把 `workspace` / `status` / `check` / `preview` / `push` / `draft-from-inbox` / `intake ... --draft` 这层结构化输出 builder 从 `src/app.rs` 迁出，`app.rs` 回到“命令编排 + 调用协议输出”的职责边界；对应纯协议测试同步迁移到协议模块，定向 `cargo nextest` 已验证通过。
- 2026-07-02: **草稿后续编排开始收共用 helper** — `src/app.rs` 里的 `IntakeFeishu`、`IntakePhotos`、`DraftFromInbox` 三条链路原本各自重复处理“草稿后是否 preview / 是否 push / JSON 怎么回 / 文本怎么回”的逻辑；现已收成共用 `finalize_draft_follow_up` helper，让 `app.rs` 更像工作流编排而不是三份并排拷贝。定向 `cargo nextest` 已覆盖飞书、照片、preview 和协议回归。
- 2026-07-02: **`app.rs` 继续收重复编排** — `src/app.rs` 里原本反复出现的配置加载、飞书 `source` 分派，以及 `step-test` / `test-zanshang` / `test-yulan` / `test-chuangzuo` 这组同形态微信后台自动化入口，已继续下沉为私有 helper；`app.rs` 从约 `983` 行降到约 `910` 行，`run()` 更接近命令路由器而不是“把每个分支细节都展开一遍”的总控文件。`cargo fmt --all -- --check` 与 8 条定向 `cargo nextest` 回归已通过。
- 2026-07-02: **`Push / Publish / Preview` 命令包装继续收口** — `src/app.rs` 里 `Push`、`Publish --target wechat-draft`、`Preview` 三个分支原本各自处理 JSON/文本包装和 target 分发；现已继续收成共用 helper，让发布主线更清楚地区分“命令路由”和“具体命令包装”两层职责。`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings` 与 9 条定向 `cargo nextest` 回归已通过。
- 2026-07-02: **`Render / Cover / Humanize` 文件型命令继续收口** — `src/app.rs` 里 `Render`、`Cover`、`Humanize` 三个分支原本还各自展开文件读取、frontmatter 解析、humanize 写回和命令包装；现已继续下沉为私有 helper，并把“就地 humanize 文件”收成共用函数，避免 `render --humanize` 与独立 `humanize` 再维护两份同样的文件读写逻辑。`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings` 与 9 条定向 `cargo nextest` 回归已通过。
- 2026-07-02: **`app_support` 内部编排模块落地** — 新增 `src/app_support.rs`，把已经稳定的命令级 helper 和上下文结构从 `src/app.rs` 真正迁出，包括配置加载、render/cover/humanize、preview/push/publish 包装，以及 draft follow-up 编排；`src/app.rs` 从约 `1027` 行降到约 `688` 行，开始更接近纯命令路由层。`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings` 与上一轮 10 条定向 `cargo nextest` 回归已通过。
- 2026-07-02: **`draft follow-up` 内部模块继续拆出** — 新增 `src/app_draft_follow_up.rs`，把 `draft-from-inbox` / `intake ... --draft` 的 preview / push / JSON / 文本收尾从 `src/app_support.rs` 进一步拆成独立内部模块；`app_support` 回到更薄的命令包装协调层。同步新增 `docs/first-run-evidence/README.md` 与 `docs/first-run-evidence/NOTES.md`，把首次体验证据从“只有清单”推进到“有统一归档目录和记录模板”。`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings` 与 10 条定向 `cargo nextest` 回归已通过。
- 2026-07-02: **首次体验证据归档目录固定** — `docs/first-run-evidence/` 继续补出 `homepage/`、`feishu/`、`photos/` 三个固定归档位，并在各目录下放入最小说明文件；`NOTES.md` 也补成按入口分类的记录模板。这样首次体验证据已经不只是“知道该补什么”，而是开始具备稳定的仓库归档结构，后续真实截图 / 录屏可以直接按目录落盘。
- 2026-07-02: **`app_support` 再按命令语义收薄一层** — 新增 `src/app_article_commands.rs` 与 `src/app_publish_commands.rs`，把 `render` / `cover` / `humanize` / `preview` 这组本地文章命令包装，以及 `push` / `publish --target wechat-draft` / 浏览器自动化错误包装这组微信发布命令包装，从 `src/app_support.rs` 继续拆开；`app_support` 回到更纯粹的配置加载和飞书 source 分派协调层。这样后面再继续收工程边界时，判断标准会更清楚，不必再围着一个混合协调层反复堆 helper。
- 2026-07-02: **照片输入源第一版落地** — 新增 `intake photos <文件或目录...>`，会把一组真实照片文件导入 `Inbox/Photos/`，按统一 Inbox 元数据写入 `source: photos`、`type: photo-note`、`external_id`、`captured_at` 等字段，并基于真实文件路径/大小/修改时间生成素材稿；如果继续加 `--draft` / `--preview` / `--push`，则复用和飞书相同的后续链路。这意味着 MoonPub 现在开始拥有第二条正式输入工作流，而不再只有飞书这一条来源。
- 2026-07-02: **照片链路 JSON 协议补齐** — `intake photos ... --draft --json` 现在会明确返回 `command: "intake-photos"`，不再复用错误的 `intake-feishu` 命令名；这样插件 / Agent / App 在接结构化结果时，终于可以正确区分飞书输入流和照片输入流。
- 2026-07-02: **`--draft --preview --json` 真实生成预览产物** — `draft-from-inbox`、`intake feishu`、`intake photos` 在 JSON 模式下，如果显式带了 `--preview`，现在会先真实执行 render/preview，再返回 `html_path`，不再出现“响应里给了 html_path，但文件其实还没生成”的协议错位。
- 2026-07-02: **照片链路补 app 级行为测试** — 为 `intake photos ... --draft --preview --no-open --json` 增加了真实 app 层回归测试，并引入最小的 test-only AI 响应替换点，确保测试里可以稳定验证 Inbox、draft、html、draft.json 产物是否一起落地，而不是只测 JSON builder。
- 2026-07-02: **飞书链路补同级 app 级行为测试** — `intake feishu <file> --draft --preview --no-open --json` 现在也补上了同等级的 app 层回归测试，验证 Inbox、draft、html 产物和结构化 JSON 一起成立。至此，飞书与照片两条正式输入工作流的测试等级开始真正对齐。
- 2026-07-03: **真实微信公众号后台回归通过** — 使用真实 Obsidian articles 根目录与当前 source build 跑通 `moonpub --articles "<path>" test-yulan --headed`：持久登录态恢复成功，无需扫码，进入草稿编辑器后完成原创声明并成功执行微信公众号后台预览发送。随后继续跑通 `moonpub --articles "<path>" configure --headed`：原创声明、赞赏、留言、创作来源 `个人观点，仅供参考` 和预览发送均成功；`[template].name` 未配置时模板插入按设计软跳过。当前结论是主后台辅助配置链路在本机当前登录态上可用，但仍需截图/录屏归档和后续 UI 变更回归。
- 2026-07-03: **真实文章池状态展示修正** — `workspace --json` / `status --json` 现在会在文章已经物理位于 `Articles/ready/` 或 `Articles/published/` 时优先反映当前阶段，并从同目录 `.media_id` 读取 detail 兜底，不再被 `.moonpub/status.jsonl` 里较旧的 `rendered` / `pushed` 状态误导；已用真实 Obsidian 文章池确认“欢迎来到这片林子...”显示为 `ready`，并补了 `status_report_prefers_ready_stage_over_stale_rendered_status` 回归测试。
- 2026-07-03: **push 后台自动化支持隔离 profile** — `push <article.md>` 与 `publish <article.md> --target wechat-draft` 新增 `--temporary-profile` 参数，并通过 `PublishContext` 传到草稿创建成功后的 `auto_configure`；默认仍复用持久 profile，显式开启时仅后台自动化使用一次性 Chrome profile，微信 API 推草稿本身不变。README / README_zh / AGENTS / help text 已同步。
- 2026-07-03: **正文排版选择增强** — 新增 `letter` / `mist` / `gallery` 三套正文主题，分别面向信笺随笔、安静生活记录和图文展陈；新增 `letter-card` / `scene-card` / `closing-card` 三种 Block，用于开篇短笺、场景记录和温柔收束。主题总数从 17 增至 20，Block 总数从 14 增至 17，README / README_zh / User Guide / Getting Started / Workflow / docs 首页 / slides / AGENTS 已同步；`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 通过，282 tests passed。
- 2026-07-03: **生活图文排版块增强** — 新增 `photo-grid` 与 `meta-strip` 两种微信兼容 Block：前者用于两列照片组和图片说明，后者用于日期、地点、天气、心情等生活记录元信息。Block 总数从 17 增至 19，AI 提示、README / README_zh / User Guide / docs 首页 / slides / AGENTS 已同步；`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 通过，284 tests passed。
- 2026-07-03: **排版配方文档** — 新增 `docs/LAYOUT_RECIPES_ZH.md`，按生活随笔、照片记录、读书笔记、技术文章和日报周报给出可复制的主题 + Block 组合，避免用户只看到组件清单却不知道如何搭配；README_zh 和 User Guide 已挂入口。本轮为文档增强，验证以 markdown 链接与格式检查为主。
- 2026-07-03: **排版配方渲染回归** — 为生活随笔配方新增整篇 Markdown 渲染测试，覆盖 `meta-strip` / `photo-grid` / `scene-card` / `closing-card` 组合，确保配方文档里的关键结构能持续被 `md_to_wechat_html` 正常渲染；`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 通过，285 tests passed。
- 2026-07-03: **排版配方命令入口** — 新增 `moonpub layout-recipes` / `moonpub --json layout-recipes`，把生活随笔、照片记录、读书笔记、技术文章和日报周报五类排版配方暴露成 CLI 可发现能力；JSON 输出包含 `guide` 和 `recipes[]`，每个配方包含 `id`、`title`、`best_for`、`themes`、`blocks`，供插件 / Agent 直接展示。CLI / app / protocol 回归测试已补齐；`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 通过，289 tests passed。
- 2026-07-04: **微信公众号浏览器自动化健康检查** — 新增 `moonpub wechat-health` / `moonpub --json wechat-health`，发文前可先检查持久 Chrome profile 和 `session.json` 是否还能进入公众号后台；输出 `ready` / `needs_login`、profile 模式、session 文件状态和下一步命令，URL 会脱敏去掉 query token。CLI / CDP / protocol 回归测试已补齐；`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 通过，293 tests passed。
- 2026-07-04: **微信公众号 profile lock 友好提示** — 真实运行 `moonpub --json wechat-health` 时发现持久 Chrome profile 已被现有 MoonPub 自动化窗口占用，Chrome 返回 `SingletonLock` / `ProcessSingleton` 并拒绝启动。现将这类底层错误收敛为可读提示：关闭现有 MoonPub 自动化 Chrome 窗口，或用 `--temporary-profile` 临时隔离验证；避免误判成微信 token 过期。随后真实重跑 `cargo run -- --json wechat-health` 已返回 `status: "ready"`、`session_file_exists: true` 和脱敏后的 `current_url`；`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 通过，295 tests passed。
- 2026-07-09: **headless 登录恢复体验收口** — `configure` / `push` 后自动配置这类不可见浏览器流程现在只静默复用已保存的持久 session；如果登录态不可复用，会立即提示用户执行 `moonpub login` 或改用 `configure --headed` 扫码，不再在用户看不见二维码时等待 120 秒。临时 profile 也会明确提示它无法复用持久 session，避免用户误以为每次都必须登录。
- 2026-07-04: **微信 API 代理诊断开关** — 将 `MOONPUB_DEBUG_PROXY=1` 正式化为微信 API 代理排障开关，调试日志会显示当前请求使用的代理或 `<none>`，但 URL 会去掉 query，避免泄露 `access_token`。本轮补了 `redact_url_query_removes_token_from_debug_url` 回归测试；`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings` 和微信代理相关定向 `cargo nextest` 已通过。
- 2026-07-04: **紧凑来源索引 Block** — 新增 `:::compact-links` block，用于把 QunMind 日报的“参考来源 / 完整素材链接”渲染成 12px 小字号资料索引。标题只作为普通强调文本，唯一链接入口保留在 `原文：完整 URL`，避免文末链接区视觉上压过正文；URL 会做 HTML 转义，避免 query 参数破坏链接属性。本轮已通过 `cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features`，297 tests passed。
- 2026-07-04: **日报周报排版配方** — `layout-recipes` 新增 `daily-report` 配方，推荐 `notebook` / `newsletter` / `editorial` 主题和 `intro` / `divider` / `summary` / `callout` / `compact-links` 组合，用于 AI/Web3 日报、资料索引和可追溯信息流；README / README_zh / User Guide / docs 首页 / slides / help text 已同步。
- 2026-07-04: **日报周报配方渲染回归** — 新增整篇 Markdown 渲染测试，覆盖 `daily-report` 推荐组合里的 `intro` / `divider` / `summary` / `callout` / `compact-links`，并确认 `compact-links` 仍保持小字号、完整原文 URL 唯一链接入口和 query 参数 HTML 转义。
- 2026-07-04: **合集开篇排版配方** — `layout-recipes` 新增 `collection-opener` 配方，面向栏目第一篇、付费合集序章和个人小专栏开场，推荐 `editorial` / `mist` / `letter` 主题与 `meta-strip` / `intro` / `letter-card` / `scene-card` / `closing-card` 组合；配方文档、README / README_zh、User Guide 和 help text 已同步，并补了整篇 Markdown 渲染回归测试。
- 2026-07-05: **口述随记排版配方** — `layout-recipes` 新增 `spoken-note` 配方，面向飞书妙记、散步录音、跑步后口述复盘和随口想法整理成文，推荐 `letter` / `mist` / `notebook` 主题与 `meta-strip` / `intro` / `letter-card` / `summary` / `closing-card` 组合；配方文档、README / README_zh、User Guide 和 help text 已同步，并补了整篇 Markdown 渲染回归测试。
- 2026-07-06: **飞书草稿接入口述随记配方** — `draft-from-inbox` / `intake feishu ... --draft` 的 AI 提示现在会在检测到 `source: feishu-minutes` 时明确引导 `spoken-note` 结构，要求 frontmatter 优先 `theme: letter`，正文优先 `intro` / `letter-card` / `summary` / `closing-card`，让飞书妙记和散步口述默认生成更克制、更像口述随记的草稿，而不是只在 `layout-recipes` 里提供配方但实际不使用。
- 2026-07-07: **公众号排版 HTML 审计入口** — 新增 `moonpub layout-audit <html>` / `moonpub --json layout-audit <html>`，用于检查渲染后的微信 HTML 是否包含公众号编辑器高风险结构，例如 `<script>` / `<style>` / `<div>`、`class` / `id` 属性、`position:absolute/fixed/sticky`、`display:grid`、`@media`、`float` 和完整 HTML 外壳。该入口先作为独立质量门，不改变现有 render / push 发布链路；后续可以接入插件或 CI。
- 2026-07-07: **生活合集排版主题增强** — 新增 `moonlit` / `porcelain` / `fieldnote` 三套正文主题，分别面向月下隐林式克制开篇、瓷白蓝灰慢读和照片/散步生活手记；`layout-recipes` 新增 `quiet-opening` 与 `memory-note` 两个配方，继续复用现有 `meta-strip` / `intro` / `letter-card` / `scene-card` / `photo-grid` / `closing-card`，不新增复杂 Block。源码测试会验证新配方渲染后通过 `layout-audit` 错误检查；真实微信端视觉效果仍需后续预览取证。
- 2026-07-09: **项目完成度与 v0.4.2 release gate 收口** — 将项目状态校准为技术用户 Beta 完成度约 89%，普通用户顺利上手度约 70-75%，v1.0 产品化约 55-60%；同步最新事实：23 套正文主题、315 条测试、PR #88 / #89 / #90 已合并。新增 `docs/RELEASE_GATE_v0.4.2_ZH.md`，明确 v0.4.2 不是继续加功能，而是补无凭证 smoke、真实微信人工回归、首次体验截图/录屏和 release 文案口径。
- 2026-07-09: **v0.4.2 本机 release smoke 通过** — 当前源码构建的 release 二进制已通过 `cargo build --release --all-features`、`target/release/moonpub --version`（`moonpub 0.4.2`）和 `target/release/moonpub init /private/tmp/moonpub-smoke-v042`。这证明源码 release build 的无凭证初始化路径可用，但正式 v0.4.2 发布前仍需下载/打包后的 release 资产 smoke、真实微信人工回归和截图/录屏证据归档。
- 2026-07-09: **本地首次使用 doctor 与插件首页诊断** — 新增 `moonpub doctor` / `moonpub --json doctor`，只检查本地可用性、Articles 根目录和配置状态，不触发微信 API、不打开 Chrome、不读取真实 secret；Obsidian 插件首页现在先消费 `doctor --json` 展示“当前是否可开始”，再消费 `workspace --json` 展示工作区状态，并把首页动作按本地安全、生成草稿、触达微信分组。
- 2026-07-10: **工作流契约发现入口** — 参考 qintopia-agent-os 的 registry / contract 思路，但不复制外部代码，新增 `moonpub workflow-registry` / `moonpub --json workflow-registry`，把当前文章、飞书妙记、照片记忆和微信公众号草稿推进四条正式路径暴露成内置只读契约；JSON 包含 `id`、`package`、`owner`、`safe_start_command`、`next_command`、风险标记、生产边界、证据状态和文档入口，供插件 / App / Agent 直接发现工作流，不再从 README 反解析。`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 通过，319 tests passed。
- 2026-07-10: **插件首页接入工作流契约** — Obsidian 插件首页现在会在 `doctor --json` 与 `workspace --json` 之间读取 `workflow-registry --json`，把正式工作流、安全起点、风险标记和证据状态展示在首页工作台；读取失败时只隐藏该区域，不阻断旧版 CLI 的首页使用。`npm run build` 已验证通过。
- 2026-07-10: **插件首页工作流安全入口按钮** — 首页里的 `workflow-registry` 区域现在会把当前文章、飞书妙记和照片记忆三条契约映射成可点击的安全开始按钮，分别进入检查/预览、飞书草稿预览和照片草稿预览；`wechat-draft` 只展示边界提示，不从首页直接触发微信草稿推进。`npm run build` 已验证通过。
- 2026-07-10: **当前文章工作台补继续操作** — `检查当前文章状态` 结果页现在不只展示 `check --json` 状态，还提供“预览当前文章”按钮；当 `draft.json` 已存在时，再显示“推进到微信草稿”按钮，并继续复用发布前风险提示。`npm run build` 已验证通过。
- 2026-07-10: **当前文章工作台接入排版审计** — 当 `check --json` 显示 HTML 已存在时，Obsidian 当前文章工作台现在会显示“排版审计”按钮，调用 `moonpub --json layout-audit <html>` 并用弹窗展示错误、警告和推荐下一步；该动作只检查本地 HTML，不触发微信 API、不打开浏览器。
- 2026-07-10: **飞书 / 照片结果工作台接入排版审计** — `intake feishu ... --draft --preview` 与 `intake photos ... --draft --preview` 返回 `html_path` 时，Obsidian 结果工作台现在也会显示“排版审计”按钮，继续调用 `moonpub --json layout-audit <html>`；该动作只检查本地 HTML 兼容风险，不触发微信 API、不打开浏览器。`npm run build` 已验证通过。
- 2026-07-10: **排版审计结果页补预览动作** — Obsidian 排版审计弹窗现在会提供“打开 HTML 预览”按钮，让用户在看到 `layout-audit` 错误 / 警告后可以直接打开本地 HTML 对照查看；`layout-audit` 本身仍然只做本地检查，不触发微信 API，也不会自动打开浏览器。
- 2026-07-10: **工作台补复制下一步命令** — Obsidian 首页工作台、当前文章工作台和飞书 / 照片结果工作台现在都提供“复制下一步命令”按钮，方便技术用户把 `next_command` 带回终端继续执行；该动作只写剪贴板，不执行命令，也不会触发微信 API。
- 2026-07-10: **发布前本地 preflight 质量门** — 新增 `moonpub preflight <article.md>` / `moonpub --json preflight <article.md>`，聚合检查 Markdown / HTML / `draft.json`、复用 `layout-audit` 审计 HTML，并把缺 `.media_id` 视作“尚未推微信草稿”的 warning；该命令不触发微信 API、不打开 Chrome，返回下一步建议给插件 / Agent / CLI 用户。`cargo fmt --all -- --check`、`git diff --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 通过，324 tests passed。
- 2026-07-10: **Obsidian 插件接入 preflight 质量门** — 当前文章工作台和飞书 / 照片结果工作台新增“发布前检查”按钮，调用 `moonpub --json preflight <article.md>` 展示本地产物、排版审计和 `.media_id` 状态；同时把插件里的结构化命令统一成全局前置 `--json` 顺序，作为插件内部更稳定的调用规范。`npm run build` 已验证通过。
- 2026-07-10: **结构化命令兼容后置 `--json`** — 为 `doctor`、`workspace`、`workflow-registry`、`layout-recipes`、`layout-audit`、`wechat-health`、`capabilities`、`status`、`check`、`preflight`、`preview`、`push`、`draft-from-inbox` 和 `intake` 增加后置 `--json` 兼容解析；插件内部仍推荐全局前置 `--json`，但用户手工执行 `moonpub workspace --json` / `moonpub check demo.md --json` 不再掉回文本输出。
- 2026-07-10: **v0.4.2 证据文件状态入口** — 新增 `moonpub evidence-status` / `moonpub --json evidence-status`，按 `docs/first-run-evidence/` 下首页、飞书、照片和真实微信回归四类固定文件检查证据是否已经归档；该命令只检查文件存在，不打开图片、不读取图片内容、不替代人工脱敏审查。同步补 `wechat/` 证据目录和 release gate 说明。
- 2026-07-10: **证据状态汇总字段** — `moonpub evidence-status` 文本输出新增 `present/required/missing` 汇总和缺失路径清单，JSON 新增 `required_count`、`present_count`、`missing_count`、`missing_paths`，方便插件、CI 和 release gate 不展开 `sections` 也能直接判断还缺多少证据；命令仍然只做本地文件存在性检查。
- 2026-07-10: **插件首页接入证据状态** — Obsidian 首页工作台现在会在 `doctor` / `workflow-registry` / `workspace` 之外读取 `moonpub --json evidence-status`，展示 v0.4.2 证据目录、已归档数量、缺失数量和部分缺失路径；读取失败不阻断首页使用，也不会打开截图或替代人工脱敏审查。
- 2026-07-10: **证据状态严格门** — `moonpub evidence-status --strict` 现在会在缺少必需证据文件时非零退出，供 v0.4.2 release 脚本或 CI gate 使用；默认 `evidence-status` / `--json evidence-status` 仍保持只读报告和插件首页友好行为，不打开图片、不读取图片内容、不替代人工脱敏审查。
- 2026-07-10: **v0.4.2 release 总门禁入口** — 新增 `moonpub release-check` / `moonpub --json release-check` / `moonpub release-check --strict`，聚合 `docs/RELEASE_GATE_v0.4.2_ZH.md` 通过标准勾选状态和 `docs/first-run-evidence/` 文件存在检查；当前真实输出显示本地 release smoke 与 CI / Windows smoke 已记录通过，但真实微信回归、11 个证据文件、文档一致性记录和隐私审查记录仍未完成。该命令只读，不触发微信 API、不打开浏览器、不扫描图片内容。
- 2026-07-11: **release 证据清单口径对齐** — `docs/RELEASE_GATE_v0.4.2_ZH.md` 的真实证据归档段落已从 7 张核心截图补齐为 `moonpub evidence-status` 实际要求的 11 个必需文件，避免 release 文档和门禁命令对“还差哪些证据”给出不同答案；证据本身仍是 `4/11 present, 7 missing`，未宣称完成。
- 2026-07-11: **飞书用户身份与真实导入复核** — 飞书官方 `minutes +search` / `+detail` 调用明确传入 `--as user`，并补参数序列回归测试；以授权用户身份真实搜索和隔离目录 `intake feishu --latest` 均成功，最新妙记已写入 `Inbox/Feishu/`，未调用 AI 或微信。插件执行 MoonPub 命令也统一按开发构建仓库根目录 / 正式二进制 Articles 根目录选择工作目录，确保 CLI 继续按既有优先级读取本地 `.env`。同时归档脱敏 `feishu-draft-opened.png`，首次体验证据推进到 `8/11 present, 3 missing`。
- 2026-07-11: **JSON 输出边界收口** — `app.rs` 不再维护“哪些命令返回专属 JSON”的第二份命令清单；该能力下沉到 `Command::has_structured_json_output`，让 CLI 命令定义和协议声明保持同一处。补齐飞书 / 照片草稿型与非草稿型入口回归，避免新增命令时误把结构化 JSON 再包装成 `{"output": ...}`。
- 2026-07-11: **当前源码 release smoke 复核** — 在 JSON 输出边界收口后的当前源码上重新构建 `--release --all-features`，并验证版本、无凭证 `init` 和 `doctor --json` 路径；release 二进制正确返回 v0.4.2，并对尚未建立 Articles 工作区的新初始化位置给出本地诊断建议。正式 v0.4.2 下载资产 smoke 仍是 release gate 的独立要求。
- 2026-07-11: **照片输入源边界收口** — 将照片素材递归采集、批次建模、稳定 external_id 和 Inbox 写入下沉至 `src/intake/photos.rs`；`src/intake.rs` 保留统一 InboxMetadata、幂等查找和飞书路径。公共 `intake photos` CLI、生成的 frontmatter 与草稿后续编排保持不变，照片与飞书 app 级回归同时覆盖。
- 2026-07-11: **依赖安全修复** — RustSec 审计发现直接依赖 `anyhow 1.0.102` 命中 `RUSTSEC-2026-0190` 的 `downcast_mut()` 健全性告警；已将锁定版本升级到修复版 `1.0.103`。重新执行严格 clippy、347 条 nextest 和 cargo audit，均通过。
- 2026-07-11: **素材外发确认与照片视觉分析** — Obsidian 插件的飞书、照片入口现在先经过阻塞式确认：飞书明确完整转写会发送到当前 AI provider；普通照片路径明确只发送文件路径、文件名、大小和修改时间，不上传图片像素。新增独立的“视觉分析当前图片目录”入口和 `intake photos ... --analyze-images`：仅在第二次明确确认后将最多 5 张 jpg/jpeg/png/webp 图片（单张 8 MiB、合计 20 MiB）发送到 OpenAI，谨慎描述可见信息并写回 Inbox，固定标为“需人工核对”；默认路径和视觉路径都不自动推进微信草稿。已补 CLI、app、Inbox 覆盖写入、OpenAI-only 边界和 HEIC 跳过回归；`cargo fmt --all -- --check`、严格 clippy、353 条 nextest、`cargo audit --no-fetch` 和 `obsidian-plugin npm run build` 均通过。真实个人照片外发和插件体验截图仍需用户在确认窗口中明确同意后取证。
- 2026-07-12: **真实飞书与照片草稿已按确认生成** — 用户已明确授权后，最新飞书妙记完整转写与当前照片目录的文件元数据分别发送给已配置 AI 服务，均成功落为本地 Inbox、Markdown 草稿、HTML 预览和 `draft.json`；未触达微信 API 或浏览器自动化。照片草稿预检暴露出渲染器仍输出 `class` 属性，现已移除正文和 Block 渲染里的这类属性，并以 `layout-audit` 回归约束微信公众号兼容性；完整回归后已重渲染真实照片草稿，`preflight` 通过。插件工作台切换也改为在首页关闭后的下一轮事件循环再打开后续弹窗，避免结果页被首页遮挡。三张脱敏截图仍待补齐，release gate 仍为 `8/11`，不能宣称完成。
- 2026-07-12: **插件首页单实例收口** — 真实取证发现重复触发首页会堆叠多个 `MoonPubWorkspaceModal`，使飞书/照片结果工作台被残留首页遮挡。`runStatus()` 现在会先关闭已打开的首页实例再展示新状态，保留一次明确的当前工作台；插件构建已通过。照片结果工作台与草稿打开脱敏证据已归档，随后飞书结果工作台证据也已补齐，release gate 已通过 `11/11` 严格证据检查。
- 2026-07-12: **v0.4.2 首次体验证据收口** — 在用户已授权的真实插件流程中重新完成最新飞书妙记到本地草稿/HTML 预览，并归档裁切脱敏的飞书结果工作台截图；照片结果工作台与草稿打开截图也已完成隐私审查。`moonpub evidence-status --strict` 显示 `11/11`，`moonpub release-check --strict` 全部通过；两条输入流均未推进微信草稿，最终发布仍不自动化。
- 2026-07-12: **插件构建纳入 CI** — `.github/workflows/build.yml` 的 `test` job 与 `.github/workflows/release.yml` 的 tag release `test` job 现在都会先执行 `obsidian-plugin` 的 `npm ci && npm run build`；PR #133 对应 GitHub Actions run `29199006304` 已实际通过新增的 `Build Obsidian plugin` 步骤以及 Rust `test`、`windows-smoke`，避免插件无法打包时仍可合并或被带入正式 release。
- 2026-07-13: **Unix release 归档 smoke** — tag release workflow 现在会解压 Linux / macOS 的 `.tar.gz` 资产，在干净临时目录依次运行 `--version`、`--help`、`init`、`new`、`render`、`check`；Windows `.zip` 原有 smoke 保持不变。这样在 GitHub Release 创建前就能发现 Unix 打包资产不可运行的问题，发布后仍需从真实下载资产补人工 smoke。
- 2026-07-13: **修复 Unix release smoke 大小写路径** — 首次 v0.4.2 tag workflow 的 Linux ARM64 archive smoke 拦截了 `Archive-Smoke.md` 与后续 `archive-smoke.md` 不一致的问题；Release 未创建。已将 PR / tag workflow 的 smoke 标题统一为小写连字符，并在本地大小写敏感路径对打包 release 二进制完成 `--version -> init -> new -> render -> check` 验证；严格 Clippy、插件构建和 353 条 nextest 也通过。
- 2026-07-13: **v0.4.2 正式发布与官方下载 smoke** — 修复 PR #134 合并后，重新指向 `v0.4.2` tag 的 GitHub Actions run `29214942987` 已通过插件构建、五个平台构建、Linux/macOS archive smoke、Windows zip smoke，并创建 [v0.4.2 Release](https://github.com/qiaopengjun5162/moonpub/releases/tag/v0.4.2)。本机已下载 `moonpub-macos-arm64.tar.gz`，SHA-256 校验通过，解压后的官方二进制完成 `--version -> init -> new -> render -> check` 无凭证 smoke。
- 2026-07-10: **插件首页接入 release 总门禁** — Obsidian 首页工作台现在会在证据状态之外读取 `moonpub --json release-check`，展示 v0.4.2 发布门禁状态、未完成 gate 和下一步命令；读取失败不阻断首页使用。`npm run build` 已验证通过。
- 2026-07-10: **Obsidian+AI 内容生产线参考收口** — 根据用户提供的文章素材新增 `docs/OBSIDIAN_AI_PIPELINE_REFERENCE_ZH.md`，把“本地 Markdown 是资产、Inbox 优先、AI 做整理和草稿辅助、内容要可追溯、少插件先跑通主线”等原则映射到 MoonPub；明确不把项目改成通用知识库、Obsidian 教程或无人值守发布器。
- 2026-07-10: **yichen-skills 参考融合地图** — 新增 `docs/YICHEN_SKILLS_REFERENCE_ZH.md`，把 `summary`、`x-article-draft-uploader`、`wechat-local-vault`、`codex-memory`、`chatgpt-web-research` 和公众号归档等参考模块映射到 MoonPub 的可吸收原则、长期研究方向和不应进入主线的高风险边界；近期只吸收草稿优先、dry-run、私有 vault、closeout/audit 和隐私边界，不复制外部代码。
- 2026-07-10: **yichen-video-content 参考补强** — 二次复查 `mcncarl/yichen-skills` 后，将 `yichen-video-content` 的逐句作用拆解、标题诊断、结构诊断和文字洁癖吸收到 MoonPub 的长期参考图里；未来可映射为只读 `draft-audit` 内容质量门，但不把短视频抓取、剪辑或“爆款模板承诺”做进 v0.4.x / v0.5 主线。
- 2026-07-10: **微信公众号归档输入源设计收口** — 参考 `wechat-mp-batch-exporter` 的安全边界和输出分层，但不复制外部代码，新增 `docs/WECHAT_ARCHIVE_WORKFLOW_ZH.md`，把未来公众号归档路线限定为“用户显式提供公开 URL -> Inbox -> Draft -> Preview”优先；批量历史、阅读数、评论和凭证辅助采集都必须保持显式确认和本地安全边界。
- 2026-07-10: **公众号归档风险分层补强** — 参考 `moore-wechat-article-downloader` 的场景划分，把公众号归档继续细分为已知 URL、Exporter 历史列表、订阅增量、代理历史和浏览增强五层；MoonPub 近期仍只考虑已知公开 URL -> Inbox，代理增强、WebView 注入、评论/指标采集和系统代理修改都不进入默认能力。
- 2026-07-10: **公众号 URL 转 Markdown 最小入口补强** — 参考 `mp-weixin-to-md`，把公众号归档 Phase 1 进一步限定为“链接 -> 标准 Markdown”的最小入口：图片默认保留远程 URL，只有显式选择才下载本地 assets；本地 HTML 只作为验证页或网络失败后的 fallback；不内置 Cookie，不绕过登录或验证页。
- 2026-07-10: **Khoj 知识助手参考收口** — 新增 `docs/KHOJ_REFERENCE_ZH.md`，把 `khoj-ai/khoj` 的本地优先知识库、语义搜索、Obsidian 多入口、Agent 和自动化能力映射为 MoonPub 的长期参考：近期不做完整二脑、向量搜索引擎或聊天 SaaS；未来如做 `search` / `ask`，第一步必须只读、返回来源、不触发微信 API、不打开浏览器、不写回文件。
- 2026-07-10: **Identity Skill 视觉流程参考收口** — 新增 `docs/IDENTITY_SKILL_REFERENCE_ZH.md`，把 `Sac-Y/identity-skill` 的参考图驱动、阻塞确认、素材拆分、素材评审和视觉 QA 台账映射为 MoonPub 的长期视觉流程参考；近期只用于官网、插件首页、本地 App 首屏或封面样张的设计纪律，不把 MoonPub 扩成个人网站生成器，也不复制外部模板代码。
- 2026-07-10: **Ian Handdrawn PPT 解释图参考收口** — 新增 `docs/IAN_HANDDRAWN_PPT_REFERENCE_ZH.md`，把 `helloianneo/ian-handdrawn-ppt` 的文章封面 / 正文解释图叙事规划、语义版式、短文字质量门和 contact sheet QA 映射为 MoonPub 的长期视觉资产参考；近期不做 PPT 生成器，不默认每篇文章生图，也不让图像生成成为发布前置依赖。
- 2026-07-10: **AstrBot README 上手表达参考收口** — 新增 `docs/ASTRBOT_README_REFERENCE_ZH.md`，把 `AstrBotDevs/AstrBot` 中文 README 的第一屏入口聚合、支持矩阵、安装路径分层、路线图和社区承接映射为 MoonPub 的文档产品化参考；近期不做聊天机器人框架、模型路由平台、Web 管理后台或插件市场包装。
- 2026-07-10: **Horizon 雷达日报参考收口** — 新增 `docs/HORIZON_REFERENCE_ZH.md`，把 `Thysrael/Horizon` 的多源素材、去重评分、来源保留、日报草稿和可解释候选映射为 MoonPub 的雷达/素材筛选长期参考；近期不做新闻抓取平台、自动资讯站、邮件分发系统或后台定时发布机器。
- 2026-07-09: **Obsidian 插件补 setup fallback 工作台** — 当插件找不到 `moonpub` CLI，或飞书 / 照片素材入口缺少 `Articles 根目录` 时，现在会打开修复工作台列出安装 CLI、填写可执行文件路径和补根目录等步骤，不再只依赖一条容易错过的 Notice；真实 Obsidian 截图证据仍需后续按取证清单补。
- 2026-07-09: **v0.4.2 onboarding PR CI 确认通过** — PR #93 / #94 均已合并，且 `test` 与 `windows-smoke` 通过；#94 对应 GitHub Actions run `29009548275` 显示 `test` pass、`windows-smoke` pass。v0.4.2 release gate 中的“CI / Windows smoke”已可标记通过，但真实微信路径截图/录屏和首次体验证据仍未补齐。
- 2026-07-01: **修复 PR Windows smoke 的 release 构建 flags** — PR `windows-smoke` workflow 之前直接执行 `cargo build --release`，仍会继承 `.cargo/config.toml` 里的 `target-cpu=native`，在 GitHub Windows runner 上触发 `STATUS_ILLEGAL_INSTRUCTION`；现已为 `.github/workflows/build.yml` 的 `windows-smoke` job 显式清空 `RUSTFLAGS`，与 `release.yml` 保持一致。
- 2026-06-30: **修复 login 浏览器生命周期 bug** — `moonpub login` 之前在打开浏览器后提前丢掉 `Browser` 句柄，导致 CDP 会话被取消并报 `oneshot canceled`；现已在登录路径显式保活浏览器直到扫码完成和 session 保存，并新增资源保活回归测试
- 2026-06-30: **临时隔离 profile 模式** — `login` / `configure` / `step-test` / `test-zanshang` / `test-chuangzuo` / `test-yulan` 新增显式 `--temporary-profile`；默认稳定持久 profile 保持不变，临时模式使用一次性 Chrome profile，且不读写 `~/.config/moonpub/session.json`；CLI / CDP / publish 路由回归测试已补齐
- 2026-06-26: **Markdown fence renderer 拆分** — `render_fence_block` 与 fence 专属 renderer 移入 `src/markdown/blocks.rs`，`markdown.rs` 回到 Markdown segment 分发与 inline 渲染入口；`cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 通过
- 2026-06-23: **Markdown parser 拆分** — `MdBlock`、`parse_blocks`、`split_fence_props` 移入 `src/markdown/parser.rs`，`cargo nextest run --all-features markdown::` 9 tests passed
- 2026-06-23: **发布副驾驶定位** — README / README_zh / BROWSER_AUTOMATION / blog outline 统一说明：API 是稳定核心，CDP 是本地辅助驾驶，不绕过平台确认
- 2026-06-23: **草稿状态体验修正** — `push` / `ship` 创建微信草稿后本地文章包移动到 `Articles/ready/`，避免把未人工确认的草稿误标为 published
- 2026-06-23: **首版发布材料** — 新增 `docs/RELEASE_CHECKLIST.md` 和 `docs/LAUNCH_ARTICLE_ZH.md`，统一 slides 发布副驾驶口径
- 2026-06-23: **首跑体验修复** — 非交互 `moonpub init` 默认写入当前目录作为 articles root；源码构建二进制已在 `/tmp/moonpub-local-check` 跑通 `init` → `new` → `render` → `cover` → `check`
- 2026-06-23: **版本查询体验** — `moonpub --version` / `-V` 输出当前版本，便于 release 资产验证和用户排查
- 2026-06-23: **v0.4.0 release smoke test** — macOS amd64 资产可通过代理下载，sha256 校验通过，`moonpub --help` 可在 Apple Silicon/Rosetta 运行；`--version` 不存在，且非交互 `init` 写入占位 root 导致 `new` 失败，因此需要 v0.4.1
- 2026-06-23: **v0.4.1 release 初次触发失败** — tag 已推送，但 release workflow 在 macOS ARM64 构建 `ring` 时因 `target-cpu=native` 触发编译期断言；release build 已改为清空 `RUSTFLAGS`，待重新触发
- 2026-06-23: **v0.4.1 release smoke test 通过** — 重新触发 tag workflow 后五个平台资产全部产出；macOS ARM64 资产 sha256 通过，`--help` / `--version` 正常，release 二进制已跑通 `init` → `new` → `render` → `cover` → `check`
- 2026-06-24: **首版对外发布计划** — 新增 `docs/LAUNCH_PLAN_ZH.md`，明确最终目标、当前 87% 进度、可试用边界和下一步截图/真实微信回归；`docs/LAUNCH_ARTICLE_ZH.md` 补 v0.4.1 release 口径
- 2026-06-24: **首发演示素材记录** — 用 v0.4.1 release 二进制在 `/private/tmp/moonpub-launch-demo` 生成本地预览 HTML、封面 HTML、draft JSON、`check` 和 `status` 输出；Codex 内置浏览器因 `file://` 安全策略未能导出截图，截图仍待普通浏览器或专门流程完成
- 2026-06-24: **首发截图与微信回归清单** — 新增 `docs/LAUNCH_SCREENSHOT_CHECKLIST_ZH.md` 和 `docs/WECHAT_REGRESSION_CHECKLIST_ZH.md`，把截图交付物、真实微信凭证前置条件、安全边界和回归记录模板拆清楚
- 2026-06-24: **Draft 模块拆分** — `new_article` / `write_article_file` 和草稿路径重复校验移入 `src/draft.rs`，`app.rs` 回到命令路由调用；新增 draft 单元测试
- 2026-06-24: **Cover 辅助边界拆分** — 封面 style 解析、cover HTML 路径/写入和 Chrome 截图辅助回收到 `src/cover.rs`，`app.rs` 不再直接拼封面路径或 headless Chrome 参数；新增 cover 辅助测试
- 2026-06-24: **Ship 编排模块拆分** — `ship_article` 移入 `src/ship.rs`，`app.rs` 只负责命令路由；新增 ship 导出源选择测试，保护 ready/published 状态边界
- 2026-06-24: **Init 模块拆分** — `init_config` 和交互向导移入 `src/init.rs`，`app.rs` 不再包含初始化提示和 `.env` 写入细节；新增 `.env` 凭证更新测试
- 2026-06-24: **AI Workflow 模块拆分** — `write` / `polish` / `expand` / `ship --ai` 编排移入 `src/ai_workflow.rs`，`app.rs` 只负责命令路由；新增 expand frontmatter 保留测试
- 2026-06-24: **首发安全文本素材** — 从 v0.4.1 release demo 生成 `--version` / `check` / `status` 安全文本输出和截图状态说明；后续已补本地预览和封面 PNG
- 2026-06-24: **v0.4.1 最终发布说明** — 新增 `docs/LAUNCH_READY_ZH.md` 和 `docs/RELEASE_NOTES_v0.4.1.md`，把“能否给别人用”、安装命令、已验证内容和剩余人工项收口到一个对外版本口径
- 2026-06-24: **中文发布文章收口** — `docs/LAUNCH_ARTICLE_ZH.md` 去掉发布前提醒，补 v0.4.1 Release 链接和完整本地试用路径，可直接作为对外发布稿继续人工润色
- 2026-06-24: **首发截图资产** — 用 v0.4.1 release 二进制生成带真实内容的本地预览和 literary 封面，并通过 Chrome headless 导出 `docs/assets/launch/01-preview.png` / `02-cover.png`
- 2026-06-24: **发布文章配图完成** — `docs/LAUNCH_ARTICLE_ZH.md` 已嵌入本地预览和封面截图，首发对外材料只剩真实微信回归证据
- 2026-06-24: **手动发布状态修复** — `mark-published` 现在会把 ready 文章包移动到 `Articles/published/`，与 `push` / `ship` 状态边界保持一致
- 2026-06-24: **长期路线图** — 新增 `ROADMAP.md`，明确 v0.4.2 真实微信回归、v0.5 插件化、v0.6 Obsidian 插件、v0.7 多平台和 v1.0 商业化方向
- 2026-06-24: **插件化核心设计** — 新增 `docs/PLUGIN_ARCHITECTURE_ZH.md`，明确 v0.5 的 ArticleBundle、PublishTarget、ExportTarget、capabilities 和安全边界
- 2026-06-24: **ArticleBundle 核心拆分** — 新增 `src/bundle.rs`，集中 `ArticleBundle`、stage 识别和 ready/published 移动逻辑；`status` / `push` / `mark-published` 行为保持兼容
- 2026-06-24: **PublishTarget 核心拆分** — 新增 `src/plugin.rs`，定义内部 `PublishTarget`、`PublishContext`、`PublishOutcome` 和调度 helper；微信草稿发布成为第一个内置 target，`push_article` 行为保持兼容
- 2026-06-24: **Capabilities 命令** — 新增 `moonpub capabilities [--json]`，输出内置 target 的网络/浏览器能力和人工确认风险提示，供 Obsidian 插件和未来本地 App 做发现
- 2026-06-24: **ExportTarget 核心拆分** — `src/plugin.rs` 新增 `ExportTarget` / `ExportContext` / `ExportOutcome`，Zola 导出成为第一个内置 export target，并出现在 `capabilities` 输出中
- 2026-06-24: **通用 target 命令入口** — 新增 `moonpub publish <article.md> --target wechat-draft [--render]`，`export` 支持 `--target zola` 且保持旧用法兼容；Obsidian 插件 / 本地 App 可以用 capabilities 发现 target，再用通用命令调用；`cargo nextest run --all-features` 163 tests passed
- 2026-06-24: **Capability 命令模板** — `capabilities --json` 为每个 target 增加 argv 风格 `command` 和 `article_arg` 占位符，插件 / App 可以替换 `"{article}"` 后直接调用，不必硬编码命令形状；`cargo nextest run --all-features` 165 tests passed
- 2026-06-24: **Capabilities Schema 版本化** — `capabilities --json` 顶层新增 `schema_version` 和 `moonpub_version`，插件 / App 可在读取 target 命令模板前判断元数据兼容性
- 2026-06-24: **Capability 前置条件元数据** — `capabilities --json` 为内置 target 增加 `required_env` / `required_config`，插件 / App 可在执行前提示微信凭据或 Zola 配置缺失；`cargo nextest run --all-features` 168 tests passed
- 2026-06-25: **Windows PR smoke CI** — PR #48 在 `windows-latest` 上通过源码构建二进制无凭证 smoke：`--version` / `--help` / `init` / `new` / `render` / `check`；Windows release zip 仍待人工下载 smoke test
- 2026-06-25: **Windows release zip smoke workflow** — release workflow 在 `windows-latest` 上解压 `moonpub-windows-amd64.zip`，并对 zip 内 `moonpub.exe` 跑 `--version` / `--help` / `init` / `new` / `render` / `check`，把 release 资产验证自动化
- 2026-06-23: **封面文本转义** — title/digest/author 统一 HTML 转义，`cargo nextest run --all-features` 130 tests passed
- 2026-06-16: **创作来源 radio value 修复** — headed + headless 均稳定，ship 端到端验证通过
- 2026-06-16: **模块拆分收尾** — cdp.rs / publish_steps.rs / markdown.rs 从 publish.rs 和 render.rs 拆分
- 2026-06-15: **lib.rs 模块化** — 拆分为 cli / config / error / article / render / export / status / preview / system / push
- 2026-06-12: **auto_configure 完善** — 原创/赞赏/留言/创作来源/预览 自动化，合集暂跳过
- 2026-06-11: **浏览器自动化** — Rust chromiumoxide 实现 CDP 浏览器控制
- 2026-06-11: **结尾模板** — footer.rs 群二维码 + banner + CTA
- 2026-06-11: **封面集成** — render_article 支持 cover_html，ship 自动注入封面
- 2026-06-10: Block 模板 + Humanize + Cover + PR workflow + radar suggest
