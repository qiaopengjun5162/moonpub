# MoonPub 工程经验库

这份文档是 MoonPub 已解决问题的长期、可检索记录。

它服务两件事：开发前先检查是否已有结论，避免重复踩坑；对外写文章时只引用已验证的真实问题和取舍。它不记录 token、二维码、账号、私人文章、照片或完整日志。

## 使用规则

开始排障前，先按关键词搜索本文档和 `AGENTS.md`。问题修复并验证后，只要符合任一条件，就必须新增或更新一条记录：

- 影响真实用户路径、数据安全、隐私或发布结果。
- 根因不是一眼可见，未来很可能再次出现。
- 修复引入了新的边界、命令、配置约定或回归测试。

每条记录至少保留：现象和影响范围、已确认根因、最终修复、防复发约束、可公开复核的证据，以及可选的文章角度。修复尚未验证时，应放在 issue、PR 或临时排障记录中，不能写成这里的既定结论。

## 高优先级经验

### 后台配置的 ✅ 必须读落库状态，不能信点击日志

**现象：** `moonpub ship` 日志里原创 / 赞赏 / 留言全部 ✅，但微信公众号后台草稿实际"未声明 / 不开启 / 不开启留言"——所有设置都没保存上去，且连续多轮运行都误报成功。

**根因：** 四层叠加——① 步骤只验证"点击动作发出"，从不验证设置生效（弹窗遮挡时"未声明不可见"被当成已声明）；② 微信编辑器设置行同时包含"未声明"和"已声明"两个块，未选中块 `display:none` 但仍在 DOM 里，读 `textContent` 会把隐藏模板文字当成真实状态；③ 确认按钮按全局文本盲点（`cdp_click_exact_last("确定")`），会命中其它未关闭弹窗的确定（留言步骤的"确定"实际点到了原创弹窗上）；④ 配置完不保存草稿就直接关浏览器，设置只活在页面内存里。

**修复：** 全部步骤改为"状态优先 + 作用域限定 + 显式保存 + 重载复核"——原创用 `.claim__original-dialog` 容器（协议勾选在 `.original_agreement`、确定按钮在 `.weui-desktop-dialog__ft`，都不在正文 `#js_original_edit_box` 内），作者信息（可见 `input.js_author` 非空）就绪后再确认，被"作者不能为空"拦截时自动重试；赞赏在声明原创后点设置行 `.js_reward_open` 开关，在含"赞赏"的可见弹窗内点确定（原创弹窗内的赞赏区域默认 `display:none`，点那个隐藏开关无效）；留言以 `input.js_interaction_setting` 的 `checked` 为真实状态，确认限定在含"留言"的可见弹窗；新增 `step_baocun` 显式点"保存为草稿"并等保存成功提示；新增 `step_fucha` 在保存后 `location.reload()` 重载编辑器，按可见性（`#js_original_open` 可见 = 已声明、`.js_reward_open` 文本、`js_interaction_setting.checked`、`.js_claim_source_selected`）逐项复核落库状态。

**防复发：** 微信编辑器任何"状态读取"必须检查可见性（`offsetParent`，`position:fixed` 元素用 `getClientRects().length`），禁止用 `textContent` 包含匹配当状态；任何"确认"点击必须限定在具体弹窗容器内，禁止全局文本盲点；picker 类弹窗的确认按钮对 JS 合成点击（`isTrusted=false`）无响应，脚本必须只取坐标、由 CDP 发可信鼠标事件；保存前必须先关闭残留弹窗；自动化步骤的产物以"重载后的页面状态"为唯一成功标准；新增编辑器自动化步骤必须配套可见性断言的脚本测试。

**证据：** `src/publish_steps.rs` 的 `step_yuanzhuang` / `step_zanshang` / `step_liuyan` / `step_baocun` / `step_fucha` 及 `ORIGINAL_*` / `ZANSHANG_*` / `COMMENT_*` / `FUCHA_SCRIPT` 脚本测试；2026-07-26《瓦尔登湖》真实 ship（草稿 100011146）：复核输出"原创已落库 / 赞赏已落库（账户： 寻月隐君）/ 创作来源已落库（个人观点，仅供参考）/ 留言已落库"，手机预览 ret=0。

