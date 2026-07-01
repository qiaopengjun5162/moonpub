# MoonPub Obsidian Plugin

这个插件不是独立发布器，而是 **Obsidian 里的 MoonPub 入口**。

它的作用是：

- 在 Obsidian 里直接调用本地 `moonpub`
- 对当前打开的 Markdown 文件执行常用操作
- 减少你在终端里手敲命令的次数

它当前仍然是实验性入口，不是一个全自动发布机器人。

## 当前支持的命令

安装后，在 Obsidian 里按 `Cmd/Ctrl + P`，可以看到：

- `发布到微信公众号`
- `预览文章`
- `AI 润色后发布到公众号`

这些命令实际调用的是本地 `moonpub`：

- `预览文章` -> `moonpub preview <当前文件>`
- `发布到微信公众号` -> `moonpub ship <当前文件>`
- `AI 润色后发布到公众号` -> `moonpub ship --ai <当前文件>`

## 产品边界

这个插件遵守和 MoonPub CLI 一样的边界：

- 本地预览不需要微信凭证
- 真正推送会触达微信 API
- 真正发布前仍然需要用户自己检查微信草稿
- 不绕过扫码、验证码、平台审核或最终人工确认

所以更准确地说，它是：

**Obsidian 中的本地发布副驾驶入口。**

## 安装方式

### 1. 先安装 MoonPub CLI

先确保终端里能运行：

```bash
moonpub --help
```

### 2. 复制插件目录

把当前仓库中的 `obsidian-plugin/` 复制到你的 vault：

```text
.obsidian/plugins/moonpub/
```

### 3. 构建插件

在插件目录中运行：

```bash
npm install
npm run build
```

这一步会安装 `esbuild`、`typescript` 和 `obsidian` 开发依赖；如果你跳过 `npm install`，`npm run build` 会直接失败。

### 4. 在 Obsidian 中启用

打开：

- `设置`
- `第三方插件`
- 启用 `MoonPub`

## 推荐使用顺序

第一次使用插件时，推荐这样走：

1. 先打开一篇 Markdown 文章
2. 先执行 `预览文章`
3. 确认本地排版没问题后，再执行 `发布到微信公众号`

不要一上来就直接发。

## 当前限制

- 目前只有桌面版可用：`manifest.json` 已设置 `isDesktopOnly: true`
- 现在还没有独立设置面板
- 主要依赖系统里已经装好的 `moonpub`
- 预览走本地 CLI；发布仍然依赖微信凭证、IP 白名单和 Chrome 自动化链路

## 后续方向

这个插件后面更适合往这些方向走：

- 展示当前文件的 MoonPub 状态
- 调用 `capabilities --json` 发现能力和风险提示
- 区分本地命令与会触达微信的命令
- 明确展示“当前命令会不会联网、会不会打开 Chrome”

但当前阶段，最重要的是：

**先把它做成一个可靠、好理解的 Obsidian 入口。**
