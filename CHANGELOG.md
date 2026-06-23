# Changelog

All notable changes to MoonPub.

## [0.4.1] — 2026-06-23

### Added
- `moonpub --version` / `moonpub -V` prints the current CLI version for install checks and support requests.
- Release workflow now builds a native macOS ARM64 asset (`moonpub-macos-arm64.tar.gz`) in addition to macOS x86_64, Linux, and Windows assets.

### Fixed
- Non-interactive `moonpub init` now writes the current directory as `[articles].root`, so the first-run local flow works from a clean directory instead of writing `/path/to/ObsidianMain`.
- The hand-written TOML parser now unescapes basic string sequences used by generated config paths.

### Verified
- v0.4.0 macOS amd64 asset download and sha256 verification passed, but first-run smoke test failed because `moonpub init` wrote the placeholder articles root. v0.4.1 is the intended first broadly shareable release candidate.

## [0.4.0] — 2026-06-17

### Added
- **AI 写作**: `write`, `expand`, `polish` 三个命令，通过 DeepSeek API 实现
- **Obsidian 插件**: Cmd+P 输入"发布"即可推送到微信
- **去 AI 味**: `humanize` 命令，6 阶段规则处理
- **自动加载 .env**: 启动时自动加载 `.env` 和 `~/.moonpub.env`
- **封面风格**: 新增 `ink`(水墨)、`sunset`(日落)、`forest`(森林) 三种风格，共 10 种
- **封面自动下载**: frontmatter 中 `cover: https://...` URL 自动下载上传为微信封面图
- **用户使用说明书**: `docs/USER_GUIDE.md`
- **新手上手指南**: `docs/GETTING_STARTED.md`
- **项目首页**: `docs/index.html`

### Changed
- **Footer 模块化**: `[footer]` TOML section 可配置结尾模板，未配置则不渲染
- **ship --ai**: 润色后发布，一步到位
- **首段摘要**: 不再自动填充 digest，由微信自行抓取

### Fixed
- 微读导入笔记的 Obsidian callout `[!abstract]` 不再渲染为超长 blockquote
- TOML 解析器按 section 区分同名 key，root 不再冲突
- 配置自动发现：不传 `--config` 时自动检测 `moonpub.toml`

### Infrastructure
- GitHub Actions release 构建 (macOS)
- CODE_OF_CONDUCT.md, CONTRIBUTING.md
- 129 个测试全过

## [0.3.2] — 2026-06-16

### Added
- **浏览器自动化全通**: 原创声明、赞赏、留言、创作来源、预览全部稳定
- **创作来源 radio value 选择器**: 精确到 `input[value="4"]`
- **预览发送**: headless 下成功发送预览到手机
- **文章状态追踪**: `status` / `check` 命令
- **阅读量数据采集**: `radar` 命令组

### Fixed
- headless 下 target="_blank" 新 tab 检测不可靠，改用 `page.goto(url)` 直接导航
- 创作来源弹窗文本匹配不稳定，改用 DOM 结构标记
- 赞赏 toggle offsetParent 不可见，改用 JS `.click()` 绕过
- `<blockquote>` 样式被微信剥离，改用 `<section>` 标签

## [0.3.0] — 2026-06-15

### Changed
- monolithic `lib.rs` 按职责拆分为 13 个模块
- `wechat.rs` 与 `app.rs` 循环依赖修复
- geek 主题从纯黑改为 GitHub light 配色
- `ship` 每次截图封面 PNG → 上传微信 → 新 media_id

## [0.2.0] — 2026-06-14

### Added
- `ship` 一键发布命令
- 封面生成 (`cover`) — 6 种风格
- 浏览器自动化 (CDP) — 原创声明/赞赏/留言/预览
- WeChat API 客户端 — ureq, 零 SDK
- Zola 博客导出
- 4 种主题: default, warm, dark, geek

## [0.1.0] — 2026-06-13

### Added
- Markdown → WeChat HTML 渲染
- Block 模板系统 (intro, callout, steps, summary 等)
- 命令行解析和配置管理
- 手写 TOML 解析器
