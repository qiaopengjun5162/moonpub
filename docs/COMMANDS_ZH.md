# MoonPub 命令参考

每个 `moonpub` 命令的 flag 级参考。一行一命令的紧凑表在 [README_zh.md](../README_zh.md)；
命令自身的帮助页直接用：

```bash
moonpub help             # 按分组看命令列表
moonpub help <command>   # 单命令用法与说明
moonpub help errors      # 全部 error/v1 错误码与示例下一步
moonpub --json help      # 机器可读命令注册表（help/v1）
moonpub --json help errors  # 机器可读错误码注册表
```

下面的命令全集以 `src/help.rs`（唯一真相源）为准，测试会保证本文件、
两份 README 和注册表不漏命令。

## 全局 flag（放在命令前面）

```
--articles <path>    存放 Articles/ 的工作区根目录（默认当前目录）
--config <path>      配置文件（默认 <root>/moonpub.toml）
--json               机器可读输出
```

## 快速上手

```
moonpub init [moonpub.toml] [--yes]   创建示例 moonpub.toml
moonpub doctor                        检查首次使用前的本地环境
moonpub workspace                     概览工作区并推荐下一步命令
moonpub status                        列出 drafts / ready / published 中的文章
moonpub help [<command> | errors]     命令列表、单命令用法或错误码清单
moonpub version                       显示版本号
```

`init` 在终端里会进入交互向导；加 `--yes`（或 stdin 非交互）时直接写默认示例配置、
不等待输入。它永远不会覆盖已存在的 `moonpub.toml`。

## 写作

```
moonpub new <title>                   创建带 frontmatter 模板的新文章
moonpub write <idea>                  从想法生成文章（AI）
moonpub expand <article.md>           把读书笔记展开成完整文章（AI）
moonpub polish <article.md>           AI 润色 + 去 AI 味
moonpub humanize <article.md>         离线去除 AI 写作痕迹
moonpub draft-from-inbox <inbox.md> [--preview] [--no-open] [--push]
                                      从 Inbox 素材生成可编辑草稿
```

`write` / `expand` / `polish` / `draft-from-inbox` 需要 AI API key
（`DEEPSEEK_API_KEY` 或 `OPENAI_API_KEY`）；`humanize` 完全离线。
`draft-from-inbox` 默认推荐 `--preview` 保守路径；只有显式 `--push`
才快进到微信草稿推送。

## 渲染与审阅

```
moonpub render <article.md> [--author <name>] [--thumb <media_id>]
                                      生成 <slug>.html 和 <slug>.draft.json
moonpub check <article.md>            检查文章包 md / html / draft.json 三件套
moonpub preflight <article.md> [--strict]
                                      发布前本地只读质量门：三件套 + 排版审计 + 下一步
moonpub layout-audit <html>           检查微信 HTML 的编辑器兼容风险
moonpub layout-recipes                列出排版配方和推荐主题
moonpub preview <article.md> [--no-open]
                                      在浏览器打开本地 HTML 预览
moonpub cover <article.md> [--style <name>] [--screenshot]
                                      从文章 frontmatter 生成封面
moonpub cards <article.md>            把文章拆成小红书式竖版卡片
```

行为说明：

- `preflight` 不触发微信 API、不打开 Chrome。它检查 Markdown / HTML /
  `draft.json` 三件套，HTML 存在时跑排版审计，缺 `.media_id` 只算警告
  （只代表还没推到微信草稿），并返回下一步安全命令。
- `preflight` 默认**即使 `passed` 为 false 也返回退出码 0**：报告是「建议性」的，
  插件和 Agent 必须自己读 `passed`。加 `--strict` 才变成进程级门禁，供脚本 / CI
  串联（`moonpub preflight a.md --strict && moonpub push a.md`）：任一项 fail 就非零
  退出，并在 stderr 逐条列出失败检查和推荐的下一步命令，此时 stdout 不输出 `checks[]`
  报告（带 `--json` 时改为输出一个 `error/v1` 失败对象）。需要完整 `checks[]`
  结构化结果时，用不带 `--strict` 的 `--json` 形式。
