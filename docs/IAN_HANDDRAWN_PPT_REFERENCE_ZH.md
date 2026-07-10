# Ian Handdrawn PPT 参考融合地图

这份文档记录 MoonPub 对 [helloianneo/ian-handdrawn-ppt](https://github.com/helloianneo/ian-handdrawn-ppt) 的参考结论。

结论先说清楚：

**Ian Handdrawn PPT 对 MoonPub 的价值在于“文章内容到封面 / 正文解释图的叙事规划与质量门”，不是让 MoonPub 近期变成 PPT 生成器或强依赖图像生成模型的发布工具。**

## 参考库的核心启发

`ian-handdrawn-ppt` 是一个 Codex Skill，用来把文章、课程笔记、提纲或已有材料整理成中文手绘技术解释风格的 PPT-style 页面图。它强调的不是传统 PPTX，而是：

- 先理解材料，提炼叙事结构。
- 再为每页选择语义版式。
- 统一锁定一套手绘视觉 DNA。
- 默认产出 21:9 文章封面和 16:9 正文解释图 PNG。
- 多页时生成 contact sheet，方便快速检查风格一致性和文字准确性。

对 MoonPub 来说，最值得吸收的是：

- 配图先服务内容结构，不做纯装饰。
- 一张图只表达一个主旨。
- 图中文字必须短、少、可检查。
- 封面和正文插图应按不同画幅和角色处理。
- 多图必须有统一视觉 shell，并通过 contact sheet 做整体 QA。
- 生成图可以作为可选增强，但不能成为基础发布链路的前置条件。

## 与 MoonPub 的映射

| Ian Handdrawn PPT 能力 | 可吸收点 | MoonPub 对应落点 | 当前判断 |
|---|---|---|---|
| 21:9 文章封面 | 封面要先表达文章核心隐喻 | `cover`、生活合集开篇封面、官网样张 | 值得吸收 |
| 16:9 正文解释图 | 每张正文图只解释一个观点 | 未来文章插图 / 技术解释配图 | 可选增强 |
| 叙事规划 | 先确定教学 / 说服 / 报告 / 产品解释结构 | `draft-audit`、`layout-recipes`、文章配图规划 | 中期可用 |
| 页面 archetype | 按语义选布局，不套固定模板 | 未来 `illustration-plan` 或配图建议 | 只做研究 |
| 视觉 DNA | 近白纸底、细线条、轻标记、大留白 | 生活合集封面、正文插图、插件引导图 | 可吸收审美原则 |
| contact sheet | 多图整体复查风格漂移 | 封面样张、正文插图组、官网视觉证据 | 值得吸收 |
| 输出质量门 | 内容、布局、视觉、画幅、文字逐项检查 | release 证据、视觉资产 QA | 值得吸收 |

## 最值得吸收的四件事

### 1. 配图从“装饰”改成“解释”

MoonPub 后续如果在文章中自动建议插图，不能只是为了让页面“有图”。更合理的原则是：

- 有明确概念、流程、对比、分支、分类或总结时才建议配图。
- 普通生活随笔不强行插图；真实照片优先于生成插图。
- 技术文章的正文图应解释结构，不替正文制造信息幻觉。
- 每张图只承担一个主旨，复杂内容拆成多图或留在正文。

这和公众号排版是一致的：图片应该帮助读者理解，而不是打断阅读。

### 2. 先做视觉蓝图，再生图

如果未来 MoonPub 增加文章配图建议，不应直接把全文丢给模型生图。更稳的流程是：

1. 读取文章标题、摘要、段落结构和已有图片。
2. 判断是否真的需要封面以外的正文配图。
3. 输出配图蓝图：位置、主旨、画幅、图像 brief、可见文字。
4. 用户确认蓝图后再生成或导入图片。
5. 生成后进入本地预览和视觉 QA，而不是直接 push。

这个流程能保护用户不被“看起来很 AI 的配图”拖累文章质感。

### 3. 图中文字要短、少、可验证

公众号文章里最容易翻车的是带中文字的生成图。可吸收的边界是：

- 封面标题可以在确定性 HTML/CSS 或后处理层叠加，不强求模型画准。
- 正文解释图只放短标签，不放长段落。
- 图中出现的文字必须能从蓝图里逐项检查。
- 如果生成图文字错了，优先减少文字或改用确定性叠字，不把错字图发出去。

这对 MoonPub 的封面系统尤其重要：标题、副标题、作者、合集名这类事实性文字应尽量保持可控。

### 4. 多张视觉资产要有 contact sheet QA

如果一篇文章有多张配图，或者一个合集有一组封面样张，单张看都不错也不够。需要整体看：

- 纸底是否一致。
- 色彩是否漂移。
- 标题大小是否忽大忽小。
- 图形复杂度是否突然变重。
- 封面和正文图的画幅是否真实匹配。
- 是否出现错字、假 URL、随机英文、水印或多余标签。

这可以映射到 MoonPub 未来的证据归档：视觉资产不是聊天里一句“挺好看”，而是有路径、有检查项、有剩余风险。

## 不建议近期吸收的部分

短期不要把下面能力做进 MoonPub 主线：

- 通用 PPT / PPTX 生成器。
- 默认给每篇文章自动生成多张正文插图。
- 发布前强制调用图像生成模型。
- 把整篇公众号正文做成图片。
- 把图中文字完全交给图像模型决定。
- 用大量 AI 手绘图替代真实照片记录。
- 在 v0.4.x / v0.5 阶段新增复杂 `deck`、`slides` 或课程课件命令。

原因：

- MoonPub 的主线仍是 Obsidian / 飞书 / 照片 -> 草稿 -> 本地预览 -> 微信草稿。
- 公众号正文应保持可读、可编辑、可审计的 HTML，不应退化成整页图片。
- 图像生成会带来成本、稳定性、错字、风格漂移和版权/事实边界问题。
- 生活合集优先尊重真实照片和真实记录，不应该被过度包装成课程图解风。

## 对 MoonPub 路线的建议

近期仍按这个顺序推进：

1. 先补 v0.4.2 首次体验证据和真实微信回归材料。
2. 继续稳定排版主题、封面风格和 `layout-audit`。
3. 如果要做“文章是否需要配图”的功能，先做只读 `illustration-plan` 或 `draft-audit` 建议，不直接生图。
4. 如果要生成封面或正文解释图，默认停在本地资产和预览确认，不直接 push。
5. 对生活照片类文章，真实照片优先；生成插图只用于抽象概念、流程或总结，不替代记忆素材。

一句话：

**MoonPub 可以学习 Ian Handdrawn PPT 的“内容语义 -> 视觉蓝图 -> 生成资产 -> contact sheet QA”纪律，但不能把公众号发布主线变成生图流水线。**

## 安全与边界备注

这类视觉 skill 往往会要求读取大量原始材料、生成图片、整理交付路径。MoonPub 如果参考它，应保持：

- 不复制外部 skill 代码、模板或资产。
- 不把用户文章、照片、封面样张上传到未知服务。
- 不把生成图直接写入公开仓库。
- 不把个人照片强行改造成虚构插画。
- 不编造来源、案例、数据或图片中文字。
- 不让图像生成绕过用户确认、排版审计和微信后台人工发布边界。

## 参考来源

- [helloianneo/ian-handdrawn-ppt](https://github.com/helloianneo/ian-handdrawn-ppt)
- [Ian Handdrawn PPT README](https://github.com/helloianneo/ian-handdrawn-ppt/blob/main/README.md)
- [Ian Handdrawn PPT SKILL.md](https://github.com/helloianneo/ian-handdrawn-ppt/blob/main/ian-handdrawn-ppt/SKILL.md)
- [Output And Quality Gates](https://github.com/helloianneo/ian-handdrawn-ppt/blob/main/ian-handdrawn-ppt/references/output-quality.md)
- [Slide Archetypes](https://github.com/helloianneo/ian-handdrawn-ppt/blob/main/ian-handdrawn-ppt/references/slide-archetypes.md)
- [Narrative Planning](https://github.com/helloianneo/ian-handdrawn-ppt/blob/main/ian-handdrawn-ppt/references/narrative-planning.md)
- [Visual DNA V6](https://github.com/helloianneo/ian-handdrawn-ppt/blob/main/ian-handdrawn-ppt/references/visual-dna-v6.md)
