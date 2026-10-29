# 编年史：2026-06-10 → 2026-10-29

479 次提交，4 个半月。这份编年史按阶段切成 7 段，每一段都记四件事：当时在做什么、最难的一段、翻过的车、能写成什么文章。

所有数字都可以自己复算，命令在最后。

## 数据总账

| 指标 | 数值 | 备注 |
| --- | --- | --- |
| 提交总数 | 479 | 2026-06-10 → 2026-10-29 |
| 其中 `fix` | 163 | **占 34.0%** |
| `feat` | 120 | 25.1% |
| `docs` | 84 | 17.5% |
| `refactor` | 35 | 7.3% |
| `test` | 17 | 3.5% |
| 单日最高 | 87 | 2026-06-11 |
| 源码规模 | 29,298 行 / 65 文件 | `src/` |
| 测试数 | 593 | `cargo nextest list` |
| 发布次数 | 14 | v0.1.0 → v0.4.8 |
| 首个提交 | 4,967 行 | `5a8bed0`，其中 `lib.rs` 单文件 3,286 行 |

**这张表里最重要的是 34%。** 三分之一的提交是修 bug，意味着这个项目不是设计出来的，是在微信后台的 DOM 上一次次撞出来的。

## 阶段一：一天从空目录到 v0.1.0

**2026-06-10 · 33 次提交 · v0.1.0**

首个提交 `5a8bed0` 就是 4,967 行，`src/lib.rs` 一个文件 3,286 行，包含 render、push、export、preview、radar、status 六个模块，49 个单测，0 clippy warning。

当天晚上 20:31 开始写浏览器自动化，到 22:44 已经打进去 20 个提交，全部在跟微信后台的隐藏 DOM 搏斗：

- `20:34` `feat: publish 集成 playwright-cli 后端自动化`
- `20:43` `feat: moonpub login — 自动微信快捷登录，一次扫码永久复用`
- `22:25` `fix: DOM 级登录超时检测 — 拦截微信假登录页面`
- `22:32` `fix: 首页自跳转获取动态 token — 根除 Cookie 被冲掉问题`
- `22:40` `fix: 草稿箱列表点击进入 — vid≠appmsgid 终极解决`
- `22:44` `fix: 先 Hover 卡片再找编辑按钮 — 微信隐藏式菜单`

**翻的车**：`vid ≠ appmsgid`。微信草稿箱列表里，URL 上的 `vid` 和草稿真实 ID 不是一个东西，直接按 `vid` 跳转编辑器必然 404。最后靠"先 hover 卡片唤出隐藏菜单，再点第二个按钮"绕过去。

**能写的文章**：

- 《我以为要写的是 Rust，结果在逆向一个编辑器》
- 《第一个提交 3286 行，以及为什么这是个错误》

## 阶段二：三个半小时，二十次尝试

**2026-06-11 → 06-12 · 151 次提交 · v0.3.0 → v0.3.1**

这是全项目最密的两天。06-11 单日 87 次提交，其中 20:31 到 23:57 这三个半小时里连续 20 多次尝试，只为了"点开草稿并让设置生效"这一件事。

按时间读这段日志，能完整看到一个人的思路是怎么被撞碎的：

```text
20:31  feat: publish 模块 — push 成功后自动打开后台编辑页
20:34  feat: publish 集成 playwright-cli 后端自动化
21:02  feat: 集成 vault 脚本 + publish 后端自动化跑通
21:28  feat: Rust headless_chrome 浏览器自动化 + PROGRESS 记账
22:19  feat: ship + push_article 双阶段自动化集成
22:44  fix:  先 Hover 卡片再找编辑按钮 — 微信隐藏式菜单
22:57  fix:  三策略强触卡片 — 全层Hover/链接剥离/盲点
23:04  fix:  提取 appmsgid 直接跳转编辑器 — 不依赖卡片点击
23:08  refactor: publish 切回 Playwright Node — headless_chrome evaluate 有 bug
23:19  refactor: 纯 Rust XPath 点击 — 无 JS eval, CDP DOM 原生查询
23:36  fix:  浏览器永不关闭 — Node 无限等待 + Rust fire-and-forget
23:39  fix:  跨 frame 原创/来源 + 账号名片插入
23:49  fix:  frameLocator 穿透 iframe — 所有点击定位到子 Frame
23:57  refactor: 纯 Rust CDP — find_elements + click(), 零 JS eval
```

注意 `21:28` 到 `23:08`：先用 Rust 的 `headless_chrome`，发现 `evaluate` 有 bug，切回 Node Playwright；到 `23:19` 又切回纯 Rust，改用 CDP 原生 DOM 查询绕开 `evaluate`。**一百分钟内换了两轮技术栈。**

06-12 继续打，主题变成"Vue 不认合成点击"：

