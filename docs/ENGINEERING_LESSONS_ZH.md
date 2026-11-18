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

### footer 关注图本地文件必须内联成 data URI，不能裸写相对路径

**现象：** 所有文章的公众号 footer「关注」图在手机预览里裂图不显示，本地 HTML 里 `<img src="wechatmoonpost.jpg">` 正常（浏览器能按相对路径解析），微信编辑器只认绝对 URL 或 data URI。

**根因：** `footer.rs` 渲染 qrcode 时有 `local_to_data_uri` 本地转 base64 逻辑，`follow_image` 分支却把配置值原样写进 `src`——`wechatmoonpost.jpg` 这类裸相对路径在微信编辑器里无法解析，被剥成裂图。qrcode 之所以一直正常，是因为 `render.rs` 先把它 join 成 articles root 绝对路径、`footer.rs` 再转 data URI；follow_image 两条链都缺失，配置里还写着不带 `social/wechat/` 前缀的裸文件名，属于"两处代码不一致 + 配置路径不规范"叠加。

**修复：** ① `render.rs` 对 `follow_image` 同样做 articles-root 相对路径解析（与 qrcode 一致）；② `footer.rs` 对非 URL 的 `follow_image` 走 `local_to_data_uri` 内联，空结果跳过 img 标签（与 qrcode 一致）；③ `moonpub.toml` 把 `follow_image` 改成带前缀的 `social/wechat/wechatmoonpost.jpg`。

**防复发：** footer 任何图片字段（qrcode / follow_image / 未来新增）只允许三种值——http(s) URL、本地绝对路径、相对 articles root 的路径；本地文件一律内联 data URI。footer 新增图片字段时 grep 检查是否都过 `local_to_data_uri`。ship 前核对 HTML 里 `alt="关注"` / `alt="群二维码"` 的 src 是否以 `data:` 开头。

**证据：** commit `5bfaa1d`（footer.rs + render.rs + 配置），2026-09-03 真实 ship（media_id 100012455）：渲染 HTML 含 2 张 data URI（qrcode 299KB png + wechatmoonpost 53KB jpeg，md5 与源文件一致），手机预览 ret=0，关注图正常显示。

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

### 无序列表渲染成两列表格，微信端首列显示为空列

**现象：** 2026-09-09 用户在手机预览里看到文章列表（「正式 release / 安装脚本 / CLI…」）前面空出一列灰色竖条，问「为什么这种表格的前面要空一列」。

**根因：** `src/markdown/plain.rs` 的 `render_list` 把 markdown 无序/有序列表渲染成 `<table>` 两列：首列 `width:34px` 放圆点/编号 span（`text-align:center`），次列放文本。微信端纯样式 span（无文字内容的圆点）不渲染，首列就变成 34px 空列 + 斑马底色，手机上看起来就是「表格前面空一列」。

**修复：** 列表行改为单格布局——标记（圆点/编号）内嵌在文本同一格：`<tr><td style="padding:8px 12px 8px 8px;...">{marker}{text}</td></tr>`，marker 加 `margin-right`。删除独立 34px 首列。

**防复发：** ① 列表/标记类内容禁止用「独立首列 + 样式标记」的两列表格结构，标记必须与内容同格；② 新排版结构 ship 后抽查生成的 HTML：grep `width:34px` 应为 0（列表型）；③ 手机预览（用户真机）是最终验收。

**证据：** `src/markdown/plain.rs` `render_list`（2026-09-09 修复）；`target/release/moonpub` 重渲染 herdr-runtime-2026-09-08：`width:34px` 列表行 0 处、单格列表行 36 处；微信手机预览 ret=0。

### release-please 把自身发版提交当作可发布变更，导致链式发版

**现象：** 2026-10-29 推送一个纯文档提交后，release-please 立刻开出 `chore(main): release 0.4.6`（PR #139），CHANGELOG 里只有一条 `Documentation`；把 `docs` 设为 hidden 并合并 0.4.6 之后，又立刻开出 `chore(main): release 0.4.7`（PR #141），且 CHANGELOG 被灌入 382 行历史 Features。发版变成无限链条。

**根因：** `.release-please-config.json` 的 `changelog-sections` 把 `docs` 与 `chore` 都列为可见类型，而 release-please 的规则是「任何非 hidden 类型的提交都算可发布变更」。合并 release PR 产生的提交恰好是 `chore(main): release X.Y.Z (#N)`，于是「发版 → 产生 chore 提交 → 再发版」形成自指闭环。PR #141 的 Maintenance 分组里确实出现 `* **main:** release 0.4.6 (#140)` 这条自引用记录。

注：382 行历史 Features 被重灌的具体原因尚未定位（`v0.4.6` tag 确认在 HEAD 上、其后 0 提交）；隐藏 `chore` 之后未再复现，仍按未定论处理。

**修复：** 给 `docs` 与 `chore` 两个 section 加 `"hidden": true`（提交 `7abde7b`、`a236059`）；同日再补 `build` 与 `ci`（提交见 `.release-please-config.json` 当前状态）。此后只有 feat / fix / perf / refactor / revert 会触发版本递增。依赖安全升级若需要发版，改用 `fix(deps)` 而不是 `chore(deps)`。

**第四层（2026-10-29 补）：** 修完上面三层后，一个 `ci:` 提交又触发了 `release 0.4.9`（PR #145），CHANGELOG 里只有一条 `CI / Deployment`。说明「可见类型」这个坑不是一次修完的，而是每新增一个类型都要重新判断一次。PR #145 本身是干净的（只有一行、没有重灌历史，证明 `draft: false` + `bootstrap-sha` 的修复生效），但**只改 CI 也升版本号会把版本推高**，因此把 `build` / `ci` 一并 hidden，PR 关闭不发版。判定原则收敛为一句话：**只有用户能感知的变更才发版**，基础设施类（docs / chore / test / style / build / ci）一律 hidden。

**防复发：** ① `changelog-sections` 里任何**不该**触发版本递增的类型（docs / chore / test / style / build / ci）必须 hidden，新增类型时同步判断；② 合并 release PR 后必须确认「没有新的 release PR 生成」再进入下一轮工作；③ 判断 release-please 是否异常，最直接的信号是 CHANGELOG 增量里出现 `release X.Y.Z` 这类自引用条目；④ 每次推送后顺手看一眼 `gh pr list --state open`，出现非预期的 release PR 就地关闭并回查提交类型。

**证据：** `.release-please-config.json`；PR #139 / #140 / #141（均已关闭或合并）；release-please run `40867017954` conclusion=success 且 `gh pr list --state open` 为空；v0.4.6 release：`https://github.com/qiaopengjun5162/moonpub/releases/tag/v0.4.6`。

**文章角度：** 自动化里的自指陷阱——当一个系统的输出会再次成为它的输入时，一条看似无害的「可见」配置就能把发版变成死循环。

### 本地有真实凭据，测出来的是「本机能跑」，不是「CI 能跑」

**现象：** 2026-10-29 重构 `push` 时把「渲染草稿」挪到了「读取凭据」之后。本机 `cargo nextest run` 593 个测试全绿，推送后 CI 的 `test` job 立刻挂在 `push_auto_render_creates_draft_json`：`push --render` 在无凭据环境下不再产出 `draft.json`。

**根因：** 本机 shell profile 里导出了真实的 `WECHAT_APPID` / `WECHAT_SECRET`，`push_auto_render_creates_draft_json` 走的是 `push_article`，它会先读凭据再渲染；凭据存在时渲染成功，测试就过了。CI 没有这两个变量，同一段代码直接返回 `MissingEnvVar`，渲染根本没执行。**测试断言的是结果，而结果被本机环境变量偷偷满足了。**

**修复：** 把「确保 `draft.json` 存在」整体前移到 `push_wechat_draft` 的最前面：缺草稿且未要求渲染 → `NoDraftJson`；要求渲染 → 先渲染落盘；两者都不依赖凭据（提交 `e5ef3a4` 之后一版）。

**防复发：** ① 本地验证必须**清掉凭据**跑一次：`env -u WECHAT_APPID -u WECHAT_SECRET cargo nextest run`，这才是 CI 等价环境；② 任何依赖环境变量存在与否的分支，测试里要用 `env -u` 或显式注入来固定前提，不要依赖「本机正好有」；③ 结论以 CI 为准，本地绿只说明「本机能跑」。

**证据：** CI run `40892503877`（`test` / `coverage` 双红，`push::tests::push_auto_render_creates_draft_json` panic at `src/push.rs:597`）；`env -u WECHAT_APPID -u WECHAT_SECRET cargo nextest run` → 593/593 通过；`docs/TEST_COVERAGE_ZH.md`。

**文章角度：** 测试的可信度取决于环境是否被固定——当本机「正好有」某个变量时，绿灯测的是机器，不是代码。

### CI 直接继承 `.cargo/config.toml` 的 target-cpu=native，rustc 以 SIGILL 崩溃

**现象：** 一个只改 Markdown 和脚本的提交，`build` 工作流的 `coverage` job 报错 `rustc ... (signal: 4 (SIGILL) (core dumped))`，`test` job 同时全绿。同一份代码本地和上一次 CI 都没有这个错误。

**根因：** `.cargo/config.toml` 为本地 macOS 开发写了 `[build] rustflags = ["-C", "target-cpu=native"]`。Cargo 的 `RUSTFLAGS` 环境变量优先级高于该配置，但 `test` / `coverage` 两个 job 都没有设置它，等于在 GitHub 的 x86_64 runner 上按编译机的 CPU 特性生成指令。runner 的 CPU 型号不固定，一旦编译期和运行期的 CPU 特性对不上就会出现非法指令；`windows-smoke` 早已用 `env: RUSTFLAGS: ""` 规避，另两个 job 漏了。

**修复：** `test` 与 `coverage` job 都补 `env: RUSTFLAGS: ""`，与 `windows-smoke` 保持一致，让 CI 构建走可移植指令集；`.cargo/config.toml` 保留，只服务本地开发。

**防复发：** 新增任何需要编译或运行 Rust 的 CI job，必须显式声明 `RUSTFLAGS: ""`，不能依赖工作流文件里其它 job 的写法；判断标准是"这个 job 会不会编译 Rust"，会就必须写。此外不要用"只改了文档所以我的 job 不会失败"来跳过排查——本次失败与代码改动无关，是 job 自身配置缺陷被暴露。

**证据：** run `40896232282` 的 `coverage` job 日志（`SIGILL`、`exit status: 101`）；同 run 中 `test` 与 `windows-smoke` 均为 success；修复提交见 `.github/workflows/build.yml` 两个 job 的 `env` 段。

**文章角度：** "只改了文档，CI 却红了"——一次和代码无关的失败，暴露的是 CI 里三个 job 的配置不一致。

### 中文标题生成 ai-lab 封面直接 panic：字符串不能按固定字节切片

**现象：** 给文章起一个正常中文标题（`title: "A一二三四五六七八"`）并选 `ai-lab` 封面样式，`moonpub cover <article.md> --style ai-lab` 直接崩溃退出码 101，stderr 为 `thread 'main' panicked at src/cover.rs:931:16: end byte index 15 is not a char boundary; it is inside '五' (bytes 13..16 of string)`，不产出封面 HTML。同样标题换 `dark` 等其它样式正常，纯英文短标题也正常，因此只在「中文长标题 + ai-lab」这条真实发布路径上暴露。

**根因：** `render_ai_lab_cover` 把转义后的标题按**字节**折行：`else if title.len() > 18 { (&title[..15], Some(&title[15..])) }`。UTF-8 里中文 3 字节、emoji 4 字节，固定字节偏移 15 不保证落在字符边界，切片必然 panic；`title.len() > 18` 用字节数当"标题是否过长"，六个汉字（18 字节）就被误判为长标题。更深一层：`title` 传入前已被 `escape_html`，含 `&amp;` 等 ASCII 实体，切点即使没 panic 也可能把实体切成 `&am` + `p;` 产出破损 HTML。同样根因还存在于 `src/publish.rs` 的 URL 日志 `&cur[..80]`，以及一处测试断言的 `&html[..80]` / `&html[len-80..]`（后者会在断言失败、正在格式化诊断信息时二次 panic，掩盖真实失败原因）。

**修复：** `src/cover.rs` 抽出纯函数 `split_ai_lab_title`：按 `chars().count()` 判断长度，用 `char_indices().nth(15)` 取恒在 char boundary 的切点，并在切点前存在未闭合 `&` 时回退到该 `&`，保证 HTML 实体不被切断；前两个分支改用 `'：'.len_utf8()` / `": ".len()` 表达分隔符长度而非魔法数字 3 / 2。`src/publish.rs` 抽出 `log_url_preview`，用 `char_indices().nth(80)` 截断日志 URL。测试断言的诊断切片改用 `char_indices()` 正反两侧定位。项目 MSRV 为 1.88，而 `str::floor_char_boundary` 到 1.91 才稳定，因此不能图省事用新 API，必须用迭代器显式定位边界。