**文章角度：** "点了"和"成了"之间隔着三次验证——自动化里最贵的不是点不中，而是点错了还以为点中了。

### Cookie 模式封面必须显式上传并传给建草稿接口

**现象：** Cookie 模式下 `moonpub ship` 推送的微信草稿封面为空；带远程 `cover:` frontmatter 的书评文章上传封面时报 `ret=200002`。

**根因：** 三层叠加——`download_cover` 把 JPEG 字节硬存成 `.cover.png`，声明的 MIME 与真实字节不符被微信拒绝；`create_draft_cookie` 忽略上传结果，封面永远取正文第一个 `https://` 图片；footer 二维码改成 data URI 后正文不再有任何 https 图片，封面彻底落空。

**修复：** 下载按 Content-Type / 魔数决定真实扩展名（`.jpg` / `.png`），写入前清理旧的 `<slug>.cover.*`；`push_article_cookie` 上传封面后把 CDN URL（和 fileid）显式传给 `create_draft_cookie` 作为 `cdn_url0` / `fileid0`，为空才回退到正文首个 https 图；cookie 模式同时把内嵌 data URI 图片（footer 二维码）解码上传 CDN 并替换，避免被微信编辑器剥离。

**补充（2026-07-25 实证）：** `filetransfer?action=upload_material&type=image` 的响应形态取决于 query——不带 `writetype=doublewrite&groupid=1` 时只返回数字 fileid（`content:"100011009"`），带上后才返回真实 `cdn_url`。用 fileid 拼造 `mmbiz.qpic.cn/bizfile/<id>/0` 之类的 CDN URL 会被 `operate_appmsg` 拒绝（ret=-1 系统错误）。封面上传成功后应同时利用两个通道：`cdn_url` 填 `cdn_url0`，数字 fileid 填 `fileid0`。

**防复发：** 图片落盘扩展名必须反映真实字节，不能按目标用途硬命名；草稿封面来源必须显式传递，不能依赖"正文碰巧有 https 图"这种隐式行为；新增图片嵌入方式（data URI、本地路径）都要过 CDN 上传回归；不要凭字段名猜微信接口返回值，先打印真实响应再写解析。

**证据：** `src/ship.rs` 的 `download_cover` / `cover_extension`、`src/push_browser.rs` 的 `UploadOutcome` / `decode_data_uri` 回归测试、2026-07-25 真实《瓦尔登湖》ship 全流程（封面上传 + 二维码 CDN 替换 + 草稿 100011016 + 手机预览 ret=0）。

**文章角度：** "能跑通"和"跑得对"之间隔着隐式假设——封面来自正文第一张图这种约定，一旦正文结构变化就无声失效。

### 浏览器句柄必须活到扫码和 session 保存结束

**现象：** `moonpub login` 打开浏览器后很快报 `oneshot canceled`，扫码无法完成。

**根因：** 登录流程提前释放底层 `Browser` 句柄，CDP 会话随之取消。

**修复：** 登录路径持有活跃 `Browser`，直到扫码完成并保存 session。

**防复发：** 登录和扫码恢复路径不能只传递 `Page`；浏览器生命周期变更必须覆盖资源保活回归。

**证据：** `src/cdp.rs`、`PROGRESS.md` 的 2026-06-30 记录。

**文章角度：** 浏览器自动化失败时，问题不一定在扫码，也可能在客户端资源生命周期。

### Headless 模式不能等待用户看不见的二维码

**现象：** 登录态失效后，无界面的后台自动化等待二维码并超时，用户没有可操作的窗口。

**根因：** 可复用 session 的后台流程和需要扫码的交互登录被混在同一条路径中。

**修复：** Headless 下无法恢复 session 时快速失败，提示运行 `moonpub login` 或 `moonpub configure --headed`；临时 profile 明确说明不能复用持久 session。

**防复发：** 不可见流程不得等待人工输入；新增浏览器入口必须明确 `headed`、持久 profile 与临时 profile 的登录语义。

