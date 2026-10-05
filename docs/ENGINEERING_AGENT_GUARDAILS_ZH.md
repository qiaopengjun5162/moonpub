# MoonPub 编程 Agent 协作护栏

这份文档是给 Codex、Claude Code 等编程 Agent 看的元规则。它补充 `AGENTS.md` 里项目-specific 的边界，说明分支、提交、文档、PR 和生产边界等通用协作方式。

## 开始协议

动手改代码前，先读：

1. `README.md` / `README_zh.md`
2. `AGENTS.md`
3. `docs/PRODUCT_WRAP_ZH.md`
4. `docs/EXECUTION_PLAN_ZH.md`
5. 目标模块的 README 或文档

如果改动涉及微信 API、浏览器自动化、AI provider、文件写回或新输入源，再读 `docs/ENGINEERING_LESSONS_ZH.md` / `docs/ENGINEERING_LESSONS.md` 和 `docs/KNOWN_LIMITS_ZH.md` / `docs/KNOWN_LIMITS.md`。

## 分支规则

- 不要直接在 `main` 上开发。
- 每个分支只聚焦一个模块、一条工作流或一个已记录的计划项。
- 提交信息使用 Conventional Commits：
  - `feat`：新功能
  - `fix`：bug 修复
  - `docs`：文档更新
  - `refactor`：行为不变的重构
  - `test`：测试补充或修复
  - `build`：构建/依赖/工具链
  - `ci`：CI/check 门禁
  - `chore`：日常维护

## 文档先行规则

新增功能、行为变更、迁移、输入源扩展或生产相邻改动，先写/更新文档，再写代码。说明：

- 目标模块和边界
- 验证命令
- 是否触及微信 API / 浏览器自动化 / AI provider / 文件写回
- 回滚或降级方式

纯拼写错误、机械格式化、单一行 bugfix 不需要额外设计文档。

## 架构边界规则

遵守 `AGENTS.md` 里的模块职责划分：

- `src/main.rs`：只加载环境、解析参数、输出结果
- `src/cli.rs`：CLI 解析和命令级协议声明
- `src/app.rs`：命令路由和用例编排
- `src/app_*_commands.rs`：具体命令包装
- `src/ai_workflow.rs`：AI 调用和文件写回编排
- `src/push.rs` / `src/wechat.rs`：微信 API
- `src/publish.rs` / `src/cdp.rs`：浏览器自动化
- `src/intake.rs` / `src/intake/*.rs`：输入源
- `src/protocol.rs`：结构化 JSON payload 收口

不要把平台特有逻辑（微信、飞书、照片、AI）耦合到 `app.rs` 或 `app_support.rs` 里。

## 结构化输出规则

新增机器可读命令（`doctor`、`workspace`、`check`、`preflight`、`push` 等）时，优先把 JSON builder 收口到 `src/protocol.rs`，并优先用 `serde::Serialize` payload 输出，避免继续扩手写 JSON 字符串；不要把 JSON builder 塞回 `src/app.rs`。

## 生产边界规则

改动如果触及以下任一边界，必须在 PR 中明确说明风险点和回滚方式：

- 微信 API（`push` / `ship` / `configure` / `login`）
- 浏览器自动化（CDP / headless Chrome / 微信后台 DOM 操作）
- AI provider（`write` / `expand` / `polish` / `ship --ai` / `intake --analyze-images`）
- 文件写回（Inbox / drafts / ready / published / `.env` / `moonpub.toml` / `.moonpub/`）
- 插件/浏览器端行为（`obsidian-plugin/`）

不要把真实凭据（`WECHAT_SECRET`、`DEEPSEEK_API_KEY`、`.env`、`moonpub.toml`、session、cookie、`pass_ticket`、`uin`、token）写入 git。

## 提交信息规范

仓库使用 [Conventional Commits](https://www.conventionalcommits.org/) 以便 Release Please 自动生成 changelog 和版本号。

允许的类型（与 `.release-please-config.json` 一致）：

- `feat`：新功能
- `fix`：bug 修复
- `docs`：文档更新
- `refactor`：行为不变的重构
- `test`：测试补充或修复
- `build`：构建/依赖/工具链
- `ci`：CI/check 门禁
- `chore`：日常维护
- `perf`：性能优化
- `revert`：回滚
- `style`：纯格式调整

格式示例：

```text
feat: add wechat-health command
fix(push): retry cover upload on timeout
docs: update onboarding guide for cookie auth
```

### 本地安装 git hook

```bash
./scripts/install-git-hooks.sh
```

安装后 `commit-msg` hook 会在每次提交前检查第一行是否符合 Conventional Commits，不符合则阻止提交。

### CI 检查

`build.yml` 会在 PR 中运行 `./scripts/check-commits.sh`，检查 `origin/main..HEAD` 范围内所有非 merge 提交；并通过 `secret-scan` job（`gitleaks/gitleaks-action@v2`）扫描提交中的真实凭据/密钥，命中即失败。

## 验证规则

代码改动至少跑：

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run --all-features
```

修改 `obsidian-plugin/` 时额外运行：

```bash
cd obsidian-plugin && npm ci && npm test && npm run build
```

文档改动推荐本地运行：

```bash
# 检查全部 Markdown 文件
npx markdownlint-cli2

# 检查当前分支变更的文件（CI 也会跑这个）
./scripts/lint-docs.sh --changed
```

## 停止条件

以下情况必须先停下来，确认用户/owner 同意后再继续：

- 引入新的编程语言或构建系统
- 启用真实微信后台发送或自动发布
- 放宽飞书/微信/AI 的权限或数据边界
- 替换或删除 `AGENTS.md`、核心架构文档或发布门禁
- 删除历史归档、rollback 材料或证据文件

## PR 创建规则

- 使用仓库提供的 PR 模板（`.github/PULL_REQUEST_TEMPLATE.md`）。
- 说明改动类型、验证命令、文档同步、生产边界和凭据检查。
- 把 PR-Agent 的评论当作建议，不能替代 CI、CODEOWNERS 或 owner review。

## 相关文件

- `.pr_agent.toml`：PR-Agent 中文审查指令
- `.markdownlint-cli2.yaml`：Markdown 文档 lint 规则
- `.github/PULL_REQUEST_TEMPLATE.md`：PR 模板
- `.github/CODEOWNERS`：代码 owner 规则
- `AGENTS.md`：项目-specific 架构与命令边界

## Release 流程

仓库使用 Release Please 管理 release PR 和 draft release：

1. 合并 feature / fix PR 到 `main` 后，Release Please action 会自动维护一个 release PR。
2. release PR 包含根据 Conventional Commits 生成的 `CHANGELOG.md`、版本号（更新 `Cargo.toml` / `Cargo.lock` / `.release-please-manifest.json`）。
3. owner 审核并合并 release PR。
4. Release Please 创建 draft GitHub Release。
5. owner 手动发布该 draft release，生成 tag。
6. 现有的 `.github/workflows/release.yml` 会监听 tag push（`v*`），自动构建多平台二进制和 Obsidian 插件包，并上传到 release assets。

不要手动创建 tag 或修改 `CHANGELOG.md` 顶部的 release 条目；这些由 Release Please 维护。`docs/ENGINEERING_LESSONS_ZH.md` / `docs/ENGINEERING_LESSONS.md` 里的工程经验记录不受此限制。