**防复发：** ① 任何对用户可见字符串（标题、摘要、URL、HTML）的截断/切分，一律用 `chars()` / `char_indices()`，禁止 `&s[..N]` / `&s[len-N..]` 这类固定字节偏移；`grep -rnE '&[a-z_]+\[\.\.[0-9]+\]' src/` 可作为自查入口。② `cover.rs` 新增回归测试覆盖中文 / emoji / 含 `&` 实体 / 全角与半角分隔符，并新增 `all_cover_styles_handle_multibyte_titles` 遍历全部 20 个样式 × 多字节标题，防止新样式再次引入字节切片。③ `publish.rs` 新增 `log_url_preview` 三组边界测试（纯 ASCII 超长、纯中文 81 字节、含中文短 URL）。④ 判断"标题过长"这类语义长度必须用字符数，不用字节数。

**证据：** 修复前真实二进制 `moonpub cover Articles/drafts/demo.md --style ai-lab` 中文标题返回 `rc=101` 及上述 panic 文本，同标题 `--style dark` 返回 `rc=0`；修复后同命令返回 `rc=0`、stderr 为空，6 组多字节标题（含 `R&D ...` 与 emoji）全部正常产出 HTML，`R&amp;D` 实体完整。`cargo test --lib cover::` 53 passed，完整 `cargo nextest run --lib`（排除另一会话在途 `src/illustrate.rs`）690 passed / 11 skipped。

**文章角度：** "一个中文标题让程序崩了"——UTF-8 里"第 15 个字节"和"第 15 个字符"不是一回事，这是中文软件绕不开的边界课。

### 带值 flag 命令的路径必须固定在第 2 个参数：同一个 bug 修了第三遍

**现象：** Obsidian 插件点「AI 润色并推进到微信草稿」必失败：插件拼的 argv 是 `ship --ai <path>`（`main.ts` 的 `buildArgs` 恒定「子命令 + 路径」），CLI 把 `--ai` 当成文章路径，报 `article path must point to a .md file: --ai`。手动调用同样中招：`moonpub cover --style ai-lab a.md` 报 `文件不存在: --style`，真实文章被忽略；`moonpub cover a.md bogus` 返回 rc=0 静默忽略 `bogus`；`moonpub cover a.md --style --screenshot` 把 `--screenshot` 当 style，截图开关静默失效。

**根因：** `render` / `cover` / `publish` / `ship` / `update-draft` / `export` 六个「单路径 + 带值 flag」命令各自把路径写死 `path_arg(rest, 1)`、flag 从 `rest[2..]` 扫；`cover` / `ship` 的 catch-all 是 `_ => {}`，多余位置参数被静默吞掉；`flag_value` 直接取下一个 token，不检查它是不是 flag。更根本的是修复方式本身：PR #168 只给「纯布尔 flag」命令建了统一入口，PR #169 只给「零 flag」命令补了尾部校验，同一类缺陷被按语法形状分批打补丁，每修一组就漏下一组。

**修复：** 新增 `path_and_flags_with_values(rest, usage, value_flags)` 作为唯一扫描器：识别「已知带值 flag + 它的值」，值缺失或以 `--` 开头直接 `MissingValue(flag)`；其余 `-` 开头 token 原样保留，第一个非 flag token 作路径，多余 token 按原顺序交回 arm，由既有 `match` 决定 `UnknownOption` / `UnknownCommand`，从而保持「未知 flag 优先于多余位置参数」的报错顺序。`path_and_flags` 降为 `value_flags = &[]` 的薄包装；`flag_value` 增加同一缺值守卫；六个 arm 统一改用新入口，`cover` / `ship` 的 `_ => {}` 改为 `UnknownCommand`。

**防复发：** ① `AGENTS.md` 写明「单路径 + flag 命令必须走 `path_and_flags*`，带值 flag 必须登记 `value_flags`」，禁止手写 `path_arg(rest, 1)` + `rest[2..]`；② `grep -nE 'path_arg\(rest, 1|rest\[2\.\.\]' src/cli.rs` 作为自查入口（`intake` / `configure` 等结构不同的例外需显式注释）；③ `VALUE_FLAG_POSITIONAL_COMMANDS` 表驱动测试覆盖 6 命令 × flag 前置/后置、多余位置参数、未知 flag、flag 当值、缺路径；④ 已知同类残渣（`intake photos` 的多位置参数、`configure` / `test-yulan` 的手写 flag 循环、`resolve_config` 用第一个非 flag token 当配置锚点）登记在此，避免下一轮从零发现。

**证据：** 修复前真实二进制 `cover --style dark /tmp/x.md` 报 `文件不存在: --style`、`cover /tmp/x.md bogus` 返回 rc=0；修复后前者进入业务层，后者报 `unknown command: bogus` 且 rc≠0；`cover /tmp/x.md --style --screenshot` 报 `missing value for --style`。`cargo nextest run --lib cli::` 91 → 99 passed；插件 argv `ship --ai drafts/post.md` 由 `InvalidArticlePath("--ai")` 变为正常解析出 `ShipAi`。

**文章角度：** 「同一个 bug 修了三遍」——当修复方式是给每类命令各打一个补丁，缺陷就会按语法形状分批复发；真正该收敛的是入口，不是分支。

### radar scrape 抓到缺闭合标签的中文页面直接 panic：字节窗口回退也要对齐字符边界

**现象：** `moonpub radar scrape --platform wechat --keyword 测试 --url file:///tmp/radar-bad.html` 面对一个 220 字节的页面（`<h2>` + 12 组中文、没有 `</h2>`）时退出码 101，stderr 为 `thread 'main' panicked at src/radar/scrape.rs:228:34: end byte index 204 is not a char boundary; it is inside '中' (bytes 202..205 of string)`，一条样本都没抓到。`--url` 是 `docs/COMMANDS.md` 公开的参数，意味着这个解析器要面对任意第三方页面，包括截断页、大写标签和不闭合的脏 HTML。

**根因：** `extract_from_html` 在找不到闭合标签时用固定字节窗口回退：`unwrap_or(content_start + 200.min(text.len() - content_start))`。`content_start` 一定在字符边界（紧跟 ASCII `>`），但 `+200` 不保证：CJK 是 3 字节、emoji 是 4 字节，剩余长度减去 200 后不是 3 的倍数时就切在汉字中间，紧接着的 `&text[content_start..content_end]` 直接 panic。触发条件是页面出现 `<h2` / `<h3` / `<a` 开标签之后，剩余全文找不到对应闭合标签（截断页、大小写不一致的 `</H2>`、只写开标签的脏 HTML），且剩余超过 200 字节。它躲过了上一轮的 UTF-8 扫描，是因为 `src/radar/scrape.rs` 整个文件没有任何 `#[cfg(test)]`，唯一沾边的 `scrape_stores_samples_in_jsonl` 用的是闭合良好的样例。

**修复：** 新增 `floor_char_boundary(text, start, end)`，用 `str::is_char_boundary` 从目标位置向前回退到最近的字符边界（最多 3 步），只替换 fallback 分支；闭合标签正常命中时输出逐字节不变。项目 MSRV 1.88，`str::floor_char_boundary` 到 1.91 才稳定，因此不能直接用标准库。

**防复发：** ① 新增 `src/radar/scrape.rs` 的 `#[cfg(test)] mod tests`（该文件首个测试模块），覆盖闭合中文标题、缺闭合标签（3 字节 CJK）、大写闭合标签、4 字节 emoji fallback、`<script>` 内容剥离、重复标题去重、纯文本无标签；② `AGENTS.md` 把约束从「禁止固定字节截断」扩到「禁止固定字节窗口 / 子串」，并写明新增解析 / 渲染 / 抓取模块必须自带测试模块；③ 自查入口在 `grep -rnE '&[a-z_]+\[\.\.[0-9]+\]' src/` 之外，还要搜 `+ 200`、`+ N` 这类字节窗口算术。

**证据：** 修复前同一命令 `rc=101`、stderr 为上述 panic、`panicked` 计数 1；修复后同一命令 `rc=0`、无 panic，输出 `scraped 1 samples → /tmp/radar-probe/.moonpub/trends.jsonl`。`cargo nextest run --lib radar::` 从 68 passed 增至 75 passed（新增 7 个测试）。

**文章角度：** 「不是所有 200 都是安全的」——抓虫时最危险的偏移往往不是裸常量，而是常量加法：它看起来像个安全窗口，实际是一次字符边界赌局。

### 破坏性命令的多余参数不能静默丢弃：`delete-draft <id> --dry-run` 会真的删草稿

**现象：** `moonpub delete-draft <media_id>` 是直接调微信接口的破坏性命令，没有 dry-run 模式，也不做二次确认。但它在解析层只取 `rest.get(1)`，把后面所有 token 丢掉，于是 `moonpub delete-draft 999999 --dry-run` 返回 `rc=0` 并打印「已删除草稿(浏览器会话): 999999」——调用方以为在预演，实际已经打到删除接口。同类静默丢弃还有：`status junk`、`version junk`、`list-drafts --bogus`、`fetch <url> extra`、`workspace extra`，全部返回 rc=0，把写错的参数当成合法调用。

**根因：** 手写解析器里有一批「固定参数个数」的命令，各自的 arm 直接返回 `Command::*`（`status` / `version` / `workspace` / `layout-recipes` / `list-drafts`）或只取一个 positional（`delete-draft` / `fetch`），从不校验 `rest[n..]` 是否为空。PR #169 的 `reject_stray_args_after_path` 只覆盖了「有一个路径的零 flag 命令」，PR #171 的 `path_and_flags_with_values` 只覆盖了「单路径 + 带值 flag 命令」，**固定 arity 且没有路径概念**的这一组从未被收口。危险程度不是均匀的：只读命令被吞参数只是误导，而 `delete-draft` 被吞掉的恰好可能是 `--dry-run` 这种「我以为安全」的开关。

**修复：** 新增 `reject_extra_args(rest, consumed)`：按命令已经消费的 token 数取 `rest.get(consumed..)`，交给既有的 `reject_unexpected` 判定（`-` 开头是 `UnknownOption`，否则 `UnknownCommand`），并保留后置 `--json` 白名单以兼容既有拼写。七个命令接入：`status` / `version` / `workspace` / `layout-recipes` / `list-drafts` 传 `consumed = 1`，`delete-draft` / `fetch` 传 `consumed = 2`；`help.rs` 给 `delete-draft` 明确写「没有 dry-run，多余参数会被拒绝」。

**防复发：** ① `AGENTS.md` 的 CLI 约束补一条：固定参数个数的命令必须调用 `reject_extra_args`，禁止让 `rest[n..]` 被静默丢弃；② `ZERO_ARG_COMMANDS` 表驱动测试覆盖 5 个零参数命令的「多余 flag / 多余位置参数 / 后置 `--json`」三面；③ 专门的 `delete_draft_rejects_dry_run_instead_of_deleting` 用破坏性命令的真实 argv 锁死这条语义——破坏性命令的「不知道的参数」必须报错，不能退化成默认执行；④ 已知未收口：`new <title>` / `write <idea>` 是自由文本（`moonpub new 我的 文章` 会静默只取「我的」），`intake photos` / `intake feishu` 允许 flag 前置会报误导性错误，`test-yulan --title --headed` 仍会把 flag 当值，均登记待后续处理。

**证据：** 修复前真实二进制 `delete-draft 999999 --dry-run` 返回 `rc=0` 并输出「已删除草稿(浏览器会话): 999999」，`status junk` / `version junk` 均 `rc=0`；修复后同一批命令分别返回 `rc=1` + `unknown option: --dry-run` 和 `unknown command: junk`，`status --json` 仍正常输出 `status/v1` 载荷。`cargo nextest run --lib cli::` 99 → 106 passed（新增 7 个）。

**文章角度：** 「Dry-run 有时候是假的」——最危险的不是命令报错，而是它成功得太安静：用户以为按下了预演按钮，程序收到的却是一张执行票。破坏性命令对未知参数必须零容忍。

### 标题被悄悄砍成第一个词：CLI 不能比底层更笨

**现象：** `moonpub new 我的 第一篇 文章` 返回 rc=0 并打印 `created`，实际只生成 `Articles/drafts/我的.md`，frontmatter 写着 `title: 我的`——「第一篇 文章」两个词凭空消失，用户不打开文件根本看不出来。`write <idea>` 同样只把第一个词交给 AI。`moonpub help preflight extra` 静默丢掉 `extra`，照常输出帮助。`test-yulan --title --headed` 更隐蔽：标题被设成 `--headed`，`--headed` 开关本身也失效。

