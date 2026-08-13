# MoonPub 命令参考与错误码

所有命令的通用形式：`moonpub --articles <dir> <subcommand> <file.md> [flags]`
- `<dir>` 是文章根目录（如 `.` 表示项目根，`drafts` 表示草稿目录）。
- `<file.md>` **相对**于 `<dir>`；传绝对路径会报 "path duplication"。
- push 成功后文章从 `drafts/` 移到 `ready/`，后续命令改用 `ready/<slug>.md`。

## 主题（theme 字段可选值）
未来/科技/动漫感：`geek-black`（深底+霓虹绿）、`cyber`（赛博霓虹）
亮色干净：`blueprint`（蓝网格）、`minimal`、`porcelain`、`moonlit`
文艺/杂志：`newsletter`、`magazine`、`ink`、`serif`、`ai-lab`
注：`clean` 含橙色强调色（`#e65100`），用户若不喜欢黄/橙应避免。

## 封面风格（cover --style 可选值）
`clean` `minimal` `ink` `serif` `blueprint` `geek-black` `ai-lab` `gradient`
科技/动漫感推荐 `geek-black`（深底+霓虹绿光+网格）。

## 命令清单
```bash
# 1. 渲染 HTML + draft.json
moonpub --articles . render <slug>.md

# 2. 发布前质量门（微信硬约束 + 图片完整性）
moonpub --articles . preflight <slug>.md

# 3. 本地 HTML 预览（不打开浏览器）
moonpub --articles . preview <slug>.md --no-open

# 4. 生成封面（必须 --screenshot 才出 PNG；不写回 frontmatter 的 cover:）
moonpub --articles . cover <slug>.md --style geek-black --screenshot

# 5. 推送到微信草稿箱（自动删旧草稿、写新 media_id、自动发手机预览）
moonpub --articles . push <slug>.md --render

# 6. 单独发手机预览（收件人回退顺序：--to > 环境变量 > .moonpub/preview_to）
moonpub --articles . test-yulan --title "<标题>"

# 7. 微信会话登录（headed 扫码，持久化到 ~/.config/moonpub/session.json）
moonpub login
```

## 错误码与根因
| 现象 | 根因 | 修复 |
|---|---|---|
| `40164 invalid ip` | 机器出口 IP 不在微信 IP 白名单 | 关代理稳定出口 → 加微信报的 `current IP` 到 公众号后台→开发→基本配置→IP白名单 |
| `40007 invalid media_id` | 封面未配（frontmatter 无 `cover:` 或 push 取不到） | 手动加 `cover: <slug>.cover.png` 到 frontmatter 再 push |
| `persistent Chrome profile is already in use` | 登录弹出的自动化 Chrome 窗口未关，锁住 profile | 手动关掉那个窗口再跑 test-yulan |
| path duplication 类报错 | render/preview 传了绝对路径 | 改用相对 `--articles` 根的路径 |

## preflight 检查项（2026-08-12 起）
- `title_required` / `title_limit`（≤64 字符，取 `wechat_title()` 真实发送值）
- `digest_limit`（≤120 字符，空则微信自动提取）
- `title_h1_match`（正文 H1 须与 frontmatter 标题一致，warn）
- `image_links`（正文本地图片引用须存在，fail）
- `orphan_images`（同目录以本 slug 前缀、未被引用的图片，warn；用 slug 前缀过滤避免 ready/ 多文章误报）

## 标准结尾模板（footer）
- 全局配置：`moonpub.toml` 的 `[footer]` 段，cli 自动从 `--articles` 根读取。
- 例：项目根 `moonpub.toml` 配 `[footer]` 后，render 自动拼入每篇结尾。
- 模板来源示例：`moonpub-data/moonpub.toml` 的 `[footer]`（寻月阁入阁模板 + `qrcode.png`）。
- 切勿把 footer 手塞进正文 body——应通过配置让渲染层追加。

## CI 门禁（build.yml）
- `test` job：fmt / clippy `-D warnings` / `cargo audit` / nextest / commit lint / markdownlint
- `secret-scan` job：`gitleaks/gitleaks-action@v2`，`fetch-depth:0` 全量扫描
- `windows-smoke` job：release 构建 + 无凭据冒烟
- 验证口诀：`cargo fmt --all -- --check` + `cargo clippy --all-targets --all-features --tests --benches -- -D warnings` + `cargo nextest run --all-features`

## 环境注意
- 本机网络出口受限时 `curl`/`go install` 会被 SIGKILL，gitleaks 等无法本地安装；改用针对性 grep 复刻规则（`secret/token/key=高熵值` + 私钥块 + `.env` + 微信会话值）做等效静态扫描。
- `.claude/settings.local.json` 含类 secret 串，但已被全局 gitignore 的 `**/.claude/settings.local.json` 排除且未跟踪，gitleaks-action（基于 git 扫描）扫不到，不会泄露。
