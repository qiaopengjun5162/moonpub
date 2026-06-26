# MoonPub

Rust CLI for publishing Obsidian markdown to WeChat 公众号.

## Build & Run

```
cargo build --release
./target/release/moonpub --help
```

## Common Commands

```bash
# Full publish flow (recommended)
moonpub ship <article.md> [--style dark|literary|warm|...]

# Step by step
moonpub render <article.md>
moonpub push <article.md>

# Cover only
moonpub cover <article.md> [--style dark] [--screenshot]

# Browser automation only
moonpub configure [--headed]
```

## Architecture

- `src/main.rs` — entry point
- `src/cli.rs` — CLI parsing (`Options`, `Command`)
- `src/config.rs` — `Config` and hand-written TOML parser
- `src/error.rs` — `AppError` and help text
- `src/app.rs` — command routing and use-case orchestration
- `src/push.rs` — WeChat draft push/update/list/delete operations
- `src/article.rs` — frontmatter parsing and article path helpers
- `src/render.rs` — Markdown → WeChat HTML, draft JSON builder
- `src/export.rs` — Zola blog export
- `src/status.rs` — article stage/status tracking
- `src/preview.rs` — open rendered HTML in system browser
- `src/system.rs` — Chrome/Chromium discovery
- `src/json_util.rs` — small JSON helpers
- `src/wechat.rs` — WeChat API client
- `src/publish.rs` — browser automation (CDP via chromiumoxide)
- `src/theme.rs` — render themes (default, warm, dark, geek)
- `src/cover.rs` — cover HTML templates
- `src/footer.rs` — article footer template
- `src/humanize.rs` — AI-pattern removal
- `src/illustrate.rs` — fenced block rendering
- `src/fetch.rs` — fetch WeChat article content via Chrome
- `src/radar.rs` — trend sample store and analysis

## Config (`moonpub.toml` in articles root)

```toml
[articles]
root = "/path/to/obsidian"

[wechat]
appid = "wxa..."
author = "作者名"
theme = "geek"          # default | warm | dark | geek
thumb_media_id = ""     # ship 命令会自动上传封面刷新此值，无需手填
qrcode = "Context/assets/qrcode.jpg"

[blog]
kind = "zola"
root = "/path/to/blog"
```

## Themes

| Name    | Background | Accent   | Use case         |
|---------|-----------|----------|------------------|
| default | #fff      | #2c2c2c  | clean minimal    |
| warm    | #fdf8f4   | #e67e22  | literary/reading |
| dark    | #1a1a1a   | #64b5f6  | dark mode        |
| geek    | #0d1117   | #3fb950  | terminal/tech    |

## ship 流程

`ship` 命令做完整一键发布：
1. 生成封面 HTML（literary 默认风格）
2. Headless Chrome 截图 → 上传微信永久素材 → 拿 thumb_media_id
3. 渲染文章 HTML（含封面注入顶部）
4. 调 WeChat draft/add API 推送草稿
5. 浏览器自动化设置原创声明、留言、预览

## Browser Automation 已知问题

WeChat 编辑器是 live web app，UI 随时会改。已知稳定/不稳定状态（截至 2026-06-16）：

| 步骤     | 状态 | 说明 |
|--------|------|------|
| 原创声明 | ✅   | 正常 |
| 留言     | ✅   | 正常 |
| 预览     | ✅   | 正常 |
| 赞赏     | ⚠    | toggle 点击后 state 不变，微信端控制，软失败 |
| 创作来源 | ✅   | 通过 radio value="4" 选择"个人观点，仅供参考"，label + js_claim_source_selected 验证 |
| 合集     | ⏸    | 已禁用，不执行 |

⚠ 软失败不影响草稿发布，只是该项未配置。

## TOML 解析

`Config::from_toml` 手写（无 toml crate），按 section header 区分同名 key：
```toml
[articles]
root = "/obsidian"   # → cfg.articles_root

[blog]
root = "/blog"       # → cfg.blog_root
```

## AI 功能

MoonPub 通过可配置 AI provider 支持 AI 辅助写作，默认仍是 DeepSeek，也支持 OpenAI：

```bash
moonpub write "一个想法"       # 从零生成文章
moonpub expand notes.md         # 读书笔记展开成文章
moonpub polish draft.md         # AI 润色 + 去 AI 味
moonpub ship draft.md --ai      # 润色后发布
```

