# MoonPub CLI Progress

## Status

Beta / early adopter ready. Current repo version is v0.4.2; the latest verified public release assets remain v0.4.1, and the macOS ARM64 release binary has passed the no-credential first-run smoke test from a clean directory. Windows release assets exist; pull request CI passes a no-credential smoke test against a source-built Windows binary, and the release workflow now smoke-tests the packaged Windows zip before publishing release assets. It is usable by technical users who can configure WeChat credentials, but still needs live WeChat regression checks, broader platform smoke tests, screenshots/recordings, and module cleanup before calling it broadly stable.

## Final Goal

MoonPub 的最终目标：让作者从 Obsidian / Markdown 出发，用一个可审计、可复现、可本地运行的 Rust CLI，把文章稳定发布到微信公众号草稿，并同步导出到个人博客；对外使用时，用户应能按 README 完成安装、配置、预览、推送和故障排查。

长期路线见 [ROADMAP.md](ROADMAP.md)：先完成真实微信回归，再做插件化核心、Obsidian 插件正式化、WordPress / Ghost 等低风险多平台发布，最后探索本地 App 和 Pro 版。v0.5 插件化设计见 [docs/PLUGIN_ARCHITECTURE_ZH.md](docs/PLUGIN_ARCHITECTURE_ZH.md)。

## Progress Bar

整体进度：`█████████░` 87%

| 领域 | 进度 | 当前判断 |
|------|------|----------|
| 核心 CLI / 配置 / 状态 | `█████████░` 90% | 常用命令完整，仍可继续改善错误提示和 dry-run |
| Markdown 渲染 / Block / Theme | `█████████░` 92% | 已能产出微信 HTML，解析、行内语法、普通段落与 fence block 渲染已拆分；正文主题增至 17 套，并支持首段导语、h4 小标题、行内高亮/删除线、任务清单、重点卡片、金句卡片和带 caption 图片 |
| WeChat API 推送 | `████████░░` 85% | draft add/update/image upload 可用，仍需更多错误场景文档 |
| CDP 浏览器自动化 | `███████░░░` 70% | 核心步骤本地验证过，但微信 UI 会变，合集/发表仍未启用 |
| 对外安装 / Release | `█████████░` 92% | v0.4.1 release 已成功产出五个平台资产，macOS ARM64 已完成 release smoke test，Windows 源码构建二进制 PR smoke CI 与 release zip smoke workflow 已就位 |
| 文档 / 教程 / 对外介绍 | `██████████` 96% | README、首版发布清单、最终可发布状态、发布说明、发布计划、演示素材记录、截图清单、微信回归清单、中文发布文章和本地预览/封面 PNG 已补齐，仍需真实微信截图 |
| 测试 / CI / 审计 | `███████░░░` 76% | CI 绿；最近 `#60`、`#61`、`#62` 的 PR build 和合并到 `main` 后的 push build 均已成功。本地 `cargo fmt --all -- --check`、`cargo clippy --all-targets --all-features --tests --benches -- -D warnings`、`cargo nextest run --all-features` 已再次 fresh 通过，当前 241 tests passed；`cargo llvm-cov nextest --all-features --summary-only` 上次测得总行覆盖 59.65%，浏览器自动化覆盖不足 |
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
9. 插件入口：`obsidian-plugin/` 需要被当成正式入口之一继续补说明和边界，而不是只当实验目录存在
10. 执行计划：当前阶段的里程碑、完成标准和推进顺序已收口到 `docs/EXECUTION_PLAN_ZH.md`

## Immediate Next Step

下一步先补真实平台证据：Windows release workflow 已开始 smoke 测试打包后的 zip 资产，主线剩余关键验证点转为真实微信草稿回归，需要用户凭证/IP 白名单/扫码配合完成。

## Completed