**根因：** 三层叠加。① `src/cli.rs` 的 `new` / `write` 用 `rest.get(1)` 只取一个 token，而 `src/draft.rs` 的单测明明拿 `"我的 第一篇 文章"` 当标题、生成 `我的-第一篇-文章.md`——底层完全支持带空格标题，反倒是 CLI 把用户输入砍了；中文标题不加引号输入几乎是必然。② 同一批「按语法形状分批收口」的残渣：#169 收零 flag 单路径、#171 收单路径 + 带值 flag、#173 收固定 arity，自由文本 / 单 topic / 手写 index 循环这三条一直没碰。③ `test-yulan` 是唯一还手写 `index += 1` 配 `rest.get(index)` 的带值 flag 命令，直接绕过 #171 加在 `flag_value` 里的守卫——守卫没被复用，就等于没修。

**修复：** 新增 `join_free_text(rest, usage)`：把 `rest[1..]` 的非 flag token 用空格拼成完整文本，放行后置 `--json`（`new` / `write` 不在 `supports_subcommand_json` 里，这个拼写会落到 arm 里），其余 `-` 开头 token 报 `UnknownOption`，空文本报 `MissingValue`；`new` / `write` 接入。`help` 调用 `reject_extra_args(rest, 2)`。`test-yulan` 改成 `rest[1..].iter()` 配 `flag_value`，与 `configure` 同构。不改错误变体、不改退出码、不动任何一条既有文档用法。

**防复发：** ① `AGENTS.md` 把规则收成一句话：**任何命令都不得静默丢掉用户给的 token**，并点名「自由文本走 `join_free_text`、带值 flag 一律走 `flag_value`、禁止 `rest.get(1)` 和手写 index 循环」；② `FREE_TEXT_COMMANDS` 表驱动测试同时覆盖多词中文 / 英文标题、单词标题、trailing `--json`、未知 flag、缺文本五个面；③ `test_yulan_flag_values_reject_a_following_flag` 把「守卫必须被复用」写成测试，避免下次又冒出一个绕过守卫的手写循环；④ 剩余的 `intake photos` / `intake feishu` 多位置参数 + flag 前置已登记待办（它们失败时会明确报错，优先级低于静默截断）。

**证据：** 修前真实二进制 `new 我的 第一篇 文章` → `我的.md` 配 `title: 我的`、`help preflight extra` rc=0；修后 → `我的-第一篇-文章.md` 配 `title: 我的 第一篇 文章`、`help preflight extra` rc=1 报 `unknown command: extra`、`help preflight --json` 仍输出 `help/v1`。`cargo nextest run --lib cli::` 106 → 116 passed（新增 10 个）。

**文章角度：** 「最危险的 bug 是它成功了」——程序返回 0、打印 created，用户却只拿到半句话；验收不能只看退出码，要看产物本身。另外「守卫写好了」不等于「守卫被用上了」，只靠人记得调用，早晚会漏掉一个分支。

### `intake` 把 flag 当文件、把 `--draft` 当关键词：固定索引解析漏掉的最后一种形状

**现象：** 真实二进制取证 6 条：`intake photos --draft photos/day1` 与 `intake photos --analyze-images a.jpg b.jpg` 都报 `missing value for intake photos <file-or-dir> [more files or dirs]`（路径明明给了）；`intake photos a.jpg --draft b.jpg` 报 `unknown command: b.jpg`；`intake feishu --draft minutes.txt` 报 `unknown option: --draft`。最隐蔽的是 `intake feishu --query --draft --push`：`--draft` 被当成 query 关键词后，命令报 `missing value for intake feishu --push requires --draft`——用户写的 `--draft` 凭空消失；`--minute-token --draft --push` 同样。上一轮把它登记为「多位置参数 + flag 前置，失败时会明确报错，优先级低」，实测证明其中还藏着一条静默吞值。

**根因：** `src/cli.rs` 的 `intake` 是「按语法形状分批收口」的最后一块：`feishu` 用 `rest.get(2)` 定来源、`rest.get(3)` 定值，再按来源硬算 `flag_start = 3 | 4` 把剩余 token 全交给 flag 解析器；`photos` 从 `rest[2..]` 扫到第一个 `-` 开头 token 就 `break`，此后所有 token（包括合法路径）都当 flag 处理。两条分支都假定「位置参数在前、flag 全在后」，且都没有复用 #171 加在 `flag_value` 里的「下一个 token 是 flag 即报缺值」守卫——守卫没被复用，等于没装。

**修复：** 新增 `split_intake_tokens(tokens, value_flags)` 单一扫描器：一遍把 token 分成位置参数、布尔 flag、带值 flag 三组，带值 flag 复用与 `flag_value` 相同的守卫（下一个 token 以 `-` 开头即 `MissingValue(usage)`）；未知 flag 仍留在 `flags` 里由既有校验器报 `UnknownOption`。`parse_intake_feishu` / `parse_intake_photos` 分别接手两种形状：feishu 四种来源互斥，混用或缺来源统一报文档用法 `intake feishu <file> | --latest | --query <kw> | --minute-token <t>`；photos 收集任意数量的位置参数。命令枚举、错误变体、JSON 契约与既有文档用法零改动。

**防复发：** ① `AGENTS.md` 把 `intake` 归口写成明确入口（`split_intake_tokens` + 两个 parse 函数），并禁止再出现 `rest.get(2)` / `rest[flag_start..]`；② 8 个表驱动 / 等价性测试覆盖 flag 前置、flag 夹在多个路径之间、四种来源形状的 flag 前置、值 flag 拒绝吞开关、来源冲突、未知 flag 任意位置、无路径、以及「同一批 token 换顺序产出同一个 `Command`」；③ `help.rs` 与两份 COMMANDS 明写「flag 与位置参数顺序无关 / 带值 selector 必须带值」，避免文档继续暗示旧顺序；④ 上一轮 PROGRESS / 两份教训里登记的 `intake` 待办在本轮清账。

**证据：** 修前真实二进制 6 条对照见「现象」；修后 `intake photos --draft <不存在文件>` / `intake feishu --draft <不存在文件>` / `intake photos a.jpg --draft b.jpg` 全部进入业务层报 `No such file or directory`（证明解析放行且未写文件），`--query --draft --push` 改为明确报 `missing value for intake feishu --query <keyword>`，既有拼写 `intake photos <path> --draft --preview --no-open` 行为不变。`cargo nextest run --all-features --lib cli::` 116 → 124 passed。

**文章角度：** 「守卫写好了 ≠ 守卫被用上了」的第二次验证——同一个仓库里第 N 次出现「某个分支绕开公共解析器自己手写索引」；真正能防住的不是再补一个 case，而是让该形状没有第二条解析路径，并用「顺序无关」的等价性测试把契约锁死。

### 参数换个顺序，文章换了套主题：配置发现锚点没跟上 flag 前置

**现象：** 真实二进制对照（隔离目录：`work/moonpub.toml` 设 `[wechat] theme = "blueprint"`，`--articles` 指向一个没有配置的空目录）：`moonpub --json render <article> --author FLAG_AUTHOR` 生成的 HTML 是 `background: #f5f8ff`（blueprint），而只把 `--author` 挪到文章前面写成 `render --author FLAG_AUTHOR <article>`，生成的却是默认白底 `color: #333; background: #fff`。两条命令 rc=0、stdout 一字不差，产物主题却不同。更严重的一组：把 `--articles` 指向一个「父目录里另有 `theme = "ai-lab"` 的 `moonpub.toml`」的子目录，flag 前置那次直接加载了这个无关配置，HTML 变成 `background: #10111f` —— 不只是「漏加载」，还会「加载错项目的配置」。

**根因：** `resolve_config` 用 `rest.iter().skip(1).find(|s| !s.starts_with('-'))` 取第一个非 flag token 当路径锚点，再向上逐级找 `moonpub.toml`。这条规则在「路径必须紧跟命令名」的年代是对的；#168/#171 放开 flag 前置、#174 支持自由文本、#175 允许多位置参数之后，第一个非 flag token 常常是 flag 的值、子命令名或标题词。向上查找于是从错误目录开始：找不到就静默回落到默认配置，找到祖先里的其它 `moonpub.toml` 就更隐蔽。读同一份 argv 的老代码没跟着改，这是新拼写引入的回归。

**修复：** 新增 `config_anchor_candidates(rest, articles_dir)`：把 `rest[1..]` 所有非 flag token 解析成绝对/相对路径候选，用稳定的 `sort_by_key(|p| !p.exists())` 把「真实存在的路径」排到前面、其余保持 argv 原顺序；`resolve_config` 逐个候选向上查找，命中第一个 `moonpub.toml` 即返回。刻意不维护「全局带值 flag 白名单」——那会随每个新命令漂移，而「存在的文件优先」对新旧拼写都成立。显式 `--config` 优先、工作区根目录优先、无路径命令（`status` 等）行为零变化。

**防复发：** ① 4 个测试覆盖 8 组 argv 表驱动（render / cover / intake photos / preview / check 的路径在前与 flag 在前）、「不能加载 decoy 无关配置」、显式 `--config` 与无路径命令不回归、候选排序本身；② `AGENTS.md` 写明锚点不得假设第一个非 flag token，且改 CLI 解析形状时必须做「同一批参数换顺序、产物是否一致」的真实二进制对照；③ 两份 COMMANDS 把配置发现与参数顺序无关写成用户可见契约；④ 上一轮日志里登记的 `resolve_config` 已知残渣在本轮清账。

**证据：** 修前 A/B 产物差异见「现象」；修后 4 组（empty / decoy × 文章在前 / flag 在前）全部 `background: #f5f8ff`。`cargo nextest run --all-features --lib cli::` 124 → 128 passed，全量 749 → 753 passed / 0 skipped。

**文章角度：** 「支持新拼写时，别忘了所有读同一份 argv 的老代码」——解析器放开了顺序，配置发现却还在按旧顺序猜路径；验收不能只看退出码和 stdout，要看产物本身。

### `radar list --platform --keyword` 返回 rc=0 的空结果：最后一个手写解析器

**现象：** 真实二进制审计 `radar` 全家：`radar list --platform --keyword` 返回 rc=0 并打印 `trend samples (empty)`——用户以为按平台查数据，实际 `platform` 被塞成字符串 `"--keyword"`，过滤后必然为空，程序却报告成功；`radar list --platform --keyword 读书` 报的是 `unknown command: 读书`，完全不提 `--platform` 缺值。`radar suggest FIRST.md SECOND.md --platform wechat` 只报 `SECOND.md: No such file or directory`，`FIRST.md` 被静默丢弃；`radar suggest A.md --platform wechat --typo-flag` 对拼错的开关一声不吭；`radar import --platform wechat x.csv` 报 `unknown command: wechat`（把 `--platform` 当成 CSV 路径）。同类问题在 `add` / `scrape` / `analyze` 的带值 flag 上全部复现。

**根因：** `src/radar/cli.rs` 是全仓最后一个没接入公共解析器的模块：每个子命令各自 `while let Some(arg) = args.next()`，取值用裸 `next_arg = args.next().cloned()`，没有 #171 加在 `crate::cli::flag_value` 里的「下一个 token 以 `--` 开头即报缺值」守卫；`parse_radar_suggest` 还有一句 `_ => {}`，未知开关和多余路径直接蒸发；`import` / `analyze` 用 `args.first()` 固定取路径，假定「路径一定在第一个」。前几轮按语法形状分批收口（`src/cli.rs` 里的零 flag / 带值 flag / 固定 arity / 自由文本 / intake / 配置发现）没有覆盖到这个**独立文件里的第二套解析器**——扫描 `src/cli.rs` 时它是干净的，所以一直躲过自查。

**修复：** 重写 `src/radar/cli.rs`：新增 `split_radar_tokens(args, value_flags)` 单一扫描器，一遍拆出「位置参数」与「已登记的带值 flag 值」，取值统一 `use crate::cli::flag_value`（守卫与主 CLI 唯一真源一致）；`RadarTokens` 提供 `require` / `parsed<T>` / `no_positionals` / `single_positional`，六个子命令只声明自己的 `*_VALUE_FLAGS` 常量与必填项。重复 flag、未知开关、多余位置参数一律报错；`import` / `analyze` / `suggest` 的路径与 flag 顺序无关。删掉 `_ => {}`、`next_arg`、`parse_u64`、`parse_radar_article_arg` 等手写路径。命令枚举、错误变体、JSON 契约与全部既有文档拼写零改动。

**防复发：** ① `AGENTS.md` 把 `src/radar/cli.rs` 写进同一条「任何命令不得静默丢 token」约束，点名禁止裸 `args.next()` / `args.first()` / `_ => {}`；② `radar/cli.rs` 自带 11 个单测（原 3 + 新 8）：表驱动覆盖六个子命令的「开关当值」与「末尾缺值」、`suggest` 多余路径 / 未知开关、无位置参数子命令的多余路径、flag 前置等价性、重复 flag、非法数字、既有文档拼写回归；③ `help.rs` 与两份 COMMANDS 明写「顺序无关 / 带值 flag 缺值会报错」；④ 审计入口从「只扫 `src/cli.rs`」改为全仓 grep 手写 `args.next()` / `starts_with('-')`，避免再漏掉独立文件里的第二套解析器。

