# MoonPub CLI Progress

## Status

Beta / early adopter ready. Core pipeline complete, v0.4.1 release assets exist, and the macOS ARM64 release binary has passed the no-credential first-run smoke test from a clean directory. It is usable by technical users who can configure WeChat credentials, but still needs live WeChat regression checks, broader platform smoke tests, screenshots/recordings, and module cleanup before calling it broadly stable.

## Final Goal

MoonPub 的最终目标：让作者从 Obsidian / Markdown 出发，用一个可审计、可复现、可本地运行的 Rust CLI，把文章稳定发布到微信公众号草稿，并同步导出到个人博客；对外使用时，用户应能按 README 完成安装、配置、预览、推送和故障排查。

长期路线见 [ROADMAP.md](ROADMAP.md)：先完成真实微信回归，再做插件化核心、Obsidian 插件正式化、WordPress / Ghost 等低风险多平台发布，最后探索本地 App 和 Pro 版。v0.5 插件化设计见 [docs/PLUGIN_ARCHITECTURE_ZH.md](docs/PLUGIN_ARCHITECTURE_ZH.md)。

## Progress Bar

整体进度：`█████████░` 87%

| 领域 | 进度 | 当前判断 |
|------|------|----------|
| 核心 CLI / 配置 / 状态 | `█████████░` 90% | 常用命令完整，仍可继续改善错误提示和 dry-run |
| Markdown 渲染 / Block / Theme | `████████░░` 85% | 已能产出微信 HTML，解析与渲染开始拆分，后续重点是排版细节和更多真实文章样本 |
| WeChat API 推送 | `████████░░` 85% | draft add/update/image upload 可用，仍需更多错误场景文档 |
| CDP 浏览器自动化 | `███████░░░` 70% | 核心步骤本地验证过，但微信 UI 会变，合集/发表仍未启用 |
| 对外安装 / Release | `█████████░` 88% | v0.4.1 release 已成功产出五个平台资产，macOS ARM64 已完成 release smoke test |
| 文档 / 教程 / 对外介绍 | `██████████` 96% | README、首版发布清单、最终可发布状态、发布说明、发布计划、演示素材记录、截图清单、微信回归清单、中文发布文章和本地预览/封面 PNG 已补齐，仍需真实微信截图 |
| 测试 / CI / 审计 | `███████░░░` 72% | CI 绿、本地 `cargo nextest run --all-features` 165 tests passed，本地无凭证闭环已验证，浏览器自动化覆盖不足 |
| 代码结构 / 可维护性 | `█████████░` 90% | Radar 已完成首轮拆分，Markdown parser、AI workflow、init、draft、bundle、plugin、cover 辅助与 ship 编排模块已拆出，capabilities 开始提供插件/App 可直接调用的 target 命令模板 |

## Current Milestone

目标：把项目从“作者本人可用”推进到“技术用户可照文档试用”。

完成标准：
- [x] v0.4.0 release 有 Linux / macOS / Windows 资产，且已验证 macOS amd64 下载与 sha256
- [x] PR CI 通过：fmt / clippy / cargo audit / nextest
- [x] README 不再指向过期 release 或不存在的 Homebrew tap
- [x] README / README_zh 第一屏明确 Beta 状态、适用人群和限制
- [x] 新手路径有一条已实测的 dry-run / preview-only 流程
- [x] `PROGRESS.md` 持续记录真实验证、覆盖率和未完成项

## Next Small Goals