```text
19:54  fix: add shadow DOM traversal to cdp_click_text
20:01  fix: restore 赞赏 + 创作来源 to original working code from git history
20:21  fix: 原创 checkbox use CDP click for 我已阅读并同意 (Vue ignores JS events)
20:35  fix: 赞赏 increase wait + retry 确定
20:46  fix: 合集 select collection with mut ok2
21:20  fix: use CDP clicks for 赞赏 + 创作来源 initial triggers (Vue ignores JS events)
```

**翻的车**：`20:01` 那条 `restore ... from git history`。意思是改崩了，从 git 历史里把之前能跑的版本捞回来。这是真实的开发状态，不是演示。

**根因（后来才彻底搞清）**：微信编辑器是 Vue 应用，JS 合成的点击事件 `isTrusted=false`，Vue 的模型直接忽略。必须由 Rust 侧通过 CDP 派发**可信鼠标事件**（先取坐标，再 `page.click(Point)`）。这一条后来写进了 `AGENTS.md` 的微信发布约束，成了项目最硬的一条规则。

**能写的文章**：

- 《Vue 不认 JS 点击：一次 CDP 可信事件的排查》
- 《一天 87 次提交，那天我什么都没做成》
- 《为什么浏览器自动化不能用单元测试覆盖》

## 阶段三：成形

**2026-06-13 → 06-30 · 142 次提交 · v0.3.2 → v0.4.2**

功能面铺开：主题系统（`11:39` theme.rs）、14 种封面、`moonpub ship` 一键链路、飞书秒记 intake、结构化 JSON 输出、Obsidian 插件 MVP、GitHub Pages 落地页。

同期开始拆 `lib.rs`：

```text
17:13  refactor: extract radar.rs from lib.rs (-1270 lines)
```

第一个提交里那个 3,286 行的 `lib.rs`，从这里开始被一块块拆出去。到 10 月，`src/` 已经是 65 个文件。

**翻的车**：`theme.rs` 连续 5 次修复同一个问题——字面量 `\n` 写进了 attribute 里（`11:42` `11:43` `11:45` `11:46` 一路修到 `11:51` 重写整个文件）。

**能写的文章**：

- 《3286 行的 lib.rs 是怎么一块块拆掉的》
- 《一个字符串转义错误，我改了五次》

## 阶段四：Cookie 模式与发布门禁

**2026-07 · 105 次提交 · v0.4.3 → v0.4.4**

两个方向同时推进。

**方向一：绕开 IP 白名单。** 微信 API 要求把服务器出口 IP 加白名单，家宽 IP 一动就 `40164 invalid ip`。解法是走 cookie 会话直接调后台接口：

```text
cdeaa6b  feat: add cookie-session push mode to bypass WeChat IP whitelist
8a321b5  fix:  use upload_mass_image for inline images so QR code gets a CDN URL
06d5465  fix:  force https for uploaded inline image URLs
```

**方向二：发布门禁。** 开始怀疑"代码写了 ≠ 能用"，于是造了一套证据体系：

```text
f94841c  feat: add release evidence status gate
c006752  feat: add strict evidence status gate
662d13a  feat: add release check gate
```

**翻的车**：后台设置"点完保存，刷新就没了"。连修六个提交才落库：

```text
e22f34a  fix(configure): make WeChat backend settings actually persist
e204f5b  fix(configure): trusted CDP click for picker confirms + leftover-dialog guard
08e3eb4  fix(configure): poll settings after reload; warn on missing qrcode path
24bf1aa  fix: WeChat draft settings persistence + cookie-mode publishing (#137)
```

**根因**：设置只存在页面内存里，不点"保存为草稿"不落库；而且保存前必须关掉残留弹窗，否则弹窗挡住保存按钮，整轮设置白做。验证方式只能是保存后 `location.reload()` 再逐项按可见性读状态——**重载后的状态才是唯一成功标准**。

**能写的文章**：

- 《点了保存，刷新就没了：微信后台设置持久化排查》
- 《我不相信"功能已完成"这四个字》

## 阶段五：整月只有 7 次提交

**2026-08 · 7 次提交 · v0.4.4 期间**

整月只有 7 次提交。不是停摆，是主力在别处，这里只处理线上暴露的问题。

这 7 条全是渲染细节，但每一条都来自**已发布文章的实测**：

```text
3d67d91  fix(render): 封面HTML不再嵌入微信正文, 修复构建tag泄漏(WEB3·DEV) D17-D19已发布文章实测
f415809  fix(render): 微信兼容代码块——弃用 <pre> 改逐行 <p>，Xcode Dark 主题，去语言标签
1d2d800  fix(render): blockquote text uses text_color for contrast on tinted backgrounds
b758b99  fix(render): render list markers inline instead of 34px leading column
```

**翻的车**：`3d67d91` 是最难堪的一条。封面 HTML 被拼进正文开头，微信编辑器剥离样式表后只剩裸 DOM，于是**已经发出去的 D17/D18/D19 三篇文章顶部，赫然印着 `WEB3 · DEV Paxon Qiao` 这行构建 tag**。读者能看到，作者事后才发现。

