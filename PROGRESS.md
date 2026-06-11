# MoonPub CLI Progress

## Status

Active development. Core pipeline complete. 74 tests, 0 clippy warnings, cargo fmt clean.

## Completed

### 基础
- `init` / `status` / `check` — 基础脚手架
- `--json` / `--config` 全局 flag

### 渲染与发布
- `render` — Markdown → WeChat HTML + draft.json（Block 模板系统 + inline CSS）
  - 支持 `--humanize` flag 在渲染时去 AI 味
  - 支持 `--author` / `--thumb` 覆盖
  - 支持 `wechat_theme` 配置（default/warm/dark），通过 Theme 系统注入 inline CSS
  - 已去掉硬编码 footer（使用寻月阁标准结尾模板）
- `preview` — 系统浏览器打开 HTML
- `push` — 原生 WeChat API 推送（无需 md2wechat）
  - **自动上传本地图片**：push 时扫描 HTML 里的本地 src，逐个上传微信素材库，替换为 CDN URL，再重建 draft.json
- `update-draft` — 更新已有微信草稿
- `export` — Zola 博客导出
- `humanize` — 独立去 AI 味命令，in-place 修改文章

### Block 模板系统
`:::blockname` 语法，12 种 block：
`book-info` / `intro` / `callout` / `steps` / `summary` / `figure` / `checklist` / `cover`
`quote-card` / `divider` / `concept-card` / `emotion-card`
- 所有样式 inline CSS，微信兼容
- 使用 `<table>` 布局处理复杂 block
- 普通 Markdown（h2/h3/p/blockquote/hr/img）完全兼容

### Theme 系统
- `wechat_theme = "default"|"warm"|"dark"` in moonpub.toml
- `Theme::section_style()` 生成 section 级 inline CSS
- render 时从 config 读取并注入，push 时同步使用

### 去 AI 味（Humanize）
- `moonpub humanize <article.md>` 独立命令
- 6 阶段规则处理：填充短语 → AI词汇 → 排比 → 修饰 → 结论 → 破折号
- 实现：`src/humanize.rs`

### 封面生成（Cover）
- `moonpub cover <article.md> [--style dark|clean|minimal|warm|serif|gradient] [--screenshot]`
- 6 套 HTML 模板：dark / clean / minimal / warm / serif / gradient
- `--screenshot`：通过 Chrome headless 自动截图 HTML → PNG（900×500px）
- 实现：`src/cover.rs` + lib.rs `find_chrome()`

### Radar
- `radar add/list/import/analyze/suggest/scrape` — 热点样本管理与标题建议
- `suggest`：4 种标题公式（痛点+方案 / 数字+结果 / 悬念冲突 / 用户标签）

### 状态追踪
- `.moonpub/status.jsonl` — render/push/ready/published 状态自动记录
- `mark-ready` / `mark-published` 命令

### WeChat API 客户端
- `src/wechat.rs` — access_token / draft_add / draft_update / upload_image / upload_image_url
- 完全替换 md2wechat，零外部 CLI 依赖，仅依赖 `ureq`

### 项目规范
- 74 个单元测试，0 clippy warnings
- PR-first 工作流（`feat/<topic>` 分支 → CI 验证 → merge）
- CI：`cargo test` + `cargo clippy -D warnings` + `cargo fmt --check`

## 项目结构

```
src/
  main.rs       # 入口
  lib.rs        # CLI 核心 / Block 模板 / 渲染引擎 / push 逻辑（~3330 行）
  radar.rs      # Radar 命令：热点管理、标题分析、抓取（~1263 行，从 lib.rs 拆分）
  wechat.rs     # WeChat API client
  humanize.rs   # 去 AI 味
  cover.rs      # 封面 HTML 模板
  theme.rs      # 渲染主题（default/warm/dark）
  illustrate.rs # 插图 block 渲染
docs/
  REFERENCES.md    # 参考项目
  WORKFLOW.md      # 完整发布工作流
```

## 依赖

仅 `ureq`（HTTP + TLS）。其他全部纯 Rust std。

## 已知问题与解法

| 问题 | 状态 | 解法/备注 |
|------|------|-----------|
| CI `cargo fmt` 检查失败 | 已解决 | 每次 commit 前先跑 `cargo fmt`，CI 同时跑 fmt check |
| `clippy::collapsible_if` | 已解决 | 嵌套 if 合并为 `&&` 条件 |
| `theme.rs` 含反斜杠换行字符 | 已解决 | 用字面量字符串重写，避免隐式转义 |
| WeChat IP 白名单限制 | 持续 | 每次 push 前确认本机 IP 在后台白名单内 |
| 合集 API 不可用 | 微信限制 | 需手动在后台选择合集 |
| update-draft 后部分设置重置 | 微信 API 行为 | update 后需在后台重新设置封面/摘要 |
| 封面图 HTML→PNG 截图 | 已实现 | `cover --screenshot`：Chrome headless 截图，`find_chrome()` 自动查找 Chrome 路径 |
| `ship` 命令 style 硬编码为 Clean | 已解决 | 支持 `--style dark|clean|minimal|warm|serif|gradient`，与 cover 一致 |
| 子命令 `--help` 不工作 | 已解决 | 在命令匹配前插入 `--help`/`-h` 检测，`cover --help` 等全部支持 |

## 版本日志

- 2026-06-11: 拆分 radar.rs（~1263 行），lib.rs 从 ~4600 → ~3330 行
- 2026-06-11: cover `--screenshot` 通过 Chrome headless 截图 HTML → PNG
- 2026-06-11: 修复子命令 `--help`（cover --help 等），ship 支持全 6 种风格
- 2026-06-11: 图片自动上传（push 时扫描本地 src → 上传微信CDN → 重建 draft.json）
- 2026-06-11: Theme 系统接入 render（wechat_theme config → inline CSS）
- 2026-06-11: help 文本补全（cover/humanize/radar suggest/scrape）
- 2026-06-10: Block 模板 + Humanize + Cover + PR workflow + radar suggest
