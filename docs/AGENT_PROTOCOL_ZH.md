# MoonPub Agent / App 入口协议

这份文档只回答一个问题：

**如果你不是直接给人用 CLI，而是要给 Obsidian 插件、本地 App、Agent 或自动化脚本接 MoonPub，应该先读哪些命令，按什么层次接？**

目标不是把所有命令重新列一遍，而是把当前已经稳定的高层入口收口成一套更清晰的协议。

## 一句话原则

优先级从高到低是：

1. `workspace --json`
2. `status --json`
3. `check <article.md> --json`
4. 具体动作命令：`preview` / `push` / `draft-from-inbox` / `intake feishu ... --draft`
5. `capabilities --json`

也就是说：

- 先判断整个工作区该走哪条入口
- 再判断当前池子里有什么
- 再判断某一篇文章当前缺什么
- 最后再执行具体动作

## 第 1 层：工作区入口

### `moonpub workspace --json`

这是当前最高层入口。

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

## 第 2 层：文章池状态

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

## 第 3 层：单篇文章状态

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

## 第 4 层：动作命令

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

## 第 5 层：能力元数据

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

1. 先接 `workspace --json`
2. 再接 `check --json`
3. 再接 `preview --json`
4. 最后接 `push --json`

如果你在做飞书链路：

1. 先接 `workspace --json`
2. 再接 `intake feishu ... --draft --json`
3. 然后回到 `check --json`
4. 确认后再接 `push --json`

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
