# MoonPub 插件化核心设计

这份设计用于 v0.5。目标不是马上实现一个动态插件市场，而是先把 MoonPub 内部能力拆成稳定边界，让后续 Obsidian 插件、多平台发布、本地 App 都能调用同一套核心流程。

## 设计目标

- 保持 CLI 是稳定入口，现有命令行为不破坏。
- 把“发布到哪里”“渲染成什么”“导出到哪里”从命令编排里拆成清晰接口。
- 优先支持本地、可审计、用户自带凭证的插件形态。
- 先做内置插件接口，不急着加载第三方动态库或脚本。

## 非目标

- 不做云端插件市场。
- 不托管 AppSecret、access token 或用户文章。
- 不开放绕过微信扫码、验证码、审核、账号权限或最终发表确认的扩展点。
- 不承诺知乎、掘金、小红书等高维护平台第一批可用。

## 当前问题

现在的 `ship` 路径把封面、渲染、微信推送、浏览器辅助配置、博客导出串在一起。这个路径已经能用，但继续扩展 WordPress / Ghost / Obsidian / App 时会遇到几个问题：

- `src/ship.rs` 需要知道太多平台细节。
- `src/export.rs` 当前默认面向 Zola，未来接 Hugo / WordPress / Ghost 时边界不够清楚。
- Obsidian 插件只能 shell 调用 CLI，缺少稳定的状态和能力描述。
- 未来本地 App 如果直接复制 CLI 逻辑，会出现两套发布流程。

## 核心模型

v0.5 先引入这些内部概念：

```text
ArticleBundle
  - markdown path
  - html path
  - draft json path
  - media id path
  - stage: drafts / ready / published

RenderTarget
  - wechat-html
  - blog-markdown

PublishTarget
  - wechat-draft
  - wordpress-post
  - ghost-post

ExportTarget
  - zola
  - hugo
  - local-folder

Hook
  - before-render
  - after-render
  - before-push
  - after-push
  - after-published
```

第一步只需要类型和内部接口，不需要对外暴露复杂配置。

## 建议模块

```text
src/plugin.rs
  核心 trait、能力描述、通用结果类型

src/platform/
  mod.rs
  wechat.rs
  wordpress.rs   # 先定义接口边界，暂不实现网络调用
  ghost.rs       # 先定义接口边界，暂不实现网络调用

src/export/
  zola.rs        # 从现有 export.rs 逐步拆出
  hugo.rs        # 后续添加

src/bundle.rs
  ArticleBundle、stage 判断、bundle 移动
```

这不是一次性大重构。建议按最小风险顺序推进：

1. 抽 `ArticleBundle`，不改变命令行为。
2. 把 ready / published 移动逻辑从 `push.rs` 移到 `bundle.rs`。
3. 抽 `PublishTarget` trait，让微信实现成为第一个内置 target。
4. 抽 `ExportTarget` trait，让 Zola 成为第一个内置 export target。
5. 给 CLI 增加能力查询命令，例如 `moonpub capabilities --json`，供 Obsidian 插件和未来 App 调用。

## Trait 草案

```rust
pub trait PublishTarget {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn requires_network(&self) -> bool;
    fn requires_browser(&self) -> bool;
    fn publish(&self, ctx: PublishContext<'_>) -> Result<PublishOutcome, AppError>;
}

pub trait ExportTarget {
    fn id(&self) -> &'static str;
    fn display_name(&self) -> &'static str;
    fn export(&self, ctx: ExportContext<'_>) -> Result<ExportOutcome, AppError>;
}
```

约束：

- `publish` 不负责读取全局环境变量；凭据解析仍由 config 层完成。
- `requires_network` / `requires_browser` 必须能被 CLI 和 Obsidian 插件展示给用户。
- `PublishOutcome` 必须包含下一步提示，例如“去微信后台人工确认”。

## CLI 影响

现有命令保持可用：

```bash
moonpub render article.md
moonpub push article.md --render
moonpub ship article.md --style literary
moonpub export article.md
```

新增命令建议：

```bash
moonpub capabilities --json
moonpub publish article.md --target wechat-draft
moonpub export article.md --target zola
```

`push` 和 `ship` 可以继续作为常用快捷命令，底层逐步转向调用 `PublishTarget` / `ExportTarget`。

## Obsidian 插件影响

Obsidian 插件不应该重新实现发布流程。它应该：

- 检测本地 `moonpub`。
- 调用 `moonpub capabilities --json` 获取能力和风险提示。
- 先检查顶层 `schema_version` / `moonpub_version`，避免插件调用不兼容的 CLI 元数据。
- 读取 capability 中的 argv 风格 `command` 模板，替换 `"{article}"` 占位符后用进程参数数组执行。
- 对当前文件执行 `render` / `preview` / `cover` / `check`。
- 只有用户明确确认时才执行 `push` / `ship`。
- 展示命令是否会触达网络、微信 API 或打开 Chrome。

## 多平台顺序

第一批只建议做：

1. `wechat-draft`：现有核心能力，作为第一个 `PublishTarget`。
2. `zola`：现有导出能力，作为第一个 `ExportTarget`。
3. `hugo`：文件系统导出，低风险。
4. `wordpress`：官方 REST API，用户自带站点和凭据。
5. `ghost`：官方 Admin API，用户自带站点和 Admin key。

暂缓：

- 知乎、掘金、小红书、B 站专栏。
- 任何需要绕过网页限制、验证码、反自动化策略的平台。

## 安全边界

- 配置优先级仍是环境变量 > 本地 env 文件 > `moonpub.toml`。
- `WECHAT_SECRET` 不写入插件配置，不进入 Obsidian 插件存储。
- 所有会触达网络的 target 必须在 capability 中标记。
- 所有会打开或控制浏览器的 target 必须在 capability 中标记。
- 自动化失败不能影响已经创建的官方 API 草稿。

## 测试策略

- `ArticleBundle`：测试 drafts → ready → published 的移动和缺失文件容错。
- `PublishTarget`：用 fake target 测试 trait 调度和 outcome。
- `wechat-draft`：保留现有无凭证单元测试，不在 CI 触达真实微信。
- `ExportTarget`：Zola 现有测试迁移到 target 测试。
- `capabilities --json`：快照测试顶层 schema/version、风险标记和 `command` 模板，保证 Obsidian 插件可依赖。

## 成功标准

v0.5 完成时应满足：

- 现有 `render` / `push` / `ship` / `export` 行为保持兼容。
- 微信发布和 Zola 导出已经走内部 target 接口。
- CLI 能输出带 schema/version 的 JSON capabilities，并给每个 target 提供 argv 风格 `command` 模板。
- Obsidian 插件可以根据 capabilities 展示风险提示。
- 没有新增云端凭据托管或无人值守最终发布能力。