可通过 `moonpub.toml` 的 `[ai]` section 配置 `provider` / `model` / `api_key`。若不在配置中写 key，则从环境变量读取；moonpub 启动时会自动加载 `.env` 和 `~/.moonpub.env`。

## 环境变量

```
WECHAT_APPID       覆盖 config 中的 appid
WECHAT_SECRET      必填，不进 config 文件
DEEPSEEK_API_KEY   DeepSeek provider 的 AI 功能需要（可选）
OPENAI_API_KEY     OpenAI provider 的 AI 功能需要（可选）
AI_API_KEY         通用 AI key fallback（本地实验用）
MOONPUB_VAULT      覆盖 articles root
```

moonpub 启动时自动加载 `.env` 和 `~/.moonpub.env`（不会覆盖已有环境变量）。

## 历史问题记录

### 2026-06-14: qrcode 图片不显示
**问题**: 渲染后的 HTML 里 qrcode src 是相对路径，upload_local_images 以 article_dir 为基础解析，但 qrcode 配置路径是相对 articles root 的，导致文件找不到、不上传。
**根因**: 路径解析基准不一致（article_dir vs articles root）。
**修复**: `render_article` 在传给 footer 前把 qrcode 路径 join articles root 转为绝对路径，upload_local_images 看到绝对路径直接用。
**经验**: config 里的资产路径（qrcode、cover）一律相对 articles root 写，代码里统一 join articles root 解析。

### 2026-06-14: config 未自动发现，render/push 不读 author/theme
**问题**: 不传 `--config` 时走 `Config::default()`，author/theme 全是默认值。
**根因**: 没有 articles root 自动发现逻辑。
**修复**: `Options::parse` 里，若无 `--config`，自动检测 articles root 下是否有 `moonpub.toml`，有则加载。
**经验**: CLI 工具应优先从项目根（articles root）自动发现配置，减少用户显式传参负担。

### 2026-06-14: author 字段被书籍导入元数据污染
**问题**: frontmatter 的 `author:` 字段在微读导入时是书籍作者（詹姆斯·希尔顿），直接用作微信文章作者。
**根因**: `author` 是通用字段，微读/导入工具会写入原作者，与账号作者语义冲突。
**修复**: 新增 `wechat_author` 专用 frontmatter 字段，只有显式声明才覆盖 config 全局作者。
**经验**: 凡是和外部工具共享 frontmatter 的字段，需要用带命名空间的专用 key（`wechat_*`）避免语义冲突。

### 2026-06-14: Obsidian callout [!abstract] 被渲染成超长 blockquote
**问题**: 微读导入的笔记头部有 `> [!abstract]` callout，渲染成 559 字 blockquote，违反微信 300 字限制。
**根因**: 渲染器把所有 `>` 开头的行都当 blockquote，没有识别 Obsidian callout 语法。
**修复**: `render_markdown_segment` 检测 blockquote 首行是否以 `[!` 开头，是则跳过整个 callout 块不渲染。
**经验**: Obsidian callout (`> [!type] title`) 是元数据容器，不适合直接发布，应在渲染层过滤。

### 2026-06-14: 浏览器自动化 headless 下不执行
**问题**: ship 调用 auto_configure 后，微信草稿的原创声明/留言/预览等步骤均未实际生效。
**根因**: setup_editor 点击编辑按钮 (target="_blank" 链接) 后等待新 tab，但 headless mode 下 chromiumoxide 无法可靠检测到 target="_blank" 触发的新 Page。步骤在错误的 page 上执行，全部静默失败。
**修复**: 改为用 JS 读 `btns[1].href` 拿到编辑 URL，直接 `page.goto(url)` 导航，完全绕过新 tab 问题。
**经验**: headless 下 target="_blank" 新 tab 检测不可靠。优先拿 href 直接导航，而非等待 pages() 变化。

### 2026-06-14: 文章标题格式（读书笔记）
**问题**: weread 导入的笔记 frontmatter 含 `author:` (书籍作者)，moonpub 直接用 `title:` (书名) 作微信标题，显示为"消失的地平线"而非"读《消失的地平线》笔记"。
**修复**: Frontmatter 新增 `author` 字段；`wechat_title()` 函数检测到 `author` 字段存在时自动格式化为 "读《{title}》笔记"；显式 `wechat_title:` 字段可覆盖自动格式化。
**经验**: 自动格式化依赖隐含字段检测（author 存在 = 读书笔记）比要求用户手填更省事，但加 `wechat_title` 显式覆盖作为逃生门。