**证据：** 修前 11 条真实二进制对照见「现象」；修后 `radar list --platform --keyword` → rc=1 `missing value for --platform`，`radar suggest FIRST.md SECOND.md` → `unknown command: SECOND.md`，`--typo-flag` → `unknown option: --typo-flag`，`radar import --platform wechat x.csv` 正确解析到 `x.csv`（业务层报该文件不存在），`radar list --platform wechat` 仍 rc=0。取证全部指向隔离目录且不触发写文件 / 联网，探针目录零文件。`cargo nextest run --all-features --lib radar:: cli::` 208 passed。

**文章角度：** 「成功了但结果是错的」比崩溃更难发现——rc=0、stdout 正常，唯一线索是结果空得不合常理。收口要按「有几套解析器」而不是「有哪些命令形状」来清点：一个独立文件里的第二套手写循环，能让主解析器再干净也白搭。

### `init` 多位置参数静默覆盖 + 子命令 `--help` 只认第二个 token

**现象：** 真实二进制取证：`moonpub init a.toml b.toml` 返回 rc=0 并只创建 `b.toml`，`a.toml` 被静默丢弃（写成别的文件名、跑错目标）；`moonpub init --yes --help` 不显示帮助、反而报 `unknown option: --help`，用户求 help 却吃到报错。两条都是「成功/报错与意图不符」，前一条还静默、后一条把 help 当未知选项。

**根因：** `init` 解析循环对每个非 `-` 前缀 token 直接 `path_arg = Some(v)`，第二个位置参数覆盖第一个、全程零报错。`Options::parse` 只在 `rest.get(1) == "--help"` 时识别子命令帮助——即只认「紧跟命令名的第二个 token」；`init --yes --help` 的 `--help` 在索引 2，于是漏过 help 检测、落到子命令解析器被当成未知选项。老代码假设「位置参数必在固定索引」，而前几轮放开 flag 前置后这个假设对 help 检测也失效了。

**修复：** `init` 第二个位置参数按既有契约报 `UnknownCommand`（与 `browser_flags` / `draft-from-inbox` 一致），不再覆盖。`Options::parse` 改为 `rest.iter().skip(1).any(|s| s == "--help" || s == "-h")`，扫描每个 trailing token。新增 3 个 cli 解析测试：`init` 多余位置参数报错、单一位置参数仍解析、`--help` 在 flag 之后仍命中（既有「`--help` 紧跟命令名」与「单 topic 帮助」测试保持不变）。

**防复发：** `AGENTS.md` 明确「子命令 `--help`/`-h` 必须扫描 `rest[1..]` 每个 token，不能只认 `rest.get(1)`」「`init` 最多一个位置路径，第二个必须报 `UnknownCommand`」。这延续了「任何命令不得静默丢 token」总纲——多位置参数被静默覆盖，本质也是丢 token（只不过丢的是第一个）。

**证据：** 修前 `init a.toml b.toml` → rc=0 + 仅 `b.toml`；修后 → rc=1 `unknown command: b.toml` 且零文件写入。`init --yes --help` 修前报 error、修后输出 `moonpub init` 帮助页（rc=0）。`cargo nextest run --all-features` 764 passed / 0 skipped（+3 新测试）。

### `configure zanshangg` 被静默接收：拼错的步骤名永不执行却照常保存草稿

**现象：** 真实二进制取证：`moonpub --json configure --boguss` 在解析层明确报 `unknown_option`；但 `moonpub --json configure zanshangg`（拼写错误的步骤名）解析层零错误、直接进入浏览器自动化。按代码语义，`run_step("zanshang")` 用精确字符串比较，永远不会匹配 `steps = ["zanshangg"]`，于是「赞赏」步骤被静默跳过，而 `step_baocun` / `step_fucha` 无条件执行、草稿照常保存——用户以为配好了，实际没有，且报告成功。

**根因：** `src/cli.rs` 的 `configure` arm 是全仓最后一个没收口的手写循环，对其他 token 直接 `s => steps.push(s.to_owned())` 照单全收。step 名是 6 个有限枚举（`yuanzhuang` / `zanshang` / `liuyan` / `chuangzuo` / `moban` / `yulan`，定义在 `src/publish.rs`），但解析层没有白名单校验；前几轮按语法形状收口了单路径、带值 flag、固定 arity、自由文本、intake、radar，唯独这个「有限枚举位置参数」形状没碰。

**修复：** 在 `src/publish.rs` 增加 `is_valid_configure_step(name)`，用 `matches!` 匹配同一组 `STEP_*` 常量（与 `run_step` 共用，零清单漂移）；`configure` arm 对每个位置 token 经它校验，未知名报 `UnknownCommand(format!("configure {s}"))`。命令枚举、错误变体、JSON 契约零改动。TDD：先写 `configure_rejects_unknown_step`（对 `zanshangg` / `YUANZHUANG` / `clips` / 中文）跑出 FAIL，显示拼错名被解析成 `steps: ["zanshangg"]`，再实现修复转 PASS；另有 `configure_accepts_every_known_step` 锁住 6 个合法名全部接受。

**防复发：** `AGENTS.md` 明确「有限枚举位置参数必须逐个经共享校验器，禁止照单全收」，并要求校验器与执行方共用同一组常量。这两条把「不得静默丢 token」从「不吞 flag / 多余参数」扩展到「枚举值也不得静默无效」。

**证据：** 修前单测 FAIL（`unwrap_err()` on `Ok(steps:["zanshangg"])`）；修后真实二进制 `configure zanshangg` / `configure YUANZHUANG` → `error/v1` `unknown_command`，`configure --boguss` 仍 `unknown_option`，6 个合法 step 由单测确认全部进入 `Configure { steps }`。全量 `cargo nextest run --all-features` 766 passed / 0 skipped（+2）；clippy `-D warnings`、rustfmt、文档 lint 全过。

### `theme: blu-print` 渲染成功却换了主题：`check --json` 还谎报 `publishable: true`

**现象：** 真实二进制取证：文章 frontmatter 写 `theme: blu-print`（拼错），`moonpub render` rc=0、正常输出 `rendered`，但生成的 HTML 背景是默认 `#fff`；正确拼写的 `blueprint` 应是 `#f5f8ff`，而完全不写 theme 也是 `#fff`。更隐蔽的是配置路径：把 `[wechat].theme = "blu-print"` 写进 `moonpub.toml`，`check --json` 明确回报 `effective_theme: blu-print` + `theme_source: wechat_config` + `publishable: true`，但实际渲染仍是默认主题。`check` 是 Agent 发布前用来验证的命令，它主动确认了一个根本没生效的主题。

**根因：** `Theme::from_name` 的 match 末尾是 `_ => Self::default()`，把任何未知名字静默降级成默认主题。这个兜底对内建主题是安全的，但 frontmatter 与配置都是用户输入，拼错就会中招；而且 `protocol/check.rs` 的 `check_json` 是完全独立的一条路径，直接把配置原文当 `effective_theme` 输出，既不渲染也不校验，于是「报告」和「实际产物」可以永久分叉。前几轮把 CLI 参数解析收口得很严，但**参数值的内容合法性**（有限枚举值）只有 `configure` step 做了校验，主题名漏了。

**修复：** 在 `theme.rs` 抽出私有 `resolve(name) -> Option<Self>` 作为单一真源（含 `geek_black` / `ai_lab` / `one_dark` 历史别名），`from_name` 改为 `resolve(...).unwrap_or_else(Self::default)`，新增 `is_known_name()` 供输入边界校验。`render_article` 在算完 `effective_theme` 后立即校验，未知名返回新增的 `AppError::InvalidThemeName`；`check_json` 同步校验并改为返回 `Result`，不再对无效主题产出 `check/v1` 成功载荷。错误码全链补齐：`ErrorCode::InvalidThemeName` + `ALL`（23→24）+ `as_str` / `summary` / `example_next_step` / `classify` / `all_failures` 样本，确保注册表与分类函数零漂移。命令枚举、既有 JSON 载荷结构零改动。

**防复发：** `AGENTS.md` 明确「主题名与 `configure` step 同属有限枚举，必须经共享校验器，且渲染与 check 两处入口都要拦；校验器与执行方共用同一个 `resolve()`」。测试用 `for name in Theme::names()` 把注册表每个名字都过一遍 `is_known_name`，新增主题却忘记更新 `resolve()` 会立刻失败；另有拼写变体表（`blu-print` / `BLUEPRINT` / `blue print` / 空串）锁住拒绝行为。

**证据：** 修前真实二进制 `render` rc=0 但主题 `#fff`、`check --json` 谎报 `effective_theme: blu-print` + `publishable: true`。修后 `render` 与 `check --json` 均返回 `error/v1` `invalid_theme_name`（`command` 分别为 `render` / `check`）；清理旧产物后复跑，拼错主题零 HTML / 零 draft.json 写入；合法 `blueprint` 仍 `#f5f8ff`、合法别名 `ai_lab` 仍正常渲染。全量 `cargo nextest run --all-features` 766 → 772 passed / 0 skipped（+6）；clippy `-D warnings`、rustfmt、文档 lint 全过。

### `radar import` 把 `likes=abc` 当成空值：rc=0 报成功，互动数据全丢

**现象：** 真实用户路径 `moonpub radar import trends.csv`：CSV 的 `likes` 列填了 `abc`（或 `N/A`、负数、`1.5`、超出 u64 范围），命令 rc=0 打印 `imported 2 samples`；样本落库后 `likes` 却是 `None`，`radar analyze` 按互动量排序时这些文章直接沉底。用户看到的是一次成功导入，实际填过的数据被悄悄抹掉，而且 stdout / stderr 都没有任何线索指向那一列。

**根因：** `src/radar/import.rs::import_csv` 用 `get_u64 = |idx| get(idx).and_then(|v| v.parse().ok())` 读数值列。上层 `get()` 已用 `.filter(|s| !s.is_empty())` 区分了「没填」和「填了」，所以进到 `parse().ok()` 的 `v` 都是用户明确写过内容的字符串；`.ok()` 把解析失败统一压成 `None`，与「这一列没填」合并成同一状态。此外代码逐行 `add_trend_sample` 边解析边写，即便之后改成报错，先前的行也已落库、重试会重复导入。前几轮把 CLI token 与有限枚举值的校验收得很严，**单元格内容合法性**这一层却漏了。

**修复：** 用 `parse_metric` 闭包替代 `get_u64`：空单元格（列缺失或没填）仍是 `Ok(None)`，非空但 `parse::<u64>()` 失败则返回 `AppError::InvalidCsv("line {n}: invalid unsigned integer in '{column}' column: {value:?}")`，带物理行号、实际表头列名与原始值（复用已有错误码，不新增枚举）。循环加 `enumerate()` 得到准确行号（表头算第 1 行）；改为先把全部合法行解析进 `Vec<TrendSample>`、校验通过后再统一写盘，失败时零写入。命令枚举、错误变体、JSON 契约与合法 CSV 行为零改动。

**防复发：** `AGENTS.md` 明确「可选数值列是『没填可以、填了必须合法』，禁止用 `.parse().ok()` 把非法值压成 `None`；批量导入先全量校验再写入，禁止半成品」。回归测试锁三种场景：非法数值被拒且报错带行号 / 列名 / 原值、失败后零样本落库、空单元格仍保持 `None` 且导入成功。审计入口从「只看参数解析」扩展到「看解析后的值内容」。

**证据：** 修前 TDD 红：`import_csv` 对 `likes=abc` / `comments=N/A` 返回 `Ok("imported 2 samples")`，两条新测试 FAIL。修后真实二进制：`radar import trends.csv`（含非法 likes）→ rc=1 `Error: invalid csv: line 3: invalid unsigned integer in 'likes' column: "abc"`，隔离目录只有输入 CSV、零样本文件；合法 CSV 仍 rc=0 `imported 1 samples`。`cargo nextest run --all-features --lib radar::` 86 passed（+3）。

**文章角度：** 「没填」和「填错了」是两种完全不同的用户意图，把它们合并成同一个 `None`，程序就失去了报错的能力——这类静默降级比崩溃更难发现，因为 stdout 一切正常。

### `moonpub humanize` 处理中英混排直接 panic：30 字节回退窗口没对齐字符边界

**现象：** 真实二进制取证：给含 `——确保abcdefghij中文测试内容继续写下去`（破折号解释句，后面没有句号 / 分号 / 换行）的文章跑 `moonpub humanize <article.md>`，直接 rc=101、stderr 是 `panicked at src/humanize.rs:133:13: end of range should be a character boundary`，文章一字未改；换成正常中文句子则 rc=0。`humanize` 是文档公开的本地命令，也会被 `render --humanize` 走到；写技术文章时「——确保 API 和 Webhook 的中文说明……」这类中英混排很常见。

