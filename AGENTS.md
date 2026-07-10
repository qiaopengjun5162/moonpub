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
- `src/app_article_commands.rs` 负责本地文章类命令包装：`render` / `cover` / `humanize` / `preview`；不要再把这些文件读写和本地预览细节塞回 `src/app.rs` 或重新堆到 `src/app_support.rs`。
- `src/app_draft_follow_up.rs` 负责 `draft-from-inbox` / `intake ... --draft` 的 preview / push / JSON / 文本收尾；不要再把这些 follow-up 分支塞回 `src/app.rs` 或 `src/app_support.rs`。
- `src/app_publish_commands.rs` 负责微信发布类命令包装：`push` / `publish --target wechat-draft` / 浏览器自动化错误包装；不要把这些 JSON/文本包装和 target 分发细节塞回 `src/app.rs`。
- `src/preflight.rs` 负责 `moonpub preflight <article.md>` 的本地发布前质量门：文章包三件套、HTML 排版审计聚合、`.media_id` 警告和下一步建议；不要在这里触发微信 API、打开 Chrome 或做后台自动化。
- `src/app_support.rs` 负责 `app.rs` 的薄协调层：配置加载、飞书 source 分派，以及这类仍未形成更明确专项模块的少量 helper；不要把这些 helper 再塞回 `src/app.rs`，也不要把它扩成新的总控中心。
- `src/app_support.rs` 现在不再承载 draft follow-up，也不再混合本地文章命令包装和微信发布命令包装；如果某段 `app_support` 逻辑已经明显形成稳定边界，优先继续下沉回对应模块，而不是无限把 `app_support.rs` 做大。
- `src/ai_workflow.rs` 负责 `write` / `draft-from-inbox` / `polish` / `expand` / `ship --ai` 的文件读写与 AI 调用编排；不要把 API key、文章写回、frontmatter 重组逻辑放回 `src/app.rs`。
- `src/init.rs` 负责 `moonpub init` 的交互/非交互配置生成和本地 `.env` 更新；不要把初始化向导细节塞回 `src/app.rs`。
- `src/draft.rs` 负责本地草稿文件创建、AI 生成文章写入和草稿路径/重复文件校验；不要把这些文件细节塞回 `src/app.rs`。
- `src/intake.rs` 负责上游素材导入到 Obsidian Inbox（如飞书秒记导出文本、`minute_token` 逐字稿拉取、飞书妙记搜索）并返回结构化 Inbox 路径；不要把飞书/照片等输入源逻辑耦合到发布模块。`intake feishu --draft` / `--preview` / `--push` 的 AI 草稿生成、本地 render、本地 preview 和“草稿后自动继续 push”都由 `src/app.rs` 编排调用 `src/ai_workflow.rs`、`src/render.rs`、`src/preview.rs`、`src/push.rs`，不要塞回 intake，也不要把微信网络细节散回 app 之外的其它模块。
- 飞书 `source: feishu-minutes` 素材生成草稿时，AI 提示应继续引导 `spoken-note` 口述随记配方：frontmatter 优先 `theme: letter`，正文优先 `intro` / `letter-card` / `summary` / `closing-card`，并保持实事求是、口语感和现场感；不要为了“像文章”把口述稿拔高成空泛长文。
- 飞书链路默认推荐 `intake feishu ... --draft --preview`：先停在可编辑草稿和本地 HTML 预览，再由用户确认是否继续。
- 照片链路第一版默认也推荐 `intake photos ... --draft --preview`：先把一组照片落到 `Inbox/Photos/`，再停在可编辑草稿和本地 HTML 预览，后续再决定是否继续推进到微信草稿。
- 只有显式 `--push` 才表示“继续推进到微信草稿”；不要把自动继续 push 当成飞书链路默认行为。
- 飞书官方秒记链路的幂等主键是 `minute_token`；只有 `--minute-token` / `--latest` / `--query` 这些路径应复用既有 Inbox 文件。不要把本地文本导入也偷偷扩成模糊去重。
- `src/intake.rs` 里的 Inbox frontmatter 现在以统一元数据结构为准：通用层优先认 `external_id`，飞书仍保留 `minute_token` 兼容字段；后续新增照片/语音输入源时，先复用这层元数据读写，不要回到手写 frontmatter 字符串。
- 微信公众号归档输入源目前只进入设计阶段，先看 `docs/WECHAT_ARCHIVE_WORKFLOW_ZH.md`；不要直接把它做成批量历史抓取或凭证采集工具。若后续实现，第一步只做用户显式提供的公开 URL -> Inbox，且不得保存或提交 cookie、pass_ticket、uin、token、二维码登录信息。参考 `moore-wechat-article-downloader` 时只吸收“已知 URL / Exporter 历史 / 订阅增量 / 代理历史 / 浏览增强”的风险分层，不要把代理增强、WebView 注入或系统代理修改做成默认能力。参考 `mp-weixin-to-md` 时只吸收“链接 -> 标准 Markdown、图片下载显式开启、本地 HTML fallback、验证页明确失败”的最小入口原则，不要内置 Cookie 或把图片下载做成默认行为。
- 若继续参考 `mcncarl/yichen-skills`，先看 `docs/YICHEN_SKILLS_REFERENCE_ZH.md`：只吸收草稿优先、dry-run、私有 vault、closeout/audit、隐私边界和逐句内容诊断等产品原则，不复制外部代码，不把微信本地库解密、桌面端 UI 操作、短视频抓取剪辑或多平台抓取塞进 v0.4.x / v0.5 主线。
- 若继续参考 `khoj-ai/khoj`，先看 `docs/KHOJ_REFERENCE_ZH.md`：只吸收本地优先知识层、来源引用、多入口共享协议等长期产品原则；不要把 MoonPub 近期改成完整二脑、向量搜索引擎、聊天 SaaS 或后台自动化发文系统。未来如做 `search` / `ask`，默认只读、返回来源、不触发微信 API、不打开浏览器、不写回文件。
- 若继续参考 `Sac-Y/identity-skill`，先看 `docs/IDENTITY_SKILL_REFERENCE_ZH.md`：只吸收参考图驱动、阻塞确认、素材拆分、素材评审和视觉 QA 台账等设计流程原则；不要把 MoonPub 近期改成个人网站生成器，不复制外部模板，不让图像生成或前端复刻成为基础发布链路前置依赖。
- 若继续参考 `helloianneo/ian-handdrawn-ppt`，先看 `docs/IAN_HANDDRAWN_PPT_REFERENCE_ZH.md`：只吸收文章内容到封面 / 正文解释图的叙事规划、语义版式、短文字质量门和 contact sheet QA 原则；不要把 MoonPub 近期改成 PPT/PPTX 生成器，不默认给每篇文章生图，不让图像生成成为发布前置依赖，也不要用生成插图替代生活照片的真实记录。
- 若继续参考 `AstrBotDevs/AstrBot` README，先看 `docs/ASTRBOT_README_REFERENCE_ZH.md`：只吸收 README 第一屏入口聚合、支持矩阵、安装路径分层、路线图和社区承接等产品表达原则；不要把 MoonPub 近期改成聊天机器人框架、多平台 Bot 系统、模型路由平台或 Web 管理后台。
- 若继续参考 `Thysrael/Horizon`，先看 `docs/HORIZON_REFERENCE_ZH.md`：只吸收多源素材进入写作系统前的去重、评分、来源保留、日报草稿和可解释候选原则；不要把 MoonPub 近期改成新闻抓取平台、自动资讯站、邮件分发系统或后台定时发布机器。
- 若继续参考用户提供的“Obsidian + AI 内容生产线”文章，先看 `docs/OBSIDIAN_AI_PIPELINE_REFERENCE_ZH.md`：只吸收本地 Markdown 资产、Inbox 优先、AI 辅助整理、内容可追溯和少插件先跑通主线原则；不要把 MoonPub 近期改成通用知识库、Obsidian 教程或无人值守自动发布器。
- 当前 CLI 实际入口是全局 `--articles <path>`，不是 `--vault`；插件和脚本仍推荐把 `--json` 放在子命令前面，但结构化工作流 / 发现命令也兼容后置 `--json`，例如 `workspace --json`。2026-07-01 已用真实 `intake feishu --latest --draft --preview --no-open` 和 `intake feishu --latest --draft --push` 实证跑通到微信公众号后台预览发送成功。
- 当前产品收口优先级是“先让用户会用，再继续扩能力”；关于项目整体定位、飞书路线是否拆分以及近期阶段计划，先以 `docs/PRODUCT_EVALUATION_ZH.md` 为准，再决定是否继续横向扩功能。
- 如果当前工作是在补“产品到底是什么”的表达，先看 `docs/PRODUCT_WRAP_ZH.md`：它负责收口一层定位、三层结构、当前正式输入工作流和正式入口层；不要再把这类信息继续散落到 README 首屏、聊天记录和零碎说明里。
- 如果当前工作是在补“第一次到底怎么开始”的表达，先看 `docs/FIRST_RUN_WALKTHROUGH_ZH.md`，优先把插件首页、飞书、照片、当前文章这几条首次体验路径收口到一起，而不是把第一次使用说明继续散落到 README、用户指南和插件 README 的不同角落。
- 如果当前工作是在复查“首次使用到底打通到什么程度”，先看 `docs/FIRST_RUN_AUDIT_ZH.md`，按“强证据 / 已通过 / 待补证据”的方式继续推进，不要把“代码已实现”直接写成“首次体验已完全打通”。
- 如果当前工作是在补“首次体验的真实截图 / 录屏 / 样例证据”，先看 `docs/FIRST_RUN_EVIDENCE_CHECKLIST_ZH.md`，按统一证据清单补首页、飞书、照片和真实微信回归材料；可以用 `moonpub evidence-status` 检查文件是否齐全，用 `moonpub evidence-status --strict` 作为证据文件非零退出检查，用 `moonpub release-check --strict` 聚合 v0.4.2 release gate；它们都不替代人工脱敏审查，不要把真实取证步骤继续散落到聊天记录里。
- 如果当前工作是在推进阶段目标或排序下一步，先看 `docs/EXECUTION_PLAN_ZH.md`，再决定本轮实现落点，避免把计划继续散落到聊天和零碎文档里。
- `obsidian-plugin/` 作为第三个用户入口时，首页先用 `moonpub --json doctor` 做本地可用性诊断，再用 `moonpub --json workflow-registry` 展示正式工作流、安全起点、用户价值和风险边界，用 `moonpub --json evidence-status` 展示 release 证据缺口，用 `moonpub --json release-check` 展示 v0.4.2 发布门禁，最后用 `moonpub --json workspace` 展示工作区状态；发布前提示优先复用 `moonpub --json capabilities` 的元数据。不要在插件里只靠 `process.env.WECHAT_*` 做硬阻断，因为 CLI 还会继续读取项目 `.env` 和 `~/.moonpub.env`。插件侧的“整体文章池状态”入口应优先消费 `doctor` / `workflow-registry` / `evidence-status` / `release-check` / `workspace` 这种高层入口对象，而不是自己重新拼 `status` + `capabilities` 或回退到解析终端文本。
- 插件首页里的 `workflow-registry` 展示应继续映射到保守安全入口：当前文章先检查/预览，飞书和照片先停在草稿与本地预览，`wechat-draft` 只提示边界，不要从首页直接触发微信草稿推进。
- Obsidian 插件缺 CLI 或素材入口缺 `Articles 根目录` 时，应打开修复工作台列出安装 / 路径 / 根目录修复步骤；不要只弹一条 Notice 让第一次使用的用户自己猜下一步。
- Obsidian 插件里的“查看整体文章池状态”现在应继续把 `moonpub --json workspace` 结果展开成工作台式展示，而不是只塞进一条长 Notice；后续如果继续优化插件首页，优先强化这层入口和推荐下一步，不要先扩更多按钮。
- `查看整体文章池状态` 这层工作区工作台现在也承担插件首页角色：优先把飞书、照片、当前文章这些高频入口收在同一个工作台里，而不是继续分散到越来越多独立命令说明里。
- 如果继续优化插件首页，优先补上下文感知推荐：当前打开的是 Markdown 就优先引导当前文章路径，当前打开的是图片就优先引导照片路径；不要先把首页做成静态按钮墙。
- Obsidian 插件里的“检查当前文章状态”也应继续把 `check --json` 结果展开成当前文章工作台，优先帮助用户判断当前文件缺什么、下一步做什么；不要只返回压缩状态串。
- Obsidian 插件工作台可以提供“复制下一步命令”这类低风险本地辅助动作；复制命令不等于执行命令，不能借此绕过微信草稿推进的显式确认。
- 当前文章工作台和飞书 / 照片结果工作台里的发布前检查按钮都应调用 `moonpub --json preflight <article.md>`，只读本地产物，不触发微信 API、不打开或控制 Chrome；缺 `.media_id` 只作为还没推进微信草稿的提醒。
- 当前文章工作台和飞书 / 照片结果工作台里的排版审计按钮都应调用 `moonpub --json layout-audit <html>`，只检查本地 HTML，不触发微信 API、不自动打开浏览器；审计结果弹窗可以提供显式“打开 HTML 预览”动作。插件拼所有全局 JSON 命令时仍优先把 `--json` 放在子命令前，例如 `moonpub --articles <path> --json check <article.md>`；CLI 已兼容 `check <article.md> --json` 这类手工后置写法，但不要把它作为插件内部规范。
- Obsidian 插件里的素材入口现在不只包括飞书，也包括照片：当用户当前打开的是图片文件时，可以直接用“当前图片所在目录”去触发 `moonpub --json intake photos ... --draft --preview`。后续如果继续扩素材入口，优先沿着“当前上下文直接起工作流”的模式扩，不要先做重输入框和重复表单。
- `src/bundle.rs` 负责 `ArticleBundle`、文章阶段识别和 `drafts` / `ready` / `published` 之间的文章包移动；不要把状态移动逻辑放回 `src/push.rs` 或 `src/status.rs`。
- `src/plugin.rs` 负责内部 target trait、能力元数据、publish/export context/outcome 和调度 helper；新增平台时先实现 target，不要复制 CLI 编排。
- `src/render.rs` / `src/markdown.rs` 负责 Markdown 到微信 HTML 和 draft JSON；`src/markdown.rs` 只做顶层 block 分发，不放具体样式渲染。
- `src/markdown/parser.rs` 只放 `:::` block 与属性解析；`src/markdown/inline.rs` 负责行内 Markdown；`src/markdown/plain.rs` 负责普通段落/表格/列表/引用/代码块；`src/markdown/blocks.rs` 负责 `:::` fence block 渲染。
- `src/cover.rs` 负责封面样式解析、封面 HTML 生成/写入和 Chrome 截图辅助；`src/app.rs` 不直接拼封面路径或 Chrome headless 参数。
- `src/push.rs` / `src/wechat.rs` 负责微信 API；`push_article` 保持兼容 wrapper，底层走 `WechatDraftTarget`。
- 全局 `--json` 默认仍是文本包装；只有 `doctor`、`workspace`、`workflow-registry`、`evidence-status`、`release-check`、`layout-recipes`、`layout-audit`、`wechat-health`、`capabilities`、`status`、`check`、`preflight`、`preview`、`push`、`draft-from-inbox`、`intake feishu ... --draft`、`intake photos ... --draft` 返回命令专属结构化 JSON。这些结构化入口也兼容后置 `--json`，用于降低手工 CLI 使用摩擦；插件和脚本仍以全局前置 `--json` 为规范。`doctor` 只做本地可用性诊断，不触发微信 API、不打开 Chrome、不读取或打印真实 secret；`workspace` 要负责收口高层入口语义：工作区类型、推荐入口、阶段分布、能力摘要和推荐下一步；`workflow-registry` 要负责暴露正式工作流契约：路径 id、package、owner、安全起点、下一步命令、用户价值、风险标记、生产边界、证据状态和文档入口；`evidence-status` 只检查 `docs/first-run-evidence/` 约定证据文件是否存在，JSON 要保留 `required_count`、`present_count`、`missing_count` 和 `missing_paths`，不打开图片、不读取图片内容、不替代人工脱敏审查；默认模式只报告状态，`--strict` 缺少必需文件时非零退出，用于 release 脚本或 CI gate；`release-check` 聚合 v0.4.2 release gate 文档勾选状态和证据文件状态，默认只报告，`--strict` 任一 gate 未完成时非零退出；`layout-recipes` 要负责暴露排版配方发现语义：配方 id、适用场景、推荐 theme 和 Block 组合；`layout-audit` 要负责本地 HTML 排版质量门，只检查公众号编辑器兼容风险，不触发 render / push / 浏览器自动化；`preflight` 要负责发布前本地只读聚合质量门，缺 `.media_id` 只警告，不代表本地产物失败；`wechat-health` 要负责发文前浏览器自动化健康检查，只输出脱敏后的 URL 和恢复建议，不能打印微信后台 token；`status` / `check` 的 JSON 都要保留 `next_command` / `next_step`，用于把“当前状态”直接收口成“推荐下一步”；`draft-from-inbox` / `intake ... --draft` 的 JSON 要保留 `command`、`action`，用于表达具体输入工作流和 `created` / `updated`；当链路显式带 `--push` 时，再额外补 `pushed`、`media_id`、`stage`、`next_step`。扩展新的机器可读输出时，先在 `src/app.rs` 明确列出命令边界，并同步 README / README_zh / USER_GUIDE / PROGRESS。
- 结构化输出的具体 builder 当前已开始收口到 `src/protocol.rs`；后续新增或修改 `doctor` / `workspace` / `workflow-registry` / `layout-recipes` / `layout-audit` / `wechat-health` / `status` / `check` / `preflight` / `preview` / `push` / `draft-from-inbox` / `intake ... --draft` 这些 payload 时，优先继续维护协议模块，不要把手写 JSON builder 再塞回 `src/app.rs`。
- `moonpub layout-audit <html>` 是排版质量门，只检查本地 HTML 结构和公众号编辑器兼容风险，不触发 render / push / 浏览器自动化；后续扩展排版主题或 Block 时优先用它补回归，不要直接把外部 skill 的组件库代码复制进来。
- 对 `draft-from-inbox` / `intake ... --draft` 来说，只要 JSON 里返回了 `html_path`，就必须先真实生成 HTML 预览产物，不能只凭路径推导结果对象。
- 如果要给 `draft-from-inbox` / `intake ... --draft` 补 app 级行为测试，优先走 test-only AI 响应替换点，验证真实 Inbox/draft/html/draft.json 产物，而不是只测 JSON builder。
- 飞书和照片既然都已经是正式输入工作流，后续新增行为测试时要尽量保持两条链路同等级，不要让一条只有 builder/unit 测试，另一条已经有 app 级回归。
- `src/export.rs` 负责 Zola 导出；`export_article` 保持兼容 wrapper，底层走 `ZolaExportTarget`。
- `src/ship.rs` 负责一键发布编排：cover → thumb upload → render → push → optional export；`src/app.rs` 只传入解析后的命令参数。
- `src/publish.rs` / `src/cdp.rs` / `src/publish_steps.rs` 负责浏览器自动化。
- `moonpub login` 和任何扫码恢复路径在等待登录完成、保存 cookie / session 之前都必须持有活跃的 `Browser` 句柄；不要只保留 `Page` 然后提前丢掉 `Browser`，否则 CDP 会话会被提前取消并报 `oneshot canceled`。
- 浏览器自动化默认走持久 profile：`~/.config/moonpub/chrome-profile` + `~/.config/moonpub/session.json`；只有显式传 `--temporary-profile` 时才切到一次性隔离 profile，且该模式不读写持久 session。`push` / `publish --target wechat-draft` 的 `--temporary-profile` 只影响草稿创建成功后的公众号后台自动化，微信 API 推草稿本身不需要浏览器 profile。
- headless 浏览器自动化不能要求用户扫码：如果持久 session 恢复失败，`configure` / `push` 后自动配置这类不可见流程必须快速失败并提示 `moonpub login` 或 `configure --headed`，不要在用户看不见二维码时等待 120 秒。
- `moonpub wechat-health` 是发文前浏览器自动化预检入口：不发草稿、不修改微信后台，只判断当前 profile/session 是否能进入公众号后台，并返回 `ready` / `needs_login` 与下一步命令。
- 如果 Chrome 启动失败并出现 `SingletonLock` / `ProcessSingleton`，通常是持久 profile 已被另一个 MoonPub 自动化 Chrome 占用；错误提示应建议关闭现有自动化 Chrome 窗口，或显式加 `--temporary-profile`，不要把它误判成微信 token 过期。
- 2026-07-03 已用真实 Obsidian articles 根目录和当前 source build 跑通 `test-yulan --headed` 与 `configure --headed`：持久 session 恢复、进入编辑器、原创声明、赞赏、留言、创作来源和微信公众号后台预览发送均成功；`[template].name` 未配置时模板插入软跳过是预期行为。
- `push` / `ship` 成功创建微信草稿后，本地文章包进入 `Articles/ready/`；只有真实自动发布成功或用户手动 `mark-published` 后才进入 `Articles/published/`。

