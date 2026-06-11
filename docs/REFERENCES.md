# MoonPub 参考资料

本文档记录 MoonPub 开发过程中参考的所有项目、工具和文章。

---

## 一、微信公众号排版

### doocs/md
- **地址**: https://github.com/doocs/md
- **Stars**: 12k+
- **说明**: 开源微信 Markdown 编辑器，支持图床、AI 写作、Chrome 扩展
- **对我们启发**: CSS 主题参考、图片处理方案、多图床支持架构
- **在线版**: https://md.doocs.org/

### markdown.com.cn/editor
- **地址**: https://markdown.com.cn/editor/
- **说明**: 在线 Markdown 编辑器，微信排版工具
- **对我们启发**: 实时预览用户体验参考

### wechat-publish-template
- **地址**: https://github.com/limin112/wechat-publish-template
- **Stars**: 214
- **说明**: Claude Code Skill，橙黑赛博朋克风格公众号排版
- **对我们启发**: **Block 模板系统的直接参考**。data-block 属性、`<table>` 布局、cover/intro/callout/steps/summary/cta 模块设计

### wechat-format
- **地址**: https://github.com/lyricat/wechat-format
- **Stars**: 4.5k
- **说明**: 经典 Web 编辑器，Markdown → 微信 HTML

### mdnice
- **地址**: https://mdnice.com
- **说明**: 在线 Markdown 编辑器，支持微信、知乎、掘金多平台

### wenyan (文颜)
- **地址**: https://github.com/caol64/wenyan
- **Stars**: 998
- **说明**: 支持微信/今日头条/知乎多平台排版美化

---

## 二、微信 API 与发布

### md2wechat
- **地址**: https://github.com/geekjourneyx/md2wechat-skill
- **Stars**: 2.8k
- **说明**: Go 语言微信公众号 CLI，40+ 主题、AI 配图、草稿推送
- **对我们启发**: WeChat API 调用流程、draft.json schema、主题系统设计
- **官网**: https://www.md2wechat.cn/

### md2wechat 模板库
- **地址**: https://github.com/md2wechat/md2wechat-templates
- **说明**: 10 组开箱即用的文章骨架（技术教程/观点/周报/产品发布等）
- **对我们启发**: `:::module` 语法设计、结构化排版组件

### awesome-wechat-markdown
- **地址**: https://github.com/md2wechat/awesome-wechat-markdown
- **Stars**: 4
- **说明**: 微信公众号 × Markdown 生态工具地图
- **对我们启发**: 生态全貌、竞品分析

### md2wechat org
- **地址**: https://github.com/md2wechat
- **说明**: md2wechat 品牌主页和生态入口

### obsidian-md2wechat
- **地址**: https://github.com/geekjourneyx/obsidian-md2wechat
- **Stars**: 227
- **说明**: Obsidian 插件，一键从 Obsidian 推送草稿到微信
- **对我们启发**: MoonPub Obsidian 插件的产品形态参考

### wechatsync
- **地址**: https://github.com/wechatsync/Wechatsync
- **Stars**: 2.6k
- **说明**: 一键同步文章到多平台（知乎/掘金/公众号等）的浏览器插件

### article-tools
- **地址**: https://github.com/eternityspring/article-tools
- **Stars**: 395
- **说明**: 一套封面制作和微信公众号排版工具（纯 HTML，零安装）
- **对我们启发**: draft.md 格式、封面生成器、零安装理念

---

## 三、微信 AI 生态

### Clawbot / iLink Bot API
- **说明**: 2026 年腾讯官方开放的个人微信 Bot API
- **协议**: HTTP/JSON，长轮询，无需公网地址
- **接入域名**: ilinkai.weixin.qq.com
- **相关文章**:
  - Clawbot 工作流占座 (36kr): https://36kr.com/p/3739641264177155
  - iLink Bot API 深度解析: https://zeeklog.com/wei-xin-zhong-yu-kai-fang-guan-fang-bot-api-clawbot-cha-jian-shen-du-jie-xi-ai-kai-fa-zhe-de-xin-ji-yu-10
  - cc-weixin (200 行桥接器): https://blog.csdn.net/kilohester/article/details/161430731

### 微信 AI 生态开放 (2026.06)
- **说明**: 微信公开课官宣面向开发者提供接入微信 AI 生态能力
- **来源**: 澎湃新闻 2026-06-08

---

## 四、AI 写作与文字处理

