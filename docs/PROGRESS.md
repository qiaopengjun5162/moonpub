# MoonPub 项目进度

> 本文件记录当前开发状态。每次会话开始时，先读此文件以确定从哪继续。

## 当前版本状态

- **分支**: `main`
- **最近提交**: 见 `git log -1 --oneline`
- **测试**: `cargo nextest run` 通过
- **工作树**: 干净，无未提交改动

## 当前会话上下文

> 如果你用 `/clear` 或 `/new` 新开会话，先读这一段。

**本次会话目标**: 修复创作来源 (step_chuangzuo) 稳定性问题。

**已完成**:
- 创作来源 修复：radio value="4" 精确定位 + `.js_claim_source_desc` wrapper + `.js_claim_source_selected` 验证
- `moonpub ship` 端到端验证通过 (headless)
- 全部 browser automation 步骤：原创声明 ✅、赞赏 ✅、留言 ✅、创作来源 ✅、预览 ✅
- 合集 ⏸ (已禁用)
- CLAUDE.md 更新，记忆文件更新

**下次继续**:
- 需微信 UI 变化时再针对性修复
- 可选：合集自动化

## 已完成

### 2026-06-16: 创作来源 radio value 修复
- 打开 picker: 直接 `querySelector('.js_claim_source_desc')`，找不到则遍历 label 找文本以"创作来源"开头的行
- 选择选项: `querySelectorAll('input[type="radio"][value="4"]')` 代替文本匹配
- 验证: 通过 `.js_claim_source_selected` span 读已选值
- headed 和 headless 模式均通过测试

### 2026-06-16: ship 端到端验证
- `moonpub ship` 完整跑通: cover → render → push → configure → export
- 全部 6 步自动配置 (除合集) 稳定通过

### 2026-06-16: 模块拆分收尾
- 将 `publish.rs` 中的 chromiumoxide 底层操作抽到 `src/cdp.rs`
- 将微信编辑器自动化步骤抽到 `src/publish_steps.rs`
- 将 Markdown → HTML 渲染抽到 `src/markdown.rs`
- `publish.rs` 现在只负责流程编排

### 2026-06-15: lib.rs 模块化
- 将 `lib.rs` 拆分为 `cli.rs` / `config.rs` / `error.rs` / `article.rs` / `render.rs` / `export.rs` / `status.rs` / `preview.rs` / `system.rs` / `push.rs`
- 修复 `wechat.rs` 与 `app.rs` 的循环依赖

## 当前架构

```
src/
  main.rs          # 入口
  app.rs           # 命令路由与用例编排
  cli.rs           # CLI 解析
  config.rs        # TOML 配置
  error.rs         # 错误类型与工具函数
  article.rs       # frontmatter 解析
  render.rs        # 文章级渲染 → HTML + draft.json
  markdown.rs      # Markdown → WeChat HTML 转换
  push.rs          # WeChat API 推送
  wechat.rs        # WeChat API 客户端
  publish.rs       # 浏览器自动化流程编排
  cdp.rs           # CDP 底层辅助（chromiumoxide）
  publish_steps.rs # 微信编辑器各配置步骤
  cover.rs         # 封面 HTML 模板
  theme.rs         # 渲染主题
  humanize.rs      # 去 AI 味
  illustrate.rs    # Block 模板渲染
  radar.rs         # 热点管理
  ...
```

## 待处理 / 下一步（建议）

当前代码层面任务已全部完成。下一步：

1. 如果微信 UI 变化导致步骤失效，针对性修复
2. 可选：合集自动化、封面图 CDP 设置
3. 可选：为 `cdp.rs` / `publish_steps.rs` 中依赖 `Page`/`Browser` 的异步函数增加 mock 测试（优先级低）

## 已知问题

见 `CLAUDE.md` 的「历史问题记录」和 `docs/BROWSER_AUTOMATION.md`。

当前没有阻塞性 bug。

## 如何继续

- 说"继续" → 默认从「待处理 / 下一步」的第一项开始
- 说"做 X" → 直接做 X
- 完成一项后，更新本文件的「已完成」和「待处理」
