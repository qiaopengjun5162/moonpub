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
