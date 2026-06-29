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

## Immediate Next Step

下一步先补真实平台证据：Windows release workflow 已开始 smoke 测试打包后的 zip 资产，主线剩余关键验证点转为真实微信草稿回归，需要用户凭证/IP 白名单/扫码配合完成。

## Completed

### 基础
- `init` / `status` / `check` — 基础脚手架
- `--json` / `--config` 全局 flag
- `preview` / `push` / `draft-from-inbox` / `intake feishu ... --draft` 在 `--json` 下返回命令专属结构化对象，便于 Agent / 插件直接读取路径、`media_id` 和下一步动作；其中 `draft-from-inbox --push` / `intake feishu ... --draft --push` 还会补充 `pushed`、`media_id`、`stage`、`next_step`；其余命令仍保持兼容的 `{"output":"..."}` 包装
- `intake feishu <file>` / `--minute-token <token>` / `--latest` / `--query <关键词>` — 飞书秒记导出文本、指定 token、最近妙记或关键词搜索结果导入 `Inbox/Feishu/`；官方秒记链路会按 `minute_token` 复用既有 Inbox 文件；加 `--draft` 后继续生成可编辑文章草稿，加 `--preview` 后本地渲染并打开 HTML 预览
- `draft-from-inbox ... --preview --no-open` / `intake feishu ... --draft --preview --no-open` — 自动化友好的预览路径：生成 HTML 和 draft JSON，但不拉起系统浏览器，适合 CI、脚本和后续 Agent 编排
- `draft-from-inbox ... --push` / `intake feishu ... --draft --push` — 生成草稿后直接继续执行 `push --render`；`--push` 与 `--preview` 互斥，且 `intake feishu` 下必须搭配 `--draft`

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