**证据：** `src/cdp.rs` 的 `headless_login_required_message`、`docs/USER_GUIDE.md`。

### 持久 Chrome profile 被占用不是 token 失效

**现象：** 浏览器启动报 `SingletonLock` 或 `ProcessSingleton`，容易被误判成微信登录态失效。

**根因：** 另一个 MoonPub 自动化 Chrome 窗口占用了持久 profile。

**修复：** 将底层错误收敛为可读提示：关闭已有自动化窗口，或显式使用 `--temporary-profile` 做一次性隔离验证。

**防复发：** 浏览器启动异常先区分 profile 锁、登录态失效和后台页面变化；不要把所有失败都提示为重新登录。

**证据：** `src/cdp.rs` 的 `browser_launch_error_message` 回归测试、`PROGRESS.md` 的 2026-07-04 记录。

**补充：** 2026-07-14 的真实回归还确认，另一个 `moonpub test-yulan --headed` 进程占用同一 profile 时，Chrome 有时先表现为“解析 WebSocket 地址超时”，不一定直接返回 `SingletonLock`。排障时应先检查是否已有 MoonPub 自动化进程和同一路径的 Chrome，再重试；不要立即把它归因于微信登录态。

### 微信编辑器定位优先 DOM 结构，不依赖可见文本

**现象：** 创作来源的文本被图标和空白分割，按 `textContent` 查找选项时会偶发失败或误判。

**根因：** 微信编辑器是动态页面，显示文案不是稳定的 DOM 接口。

**修复：** 打开入口使用 `.js_claim_source_desc`，选择固定来源使用 `input[type="radio"][value="4"]`，最后通过 `.js_claim_source_selected` 验证状态。

**防复发：** 自动化优先使用 DOM 结构、class 和 input value；设置步骤软失败，不能影响微信 API 草稿创建。

**证据：** `src/publish_steps.rs`、`docs/BROWSER_AUTOMATION.md`、`PROGRESS.md` 的 2026-07-03 真实回归。

**文章角度：** 为什么 MoonPub 不把浏览器自动化承诺为永远全自动。

### 配置资产与文章相对资产必须有不同解析基准

**现象：** footer 的二维码图片在本地 HTML 中存在，推送微信后却无法上传或显示。

**根因：** 文章内相对图片应以文章目录解析，而配置中的 `qrcode` / `cover` 应以 articles root 解析；两种路径被混用。

**修复：** 渲染时先把配置资产按 articles root 解析为绝对路径，再交给上传逻辑。

**防复发：** 配置路径在配置边界统一归一化；新增配置资产时补 articles root 解析测试，不能复用文章目录规则。

**证据：** `AGENTS.md` 的配置资产约束、`CLAUDE.md` 的历史问题记录。

### 微信封面 media_id 不能当作永久缓存

**现象：** 曾经可用的永久素材被删除后，旧 `thumb_media_id` 导致 `ship` 推送失败。

**根因：** 微信素材生命周期不由本地配置控制，缓存的 media_id 会失效。

**修复：** `ship` 每次生成封面 PNG 并上传，使用本次返回的新 media_id；配置值仅作为最后兜底。

**防复发：** 外部平台资源标识都视为可失效缓存，写清刷新策略和失败恢复方式。

**证据：** `src/ship.rs` 的封面上传行为和对应回归测试、`CLAUDE.md` 的历史问题记录。

### 微信 IP 白名单失败可能来自公网出口漂移

**现象：** 用户已经把前一次 `40164 invalid ip` 中的地址加入白名单，稍后重试仍失败，而且错误中的 `current IP` 已经变化。

**根因：** 微信校验的是每次 API 请求的实际公网出口；家庭宽带、移动网络和旋转代理都可能让出口变化。历史 IP 已在白名单不代表当前请求仍从该地址发出。

**修复：** `40164` 错误除输出当前 IP 外，明确提示先关闭旋转代理或使用稳定出口，再更新白名单；用户文档同步说明不能靠累积历史 IP 解决漂移。

