# MoonPub 执行计划

这份文档不是愿景，也不是零散 TODO。

它只回答 3 个问题：

1. **现在这项目到底先做什么**
2. **做到什么算当前阶段完成**
3. **接下来按什么顺序推进**

目标是把“整体评估”继续收口成一份可执行计划，避免项目再次回到“持续优化，但主线越来越散”的状态。

如果你现在还在判断“MoonPub 现在到底是什么产品，而不是一堆能力和命令”，先看 [PRODUCT_WRAP_ZH.md](PRODUCT_WRAP_ZH.md)，再回来看这份执行计划会更容易对齐里程碑。

## 总目标

MoonPub 当前阶段的总目标不是“能力尽量多”，而是：

**先把 MoonPub 做成一个用户看得懂、跑得通、可被插件 / Agent 复用的本地发布内核。**

更具体地说：

- 用户第一次看到项目时，知道自己该走哪条路径
- 至少有三条主路径已经进入正式收口状态：
  - 普通 Markdown 文章路径
  - 飞书秒记路径
  - 照片素材路径
- CLI、Obsidian 插件、未来 Agent 不再各自发明一套流程，而是复用同一层入口协议

## 当前产品判断

### 1. 飞书路线

当前结论已经明确：

- **现在不拆成新项目**
- **先作为 MoonPub 内部正式模块推进**

理由：

- 当前最大价值依赖 MoonPub 主链路
- 还没有形成独立产品边界
- 当前瓶颈是“用户不知道怎么用”，不是“仓库拆得不够开”

### 2. AI Agent 方向

当前更准确的说法不是“另做一个 AI Agent 项目”，而是：

- **把 MoonPub 包装成 Agent-ready 的本地发布内核**

也就是：

- 输入源：飞书、Obsidian、未来照片/语音/摘录
- 状态层：workspace / status / check
- 动作层：preview / push / draft-from-inbox / intake feishu
- 风险层：capabilities

## 三层结构

### MoonPub Core

负责：

- 渲染
- 封面
- 推送
- 微信后台自动化
- 导出
- 状态追踪

### Input Workflows

负责：

- 飞书秒记
- 照片整理
- 未来语音笔记
- 未来读书摘录

### User Surfaces

负责：

- CLI
- Obsidian 插件
- 未来本地 App
- 未来 Agent 包装

## 当前阶段里程碑

### M1：用户看得懂入口

目标：

- 用户第一次进入项目时，知道自己属于哪条路径

完成标准：

- [x] README 第一屏明确用户入口
- [x] 推荐工作流文档存在
- [x] 飞书路线与普通文章路径并列出现
- [x] Obsidian 插件被当成正式入口之一说明

当前证据：

- `docs/RECOMMENDED_WORKFLOWS_ZH.md`
- `README.md`
- `README_zh.md`
- `docs/PRODUCT_EVALUATION_ZH.md`

### M2：用户跑得通主路径

目标：

- 用户不只看得懂，还至少能沿一条主路径真正跑通

完成标准：

- [x] 普通文章路径清楚可见
- [x] 飞书秒记路径文档已收口
- [x] 飞书秒记真实闭环已验证到微信公众号后台预览发送
- [x] Obsidian 插件至少能承担一个不迷路的入口
- [x] 真实微信回归截图 / 录屏补齐
- [x] 首次试用者视角的完整 walkthrough 已按首页、飞书、照片和当前文章路径归档

当前证据：

- `PROGRESS.md` 中 2026-07-01 的真实飞书闭环验证记录
- `obsidian-plugin/README.md`
- `docs/USER_GUIDE.md`
- `docs/first-run-evidence/`
- `docs/FIRST_RUN_WALKTHROUGH_ZH.md`
- `docs/RELEASE_GATE_v0.4.2_ZH.md`

### M3：入口协议稳定

目标：

- 插件 / App / Agent 不再各自拼工作区语义

完成标准：

