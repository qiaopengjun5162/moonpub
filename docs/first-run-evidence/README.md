# MoonPub First-Run Evidence

这个目录是首次体验证据的统一归档位。

当前只收这 3 类证据：

- `homepage/`：插件首页工作台
- `feishu/`：飞书导入到草稿与本地预览
- `photos/`：照片导入到草稿与本地预览

目录已经按这 3 类固定下来：

- `docs/first-run-evidence/homepage/`
- `docs/first-run-evidence/feishu/`
- `docs/first-run-evidence/photos/`

正式取证前，先按 [RUNBOOK_ZH.md](RUNBOOK_ZH.md) 走一遍。
它规定了每条路径该截图哪些节点、怎样判断通过，以及哪些敏感信息不能入库。

当前证据状态：

| 路径 | 必需证据 | 当前状态 |
|------|----------|----------|
| 插件首页 | `homepage-workspace.png` / `homepage-context.png` | 待补真实截图 |
| 飞书首次体验 | `feishu-home-entry.png` / `feishu-result-modal.png` / `feishu-draft-opened.png` | 待补真实截图或录屏 |
| 照片首次体验 | `photos-image-opened.png` / `photos-result-modal.png` / `photos-draft-opened.png` | 待补真实截图或录屏 |

这些文件缺失时，只能说“代码、文档和测试已到位”，不能写成“真实首次体验已经完全打通”。

建议文件名保持和 `docs/FIRST_RUN_EVIDENCE_CHECKLIST_ZH.md` 一致，例如：

- `homepage/homepage-workspace.png`
- `homepage/homepage-context.png`
- `feishu/feishu-home-entry.png`
- `feishu/feishu-result-modal.png`
- `feishu/feishu-draft-opened.png`
- `photos/photos-image-opened.png`
- `photos/photos-result-modal.png`
- `photos/photos-draft-opened.png`

敏感信息要求：

- 不要提交 `.env`、token、cookie、二维码、AppSecret
- 如果截图里包含本地用户名、路径或私人图片，请先手动裁切

每次补证据时，同时更新同目录下的 `NOTES.md`。

如果只是先占位、还没有真实截图，也不要伪造图片。保留各目录下的 `README.md` 即可，等真实取证时再把文件补进去。