### Humanizer-zh
- **地址**: https://github.com/op7418/Humanizer-zh
- **说明**: 中文去 AI 味 Codex Skill
- **对我们启发**: **moonpub humanize 命令的直接参考**。20+ 规则：填充语、AI 词汇、排比打破、破折号、过度修饰、通用结论
- **来源**: 翻译自 blader/humanizer，参考 hardikpandya/stop-slop

### stop-slop
- **地址**: https://github.com/hardikpandya/stop-slop
- **Stars**: 9.8k
- **说明**: AI 文本痕迹去除 Skill 文件（Humanizer-zh 的原始参考源）

### dbskill
- **地址**: https://github.com/dontbesilent2025/dbskill
- **说明**: 中文内容诊断工具 — 选题、商业表达、hook、标题、爆款拆解

### content-research-writer
- **地址**: https://github.com/ComposioHQ/awesome-codex-skills/tree/master/content-research-writer
- **说明**: 完整写作流程 Skill — 选题→资料→提纲→引用→初稿

### NotebookLM Claude Code Skill
- **地址**: https://github.com/PleasePrompto/notebooklm-skill
- **说明**: 基于已有资料库写作的 Skill，适合行业研究和深度文章

### khazix-skills
- **地址**: https://github.com/KKKKhazix/khazix-skills
- **说明**: 公众号长文和万字研究报告 Skill

---

## 五、配图与视觉生成

### ian-xiaohei-illustrations
- **地址**: https://github.com/helloianneo/ian-xiaohei-illustrations
- **说明**: 中文文章配图 Skill，"小黑"风格插画
- **对我们启发**: MoonPub 配图系统参考 — 从文章观点/情绪/隐喻生成插画

### guizang-social-card-skill
- **地址**: https://github.com/op7418/guizang-social-card-skill
- **说明**: 小红书图文 + 公众号封面生成 Skill
- **对我们启发**: 文章→多张卡片分发，封面模板参考

### guizang-ppt-skill
- **地址**: https://github.com/op7418/guizang-ppt-skill
- **Stars**: 16k
- **说明**: AI 生成 HTML 幻灯片，编辑杂志和瑞士风格布局
- **对我们启发**: HTML 模板渲染 → 配图/封面生成的可行路径

### baoyu-skills
- **地址**: https://github.com/JimLiu/baoyu-skills
- **说明**: 中文创作者的视觉工具箱 — 封面图/信息图/结构图/图解

### html-anything
- **地址**: https://github.com/nexu-io/html-anything
- **说明**: Markdown/文案 → HTML 页面/海报/卡片/PNG

---

## 六、浏览器自动化

### Playwright MCP
- **地址**: https://github.com/microsoft/playwright-mcp
- **Stars**: 7.3k
- **说明**: 微软官方 Playwright MCP Server
- **相关文章**: 2026 年 AI Agent 浏览器操控实践

### playwright-cli
- **地址**: npm `@playwright/cli`
- **说明**: Playwright 的 CLI 封装，Bash 通道操作，比 MCP 省约 4 倍 Token
- **MoonPub 用途**: 微信后台浏览器自动化的临时方案
- **相关文章**: "用 playwright-cli 实现前后端全链路联调" (费曼的技术工坊)

### Agent-Reach
- **地址**: https://github.com/Panniantong/Agent-Reach
- **Stars**: 26.2k
- **语言**: Python
- **说明**: AI Agent 跨平台互联网读写脚手架。封装 twitter-cli、yt-dlp、rdt-cli、gh CLI、Jina Reader 等工具，让 Agent 免费搜索和阅读 Twitter/Reddit/YouTube/GitHub/B站/小红书/LinkedIn/RSS
- **对我们启发**: `moonpub fetch` 和 `radar scrape` 的数据源接入方案 — 用成熟 CLI 工具替代 API 付费，零配置可插拔架构

### Obscura
- **地址**: https://github.com/h4ckf0r0day/obscura
- **Stars**: 15k
- **语言**: Rust
- **说明**: AI Agent 和网页抓取的 headless 反检测浏览器
- **MoonPub 用途**: 替代 playwright-cli，实现 `moonpub backend` 纯 headless 微信后台自动化

### CloakBrowser
- **地址**: (Python Package)
- **说明**: 58 个 C++ Chromium 源码补丁的反检测浏览器
- **相关文章**: Playwright/CloakBrowser/BrowserAct 三工具横评