## 微信发布约束

- 微信编辑器是 live web app，DOM 和文案会变；自动化步骤应优先软失败，不能影响 API 草稿推送主流程。
- 本地 `preview` / `--preview` 只表示本地 HTML 预览；`configure` / `test-yulan` / `ship` 里的 preview-send 才是微信公众号后台预览发送，不要混为一谈。
- 定位微信编辑器元素时优先用 DOM 结构、class、input value；不要依赖不稳定的 `textContent`。
- 创作来源当前稳定路径是 `.js_claim_source_desc` 打开 picker，`input[type="radio"][value="4"]` 选择，`.js_claim_source_selected` 验证。
- API push 的 HTML 优先使用微信更稳定的 `<section>` / `<p>` / `<table>` 和 inline CSS；避免依赖会被编辑器剥离的标签样式。
- `MOONPUB_DEBUG_PROXY=1` 只用于微信 API 代理排障；日志必须使用脱敏 URL，不能打印 `access_token` query。
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
- 渲染主题当前为 23 种：`default` / `warm` / `dark` / `geek` / `paper` / `magazine` / `notebook` / `classic` / `forest` / `sunset` / `ocean` / `mono` / `editorial` / `zen` / `newsletter` / `academic` / `cyber` / `letter` / `mist` / `gallery` / `moonlit` / `porcelain` / `fieldnote`。
- Block 模板当前为 20 种：`book-info` / `intro` / `callout` / `steps` / `summary` / `figure` / `checklist` / `key-points` / `pull-quote` / `cover` / `letter-card` / `scene-card` / `closing-card` / `compact-links` / `photo-grid` / `meta-strip` / `quote-card` / `divider` / `concept-card` / `emotion-card`。`compact-links` 用于渲染资料索引小字号链接行，适合 QunMind 日报的“参考来源 / 完整素材链接”区；标题不再额外做链接，唯一链接入口应保留在完整原文 URL。
- `PROGRESS.md` 记录真实完成度；不要把本地单元测试通过写成真实微信端验证通过。
- 对外主推 release 前必须下载 release 资产跑 smoke test；源码构建二进制通过不能替代 release 二进制验证。
- `.cargo/config.toml` 的 `target-cpu=native` 只适合本地开发；CI/release 构建必须覆盖为可移植 flags，避免 macOS ARM64 上 `ring` 编译失败。
- 当前入口文档（`README*`、`docs/GETTING_STARTED.md`、`docs/USER_GUIDE.md`、`docs/index.html`、`docs/slides.html`、`PROGRESS.md`）要跟最新源码能力和当前主推安装版本同步。
- 发布归档/快照文档（如 `docs/RELEASE_NOTES_v0.4.1.md`、`docs/LAUNCH_*`、`docs/WECHAT_REGRESSION_CHECKLIST_ZH.md`）保留当时的版本事实；不要为了“统一口径”随手改成新版本。
- `docs/moonpub.rb` 是带真实 `sha256` 的 Homebrew 维护模板；没有对应 release 资产的真实校验和时，不要只改版本号和下载链接。