- `layout-audit` 检查禁用标签、禁用属性、整页外壳和高风险 CSS。
- `layout-recipes` 覆盖生活随笔、安静开场、口述随记、合集开篇、记忆手记、
  照片故事、日更贴图、读书笔记、技术长文、AI 工程笔记、系统设计评审、
  带来源索引的日报和透明信源简报；同时返回主题分组，方便做主题选择器。
- `cover` 风格解析顺序：`--style` > frontmatter `cover_style` > `theme` >
  `editorial`；`ship` 走同一条链，两处渲染结果一致。
- `preview` 是本地 HTML 预览，不是微信公众号后台预览发送。
- `render` 与 `cover` 的 flag 写在 `<article.md>` 前后都行
  （`cover --style dark a.md` 等价于 `cover a.md --style dark`）；未知 flag、
  多余的位置参数、把 flag 当成另一个 flag 的值，都会非零退出，不再静默忽略。

## 发布与微信后台

```
moonpub login [--temporary-profile]   一次性扫码登录微信后台
moonpub wechat-health [--headed] [--temporary-profile]
                                      检查已保存的浏览器会话是否可复用
moonpub push <article.md> [--render] [--temporary-profile]
                                      推送草稿、写 .media_id、移动到 ready/
moonpub publish <article.md> --target wechat-draft [--render] [--temporary-profile]
                                      通用发布入口（当前只有 wechat-draft）
moonpub update-draft <article.md> [--media-id <id>]
                                      把更新后的 HTML 重推到已有微信草稿
moonpub ship <article.md> [--style <name>] [--ai]
                                      封面 + 渲染 + 推送 + 配置 + 导出一步到位
moonpub configure [<step>..] [--headed] [--temporary-profile] [--evidence-dir <dir>]
                                      自动配置原创 / 赞赏 / 留言 / 创作来源并预览发送
moonpub step-test [--headed] [--temporary-profile]
                                      交互式浏览器自动化测试
moonpub test-zanshang [--headed] [--temporary-profile]
                                      只测试赞赏步骤
moonpub test-chuangzuo [--headed] [--temporary-profile]
                                      只测试创作来源步骤
moonpub test-yulan [--to <wxid>] [--title <text>] [--headed] [--temporary-profile]
                                      只测试后台预览发送步骤
moonpub list-drafts                   列出所有微信草稿及 media_id 和标题
moonpub delete-draft <media_id>       按 media_id 删除草稿
moonpub mark-ready <article.md>       把文章包移动到 Articles/ready
moonpub mark-published <article.md>   把文章包移动到 Articles/published
moonpub export <article.md> [--target zola]
                                      通用导出入口（当前只有 zola）
```

行为说明：

- 第一次后台预览发送需要接收人：`--to <wxid>`、`WECHAT_PREVIEW_TO` 或已保存的
  `.moonpub/preview_to`。成功一次后后续自动；没配接收人时该步骤打印一次性提示并跳过，
  其余流程照常完成。
- `[wechat].auth_method = "cookie"`（或 `WECHAT_AUTH_METHOD=cookie`）时 `ship`
  复用 `moonpub login` 保存的浏览器会话绕开微信 IP 白名单；cookie 模式下 `ship`
  不需要 `WECHAT_APPID` / `WECHAT_SECRET`。
- `push` 遇到文章包旁已有 `.media_id` 时，会先建新草稿并更新本地 `.media_id`，
  成功后按旧 `media_id` 尽力删除旧草稿（best-effort）；不会按标题删除，避免误删同名草稿。
- `--temporary-profile` 让推送后的后台自动化使用一次性隔离 Chrome profile，
  通常需要重新扫码。
- `delete-draft` 是立即删除：没有 `--dry-run`，`media_id` 之后的任何参数都会被拒绝而不是被忽略。固定参数个数的命令（`status` / `version` / `workspace` / `layout-recipes` / `list-drafts` / `fetch` / `delete-draft`）同样拒绝多余参数。
- `publish`、`update-draft`、`ship`、`export` 与 `render` / `cover` 遵循同一套参数规则：
  flag 写在路径前后都行；缺值、未知 flag、多余位置参数都会非零退出。这条规则同时保证
  Obsidian 插件发出的 `ship --ai <路径>` 能正常解析。
