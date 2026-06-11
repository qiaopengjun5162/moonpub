# Claude Code + Obsidian 构建推特内容素材库

> 来源: wsl8297 推文教程
> 保存时间: 2026-06-11

## 核心流程

捕捉素材 → 整理来源卡片 → 提炼观点 → 生成推文/Thread → 人工校对发布 → 记录复盘

## 工具

- **Obsidian**: Markdown 笔记，素材库/选题库/草稿库
- **Claude Code**: 终端中读取编辑素材库，批量整理、分类、生成草稿

## 目录结构

```
X-Content-Library/
├── 00-Inbox/                 # 临时收集区
├── 01-Sources/               # 来源素材
├── 02-Ideas/                 # 选题和观点
├── 03-Drafts/                # 短帖和 Thread 草稿
├── 04-Published/             # 已发布内容和复盘
├── 05-Topics/                # 长期关注主题
├── 06-People/                # 作者/博主/品牌
├── 07-Assets/                # 截图/封面/附件
├── 08-Templates/             # Obsidian 笔记模板
├── 09-Dashboards/            # 内容看板
├── 99-Archive/               # 归档
├── .claude/skills/           # 可复用工作流
└── CLAUDE.md                 # 长期工作规范
```

## Skills 设计

| Skill | 功能 |
|-------|------|
| `/capture-source` | 链接/摘录 → 来源素材卡片 |
| `/develop-idea` | 素材 → 观点卡片（3 角度） |
| `/draft-thread` | 观点 → Thread 草稿（稳健+张力版） |
| `/weekly-review` | 周复盘 + 下周选题推荐 |

## 关键原则

1. 来源、观点、草稿分开——不同职责不同文件
2. AI 负责机械劳动，人负责最终判断
3. 不要自动发布到任何平台
4. 素材库价值不在存了多少，在于能否反复转化成内容
5. 每增加一个插件/自动化，增加维护成本

## 素材处理工作流

**每天**: 快速捕捉到 Inbox，写最少信息
**每2-3天**: 批量整理 Inbox → 来源卡片 → 观点建议
**每周**: `/weekly-review` → 挑3-5个选题
**写作时**: 来源 → 观点卡片 → 草稿 → 人工修改 → 发布复盘
