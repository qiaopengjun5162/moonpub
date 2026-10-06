# Changelog

All notable changes to MoonPub.

## [0.4.5](https://github.com/qiaopengjun5162/moonpub/compare/v0.4.4...v0.4.5) (2026-10-06)


### Features

* add cards + wechat-checklist commands and cover/theme metadata ([dff8525](https://github.com/qiaopengjun5162/moonpub/commit/dff8525ae40829a10e1e31ec7b45bdb93390d10d))
* **cover:** AI 封面生成能力（cover --style ai-art/cartoon/anime） ([55db2e0](https://github.com/qiaopengjun5162/moonpub/commit/55db2e0459eb1a5919b0d37a78bb130739bf1109))
* **cover:** 新增 5 套高级海报封面风格（swiss/aurora/riso/noir/bauhaus） ([317a1ef](https://github.com/qiaopengjun5162/moonpub/commit/317a1ef04fa8c92b096d35e96a5fe3a2611d83d5))
* **cover:** 新增 Editorial 内容驱动少文字封面风格 ([329b9cb](https://github.com/qiaopengjun5162/moonpub/commit/329b9cb0380873a4492e17dae7d6cbbf842d595e))
* **mcp:** add MoonPub MCP server and companion publish skill ([2e7c2b0](https://github.com/qiaopengjun5162/moonpub/commit/2e7c2b0912dfefcbd20402a7cd71f6adb58bb61d))
* **ship:** 在 ship 命令集成 AI 封面（--style ai-art） ([2d9e74f](https://github.com/qiaopengjun5162/moonpub/commit/2d9e74fd6e8d02154f272e4acb4274c6b7bf26f0))
* **ship:** 封面风格校验防静默fallback + 输出带style ([4c9d8dc](https://github.com/qiaopengjun5162/moonpub/commit/4c9d8dc03a42d696c11b2943ecbe1b8c6b198e33))
* **theme:** 新增 5 套编辑器配色主题（monokai/dracula/nord/one-dark/gruvbox） ([e9bf1f8](https://github.com/qiaopengjun5162/moonpub/commit/e9bf1f8907ab8dc67ba3ee6616af7578ce9d44ee))


### Bug Fixes

* **cover:** AI 封面 provider 守卫前置，避免误导性报错 ([9c3790a](https://github.com/qiaopengjun5162/moonpub/commit/9c3790a58d61474995493fea07764212f82be8df))
* **cover:** replace internal build signature in geek-black cover with neutral text ([58be536](https://github.com/qiaopengjun5162/moonpub/commit/58be5361d515835dcd99f3220d673e60220cb502))
* **cover:** replace internal build signature in geek-black cover with neutral text ([cd83b7f](https://github.com/qiaopengjun5162/moonpub/commit/cd83b7fcfb3ff30fba35611bae4e5fed98f46484))
* **footer:** inline local follow_image as data URI like qrcode ([5bfaa1d](https://github.com/qiaopengjun5162/moonpub/commit/5bfaa1d8566a47bbbab2103029fc85dbfeb50db5))
* **render:** blockquote text uses text_color for contrast on tinted backgrounds ([1d2d800](https://github.com/qiaopengjun5162/moonpub/commit/1d2d80026b8565d2930826744be4adced4f0b493))
* **render:** render list markers inline instead of 34px leading column (empty column on WeChat mobile) ([b758b99](https://github.com/qiaopengjun5162/moonpub/commit/b758b99431fd37a5ad67cda00dd719743e9add11))
* **render:** 封面HTML不再嵌入微信正文, 修复构建tag泄漏(WEB3·DEV) D17-D19已发布文章实测 ([3d67d91](https://github.com/qiaopengjun5162/moonpub/commit/3d67d91cbe12d59ee5b6d970d285ef036e5844d0))
* **render:** 微信兼容代码块——弃用 &lt;pre&gt; 改逐行 &lt;p&gt;，Xcode Dark 主题，去语言标签 ([f415809](https://github.com/qiaopengjun5162/moonpub/commit/f4158095145729a5f94c51244b8a4066781d785d))


### Refactoring

* **protocol:** 将单文件 protocol.rs 抽取为 protocol/ 模块 ([c4561ef](https://github.com/qiaopengjun5162/moonpub/commit/c4561ef49220d1249a9d7a6d5d5fb9e21491e769))


### Documentation

* **lessons:** footer follow_image data URI 内联修复记录 ([9dc8d17](https://github.com/qiaopengjun5162/moonpub/commit/9dc8d1758b88d01116ab5e78688216a9c2a6d3df))


### Maintenance

* 忽略生成物目录与覆盖率产物 ([c42f595](https://github.com/qiaopengjun5162/moonpub/commit/c42f59521f46cf27f3a2b86e982e15df569a1931))
* 补充仓库基础工程配置与文档 ([5dc9417](https://github.com/qiaopengjun5162/moonpub/commit/5dc94178bcad61352f58d34003963b4395d5182e))

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
