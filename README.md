# MoonPub

![Rust Version](https://img.shields.io/badge/rust-%3E%3D1.85-blue)
![License](https://img.shields.io/badge/license-MIT-green)

纯 Rust CLI：Markdown → 微信公众号全流程自动化。

`MoonPub` 围绕几个稳定的边界设计：

- `render`: Markdown → WeChat HTML + draft.json（Block 模板系统 + 去 AI 味）
- `push`: 原生 WeChat API 客户端（零 md2wechat 依赖，直连 draft/add）
- `export`: Zola 博客导出（YAML → TOML frontmatter）
- `radar`: 平台热点样本管理 + 标题建议

## 为什么不用 md2wechat

大多数微信发布工具依赖付费 API 或第三方 CLI。MoonPub 直接从零实现：

- WeChat API client（Rust + ureq），不依赖任何外部工具
- 40+ 种排版模板 → 内置 Block 模板系统，免费可定制
- 所有转换完全离线，零网络依赖（除 push 时调微信 API）

## 快速开始

```bash
cargo install --path .
moonpub init                    # 创建 moonpub.toml
moonpub status                  # 查看文章流水线
moonpub render article.md       # 生成 HTML + draft.json
moonpub push article.md         # 推送微信草稿
moonpub export article.md       # 导出 Zola 博客
```

推送需要微信凭证：

```bash
export WECHAT_APPID=wx***
export WECHAT_SECRET=your_secret
```

## Block 模板系统

在 Markdown 中使用 `:::blockname` 语法：

```markdown
:::book-info
title: 书名
author: 作者
cover: https://...
publisher: 出版社
rating: 8.1
:::

:::intro
1-3 句话导语，迅速抓住读者
:::

:::callout
label: 核心结论
这里写你最想让读者带走的一句话
:::

:::steps
1. 第一步说明
2. 第二步说明
3. 第三步说明
:::

:::summary
结尾总结
:::
```

支持的 12 种 Block：`book-info` / `intro` / `callout` / `steps` / `summary` / `figure` / `checklist` / `cover` / `quote-card` / `divider` / `concept-card` / `emotion-card`

## 去 AI 味

```bash
moonpub humanize article.md              # 单独处理
moonpub render --humanize article.md     # render 时一并处理
```

6 阶段规则处理：填充短语 → AI 词汇替换 → 排比打破 → 修饰简化 → 通用结论 → 破折号清理

## 全部命令

```bash
moonpub init [path]            # 创建配置
moonpub status                 # 查看文章流水线 + 状态追踪
moonpub check <article.md>     # 检查文章三件套
moonpub render <article.md>    # Markdown → HTML + draft.json
moonpub preview <article.md>   # 浏览器预览
moonpub push <article.md>      # 推送到微信草稿
moonpub update-draft <article.md>  # 更新已有草稿
moonpub export <article.md>    # 导出 Zola 博客
moonpub humanize <article.md>  # 去 AI 味
moonpub mark-ready <article.md>    # 标记预览已确认
moonpub mark-published <article.md>  # 标记已发表

moonpub radar add --platform <name> --keyword <kw> --title <title>
moonpub radar list [--platform <name>]
moonpub radar import <file.csv>
moonpub radar analyze <article.md> --platform <name>
moonpub radar scrape --platform <name> --keyword <kw>
```

全局 flag：`--vault <path>` / `--config <moonpub.toml>` / `--json`

## 开发

```bash
cargo fmt
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run
```

使用 `cargo nextest`，不是 `cargo test`。

## 架构规则

- 业务逻辑纯 Rust，零外部依赖（仅 `ureq` 用于 HTTP）
- Block 模板系统：`// ── Block renderers` 段在 `src/lib.rs` 中
- WeChat API client：`src/wechat.rs`
- 去 AI 味：`src/humanize.rs`
- 所有样式 inline CSS，微信兼容

## 贡献

使用 PR-first 工作流。创建 `codex/<short-topic>` 分支，保持改动聚焦，运行 `cargo clippy` 和 `cargo nextest`，推送分支，向 `main` 发起 PR。详见 [CONTRIBUTING.md](CONTRIBUTING.md)。

## License

MIT
