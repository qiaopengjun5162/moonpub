# MoonPub v0.4.1 首发演示素材记录

这份记录用于准备对外发布文章、README 截图和短视频。素材必须来自 release 二进制，而不是源码构建产物。

## 本次验证

- 日期：2026-06-24
- 二进制：`/private/tmp/moonpub-release-smoke-v041/moonpub`
- 版本输出：`moonpub 0.4.1`
- 工作目录：`/private/tmp/moonpub-launch-demo`
- 是否触达微信 API：否
- 是否读取或打印真实凭据：否

## 复现命令

```bash
rm -rf /private/tmp/moonpub-launch-demo
mkdir -p /private/tmp/moonpub-launch-demo
cd /private/tmp/moonpub-launch-demo

/private/tmp/moonpub-release-smoke-v041/moonpub --version
/private/tmp/moonpub-release-smoke-v041/moonpub init
/private/tmp/moonpub-release-smoke-v041/moonpub new "我的第一篇 MoonPub 文章"
/private/tmp/moonpub-release-smoke-v041/moonpub render "Articles/drafts/我的第一篇-MoonPub-文章.md"
/private/tmp/moonpub-release-smoke-v041/moonpub cover "Articles/drafts/我的第一篇-MoonPub-文章.md" --style literary
/private/tmp/moonpub-release-smoke-v041/moonpub check "Articles/drafts/我的第一篇-MoonPub-文章.md"
/private/tmp/moonpub-release-smoke-v041/moonpub status
```

## 生成结果

```text
moonpub 0.4.1
created /private/tmp/moonpub-launch-demo/moonpub.toml
created
  /private/tmp/moonpub-launch-demo/Articles/drafts/我的第一篇-MoonPub-文章.md
rendered
  html:  /private/tmp/moonpub-launch-demo/Articles/drafts/我的第一篇-MoonPub-文章.html
  draft: /private/tmp/moonpub-launch-demo/Articles/drafts/我的第一篇-MoonPub-文章.draft.json
cover generated
  /private/tmp/moonpub-launch-demo/Articles/drafts/我的第一篇-MoonPub-文章.cover.html
article bundle
  markdown: ok /private/tmp/moonpub-launch-demo/Articles/drafts/我的第一篇-MoonPub-文章.md
  html: ok /private/tmp/moonpub-launch-demo/Articles/drafts/我的第一篇-MoonPub-文章.html
  draft_json: ok /private/tmp/moonpub-launch-demo/Articles/drafts/我的第一篇-MoonPub-文章.draft.json
  media_id: missing /private/tmp/moonpub-launch-demo/Articles/drafts/我的第一篇-MoonPub-文章.media_id
  publishable: yes
-- drafts --
  我的第一篇-MoonPub-文章.md [rendered]
-- ready --
  (empty)
-- published --
  (empty)
```

## 可用于对外材料的素材

- 本地预览 HTML：`/private/tmp/moonpub-launch-demo/Articles/drafts/我的第一篇-MoonPub-文章.html`
- 封面 HTML：`/private/tmp/moonpub-launch-demo/Articles/drafts/我的第一篇-MoonPub-文章.cover.html`
- 安全命令输出：`--version` / `check` / `status`

## 仍未完成

- 截图文件尚未完成。Codex 内置浏览器因安全策略阻止访问本地 `file://` HTML，不能用它直接导出截图。
- 需要用普通系统浏览器或后续专门截图流程打开上述 HTML，再生成本地预览截图和封面截图。
- 真实微信草稿截图仍未完成；需要用户凭证、IP 白名单和扫码登录，不应在自动日志里打印 secret。