### 2026-06-14: geek 主题纯黑背景不可读
**问题**: geek 主题 section_bg 为 #0d1117（纯黑），WeChat 移动端渲染整篇文章背景极深，排版丑陋。
**根因**: 微信移动端对深色背景渲染一致性差，纯黑 section 背景和正文混排体验很差。
**修复**: 改为浅灰背景 #f6f8fa（GitHub light），绿色 #2da44e 强调色，代码块保留暗色 #0d1117 + 绿色代码字 #7ee787，保留 geek 感但可读性大幅提升。
**经验**: WeChat 文章正文背景应用浅色（白或浅灰），深色主题感靠代码块和强调色体现，不要靠整体背景。

### 2026-06-14: `<blockquote>` 样式被新版微信编辑器剥离
**问题**: 新版微信编辑器（mpeditor=1）会剥离 `<blockquote>` 的 inline style，导致样式丢失（doocs/md issue #447）。
**修复**: render_blockquote 改用 `<section>` 标签代替 `<blockquote>`，样式通过 inline style 保留。
**经验**: WeChat 对 `<blockquote>/<ul>/<li>` 有特殊处理，API push 的文章应优先用 `<section>/<p>/<table>` 代替。

### 2026-06-14: 排版参考资料
**来源**: `docs/REFERENCES.md` 有完整链接，关键参考：
- doocs/md: `letter-spacing: 0.1em; word-spacing: 0.05em; text-align: justify` — 中文 justify 效果差异最大
- wechat-publish-template: block 系统 (cover/intro/heading/callout/steps/summary/cta)，font-size 15px，line-height 1.85
- mdnice: h2/h3 border-left 装饰 + background tint，blockquote box-shadow


**问题**: WeChat 永久素材 media_id 会失效（删除后报 40007），导致 `ship` 推送失败。
**根因**: ship 只读 config 里的 thumb_media_id，没有自动刷新。
**修复**: ship 命令现在每次截图封面 PNG → 上传微信 → 用新 media_id，config 里的作为最后兜底。

### 2026-06-14: 创作来源点"未添加"弹出原创声明弹窗
**问题**: step_chuangzuo 点击"未添加"后弹出的是原创声明弹窗，不是创作来源选择框。
**根因**: 微信编辑器 UI 更新，把创作来源入口合并进了原创声明弹窗。
**修复**: 检测到弹窗内有"声明类型"/"文字原创"/"无需声明"文字时，按 Escape 关闭，记 ⚠ 跳过。
**经验**: 微信编辑器是 live web app，UI 随时会改。browser automation 步骤应做软失败（⚠）而非硬失败，核心流程（API push）不受影响。

### 2026-06-16: 创作来源点击后未选择选项
**问题**: `configure` 运行时 创作来源 步骤只点击了"未添加"，没有实际选择来源，随后因检测到 原创声明 弹窗文字而跳过。
**根因**: 弹窗文字检测（"声明类型"/"文字原创"/"无需声明"）过于宽泛，会把页面上已可见的 原创声明 设置行误判为弹窗；跳过逻辑导致选择步骤从未执行。后续排查 Git 历史发现，此前跑通的实现是直接点击最后一个"未添加"，选择"个人观点，仅供参考"，再点"确认"。
**修复**: 移除弹窗类型检测；改为定位到包含"创作来源"文本的行，优先点击 `.js_claim_source_desc` / `.allow_click_opr` 可点击包装器；选项选择用 `indexOf` 包含匹配（兼容图标/空白），优先"个人观点，仅供参考"和"个人观点"；点击"确认"后回读 `.js_claim_source_selected` 或行内文本验证设置是否生效。找不到可选项时再关闭弹窗并软失败跳过。同时删除 `cdp` 中不再使用的 `has_visible_text` 辅助函数。
**经验**: 不要通过页面上的静态文字判断弹窗类型，容易误判。参考历史跑通的实现，优先点击微信绑定的专用容器类，并用包含匹配选择固定选项。Browser automation 步骤应做软失败（⚠）而非硬失败。