**防复发：** 排查时分别验证默认环境和移除代理后的请求；若两者 IP 相同但与历史值不同，按公网出口变化处理。不要把浏览器登录态、微信 token 过期和 API IP 白名单混为同一类问题。

**证据：** 2026-07-15 同一 `update-draft` 先后返回 `1.80.191.120`、`1.80.191.168` 和 `117.35.173.2`；当前默认与直连均为 `117.35.173.2`。`src/wechat.rs` 的 `errcode_detected` 回归覆盖稳定出口提示。

### 封面截图不能用旧 PNG 的存在性判断本次成功

**现象：** 修改封面模板后重新执行 `cover --screenshot`，命令报告成功，但看到的仍是上一次生成的旧封面。

**根因：** Chrome headless 不会可靠覆盖已有截图文件；截图流程没有先移除旧 PNG，且执行后只检查路径是否存在，导致把旧文件误判为本次生成结果。

**修复：** Chrome 先写入同目录临时截图，确认临时文件真实生成后再覆盖正式 PNG；Chrome 失败时保留上一版可用封面。

**防复发：** 所有固定路径生成物都不能只用“执行后文件存在”证明本次成功；覆盖生成应先写独立临时产物，成功后再替换正式文件。

**证据：** `src/cover.rs` 的 `completed_capture_replaces_stale_png_and_removes_temp_file` 回归测试，以及 2026-07-15 `workflow` 封面真实 PNG 复核。

### 封面原图完整不代表公众号裁切后可用

**现象：** 900×500 原图中的流程卡片和手机模型都完整，但按公众号横版比例居中裁切后，底部素材入口和手机确认按钮被截断；方形缩略图还会切掉左对齐标题。

**根因：** 封面设计只按生成画布排版，没有为公众号列表横图、分享方图和小尺寸缩略图保留共同安全区。

**修复：** 将 `workflow` 封面的标题改为居中布局，压缩流程组件高度，把三类输入、MoonPub 核心和手机确认收进中央横版安全区。

**防复发：** 修改封面后至少检查 900×500 原图、中央 900×383 裁切、500×500 方形裁切和约 360px 缩略图；不能以原图截图作为唯一视觉验收证据。

**证据：** 2026-07-15 对 `docs/MOONPUB_INTRO_ARTICLE_ZH.cover.png` 的三种裁切实测，以及 `AGENTS.md` 的封面视觉 QA 约束。

### 手机端公众号排版不能依赖多列表格

**现象：** 项目介绍文章在本地预览中看起来正常，但手机预览里 `meta-strip` 这类两列信息块被压成表格，日期、心情和说明文字挤在一起；资料链接也像普通正文，不够醒目。

**根因：** 微信手机端可读宽度有限，横向 `<table>` 适合少量对齐，不适合承载正文级信息块。裸 URL 之前只按普通文本渲染，缺少链接视觉权重。

**修复：** 将 `summary`、`callout`、`steps`、`checklist`、`key-points`、`photo-grid`、`meta-strip`、普通 Markdown 表格，以及 `comparison` / `concept-card` / `timeline` 等解释图块改为纵向卡片或信息块；行内 Markdown 显式链接和裸 URL 都渲染为加粗高亮链接。

**防复发：** 新增 Markdown / Block / illustration 回归，断言这些手机敏感块不再输出 `<table>`；新增链接高亮回归。新增排版块时先用真实手机预览或至少检查窄宽度截图，不只看桌面本地 HTML。

**证据：** `src/markdown/blocks.rs`、`src/markdown/plain.rs`、`src/markdown/inline.rs`、`src/illustrate.rs` 的回归测试；2026-07-15 真实文章重新渲染后 `layout-audit` 通过，并成功更新同一微信草稿、完整跑通原创 / 赞赏 / 留言 / 创作来源 / 手机预览。

### 输入素材默认保守，视觉分析必须显式确认

**现象：** 照片整理若默认上传图片像素或把模型描述当事实，会造成隐私泄露和内容失真风险。

**根因：** 文件元数据整理与图像可见信息识别是两种不同的数据处理边界。

