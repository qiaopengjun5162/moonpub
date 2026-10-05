# MoonPub 已知限制与边界

这份文档不是免责声明，而是把当前真实边界写清楚，减少“以为是 bug，实际是设计选择”的反馈。

## 微信侧限制

- 不自动最终发表：ship / push / configure 只把文章推到微信公众号草稿并辅助后台配置，最终发表按钮必须由用户自己点。
- 不绕过扫码、验证码和审核：moonpub login 保存浏览器 session，但 session 会过期，微信也可能要求重新扫码或验证。
- 浏览器自动化可能因微信后台 UI 变化而软失败：原创、赞赏、留言、创作来源、预览发送等步骤会尽量软失败，不影响草稿创建主流程。
- IP 白名单（AppID / AppSecret 模式）：需要把当前公网出口 IP 加到微信公众平台 IP 白名单；代理 / 移动网络出口漂移会导致 invalid ip 错误。
- 封面/正文图片格式：微信对图片 MIME、尺寸、内容敏感；非真实图片文件、过大或过小都可能被拒绝。

## 插件侧限制

- Obsidian 插件只是 CLI 入口，不是独立发布器；必须本地安装 MoonPub CLI 并配置可执行文件路径。
- 插件不会自动同步 Obsidian 设置到 CLI 配置；Articles 根目录和微信预览接收人需要单独在插件设置里填写。
- 社区市场尚未上架（当前暂缓），推荐通过 BRAT 或手动安装。

## 本地侧限制

- AI 写作不是核心路径：write / expand / polish / ship --ai 需要 DeepSeek / OpenAI API key，且结果需要人工审查。
- 照片视觉分析（--analyze-images）需要人工核对：图像分析结果会写回 Inbox 并标注“需人工核对”，不会自动推进到微信草稿。
- Markdown 渲染受微信编辑器限制：微信会剥离部分 HTML 标签和样式，layout-audit 可以帮助检查但无法覆盖所有编辑器行为。

### 反馈入口

遇到问题时：

1. 先跑 moonpub doctor 和 moonpub preflight <article.md>（本地不触达微信）。
2. 查看 `docs/ENGINEERING_LESSONS_ZH.md` 中文 / `docs/ENGINEERING_LESSONS.md` English 是否已记录类似现象。
3. 仍有疑问，到 GitHub Discussions 提问或开 Issue。
