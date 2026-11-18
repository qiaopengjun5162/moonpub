# LoopForge 理念对照 MoonPub：借鉴什么、拒绝什么、已落实什么

2026-11-17 腾讯开源团队发表了《登上 GitHub Trending｜腾讯开源 LoopForge》，复盘一个**云端团队级研发平台**的五步演进：云端 7×24 工作区 → 共享会话 → 工程级 SOP → 定时自动化 → 工程级 RSI 自进化。文章的核心洞察是：**让 Agent 靠谱，不靠更长的提示词，而靠把「人的工程经验」转化为「系统的运行秩序」。**

MoonPub 是**本地优先的个人发布 CLI**，定位是副驾驶而非无人值守机器人。这篇对照说明：哪些理念已经具备、哪些可以借鉴落地、哪些是产品边界必须坚持不做的。

## 一句话结论

文章最值得借鉴的不是"云端"或"多角色群聊"，而是两件事：**客观门禁**和**把每次踩坑沉淀成系统默认遵守的秩序**。MoonPub 的门禁本来就扎实，缺的是"失败 → 诊断 → 读历史教训"这条连接；这次把这条连接补成了机器可读契约。云端工作区、共享会话、云端代发则是 MoonPub 明令禁止的方向，坚决不碰。

## 六概念对照表

| LoopForge 概念 | MoonPub 现状 | 判断 |
| --- | --- | --- |
| 云端 7×24 工作区 | 无；本地运行、凭证留本机 | **不做**：产品边界，见 `SPONSORSHIP_ZH.md` |
| 多人共享会话 + Main Bot | 无；单人 CLI + Obsidian 插件入口 | **不做**：多角色越权风险与本项目场景不符 |
| 五层标准 SOP | 有轻量版：`workflow-registry/v1` 定义工作流、安全起点、生产边界、证据状态 | **部分具备**：这次补上每条工作流的 `safe_commands` 白名单 |
| 客观门禁（编译/单测/规范硬卡点） | 已具备且严格：覆盖率地板 73%（只升不降）、MSRV 1.88、gitleaks、clippy -D warnings、契约测试、pre-commit | **已具备**：这是 MoonPub 最强项之一 |
| 定时拉单 / 无人值守自动化 | 无；`preflight` / `release-check` 是只读门禁，最终发布人工确认 | **不做**：与"不托管凭证、不云端代发"边界冲突；外部调度器可只读调用，另说 |
| 工程级 RSI（复盘→开处方→改写流程） | 有资产沉淀：`ENGINEERING_LESSONS`（中英）、`writing/` 六要素记录、错误码注册表 | **部分具备**：这次把错误码↔教训做成机器可读映射，是 RSI 的本地最小形态 |

## 这次真正落实的两件事

### 1. 错误码 ↔ 教训映射 + 诊断首查（文章"内环自愈 + 外环复盘"的本地版）

文章里的内环是"Agent 拿真实报错自动纠错循环"，外环是"复盘开处方改写 SOP"。MoonPub 不写自动改代码的循环（那会把判断权交给机器，违反协作护栏），但补上了循环所需要的**信息连接**：

- 每个 `error/v1` 错误码现在带 `first_check`（诊断首查提示），固定遵循项目的排障顺序：**评估方法 → 环境/配置 → 系统本身**。拿到失败先查评估方法有没有测错东西，再查环境和配置，最后才动业务代码。
- 有历史踩坑记录的错误码，额外带 `lesson` 指针（`doc` + `heading`），指向 `ENGINEERING_LESSONS_ZH.md` 里的精确章节标题。Agent 在改代码前先读这条记录。
- 两个入口都暴露：`moonpub --json help errors`（机器可读）和 `moonpub help errors`（人读）。
- 用桥接测试把 `lesson.heading` 和文档标题绑死：标题改名或引用写错，测试立刻变红——这正是文章"让教训成为系统默认秩序"的可验证形式。

