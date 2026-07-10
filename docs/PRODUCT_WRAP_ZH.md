# MoonPub 产品包装说明

这份文档只回答一个问题：

**MoonPub 现在应该怎样被理解，才不会把它看成一堆零散命令。**

它不是愿景文档，也不是实现细节说明。目标是把当前已经存在的能力，收口成一个用户、插件、App 和 Agent 都能复用的产品形态。

## 一句话定位

**MoonPub 是一个本地优先的内容发布内核。**

它把各种输入素材接进来，整理成草稿，完成本地预览、公众号草稿推送和后台辅助配置，并把“下一步该做什么”明确交给上层入口。

## 它现在不是什么

当前不要把 MoonPub 理解成下面这些东西：

- 不是无人值守自动发文机器人
- 不是只会推公众号草稿的单一命令
- 不是只服务飞书秒记的一次性脚本
- 不是已经完整产品化的多端 SaaS

更准确地说，它现在是：

- 一个可以真实跑通内容发布主链路的本地内核
- 一个已经开始具备多输入源能力的工作流系统
- 一个适合继续被插件、App、Agent 包装的底层运行时

## 三层结构

为了避免继续把所有能力混成“很多命令”，现在建议明确按三层理解 MoonPub。

### 第一层：MoonPub Core

定位：本地发布内核

负责：

- Markdown / Obsidian 文章渲染
- 微信兼容 HTML 与 draft JSON 生成
- 封面生成
- 微信 API 草稿推送
- 微信后台浏览器自动化辅助
- 文章阶段管理
- 博客导出
- 结构化 JSON 协议

这层解决的是：

**文章怎样稳定地从本地文件推进到可发布状态。**

### 第二层：Input Workflows

定位：输入源工作流层

负责把“还不是文章”的内容先接进系统，再推进到草稿、预览、发布这条主线。

当前已经正式存在的输入工作流有两条：

1. 飞书秒记
   `Feishu Minutes -> Inbox -> Draft -> Preview -> WeChat Draft`
2. 照片素材
   `Photos -> Inbox -> Draft -> Preview -> WeChat Draft`

正在评估但还不是正式入口的输入工作流：

- 微信公众号归档
  `Known WeChat URL -> Inbox -> Draft -> Preview`

这条线应先按 [WECHAT_ARCHIVE_WORKFLOW_ZH.md](WECHAT_ARCHIVE_WORKFLOW_ZH.md) 的安全边界推进：默认只处理用户显式提供的公开 URL，不自动抓历史列表，不保存敏感凭据。

这层解决的是：

**不同来源的原始素材，怎样先被整理成可编辑草稿。**

### 第三层：User Surfaces

定位：用户入口层

负责把 MoonPub 的能力，以不同入口方式交给真实用户或上层系统使用。

当前已经正式存在的入口有两个：

1. CLI
2. Obsidian 插件

正在形成中的入口有两个：

1. 本地 App
2. Agent 包装

这层解决的是：

**用户或系统，应该从哪里进来，先看到什么，下一步怎么走。**

## 当前正式能力地图

如果按产品形态来讲，MoonPub 现在已经不是“只有 push 命令”的状态了，而是至少有下面这些正式部件：

### 已正式存在的输入工作流

- `已有 Markdown 文章`
- `飞书秒记`
- `照片素材`

### 已记录但未正式启用的输入工作流

- `微信公众号归档 URL`

### 已正式存在的入口层

- `moonpub` CLI
- `obsidian-plugin/`

### 已经稳定到可以被上层复用的协议层

- `workspace --json`
- `workflow-registry --json`
- `status --json`
- `check --json`
- `preview --json`
- `push --json`
- `draft-from-inbox --json`
- `intake feishu ... --draft --json`
- `intake photos ... --draft --json`
- `capabilities --json`

## 为什么飞书路线现在不拆项目

当前结论很明确：

**飞书秒记先作为 MoonPub 内部正式模块推进，不拆新项目。**

原因不是“以后永远不拆”，而是现在最强价值仍然依赖 MoonPub 主链路：

- 飞书的价值不只是导入文本，而是导入后能直接变成可发布草稿
- 现在真正缺的是用户入口和产品表达，不是仓库拆分
- 输入源和发布内核仍然需要高频一起迭代

同样的判断也适用于照片路线：先做成正式输入工作流，而不是急着独立成另一个项目。

## Agent 应该怎样接 MoonPub

当前也不建议把 Agent 理解成“另起一个新产品”。

更合理的方向是：

**把 MoonPub 包装成 Agent-ready 的本地发布内核。**

也就是：

- Agent 先看 `workspace --json`，判断当前工作区处于什么状态
- 再看 `workflow-registry --json`，读取当前正式支持的工作流契约、风险边界和安全起点
- 再根据 `status --json` / `check --json` 判断整体池子或当前文章
- 最后才去触发 `preview`、`draft-from-inbox`、`intake feishu`、`intake photos`、`push`

这样 Agent 不需要重新发明工作流语义，而是继续复用 MoonPub 已经开始稳定下来的协议层。

## 现在最推荐的第一次体验

如果是第一次接触 MoonPub，当前最推荐的顺序不是一下子看完所有命令，而是：

1. 先从 Obsidian 里的 `MoonPub 首页工作台` 进入
2. 再按上下文去选当前文章、飞书或照片入口
3. 第一轮先停在草稿和本地预览
4. 确认你理解工作流节奏后，再推进到真实微信草稿

原因很简单：

- 插件首页现在已经开始承担统一入口层角色
- 飞书路径最能体现工作流价值
- 照片路径说明 MoonPub 已经开始具备正式多输入源形态
- 先停在草稿和本地预览，最符合 MoonPub 当前“副驾驶而不是无人值守机器人”的边界

## 当前目标用户

MoonPub 现在最适合的用户，不是泛大众，而是下面这类人：

- 平时已经在 Obsidian / Markdown 里写作的人
- 愿意接受“先预览、再确认、再发布”的人
- 能自己配置本地环境和公众号凭证的技术用户
- 需要把飞书秒记、照片、后续语音等素材沉淀成文章的人

## 近期不做什么

为了避免继续分散，现在也明确几件短期内不作为主线推进的事：

- 不把 MoonPub 先包装成无人值守全自动发文机器人
- 不因为飞书路线有潜力就立刻拆成独立仓库
- 不先做很多新平台发布，而忽略当前入口收口
- 不先扩大量新命令，而忽略用户其实不知道先跑哪条路径

## 和其它文档的关系

如果你现在想解决的是不同层面的问题，可以继续这样看：

- 想知道第一次该走哪条路径：看 [RECOMMENDED_WORKFLOWS_ZH.md](RECOMMENDED_WORKFLOWS_ZH.md)
- 想知道当前阶段到底先做什么：看 [EXECUTION_PLAN_ZH.md](EXECUTION_PLAN_ZH.md)
- 想知道为什么项目现在这样定位：看 [PRODUCT_EVALUATION_ZH.md](PRODUCT_EVALUATION_ZH.md)
- 想知道插件 / App / Agent 应该怎么接协议：看 [AGENT_PROTOCOL_ZH.md](AGENT_PROTOCOL_ZH.md)
- 想知道输入源应该怎样统一建模：看 [INPUT_MODEL_ZH.md](INPUT_MODEL_ZH.md)

## 当前最简产品结论

如果只保留一句最重要的话：

**MoonPub 现在最应该被理解成：一个本地发布内核，已经开始长出正式输入工作流和正式用户入口层。**
