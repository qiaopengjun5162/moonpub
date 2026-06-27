# AGENTS.md

MoonPub 是纯 Rust CLI，用于把 Obsidian/Markdown 文章发布到微信公众号草稿，并自动完成封面、渲染、微信 API 推送、CDP 浏览器配置和 Zola 博客导出。

## 沟通与协作

- 默认用中文回复。
- 先验证真实状态，再更新进度或下结论；不要只依赖文档自述。
- 改动后同步更新与本次变更直接相关的 README / README_zh / PROGRESS / CLAUDE / docs。
- 不要读取、打印或提交真实凭据；`.env`、`moonpub.toml` 可能包含本地账号配置。

## 常用验证

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run --all-features
```

项目约定使用 `cargo nextest`，不是 `cargo test`。

## 架构边界

- `src/main.rs` 只负责加载环境变量、解析参数和输出结果。
- `src/cli.rs` 负责 CLI 解析；新增命令时同步更新 `src/error.rs` 的 help text 和 README 命令列表。
- `src/app.rs` 负责命令路由和用例编排，具体平台/API 细节放回对应模块。
- `src/ai_workflow.rs` 负责 `write` / `polish` / `expand` / `ship --ai` 的文件读写与 AI 调用编排；不要把 API key、文章写回、frontmatter 重组逻辑放回 `src/app.rs`。
- `src/init.rs` 负责 `moonpub init` 的交互/非交互配置生成和本地 `.env` 更新；不要把初始化向导细节塞回 `src/app.rs`。
- `src/draft.rs` 负责本地草稿文件创建、AI 生成文章写入和草稿路径/重复文件校验；不要把这些文件细节塞回 `src/app.rs`。
- `src/bundle.rs` 负责 `ArticleBundle`、文章阶段识别和 `drafts` / `ready` / `published` 之间的文章包移动；不要把状态移动逻辑放回 `src/push.rs` 或 `src/status.rs`。
- `src/plugin.rs` 负责内部 target trait、能力元数据、publish/export context/outcome 和调度 helper；新增平台时先实现 target，不要复制 CLI 编排。
- `src/render.rs` / `src/markdown.rs` 负责 Markdown 到微信 HTML 和 draft JSON；`src/markdown.rs` 只做顶层 block 分发，不放具体样式渲染。
- `src/markdown/parser.rs` 只放 `:::` block 与属性解析；`src/markdown/inline.rs` 负责行内 Markdown；`src/markdown/plain.rs` 负责普通段落/表格/列表/引用/代码块；`src/markdown/blocks.rs` 负责 `:::` fence block 渲染。
- `src/cover.rs` 负责封面样式解析、封面 HTML 生成/写入和 Chrome 截图辅助；`src/app.rs` 不直接拼封面路径或 Chrome headless 参数。
- `src/push.rs` / `src/wechat.rs` 负责微信 API；`push_article` 保持兼容 wrapper，底层走 `WechatDraftTarget`。
- `src/export.rs` 负责 Zola 导出；`export_article` 保持兼容 wrapper，底层走 `ZolaExportTarget`。
- `src/ship.rs` 负责一键发布编排：cover → thumb upload → render → push → optional export；`src/app.rs` 只传入解析后的命令参数。
- `src/publish.rs` / `src/cdp.rs` / `src/publish_steps.rs` 负责浏览器自动化。
- `push` / `ship` 成功创建微信草稿后，本地文章包进入 `Articles/ready/`；只有真实自动发布成功或用户手动 `mark-published` 后才进入 `Articles/published/`。

## 微信发布约束

- 微信编辑器是 live web app，DOM 和文案会变；自动化步骤应优先软失败，不能影响 API 草稿推送主流程。
- 定位微信编辑器元素时优先用 DOM 结构、class、input value；不要依赖不稳定的 `textContent`。
- 创作来源当前稳定路径是 `.js_claim_source_desc` 打开 picker，`input[type="radio"][value="4"]` 选择，`.js_claim_source_selected` 验证。
- API push 的 HTML 优先使用微信更稳定的 `<section>` / `<p>` / `<table>` 和 inline CSS；避免依赖会被编辑器剥离的标签样式。
- 配置里的资产路径（如 qrcode、cover）按 articles root 解析；文章内相对封面路径按文章所在目录解析。
- `[footer].qrcode` 为空时，页脚不渲染社群标题、介绍、规则和入群提示；只保留 `follow_image` / `follow_text` 这类关注 CTA。

## 配置与环境

- 配置优先级：环境变量 > `.env` / `~/.moonpub.env` > `moonpub.toml`。
- `WECHAT_SECRET` 不进配置文件，必须来自环境变量或本地 env 文件。
- AI provider 当前支持 `deepseek` / `openai`；默认仍是 `deepseek`。
- `DEEPSEEK_API_KEY` / `OPENAI_API_KEY` 只用于 `write` / `expand` / `polish` / `ship --ai`；`AI_API_KEY` 仅作本地实验 fallback。
- `[template].name` 用于 `configure` / `ship` 的 `moban` 模板插入步骤；未配置时该步骤必须软跳过。
- 非交互 `moonpub init` 必须生成可直接使用的当前目录配置；不要把示例占位路径写入真实初始化文件。
- `Config::from_toml` 是手写解析器，只支持本项目已知 key；扩展配置时补测试。

## 文档同步点

- 封面风格当前为 10 种：`dark` / `clean` / `minimal` / `warm` / `serif` / `gradient` / `literary` / `ink` / `sunset` / `forest`。
- 渲染主题当前为 17 种：`default` / `warm` / `dark` / `geek` / `paper` / `magazine` / `notebook` / `classic` / `forest` / `sunset` / `ocean` / `mono` / `editorial` / `zen` / `newsletter` / `academic` / `cyber`。
- Block 模板当前为 14 种：`book-info` / `intro` / `callout` / `steps` / `summary` / `figure` / `checklist` / `key-points` / `pull-quote` / `cover` / `quote-card` / `divider` / `concept-card` / `emotion-card`。
- `PROGRESS.md` 记录真实完成度；不要把本地单元测试通过写成真实微信端验证通过。
- 对外主推 release 前必须下载 release 资产跑 smoke test；源码构建二进制通过不能替代 release 二进制验证。
- `.cargo/config.toml` 的 `target-cpu=native` 只适合本地开发；CI/release 构建必须覆盖为可移植 flags，避免 macOS ARM64 上 `ring` 编译失败。
- 当前入口文档（`README*`、`docs/GETTING_STARTED.md`、`docs/USER_GUIDE.md`、`docs/index.html`、`docs/slides.html`、`PROGRESS.md`）要跟最新源码能力和当前主推安装版本同步。
- 发布归档/快照文档（如 `docs/RELEASE_NOTES_v0.4.1.md`、`docs/LAUNCH_*`、`docs/WECHAT_REGRESSION_CHECKLIST_ZH.md`）保留当时的版本事实；不要为了“统一口径”随手改成新版本。
- `docs/moonpub.rb` 是带真实 `sha256` 的 Homebrew 维护模板；没有对应 release 资产的真实校验和时，不要只改版本号和下载链接。