- [x] `workspace --json`
- [x] `status --json`
- [x] `check --json`
- [x] `capabilities --json`
- [x] Obsidian 插件改接 `workspace --json`
- [x] Agent / App 入口协议文档存在
- [ ] 再有一个真实入口继续复用这层协议

当前证据：

- `src/app.rs`
- `obsidian-plugin/main.ts`
- `docs/AGENT_PROTOCOL_ZH.md`

### M4：飞书正式模块化

目标：

- 飞书路线不再像附加功能，而是核心输入工作流

完成标准：

- [x] 飞书路线文档成为一级入口
- [x] 默认保守模式与显式快速模式已明确
- [x] `--draft` / `--preview` / `--push` 语义稳定
- [x] 幂等更新与 `action: created | updated` 已稳定
- [x] 为未来照片 / 语音输入源整理统一输入模型（首版文档已落到 `docs/INPUT_MODEL_ZH.md`，飞书 Inbox 已开始补通用 `external_id`）
- [ ] 飞书入口继续收口成更完整的流程说明或 UI 入口

当前证据：

- `src/intake.rs`
- `src/ai_workflow.rs`
- `docs/RECOMMENDED_WORKFLOWS_ZH.md`
- `docs/AGENT_PROTOCOL_ZH.md`
- `docs/INPUT_MODEL_ZH.md`

## 未来 2-4 周执行顺序

### P1：真实用户证据收口（已完成）

优先级：最高

已完成：

1. 补真实微信回归截图 / 短录屏
2. 按“第一次试用用户”视角再走一次主路径
3. 把踩坑写回文档

完成证据：

- 真实公众号草稿创建、编辑器配置和后台预览发送的脱敏截图已归档
- 首页、飞书、照片路径共 11 份首次体验证据已归档，并通过 `moonpub evidence-status --strict`
- `moonpub release-check --strict` 已通过；最终公开发布仍需人工完成 PR 审阅、合并、tag 与 release asset smoke

### P2：把插件入口做得更像首页

优先级：高

要做什么：

1. 基于 `workspace --json` 做更清晰的提示
2. 区分“工作区入口”和“当前文章入口”
3. 把风险 target 展示得更自然

完成标准：

- 插件不只是弹 Notice
- 用户能在 Obsidian 里更自然判断下一步

当前状态：

- 首页工作台已经形成第一版
- 真实证据已补齐；当前重点转向“补细节、补一致性和插件回归”

### P3：定义统一输入模型

优先级：高

要做什么：

1. 归纳飞书、照片、语音的共同输入形态
2. 明确哪些字段属于原始素材，哪些字段属于草稿元数据
3. 为未来输入源抽象留接口

完成标准：

- 至少一份输入模型说明文档
- 飞书不再像特例

### P4：继续把 Agent-ready 边界写清楚

优先级：中高

要做什么：

1. 基于 `docs/AGENT_PROTOCOL_ZH.md` 再收口一次
2. 明确哪个命令属于状态层，哪个属于动作层
3. 如果有必要，继续补 `workspace` 上层集成示例

完成标准：

- 未来新入口不需要重新猜 MoonPub 的流程边界

## 当前不优先做的事

这些事现在都不是最该先做的：

- 立刻拆飞书成单独仓库
- 立刻承诺“完整 AI Agent 产品”
- 继续横向扩很多平台
- 云端托管 AppSecret
- 把“自动最终发表”当卖点

原因只有一个：

**当前主问题仍然是“让用户会用”，不是“让能力继续发散”。**

## 当前下一步

如果只看眼前最值得做的 3 件事，就是：

1. 补真实微信回归证据
2. 补插件首页 / 飞书 / 照片三条首次体验证据
3. 为飞书 / 照片 / 语音整理统一输入模型

## 使用方式

后面每做一次比较大的推进，都应该回到这份文档更新：

- 哪个里程碑被推进了
- 哪个完成标准已经有证据
- 当前下一步是否改变

如果后续判断发生变化，也优先改这份文档，而不是继续让计划散落在 README、ROADMAP、PROGRESS 和聊天记录里。