**根因：** `src/humanize.rs::break_parallelism` 处理 `——确保` 时先找 `。；\n` 当句子边界，找不到就用回退窗口 `start + 30.min(out.len() - start)` 作删除终点。`start` 来自 `find("——确保")`，一定在字符边界，但 `+30` 是**字节**数：前面 10 个 ASCII 加后面 3 字节一个的汉字，30 正好落进第三个汉字中间，`out.replace_range(start..segment_end, "")` 随即 panic。这是 UTF-8 固定字节偏移的又一种形态（字节窗口），上一轮修 `radar/scrape.rs` 时已在 `AGENTS.md` 立过铁律，但自查只扫了 markdown / theme / scrape 等模块，`humanize.rs` 躲过去了——它只有 6 个测试，没有一条覆盖 `——确保` 的无标点回退路径。

**修复：** 回退分支改为遍历 `out[start..].char_indices()`，累加 `ch.len_utf8()`，取「不超过 30 字节」的最后一个字符边界当 `segment_end`；有标点的原路径逐字节不变。MSRV 1.88 没有 `floor_char_boundary`，不靠新 API 图省事。同文件另一处字节切片 `start + end + 6`（6 正好是「而且」的 UTF-8 长度）经复核恒在边界，未动。

**防复发：** `AGENTS.md` 字符串边界铁律点名 `break_parallelism` 这个坑，明确「任何取 N 字节窗口后切片 / 替换的逻辑，必须先用 `char_indices()` / `is_char_boundary` 对齐」。新增两条测试：精确的 ASCII + 汉字触发点，以及表驱动覆盖 emoji / 纯中文 / 纯 ASCII / 短文本的 `——确保` 回退路径；测试直接调 `humanize()`，后续任何 phase 改动引入同类 panic 都会立刻红。

**证据：** 修前真实二进制 rc=101 + `end of range should be a character boundary`，正常对照文章 rc=0；TDD 两条新测试红在同一行。修后同一输入 rc=0、输出 `测试内容继续写下去`，更真实的文章（含 API / Webhook 中英混排、无句号结尾）rc=0 且破折号解释句被正确删除；`humanize::` 8 passed（+2）。

**文章角度：** 「没找到标点就取 30 个字节」读起来很合理，直到第 30 个字节落在汉字中间。UTF-8 的坑不只在 `&s[..N]`，任何「偏移 + 字节常数」的窗口算法都要先对齐再切。

### 微信摘要 120 字上限被当成 120 字节：preflight 报 pass，draft.json 只剩 40 字

**现象：** frontmatter `digest` 写 100 个汉字（100 chars / 300 bytes），`moonpub render` 仍 rc=0 输出 `rendered`，但生成的 `<slug>.draft.json` 里 `digest` 只有前 40 个字；同一篇文章跑 `moonpub preflight` 却明确打印 `digest_limit [pass]: digest within 120-char limit (100 chars)`。`render` / `push` / `ship` 都走 `build_draft_json`，所以问题会一路带到微信草稿；质量门说通过，产物却是半句话，stdout / stderr 没有任何线索指向单位错误。

**根因：** `src/render.rs::build_draft_json` 用 `120usize.min(digest.len())` 作截断终点，`digest.len()` 是**字节数**；`while !digest.is_char_boundary(end) { end -= 1; }` 只把切点回退到合法边界、避免 panic，并不改变「120 字节 ≈ 40 汉字」的事实。而 `src/preflight.rs` 对同一上限用 `d.chars().count()` 校验，注释还专门写明 character-based (not UTF-8 bytes)。校验端与写入端单位不一致，产物可以永久偏离质量门结论。

**修复：** 改成字符语义：`match digest.char_indices().nth(120)`，有第 121 个字符时取它的字节下标 `&digest[..byte_idx]` 截断，没有则原串逐字节返回。`char_indices()` 天然给出合法边界，MSRV 1.88 没有 `floor_char_boundary` 也不受影响；≤120 字符（含空串、短摘要、恰好 120 字）行为逐字节不变，`escape_json` 与 JSON 结构零改动。

**防复发：** `AGENTS.md` 新增「N 字限制」铁律：`str::len()` 是字节数，校验端与写入端必须同口径，禁止 `N.min(s.len()) + is_char_boundary` 这种只防 panic 的写法。新增 5 条测试：100 汉字不腰斩、115 ASCII + 5 汉字恰好 120 字符不截断、121 汉字截到 120、短摘要控制、纯 ASCII 121 控制；真实二进制前后对照 40 → 100，控制用例 2 → 2。

**证据：** TDD 先红：前三条分别报 `left: 40 / right: 100`、`left: 116 / right: 120`、`left: 40 / right: 120`，两条控制用例绿；修复后 7/7 绿。真实二进制（隔离临时目录）：修复前 `cjk` draft digest 40 字、`control` 2 字，preflight 对 100 字摘要报 pass；修复后 `cjk` 100 字、`control` 2 字，preflight 结论不变。

**文章角度：** 「同一个限制，校验一遍、写入一遍」最容易出现单位分叉。字节数能防崩溃，但不能防中文被静默腰斩；只有校验和写入共用 `chars()` / `char_indices()` 口径，质量门的 pass 才和产物一致。

### 无空格井号标题与表格管道符：微信正文里的两个排版泄漏

**现象：** 真实二进制 `moonpub render` 取证两处异常。① 中文作者写 `#小标题`（井号后无空格）时，标题没被识别，井号原样暴露在正文里变成一行 `#小标题`；只有写 `# 小标题`（带空格）才渲染成标题。② 表格单元格写 `| a \| b |`（转义管道）或 `` | 正则 | `a|b` 匹配 | ``（行内代码含管道）时，单元格被从中间劈开：`a \| b` 渲成标签 `a \` + 值 `b`，`` `a|b` `` 变成裸反引号 `` `a `` 且后面列丢失；只有表头没有数据行的表格还会吐出一个空 `<section>`。

**根因：** `src/markdown/plain.rs` 的标题判定是四个 `line.strip_prefix("# ")` 分支，硬性要求井号后跟一个空格——这是 CommonMark 的习惯但不符合中文写作实际（`C#`、`#话题` 在正文中间则本来就该保留，行首无空格标题却要识别）。表格用朴素 `inner.split('|')` 切单元格，不知道 `\|` 是转义、也不知道反引号里的 `|` 是代码内容，于是所有管道一律当分隔符。

**修复：** 抽 `heading_parts()` 单一判定：数行首连续 `#`（1~4 个，`#####` 不识别），`trim()` 后正文非空才算标题，1/2 级映射 h2、3→h3、4→h4；只认行首，正文中间的井号不受影响。表格改用 `split_table_cells()`，逐字符扫描：反引号切换 `in_code` 状态、遇到 `\` 紧跟 `|` 就吃成字面量、只有不在代码态的 `|` 才切列；headers 与 rows 从 `&str` 改为拥有所有权的 `String` 以承载去掉反斜杠后的内容；无数据行时提前返回空串，不再吐空 section。

**防复发：** `AGENTS.md` 点名 `heading_parts()` / `split_table_cells()` 为唯一入口并禁止退回散落分支与朴素 split；新增 3 条测试：无空格井号标题（含正文井号不误伤）、转义管道与行内代码管道不切坏、只有表头不吐空 section。修后真实二进制复核：无空格标题输出 `<h2>无空格标题</h2>`、`C#` / `#话题` 仍留在正文，`a \| b` 显示为 `a | b`、`` `a|b` `` 完整渲成 code span、空表格零 section。

