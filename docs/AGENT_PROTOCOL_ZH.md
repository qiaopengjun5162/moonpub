# MoonPub Agent / App 入口协议

这份文档只回答一个问题：

**如果你不是直接给人用 CLI，而是要给 Obsidian 插件、本地 App、Agent 或自动化脚本接 MoonPub，应该先读哪些命令，按什么层次接？**

目标不是把所有命令重新列一遍，而是把当前已经稳定的高层入口收口成一套更清晰的协议。

## 一句话原则

优先级从高到低是：

1. `doctor --json`
2. `workflow-registry --json`
3. `evidence-status --json`
4. `release-check --json`
4. `workspace --json`
5. `status --json`
6. `check <article.md> --json`
7. `preflight <article.md> --json`
8. 具体动作命令：`preview` / `push` / `draft-from-inbox` / `intake feishu ... --draft`
9. `capabilities --json`

也就是说：

- 先检查本地是否能开始，尤其是插件首页或首次打开
- 再读取 MoonPub 内置的正式工作流契约，避免从 README 或终端文本里猜路径
- 再读取 release / 首次体验证据缺口，避免把“代码已实现”误写成“用户已验证”
- 再判断整个工作区该走哪条入口
- 再判断当前池子里有什么
- 再判断某一篇文章当前缺什么
- 再做发布前本地只读质量门
- 最后再执行具体动作

## 第 1 层：本地诊断入口

### `moonpub doctor --json`

这是首次使用和插件首页的本地诊断入口。

它回答的是：

- 当前 MoonPub CLI 版本是什么
- 当前 Articles 根目录是什么
- 是否能找到本地配置
- 本地首次使用还有哪些 warning
- 下一步应该先初始化、创建文章，还是进入工作区首页

适合用途：

- Obsidian 插件首页顶部的“当前是否可开始”
- 本地 App 首次启动检查
- Agent 在执行动作前确认本地环境

当前关键字段：

- `moonpub_version`
- `articles_root`
- `config_status`
- `capabilities_summary`
- `warnings`
- `next_command`
- `next_step`

约束：

- 不触发微信 API
- 不打开或控制 Chrome
- 不读取、打印或返回真实 secret

## 第 2 层：工作区入口

### `moonpub workspace --json`

这是工作区级入口。

它回答的是：

- 当前工作区是什么类型
- 当前更适合走哪条入口
- 当前文章池里有多少 drafts / ready / published
- 内置 target 有哪些风险边界
- 现在最推荐先执行哪条命令

适合用途：

- Obsidian 插件首页
- 本地 App dashboard
- Agent 第一次接手一个 vault
- 自动化脚本的“先看全局再决定”

当前关键字段：

- `workspace_kind`
- `entry_path`
- `entry_path_label`
- `total_articles`
- `stage_counts`
- `stages`
- `capabilities`
- `next_command`
- `next_step`

推荐做法：

- 把 `entry_path_label` 展示给用户看
- 把 `next_command` 作为最小下一步动作
- 把 `capabilities` 里的 `requires_network` / `requires_browser` 作为风险提示

## 第 3 层：正式工作流目录

### `moonpub workflow-registry --json`

这是 MoonPub 内置工作流契约目录。

它回答的是：

- 当前正式支持哪些主路径
- 每条路径属于哪个 package / owner
- 哪条命令适合作为安全起点
- 哪条命令会进入下一阶段
- 这条路径是否需要网络或浏览器
- 当前证据状态和应阅读的文档入口是什么

适合用途：

- Obsidian 插件首页的路径选择区
- 本地 App 的 workflow picker
- Agent 接手任务前的能力发现
- 避免从 README 文本反解析命令

当前关键字段：

- `id`
- `package`
- `status`
- `owner`
- `entry_command`
- `safe_start_command`
- `next_command`
- `requires_network`
- `requires_browser`
- `production_boundary`
- `evidence_status`
- `docs`

约束：

- 不触发微信 API
- 不打开或控制 Chrome
- 不读取、打印或返回真实 secret
- 当前是内置静态契约，不从外部 registry 下载内容

## 第 4 层：证据状态

### `moonpub evidence-status --json`

这是 release gate 和首次体验证据的本地只读状态接口。

它回答的是：

- v0.4.2 所需证据文件是否已经落到固定目录
- 当前需要多少个文件
- 已归档多少个文件
- 还缺多少个文件和哪些路径
- 下一步应该补证据，还是进入人工脱敏复查

适合用途：

- Obsidian 插件首页的 release 证据提示区
- 本地 App 的 release gate 面板
- Agent closeout 时确认“代码完成”和“真实用户证据”没有混在一起

如果是 release 脚本或 CI 门禁，使用 `moonpub evidence-status --strict`；默认模式只报告状态，`--strict` 在缺少必需证据文件时非零退出。两种模式都不会打开图片、读取图片内容或替代人工脱敏审查。

当前关键字段：

- `base_dir`
- `passed`
- `required_count`
- `present_count`
- `missing_count`
- `missing_paths`

## 第 5 层：Release Gate 状态

### `moonpub release-check --json`

这是 v0.4.2 发布前的本地只读总门禁接口。

它回答的是：