### 基础
- `init` / `status` / `check` — 基础脚手架
- `--json` / `--config` 全局 flag
- `status` / `check` / `preview` / `push` / `draft-from-inbox` / `intake feishu ... --draft` 在 `--json` 下返回命令专属结构化对象，便于 Agent / 插件直接读取阶段列表、产物状态、`media_id` 和下一步动作；其中 `draft-from-inbox --push` / `intake feishu ... --draft --push` 还会补充 `pushed`、`media_id`、`stage`、`next_step`；其余命令仍保持兼容的 `{"output":"..."}` 包装
- `intake feishu <file>` / `--minute-token <token>` / `--latest` / `--query <关键词>` — 飞书秒记导出文本、指定 token、最近妙记或关键词搜索结果导入 `Inbox/Feishu/`；官方秒记链路会按 `minute_token` 复用既有 Inbox 文件；加 `--draft` 后继续生成可编辑文章草稿，加 `--preview` 后本地渲染并打开 HTML 预览
- `draft-from-inbox ... --preview --no-open` / `intake feishu ... --draft --preview --no-open` — 自动化友好的预览路径：生成 HTML 和 draft JSON，但不拉起系统浏览器，适合 CI、脚本和后续 Agent 编排
- `draft-from-inbox ... --push` / `intake feishu ... --draft --push` — 生成草稿后直接继续执行 `push --render`；`--push` 与 `--preview` 互斥，且 `intake feishu` 下必须搭配 `--draft`
- 飞书默认保守模式已固化到 help text / AGENTS / 文档：推荐先走 `intake feishu ... --draft --preview`，只有显式 `--push` 才表示继续推进到微信草稿；本地 preview 与微信公众号后台 preview-send 现已明确分层

### 渲染与发布
- `render` — Markdown → WeChat HTML + draft.json（Block 模板系统 + inline CSS）
  - 支持 `--humanize` flag 在渲染时去 AI 味
  - 支持 `--author` / `--thumb` 覆盖
  - 支持 theme 配置（default/warm/dark/geek/paper/magazine/notebook/classic/forest/sunset/ocean/mono/editorial/zen/newsletter/academic/cyber），通过 Theme 系统注入 inline CSS
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
`:::blockname` 语法，14 种 block：
`book-info` / `intro` / `callout` / `steps` / `summary` / `figure` / `checklist` / `cover`
`key-points` / `pull-quote` / `quote-card` / `divider` / `concept-card` / `emotion-card`

### Theme 系统
- `theme = "default"|"warm"|"dark"|"geek"|"paper"|"magazine"|"notebook"|"classic"|"forest"|"sunset"|"ocean"|"mono"|"editorial"|"zen"|"newsletter"|"academic"|"cyber"` in moonpub.toml
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
- [ ] 解决 headless 模式下的登录持久化（cookie 存储/复用）
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
- 2026-07-01: **飞书秒记真实闭环验证** — 使用真实 Obsidian articles 路径运行 `moonpub --articles "<path>" --json intake feishu --latest --draft --preview --no-open`，成功拿到真实 `inbox_path` / `draft_path` / `html_path`；继续运行 `moonpub --articles "<path>" --json intake feishu --latest --draft --push`，成功恢复微信会话、进入编辑器、自动完成原创/赞赏/留言/创作来源，并完成微信公众号后台“预览发送到手机”，最终返回 `pushed: true`、真实 `media_id` 和 `stage: ready`。同时确认当前 CLI 实际入口是 `--articles`，不是 `--vault`，且 `--json` 必须放在子命令前面。
- 2026-07-01: **整体评估与阶段计划** — 新增 `docs/PRODUCT_EVALUATION_ZH.md`，基于当前代码、README、ROADMAP 和 PROGRESS 现状，明确项目当前应定位为“本地发布内核”，飞书秒记应先作为内部正式模块而非立刻拆新项目，并把后续重点收口为“先让用户会用，再继续扩能力”；同步 README_zh / ROADMAP / PROGRESS 入口说明。
- 2026-07-01: **推荐工作流入口收口** — 新增 `docs/RECOMMENDED_WORKFLOWS_ZH.md`，把当前最主推的两条用户路径单独写清楚：`已有 Markdown 文章 → 本地预览 → 微信草稿` 与 `飞书秒记 → 草稿 → 预览 → 微信草稿`；同步 README_zh / USER_GUIDE / WORKFLOW 的入口提示，减少“用户拿到项目却不知道先跑哪条路径”的理解成本。
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
