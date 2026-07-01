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
- `src/ai_workflow.rs` 负责 `write` / `draft-from-inbox` / `polish` / `expand` / `ship --ai` 的文件读写与 AI 调用编排；不要把 API key、文章写回、frontmatter 重组逻辑放回 `src/app.rs`。
- `src/init.rs` 负责 `moonpub init` 的交互/非交互配置生成和本地 `.env` 更新；不要把初始化向导细节塞回 `src/app.rs`。
- `src/draft.rs` 负责本地草稿文件创建、AI 生成文章写入和草稿路径/重复文件校验；不要把这些文件细节塞回 `src/app.rs`。
- `src/intake.rs` 负责上游素材导入到 Obsidian Inbox（如飞书秒记导出文本、`minute_token` 逐字稿拉取、飞书妙记搜索）并返回结构化 Inbox 路径；不要把飞书/照片等输入源逻辑耦合到发布模块。`intake feishu --draft` / `--preview` / `--push` 的 AI 草稿生成、本地 render、本地 preview 和“草稿后自动继续 push”都由 `src/app.rs` 编排调用 `src/ai_workflow.rs`、`src/render.rs`、`src/preview.rs`、`src/push.rs`，不要塞回 intake，也不要把微信网络细节散回 app 之外的其它模块。
- 飞书链路默认推荐 `intake feishu ... --draft --preview`：先停在可编辑草稿和本地 HTML 预览，再由用户确认是否继续。
- 只有显式 `--push` 才表示“继续推进到微信草稿”；不要把自动继续 push 当成飞书链路默认行为。
- 飞书官方秒记链路的幂等主键是 `minute_token`；只有 `--minute-token` / `--latest` / `--query` 这些路径应复用既有 Inbox 文件。不要把本地文本导入也偷偷扩成模糊去重。
- 当前 CLI 实际入口是全局 `--articles <path>`，不是 `--vault`；全局 `--json` 也必须放在子命令前面。2026-07-01 已用真实 `intake feishu --latest --draft --preview --no-open` 和 `intake feishu --latest --draft --push` 实证跑通到微信公众号后台预览发送成功。
- 当前产品收口优先级是“先让用户会用，再继续扩能力”；关于项目整体定位、飞书路线是否拆分以及近期阶段计划，先以 `docs/PRODUCT_EVALUATION_ZH.md` 为准，再决定是否继续横向扩功能。
- `obsidian-plugin/` 作为第三个用户入口时，发布前提示优先复用 `moonpub capabilities --json` 的元数据；不要再在插件里只靠 `process.env.WECHAT_*` 做硬阻断，因为 CLI 还会继续读取项目 `.env` 和 `~/.moonpub.env`。插件侧的“整体文章池状态”入口应优先消费 `moonpub workspace --json` 这种高层入口对象，而不是自己重新拼 `status` + `capabilities` 或回退到解析终端文本。
- `src/bundle.rs` 负责 `ArticleBundle`、文章阶段识别和 `drafts` / `ready` / `published` 之间的文章包移动；不要把状态移动逻辑放回 `src/push.rs` 或 `src/status.rs`。
- `src/plugin.rs` 负责内部 target trait、能力元数据、publish/export context/outcome 和调度 helper；新增平台时先实现 target，不要复制 CLI 编排。
- `src/render.rs` / `src/markdown.rs` 负责 Markdown 到微信 HTML 和 draft JSON；`src/markdown.rs` 只做顶层 block 分发，不放具体样式渲染。
- `src/markdown/parser.rs` 只放 `:::` block 与属性解析；`src/markdown/inline.rs` 负责行内 Markdown；`src/markdown/plain.rs` 负责普通段落/表格/列表/引用/代码块；`src/markdown/blocks.rs` 负责 `:::` fence block 渲染。
- `src/cover.rs` 负责封面样式解析、封面 HTML 生成/写入和 Chrome 截图辅助；`src/app.rs` 不直接拼封面路径或 Chrome headless 参数。
- `src/push.rs` / `src/wechat.rs` 负责微信 API；`push_article` 保持兼容 wrapper，底层走 `WechatDraftTarget`。
- 全局 `--json` 默认仍是文本包装；只有 `workspace`、`capabilities`、`status`、`check`、`preview`、`push`、`draft-from-inbox`、`intake feishu ... --draft` 返回命令专属结构化 JSON。`workspace` 要负责收口高层入口语义：工作区类型、推荐入口、阶段分布、能力摘要和推荐下一步；`status` / `check` 的 JSON 都要保留 `next_command` / `next_step`，用于把“当前状态”直接收口成“推荐下一步”；`draft-from-inbox` / `intake feishu ... --draft` 的 JSON 要保留 `action`，用于表达 `created` / `updated`；当链路显式带 `--push` 时，再额外补 `pushed`、`media_id`、`stage`、`next_step`。扩展新的机器可读输出时，先在 `src/app.rs` 明确列出命令边界，并同步 README / README_zh / USER_GUIDE / PROGRESS。
- `src/export.rs` 负责 Zola 导出；`export_article` 保持兼容 wrapper，底层走 `ZolaExportTarget`。
- `src/ship.rs` 负责一键发布编排：cover → thumb upload → render → push → optional export；`src/app.rs` 只传入解析后的命令参数。
- `src/publish.rs` / `src/cdp.rs` / `src/publish_steps.rs` 负责浏览器自动化。
- `moonpub login` 和任何扫码恢复路径在等待登录完成、保存 cookie / session 之前都必须持有活跃的 `Browser` 句柄；不要只保留 `Page` 然后提前丢掉 `Browser`，否则 CDP 会话会被提前取消并报 `oneshot canceled`。
- 浏览器自动化默认走持久 profile：`~/.config/moonpub/chrome-profile` + `~/.config/moonpub/session.json`；只有显式传 `--temporary-profile` 时才切到一次性隔离 profile，且该模式不读写持久 session。
- `push` / `ship` 成功创建微信草稿后，本地文章包进入 `Articles/ready/`；只有真实自动发布成功或用户手动 `mark-published` 后才进入 `Articles/published/`。

## 微信发布约束

- 微信编辑器是 live web app，DOM 和文案会变；自动化步骤应优先软失败，不能影响 API 草稿推送主流程。
- 本地 `preview` / `--preview` 只表示本地 HTML 预览；`configure` / `test-yulan` / `ship` 里的 preview-send 才是微信公众号后台预览发送，不要混为一谈。
- 定位微信编辑器元素时优先用 DOM 结构、class、input value；不要依赖不稳定的 `textContent`。
- 创作来源当前稳定路径是 `.js_claim_source_desc` 打开 picker，`input[type="radio"][value="4"]` 选择，`.js_claim_source_selected` 验证。
- API push 的 HTML 优先使用微信更稳定的 `<section>` / `<p>` / `<table>` 和 inline CSS；避免依赖会被编辑器剥离的标签样式。
- 配置里的资产路径（如 qrcode、cover）按 articles root 解析；文章内相对封面路径按文章所在目录解析。
- `[footer].variant = "minimal"` 用于闲月隐林/随笔类结尾，只渲染 `follow_image` / `follow_text`；`community` 保留社群结尾。旧配置里 `[footer].qrcode` 为空时也会隐藏社群标题、介绍、规则和入群提示。

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