**它解决什么**：过去错误码只有 code + next_step，Agent 常直接照 next_step 改代码，跳过"这个坑以前踩过吗"。现在失败会主动把人引向诊断顺序和历史教训。错误码 `invalid_theme_name`（拼错主题名却静默换主题）、`invalid_csv`（互动数据被当空值）这类反复出现的坑，有了机器可读的记忆入口。

### 2. 角色 ↔ 命令白名单（文章"五层标准 / 岗位能力"的轻量版）

文章用五层标准把"架构师/前端/后端/测试"岗位能力固化成资产。MoonPub 是单人 CLI，重型多角色编排是过度工程，只借最有用的一层：

- `workflow-registry/v1` 的每条工作流增加 `safe_commands`,列出"服务这条工作流时应该待在哪些命令里"。例如当前文章路径只该用 `check / preview / render / preflight`，微信发布路径只该用 `wechat-health / doctor / push / configure`。
- 这是**给调用方的引导，不是沙箱**：CLI 不强制拦截越界命令，也不假装做权限隔离。名字叫 `safe_commands` 而非 `allowed_commands` 就是为了不误导。
- 桥接测试确保每个白名单命令都在 `help` 注册表里，防止命令名漂移——历史上 #150/#155/#160/#177 四个 PR 都在补这类清单漂移，这次从设计上掐断。

**它解决什么**：44 条命令曾让用户和 Agent"不知道先跑哪个"。白名单把"一条工作流里该用哪些命令"变成结构化数据，上层入口（Obsidian 插件首页、未来的 Agent）可以直接读，不必自己猜。

## 借鉴的边界：为什么不做云端那三件

- **云端 7×24 工作区**：要求把工作区和执行环境托管到平台，与 MoonPub"文章、配置、凭证默认留用户机器"的第一原则冲突。用户信任一个本地工具，前提就是凭证不离开本机。
- **多人共享会话 / Main Bot**：MoonPub 的真实场景是作者一个人写作发布，引入多人写权限和越权风险只增加复杂度、不增加价值。
- **定时拉单 / 无人值守自动发**：`SPONSORSHIP_ZH.md` 的硬边界明确"不云端代发、最终发布人工确认"。`preflight` / `release-check` 保持只读，不做成自动修复的触发器。

文章把"人的经验变成系统秩序"这个方向是对的；MoonPub 选择把它落在**本地、只读、可审计、人来拍板**的形态上，而不是照搬它的云端形态。

## 验证

- 错误码教训引用桥接测试、白名单注册桥接测试均通过受控变异验证（改错标题/命令名即变红，恢复后文件 sha256 一致）。
- 相关测试：13 条帮助/错误码测试 + 5 条 workflow 测试全绿。
- 真实二进制：`moonpub --json help errors` 单条含 `first_check` 与 `lesson`；`moonpub --json workflow-registry` 每条含 `safe_commands`。
- 补充（2026-11-18）：最初这两个字段只出现在 `help errors` 清单里，真实失败的 `error/v1` 对象仍只有 `code + message + next_step`——等于 Agent 拿到失败还得先想起来去查表。现已在 `ErrorDetail` 上补齐：`first_check` 必填，`lesson` 有登记才出现（无登记省略而非 `null`），并有两条契约测试锁住「与注册表同源 + 无教训时省略」。

## 相关文档

- 商业模式与边界：[SPONSORSHIP_ZH.md](SPONSORSHIP_ZH.md) / [SPONSORSHIP.md](SPONSORSHIP.md)
- 协作纪律（诊断顺序、gate 可信度）：[AI_COLLABORATION_PROTOCOL_ZH.md](AI_COLLABORATION_PROTOCOL_ZH.md)
- 踩坑资产库：[ENGINEERING_LESSONS_ZH.md](ENGINEERING_LESSONS_ZH.md)
- 产品定位：[PRODUCT_WRAP_ZH.md](PRODUCT_WRAP_ZH.md)