- 最终发布始终由人工在微信后台确认，这是设计边界。

## 导入与研究

```
moonpub fetch <url>                   抓取微信文章并提取标题 + 正文（需要 Chrome）
moonpub intake feishu <file> | --latest | --query <kw> | --minute-token <t>
                                      导入飞书秒记 / 妙记到 Obsidian Inbox
moonpub intake photos <文件或目录>... [--analyze-images] [--draft] [--preview] [--no-open] [--push]
                                      导入一组真实照片到 Inbox/Photos
moonpub radar <add|list|import|analyze|suggest|scrape> ...
                                      存储并分析平台热点样本
```

intake 行为说明：

- 两个输入源的保守默认都是 `--draft --preview`；只有显式 `--push` 才继续推进到
  微信草稿。预览分两级：本地 HTML 预览（`moonpub preview`，不需要凭证）和微信
  后台预览发送（`configure` 的一部分，或单独 `test-yulan`）。
- 飞书官方妙记路径（`--minute-token` / `--latest` / `--query`）按统一输入元数据的
  `external_id` 幂等：重复执行复用同一 Inbox 文件，重复生成草稿复用同一草稿路径并以
  `action: "created" | "updated"` 区分；`minute_token` 保留为来源专属兼容字段。
- `intake photos` 把真实图片归档到 `Inbox/Photos/`，写统一 Inbox 元数据
  （`source: photos`、`type: photo-note`、`external_id`、`captured_at`），并基于
  文件路径 / 大小 / 时间生成事实型素材稿。`--analyze-images` 是显式 OpenAI 专属
  选项：最多 5 张 jpg/jpeg/png/webp（单张 8 MiB、合计 20 MiB）做谨慎的可见信息
  描述，结果写回 Inbox 并标为「需人工核对」，绝不自动推送微信草稿。

radar 子命令签名：

```
moonpub radar add --platform <name> --keyword <text> --title <text> [--url <url>] [--author <name>] [--likes/--collects/--comments <n>] [--source <text>]
moonpub radar list [--platform <name>] [--keyword <text>]
moonpub radar import <file.csv> [--platform <name>]
moonpub radar analyze <article.md> --platform <name> [--top <n>]
moonpub radar suggest <article.md> --platform <name> [--top <n>]
moonpub radar scrape --platform <name> --keyword <text> [--count <n>] [--url <url>]
```

## 集成与发布门禁

```
moonpub capabilities                  列出内置 target 的能力和风险元数据
moonpub workflow-registry             列出插件 / App / Agent 可发现的工作流契约
moonpub wechat-checklist              本地只读公众号内容复盘清单
moonpub evidence-status [--strict]    检查 release 证据文件是否齐全
moonpub release-check [--strict]      汇总 release gate 状态
```

`evidence-status` 只检查文件是否存在，不打开、不检查内容；`--strict` 在缺失时非零退出。
`release-check --strict` 在任一 gate 未完成时非零退出。

## JSON 输出契约

`--json` 主要面向自动化。插件和脚本推荐写 `moonpub --json <command>`；下面这些
结构化工作流 / 发现命令也兼容后置 `moonpub <command> --json`，`help` / `version`
同样支持后置形式（`version` 走 `{"output":"..."}` 包装）。不在集合里的命令用前置
`--json` 时仍退回 `{"output":"..."}` 包装。

