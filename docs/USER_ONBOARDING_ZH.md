# MoonPub 新用户上手指南

> 目标：用 10 分钟理解 MoonPub 能做什么，再用 30 分钟真实走完第一条路径。
>
> 如果你更想先知道“第一次先点哪里”，直接看 [FIRST_RUN_WALKTHROUGH_ZH.md](FIRST_RUN_WALKTHROUGH_ZH.md) 中文 / [FIRST_RUN_WALKTHROUGH.md](FIRST_RUN_WALKTHROUGH.md) English；如果你更想先理解产品定位，先看 [PRODUCT_WRAP_ZH.md](PRODUCT_WRAP_ZH.md) 中文 / [PRODUCT_WRAP.md](PRODUCT_WRAP.md) English。

## 一句话理解

MoonPub 是**本地优先的内容发布副驾驶**。它先把零散素材整理成可编辑草稿，让你在本地确认排版和预览，最后再显式推进到微信公众号草稿。

它不是自动发文机器人，也不替你做最终发表确认。

## 第一步：安装 CLI

当前推荐从 GitHub Release 下载 `v0.4.2` 二进制。

**macOS ARM64：**

```bash
curl -L https://github.com/qiaopengjun5162/moonpub/releases/download/v0.4.2/moonpub-macos-arm64.tar.gz | tar xz
sudo mv moonpub /usr/local/bin/
```

