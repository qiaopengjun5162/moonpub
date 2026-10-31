# 截图与录屏

放文章要用的图。命名规则见 [`../PROTOCOL.md`](../PROTOCOL.md)。

## 命名

```text
YYYY-MM-DD-主题-序号-说明.png
```

例：

```text
2026-10-29-coverage-01-before-72pct.png
2026-10-29-coverage-02-after-74pct.png
2026-10-29-mock-api-01-draft-add-request.png
```

只截四种：失败现场、修复前后对比、真实产物、关键决策点。

## 提交前必查

- 没有 `access_token`、`AppSecret`、cookie、`publish_id`
- 没有二维码（个人号、群、登录码都算）
- 没有手机号、微信号、真实姓名（作者本人除外）
- 没有含 `token=` / `ticket=` 的后台 URL
- 没有服务器 IP、内网地址

拿不准就裁掉，或者重截一张。

## 已有图片在哪

项目早期做了不少取证材料，不要重复造：

| 用途 | 位置 |
| --- | --- |
| 首次体验证据 | `docs/first-run-evidence/` |
| 发布文章配图 | `docs/assets/launch/` |
| 微信后台取证 | `moonpub configure --evidence-dir <dir>` 产出 |

新文章优先复用已有图，缺哪张再补哪张。

## 已归档图片

| 文件 | 证明什么 | 来源 |
| --- | --- | --- |
| `2026-10-30-moonpub-01-rendered-preview.png` | MoonPub 渲染出的公众号排版（标题节、引用块、表格、步骤卡片都是 Markdown 生成） | 复制自 `docs/assets/launch/01-preview.png` |
| `2026-10-30-moonpub-02-draft-in-wechat.png` | 草稿进入微信后台后的真实状态：封面已挂、字数 179、右侧原创/赞赏/留言仍待人工确认 | 复制自 `docs/first-run-evidence/wechat/wechat-draft-created.png` |
| `2026-10-30-moonpub-03-headed-settings-panel.png` | 有头浏览器里的公众号编辑器：右侧一列原创/赞赏/付费/留言/合集/创作来源就是 CDP 要点的目标 | 复制自 `docs/first-run-evidence/wechat/configure-headed.png` |
| `2026-10-30-moonpub-04-dialog-over-settings.png` | 预览弹窗浮在编辑器上，被压住的正是设置行和底部「保存为草稿」按钮——遮挡是静默的，画面里看不出点击会失败 | 复制自 `docs/first-run-evidence/wechat/preview-sent.png` |
| `2026-10-30-moonpub-05-cover.png` | 第 05 篇封面：`geek-black` 终端风，主标题「8 篇已发布文章」、副标题「正文顶部印着构建 tag」、tag `MOONPUB · 05 / 07`；底部 `$ moonpub publish` 行 | `moonpub cover --screenshot` 生成 |
| `2026-10-30-moonpub-05-body-first-screen.png` | 第 05 篇渲染后正文第一屏：开头直接是引言段，无任何封面 DOM 残骸；`WEB3 · DEV`/`Paxon Qiao` 只作为代码块示例文本出现 | Chrome headless 截 `05-*.html` 顶部 |

复制而不是移动，原位置是历史取证材料，不要破坏。