失败时退出码非零、人类可读错误文本在 stderr。纯文本模式 stdout 为空；带 `--json`
时 stdout 输出**一个** `error/v1` 对象：`schema_version`、`ok: false`、`command`、
`error.code`、`error.message`、`cause_chain[]` 和 `next_step`
（例如 `unknown_command` → `moonpub help`，`push_failed` → `moonpub wechat-health`）。
`next_step` **永远是可执行的 `moonpub` 命令**：上游报告若给的是散文，就回落到该
错误码的 `example_next_step`，原始提示仍留在 `error.message`。
`error.code` 的全量清单可在运行时用 `moonpub --json help errors` 查（人读用
`moonpub help errors`），**不要在代码里硬抄文档里的清单**。完整约定见
[AGENT_PROTOCOL_ZH.md](AGENT_PROTOCOL_ZH.md) 的「失败契约」。
**stdout 能解析出对象不代表成功**，仍要检查退出码。

- `moonpub --json help` → `schema_version: "help/v1"`、`moonpub_version`、`global_options[]`、`groups[]`、`commands[]`（完整命令注册表）、`error_codes[]`（完整 error/v1 错误码表）
- `moonpub help <command> --json` → 同一注册表里的单命令页
- `moonpub --json help errors` → `schema_version: "help/v1"`、`moonpub_version`、`error_schema_version: "error/v1"`、`error_codes[]`；每项含 `code` / `summary` / `example_next_step`
- `moonpub doctor --json` → `command`、`schema_version: "doctor/v1"`、`moonpub_version`、`articles_root`、`config_status`、`capabilities_summary[]`、`warnings[]`、`next_step`、`next_command`
- `moonpub workspace --json` → `command`、`schema_version: "workspace/v1"`、`workspace_kind`、`entry_path`、`entry_path_label`、`total_articles`、`stage_counts`、`stages[]`、`capabilities[]`、`next_command`、`next_step`
- `moonpub workflow-registry --json` → `command`、`schema_version: "workflow-registry/v1"`、`source`、`workflows[]`；每条含 `id`、`package`、`status`、`owner`、`safe_start_command`、`next_command`、`user_value`、风险标记、生产边界、证据状态和文档入口
- `moonpub evidence-status --json` → `command`、`schema_version: "evidence-status/v1"`、`base_dir`、`passed`、`required_count`、`present_count`、`missing_count`、`missing_paths[]`、`sections[]`、`next_step`、`next_command`
- `moonpub release-check --json` → `command`、`schema_version: "release-check/v1"`、`release_version`、`repo_root`、`passed`、`checks[]`、`next_step`、`next_command`。其中 `next_command` 是**人工核对提示**（可能是 `review README.md ...` 这类散文，也可能是 `git status --short` 这类 shell 片段），不保证是 `moonpub` 命令，不要直接当命令执行；加 `--strict` 失败时，`error/v1` 里的 `next_step` 仍保证可执行
- `moonpub wechat-checklist --json` → `command`、`schema_version: "wechat-checklist/v1"`、`source`、`boundary`、`sections[]`、`next_command`、`next_step`
- `moonpub layout-recipes --json` → `command`、`schema_version: "layout-recipes/v1"`、`guide`、`theme_groups[]`、`theme_spotlights[]`、`recipes[]`；配方含 `id`、`title`、`best_for`、`themes[]`、`blocks[]`
- `moonpub --json layout-audit <html>` → `command`、`schema_version: "layout-audit/v1"`、`html_path`、`passed`、`errors[]`、`warnings[]`、`next_step`
- `moonpub wechat-health --json` → `command`、`schema_version: "wechat-health/v1"`、`status`、`profile_mode`、`session_file`、`session_file_exists`、脱敏后的 `current_url`、`next_command`、`next_step`
- `moonpub status --json` → `command`、`schema_version: "status/v1"`、`stages[]`、`next_command`、`next_step`；每个 stage 含 `stage`、`count`、`files[]`，文件项含 `file`、`slug`、`latest_status`、`latest_detail`
- `moonpub check <article.md> --json` → `command`、`schema_version: "check/v1"`、`article_path`、`html_path`、`draft_json_path`、`media_id_path`、`has_markdown`、`has_html`、`has_draft_json`、`has_media_id`、`publishable`、`theme`、`effective_theme`、`theme_source`、`next_command`、`next_step`
- `moonpub preflight <article.md> --json` → `command`、`schema_version: "preflight/v1"`、`article_path`、`html_path`、`draft_json_path`、`media_id_path`、`passed`、`checks[]`、`next_command`、`next_step`
- `moonpub preview <article.md> --json` → `command`、`schema_version: "preview/v1"`、`article_path`、`html_path`、`opened_browser`、`next_command`
- `moonpub push <article.md> --json` → `command`、`schema_version: "push/v1"`、`article_path`、`media_id`、`stage`、`next_step`
- `moonpub draft-from-inbox <inbox.md> --json` → `command`、`schema_version: "draft-from-inbox/v1"`、`input_path`、`draft_path`、可选 `html_path`、`action`、`next_command`；加 `--push` 时还有 `pushed`、`media_id`、`stage`、`next_step`
- `moonpub intake feishu ... --draft --json` → `command`、`schema_version: "intake-feishu/v1"`、`inbox_path`、`draft_path`、可选 `html_path`、`action`、`next_command`；加 `--push` 时还有 `pushed`、`media_id`、`stage`、`next_step`
- `moonpub intake photos ... --draft --json` → `command: "intake-photos"`、`schema_version: "intake-photos/v1"`、`inbox_path`、`draft_path`、可选 `html_path`、`action`、`next_command`；加 `--push` 时还有 `pushed`、`media_id`、`stage`、`next_step`
- `moonpub capabilities`（始终结构化）→ 顶层 `schema_version` / `moonpub_version` 加每个 target 的风险元数据、前置条件和 argv 风格 `command` 模板。调用方应先检查 schema，展示缺失的 `required_env` / `required_config`，替换 `"{article}"` 占位符后用进程参数数组调用，不要拼 shell 字符串、不要存真实 secret。