1. 对外定位：更新 README / README_zh，明确当前是 Beta，适合技术用户试用；说明哪些步骤会触达微信 API，哪些只是本地渲染。
2. 新手闭环：补一条不需要真实微信凭证的本地体验路径：`init` → `new` → `render` → `preview` → `cover`。（源码构建二进制已实测 `init` → `new` → `render` → `cover` → `check`）
3. 文档一致性：把 `PROGRESS.md`、`docs/GETTING_STARTED.md`、`docs/USER_GUIDE.md` 的安装、状态和风险描述统一。（已完成首轮，后续随功能变化继续维护）
4. 结构清理：`src/radar.rs` 已完成首轮拆分，分出 `radar/cli.rs`、`radar/store.rs`、`radar/analyze.rs`、`radar/scrape.rs`；Markdown 已拆出 `markdown/parser.rs`；AI 命令编排已拆到 `src/ai_workflow.rs`；初始化向导已拆到 `src/init.rs`；本地草稿创建/写入已拆出 `src/draft.rs`；文章包状态和移动已拆到 `src/bundle.rs`；内部 target trait 已拆到 `src/plugin.rs`，微信草稿发布已成为第一个 `PublishTarget`，Zola 导出已成为第一个 `ExportTarget`；封面 style/HTML/PNG 辅助已回收到 `src/cover.rs`；ship 一键发布编排已拆到 `src/ship.rs`；通用 `publish --target` / `export --target` 命令开始承接插件化核心。
5. 自动化风险：浏览器自动化已明确为本地辅助驾驶，不绕过扫码/验证码/审核/最终人工确认；后续继续补真实微信回归清单。
6. 长期产品化：路线已确定为 CLI 稳定核心 → 插件化扩展点 → Obsidian 插件正式化 → WordPress / Ghost 等平台 → 本地 App / Pro 版。

## Immediate Next Step

下一步把 v0.5 插件化入口继续收口：已补 `moonpub publish <article.md> --target wechat-draft`、`moonpub export <article.md> --target zola`，并让 `capabilities --json` 暴露 argv 风格命令模板。真实微信草稿回归仍需要用户凭证/IP 白名单/扫码配合完成。

## Completed

### 基础
- `init` / `status` / `check` — 基础脚手架
- `--json` / `--config` 全局 flag

### 渲染与发布
- `render` — Markdown → WeChat HTML + draft.json（Block 模板系统 + inline CSS）
  - 支持 `--humanize` flag 在渲染时去 AI 味
  - 支持 `--author` / `--thumb` 覆盖
  - 支持 theme 配置（default/warm/dark/geek），通过 Theme 系统注入 inline CSS
- `preview` — 系统浏览器打开 HTML
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
`:::blockname` 语法，12 种 block：
`book-info` / `intro` / `callout` / `steps` / `summary` / `figure` / `checklist` / `cover`
`quote-card` / `divider` / `concept-card` / `emotion-card`

### Theme 系统
- `theme = "default"|"warm"|"dark"|"geek"` in moonpub.toml
- `Theme::section_style()` 生成 section 级 inline CSS

### 去 AI 味（Humanize）
- 6 阶段规则处理：填充短语 → AI词汇 → 排比 → 修饰 → 结论 → 破折号

### 封面生成（Cover）
- `moonpub cover [--style ...] [--screenshot]`
- 10 套 HTML 模板：dark / clean / minimal / warm / serif / gradient / literary / ink / sunset / forest
- 封面模板会转义 frontmatter 中的标题、副标题和作者文本，避免特殊字符破坏 HTML

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
  markdown.rs      # Markdown → WeChat HTML 转换
  markdown/parser.rs # ::: block 与属性解析
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
- 2026-06-23: **封面文本转义** — title/digest/author 统一 HTML 转义，`cargo nextest run --all-features` 130 tests passed
- 2026-06-16: **创作来源 radio value 修复** — headed + headless 均稳定，ship 端到端验证通过
- 2026-06-16: **模块拆分收尾** — cdp.rs / publish_steps.rs / markdown.rs 从 publish.rs 和 render.rs 拆分
- 2026-06-15: **lib.rs 模块化** — 拆分为 cli / config / error / article / render / export / status / preview / system / push
- 2026-06-12: **auto_configure 完善** — 原创/赞赏/留言/创作来源/预览 自动化，合集暂跳过
- 2026-06-11: **浏览器自动化** — Rust chromiumoxide 实现 CDP 浏览器控制
- 2026-06-11: **结尾模板** — footer.rs 群二维码 + banner + CTA
- 2026-06-11: **封面集成** — render_article 支持 cover_html，ship 自动注入封面
- 2026-06-10: Block 模板 + Humanize + Cover + PR workflow + radar suggest