**macOS Intel：** 把文件名换成 `moonpub-macos-amd64.tar.gz`。
**Linux：** 换成 `moonpub-linux-amd64.tar.gz` 或 `moonpub-linux-arm64.tar.gz`。
**Windows：** 从 [Releases](https://github.com/qiaopengjun5162/moonpub/releases) 下载 zip，解压后把 `moonpub.exe` 加入 PATH。

验证：

```bash
moonpub --version
```

## 第二步：初始化你的文章目录

进入你的 Obsidian vault 或任意文章目录，运行：

```bash
moonpub init
```

这会生成 `moonpub.toml` 示例文件。打开它，把 `articles.root` 改成你的真实文章目录：

```toml
[articles]
root = "/Users/你的用户名/Obsidian/Articles"

[wechat]
appid = "wx..."
author = "你的公众号作者名"
theme = "geek"

[ai]
provider = "deepseek"
```

如果还没有微信公众号，可以先不填 `appid`，本地渲染和预览一样能用。
技术文章可以继续用 `geek`；终端感用 `geek-black`，系统设计用 `blueprint`，AI 实验和 Agent 工作流用 `ai-lab`。
如果不知道该选哪套主题，先跑 `moonpub layout-recipes`，它会列出主题分组、重点主题和可复制的排版配方；在 Obsidian 插件首页里也可以直接复制正文 `theme` 和封面命令。

## 第三步：创建并渲染第一篇文章

```bash
moonpub new "我的第一篇文章"
```

会在 `Articles/drafts/` 下生成 `我的第一篇文章.md`，带 frontmatter 模板。编辑它，写一段正文。

然后渲染：

```bash
moonpub render Articles/drafts/我的第一篇文章.md
```

会生成两个文件：

- `Articles/drafts/我的第一篇文章.html`：本地预览用
- `Articles/drafts/我的第一篇文章.draft.json`：微信草稿用

本地预览：

```bash
moonpub preview Articles/drafts/我的第一篇文章.md
```

这会打开系统浏览器，让你在微信编辑器之外先确认排版。

## 第四步：生成封面

```bash
moonpub cover Articles/drafts/我的第一篇文章.md --style literary
```

会生成 `Articles/drafts/我的第一篇文章.cover.html`，加 `--screenshot` 会用 Chrome 截出封面图。

## 第五步：推到微信公众号草稿（可选）

如果你还没有微信凭证，先获取它们：

1. 打开 [微信公众平台](https://mp.weixin.qq.com)，扫码登录
2. 左侧菜单 → **设置与开发** → **基本配置**
3. 复制 **AppID**（以 `wx` 开头）和 **AppSecret**（点击"重置"获取）

设置环境变量（推荐写入 `~/.zshrc` 或 `~/.bashrc`）：

```bash
export WECHAT_APPID=wx***
export WECHAT_SECRET=你的secret
```

如果你不想处理 IP 白名单，可以在 `moonpub.toml` 里设置：

```toml
[wechat]
auth_method = "cookie"
```

或在环境变量里设置 `WECHAT_AUTH_METHOD=cookie`。这样 `push` / `ship` 会复用 `moonpub login` 保存的浏览器会话来推送和配置后台，不再需要 `WECHAT_APPID` / `WECHAT_SECRET` 和 IP 白名单。

如果你使用 AppID / AppSecret 模式，并且当前机器 IP 已加入微信公众平台 IP 白名单，就可以推送：

```bash
moonpub push Articles/drafts/我的第一篇文章.md --render
```

推送成功后，文章包会自动移动到 `Articles/ready/`，并写入 `.media_id`。

## 第六步：后台辅助配置（可选）

草稿已经进微信后台，但还需要设置原创、赞赏、留言、创作来源等。运行：

```bash
moonpub login       # 首次扫码登录
moonpub configure   # 自动配置后台并发送手机预览
```

`configure` 默认会发送手机预览。首次需要指定接收微信号：

```bash
moonpub configure --to 你的微信号
```

成功后，下次不再重复输入。

## 一键全流程

熟悉后可以直接用：

```bash
moonpub ship Articles/drafts/我的第一篇文章.md --style literary
```

`ship` 会做：封面 → 渲染 → 推微信草稿 → 后台配置 → 发送手机预览。最终发表仍然需要你自己在微信后台点。

## 从飞书秒记开始

如果你当前只有一段飞书妙记转写，而不是完整文章：

```bash
moonpub --articles "/Users/你的用户名/Obsidian" intake feishu --latest --draft --preview
```

这条命令会：

1. 拉取最近一条飞书妙记
2. 保存到 `Inbox/Feishu/`
3. 生成可编辑草稿
4. 渲染本地 HTML 预览
5. 停下来等你确认

确认草稿没问题后，再单独执行：

```bash
moonpub push Articles/drafts/<草稿文件名>.md --render
moonpub configure
```

如果你已经很熟悉这条链路，想一次到位：

```bash
moonpub --articles "/Users/你的用户名/Obsidian" intake feishu --latest --draft --push
```

## 从照片素材开始

如果你在 Obsidian 里打开了一张图片，想整理一组照片：

```bash
moonpub --articles "/Users/你的用户名/Obsidian" intake photos photos/day1 --draft --preview
```

会：

1. 把照片归档到 `Inbox/Photos/`
2. 根据文件信息生成素材草稿
3. 渲染本地 HTML 预览
4. 停下来等你确认

如果需要基于照片本身的可见信息（比如照片中的文字、场景），显式加：

```bash
moonpub --articles "/Users/你的用户名/Obsidian" intake photos photos/day1 --analyze-images --draft --preview
```

限制：OpenAI provider，最多 5 张 jpg/jpeg/png/webp，单张 8 MiB、合计 20 MiB。分析结果会标注“需人工核对”，不自动推进微信。

## 用 Obsidian 插件操作

1. 在 Obsidian 安装并启用 BRAT
2. 添加 beta 插件：`https://github.com/qiaopengjun5162/moonpub`
3. 启用 `MoonPub` 插件
4. 设置里填写：
   - `MoonPub 可执行文件路径`
   - `Articles 根目录`
   - `微信预览接收人`（可选）

然后点击左侧 MoonPub 图标，打开首页工作台。首页会展示：

- 当前文件适合走哪条路径
- 工作区文章统计
- 推荐下一步
- 飞书、照片、当前文章入口

常用命令：

- `检查当前文章状态`：对当前 Markdown 跑 `check`
- `预览文章`：对当前文件跑 `preview`
- `导入最近一条飞书妙记并生成草稿预览`：停在草稿 + 本地预览
- `发布到微信公众号`：对当前文章跑 `ship`

## 发布前本地检查

在触达微信之前，先用这些命令检查：

```bash
moonpub doctor                          # 本地环境诊断
moonpub check Articles/drafts/文章.md     # 文章包状态
moonpub preflight Articles/drafts/文章.md # 发布前质量门
moonpub layout-audit 文章.html          # HTML 排版风险
```

这些命令都不调用微信 API，也不控制 Chrome。

## 安全边界与常见误解

- **不自动最终发表**：`ship` 只推进到“可人工确认发布”状态，最终发表按钮必须你自己点。
- **不绕过登录**：`moonpub login` 保存的 session 会过期，必要时需要重新扫码。
- **浏览器自动化可能软失败**：微信后台页面变化时，原创/赞赏/留言等设置可能失败，但不影响草稿推送主流程。
- **IP 白名单**：AppID/AppSecret 模式下，当前公网出口 IP 必须加白；代理或移动网络出口漂移会导致 `invalid ip` 错误。
- **AI 内容需要审查**：`write` / `expand` / `polish` / `ship --ai` 生成的内容必须人工确认。
- **照片分析需核对**：`--analyze-images` 结果会写回 Inbox 并标注“需人工核对”，不自动作为事实推进。

## 常见命令速查

| 命令 | 场景 |
|------|------|
| `moonpub init` | 初始化配置 |
| `moonpub new "标题"` | 创建文章模板 |
| `moonpub render 文章.md` | 渲染微信 HTML |
| `moonpub preview 文章.md` | 本地打开预览 |
| `moonpub cover 文章.md --style literary` | 生成封面 |
| `moonpub check 文章.md` | 检查文章状态 |
| `moonpub preflight 文章.md` | 发布前质量门 |
| `moonpub push 文章.md --render` | 推微信草稿 |
| `moonpub ship 文章.md` | 一键全流程 |
| `moonpub login` | 扫码登录 |
| `moonpub configure` | 配置微信后台并发送预览 |
| `moonpub wechat-health` | 检查浏览器登录态 |
| `moonpub doctor` | 本地环境诊断 |

## AI 功能与常见问题

### AI 功能

以下命令需要你自己的 AI provider key，默认读取 `DEEPSEEK_API_KEY`，在 `moonpub.toml` 里设置 `provider = "openai"` 后读取 `OPENAI_API_KEY`：

```bash
moonpub write "写一篇关于《活着》的读书笔记"       # 从想法生成文章
moonpub expand Articles/drafts/笔记.md             # 读书笔记展开成文章
moonpub polish Articles/drafts/草稿.md              # 润色优化
moonpub ship Articles/drafts/文章.md --ai           # 先润色再发布
```

AI 生成的内容必须人工审查，不要直接发布。

### 常见问题

**Q：推送报错 `invalid ip` 怎么办？**

去 [微信公众平台 → 基本配置 → IP 白名单](https://mp.weixin.qq.com) 添加报错信息里的 IP。如果昨天加过今天又报新 IP，先关闭会换节点的代理或固定稳定出口，再把本次报错里的 `current IP` 加入白名单，不要无限累积历史地址。

**Q：浏览器自动化某一步失败？**

微信编辑器是 live web app，UI 偶尔会变。原创/赞赏/留言等步骤软失败不影响草稿推送主流程。加 `--headed` 看具体哪一步没成功：

```bash
moonpub configure --headed
```

**Q：Chrome 找不到？**

确保系统装了 Chrome 或 Chromium。macOS 默认有，Linux 用 `apt install chromium-browser`，Windows 会自动搜 Program Files。

**Q：文章渲染后样式不对？**

- 不要用 `<style>` 标签（微信会剥离）
- 用 `moonpub render` 渲染，样式都写进 inline CSS
- `:::blockname` 拼写正确，区分大小写

## 下一步

- 三条路径完整说明：[RECOMMENDED_WORKFLOWS_ZH.md](RECOMMENDED_WORKFLOWS_ZH.md) 中文 / [RECOMMENDED_WORKFLOWS.md](RECOMMENDED_WORKFLOWS.md) English
- 插件首次体验：[FIRST_RUN_WALKTHROUGH_ZH.md](FIRST_RUN_WALKTHROUGH_ZH.md) 中文 / [FIRST_RUN_WALKTHROUGH.md](FIRST_RUN_WALKTHROUGH.md) English
- 产品定位理解：[PRODUCT_WRAP_ZH.md](PRODUCT_WRAP_ZH.md) 中文 / [PRODUCT_WRAP.md](PRODUCT_WRAP.md) English
- 命令完整参考：[USER_GUIDE.md](USER_GUIDE.md)
- 故障排查经验：[ENGINEERING_LESSONS_ZH.md](ENGINEERING_LESSONS_ZH.md) 中文 / [ENGINEERING_LESSONS.md](ENGINEERING_LESSONS.md) English
- 已知限制：[KNOWN_LIMITS_ZH.md](KNOWN_LIMITS_ZH.md) 中文 / [KNOWN_LIMITS.md](KNOWN_LIMITS.md) English
