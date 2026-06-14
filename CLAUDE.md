# MoonPub

Rust CLI for publishing Obsidian markdown to WeChat 公众号.

## Build & Run

```
cargo build --release
./target/release/moonpub --help
```

## Common Commands

```bash
# Full publish flow (recommended)
moonpub ship <article.md> [--style dark|literary|warm|...]

# Step by step
moonpub render <article.md>
moonpub push <article.md>

# Cover only
moonpub cover <article.md> [--style dark] [--screenshot]

# Browser automation only
moonpub configure [--headed]
```

## Architecture

- `src/lib.rs` — all logic: CLI parsing, config, rendering, API calls
- `src/publish.rs` — browser automation (CDP via chromiumoxide)
- `src/theme.rs` — render themes (default, warm, dark, geek)
- `src/cover.rs` — cover HTML templates
- `src/wechat.rs` — WeChat API client

## Config (`moonpub.toml` in vault root)

```toml
[vault]
root = "/path/to/obsidian"

[wechat]
appid = "wxa..."
author = "作者名"
theme = "geek"          # default | warm | dark | geek
thumb_media_id = ""     # ship 命令会自动上传封面刷新此值，无需手填
qrcode = "Context/assets/qrcode.jpg"

[blog]
kind = "zola"
root = "/path/to/blog"
```

## Themes

| Name    | Background | Accent   | Use case         |
|---------|-----------|----------|------------------|
| default | #fff      | #2c2c2c  | clean minimal    |
| warm    | #fdf8f4   | #e67e22  | literary/reading |
| dark    | #1a1a1a   | #64b5f6  | dark mode        |
| geek    | #0d1117   | #3fb950  | terminal/tech    |

## ship 流程

`ship` 命令做完整一键发布：
1. 生成封面 HTML（literary 默认风格）
2. Headless Chrome 截图 → 上传微信永久素材 → 拿 thumb_media_id
3. 渲染文章 HTML（含封面注入顶部）
4. 调 WeChat draft/add API 推送草稿
5. 浏览器自动化设置原创声明、留言、预览

## Browser Automation 已知问题

WeChat 编辑器是 live web app，UI 随时会改。已知稳定/不稳定状态（截至 2026-06-14）：

| 步骤     | 状态 | 说明 |
|--------|------|------|
| 原创声明 | ✅   | 正常 |
| 留言     | ✅   | 正常 |
| 预览     | ✅   | 正常 |
| 赞赏     | ⚠    | toggle 点击后 state 不变，微信端控制，软失败 |
| 创作来源 | ⚠    | 微信已将此入口合并至原创声明弹窗，跳过 |

⚠ 软失败不影响草稿发布，只是该项未配置。

## TOML 解析

`Config::from_toml` 手写（无 toml crate），按 section header 区分同名 key：
```toml
[vault]
root = "/obsidian"   # → cfg.vault_root

[blog]
root = "/blog"       # → cfg.blog_root
```

## 环境变量

```
WECHAT_APPID   覆盖 config 中的 appid
WECHAT_SECRET  必填，不进 config 文件
```

## 历史问题记录

### 2026-06-14: ship 命令 thumb_media_id 失效
**问题**: WeChat 永久素材 media_id 会失效（删除后报 40007），导致 `ship` 推送失败。
**根因**: ship 只读 config 里的 thumb_media_id，没有自动刷新。
**修复**: ship 命令现在每次截图封面 PNG → 上传微信 → 用新 media_id，config 里的作为最后兜底。

### 2026-06-14: 创作来源点"未添加"弹出原创声明弹窗
**问题**: step_chuangzuo 点击"未添加"后弹出的是原创声明弹窗，不是创作来源选择框。
**根因**: 微信编辑器 UI 更新，把创作来源入口合并进了原创声明弹窗。
**修复**: 检测到弹窗内有"声明类型"/"文字原创"/"无需声明"文字时，按 Escape 关闭，记 ⚠ 跳过。

### 2026-06-13: 赞赏 toggle offsetParent 不可见
**问题**: cdp_click_css 用 `offsetParent !== null` 判断可见性，赞赏 toggle 在 DOM 中但不可见，点击无效。
**修复**: 改用 JS `.click()` 直接点击绕过可见性检查。当前 toggle 点击后 state 仍为"不开启"，属微信端控制，软失败处理。

### 2026-06-13: TOML root key 顺序 bug
**问题**: vault.root 和 blog.root 都叫 `root`，没有 section 跟踪时后者会覆盖前者。
**修复**: `Config::from_toml` 加 `let mut section = ""` 跟踪当前 section header。