**修复：** 默认照片路径只处理路径、文件名、大小和修改时间；只有显式 `--analyze-images` 才发送受限数量的图片给 OpenAI，结果写回 Inbox 并标为需要人工核对。

**防复发：** 扩大素材数据范围的能力必须有独立确认；AI 照片描述不得自动作为已证实事实推进到微信草稿。

**证据：** `src/ai_workflow.rs`、`src/intake/photos.rs`、`docs/FIRST_RUN_WALKTHROUGH_ZH.md`。

**文章角度：** 如何把照片变成生活记录，而不让工具替生活编故事。

### Release smoke 必须在大小写敏感路径中运行

**现象：** `v0.4.2` 首次 tag workflow 的 Linux ARM64 archive smoke 因 `Archive-Smoke.md` 与 `archive-smoke.md` 不一致而失败。

**根因：** smoke 标题和后续命令的文件路径大小写不一致，依赖了大小写不敏感环境。

**修复：** 工作流统一使用小写连字符标题，并对官方下载的 macOS ARM64 资产执行无凭证 smoke。

**防复发：** 生成文件后复用实际路径或统一规范化命名；跨平台 release smoke 把大小写敏感环境当作基线。

**证据：** `.github/workflows/release.yml`、`docs/RELEASE_GATE_v0.4.2_ZH.md`、`PROGRESS.md` 的 2026-07-13 记录。

### 草稿标题含空格时自动化找不到目标草稿

**现象：** cookie 模式下 `ship` 推送成功（media_id 已生成），但 `auto_configure` 报 `draft title not found`，调试输出里目标草稿明明排在列表第一位。

**根因：** 草稿卡片 `innerText` 中标题的 ASCII 空格被微信渲染为不换行空格（或多个空白字符），`card.innerText.includes(targetTitle)` 原始字符串匹配失败；调试输出里的卡片文本经过 `\s+ → ' '` 归一化，看起来像匹配。

**修复：** `setup_editor_for_title` 的选择脚本对卡片文本和目标标题两侧都做 `replace(/\s+/g, ' ').trim()` 归一化后再 `includes` 比较（`src/cdp.rs`）。

**防复发：** DOM 文本匹配前先归一化空白；调试输出用的归一化逻辑必须和判定逻辑一致，否则调试信息会掩盖真实不匹配。

**证据：** 2026-07-26 实测 `ship` 全链路：标题 `ship 自动化验证` 归一化后 `click ship 自动化验证 btn[0] of 10`，原创/赞赏/留言/创作来源/预览全部配置成功（media_id 100011077，旧草稿自动删除）。

### 封面装饰文案不能泄漏工具内部构建信息

**现象：** 2026-08-13 用户从已发布公众号文章 URL 复查发现，`geek-black` 封面卡片正文第一屏写着 `$ BUILD NOTES` 和 `moonpub render Paxon Qiao`——工具内部构建信息直接展示给了读者。该样式是默认发布主题，影响面覆盖全部 8 篇已发布文章（08-06 arb-layers → 08-12 ai-judgment / guide-first-flow）。

**根因：** `src/cover.rs` 的 `render_geek_black_cover` 模板把终端窗口装饰标题硬编码为 `$ BUILD NOTES`、底部 meta 硬编码为 `moonpub render {author}`，本意是「终端风装饰」，实际等于把 moonpub 的构建签名泄漏进每篇正文第一屏。其余 11 种封面样式均为中性文案，只有这款中招。ship 流程设计上把封面 HTML 作为正文第一屏卡片拼入草稿（`draft.json` 的 content = 预览外壳 + 封面 HTML + 正文），因此泄漏会完整进入微信草稿并发布。

**修复：** 模板文案改为中性装饰：`$ BUILD NOTES` → `$ TECH NOTES`，`moonpub render` → `WEB3 · DEV`；同步更新 `render_geek_black_cover` 单元测试断言（`contains("TECH NOTES")` / `contains("WEB3 · DEV")` / `!contains("moonpub render")` / `!contains("BUILD NOTES")`）。

