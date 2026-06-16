# MoonPub 项目进度

> 本文件记录当前开发状态。每次会话开始时，先读此文件以确定从哪继续。

## 当前版本状态

- **分支**: `main`
- **最近提交**: 见 `git log -1 --oneline`
- **测试**: `cargo test` 通过，114 个测试全部通过
- **工作树**: 干净，无未提交改动

## 已完成

### 2026-06-16: 模块拆分收尾
- 将 `publish.rs` 中的 chromiumoxide 底层操作抽到 `src/cdp.rs`
- 将微信编辑器自动化步骤抽到 `src/publish_steps.rs`
- 将 Markdown → HTML 渲染抽到 `src/markdown.rs`
- `publish.rs` 现在只负责流程编排
- `render.rs` 现在只负责文章级渲染编排（文件 I/O、frontmatter、draft JSON）
- `lib.rs` 已声明三个新模块

### 2026-06-15: lib.rs 模块化
- 将 `lib.rs` 拆分为 `cli.rs` / `config.rs` / `error.rs` / `article.rs` / `render.rs` / `export.rs` / `status.rs` / `preview.rs` / `system.rs` / `push.rs`
- 修复 `wechat.rs` 与 `app.rs` 的循环依赖：把 `extract_ip_from_message` 下沉到 `error.rs`
- 测试分散到各模块

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
  ...
```

## 进行中

无。

### 2026-06-16: 为新拆分模块补测试
- 为 `markdown.rs` 增加 fence block 解析测试：
  - 多个 fence 连续出现
  - 未闭合 fence
  - `split_fence_props` 正确解析 key:value
  - `split_fence_props` 遇到非属性行停止
  - `md_to_wechat_html` 渲染 `intro` 和 `callout` fence
- 为 `cdp.rs` 增加 `js_str` 测试：
  - 普通文本、双引号、反斜杠、控制字符、空字符串

### 2026-06-16: 清理 dead code
- 删除 `cdp.rs` 中 4 个未使用的 `#[allow(dead_code)]` 函数：
  - `xclick_editor`
  - `retry_click_editor`
  - `cdp_click_xpath`
  - `dump_buttons`
- 删除后 `cdp.rs` / `publish_steps.rs` 中无 `#[allow(dead_code)]` 残留

### 2026-06-16: 更新 README 架构描述
- 将 Architecture 部分的文件指向更新为当前模块结构
- 补充 `cli.rs` / `config.rs` / `article.rs` / `markdown.rs` / `illustrate.rs` / `cdp.rs` / `publish_steps.rs`

### 2026-06-16: 优化 browser automation 重试和判断逻辑
- `cdp.rs` 新增 `has_visible_text` / `has_visible_text_js` / `close_dialog` 辅助函数
- `publish_steps.rs` 中 `step_chuangzuo` 改用 `has_visible_text` 检测原创声明弹窗，用 `close_dialog` 关闭
- `step_liuyan` 改用 `close_dialog` 关闭弹窗
- `step_zanshang` 增加多个 toggle 选择器（`.js_reward_setting_tips`、`.weui-switch`、`label[for*="reward"]` 等）
- 为 `has_visible_text_js` 增加单元测试

### 2026-06-16: 修复原创声明协议勾选失败（第二轮）
- 重写 `cdp::check_agreement`：直接定位"我已阅读并同意" label，点击其内部的 checkbox input / 图标 / label 本身
- 删除未使用的 `cdp_click_any_text`

### 2026-06-16: Browser automation 稳定性验证通过
- 实际运行 `./target/release/moonpub test-zanshang --headed`
- 原创声明：协议自动勾选/主动勾选成功，"文字原创"选中，确定生效
- 赞赏：toggle 成功切换，state 从"不开启"变为"账户: 寻月隐君"
- **当前实现已冻结**，不再修改 browser automation 逻辑，避免回归

### 2026-06-16: 修复创作来源步骤点击错元素
- 问题：`step_chuangzuo` 用 `cdp_click_exact_last("未添加")` 会点页面上最后一个"未添加"，容易点到原创声明/原文链接等错误行
- 修复：改为 JS 定位包含"创作来源"文本的行/label，再点击其内部可点击元素
- 状态：已构建，待重新运行 `moonpub configure --headed` 验证

## 待处理 / 下一步（建议）

当前代码层面任务已全部完成。下一步：

1. **验证创作来源步骤** — 重新运行 `./target/release/moonpub configure --headed`，确认创作来源能正确设置为"个人观点，仅供参考"
2. 如果 `moonpub ship` 在实际发布中遇到新的微信 UI 变化，再针对性修复
3. 可选：为 `cdp.rs` / `publish_steps.rs` 中依赖 `Page`/`Browser` 的异步函数增加 mock 测试（优先级低）

## 已知问题

见 `CLAUDE.md` 的「历史问题记录」和 `docs/BROWSER_AUTOMATION.md`。

当前没有阻塞性 bug。

## 如何继续

- 说"继续" → 默认从「待处理 / 下一步」的第一项开始
- 说"做 X" → 直接做 X
- 完成一项后，更新本文件的「已完成」和「待处理」
