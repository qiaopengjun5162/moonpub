# MoonPub 输入模型

这份文档只回答一个问题：

**飞书、照片、语音、读书摘录这些输入源，进入 MoonPub 时到底应该长什么样？**

目标不是立刻把所有输入源都做完，而是先把当前已经稳定的飞书链路抽象出来，避免后面每增加一个输入源就重新发明一套字段和流程。

## 为什么现在要做这件事

当前飞书链路已经稳定到可以抽象：

- 能导入原始素材到 Inbox
- 能保留原始来源信息
- 能继续生成草稿
- 能输出结构化 `action` / `next_command`

但如果不现在收口输入模型，后面做：

- 照片整理
- 语音笔记
- 读书摘录

就很容易变成：

- 每条输入链路都各自定义 frontmatter
- 每条链路各自返回不同 JSON 字段
- 插件 / Agent 继续为每种输入源单独写逻辑

这会直接破坏我们已经在做的“统一入口协议”。

## 一句话模型

MoonPub 的输入模型应该分成两层：

1. **原始素材层**
2. **可编辑草稿层**

也就是说：

- 输入源先进入 `Inbox/`
- `Inbox` 里的对象保留真实来源和原始内容
- 后续再由 `draft-from-inbox` 或等价流程变成文章草稿

## 当前推荐的统一结构

### A. 原始素材层：Inbox Item

每一个输入源最终都应该先落成一个 Inbox 文件。

它至少应包含：

- `source`
- `status`
- `created`
- `type`

当前飞书已经这样做了：

```yaml
---
source: feishu-minutes
status: inbox
created: 2026-07-02
type: voice-note
minute_token: "obcn123"
source_url: "https://..."
original_file: "/path/to/transcript.txt"
---
```

这说明 Inbox 层应该承担的是：

- 描述来源是谁
- 描述原始素材属于什么类型
- 记录上游可追溯标识
- 保留原始文本或原始信息

### B. 可编辑草稿层：Draft Item

草稿层不应该继续关心“这是飞书还是照片”，而应该关心：

- 这是不是一篇准备编辑和发布的文章
- 当前草稿处于什么阶段
- 对应预览和推送的下一步是什么

草稿层更适合承载：

- 文章标题
- frontmatter 中的发布相关字段
- 文章正文
- 预览 / 渲染 / 推送后续动作

## 统一字段建议

### Inbox 层最小必需字段

| 字段 | 用途 | 说明 |
|------|------|------|
| `source` | 来源标识 | 如 `feishu-minutes`、未来 `photos`、`voice-memo` |
| `status` | 当前阶段 | 当前建议固定为 `inbox` |
| `created` | 进入系统时间 | 不是原素材拍摄时间，而是进入 MoonPub 的时间 |
| `type` | 素材类型 | 如 `voice-note`、`photo-note`、`reading-note` |

### Inbox 层可选来源字段

| 字段 | 用途 |
|------|------|
| `source_url` | 上游链接 |
| `original_file` | 本地原始文件路径 |
| `external_id` | 通用来源主键 |
| `captured_at` | 原素材发生时间 |
| `source_title` | 来源系统里的标题 |

### 来源专属字段

来源专属字段允许存在，但应该尽量收口：

- 飞书当前是 `minute_token`
- 后续照片可以有 `photo_group_id`
- 语音可以有 `recording_id`

建议原则：

- **通用系统只识别 `external_id`**
- **来源专属字段保留给具体输入源使用**

也就是说，未来更理想的写法是：

```yaml
external_id: "obcn123"
minute_token: "obcn123"
```

这样：

- 上层不用理解每个平台私有 ID
- 输入源实现仍然可以保留来源特性

## 当前飞书应该怎样对齐这个模型

飞书当前其实已经很接近统一模型了：

- `source: feishu-minutes`
- `status: inbox`
- `type: voice-note`
- `minute_token`
- `source_url`
- `original_file`

现在这一步已经不只是“未来应考虑”了，而是已经开始落地：

1. 这是 MoonPub 的第一条正式输入模型
2. 后续新输入源应尽量对齐这套字段层次
3. 飞书当前会同时写入通用 `external_id` 和来源专属 `minute_token`
4. 现有飞书复用逻辑已经开始优先按 `external_id` 对齐，同时兼容旧文件里只有 `minute_token` 的情况

## 未来输入源怎样映射

### 照片

适合映射为：

- `source: photos`
- `type: photo-note`
- `external_id: <同一天或同一组照片的稳定 ID>`
- `captured_at`
- `original_file` 或照片目录引用

素材正文可以是：

- 照片列表
- EXIF / 时间 / 地点摘要
- AI / 工具提取出的事实信息

### 语音笔记

适合映射为：

- `source: voice-memo`
- `type: voice-note`
- `external_id`
- `captured_at`
- `original_file`

素材正文可以是：

- 原始转写
- 说话片段
- 时间戳摘要

### 读书摘录

适合映射为：

- `source: wechat-read` 或 `reading-notes`
- `type: reading-note`
- `external_id`
- `source_title`
- `source_url`

素材正文可以是：

- 摘录原文
- 批注
- 章节信息

### 微信公众号归档

适合映射为：

- `source: wechat-mp-article`
- `type: archived-article`
- `external_id`
- `source_title`
- `source_url`
- `source_author`

素材正文可以是：

- 原文标题、作者、发布时间
- 正文 Markdown / 纯文本
- 原文 URL
- 可选的原始 HTML 或结构化元数据引用

这条线更像“归档输入源”，不是发布动作。安全边界见 [WECHAT_ARCHIVE_WORKFLOW_ZH.md](WECHAT_ARCHIVE_WORKFLOW_ZH.md)：默认只处理用户显式提供的公开 URL，不自动抓历史列表，不保存或提交 cookie、pass_ticket、uin、token、二维码登录信息。

## 与 Agent / 插件协议的关系

输入模型并不替代 `workspace / status / check` 这层协议。

关系应该是：

1. 输入源负责产出 Inbox Item
2. `draft-from-inbox` 负责把 Inbox Item 变成 Draft
3. `workspace / status / check` 负责让入口层知道当前状态
4. `preview / push` 负责继续推进发布动作

也就是说：

- 输入模型解决“素材长什么样”
- 入口协议解决“现在该做什么”

## 当前推荐落地顺序

### 第一步

先把这个输入模型文档定下来。

### 第二步

让飞书链路成为第一个明确对齐这套模型的输入源。

### 第三步

后续新增照片 / 语音输入源时：

- 优先复用 `Inbox` 层字段
- 尽量补 `external_id`
- 不再临时发明 frontmatter

## 当前结论

当前最合理的做法不是马上把所有输入源都代码实现，而是先明确：

- **MoonPub 的输入模型以 Inbox Item 为中心**
- **飞书是第一条正式输入模型**
- **未来照片 / 语音 / 摘录都应尽量映射到同一层结构**

如果继续推进，这份文档后面最值得补的是：

- `external_id` 的正式引入策略
- 照片输入源的第一版字段草案
- 哪些字段属于原始素材，哪些字段进入文章草稿 frontmatter