**防复发：** ① 新增封面样式时，所有装饰文案必须是通用技术语汇，禁止出现工具名、作者品牌或版本信息（可加测试断言 `!contains("moonpub")`）；② 每次 ship 后抽查已发布文章正文第一屏（浏览器打开 mp.weixin.qq.com 链接，`js_content` 开头应直接是正文标题）；③ 封面 HTML 属于「用户可见内容」，评审视角与正文一致，不能当内部实现看待。

**证据：** `src/cover.rs` `render_geek_black_cover` + `cover::tests` 断言（2026-08-13 更新）；`cargo test` 29 项全过；新二进制 `target/release/moonpub` 重渲染验证 `TECH NOTES` / `WEB3 · DEV` 各 1 处、旧文案 0 处；影响面核对：8 篇已发布文章本地 `.html` / `.draft.json` 全部含旧泄漏文案（用户决策：老文章不动，仅修复模板）。

**文章角度：** 发布流水线里的「装饰性细节」也是产品的一部分——工具把自己的构建签名烙进用户内容，等于把内部实现暴露给读者。

## 新记录模板

```markdown
### <现象或失败信息>

**现象：** <用户看到什么，影响到哪条路径。>

**根因：** <已经验证的原因。>

**修复：** <最终代码或流程修复。>

**防复发：** <测试、约束、监控或文档入口。>

**证据：** <源码 / 测试 / PR / 文档，不含敏感数据。>

**文章角度：** <可选，适合转化成哪篇公开文章。>
```

### 封面 HTML 不得嵌入微信正文——构建 tag 会泄漏给读者

**现象：** 已发布文章（D17/D18/D19 实测）正文开头出现裸文字「WEB3 · DEV Paxon Qiao」以及重复的标题 + digest 段落，位于正文第一节之前。本地预览 `.html` 和推送给微信的 `draft.json` content 均含整段 `<main class="cover" data-cover-style="…">` 封面模板。`ship` 不报任何错误。

**根因：** `render_article` 收到 `cover_html` 后无条件拼到正文开头（`format!("{cover}\n{html_body}")`）。该 `cover_html` 是完整封面模板（标题 + digest + tag 行 + 作者），不是封面图片。微信编辑器会剥离封面样式表，只保留裸 DOM，于是 tag 行（WEB3 · DEV / READING NOTES / BUILD NOTES）和重复的标题摘要直接暴露在正文顶部。封面图片本身已通过 `thumb_media_id` 单独设置，正文里这份模板是纯冗余。

**修复：** `src/render.rs` 不再拼接 `cover_html`（`let _ = cover_html;`，参数保留仅为兼容现有调用方），本地预览与微信推送正文一致地不含封面段；封面效果看 `.cover.html` / `.cover.png` 独立产物。新增回归测试 `cover_html_not_embedded_into_body`：传 `Some(cover)` 时断言 `.html` 与 `.draft.json` 均不含 `data-cover-style` / `WEB3 · DEV`。

**防复发：** 任何「把构建产物嵌入文章正文」的需求必须过一道微信剥离 CSS 后的裸文本模拟（把样式表删掉看还剩什么）；发布后核验词表必须包含封面 tag 文字（`WEB3 · DEV` / `BUILD NOTES` / `READING NOTES`）+ `data-cover-style`，命中即视为泄漏；正文里出现与 `thumb_media_id` 封面重复的标题/摘要段落 = 冗余，删。

**证据：** `src/render.rs` 的 `render_article` 与回归测试；406 个 nextest 全过；D19 文章（`2026-08-23-d19-four-direction`）重新 render 后 `.html` / `.draft.json` 均无 `data-cover-style` / `WEB3 · DEV`，正文与 footer（群二维码 base64 + 免责声明）完整。

**文章角度：** 发布管线里「本地好看的组件」和「线上可读的组件」不是一回事——构建标记、装饰性 tag 这类开发语言，进正文前必须想清楚微信会不会原样吐给读者。

## 相关记录

