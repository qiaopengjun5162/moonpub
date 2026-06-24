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
- `src/draft.rs` 负责本地草稿文件创建、AI 生成文章写入和草稿路径/重复文件校验；不要把这些文件细节塞回 `src/app.rs`。
- `src/render.rs` / `src/markdown.rs` 负责 Markdown 到微信 HTML 和 draft JSON；`src/markdown/parser.rs` 只放 `:::` block 与属性解析，不放微信样式渲染。
- `src/push.rs` / `src/wechat.rs` 负责微信 API。
- `src/publish.rs` / `src/cdp.rs` / `src/publish_steps.rs` 负责浏览器自动化。
- `push` / `ship` 成功创建微信草稿后，本地文章包进入 `Articles/ready/`；只有真实自动发布成功或用户手动 `mark-published` 后才进入 `Articles/published/`。

## 微信发布约束

- 微信编辑器是 live web app，DOM 和文案会变；自动化步骤应优先软失败，不能影响 API 草稿推送主流程。
- 定位微信编辑器元素时优先用 DOM 结构、class、input value；不要依赖不稳定的 `textContent`。
- 创作来源当前稳定路径是 `.js_claim_source_desc` 打开 picker，`input[type="radio"][value="4"]` 选择，`.js_claim_source_selected` 验证。
- API push 的 HTML 优先使用微信更稳定的 `<section>` / `<p>` / `<table>` 和 inline CSS；避免依赖会被编辑器剥离的标签样式。
- 配置里的资产路径（如 qrcode、cover）按 articles root 解析；文章内相对封面路径按文章所在目录解析。

## 配置与环境

- 配置优先级：环境变量 > `.env` / `~/.moonpub.env` > `moonpub.toml`。
- `WECHAT_SECRET` 不进配置文件，必须来自环境变量或本地 env 文件。
- `DEEPSEEK_API_KEY` 只用于 `write` / `expand` / `polish` / `ship --ai`。
- 非交互 `moonpub init` 必须生成可直接使用的当前目录配置；不要把示例占位路径写入真实初始化文件。
- `Config::from_toml` 是手写解析器，只支持本项目已知 key；扩展配置时补测试。

## 文档同步点

- 封面风格当前为 10 种：`dark` / `clean` / `minimal` / `warm` / `serif` / `gradient` / `literary` / `ink` / `sunset` / `forest`。
- 渲染主题当前为 4 种：`default` / `warm` / `dark` / `geek`。
- `PROGRESS.md` 记录真实完成度；不要把本地单元测试通过写成真实微信端验证通过。
- 对外主推 release 前必须下载 release 资产跑 smoke test；源码构建二进制通过不能替代 release 二进制验证。
- `.cargo/config.toml` 的 `target-cpu=native` 只适合本地开发；CI/release 构建必须覆盖为可移植 flags，避免 macOS ARM64 上 `ring` 编译失败。