**证据：** TDD 三条先红（分别得到井号原样正文、切坏的 `a \` 标签、空 `<section>`），修复后全绿。取证目录隔离在 `/tmp`，不碰用户 Obsidian 数据；WIP `src/cover.rs` / `src/illustrate.rs` 备份后临时还原到 HEAD 取证、复核 sha256 精确复原，未进提交。

**文章角度：** 「分隔符就是分隔符」是纯 ASCII 时代的直觉；一旦内容里有转义和代码，朴素 split 就会把用户的一个单元格拆成两列。标题同理——「井号后必须有空格」是规范，但用户不会因为你遵循规范就少写错别字。

### 一个孤立的竖线让整篇文章渲染崩溃：首尾判定不等于合法区间

**现象：** 真实二进制取证：文章正文里出现单独一行 `|`（复制残留、写了一半的表格都很常见），`moonpub render` 直接 rc=101，panic 在 `src/markdown/plain.rs:248` — `byte range starts at 1 but ends at 0`，整篇文章一字未改。把那行去掉就 rc=0 正常输出。

**根因：** 主循环与 `render_table()` 都用 `trimmed.starts_with('|') && trimmed.ends_with('|')` 判定表格行。单独一个 `|` 同时满足首尾两个条件，于是被塞进 `table_buf`，随后 `&trimmed[1..trimmed.len() - 1]` 中 `len == 1`，切片区间变成 `[1..0]`，直接 panic。上一轮刚把单元格切分升级成尊重转义与行内代码，却漏了回头检查「进入切分的前置条件」本身——首尾字符判定只说明了瓷器长什么样，没保证它至少有两个字节。

**修复：** 抽 `is_table_row(trimmed)` 作单一判定：`trimmed.len() >= 2 && trimmed.starts_with('|') && trimmed.ends_with('|')`，主循环与 `render_table()` 共用。孤立的 `|` 不再被当成表格行，落入普通正文路径按 `<p>|</p>` 原样保留——既不 panic，也不静默丢掉用户内容。合法表格的解析与产物逐字节不变。

**防复发：** `AGENTS.md` 点名 `is_table_row()` 为表格行唯一入口，禁止只判首尾就切片。TDD 两条先红（都复现同一 panic）后绿：一条覆盖正文中间孤立 `|`，一条覆盖两个完整表格中间夹孤立 `|`（确认不会把前后表格一起带崩）。修后真实二进制：同一触发文章 rc=101 → rc=0，`|` 成为正文段落、前后正文都在；正常表格对照 a/b 两行渲染不变。

**证据：** 全量 785 → 787 passed（+2），质量门 6/6；取证隔离在 `/tmp`，WIP `src/cover.rs` / `src/illustrate.rs` 备份后临时还原到 HEAD 取证、复核 sha256 精确复原，未进提交。

**文章角度：** 一个只检查「首尾是什么」的判定，会把长度为 1 的边缘输入放进为长度 ≥ 2 准备的切片里。崩溃不在表格解析逻辑里，而在「这行算不算表格」的入口处——防御要加在门禁上，不是加在门内的每一条走廊。

### BOM 与写反的方括号：两个藏在 frontmatter 入口的元数据杀手

**现象：** 真实二进制取证两处。① Windows 记事本保存的文章开头带 UTF-8 BOM（U+FEFF），`moonpub render` 仍 rc=0 打印 `rendered`，但 `draft.json` 的 title / digest 全是空字符串、`theme: blueprint` 静默退回默认主题，YAML 那几行连 BOM 字符一起被塞进正文渲染；`preflight` 报 `title_required [fail]: title missing`——用户明明写了 title。② frontmatter 写 `tags: ]生活[`（方括号写反），`moonpub render` 直接 rc=101 panic 在 `src/article.rs:204` — `byte range starts at 14 but ends at 6`，整篇零改动。正常文章对照 rc=0。

**根因：** ① `parse_frontmatter()` / `strip_frontmatter()` 都用 `md.trim_start()` 找 `---` 开头，但 BOM 不属于 `char::is_whitespace()`，`trim_start()` 去不掉，于是 `starts_with("---")` 失败、整段 frontmatter 被当成正文。② `parse_yaml_string_array()` 用 `find('[')` 和 `rfind(']')` 定位，`]生活[` 里 `]` 在 `[` 前面，直接做 `[bracket_start+1..bracket_end]` 变成 `[14..6]` panic。两处都在 frontmatter 入口，都是「输入形态没校验就当成正常结构处理」。

**修复：** ① 抽 `strip_bom(md)` 单一 helper，`parse_frontmatter()` / `strip_frontmatter()` 先剥 BOM 再 trim；`strip_frontmatter()` 的「无 frontmatter / 不完整」返回路径也剥 BOM，让 BOM 后的正文 H1 仍能提取到标题。② `parse_yaml_string_array()` 切切片前校验 `bracket_end > bracket_start`，方括号写反按空数组处理。合法文章逐字节不变。

**防复发：** `AGENTS.md` 点名 `strip_bom()` 为 frontmatter 必备前置、方括号切片必须先校验顺序；新增 4 条测试：BOM frontmatter 解析、BOM 正文不泄漏 YAML、写反方括号不 panic、BOM + 无 frontmatter 的 H1 仍可提取。修后真实二进制：title/digest 恢复、YAML 与 BOM 字符不再进正文、`preflight` 的 `title_required [fail]` 消失（digest_limit 转 pass），tags 写反 rc=0 正常渲染，对照文章不变。

**证据：** TDD 四条先红（title 全 None / YAML 泄漏 / 同一 panic / H1 提取失败）后绿；全量 787 → 791 passed（+4）。取证隔离在 `/tmp`，WIP `src/cover.rs` / `src/illustrate.rs` 备份后临时还原到 HEAD 取证、复核 sha256 精确复原，未进提交。

**文章角度：** 「文件开头有几个看不见的字符」和「用户把方括号打反了」都不是罕见输入，而是编辑器默认行为与普通人手抖。解析器若假设输入总是规范形态，失败方式不是报错而是静默丢失或崩溃——入口处的宽容，比事后补救便宜得多。

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

## 2026-11-17 命令注册表与解析器的双向桥接测试

**现象：** 帮助注册表（`help::COMMANDS`，44 条）、CLI 解析器 `parse_command` 的命令名分支、结构体化 JSON 输出的名单（`supports_subcommand_json` 字符串清单 + `has_structured_json_output` 枚举清单）是四份独立维护的清单。历史上 #150 / #155 / #160 / #177 四个 PR 都在补「某一处加了命令、另一处忘了同步」的漂移：notion 里注册了、解析器报 unknown，或文档说支持 `--json`、实际被当普通参数吞掉。

**根因：** 只有一个方向的测试（注册表 → 四份文档），没有任何测试把「注册表 ↔ 解析器」「JSON 名单 ↔ 注册表」对起来；唯一相关测试 `trailing_json_is_stripped_for_every_machine_readable_command` 内联了一份手写数组，再逐个断言 `supports_subcommand_json(name)`——拿自己的手写副本测自己，两份一起漂移时永远绿。

**修复：** 在 `src/cli.rs` 测试模块抽 `JSON_CAPABLE` 共享常量（旧自证测试改为读它），新增两条桥接测试：`every_registered_command_is_recognized_by_parser` 遍历 `COMMANDS`，用 `parse_command(spec.name, …)` 断言没有一个被报 `UnknownCommand`；`every_json_capable_name_is_a_registered_command` 遍历 `JSON_CAPABLE`，断言每个名字 `for_command(name)` 都在注册表。刻意不把 `version` 加进 `has_structured_json_output`——它走 fallback 包装，是设计如此，不是漂移。

**防复发：** `AGENTS.md` 点名两条桥接测试和共享常量，禁止再写自证测试。受控变异取证：删掉 help 里的 `version` 条目 → 反向桥红；把 help 里 `status` 改名 → 正向桥红；恢复后三条全绿。注意 `cp` 恢复文件后 Cargo 可能因 mtime 未变而复用旧目标文件，务必 `touch` 或确认指纹后重编。

**文章角度：** 「同一份事实存了三份」本身不可怕，可怕的是只有单向核对。单向测试保证 A 出现时 B 也在，却抓不到 B 多出来的幽灵项；真正的护栏要双向走一遍，并且每一侧都从独立真源派生——拿自己的副本测自己，只会把漂移锁得更死。

## 2026-11-18 安装链接写死版本号，三份新用户入口文档落后十个版本

**现象：** `docs/USER_GUIDE.md`、`docs/USER_ONBOARDING_ZH.md`、`docs/USER_ONBOARDING.md` 的安装命令都写死 `releases/download/v0.4.2/...`，而 GitHub Latest 与 `Cargo.toml` 都已是 `v0.4.13`（2026-11-06 发布）。新用户按这三份文档装，会拿到 2026-06-23 的二进制，比当前落后 10 个版本；而这三份文档分别自称"用户使用说明书""完整用户指南"。同时 `ROADMAP.md` 的「当前源码版本」停在 `v0.4.12`，`docs/PROGRESS.md` 存根仍写着 `v0.4.2` / `v0.4.1`。

**根因：** 2026-11-06 修过同一类问题（`docs/index.html`、`docs/slides.html`、`GETTING_STARTED.md` 改为 `releases/latest/download/`），但那次是逐文件手工点修、没有留下任何机读约束，于是 `USER_GUIDE` 与两份 `USER_ONBOARDING` 被漏掉；`ROADMAP` 与 `docs/PROGRESS.md` 不在那次范围内，继续按老写法漂移。`docs/PROGRESS.md` 更典型：它本来是"给旧链接用的重定向存根"，却又抄了一份"当前摘要"，等于给同一事实造了第二个真源。

**修复：** 三份入口文档的安装链接统一改成 `releases/latest/download/<asset>`（GitHub 的 `latest` 别名随 release 自动跳转，不需要改版本号），文字同时说明"不用记版本号、每个包配同名 `.sha256`"；`ROADMAP.md` 的当前源码版本按真源改为 `v0.4.13`；`docs/PROGRESS.md` 整段删掉"当前摘要"，只留"统一看根目录 `PROGRESS.md`"的说明与 `moonpub --version` / `--json doctor` / `--json workspace` 三条自查命令——消灭第二真源，而不是同步它。

**防复发：** 新增 `tests/doc_version_contract.rs` 三条契约：① 递归扫描仓库内 `.md` / `.html`，活文档出现 `releases/download/vX.Y.Z` 即失败（快照文档按 `RELEASE_NOTES_v*` / `RELEASE_GATE_v*` 文件名前缀与 `LAUNCH_ARTICLE_ZH.md` 显式白名单放行，白名单要求写明理由）；② `ROADMAP.md` 的「当前源码版本」必须等于编译期 `env!("CARGO_PKG_VERSION")`，发版抬版本后忘了同步即刻变红；③ `docs/PROGRESS.md` 不得再出现「当前源码版本」字样，防止有人把摘要抄回去。受控变异取证：把 `USER_GUIDE` 的 `latest` 改回 `v0.4.2`、`ROADMAP` 改回 `v0.4.12`，两条契约同时变红且报错直接给出文件行号与修法；恢复后三条全绿。

**证据：** `docs/USER_GUIDE.md`、`docs/USER_ONBOARDING.md`、`docs/USER_ONBOARDING_ZH.md`、`ROADMAP.md`、`docs/PROGRESS.md`、`tests/doc_version_contract.rs`；GitHub Latest 实测 `v0.4.13`（2026-11-06），`Cargo.toml` 与 `.release-please-manifest.json` 同为 `0.4.13`。

**文章角度：** "改成最新链接"是一次修复，"让旧写法再也过不了 CI"才算修完。同一个问题修过一遍还复发，通常不是手滑，而是那次只改了文件、没改规则。尤其当文档自称"存根"却仍抄一份数据时，它就成了第二个真源——对付第二真源的办法是删掉它，不是同步它。

## 2026-11-18 省略首尾竖线的 Markdown 表格整块漏成正文，微信里露出一排 `|`

**现象：** 手写或粘贴来的 Markdown 表格经常省略首尾竖线（`日期 | 事件 | 备注` / `--- | --- | ---` / `11-01 | 报名 | 20 人`）。这是 GFM 合法写法，但 `moonpub render` 把它整块当正文，微信里出现三段带竖线的段落。用户长期反馈的「表格在微信公众号上显示异常」有一类就是这个。

**根因：** `is_table_row()` 的契约是「trim 后长度 ≥ 2 且首尾都是 `|`」。这个条件是为修「孤立 `|` 触发 `[1..0]` panic」而收窄的，收窄后顺带把无边框写法整类排除在表格识别之外——只看行的两端，不看 GFM 真正的表格信号（表头行 + 分隔行）。判定与渲染还各自散落一份「剥首尾竖线」的假设（`&trimmed[1..trimmed.len() - 1]`），无边框行即便进了表格也会被砍掉首尾各一个字符。

**修复：** 表格识别统一成三种入口：① 传统带首尾竖线；② 本行含 `|` 且下一行是分隔行（`is_delimiter_row()`，每格形如 `:?-+:?`）；③ 已在表格块内、含 `|` 且列数与表头一致（`is_table_continuation()`）。新增 `strip_edge_pipes()` 统一切分前剥边，代替 `[1..len-1]` 切片；结尾是转义管道（`\|`）时不剥，那属于单元格内容。单独一行 `---` 仍是分隔线，不当单列表格。列数不一致的续行退回正文而不是吞进表格——宁可原样显示，也不猜。

**防复发：** 三条 TDD 用例：无边框表格必须渲染成一张卡片且正文不残留 `|`；表头带边框、数据行不带的混合写法必须留在同一张表里；只有「含 `|` 但下一行不是分隔行」的正文（如 `用 A | B 表示两个变量。` 后紧跟 `---`）必须保持正文，不许凭空造卡片。受控变异：把 `starts_table` 退回 `is_table_row(trimmed)`，前两条立刻变红，恢复后全绿。另外用 400 行随机管道 / 反斜杠 / 中文混排喂给 `render`，退出码 0、无 panic。

**证据：** `src/markdown/plain.rs`（`strip_edge_pipes` / `is_delimiter_row` / `is_table_continuation` / `render_table`）、`src/markdown.rs` 三条新用例；真实二进制渲染无边框表格后，正文 `|` 残留 0 处，两处示例表格都成为卡片。

**文章角度：** 表格判定收窄是为了修 panic，但收窄只该收到「不 panic」为止，不该顺手把一种合法写法整类排除。当判定条件描述的是「这一行长什么样」而不是「这一块是什么结构」时，异常输入就会成片漏掉——GFM 的表格信号是表头 + 分隔行，不是首尾那两个字符。

## 2026-11-18 带括号的链接被截成死链，段中图片吐嵌套 `<p>`

**现象：** 两处都来自同一类输入，都在微信里可见：

1. `[维基](https://zh.wikipedia.org/wiki/Rust_(编程语言))` 渲染成 `href="https://zh.wikipedia.org/wiki/Rust_(编程语言"`，并且正文多出一个 `)`——点开是死链。维基百科条目 URL（`Rust_(programming_language)`、`Mercury_(planet)`）大量带成对括号。
2. `文字 ![图](/x.png) 文字` 渲染成 `<p>…文字 <p style="margin:1.5em 0"><img …></p> 文字</p>`——段落里套段落，微信编辑器会把它拆成几块，排版直接散掉。

**根因：** `parse_link()` / `parse_image()` 取目标时都用 `position(|&c| c == ')')`，即「第一个 `)`」。这在纯 URL 上没问题，但 URL 语法允许括号成对出现；一旦 URL 里先出现 `(`，收尾位置就认错了。行内图片那条更直接：`inline_md()` 复用了「整行一张图」的块级样式（外层 `<p>`），可它被调用时外层**已经是** `<p>`——函数只关心自己输出什么，没有关心自己输出到哪。

**修复：** 新增 `destination_end()`，按括号深度配平找真正的收尾 `)`，`\(` / `\)` 视为字面量（`unescape_parens()` 还原）；深度始终不归零时退回「第一个 `)`」的老口径，避免把本来能用的 `/a(b` 变成纯文本。行内图片改为行内 `<img>`（`display:block` 保留视觉，但不再有 `<p>`），`src` / `alt` 与块级图 `render_image_figure()` 一并补上 `html_escape()`——含 `&` 或 `"` 的 URL 以前会把属性截断。

**防复发：** 五条 TDD：成对括号必须留在 `href` 里且 `</a>)` 不得出现；`\(` / `\)` 必须还原；不平衡左括号走老口径；`inline_md` 输出**不含** `<p`；`src` / `alt` 必须转义。另加块级图属性转义用例。受控变异：把 `if depth == 0` 改成永远成立、把行内图片换回 `<p>` 包裹且不转义，三条用例同时变红；恢复（sha256 校验）后 21 条行内用例全绿。真实二进制对同一篇探测稿：`href` 完整、嵌套 `<p>` 消失。

**证据：** `src/markdown/inline.rs`（`destination_end` / `unescape_parens` / `parse_link` / `parse_image` / 行内图片分支）、`src/markdown/plain.rs`（`render_image_figure` 转义）、`src/markdown.rs` 块级图用例。

**文章角度：** 两个 bug 是同一个思维习惯：**只处理自己看到的那一层**。解析 URL 时只看到「第一个 `)`」，没看到「括号是配对的」；渲染行内图片时只看到「图片应该有间距」，没看到「我被谁包着」。写解析器时问一句「这个分隔符真的不可能出现在内容里吗」，写片段渲染函数时问一句「调用我的上下文长什么样」，能挡掉这一整类问题。

## 2026-11-18 嵌套列表被压平成同级，引用里的列表漏出字面 `-` 号

**现象：** 一篇覆盖列表写法的探测稿喂给 `render`，两处都在微信里可见：

1. 顶层 `-` 项加缩进两格的 `-` 子项，渲染出两行 `padding-left:1.2em` 的段落，二级和顶层长得一模一样，读者看不出从属关系；`1.` 父项下的 `1.` / `2.` 子项排成「1. 1. 2. 2.」四个同级序号，像写错编号。
2. `> - 引用里的列表` 在引用卡片里原样输出「- 引用里的列表」，正文出现字面的连字符。

**根因：** 两处都源于「谁负责理解缩进和标记」。列表主循环把 `line.trim()` 交给 `parse_list_item()` / `parse_checklist_item()`，缩进在进入解析器之前就没了——而缩进是列表**唯一**的层级信号，`trim()` 一旦执行，层级信息不可恢复。引用分支更早：`render_blockquote()` 把每行直接丢给 `inline_md()`，而 `-` / `1.` 是块级标记，行内解析器不认识，于是沉默地原样保留。

**修复：** 解析入口改成接收原始行，用 `indent_columns()` 数前导空白（tab 按 4 列对齐），`ListItem` / `ChecklistItem` 带上 `indent`。层级在渲染时用**缩进栈**（`push_indent()`）算：比栈顶深就进一级、浅就弹回——这样 2 空格（`-` 的子项）、3 空格（`1.` 的子项）、4 空格（编辑器把 Tab 展开）三种习惯都只进一级，而不是「缩进 ÷ 固定步长」把 4 空格直接跳过两级。层级封顶三级（`MAX_LIST_DEPTH`），手机端正文窄，再深就把文字挤出可视区。`render_blockquote()` 逐行先认列表标记，命中就渲染成圆点 / 序号，层级用 `&nbsp;&nbsp;` 撑（引用卡片没有悬挂缩进，空 `span` 又会被微信清掉）。

**防复发：** 六条 TDD：二级项 `padding-left` 必须是 2.5em、三空格缩进的有序子项要进级、四级五级都封在 5.1em、4 空格只进一级、引用里不再出现字面 `- 第一条` 且有圆点、普通引用不许长出圆点。受控变异三组：缩进改回按 `trim()` 后的行算 → 4 条变红；引用退回纯 `inline_md` → 2 条变红；删掉 `.min(MAX_LIST_DEPTH)` → 封顶用例变红；恢复（sha256 校验一致）后 13 条全绿。另核对真实文章 `ready/gzh-tools-roundup-2026.md` 修复前后 HTML **逐字节相同**，确认只影响嵌套/引用列表，不波及现有排版。

**证据：** `src/markdown/plain.rs`（`indent_columns` / `push_indent` / `resolve_depths` / `parse_list_item` / `parse_checklist_item` / `render_blockquote` / `LIST_PADDING` / `CHECKLIST_PADDING`）。真实二进制复测同一探测稿：缩进从 `1.2 / 1.2 / 1.2` 变成 `1.2 / 2.5 / 2.5`；引用里两行从字面 `- 引用里的列表` 变成 `• 引用里的列表` + 缩进的 `• 引用里的二级`。

**文章角度：** 缩进不是「多余的空格」，它是列表唯一的层级信号——`trim()` 被调用的那一刻，列表就已经被压平了，后面再想恢复层级不可能。第二处更值得记：`-` 是块级标记，被喂给行内解析器时，行内解析器并没有错，它只是不认识，于是沉默地原样输出。管道两端各自都「做对了」，中间的语义落差就成了读者看到的字面连字符。

## 2026-11-18 链接标题被塞进 href，自动链接被拆成 `<<a href="…&gt;">`

**现象：** 一篇覆盖各种 Markdown 写法的探测稿，两处在微信里都是「点不开 / 看着怪」：

1. `[示例](https://example.com "标题文字")` 渲染成 `href="https://example.com &quot;标题文字&quot;"`——GFM 里这是「URL + 标题」，标题被当成 URL 的一部分，链接直接死。图片同病：`![替](/a.png "图片标题")` 输出 `src="/a.png &quot;图片标题&quot;"`，图片加载不出来。
2. `<https://example.com>` 渲染成 `<<a href="https://example.com&gt;" …>https://example.com&gt;</a>`——正文多一个裸 `<`，`href` 尾巴挂着 `&gt;`。

**根因：** 两处都是「语法标记没有从数据里剥离」。① `parse_link()` / `parse_image()` 把 `(` 与 `)` 之间的**整段**当目标，可 GFM 的目标语法是「URL + 可选标题」两段；括号里出现空白加引号时，实现看不出那是标题。② 自动链接的尖括号在 `inline_md()` 里没有分支：`<` 先按普通字符输出，`i` 前进一格后 `starts_url()` 认出 `https://`，`take_url()` 又不把 `>` 当终止符，于是 `>` 被吞进 URL。

**修复：** 新增 `split_destination()` —— 目标是 `<...>` 形式就剥尖括号并保留内部空格，否则在第一个空白处切掉标题（标题在微信里没有可见价值，`title` 属性会被编辑器剥掉，直接丢弃比留个半死属性干净）。新增 `parse_autolink()`，只在尖括号内以 `http(s)://` 开头且不含空白时成立，输出不带尖括号的链接；`take_url()` 增加 `<` / `>` 终止符，裸 URL 不再吞进 ASCII 尖括号。

**防复发：** 七条 TDD：双引号标题不进 `href` 也不漏进正文、单引号标题同理、图片标题不进 `src` 且整段被消费、自动链接的 `href` 与文字都不带 `&gt;` 且正文无裸 `<`、`<…>` 目标里的空格保留、无标题链接行为不变、`a < b` 这种普通比较不许被当自动链接。受控变异两组：`split_destination()` 原样返回 → 3 条变红；`parse_autolink()` 恒返回 `None` → 自动链接用例变红；恢复（sha256 一致）后 33 条行内用例全绿。真实文章 `ready/gzh-tools-roundup-2026.md` 修复前后 HTML 逐字节相同。

**证据：** `src/markdown/inline.rs`（`split_destination` / `parse_autolink` / `parse_link` / `parse_image` / `take_url`）。真实二进制复测同一探测稿：`href="https://example.com&gt;"` 归零、`<<` 归零、`src="/a.png &quot;` 归零，两条链接的 `href` 都是干净的 `https://example.com`。

**文章角度：** 「括号里就是链接」这个假设，在遇到第一对引号时就不成立了。解析器的失败往往不是崩，而是**把一个合法但更长的语法片段当成单个值**——调用方拿到一个能用的字符串，只是它包含了不该包含的东西，于是错误一路走到用户点不动的那一刻。判断依据要从「我截到了什么」换成「这个语法的完整形状是什么」：目标 = URL + 可选标题，而不是「括号内全部」。

## 2026-11-18 三级强调漏出裸星号，五六级标题把井号印进正文

**现象：** 同一篇探测稿里还剩两处「原始标记漏出」：

1. `***粗斜***` 渲染成 `<strong style="…">*粗斜</strong>*`——正文里多出两个裸 `*`，粗体里还夹着一个。
2. `##### 五级标题` / `###### 六级标题` 整行当正文输出，井号原样印在文章里。

**根因：** 两处都是「判定条件写窄了，且窄掉的那一档没有兜底」。① `inline_md()` 里 `**` 分支排在 `***` 前面，而它找收尾用的是 `windows(2)`：`***粗斜***` 第一个 `**` 命中后，中间是 `*粗斜`，收尾停在倒数第二个 `*`，最后剩一个。② `heading_parts()` 限定 `1..=4` 个 `#`，CommonMark 有六级，五六级直接落空，主循环把它当普通段落。

**修复：** 新增 `***` 分支并排在 `**` 之前（`windows(3)` 找收尾，输出 `<strong><em>` 嵌套）；`heading_parts()` 放宽到 `1..=6`，五六级沿用 h4 样式——微信正文没有第五套小标题层级，降级到 h4 比把井号印给读者正确。七个 `#` 仍是正文（超出 CommonMark 六级）。

**防复发：** 四条 TDD：`***粗斜***` 整体不含裸 `*` 且套上 `<em>` + `<strong>`、句中三级强调同样不漏、五六级标题渲染成 `<h4>` 且不漏井号、七个井号不被当标题。受控变异两组：`***` 分支失效 → 2 条变红；`1..=6` 退回 `1..=4` → 标题用例变红；恢复（sha256 一致）后全绿。真实文章 `ready/gzh-tools-roundup-2026.md` 修复前后 HTML 逐字节相同。

**证据：** `src/markdown/inline.rs`（`***` 分支）、`src/markdown/plain.rs`（`heading_parts`）。真实二进制复测同一探测稿：裸 `*` 计数 0、`#####` 计数 0，输出 `<strong style="color: #1a1a1a;"><em>粗斜</em></strong>`。

**文章角度：** 解析器最危险的形态不是「报错」，而是「认错了一个更短的前缀」——`**` 合法、`***` 也合法，先匹配短的那个不会失败，只会把剩下的那个字符留成噪音。分支顺序本身就是语法优先级，写的时候要按「最长匹配优先」排。另一处是同一件事的另一面：判定范围写窄了（只认四级标题），窄掉的输入不会报错，会静默降级成正文——凡是「有限枚举」的判定，都得给超出枚举的那一档一个明确归宿。

## 2026-11-18 `init` 生成的配置读不回来，新用户第一次 `render` 就报 invalid theme name（v0.4.14 因此撤包）

**现象：** v0.4.14 发版构建跑到 macOS / Linux 的打包 smoke（`init` → `new` → `render`）时整批失败：

```text
Error: invalid theme name: "default" # default | warm | dark | ...
```

而这条错误消息自己列出的第一个合法值就是 `default`。本地复现：`moonpub init --yes` 之后 `moonpub render`，退出码 1。

**根因：** 配置解析是手写的极简 TOML。`split_toml_pair()` 只在「整行以引号结尾」时才去引号：

```rust
let value = rest.strip_prefix('"').and_then(|s| s.strip_suffix('"')).unwrap_or(rest);
```

而 `init` 生成的配置自带行尾说明：`theme = "default" # default | warm | ...`。整行以 `gruvbox` 结尾，`strip_suffix('"')` 失败 → `unwrap_or(rest)` 把 `"default" # default | ...` 整段当成值。同一个坑还埋着 `auth_method = "appsecret" # ...` 与 `variant = "community" # ...`。

这个脏值此前**一直存在**，只是被「未知主题静默回落默认」盖住了；本轮把主题名收成有限枚举、未知名直接报错，脏值立刻变成硬失败。而 `init` 的测试只断言「文件内容包含 `theme = "default"`」——从来没把写出来的配置读回来跑一遍。

**修复：** 新增 `parse_toml_value()`：引号字符串读到配对的收尾引号为止（`\"` 不算收尾，引号里的 `#` 也不是注释），未加引号的值在第一个空格加 `#` 处截断，引号没闭合时退回原样。

**防复发：** 五条单元用例（行尾注释 / 未加引号的布尔 + 注释 / 引号内的 `#` / 转义引号 / 示例配置回读后主题合法）＋ 新增 `tests/first_run_contract.rs` 两条进程级契约：`init --yes` 后 `render` 必须退出 0 且产出 HTML；`check --json` 的 `effective_theme` 必须是 `default`。受控变异两组：退回旧解析 → 5 条单元 + 2 条契约全红；无条件在第一个 `#` 处截断（吃掉引号内的 `#`）→ 4 条红；恢复（sha256 一致）后全绿。这条链路以前只在 `release.yml` 的打包 smoke 里跑，等于「构建到一半才发现」——现在它跑在每次 PR 的 `test` job 里。

**证据：** `src/config.rs`（`parse_toml_value` / `split_toml_pair`）、`tests/first_run_contract.rs`；v0.4.14 的 release run 44234295565 中 linux-arm64 job 的 smoke 失败日志；本地 `init --yes` + `render` 复现与修复后退出码 0。

**文章角度：** 这是「静默兜底」的教科书案例：脏值存在了很久，因为下游有个「认不出来就用默认」的兜底，于是没人发现解析器从来没真正工作过。一旦把兜底换成显式报错，问题立刻浮出水面——**报错不是 bug，报错是把 bug 暴露出来的那一步**。真正的教训在第二层：任何「自己写出来的文件」，测试必须把它读回来跑一遍；只断言写入内容包含某个字符串，等于只验证了 `format!`，没验证解析器。

## 2026-11-18 引用块里的块级结构整类降级：嵌套引用漏 `>`、代码块吞掉语言标记、标题漏井号

**现象：** 一篇专测引用的探测稿，四处同时出问题：

1. `> > 内层一句话` 渲染成「外层一句话。> 内层一句话。」——内层那个 `>` 原样印进正文。
2. `> ```js` 包着的代码块变成行内代码，且语言标记被拼进代码正文：`` `jsconst a = 1;` ``。
3. `> ## 小标题` 把 `## 小标题` 当正文输出。
4. `>无空格引用`（CommonMark 合法）整行不被识别，字面 `>` 留在正文；引用里的空引用行 `>` 被整行丢掉，两段挤成一段。

**根因：** `render_blockquote()` 把引用正文当成「一串行内文本」处理——逐行 `inline_md()`，再拿 `<br>` 拼起来。于是引用里任何**块级**结构都没有解析器：`>` 不是行内标记，沉默保留；` ``` ` 不是行内标记，进 `inline_md()` 后只有反引号分支命中，语言标记与代码被拼成一个行内代码；`##` 同理。上一轮只给列表打了补丁（在引用里手写圆点 / 序号），于是「引用里有一套半套 Markdown」——列性能渲染，其余照旧漏。`strip_prefix("> ")` 又要求 `>` 后必须有空格，`>引用` 直接不成引用。

**修复：** 引用正文含块级结构时不再自己渲染，而是**剥掉一层 `>` 后交回 `render_markdown_segment()` 递归渲染**——嵌套引用、代码块、标题、列表、表格一次性全部由同一个块解析器负责，引用里不再有第二套语法。纯文字引用（无块级标记、无空行）仍走「逐行 `inline_md` + `<br>`」快路径，保住现有紧凑排版。新增 `strip_quote_marker()`：`>` 后空格可选、只剥一层；引用里的空引用行保留成空行，交回引用渲染器作段落分隔。

**防复发：** 五条 TDD：嵌套引用卡片数 ≥ 2 且可见文本不含字面 `> 内层`、引用里的代码块必须出现 `white-space:pre-wrap` 且不含 `jsconst`、引用里的标题必须渲成 `<h2>`/`<h3>`/`<h4>` 且不含 `## 小标题`、`>无空格引用` 必须被识别、引用里两段必须产出两个 `<p>`。测试里补了 `visible_text()` 去标签工具——第一版实现把「紧跟在标签后的字面 `>`」也当标签收尾吃掉了，导致「`>` 漏进正文」的断言永远抓不住，受控变异时才发现。三组受控变异：引用正文退回逐行 `inline_md` → 6 条红（含列表两条）；`strip_quote_marker` 要求 `>` 后必须有空格 → 2 条红；空引用行继续被丢掉 → 1 条红；恢复（sha256 一致）后 10 条全绿。

**证据：** `src/markdown/plain.rs`（`strip_quote_marker` / `quote_body_needs_blocks` / `render_blockquote` / 引用行累积分支）。真实二进制复测同一探测稿：字面 `>` 残留 0、`## 引用里的标题` 残留 0、`white-space:pre-wrap` 出现、`jsconst` 消失，引用卡片 5 个（嵌套那处渲成两层）。全仓库 82 篇 md 语料修复前后对比：只有 `docs/USER_ONBOARDING.md` 与 `_ZH` 两篇输出变化——源文件用 `>` 空行分成两段，此前被 `<br>` 拼成一段，现在正确分成两个 `<p>`。

**文章角度：** 这是「为修一个症状而长出一套平行实现」的典型：列表漏 `-` 号时，最省事的修法是在引用里手写一遍圆点和序号；可那样一来引用就有了一套自己的、只有一半的 Markdown，剩下的语法继续沉默漏出。正确的判断是问「这个容器里到底允许哪些块」，答案是「全部」——那就把正文交回同一个解析器，而不是在容器里再写一个。第 4 条是同一件事的另一面：`>` 后必须带空格这种「比规范更严」的判定，会把合法输入整类挡在外面；判定比规范严，输入就会在判定之外安静地变成正文。

## 2026-11-18 引用式链接：定义行被印成正文，`[文字][标签]` 从来没被解析过

**症状：** 渲染仓库自己的 `CODE_OF_CONDUCT.md`，一句话里同时露出两个缺陷：`This Code of Conduct is adapted from the [Contributor Covenant][homepage],` 把标记原样留在段落里（完全没有链接），文末的 `[homepage]: https://www.contributor-covenant.org` 变成一整段正文，里面的裸 URL 还被自动链接成「可点的正文」。

**根因：** 行内层只认一种链接写法 `[文字](url)`。引用式链接是**两段式**结构——正文里的标记 + 文档任意位置的定义行——而整条链路没有任何一步去收第二段。于是标记落进「认不出来」的分支、方括号照原样输出，定义行对块解析器来说只是又一行普通 Markdown：它变成段落，里面的裸 URL 又触发了自动链接。这个语法的两半都被当成内容渲染了。

**修复：** 在 `render_markdown_segment()` 入口加一道前置扫描。`collect_link_refs()` 走一遍本段，把每个 `[标签]: url` 行从正文里摘走、建一张 `LinkRefs` 表，块解析器此后只看见正文。`parse_link()` 补齐了另外三种形态——`[文字][标签]`、`[文字][]`（折叠式：文字自己就是标签）、`[标签]`（简写）——并到表里查目标。查询是**有条件的**：标签没定义就返回 `None`、方括号原样保留，所以正文里的 `[0]`、`[x]` 不会被「长得像」的语法吃掉。标签比对统一走 `normalize_label()`：大小写不敏感、内部连续空白折叠成一个空格，这是 CommonMark 的口径，否则 `[Homepage]` 与 `[homepage]` 会安静地配不上。定义表必须递归带下去，引用块里的 `> 见[文档]` 才能用到块外写的定义，本段自己的定义优先于外层同名定义。

**防复发：** 先写 10 条 TDD 再动手：定义行不产出正文；完整式 / 折叠式 / 简写三种都解析；未定义标签保持字面；标签忽略大小写与多余空格；代码围栏里的定义原样保留；引用块能继承外层定义表；引用式图片能解析；外加 `CODE_OF_CONDUCT` 真实两行摘录。其中一条断言被受控变异证明是弱的，必须换掉：`!visible_text.contains("[homepage]")` 在把缺陷改回来后依然绿——因为定义进了表之后，裸写的 `[homepage]` 会被简写引用解析成链接，**方括号本身消失了**，断言再也看不见它。真正能抓住定义行的是数段落元素（`<p` 出现次数 == 1）。三组受控变异：定义行留在正文 → 2 条红；`normalize_label` 退化成精确匹配 → 恰好 1 条红；关掉简写解析 → 2 条红。恢复后（sha256 校验一致，并 `touch` 刷新 mtime 防止 Cargo 复用旧目标文件）12 条全绿。

**证据：** `src/markdown/plain.rs`（`collect_link_refs` / `parse_link_definition` / `render_segment_with_refs` / 逐层传递 `LinkRefs`）、`src/markdown/inline.rs`（`parse_link` / `parse_image` / `normalize_label`）。真实二进制渲染 `CODE_OF_CONDUCT.md`：那句署名现在产出 `<a href="https://www.contributor-covenant.org">Contributor Covenant</a>`，定义行彻底消失。全语料回归 536 篇 md（仓库 + 一棵 `node_modules` 树当额外输入）：只有 2 篇变化，且都属正确修复。

**文章角度：** 链接定义是**关于文档的数据**，不是文档里的文字；解析器没地方放数据，就只能把它当文字。更有意思的是二阶效应：定义一旦能查，全文每一个方括号对都多出一种可能含义，所以这条修复必须加「标签真的存在」这个闸门。任何语法一旦变成有状态的，爆炸半径就是所有原本「同样字符但含义惰性」的位置。

## 2026-11-18 链接文字在第一个 `]` 处被截断，`[![徽章][图]][链接]` 渲染成乱码

**症状：** 这条不是用户报的，是给引用式链接做全语料对比时抓出来的。`obsidian-plugin/node_modules/moment/README.md` 从字面 `[![NPM version][npm-version-image]][npm-url]` 变成 `<a href="…v.svg">![NPM version</a>]<a>npm-url</a>`——比修复前更糟：锚点指向了**徽章图片**的地址，残留的 `]`、`![` 和第二个锚点撒进段落。同一形态在语料里另有 4 篇 README（typescript、两个 codemirror 包）；而行内变体 `[![a](x.png)](https://x)` 在本轮动手**之前**就已经同样坏掉了。

**根因：** `parse_link()` 用 `position(|&c| c == ']')` 找链接文字的结尾——也就是**第一个** `]`。但链接文字里允许再套一层方括号，而「可点击徽章」这个惯用法 `[![alt][img]][url]` 正是把图片套在链接里，第一个 `]` 属于图片。文字被截成 `![NPM version`，剩下的大半截被当成互不相关的行内语法重新解析。这个缺陷一直存在，只是此前不可见：引用式链接不通时，整个构造渲染成惰性字面文本，看起来像「还不支持」，而不是「算错了」。

**修复：** 抽一个共用的 `matching_bracket(chars, open)`：从 `[` 出发数嵌套层数、把 `\]` 当字面量，返回配对的 `]`。`parse_link()` 与 `parse_image()` 都改用它，于是链接文字是完整的嵌套构造，再交回 `inline_md` 渲染——内层图片走同一张引用表。输出现在是正确的 `<a href="链接"><img src="徽章" alt="alt"></a>`。

**防复发：** 两条 TDD，先红后绿：引用式 `[![NPM version][img]][url]` 必须产出 `href` 指向**链接**目标、`src` 指向**徽章**地址，且可见文本里不留 `][`；行内式必须同时保住外层 `href` 与内层 `src`。第三道检查是全语料——536 篇修复前后逐字节对比，4 篇徽章 README 现在都渲染成正确的嵌套锚点。

**文章角度：** 这个项目最便宜的模糊测试就是「把自家语料渲染两遍再 diff」。这个缺陷对特性自己的测试套件不可见，对本轮瞄着的那篇文件也不可见，只因为多问了一句「还有别的东西变了吗」才浮出来。更深一层：给语法加特性，等于**重新启用一批原本惰性的代码路径**。`[![a][b]][c]` 一直有这个问题，但在整个构造只是纯文本时没人看得见。一个语法开始工作之后，要回头重查旁边的解析代码——缺陷就睡在那里，等着这个特性把它变成可达。

## 2026-11-18 `status` 打印的文件名，它自己的 `render` 不认

**症状：** 全新的工作区、一篇文章、三条命令，第三条就失败了。`moonpub new "我的第一篇文章"` 建出 `Articles/drafts/我的第一篇文章.md`；`moonpub status` 把它列为 `我的第一篇文章.md`；把这一行原样粘回去 `moonpub render 我的第一篇文章.md`，得到 `Error: /…/我的第一篇文章.md: No such file or directory (os error 2)`。去掉扩展名也不行——`article path must point to a .md file`。**工具印出来的名字，不是工具自己的合法参数。** 另一个死胡同是空工作区：三行 `(empty)`、退出码 0、不给任何下一步提示。

**根因：** 两份清单各自长大了。`status --json` 的下一步判断格式化出的是 `Articles/drafts/{file}`——一个可用的路径；而人类看的 `format_status()` 只打印 `file.file`。同一段四分支的「下一步」逻辑还存在两份（`status_json()` 与 `next_workspace_action()`），逐字相同，所以没人注意到第三个使用者——文本输出——写在这两者之前，既没拿到路径前缀，也没拿到 `next:` 行。文本视图被当成了装饰，而不是接口。

**修复：** `format_status()` 每一行打印 `Articles/<stage>/<file>`，状态标注（`[rendered] ok`）排在路径之后，保证「第一个空白前的 token」永远是合法参数；末尾补上和 `workspace` 同款的 `next:` / `step:`。重复判断收敛进 `crate::status::next_action()`，`status` 文本、`workspace` 文本、`status --json` 三处共用——一个判断，三个出口。空工作区现在以 `next: moonpub new "你的第一篇文章"` 收尾，而不是三行 `(empty)`。

**防复发：** 两条 TDD。重要的是**往返测试**：解析自己打印的清单，取每行第一个 token，喂回 `check_article()`——工具读不了自己的输出就变红。第二条断言空工作区带 `next: moonpub new`。受控变异：去掉路径前缀，恰好往返测试红；去掉 `next:` 行，恰好空工作区测试红；两次恢复都用 sha256 校验。README 的 30 秒命令块不再靠读起来像对的，而是在真实临时目录里逐条跑通（中英文 slug 各一遍）。

**文章角度：** CLI 的输出**就是**它的 API——只要有人（或 agent）复制一行去执行，装饰就已经变成契约。`status --json` 早就正确，却对看着终端的人无用；机器比它的用户掌握更多信息。可推广的检查很便宜：**把自己打印的东西解析回来，喂进自己的入口。** 一个吃不下自己输出的命令不算做完，表格再好看也不算。

**证据：** `src/status.rs`（`next_action`、`format_status`、`status_text_prints_paths_that_are_valid_arguments`、`status_text_gives_next_step_when_workspace_is_empty`）、`src/protocol/workspace.rs`（两处调用点收敛到 `next_action`）。真实二进制：空工作区以 `next: moonpub new "你的第一篇文章"` 收尾；`new` 之后清单打印的 `Articles/drafts/我的第一篇文章.md` 原样粘进 `render` 成功。852 → 854 个测试，覆盖率 79.08%。

## 相关记录

- `AGENTS.md`：开发时必须遵守的当前约束和模块边界。
- `CLAUDE.md`：较早期的详细排障笔记，保留作背景资料；新结论以本文为准。
- `PROGRESS.md`：按时间记录已完成的功能、验证和发布事实。
- `docs/WECHAT_REGRESSION_CHECKLIST_ZH.md`：微信真实回归执行清单，不替代根因记录。