## 排障

- 微信 API 网络不通、代理不确定时，临时加 `MOONPUB_DEBUG_PROXY=1` 查看实际选择的代理。
  调试日志只输出脱敏后的 URL，不打印 `access_token` query。
- `errcode=40164` 里 `current IP` 反复变化时，先关闭旋转代理或固定稳定公网出口，
  再更新微信 IP 白名单；不要把不断变化的历史 IP 全部加进白名单。

## 已有文档但还不是命令的方向

这些有参考文档和产品边界，但当前没有 CLI 入口：

- 微信公众号归档输入源：安全第一步是「用户显式提供的公开文章 URL →
  `Inbox/WechatArchive/` → 草稿和本地预览」，不默认抓历史、不保存
  cookie / `pass_ticket` / `uin` / token。见
  [WECHAT_ARCHIVE_WORKFLOW_ZH.md](WECHAT_ARCHIVE_WORKFLOW_ZH.md)。
- Khoj 式本地知识助手：只读搜索 `Articles/` / `Inbox/`，返回文件来源，
  不触发微信 API、不打开浏览器。见 [KHOJ_REFERENCE_ZH.md](KHOJ_REFERENCE_ZH.md)。
- Identity Skill 式参考图驱动视觉流程：brief → 参考图 → 素材拆分 → 视觉 QA。
  见 [IDENTITY_SKILL_REFERENCE_ZH.md](IDENTITY_SKILL_REFERENCE_ZH.md)。
- 手绘解释图：先只读配图蓝图、人工确认后再本地生成资产，不作为发布前置。
  见 [IAN_HANDDRAWN_PPT_REFERENCE_ZH.md](IAN_HANDDRAWN_PPT_REFERENCE_ZH.md)。
- AstrBot 式产品上手表达（首屏、支持矩阵、安装路径、路线图），不是产品转向。
  见 [ASTRBOT_README_REFERENCE_ZH.md](ASTRBOT_README_REFERENCE_ZH.md)。
- Horizon 式雷达 / 日报流水线：只做只读候选排序、去重、来源索引和本地日报草稿；
  当前做 AI 早报优先用 `transparent-briefing` 配方。见
  [HORIZON_REFERENCE_ZH.md](HORIZON_REFERENCE_ZH.md)。

Agent / 插件集成协议（分层、状态层与动作层、发现机制）见
[AGENT_PROTOCOL_ZH.md](AGENT_PROTOCOL_ZH.md)。