- release gate 文档是否存在
- 本地 release smoke 和 CI / Windows smoke 是否已经在文档中记录为完成
- 真实微信回归、证据文件、文档一致性和隐私审查是否仍未完成
- 下一步应该先补哪一个 gate

适合用途：

- release 脚本或 CI gate
- Agent closeout 时判断“代码已完成”和“v0.4.2 可发布”之间还差什么
- 插件 / App 的发布前状态面板

`release-check` 默认只报告状态；`release-check --strict` 在任一 gate 未完成时非零退出。它不触发微信 API、不打开浏览器、不扫描图片内容，也不替代人工脱敏审查。

当前关键字段：

- `release_version`
- `repo_root`
- `passed`
- `checks`
- `next_command`
- `next_step`

约束：

- 不打开图片
- 不读取图片内容
- 不触发微信 API
- 不替代人工脱敏审查

## 第 6 层：文章池状态

### `moonpub status --json`

这是“文章池层”的状态接口。

它回答的是：

- drafts / ready / published 三个阶段分别有哪些文章
- 每篇文章最近一次状态记录是什么
- 如果只看当前池子，推荐先处理哪一篇

适合用途：

- 列表页
- 阶段筛选页
- 文章池统计视图

如果你已经先调用了 `workspace --json`，通常只有在需要展示更细的文件列表时，才继续调用 `status --json`。

## 第 7 层：单篇文章状态

### `moonpub check <article.md> --json`

这是“单篇文章层”的状态接口。

它回答的是：

- 当前文章有没有 markdown
- 有没有 html
- 有没有 draft.json
- 有没有 media_id
- 当前是否已经可进入下一步
- 下一步最推荐执行什么命令

适合用途：

- 当前文件详情面板
- 单篇文章状态检查按钮
- 发布前自检

### `moonpub preflight <article.md> --json`

这是“发布前本地质量门”。

它回答的是：

- 当前文章包 Markdown / HTML / draft.json 是否齐全
- 渲染后的 HTML 是否通过公众号排版审计
- `.media_id` 是否已经存在
- 当前是否可以继续触达微信 API
- 下一步应该 render、修 HTML、push，还是先检查浏览器登录态

适合用途：

- 当前文章工作台的“发布前检查”
- Agent 在 `push` 前的强制只读检查
- CI 或脚本里的本地产物质量门

约束：

- 不触发微信 API
- 不打开或控制 Chrome
- 缺 `.media_id` 只算 warning，因为它表示还没推到微信草稿，不代表本地产物失败

## 第 8 层：动作命令

这些命令不是用来“先判断”，而是用来“实际推进流程”的。

### `moonpub preview <article.md> --json`

适合：

- 本地 HTML 预览
- 打开或输出本地预览路径

### `moonpub push <article.md> --json`

适合：

- 把文章推进到微信草稿
- 返回 `media_id`、`stage` 和下一步建议

### `moonpub draft-from-inbox <inbox.md> --json`

适合：

- 从已有 Inbox 文本生成可编辑草稿

### `moonpub intake feishu ... --draft --json`

适合：

- 从飞书秒记导入并继续生成草稿

这两个“生成草稿”命令都支持：

- `action: created | updated`
- 可选 `html_path`
- `next_command`

显式加 `--push` 时，还会继续返回：

- `pushed`
- `media_id`
- `stage`
- `next_step`

## 第 9 层：能力元数据

### `moonpub capabilities --json`

这个命令不告诉你“当前工作区怎么样”，而是告诉你：

- 内置 target 有哪些
- 每个 target 是否联网
- 是否可能打开浏览器
- 依赖哪些 env / config
- 风险和后续人工步骤是什么

适合用途：

- 发布前提示
- App 按钮权限说明
- 执行动作前的风险弹窗

## 当前推荐接入顺序

如果你在做一个新的用户入口，推荐按这个顺序接：

1. 先接 `doctor --json`
2. 再接 `workflow-registry --json`
3. 再接 `evidence-status --json`
4. 再接 `workspace --json`
5. 再接 `check --json`
6. 再接 `preflight --json`
7. 再接 `preview --json`
8. 最后接 `push --json`

如果你在做飞书链路：

1. 先接 `doctor --json`
2. 再接 `workflow-registry --json`
3. 再接 `evidence-status --json`
4. 再接 `workspace --json`
5. 再接 `intake feishu ... --draft --json`
6. 然后回到 `check --json`
7. 确认后再接 `preflight --json`
8. 最后接 `push --json`

## 不推荐的接法

当前不推荐：

- 直接从纯文本输出里反解析状态
- 插件自己拼装“当前工作区入口语义”
- 跳过 `workspace` 直接假设用户要发文
- 用 `capabilities --json` 代替工作区状态判断

因为这几种方式都会让“入口层”重新分裂，最后又回到“功能有了，但用户不知道怎么用”。

## 现在这套协议意味着什么

这套协议说明 MoonPub 现在已经不只是“命令集合”了。

它已经开始具备：

- 本地发布内核
- 输入工作流
- 用户入口协议

接下来无论是继续做 Obsidian 插件、飞书路线，还是未来的本地 App / Agent，最重要的都不是再造一套逻辑，而是继续复用并稳定这层入口协议。