**能写的文章**：

- 《我的公众号文章顶部，印着三个月的构建 tag》
- 《微信会剥掉你的 CSS：20 个排版陷阱》

## 阶段六：第二春

**2026-09 · 17 次提交**

功能上做了 AI 封面（`--style ai-art`）和 5 套海报风格、5 套编辑器主题；结构上把单文件 `protocol.rs` 拆成模块：

```text
55db2e0  feat(cover): AI 封面生成能力（cover --style ai-art/cartoon/anime）
317a1ef  feat(cover): 新增 5 套高级海报封面风格（swiss/aurora/riso/noir/bauhaus）
e9bf1f8  feat(theme): 新增 5 套编辑器配色主题（monokai/dracula/nord/one-dark/gruvbox）
c4561ef  refactor(protocol): 将单文件 protocol.rs 抽取为 protocol/ 模块
9c3790a  fix(cover): AI 封面 provider 守卫前置，避免误导性报错
```

**翻的车**：`9c3790a`——没配 API key 时，报错指向了一个完全无关的地方，用户按提示去查会发现一切正常。守卫必须前置。

**能写的文章**：

- 《报错信息指错了地方，比不报错更糟》

## 阶段七：工程化收口

**2026-10 · 24 次提交 · v0.4.5 → v0.4.8**

这个月不写功能，只补债。四场硬仗：

**1. 版本漂移**（`9b11a91`）
`release_check.rs` 里硬编码了 `docs/RELEASE_GATE_v0.4.2_ZH.md`，版本一升，检查就去找一个不存在的文件。改成从 `env!("CARGO_PKG_VERSION")` 派生，永久消除。

**2. 六个依赖漏洞**（`c2caae0`）
`cargo audit` 报 6 个 RUSTSEC。逐条追到 rustls 生态，三个 crate 做 patch 级升级，零级联。

**3. release-please 自触发发版循环**（`7abde7b` → `a236059` → `47c1ebc`）
最绕的一个。合并 release PR 后，release-please 又开一个新 PR，把 355 行历史 Features 重新灌一遍。三层根因：

- `docs` 类型可见 → 发版 → 隐藏
- `chore` 类型可见 → **release 提交本身是 chore 类型，构成自指循环** → 隐藏
- `draft: true` 让新建 release 对 latest-release 查询不可见 → 改 `draft: false` + `bootstrap-sha` 兜底

**4. 覆盖率是假的**（`c1c1286`）
总覆盖率 71.99%，但拆开看：`wechat.rs` 40%、`push.rs` 43.6%、`publish.rs` 1.57%。根因不是漏写测试，是 `WechatClient` 把域名写死在每个方法里，**根本没有注入点**——原有的 15 个测试全在测 JSON 字符串解析，没有一个真的调用过 `create_draft`。

修法：加 `with_base_url` + `WECHAT_API_BASE`，写一个标准库实现的 `MockWechat` 服务，把推送管线抽成可注入函数。覆盖率 71.99% → 74.2%，`wechat.rs` 40.1% → 93.6%。

**翻的车（最值得写的一段）**：重构过程中**自己引入两个回归，都被 CI 抓到，本地全是绿的**。

- `e5ef3a4`：抽函数后凭据校验跑到了草稿校验前面 → 无草稿时报 `MissingEnvVar` 而不是 `NoDraftJson`
- `191d2f7`：只修校验顺序、把渲染留在内层 → `push --render` 在无凭据时不再产出 `draft.json`

共同根因：**本机 shell profile 导出了真实 `WECHAT_APPID`/`WECHAT_SECRET`，CI 没有。** 本地绿、CI 红，查了两轮才想到。修完之后本地验证一律加 `env -u WECHAT_APPID -u WECHAT_SECRET`。

**能写的文章**：

- 《release-please 给自己发了三次版：一个自指循环的排查》
- 《覆盖率 71.99% 是骗人的》
- 《本地全绿，CI 全红：环境变量埋的雷》

## 复算命令

```bash
# 提交总数与类型分布
git log --oneline | wc -l
git log --format="%s" | sed 's/(.*//' | awk '{print $1}' | sed 's/:$//' | sort | uniq -c | sort -rn

# 单日提交量排行
git log --format="%ad" --date=short | sort | uniq -c | sort -rn | head -12

# 首个提交的规模
git show --stat 5a8bed0 | tail -3

# 源码规模与测试数
find src -name '*.rs' | xargs wc -l | tail -1
cargo nextest list --config-file /tmp/nextest-fast-timeout.toml | grep -c 'moonpub '

# 某个阶段的完整日志
git log --format="%ad %s" --date=format:"%H:%M" \
  --since="2026-06-11 20:00" --until="2026-06-11 23:59"
```