### BrowserAct
- **地址**: https://github.com (相关仓库)
- **Stars**: 2.2k
- **说明**: 面向 AI Agent 的浏览器自动化平台（索引化交互 + 反检测 + CAPTCHA）

---

## 七、Rust 与 AI 开发

### rust-template
- **地址**: https://github.com/qiaopengjun5162/rust-template
- **说明**: 项目开发模板参考（Rust 全栈 + WASM 模式）

### AI Coding 实践
- **相关文章**: PingCAP/Data & AI Meetup 分享实录 — "从古法编程手艺人进化到 AI 软件工厂厂长"
- **核心观点**: Rust 编译器是 AI 最强的 oracle（判定者），形成短反馈回路

---

## 八、相关工具清单（awesome 精选）

| 类型 | 工具 | 说明 |
|------|------|------|
| Web 编辑器 | doocs/md, mdnice, wechat-format | 浏览器端排版 |
| CLI | md2wechat, wechatsync | 命令行发布/同步 |
| Agent Skill | Humanizer-zh, dbskill, content-research-writer | Codex/Claude Code 写作 |
| 视觉 | guizang-ppt, ian-xiaohei, baoyu-skills | 配图/封面/PPT |
| 浏览器 | playwright-cli, Obscura, CloakBrowser | 自动化 |
| MCP | wenyan-mcp, md2wechat-mcp-server | AI 直连发布 |

---

## 九、排版工具与编辑器

### 135编辑器
- **地址**: https://www.135editor.com/
- **说明**: 微信公众号排版编辑器，提供大量模板
- **对我们启发**: 读书笔记模板结构、封面图规范（900x383px 2.35:1）

### 壹伴
- **地址**: https://yiban.io/
- **说明**: 公众号运营工具，文章开头结尾方法论
- **对我们启发**: SCQA 开头法、结尾 CTA 设计

### 小墨鹰编辑器
- **地址**: https://www.xmyeditor.com/
- **说明**: 2026 最新微信公众号排版指南
- **对我们启发**: 配色方案、字体大小规范

---

## 十、内容创作 Agent Skills

### baoyu-skills
- **地址**: https://github.com/jimliu/baoyu-skills
- **说明**: 公众号/文章自动配图、信息图、知识漫画生成 + 一键发布到微信/X
- **对我们启发**: 自动配图 pipeline、多平台一键发布

### claude-blog
- **地址**: https://github.com/AgriciDaniel/claude-blog
- **说明**: 全流程博客/长文写作套件（30+ 子 Skills + 5 Agent），5-gate 质检，SEO + AI 引用双优化，10 分钟出稿

### awesome-agent-skills
- **地址**: https://github.com/VoltAgent/awesome-agent-skills
- **说明**: 1000+ 精选 Agent Skills 合集，内容创作类超多，一站式安装

### awesome-claude-skills (ComposioHQ)
- **地址**: https://github.com/ComposioHQ/awesome-claude-skills
- **说明**: content-research-writer 等，研究+大纲+写作+引用+迭代反馈

### social-media-skills
- **地址**: https://github.com/blacktwist/social-media-skills
- **说明**: 社交媒体内容全套（帖子、线程、钩子、复用、分析），教 AI 懂你的声音和平台算法

### alirezarezvani/claude-skills
- **地址**: https://github.com/alirezarezvani/claude-skills
- **说明**: content-creator 等营销写作 Skills，博客、SEO、Landing Page 高效生成

---

## 十一、用户分享的参考链接

### Twitter/X 参考推文
- wsl8297 教程: https://x.com/wsl8297/status/2063516310798250395
  - 内容: Claude Code + Obsidian 构建推特内容素材库完整教程
  - 已保存到: `docs/TWITTER-CONTENT-SYSTEM.md`
- Smartpigai 工具推荐: https://x.com/Smartpigai/status/2063095358075597009
  - 内容: 高效获取热点的 13 个工具推荐（NewsNow、今日热榜等）

### 用户实际文章
- 寻月阁 加群引导文章: https://mp.weixin.qq.com/s/L7pQNQyWQwHwXmmuzRlXrA
  - 结尾模板参考：社群介绍 + 规矩 + 加群方式 + 结尾金句

### 微信后台模板
- 模板管理: https://mp.weixin.qq.com/cgi-bin/appmsgtemplate
  - 模板类型：封面模板、书本模板、分隔栏模板、结尾模板
  - 需手动在后台应用，MoonPub 目标自动化

---

> 最后更新: 2026-06-11