- `AGENTS.md`：开发时必须遵守的当前约束和模块边界。
- `CLAUDE.md`：较早期的详细排障笔记，保留作背景资料；新结论以本文为准。
- `PROGRESS.md`：按时间记录已完成的功能、验证和发布事实。
- `docs/WECHAT_REGRESSION_CHECKLIST_ZH.md`：微信真实回归执行清单，不替代根因记录。

### 标准模板结尾的视觉结构必须稳定且可替换

**现象：** 用户反馈标准模板结尾里出现不想要的灰色背景框、重复的「寻月隐君」字样、以及「公众号」字样；同时群二维码区域在只配了文字没配图时不稳定显示。

**根因：** 品牌卡片用 `<table>` + 灰色背景 + `border-radius`，在微信编辑器里呈现出明显的「框」；品牌简介里重复写了名称；`follow_image` 的 alt 文案含「公众号」三字；社群区显示条件只包含 `description` / `rules` / `qrcode`，没包含 `title` 和 `qrcode_note`，导致只写文字时整区消失。

**修复：** 品牌卡片保留 table 布局（微信编辑器会剥 flex），但去掉灰色背景和圆角；简介里不再重复名称；`follow_image` alt 改为「关注」；社群区显示条件扩展为 `title` / `description` / `rules` / `qrcode_note` / `qrcode` 任一非空；`render` 阶段增加本地 qrcode 路径不可读的终端警告。

**防复发：** footer 的视觉结构改动必须同时检查：① 微信编辑器是否会剥离关键样式；② 品牌名称是否重复；③ 任何 alt/文案中是否出现不想要的产品称谓；④ 文字-only 配置是否仍能稳定渲染；⑤ 本地二维码路径在 render 时就给出可读性反馈，而不是推到微信后台才发现缺失。

**证据：** `src/footer.rs` 的品牌卡片和社群区渲染逻辑、`src/render.rs` 的 qrcode 可读性检查、`footer::tests` 中的相关断言；2026-07-26 调整后用户确认「可以这个可以，固定下来，以后只要替换群二维码图片即可」。

**文章角度：** 模板结尾不是越丰富越好——固定结构、可替换素材、无冗余文案，才能让用户只关心自己要替换的那一张图。

### Obsidian 插件首页必须从「状态串」进化为「卡片化工作台」

**现象：** 用户打开插件首页后，看到的是一连串 ul/li 和多个 h3，不知道当前最该点什么；首次上手时容易在「工作区状态」「当前上下文」「推荐下一步」「风险边界」之间迷失。

**根因：** 早期首页只是把 `workspace --json` 的字段平铺展示，没有信息层级和主按钮；「当前文件」和「工作区概览」混在一起；动作按钮散落各处。

**修复：** 首页拆成 8 层卡片：当前文件（含 context kind / 路径 / 推荐 / 主按钮）、工作区概览（CLI 状态 / 阶段统计）、推荐下一步 + 首次建议、可用工作流、v0.4.2 证据/门禁、操作入口、触达微信提醒、常驻帮助提示。当前文章、飞书/照片结果、发布前检查、排版审计工作台也统一用同一套 `moonpub-card` + `moonpub-action-row` 样式。所有辅助弹窗（设置修复、外部输入确认、微信预览接收人）也统一加 `moonpub-homepage` class。

**防复发：** 新增插件工作台必须复用 `moonpub-card` / `moonpub-card-title` / `moonpub-action-row`；首页信息分层遵循「当前文件 → 工作区 → 下一步 → 工作流 → 门禁 → 操作 → 提醒 → 帮助」的顺序；任何动作按钮必须先关闭当前 modal 再触发下一步，避免 Notice 被遮挡。

**证据：** `obsidian-plugin/main.ts` 的 `MoonPubWorkspaceModal` / `MoonPubArticleModal` / `MoonPubIntakeResultModal` / `MoonPubPreflightModal` / `MoonPubLayoutAuditModal`、`obsidian-plugin/styles.css` 的卡片样式、`obsidian-plugin/README.md` 和 `docs/USER_GUIDE.md` 的插件说明。

**文章角度：** CLI 工具到普通用户的距离，往往只差一层「我现在该点什么」的界面。
