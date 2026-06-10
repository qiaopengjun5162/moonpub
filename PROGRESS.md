# MoonPub CLI Progress

## Status

Active development. Core pipeline complete, block template system, humanize, and cover generation done.

## Completed

### 基础
- `init` / `status` / `check` — 基础脚手架
- `--json` / `--config` 全局 flag

### 渲染与发布
- `render` — Markdown → WeChat HTML + draft.json（Block 模板系统 + inline CSS）
  - 支持 `--humanize` flag 在渲染时去 AI 味
  - 支持 `--author` / `--thumb` 覆盖
  - 已去掉硬编码 footer（使用寻月阁标准结尾模板）
- `preview` — 系统浏览器打开 HTML
- `push` — 原生 WeChat API 推送（无需 md2wechat）
- `update-draft` — 更新已有微信草稿
- `export` — Zola 博客导出

### Block 模板系统
`:::blockname` 语法，12 种 block（新增 quote-card/divider/concept-card/emotion-card）：
`book-info` / `intro` / `callout` / `steps` / `summary` / `figure` / `checklist` / `cover`
- 所有样式 inline CSS，微信兼容
- 使用 `<table>` 布局处理复杂 block
- 普通 Markdown（h2/h3/p/blockquote/hr/img）完全兼容

### 去 AI 味（Humanize）
- `moonpub humanize <article.md>` 独立命令
- 6 阶段规则处理：填充短语 → AI词汇 → 排比 → 修饰 → 结论 → 破折号
- 参考：Humanizer-zh (op7418) + stop-slop (hardikpandya)
- 实现：`src/humanize.rs`

### 封面生成（Cover）
- `moonpub cover <article.md> [--style dark|clean|minimal]`
- 3 套 HTML 模板：dark（默认，蓝调）/ clean（浅色，橙调）/ minimal（居中，衬线）
- 生成 900×500px 独立 HTML 文件
- 参考：guizang-ppt-skill + article-tools
- 实现：`src/cover.rs`

### Radar
- `radar add/list/import/analyze/scrape` — 热点样本管理与标题建议

### 状态追踪
- `.moonpub/status.jsonl` — render/push/ready/published 状态自动记录
- `mark-ready` / `mark-published` 命令

### WeChat API 客户端
- `src/wechat.rs` — 直接调用微信 API（access_token/draft_add/draft_update/upload_image）
- 完全替换 md2wechat，零外部 CLI 依赖
- 仅依赖 `ureq`（HTTP + TLS）

### 项目规范
- 55 个单元测试，0 clippy warnings
- PR-first 工作流（`codex/<topic>` 分支 → `gh pr create` → merge）
- README.md + CONTRIBUTING.md + docs/REFERENCES.md + docs/BROWSER_AUTOMATION.md
- 参考项目：qunmind（PR 规范）

## 项目结构

```
src/
  main.rs       # 入口
  lib.rs        # CLI 核心 / Block 模板 / 渲染引擎
  wechat.rs     # WeChat API client
  humanize.rs   # 去 AI 味
  cover.rs      # 封面 HTML 模板
docs/
  REFERENCES.md           # 30+ 参考项目文档
  BROWSER_AUTOMATION.md   # playwright-cli 浏览器自动化参考
scripts/
  moonpub-backend.sh      # 微信后台自动化脚本（参考用）
```

## 依赖

仅 `ureq`（HTTP + TLS）。其他全部纯 Rust std。

## 已知问题

| 问题 | 状态 | 备注 |
|------|------|------|
| 浏览器自动化不稳定（光标/弹窗） | 待优化 | 考虑用 Obscura 替代 playwright-cli |
| 合集 API 不可用 | 微信限制 | 需手动选择 |
| update-draft 后部分设置重置 | 微信 API 行为 | 浏览器重新设置 |
| 文章配图 | 待实现 | `:::figure` block 已有，缺自动生成 |
| 封面图自动截图（HTML→PNG） | 待实现 | 可接 playwright-cli screenshot |
| IP 经常变 | 网络限制 | 每次 push 前需确认白名单 |

## 版本

- 2026-06-10: Block 模板 + Humanize + Cover + PR workflow