### 2026-06-16: 创作来源仍不稳定，文本匹配不可靠
**问题**: 上次修复用了文本包含匹配选择选项，但 WeChat DOM 里文本被图标、空白分割，`indexOf` 匹配不稳定。
**根因**: 选项文本被 HTML 结构分割成多段，`textContent` 拼接后的字符串不可预期。
**修复**: 改为 `input[type="radio"][value="4"]` 精确定位单选按钮，不再依赖文本匹配。同时打开 picker 改为直接 `.js_claim_source_desc` wrapper，验证改为读 `.js_claim_source_selected` span。
**经验**: WeChat 编辑器的 label 文本不可靠，优先用 DOM 结构标记（class、input value）而非文本内容定位元素。headed 和 headless 均验证通过。

### 2026-06-13: 赞赏 toggle offsetParent 不可见
**问题**: cdp_click_css 用 `offsetParent !== null` 判断可见性，赞赏 toggle 在 DOM 中但不可见，点击无效。
**修复**: 改用 JS `.click()` 直接点击绕过可见性检查。当前 toggle 点击后 state 仍为"不开启"，属微信端控制，软失败处理。

### 2026-06-13: TOML root key 顺序 bug
**问题**: vault.root 和 blog.root 都叫 `root`，没有 section 跟踪时后者会覆盖前者。
**修复**: `Config::from_toml` 加 `let mut section = ""` 跟踪当前 section header。

### 2026-06-15: lib.rs 过大导致修改不稳定
**问题**: 所有逻辑和测试都堆在 `src/lib.rs`，每次优化/修复都容易引入回归；反复修改后状态不可控。
**根因**: monolithic 文件把 CLI 解析、配置、渲染、API 调用、测试全部耦合在一起。
**修复**: 按职责拆分为 `cli.rs` / `config.rs` / `error.rs` / `article.rs` / `render.rs` / `export.rs` / `status.rs` / `preview.rs` / `system.rs` / `push.rs`，测试分散到各模块。
**经验**: Rust crate 根应保持轻量，只做模块声明；公共测试 helper 用 `#[cfg(test)] pub(crate) mod test_helpers` 集中管理。

### 2026-06-15: wechat.rs 与 app.rs 循环依赖
**问题**: 模块拆分后发现 `wechat.rs` → `app.rs` → `wechat.rs` 循环依赖，编译和架构方向混乱。
**根因**: `extract_ip_from_message` 被放在 `app.rs`，而 `wechat.rs` 需要用它解析错误消息中的 IP。
**修复**: 把 `extract_ip_from_message` 移到 `error.rs`，`wechat.rs` 改从 `error` 模块引入。
**经验**: 纯工具函数不要放在业务编排层；遇到底层模块需要引用上层模块时，优先把工具函数下沉到更底层。

### 2026-06-17: footer 硬编码，其他用户无法使用
**问题**: `footer.rs` 里写死了"寻月阁"品牌文字、关注图片、群二维码，其他用户用 moonpub 会在文章末尾看到别人的品牌信息。
**修复**: Footer 改为 `[footer]` TOML section 配置。无配置或 `enabled = false` 时不渲染。字段全部可选：title、description、rules、qrcode、qrcode_note、follow_image、follow_text、divider。`\n` 转义支持多行文本。
**经验**: 跟品牌/个人信息相关的内容必须可配置、默认关闭，否则工具只能自己用。

### 2026-06-17: 交互式 init 向导
**功能**: `moonpub init` 从直接生成模板改为交互式问答向导。逐题询问：文章目录、AppID、AppSecret、作者名、主题风格（1-4选择）、是否需要 footer 及各项配置、是否有博客。AppSecret 写入 `.env` 不进 toml。非 TTY 环境自动 fallback 到模板生成。
**经验**: CLI 工具的初始化应该交互式引导，不能期望新用户手改 TOML。

### 2026-06-17: 版本统一到 0.4.0
`Cargo.toml` 版本从 0.1.0 同步到 0.4.0，与 GitHub Release 一致。

### 2026-06-17: slides.html 中文化
演示幻灯片从英文全改为中文，更新内容匹配当前项目状态（10 种封面、12 种模板、AI 可选、交互式 init 等）。
