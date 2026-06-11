# 高效获取热点的 13 个工具

> 来源: https://x.com/Smartpigai/status/2063095358075597009
> 作者: Smartpig (@Smartpigai)

我发现大多数人获取热点的方式都太低效了。

刷微博、刷知乎、刷X、刷B站……
一天花几个小时，却不知道真正重要的事情发生了什么。
后来我把工具精简到了 13 个：

1. **NewsNow** — 全网热榜聚合，一页看完微博、知乎、B站、V2EX
   https://github.com/ourongxing/newsnow

2. **今日热榜** — 收录最全的中文热榜导航
   https://tophub.today

3. **SoPilot** — 专门追踪 X 上的爆款推文和热门讨论
   https://sopilot.net

4. **DailyBrief** — AI 自动生成每日科技与 AI 简报
   https://github.com/leiting-eric/DailyBrief

5. **Trends Hub** — 聚合国内外数十个平台热榜，支持 API 和 MCP
   https://github.com/bytesfly/trends-hub

6. **RSSHub** — 把几乎任何网站变成 RSS 信息源
   https://github.com/DIYgod/RSSHub

7. **FreshRSS** — 最强开源 RSS 阅读器之一，自建信息中心必备
   https://github.com/FreshRSS/FreshRSS

8. **awesome-ai-news** — 收录 AI 新闻、资讯和聚合工具大全
   https://github.com/taielab/awesome-ai-news

9. **ReadYou** — 颜值很高的开源 RSS 阅读器
   https://github.com/Ashinch/ReadYou

10. **Glance** — 打造属于自己的热点监控仪表盘
    https://github.com/glanceapp/glance

11. **Hacker News** — 全球开发者每天都在看的技术新闻源
    https://github.com/HackerNews/API

12. **Lobsters** — 高质量程序员社区，信噪比远高于大部分论坛
    https://github.com/lobsters/lobsters

13. **GitHub Trending** — 发现正在爆发的新项目和技术趋势
    https://github.com/trending

---

现在每天早上花 5 分钟：
- 知道 AI 圈发生了什么
- 知道 GitHub 在火什么项目
- 知道 X 上在讨论什么
- 知道产品圈和开发者圈的新趋势

信息差的本质，不是知道得更多。而是更早知道重要的事。

---

## 对 MoonPub 的启发

这些工具可作为 `radar scrape` 的数据源：
- **Trends Hub** 有 API 和 MCP 接口，可直接对接
- **RSSHub** 可为任何平台生成 RSS 源
- **NewsNow** 开源热榜聚合，可参考其架构
- **tophub.today** 中文最全热榜导航
