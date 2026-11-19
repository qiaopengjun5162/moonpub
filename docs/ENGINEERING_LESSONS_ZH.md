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

### 配置文案也是用户文本：footer 里的一句 `<script>` 变成真标签

**现象：** 一篇探测稿在 `[footer]` 配置段写 `description: 描述 <script>x</script>`、`follow_text: 关注 <img src=x onerror=alert(2)>`，在 frontmatter 写 `footer_qrcode_note: 扫码 <div>`、`footer_qrcode: https://x.example/" onerror="alert(1)`。成品 HTML 里这些全是**真标签、真事件**：`<script>` 会执行、`onerror` 被注入到二维码 `<img>` 上。本地预览是在浏览器打开的，这是可直接触发的脚本执行面。

**根因：** `footer.rs` 里定义了 `html_escape()`，但只用在品牌卡片三字段（`brand_name` / `brand_avatar` / `brand_bio`）；其余六个文案字段（`divider` / `title` / `description` / `rules` / `qrcode_note` / `follow_text`）全部裸拼，两个 URL 字段（`qrcode` / `follow_image`）进 `src` 既不转义也不过协议过滤。前几轮的直觉是「渲染器里的用户文本要转义」，但 footer 走的是**另一条渲染路径**（配置 → 模板），被漏在审计之外。把「配置」当成可信常量，是这类漏洞的共同心理根源。

**修复：** ① 六个文案字段一律 `html_escape`；多行字段（`description` / `rules` / `qrcode_note`）走新增的 `escaped_multiline()`——**先转义文本、再把换行换成 `<br>`**，顺序反了要么把 `<br>` 转掉、要么留着注入。② 两个 URL 字段的 http(s) 分支走 `html_escape(safe_destination(x))`，先按**原始** URL 判协议、再转义（`https://…" onerror=…` 里的引号由转义封死在属性值内）；本地路径转出的 data URI 不过协议过滤（base64 会被换成 `#`），但仍转义防引号截断。③ 品牌头像 `avatar` 进 `src` 补上协议过滤。

**防复发：** 进程级契约 `escape_contract::footer_copy_and_urls_cannot_inject_markup` 用真实二进制渲染一份带 footer 配置和 frontmatter 覆盖的稿子，断言成品没有 `<script>` / `<div>` / `<img src=x` / `onerror="alert(1)`，且反转义后读者看得到字面标记、我们自己的 `<br>` 换行仍在。受控变异把 `description` 改回裸拼、把 `qrcode` 的转义去掉，契约立刻红。

**证据：** `src/footer.rs`（`escaped_multiline()` + 九处调用）、`tests/escape_contract.rs`。探测稿修复前 `<script>` / `<div>` / `onerror="alert(1)` 全部命中，修复后全为 0；182 篇语料新旧二进制 HTML **逐字节零变化**（仓库里没有在 footer 写危险标记，确认不误伤正常输出）。955 passed，覆盖率 79.41%。

### `href` / `src` 只转义还不够：协议本身也要过一遍

**现象：** 一篇探测稿里写 `[点我](javascript:alert(1))`、`![图](javascript:alert(3))`、`:::figure` 的 `image: javascript:alert(5)`，渲染出的 HTML 里这些目标原样保留。转义已经做对了（引号截不断属性），但浏览器照样会把一个合法的 `javascript:` 当协议执行。公众号后台大概率会过滤，**本地预览的浏览器不会**——从 RSS / 抓取内容里来的一条恶意链接，点一下就能执行。

**根因：** 此前所有目标只做了 `html_escape`，它解决的是「属性被引号截断」；协议是属性值的正常内容，转义动不了它。

**修复：** 新增 `src/html.rs::safe_destination()` 作为唯一入口，只放行 `http` / `https` / `mailto` / `tel` 与不带协议的相对地址，其余协议换成 `#`。判协议时按浏览器口径忽略前面的空白与控制字符、大小写不敏感；实体编码的协议头里的 `&` 会被转义成 `&amp;`，解码后只是普通文本。`inline.rs` / `blocks.rs` / `plain.rs` 的所有 `href=` / `src=` 拼接点统一接入，包含引用式链接、块级图片、`:::figure` 的 `image:`、`:::compact-links` 的 URL。

**防复发：** 进程级契约 `escape_contract::dangerous_url_schemes_never_reach_an_attribute` 用真实二进制扫所有属性；单测覆盖危险协议与正常目标各一组。受控变异把 `safe_destination()` 改成原样返回，契约与单测同时变红；恢复后 sha256 一致。

**证据：** 探测稿渲染后 `javascript:` / `vbscript:` / `data:` / `file:` 的目标全部变成 `#`，`https://…` / `mailto:` / `./a.md` 原样保留；181 篇语料新旧二进制逐字节对比 **0 篇变化**（语料里没有这些协议，不构成回归）；954 passed，覆盖率 79.26% → 79.29%。

### 行内元素之间的空格被后台删掉：文字粘在一起

**现象：** 「行内代码紧跟粗体」这种两个行内元素紧挨着写的句子，线上成品读成 `step_yulan调用它`；列表里的「正文紧跟行内代码、行内代码紧跟粗体」读成 `列表里a b 相邻`——连正文与元素之间的空格也一起丢了。把仓库里 181 篇可渲染文档全跑一遍，116 篇共 1753 处「夹在两个行内元素之间的纯空白文本节点」。

**根因：** 公众号后台的真实规则不是「删掉所有空白」，而是「文本节点里还有正文 → 空格转成 `&nbsp;` 留下；整段只有空白 → 节点直接删掉」。证据取自线上成品：`没有&nbsp;</span><span …><span leaf="">before</span></span>`——元素两侧的空格都被转成 `&nbsp;` 并单独成 span；而代码块里 `</span> <span>` 这种独立空白节点全部消失（`</span> <span` 计数为 0）。`inline_md()` 把空格原样 `s.push(ch)`，两个元素相邻时那个空格就成了独立空白节点。

**修复：** `inline_md()` 里「上一字符是 `>`（刚从元素里出来）且后面紧跟另一个元素」的空白落成 `&nbsp;`（`starts_inline_element()` 做前瞻）；正文与元素之间的空格保持裸空格——那是正常断行点，后台会替我们转成 `&nbsp;`。模板里手拼的四处一并改：`render_tip` 的 emoji 与正文、`compact-links` 的列间、术语表的表头与值，以及代码块窗口的三个圆点（`display:inline-block` 元素之间的空白会真的撑出缝隙，本地预览有缝、线上无缝，两边不一致）。顺带修 `:::tip` 的正文没走 `body_html()`——卡里的反引号会原样显示。

**防复发：** 单测 `whitespace_after_an_inline_element_is_escaped`（`</span>&nbsp;<strong` 必须出现，`</strong> 后面` 的裸空格不许被顺手改掉）；进程级契约 `no_blank_text_node_between_inline_elements` 用 `blank_between_inline_tags()` 全文扫「`</span>` / `</strong>` / `</em>` / `</code>` / `</del>` / `</a>` 后跟空白再跟行内开标签」，一处都不许有；契约里的 `drop_blank_text_nodes()` 也按真实规则修正（只有整段都是空白的节点才删，节点里还有正文时其前后的空白要保留）。

**证据：** `src/markdown/inline.rs`（空白分支 + `starts_inline_element()`）、`src/illustrate.rs`（tip 与三个圆点）、`src/markdown/blocks.rs`（compact-links、tip 改走 `body_html`）、`src/markdown/plain.rs`（术语表）、`tests/render_syntax_contract.rs`。受控变异：把条件改成 `&& false` → 单测与契约同时变红；恢复后 `shasum -a256 -c` 一致、951 passed、覆盖率 79.26%。语料新旧二进制对比：181 篇里 42 篇读者文本有变化，**全部是「补回原本被删掉的空格」**（`2026-08-10X 推文` → `2026-08-10 X 推文`），0 篇内容变化、0 篇变短。

### 正文里的一句 `<div>` 能吃掉半篇文章：用户文本必须转义

**现象：** 一篇覆盖各语法位置的探测稿（标题、正文、粗体、行内代码、链接文字、列表、引用、表格各写一处 `<div>` 和 `&`），成品 HTML 里全是裸标签。用浏览器解析成品，只读得到「标题里的」四个字——标题里那个 `<div>` 开出一个真 div，把后面的整篇内容装了进去。同一轮还查出三处**属性**注入：`:::figure` 的 `image: /a.png" onerror="alert(1)` 直接截断 `src`，`:::book-info` 的 `cover:`、`:::photo-grid` 的图片地址同病。

**根因：** `inline_md()` 的兜底分支 `s.push(chars[i])` 把正文原样写进 HTML；`blocks.rs` 的 `render_book_info` / `render_callout` / `render_figure` / `render_cover` / `render_photo_grid` 和 `illustrate.rs` 的 quote-card / divider / concept-card / emotion-card / timeline / comparison / tip 也都直接插值 props 与正文。此前只有行内代码、链接目标、图片 `alt` 做了转义——等于「看得见的那一半」修了，「看不见的那一半」没修。CommonMark 默认透传 HTML，但公众号不支持自定义标签，透传等于丢内容。

**修复：** `inline_md` 兜底分支转义 `<` 与 `>`；`&` 只在**不是**合法字符引用时转义（`&nbsp;` / `&#160;` / `&#x1F600;` 透传——作者写它是想让人看到空格，转义成 `&amp;nbsp;` 会让读者看到七个字面字符；这不构成注入面，解析器不会把解码出来的 `<` 再当成标签起始）；`blocks.rs` / `illustrate.rs` 的每个 props 值与插画字符串在进入 `format!` 之前 `html_escape` / `esc`。

**防复发：** 新增 `tests/escape_contract.rs` 两条进程级契约——① 九个语法位置各写 `<div>`，成品里不许出现 `<(div|b|i|script)`，且反转义后的可见文本要一字不差；② 卡片属性里带 `<div>` 与 `" onerror="`，成品里不许出现 `onerror="`（转义后只剩 `&quot;`）。受控变异：把 `'<' => "&lt;"` 改回裸 `<`、去掉 figure 的 `html_escape(image)` → 两条契约同时变红；恢复后 `shasum -a256 -c` 一致、全量转绿。判据一句话：**文本节点和属性是两条路，转义必须在进入 `format!` 之前完成，不能「先拼好再补」。**

**证据：** `src/markdown/inline.rs`（转义分支 + `starts_char_ref()` + 3 条单测）、`src/markdown/blocks.rs`（5 处 props 转义）、`src/illustrate.rs`（7 处插画字符串转义）。512 篇语料新旧对比：35 篇的可见文本变了，**全部是「旧版吞掉的内容被复原」**（例：`.agents/skills/lark-mail/SKILL.md` 里的 `Scans HTML <img src> local paths` 以前显示成 `Scans HTML  local paths`），没有一篇变短；`ready/` 下的真实成品文章 0 处变化。全量 945 → 949 passed，覆盖率 79.21% → 79.25%。

### 报错里建议的命令必须真的能修好它：`--render` 曾是空操作

**现象：** `stale_render` 明确让人「add `--render`」，用户照着做了一遍，推上去的仍是旧 HTML——盘上那份 `.html` 里的 `white-space:pre-wrap` 一个字都没变。报错说得笃定，用户按提示操作，结果原地打转，且这次没有任何新的错误信息提示他方向错了。

**根因：** 「要不要重渲」的判断写在 `if !draft_json.exists()` 里面，`--render` 只是这个分支的执行条件，不是独立意图。而 `stale_render` 的触发前提恰恰是 **`draft.json` 存在 + 产物过期**——也就是说，建议的命令在它唯一被建议的场景里恒为空操作。这是「文案承诺了一个实现没提供的能力」：错误提示与实际行为各自成立，只是对不上，而且两处都没有测试把它们绑在一起。更麻烦的是这段判断在 AppSecret 与 cookie 两条路径各写了一份，两边漏的是同一条。

**修复：** 把判断抽成 `push::render_gate(draft_exists, auto_render)`——`(_, true) → Render`，只要用户明确要重渲就无条件重渲，不看产物新旧；两条路径共用这一份，去掉「各写一遍、一起错」的机会。`stale_render` 的 `next_step` 同时改成 `moonpub push <article.md> --render`：报错在 `push` 里发生，就从 `push` 的命令重来，少一次「先 render 再手动重跑」的往返。

**防复发：** 真值表单测 `explicit_render_always_wins_over_an_existing_draft` 逐一锁住四种组合，第一条就是 `(draft 存在, --render)` → `Render` 并写明「否则报错的建议是假的」。这类「提示与实现漂移」值得单独立一条：**凡是错误文案里出现的命令或开关，都要有一条测试证明它在报错的那个场景里真的有效**，否则它是误导，比不提示更贵。

**证据：** 复现用一份 `white-space:pre-wrap` 的旧 HTML 加一个存在的 `draft.json`，`push --render` 后产物变成 `white-space:normal`，再跑一次不带 `--render` 直接放行到凭据检查（`missing env var: WECHAT_APPID`），说明闸门确实是「拦过期、放新鲜」。一篇正文写着 `white-space:pre-wrap` 的文章渲染后同样放行，证明收窄后的判据不会误伤。全量 972 passed。

### 产物存在不等于产物是新的：`push` 一直在推上一版 HTML

**现象：** 渲染器修完、`v0.4.19` 发出去、本机也重装了，用户再推一篇文章，线上排版依旧是旧的。查他的工作区：7 个已渲染 HTML 里 **6 个仍带 `white-space:pre-wrap`**，也就是全都出自旧二进制。修复没漏，只是**从来没有被用上**。

**根因：** `push` 只在 `<slug>.draft.json` **不存在**时才渲染；文件一旦存在，就直接复用磁盘上的 `.html`，既不看 Markdown 有没有改过，也不认这份 HTML 是哪个二进制写的。「渲染器修好了」和「产物更新了」是两件事，中间没有任何东西把它们连起来——而用户看到的永远是产物。这和前面那个「空主题名让 render 整条失败、盘上 HTML 从不更新」是同一个失效模式的两个入口：**只要产物是旧的，源码怎么修都没用**。

**修复：** 两条推送路径（AppSecret 与 cookie 会话）共用一个 `render::ensure_render_is_fresh()`，在**任何网络请求之前**执行。两条判据任一命中就报 `stale_render`：①产物的 `style` 属性里带 `white-space:pre*`——当前渲染器（`#228` 起）每行显式写 `normal`，绝不会输出这个值，出现即旧二进制遗留；判据只看属性，正文或代码里提到这个字符串不算（否则一篇讲 CSS 的文章会被永久拦住）；②Markdown 的 mtime 晚于 HTML——改了源没重渲。`git checkout` 或复制文件也会让 mtime 变新，误报的代价只是多跑一次 `--render`，比静默推旧内容便宜得多。**`preview` 接的是同一道闸**：打开浏览器之前就校验，否则作者会拿上一版的排版做判断，然后去改本来正确的代码——同一个失效模式，只是伤的是判断而不是读者。

**防复发：** 四条单测锁住闸门本身：旧签名必须报错且提示 `/--render/`、HTML 比源旧必须报 `Markdown is newer`（用 `File::set_modified` 把时间拨回一分钟，不依赖文件系统精度）、产物新鲜时必须放行（否则每次推送都要 `--render`，闸就白设）、正文里提到 `white-space:pre` 的文章不能被误判（判据只看 `style` 属性）。`preview` 另加两条：旧签名与源更新都必须在打开浏览器之前失败。另外新增 `ErrorCode::StaleRender`（`stale_render`）并接进 `help errors`，让 `classify()` 的穷尽性测试与注册表一致性测试一起盯着它。

**证据：** `src/push.rs` 与 `src/push_browser.rs` 在 `draft_json` 存在分支调用 `ensure_render_is_fresh`；`src/render.rs` 实现检查；`src/protocol/error.rs` 新增错误码。真实场景三连测：旧签名 → 拦、`touch` 源文件 → 拦、`touch` 产物 → 放行（随后才报缺 `WECHAT_APPID`，说明闸门确实在联网之前）。用户工作区 6 篇历史产物按 0.4.19 重渲后 `white-space:pre` 与 `</span>&nbsp;<span` **全部归零**。全量 967 → 974 passed。

### `&nbsp;` 不是免死金牌：判据是「节点里有没有可见字符」

**现象：** 上一条按「每个空格都写成 `&nbsp;`」修完、发出去之后，线上代码块**仍然**粘成 `letSome` / `ifmeta`，只是形态从「整行粘住」退化成「只剩跨 token 的空格丢」。用户提供的判断成了决定性线索：后台删的不是"空格字符"，而是"只含空白的文本节点"。

**根因：** 上一轮的修法把空格换成了 `&nbsp;`，但**位置没变**——`<span>let</span>&nbsp;<span>Some</span>` 里那个 `&nbsp;` 仍然是一个独立文本节点。它解码后是 U+00A0，在某些空白判定口径下（不只是 ASCII 的 `<= ' '`）依旧算空白，于是整段被删。**换成 `&nbsp;` 只改变了字符，没有改变节点结构。** 判据必须是「这个文本节点 trim 之后还剩不剩可见字符」，而不是「我用了哪个实体」。

**修复：** `xcode_highlight` 末尾统一走 `absorb_blank_nodes()`，把空白**搬进**相邻 token 的 `<span>` 内部：`</span>&nbsp;` → `&nbsp;</span>`，节点变成 `let\xa0`，trim 后还有 `let`，就不再是"纯空白节点"。三个边界单独处理：①行首缩进只有紧跟 token 时才搬进该 span，否则补 `&#8203;` 锚点——盲目搬会把「缩进后的 `--limit`」变成「`--` 后再补两个空格」；②空行补 `&#8203;&nbsp;`，否则整行只有一个空白节点，被删后塌成 0 高；③词间空白一律搬进前一个 token，不新开节点。

**防复发：** ①单测 `code_lines_never_leave_a_blank_text_node` 逐行解码实体后断言「不是纯空白」；②`word_spaces_survive_wechat_blank_node_deletion` **模拟后台动作**——把所有纯空白文本节点清空（保留标签），再断言 `let Some` / `if meta` / `else {` / 四空格缩进仍在。判据一句话：**别问「我用没用 `&nbsp;`」，问「这个节点里有没有能被看见的字」。** 上一轮的测试只检查 `</span> <span` 不存在，而 `</span>&nbsp;<span` 恰好绕过了它——**测试盯着错误特征时，改法只要换个特征就能骗过测试**。

**证据：** 16 行回归样例（`let Some(meta) = meta else { continue };`、8 空格缩进、行首 `--limit`、空行、中文注释）在模拟删空白节点后逐行与原码逐字一致，`letSome` / `ifmeta` 均为 0；`v0.4.19` 发布资产（sha256 `a921c9a9…`）重跑同一用例，产物 `white-space` 16 处全为 `normal`、`<pre>` 0 个、`</span>&nbsp;<span` 0 处。全量 965 → 967 passed。

### 代码块在公众号成品里连成一片：空格是字符，不是排版

**现象：** 线上文章（2026-11-18 22:20 发布）里代码块不再竖排，而是**整行粘住**：`npx tsx simple-indexer.ts` 显示成 `npxtsxsimple-indexer.ts`，`Found 20 signatures` 显示成 `Found20signatures`，`--limit 40` 显示成 `--limit40`。缩进还在（行首是 `&nbsp;`），词与词之间的空格却全没了，命令没法直接复制执行。顺带说明：那篇成品的样式串里仍有 `white-space:pre-wrap`，说明它是用 `v0.4.15` 旧二进制发的（`#217` 还没进用户手里的版本）；但即使换上 `#217` 的代码，词间空格照样会丢——两层是两个坑。

**根因：** 上一轮删掉 `white-space:pre-wrap`、改成「一行一个 `<p>` + 缩进用 `&nbsp;`」，但**词间空格仍留着一个裸 ASCII 空格**（`xcode_highlight` 里 `out.push(' ')`，当时的理由是"给浏览器留个断行点"）。公众号后台会把「只含空白字符的文本节点」整段删掉：直接查线上成品 HTML，`</span> <span` 出现 **0** 次、`&nbsp;` 出现 **428** 次——裸空格全灭、`&nbsp;` 全活。而代码高亮给每个 token 都包了独立 `<span>`，于是每一处词间空格都是「跨元素」的空格。同一个文本节点内部的空格不会被删（正文里 `课程的索引器 demo`、`20 条签名` 完好无损），所以这个坑只在代码块里显形。

**修复：** `xcode_highlight` 的空格分支删掉「留一个真实空格」那条路，**每个空格都写成 `&nbsp;`**，行首缩进与词间间隔同一套。代价是代码行没有软换行点，超宽行只能由 `overflow-wrap:break-word` 从容器边缘折断（`section` 上有 `overflow:hidden`，不折断就会被裁掉）。顺带修掉同函数里另一处一直静默的错误：兜底分支 `out.push(c)` 不转义标点，代码块里讲 HTML 时 `<div>` 会变成真标签被浏览器吞掉、`</T>` 会被当成结束标签把后面内容一起吃掉——标点现在走 `&lt;` / `&gt;` / `&amp;` / `&quot;`。

**防复发：** ①单测 `code_block_spaces_survive_whitespace_stripping`：成品里 `</span> <span` 一处都不许有，且把「只含空白的文本节点」删掉之后，四空格缩进 + `pure Rust` 仍要读得出来；②进程级契约 `code_block_words_survive_blank_node_stripping` 用 `drop_blank_text_nodes()` 把后台的删法模拟进测试，命令块删完仍要读成 `npx tsx simple-indexer.ts \`、第二行的两空格缩进与 `--limit 40 --reset` 也都要在；③单测 `code_block_escapes_markup_so_examples_survive`：`<div>` / `</div>` 不许漏成真标签。判据一句话：**空格必须是字符，不能是排版**——「我留个空格让它能断行」这种想法，在任何会重新序列化 DOM 的上游链路上都不成立。

**证据：** `src/illustrate.rs`（空格分支 + 标点转义 + 3 条单测）、`tests/render_syntax_contract.rs`（`drop_blank_text_nodes()` + 新契约）、`tests/directive_body_contract.rs`（`visible_text()` 现在认 `&nbsp;`）。真实文章 `2026-11-12-solana-indexer-lab.md`（17 个代码块 / 110 行）用新二进制渲染后逐行与源码比对：不一致 **0** 行、裸 `<` 泄漏 **0** 处。全量 943 → 945 passed，覆盖率 79.18% → 79.21%。

### 代码块在公众号后台竖着倒下来：`white-space` 是排版，不是结构

**现象：** 本地预览里代码块好好的，推送到公众号后台却变成「一个 token 一行」——`const` / `opts` / `:` / `any` / `=` / `{` / `limit` / `:` 竖着排下来，整块代码像倒下来的多米诺。代码块是顶格正文，外面没有任何 `:::` 包裹，所以不是卡片渲染的问题。

**根因：** 代码行靠 `white-space:pre-wrap` 保住空格与缩进。换行本来已经由「一行一个 `<p>`」保证，`pre-wrap` 多此一举，而且它还会把 HTML **源码里的每个换行**都当成真实断行。公众号后台的编辑器会把我们贴进去（或推送进去）的 HTML 重新序列化一遍——在元素之间插入换行——那些换行落进 `pre-wrap` 的元素里就变成可见的断行，于是每个 `<span>` 前面都多一行。本地看没问题，因为本地 html 是我们自己写出来的，元素之间没有换行。

**修复：** 代码行的排版改由「结构 + 显式编码」两件事负责，一点不碰 `white-space`：换行仍是一行一个 `<p>`；缩进与多空格全部落成 `&nbsp;`（`xcode_highlight` 的空格分支本来就在做，只是以前每段空格留了一个真实空格给浏览器当断行点）；制表符在渲染前换成 4 个硬空格（以前靠 `pre-wrap` 保留，现在必须自己来）；样式串里的 `white-space:pre-wrap` 直接删掉。`word-break:normal;overflow-wrap:break-word` 保留，长行仍在需要时断。

**防复发：** 判据一句话：**排版信息要么由结构给，要么显式编码，不能寄望上游保留某个 CSS 属性**——我们无法控制微信编辑器怎么序列化 HTML，能控制的只有「即使所有样式被改写，这行字仍然是对的」。测试锁在 `illustrate::tests`：成品里不许出现 `white-space`（出现就说明又被寄望了）、4/8 空格缩进必须是 `&nbsp;`、制表符不许漏进成品。同时把「这是不是代码块」的判据从 `white-space:pre-wrap` 换成 `font-family:Menlo`——用一个即将被删掉的属性当测试标记，删它的时候所有测试都会空转。

**证据：** `src/illustrate.rs`（`render_code_block` 删掉 `white-space:pre-wrap`、制表符换 4 空格 + 2 条新单测）；`plain.rs` / `blocks.rs` / `tests/directive_body_contract.rs` 里的代码块标记换成 `font-family:Menlo`。验证：全量 940 → 943 passed，覆盖率 79.13% → 79.18%。视觉零变化的证据用了两条：①同一篇文章改动前后各渲染一次，两版 html 去掉 `white-space:pre-wrap;` 声明后**逐字节相同**；②两版在无头 Chrome 里以 420px 宽截图，**两张截图的 sha256 相同**。语料回归 512 篇：415 篇仅差这条样式声明，1 篇（带制表符缩进的 JSON 示例）是制表符换成 `&nbsp;`，其余 96 篇无变化——没有任何内容差异。

> 当晚补记：这一条只修了「竖排」，词间空格当时还是裸空格，于是线上出现「连成一片」，见上一条。两条必须一起满足。

**文章角度：** 「样式应该是锦上添花，不应该是承载结构的那根梁」。一旦某个排版信息只存在于某个 CSS 属性里，你就把它交给了别人保管：浏览器、微信编辑器、任何一次 HTML 重新序列化都能没收它。代码块这种「空格就是语义」的地方，尤其要把信息写进字符本身。

### `:::` 块的正文也是 Markdown：代码块在卡片里被煮成一坨行内 span

**现象：** 把一段代码示例放进 `:::card` / `:::callout` / `:::intro`，成品里根本没有代码块：出现三个**空的行内代码 span**（围栏那两个 ```` ``` ```` 各自变成一个空 span），语言标记 `ts` 和整段代码挤进同一个 `family:monospace` 的 `<span>` 里，换行与缩进全靠浏览器猜。同一个根因还吃了两处：`:::intro` 里用一个**空行**分开的两段，被压成一行（`ready/gzh-tools-roundup-2026.md` 那篇真实文章第一段就是），而 `parse_blocks()` 用 `find("\n:::")` 扫原始字符串，代码块里教 `:::callout` 怎么写的那一行会被当成真指令，从半路把代码块切开——仓库自己的 `README.md` / `README_zh.md` / `docs/USER_GUIDE.md` 的模板示例都因此被渲染成了真卡片，围栏散架。

**根因：** `src/markdown/blocks.rs` 里所有「正文」类渲染器（`render_intro` / `render_callout` / `render_summary` / `render_letter_card` / `render_scene_card` / `render_closing_card` / `render_pull_quote` / `render_generic_fence`）一律 `inline_md(body)`：一整段**块级**正文被当成一串**行内**文本。行内渲染器只认得反引号、星号、链接，于是围栏退化成行内代码，块级语法全部原样漏出；HTML 里 `\n` 只是空白，两段自然合成一行。`parse_blocks()` 则是**字符串扫描**，完全不知道有这个语言里有「代码围栏」这回事——它跟上一轮修的 `~~~` 是同一个病：同一种语法，只有一处认。

**修复：** `blocks.rs` 新增 `body_html()`：正文先用 `plain::body_needs_blocks()`（与引用块共用同一份判据）判断，出现围栏/列表/标题/表格/引用/空行就交回 `render_markdown_segment()`，只有纯文字正文才走原来的 `inline_md`——视觉不变，`:::summary` 不会因为这次修复整体换排版。正文变成块级后外壳跟着换：`body_wrapper()` 在块级时出 `<section>`、纯文字时出 `<p>`（块级元素嵌在 `<p>` 里会被浏览器提前闭合，微信端再多拆一层）。`parse_blocks()` 改成**按行扫描 + 跟围栏状态**：`fence_open()` / `fence_closes()` 从 `plain` 开放给兄弟模块，代码块里的 `:::` 与块正文里的 `:::` 都不再被当成指令/收尾；收尾只认整行围墙（允许尾部空白与多余冒号），`::: 后面随便写` 不是收尾。

**防复发：** 两条判据：①**块里的正文还是 Markdown**——凡是"某个块的正文"（`>` 引用、`:::` 指令、将来的任何容器）都必须问一句「块级语法谁接住」，答不上就说明还差一个 `render_markdown_segment`；②**结构判定不许直接扫字符串**——`find("\n:::")` 不知道围栏、不知道行内代码、不知道转义，凡是「某一行是否有语法意义」的问题都得逐行走状态机。测试锁在两个层次：`blocks.rs` 三条单测（五种 `:::` 块里的代码块必须是代码块、块级正文不许套 `<p>`、纯文字正文保持原排版——最后一条是反向保险，防止为了修 bug 把所有卡片都改掉），`parser.rs` 三条单测（围栏里的 `:::` 不切文档、块正文里的围栏保留、`:::` 后带文字不算收尾），`tests/directive_body_contract.rs` 四条真实二进制契约。受控变异两组：`body_html` 退回 `inline_md` → 4 条红（单元 + 进程两层）；围栏状态失效 → 3 条红。

**证据：** `src/markdown/blocks.rs`（`body_html` / `body_wrapper` + 3 新单测）、`src/markdown/parser.rs`（重写 `parse_blocks`，行扫描 + 围栏状态 + 3 新单测）、`src/markdown/plain.rs`（`fence_open` / `fence_closes` 开放为 `pub(super)`，`quote_body_needs_blocks` 改名 `body_needs_blocks` 供两处共用）、新增 `tests/directive_body_contract.rs` 4 条进程级契约。真实二进制复测：`:::card` 里的 ```ts 现在是一行一个等宽 `<p>`、可见文本里没有 `ts`；带 `:::` 的示例代码三行全在；`:::summary` 里的代码块外壳是 `<section>`。改动前后两个二进制各渲染 512 篇真实文档，6 篇变化，全部属正确修复：README / README_zh / USER_GUIDE / LAYOUT_RECIPES_ZH / MOONPUB_INTRO_ARTICLE_ZH 是「围栏里的 `:::` 示例不再被执行」，`ready/gzh-tools-roundup-2026.md` 是「`:::intro` 里空行分开的两段不再压成一行」。全量 930 → 940 passed，覆盖率 78.85% → 79.13%。

**文章角度：** 「容器的正文也是文档」。我们花了好几轮给顶层正文补块级解析器，却忘了每个容器（引用、卡片、指令）都会长出一个自己的小正文——容器不只负责配色和内边距，它还得决定正文由谁解析。谁忘了这一条，谁就会在每个容器里养出第二套半残的 Markdown。

### 同一个文件，两种写法：工具印出来的路径粘不回自己的命令

**现象：** 在一个临时工作区里走新手路径，`moonpub new 测试文章` 印的是：

```
created
  /private/tmp/pathtest/Articles/drafts/测试文章.md
next: moonpub render /private/tmp/pathtest/Articles/drafts/测试文章.md
```

紧接着 `moonpub status` 印的是 `Articles/drafts/测试文章.md`。同一个文件，上一秒是这个字符串、下一秒是那个字符串。`render` / `check` / `preview` / `preflight` / `push` 全都跟着第一套走（`check` 连 `next:` 都是绝对路径），只有 `status` / `workspace` 走第二套——于是 `status --json` 的 `next_command` 是可粘的相对路径，`check --json` 的 `next_command` 是绝对路径，同一条命令的两个入口给出两个字符串。额外的坑在 macOS 上：绝对路径要过符号链接规范化，用户敲的是 `/tmp/x`，工具印的是 `/private/tmp/x`，两边字符串永远不相等，`diff`、`grep`、目视比对全部失效。

**根因：** 打印是**分散的**——每条命令各自把 `PathBuf` 交给 `.display()`，没有一处负责「同一个文件只许一种写法」。绝对路径能被印出来，只是因为工作区根恰好是绝对路径（`current_dir()` 的结果）；而 `status` 之所以印相对形式，是因为它手里只有文件名、临时拼了前缀，并不是有意统一口径。两套写法都是「顺手」的产物，没有一处是「决定」。

**修复：** 新增 `src/human_path.rs::display_path(root, path)` 作为唯一出口：路径落在工作区里就印**工作区相对形式**（`Articles/drafts/x.md`，与 `status` 列表逐字相同），落在工作区外就原样印绝对路径（用户显式指定别处的文章是合法用法，裁成一串 `../../` 只会更难核对），工作区根自己不会被印成空字符串，`/tmp/demo2` 也不会被误当成 `/tmp/demo` 的子路径（`strip_prefix` 按路径分量比，不是按字符串前缀）。分隔符统一成 `/`：`Path::display` 在 Windows 上印反斜杠，同一条命令在两边长得不一样，而它其实两边都能跑。`new` / `render` / `check`（文本与 JSON）/ `preview`（文本与 JSON）/ `preflight`（文本与 JSON）/ `push` 的 `moved to` / `cover` / `humanize` 全部改为经它打印——判断逻辑一个都没动，只把 `root` 传到打印点。

**防复发：** 判据是两句话：①**工具印出来的 token 必须能原样作为它自己的参数**（`new` 印出的路径直接粘给 `check`）；②**同一个文件在任何命令、任何模式下只许出现一种写法**（文本、JSON、清单、`next:` 引导共用一套）。测试锁在进程边界上（`tests/workspace_path_contract.rs`）：`new` 的路径不带工作区前缀且能跑通 `check`；`render` / `check` / `status` / `preflight` 对同一篇文章印同一个字符串；`status` 的 `next:` 拆成 argv 能跑通且跑完还是同一字符串；工作区外的路径保持绝对。受控变异是把 `display_path` 改成恒返回 `path.display()`——单元、模块级、进程级三层同时变红（12 个），说明这条口径不是靠某一个测试撑着的。

**证据：** `src/human_path.rs`（`display_path` + 6 个边界单测）、`src/draft.rs` / `src/render.rs` / `src/bundle.rs`（`report(root)`、`next_step(root)`）/ `src/preview.rs`（`next_step_after_preview(root, …)`）/ `src/push.rs` / `src/app_article_commands.rs` / `src/protocol/check.rs`（`check_json` / `preflight_text` / `preflight_json` / `preview_json` 增 `root`）、新增 `tests/workspace_path_contract.rs` 4 条真实二进制契约。真实二进制复测：`new` → `status` → `check` 三处对同一文件印同一个 `Articles/drafts/测试文章.md`，`check --json` / `preview --json` / `status --json` 的路径与 `next_command` 全部一致。改动前后两个二进制各渲染 512 篇真实文档（仓库全部 `.md`），html 逐字节零差异——本次不改渲染，只改打印。全量 920 → 930 passed。

**文章角度：** 「打印路径」看着像 formatting，其实是一份接口契约：用户复制粘贴的那一串字符，就是工具之间的握手协议。同一个东西给两种写法，用户就要多做一次心算——而新手的心算额度是零。

### 「推到哪儿了」只能看盘上的 `.media_id`，状态日志不是进度

**现象：** 一篇文章推到了微信草稿箱（`Articles/drafts/x.media_id` 里有 id），`moonpub status` 却把它显示成 `[rendered]`，和「只渲染过、还没推」一模一样；`moonpub preview` 看完本地预览，`next:` 还是 `moonpub push … --render`。用户想改错字得先 `update-draft`，那个 id 是工具生成的，屏幕上没有第二处能给出它，只能去 `cat` 一个隐藏文件。

**根因：** 三层叠在一起。①`status` 的 `drafts` 阶段根本不读 `.media_id`（只有 `ready` / `published` 拿它当状态回退）；②`read_bundle_media_id` 与 `bundle` 的判断各自存在，前者还被 `Result` 包着，看起来「只在需要时读」；③`preview` 的下一步是**写死**的字符串，`--json` 版更是自己又拼了一遍 `push`。真正的原因是状态来源选错了：`.moonpub/status.jsonl` 是**追加**日志——`push` 写一条 `ready + media_id`，用户改错字再 `render` 又追加一条 `rendered + 空`。只看最新一条，就等于把「已经推送过」这件事主动忘掉。而 `.media_id` 是**文件**，它是唯一不会因为「后来又跑了一条别的命令」而失真的记录。

**修复：** `StatusFileEntry` 增 `media_id`，所有阶段都从 `<slug>.media_id` 读；读取收成 `crate::bundle::read_media_id()`，`status`、`preview`、`bundle` 三处共用。文本清单在状态标注后面多印 `media_id: <id>`，位置仍在路径之后，保住「第一个空白前的 token 是合法参数」。`next_action()` 的阶梯补齐第四段，与 `ArticleBundle::next_step()` 逐段对齐：没 html → `render`；有 html 没 media_id → `check`；有 media_id → `preview`。`preview` 的文本版和 `--json` 版共用 `next_step_after_preview()`，已推送时给出 `update-draft` 而不是 `push`。JSON 的文件项收成同一个 `file_json()`，`status` 与 `workspace` 不再各写一遍。

**防复发：** 「这个包推到哪儿了」是**文件状态**，不是**操作历史**；任何用追加日志回答它的地方都会在「push 后又跑了别的命令」时失真。判据写进 `AGENTS.md`：`status` / `preview` / `check` 一律读盘，禁止读日志。第二层是「有没有 id」与「id 是什么」必须同一口径：`exists()` 会把一个空文件算成「已推送」，而 `fs::write` 中途失败正好会留下空文件——空串传给 `delete-draft ""` 报的错跟真实原因毫无关系，所以 `read_media_id()` 用「trim 后非空」。第三层是**同一条命令的文本和 JSON 必须共用判断**：`preview --json` 自己拼字符串那次，人看到 `update-draft`、脚本拿到 `push`，而脚本是自动化的那一端，错了没人会察觉。已推送时禁止再建议 `push`：`push` 不是更新，它新建草稿、删掉旧的、换掉 `media_id`，用户抄走的 id 和后台草稿链接一起失效。

**证据：** `src/bundle.rs` 的 `read_media_id_file()` / `read_media_id()` / `ArticleBundle::media_id()`；`src/status.rs` 的 `StatusFileEntry::media_id` 与补齐四段的 `next_action()`；`src/preview.rs` 的 `PreviewNext` / `next_step_after_preview()`；`src/protocol/workspace.rs` 的 `file_json()` / `stage_json()`。测试：3 个新单测（已推送草稿必须打印 id 且下一步是 `preview`、空 `.media_id` 不算已推送、`preview` 已推送时指向 `update-draft`）+ 新增 `tests/status_contract.rs` 5 个真实二进制契约（文本给 id、JSON 给 id、空文件不算、`preview` 文本指向 `update-draft`、`preview` 文本与 `--json` 的 `next_command` 必须逐字相同）。受控变异三处各自精准命中：`status` 不读 media_id → 2 契约 + 1 单测变红；`next_action` 不看 media_id → 2 契约 + 1 单测变红（空 id 那条仍然绿，说明它没依赖这条分支）；`preview` 永远建议 `push` → 1 契约 + 1 单测变红；`preview --json` 恢复自己拼 `push` → 只有「文本与 JSON 一致」那条变红。恢复后 `shasum -a 256 -c` 全部一致。

### 渲染器里"一行 = 一个块"，会把作者写的一段话拆成三段

**现象：** 文章里一段文字写了三行（中文写作最正常的样子：一句一行，写到句号换行）。成品里那三行变成三个独立段落，每段之间 1.5em 空隙——用户写的是一段话，读者看到的是三个短句。连带一个更隐蔽的后果：跨行的行内代码、加粗、链接全都被拆断，两半各看到奇数个标记，`` `intake photos --draft `` 换行接 `` photos/day1` `` 会把反引号原样印进正文。

**根因：** 块解析器把「一行」当成了「一个块」的唯一单位。段落分支收到一行就直接 `render_p(line)` 输出，没有"上一行还属于这一段吗"的概念。空行确实分段，但**非空行之间没有分段的依据**——CommonMark 里那是软换行，仍是同一段。这和列表续行是同一个病：行没有归属，被下游当成了新的块。

**修复：** 正文行攒进 `paragraph_buf`，由 `flush_paragraph()` 一次性输出；行间用 `<br>` 而不是空格连接（中文写作里换行有语义，「现象 / 原因 / 解决」写成三行，压成一句读起来像糨糊）。关键是**先拼再渲染**：`render_paragraph_lines()` 先 `join("\n")` 走行内渲染，再把 `\n` 换成 `<br>`——逐行渲染会让跨行的行内语法各看到奇数个标记。

**防复发：** 段落缓冲是最容易漏 flush 的缓冲，漏一处就是内容顺序错乱（读者能看见，但极难归因）。规则要写成「每个块级分支在自己那一行被认出来时必须先 flush」，并且区分两种 flush：围栏/分隔线/标题/图片/空行/文末用全量 `flush_blocks!()`；**表格续行、列表项、勾选项只能用 `flush_prose!()` + `end_blockquote!()`**——用全量 flush 会把刚推进去的表头/列表项提前打印，一个表格或列表被劈成多个 `<section>`（这个错误在本次修复的第一版里真的发生了，靠混排测试抓到）。测试要锁边界：三行一段必须只有一个 `<p>` 且两个 `<br>`、空行必须两段、跨行行内语法必须闭合、段落与标题/列表/表格的顺序必须正确。

**证据：** `src/markdown/plain.rs` 的 `paragraph_buf` / `flush_paragraph()` / `render_paragraph_lines()` 与 4 个新单测；`tests/render_syntax_contract.rs` 新增 2 个进程级契约；受控变异（段落分支里立刻 flush，还原"每行一段"）恰好命中 3 个测试变红（2 单测 + 1 契约），恢复后 `shasum -a 256 -c` 一致；80 篇真实文档改动前后各渲染一遍，13 篇变化，段落数显著下降（其中一篇 71 → 18）、列表项与勾选项数一字不变、可见文字 11 篇完全一致，另 2 篇各少一个反引号——正是跨行行内代码被正确闭合。

**文章角度：** 「一行」在 Markdown 里既可以是段落的全部，也可以是段落的一行。把两者当成一回事的渲染器，会让最自然的写作方式产出最不自然的结果。

### 一行没有归属的 Markdown，会被下游当成另一段话

**现象：** 文章里写 `- 输出都以 next: 收尾`，下面紧跟两行缩进的说明——中文写列表最常见的写法，一项写长了就换行。成品里那两行变成了顶格段落，还多一个段间距：列表只剩一行，剩下的文字看起来像另一段话。连带一个更隐蔽的后果：一行行内代码在换行处被切断（`intake photos --draft` 换行接 `photos/day1`），两半被打进两个段落，各自看到奇数个反引号，于是反引号原样印进读者可见的正文。

**根因：** `parse_list_item()` 只认以短横、星号或序号开头（后接一个空格）的行。缩进续行不匹配这些前缀，于是走到 `else if !list_buf.is_empty()` 那一支——列表先被 flush 收尾，这一行再继续往下走，落到段落分支。所以它不是「没被支持」，而是**被当成了另一种块**：本该属于列表项的内容，被下游认成了新段落。这类缺陷在真实语料里覆盖面很大——仓库自己的 `docs/` 就有 116 处这种写法。

**修复：** 加 `is_list_continuation(line, prev_blank)`，把缩进行拼进上一个列表项的文本；bullet、有序、勾选三条路径共用同一份判定。三个条件缺一不可：缩进至少 2 列（短横之后的内容从第 2 列开始，对齐到它才算「属于这一项」）、上一行不是空行（空行是用户明确的分段信号，跨过它继续拼就是替用户改结构）、自己不是别的块级语法（`#` / `>` / `|` / 分隔线宁可退化成正文）。判定必须排在 `parse_list_item()` **之后**，否则缩进两格的嵌套项（`- 子项`）会被上一项吃掉。

**防复发：** 为每个块级语法问一句「它的**另一种合法写法**由谁接住」。列表项有续行、标题有 setext、围栏有波浪线、表格有无边框——这些都是同一种块的次要拼写，只认主拼写就等于给下游留了一个「被当成别的块」的入口。测试要锁边界而不是锁结果：`- 项` + 两格缩进必须留在列表容器内、空行之后的缩进必须是独立段落、一格缩进不认、缩进的 `#`/`>`/`|`/分隔线不被吞、嵌套项在续行之后仍是独立项。

**证据：** `src/markdown/plain.rs` 的 `is_list_continuation()` 与 7 个新单测；`tests/render_syntax_contract.rs` 新增 2 个进程级契约（真实二进制渲染后断言续行文字在列表 `<section>` 内、空行之后的缩进段落在容器外）；受控变异（让 `is_list_continuation` 恒返回 false）恰好命中 5 个测试变红（4 单测 + 1 契约），恢复后 `shasum -a 256 -c` 一致；80 篇真实文档改动前后各渲染一遍，10 篇变化，段落数全部下降、列表项数一字不变、可见文字除一处外完全一致——那一处正好是跨行行内代码被正确闭合、两个反引号不再漏进正文。

**文章角度：** 「不支持」和「被当成别的东西」是两件事。前者用户能看见，后者用户只能看见结果不对——而后者往往来源更广，因为它是把一个块塞进了另一个块的解析路径。

### 同一种语法有两套判定时，漏掉的那一套会把代码煮成正文

**现象：** 文章里用波浪线围栏写代码示例（`~~~bash`），成品里围栏标记自己印成一段正文，围栏里的内容被当成普通 Markdown 重新排：`# 这不是标题，是注释` 变成小标题、`echo "**不该加粗**"` 变成粗体、`if [ a == b ]` 被 `==高亮==` 吃成 `<mark>` 且引号位置错乱、`- 这不是列表` 变成列表项、`| a | b |` 进了表格管道。更严重的一种：`~~~markdown` 里教「引用式链接定义」，那行 `[docs]: https://example.com/x` 被当成本文的定义抽走——代码块少一行，围栏外的同名引用还指向了「代码里的」地址。

**根因：** 同一个仓里对「什么是一段代码围栏」有三套判定：`article.rs::fence_marker` 认 ``` 与 `~~~`，`quote_body_needs_blocks` 也认两种，而真正渲染代码的两个入口（块解析器、`collect_link_refs`）只认反引号。于是 `~~~` 在「提取标题时跳过代码」这条路径上被当围栏，在「渲染」和「抽链接定义」这两条路径上被当正文。**判定分散 = 迟早有一处漏**，而漏的那处正好是产出成品的地方。

**修复：** 围栏判定收成一对函数——`fence_open()` 返回 (围墙字符, 连续长度, 语言标记)，`fence_closes()` 要求同字符、整行只有围墙字符、且长度不短于开头；块解析器、`collect_link_refs`、`quote_body_needs_blocks` 三处共用。收尾长度这条不是洁癖：`~~~` 围栏里的 `~~~~`、四反引号围栏里的三反引号都是代码内容，认成收尾会让外层提前关闭，后面的正文被整块吞进代码块。

**防复发：** 一种语法只要在仓里出现两处以上判定，就必须先合并成一个函数再谈修 bug；新增围栏字符只改那一处。测试要锁「围栏里的东西不会被另一套语法解释」而不是「围栏渲染成 `<pre>`」——前者能抓住变形，后者抓不住。单测覆盖四种围栏错配（短于开头、字符不同、围栏内出现更长波浪线、围栏内出现三反引号），契约测试用真实二进制断言可见正文里没有 `~~~`、代码原文没被改写。

**证据：** `src/markdown/plain.rs` 的 `fence_open` / `fence_closes` 与 4 个新单测；`tests/render_syntax_contract.rs` 新增 2 个进程级契约；受控变异（`fence_open` 只认反引号）恰好命中 4 个测试变红，恢复后 `shasum -a 256 -c` 一致；60 篇真实文档改动前后各渲染一遍，只有 2 篇变化，且都是「四反引号包住三反引号示例」的文档——旧版把内层三反引号当成收尾、外层提前关闭，41 段正文被吞进代码块，新版按 CommonMark 正确嵌套。

**文章角度：** 同一件事写两遍判定，就等于给自己留了一个「只在某条路径上出错」的 bug——而且最常出错的那条，正好是用户看得到的成品。

### 认不出的块级语法不是「退化成正文」，而是变形

**现象：** 文章里写 setext 标题（`社区营造` 下面一行 `====`），预览和最终草稿里标题降级成普通段落，紧跟一坨 `<mark style="…"></mark><mark style="…"></mark>==`——读者看到的是标签残渣，作者看到的是「我的标题去哪了」。

**根因：** 块解析器只认 ATX（`#`）这一半拼写，`====` 那一行没被任何块级分支接住，落进了行内渲染器；而行内渲染器支持 `==高亮==`，于是把整行 `=` 当成一对空的高亮标记 + 剩下的 `==`。**这才是真正的成本**：不支持的语法如果原样保留，用户顶多觉得"没渲染"；落到行内渲染器就会被另一套语法重新解释，输出里出现连作者都没写过的标签。同一根因族的另一个出口是 `~~~` 围栏（只认 ```），波浪线围栏里的代码会被当正文重新解析。

**修复：** 在段落分支前加一次前视：下一行若是整行 `=`（顶格或最多 3 个空格缩进），当前行按 setext 标题渲染成与 `##` 同级，并把下划线行吃掉（`is_setext_underline()` / `render_setext_heading()`）。`---` 刻意不收成标题——它在真实语料里更常是分隔线或 frontmatter 示例的边界，收进去会把「一行 `key: value` + 一条线」误升级成小标题；而它退化成「正文 + 分隔线」时读起来并不误导。`=` 没有第二重身份，所以只收它。

**防复发：** 新增任何块级语法的识别，必须同时回答「认不出来时这一行会掉到哪一层」；落进行内渲染器的块级标记（`=`、`*`、`_`、`~`、`|`）都要有「原样保留」的测试。块解析的单测必须覆盖「贴合法成立 / 空行隔开不成立 / 缩进超限不成立 / 混合符号不成立」四类边界，不能只测 happy path。渲染类修复的验收口径是**成品文件**：`tests/render_syntax_contract.rs` 用真实二进制渲染后断言 html 里有标题标签、且可见正文里一个 `=` 都不剩。

**证据：** `src/markdown/plain.rs` 的 `is_setext_underline` / `render_setext_heading` 与 6 个单测（`setext_equals_underline_becomes_a_heading_not_highlighter_debris` 等）；`tests/render_syntax_contract.rs` 两个进程级契约；60 篇真实文档（`docs/` / `output/` / `ready/` / `writing/`）改动前后用两个二进制各渲染一遍，`diff -rq` 仅状态日志不同，html 零差异——收窄到 `=` 就是为了守住这条零回归线。

**文章角度：** 「不支持」是安全的，「变形」才是危险的——解析器每多认一种语法，都要交代认不出时那一行进哪去。

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

### 合并发版 PR 的那次 push 会开出一个上百行的假发版 PR

**现象：** 每次发布后，main 上立刻多出一个 `chore(main): release X.Y+1` 的 PR，`CHANGELOG.md` diff 有 100 行上下，把 `#150`、`#164` 这类早就发过的 PR 重新列进新版本段落。2026-11-19 一天内连续出现五次（`#226` / `#232` / `#234` / `#236` / `#238`），每次都要人工判据 + 关闭。

**根因：** 发版流程里 tag 是**人工**建的：合并 release PR 之后要等 draft 出现，再 `gh release edit --draft=false` 才生成 tag。而合并动作本身会 push 一次，release-please 恰好在这个「tag 还不存在」的窗口里跑了一遍——找不到发布锚点，只能回溯到很早的提交重算 changelog。squash-merge 让 release PR 的 head SHA 与 main 上的提交 SHA 不同，那个 SHA 也当不了锚点。注意它与上面那条**不是同一个 bug**：这里 CHANGELOG 里没有 `release X.Y.Z` 自引用条目，特征是「历史重列」。

**修复：** 给 `.github/workflows/release-please.yml` 的 job 加一个 `if:`，跳过 `head_commit.message` 以 `chore(main): release` 开头的那次 push。跳过**不丢功能**：manifest 已经由发版 PR 写好，下一次真实提交会以刚建好的 tag 为锚点，正常开出下一版 PR。

**防复发：** 判据是「CHANGELOG 增量几十上百行」，不是「版本号跳了」——后者在正常发版时也会发生。这条也是「流程缺陷用代码堵」的例子：同一个动作人工重复五次之后，才值得把处置前移到一次条件判断；在此之前关闭它是对的，因为误关真发版 PR 的代价比多点五次更高。

**证据：** `#238` 的 CHANGELOG diff 为 `+102`（正常增量是 +7）；`v0.4.22` 发布后立即关闭，并在评论里写明锚点原因。`if:` 上线后由下一次发版验证：合并 release PR 不应再出现新的 release PR。

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

**防复发：** 五条 TDD：嵌套引用卡片数 ≥ 2 且可见文本不含字面 `> 内层`、引用里的代码块必须出现 `font-family:Menlo` 且不含 `jsconst`、引用里的标题必须渲成 `<h2>`/`<h3>`/`<h4>` 且不含 `## 小标题`、`>无空格引用` 必须被识别、引用里两段必须产出两个 `<p>`。测试里补了 `visible_text()` 去标签工具——第一版实现把「紧跟在标签后的字面 `>`」也当标签收尾吃掉了，导致「`>` 漏进正文」的断言永远抓不住，受控变异时才发现。三组受控变异：引用正文退回逐行 `inline_md` → 6 条红（含列表两条）；`strip_quote_marker` 要求 `>` 后必须有空格 → 2 条红；空引用行继续被丢掉 → 1 条红；恢复（sha256 一致）后 10 条全绿。

**证据：** `src/markdown/plain.rs`（`strip_quote_marker` / `quote_body_needs_blocks` / `render_blockquote` / 引用行累积分支）。真实二进制复测同一探测稿：字面 `>` 残留 0、`## 引用里的标题` 残留 0、`font-family:Menlo` 出现、`jsconst` 消失，引用卡片 5 个（嵌套那处渲成两层）。全仓库 82 篇 md 语料修复前后对比：只有 `docs/USER_ONBOARDING.md` 与 `_ZH` 两篇输出变化——源文件用 `>` 空行分成两段，此前被 `<br>` 拼成一段，现在正确分成两个 `<p>`。

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

## 2026-11-18 推荐的下一步，是个只读的死胡同

**症状：** `moonpub new` 之后，`status` 用 `next: moonpub check Articles/drafts/x.md` 收尾。照着跑，回四行结果加一句 `publishable: no`，然后结束——退出码 0，没有任何建议。**两跳，零进展。** 而同一个包上 `check --json` 返回的 `next_command` 是 `moonpub render …`：机器知道该干什么，坐在终端前的人不知道。链条还断在另外两处：`moonpub new` 只打印 `created <path>`，`moonpub render` 只打印 `rendered` 和两个文件路径——新手最先跑的两条命令，偏偏是唯一不说下一步的两条。

**根因：** 把「检查」当成了「推进」。`status` 挑出第一篇草稿就推 `check`，因为那是最安全的只读命令——可只读命令推不动工作区，推荐它只是把死胡同往后挪一跳。推荐逻辑也根本不知道这篇文章有没有渲染过：`status` 只能看见 `.moonpub/status.jsonl`，它可能不存在（`new` 不往里写）、也可能过期，而真正的事实只是 `<slug>.html` 在不在。另一头，`check` 的文本报告把 `publishable: no` 当最后一行就收尾，丢掉了 `check --json` 早就在跑的四分支判断——正是 `status` 那边刚消灭掉的「文本与 JSON 分家」。

**修复：** `StatusFileEntry` 增加 `has_html`，从文件系统取，不看状态日志；未渲染的草稿推 `moonpub render`，已渲染的仍推 `check`。文章包的四分支判断收进 `ArticleBundle::next_step()`，文本报告与 `check --json` 共用。`new` 以 `next: moonpub render …` 收尾，`render` 以 `next: moonpub preview …` 收尾，`preview`（早就有 `next:` 却没有 `step:`）补齐。整条上手链现在是 `new` → `render` → `preview` → `push`，每一跳都点名下一跳。

**防复发：** 三条 TDD：未渲染推 `render`、渲染后推 `check`（两次断言之间真的建出 `.html`）；包报告缺 `draft.json` 时说 `render`，齐全之后说 `push`；`new` 与 `render` 的输出必须包含各自的后续命令。受控变异：去掉包报告里的 next 行，恰好两条报告测试红；忽略 `has_html`，恰好草稿推荐那条红；同时去掉 `new`/`render` 的 next 行，恰好这两条红。恢复后 sha256 一致。最后用 release 二进制在临时工作区把整条链走了一遍，每条命令都是从上一条的 `next:` 里抄下来的。

**文章角度：** 推荐是一种承诺。判断一个「下一步」不能问「这条命令安全吗」，要问 **「跑完它，世界动了吗」**——只读检查是好的审阅工具，却是糟糕的下一步。第二个更便宜的追问：给人类看的那个出口，和给机器读的那个出口，取的是同一处数据吗？这里的 JSON 已经正确了好几个月，而三条文本命令一直扣着不说。

## 2026-11-18 缺微信凭据：报错只说缺什么，`doctor` 根本不看凭据

**症状：** 新手走完 `init → new → render → preview`，第一次 `push` 只得到一行 `Error: missing env var: WECHAT_APPID`——没说这值在哪、去哪拿，也没提还有一条完全不需要 AppSecret 的 cookie 通道。`push --json` 的 `next_step` 指向 `moonpub doctor`，而 `doctor` 从头到尾不看凭据（它自己的 `first_check` 还写着「环境变量来源（env > .env > toml）」）。同一份工作区、同一个 `WECHAT_AUTH_METHOD=COOKIE`，`ship` 走 cookie 通道，`push` 却退回 appsecret 要 AppSecret；等 toml 里出现 `auth_method = "appsecret"`（`init` 的默认值），`WECHAT_AUTH_METHOD=cookie` 被直接忽略。

**根因：** 三条路各自为政。① 错误只承担「报事实」，不承担「给出路」，而这一步恰好是用户唯一必须去机器之外取东西的环节。② `doctor` 把自己定义成「本地就绪检查」，却没把凭据算进「就绪」。③ auth_method 的判定写了两份：`push` 是 `cfg.wechat_auth_method.or_else(env)` 加 `== "cookie"`（toml 优先、区分大小写），`ship` 是 env 优先加 `eq_ignore_ascii_case`。同一份环境给出相反结论，用户只能靠试。

**修复：** 判定收进 `Config::is_cookie_auth(env)`（非空 env 覆盖 toml、大小写不敏感、两端空白不算），`push` 与 `ship` 共用。`MissingEnvVar` 在事实行之后固定给两条可照抄的路：cookie 通道（`moonpub login`，然后 `WECHAT_AUTH_METHOD=cookie moonpub push <article.md>`）与 AppSecret（微信后台 → 开发 → 基本配置，两行写进 `.env`）；不是微信变量的只补一句 set it in your shell or .env，不套用微信教程。`doctor` 增加 `wechat_auth:` 行（文本，以及 `doctor/v1` 的 `wechat_auth` 字段），说明 `push` 会走哪条通道、缺哪个变量、两条出路各是什么。

**防复发：** 单测锁四件事：toml 写 appsecret 时 env 的 cookie 必须赢；`COOKIE`、前后带空白的 `Cookie`、`cookie` 三者等价；env 为空回落 toml，两处都没写就是 appsecret；`wechat_auth_status_with` 四个分支全覆盖（缺两个 / 缺 secret / 缺 appid / 齐全，含「toml 里的 appid 算数、纯空白不算」）。`doctor` 这一行只陈述事实、刻意不加 warning：warning 条数取决于跑命令的机器有没有设环境变量，同一份工作区会在不同机器上打印出不同结构，测试也会在别人电脑上变红。release 二进制复测：无凭据 `push` 打印两条路；`WECHAT_AUTH_METHOD=COOKIE` + toml appsecret 现在走 cookie 并提示 `moonpub login`；`doctor` 三种状态分别打印 missing / ready / cookie。

**证据：** `src/config.rs`（`Config::is_cookie_auth` 及四条测试）、`src/push.rs`（改为调用同一判定）、`src/ship.rs`（同一判定，删掉重复实现）、`src/error.rs`（`missing_env_var_hint` 及两条测试）、`src/app.rs`（`wechat_auth_status` / `wechat_auth_status_with` 及两条测试）、`src/protocol/doctor.rs`（文本与 JSON 的 `wechat_auth`，两条测试）。真实二进制在 `/tmp/mp_cfg`：无凭据 `push` 打印两条路；`WECHAT_AUTH_METHOD=COOKIE` + toml appsecret 走到 cookie 会话错误；`doctor` 分别打印 `missing WECHAT_APPID + WECHAT_SECRET` / `ready` / `cookie`。测试 859 → 868。

**文章角度：** 「缺什么」是事实，「怎么补」才是帮助。一个错误信息如果在用户唯一无法自己解决的环节上只报事实，它就是把门锁上再把钥匙藏起来。更值得警惕的是第二层：给机器看的 `next_step` 指向的命令，自己也不知道答案——推荐一处，就得让那处真的能回答。

## 2026-11-18 cookie 模式没覆盖的命令，被缺凭据提示当成了出路

**症状：** 上一轮给 `AppError::MissingEnvVar` 加了「两条路」提示之后，`cookie mode (no AppSecret)` 这句出现在**每一个**缺凭据的失败里。可是 cookie 通道只实现了 `push` / `ship` / `delete-draft` 三件事（`push_browser` 里有建草稿与删草稿的封装，没有列表与改草稿的）。于是 cookie 用户跑 `list-drafts` 得到的是同一句「缺 WECHAT_APPID」，照着提示 `moonpub login` + `WECHAT_AUTH_METHOD=cookie` 再跑，还是同一句——**提示把他送回起点，命令却始终没跑成**。`update-draft` 更刺眼：它在 cookie 模式下的正确做法明明是 `moonpub push --render`（同样走 cookie、会重建草稿），报错却只字未提。

**根因：** 提示把「有一条不需要 AppSecret 的路」当成了通用事实，而它其实只是**部分命令**的事实。范围没有单一来源：文案里写死一个 `moonpub push` 的例子，读的人自然会把「cookie 能用」推广到当前这条命令上；代码里也没有任何地方记录「哪些命令支持 cookie」，因为支持与否是散在 `push_browser` 的函数名里的。同类毛病在项目里有过先例：`workflow-registry` 的 `safe_commands` 就是因为清单漂移才被测试锁住。

**修复：** 新增 `config::COOKIE_AUTH_COMMANDS = ["push", "ship", "delete-draft"]` 作为唯一事实，缺凭据提示、错误文案、文档都从它取；并新增 `cookie_auth_unsupported` 错误码：cookie 模式下跑没有实现的命令（`list-drafts` / `update-draft`）时，直接说「cookie 不覆盖这条命令」、「cookie 覆盖哪些」，再给一条真正能推进的替代（`update-draft` → `moonpub push <md> --render` 并说明 media_id 会变；`list-drafts` → `moonpub status` 看本地）。判定收进 `cookie_mode_block(appsecret_ready, is_cookie_auth)`：只有「选了 cookie 且没有 appsecret 凭据」才拦；凭据齐全时照旧走 appsecret（cookie 只是没实现，不是禁用），两样都没有且非 cookie 时仍交给 `MissingEnvVar` 报缺哪个变量。顺带把 `update-draft` 里第三份 appid/secret 读取删掉，改调 `wechat_client(cfg)`。

**防复发：** 五条断言各锁一面：① 提示里的覆盖范围必须等于 `COOKIE_AUTH_COMMANDS.join(" / ")`（写死文案就会红）；② 常量里的命令必须在 `help` 注册表里（防改名漂移）；③ `list-drafts` / `update-draft` **不得**出现在常量里（没实现就不许承诺）；④ `cookie_mode_block` 四个分支（cookie 缺凭据拦、cookie 有凭据放、appsecret 缺凭据放、都不是放）；⑤ `tests/json_error_contract.rs` 在真实二进制上跑两条命令，断言 `error.code == cookie_auth_unsupported`、`next_step` 是可执行命令、stderr 里有同一句话。受控变异：把 `list-drafts` 加进常量 → ③ 红；`cookie_mode_block` 退回 `is_cookie_auth` → ④ 红且 ⑤ 红（凭据齐全时也被拦）。

**证据：** `src/config.rs`（`COOKIE_AUTH_COMMANDS`、`appsecret_parts_from` / `appsecret_ready` / `cookie_auth_from_env` 及四条测试）、`src/push.rs`（`ensure_cookie_mode_can_run` / `cookie_mode_block` / `shell_quote`、`update_draft` 与 `list_drafts` 入口）、`src/error.rs`（`CookieAuthUnsupported`、提示中的覆盖范围）、`src/protocol/error.rs`（注册表第 25 个码、`classify` 分支）、`tests/json_error_contract.rs`。真实二进制（`/tmp/mp_cfg2`）：cookie 模式 `list-drafts` → `cookie mode does not cover \`list-drafts\``，`update-draft` → 附 `moonpub push /private/tmp/.../Auth-precedence-probe.md --render`；appsecret 模式缺凭据仍然是「缺 WECHAT_SECRET + 两条路」。

**文章角度：** 提示比错误更危险，因为人会照做。写「有一条路」时必须同时写清它通向哪里——范围不是措辞细节，而是承诺的边界。

## 2026-11-18 回退到一条跑不起来的通道，把真实原因也一起删掉了

**症状：** cookie 模式下 `delete-draft 2247483648` 输出两行，然后报了一个毫不相干的错误：

```
  ⚠ cookie 会话删除失败 (WeChat cookie session not ready: session.json 不存在，cookie 模式无法删除草稿
  run 'moonpub login' to scan the QR code, then retry push (cookie mode bypasses the IP whitelist))，回退 appsecret 通道
Error: missing env var: WECHAT_SECRET
  two ways forward:
    cookie mode (no AppSecret, covers push / ship / delete-draft): run `moonpub login`, then `WECHAT_AUTH_METHOD=cookie moonpub push <article.md>`
```

三个毛病叠在一起：① 真实原因（登录态没就绪）被后面那句「缺 WECHAT_SECRET」盖掉，`error.code` 也跟着撒谎（JSON 里是 `missing_env_var`）；② 那句提示的出路正是**刚刚失败**的 cookie 通道——闭环；③ 提示里写死 `moonpub push <article.md>`，而用户要删草稿，照着做只会多出一篇。同一个函数在 appsecret 模式下还会**每删一次**都打印一段用户从没选过的通道的警告，且因为把多行错误塞进括号，整句被截断成「半句英文半句中文 + retry push」。

**根因：** 回退（fallback）被写成了无条件动作。`delete_draft` 先试 cookie、失败就打一行警告、然后走 appsecret——可 appsecret 也需要凭据，凭据缺失时它唯一能做的就是把 cookie 的失败原因替换成「缺变量」。回退本身没检查「退过去的那条路能不能跑」，于是它退进了一个更没信息量的错误。另外两处文案各自写死了命令名：`CookieSessionRequired` 的模板固定 `retry push`，`MissingEnvVar` 的提示固定 `moonpub push <article.md>`——同一个模板被 `push` / `ship` / `delete-draft` 三条命令共用，模板里写死任何一条都会误导另外两条。

**修复：** ① 判定抽成 `cookie_delete_outcome(appsecret_ready, cookie_selected)` 三态：appsecret 能跑就回退（只有用户自己选了 cookie 才提示，否则这是他没选过的通道）；appsecret 跑不了且用户选了 cookie 就把 cookie 的真实错误原样抛出来（`cookie_session_required`，重试命令是 `moonpub delete-draft <id>`）；appsecret 跑不了且用户没选 cookie 就继续交给 `MissingEnvVar`（那才是这个模式下的实情）。② 警告只取错误第一行（`first_line`）：多行错误塞进括号会把整句截断，读者看到的只会是半句英文半句中文。③ `CookieSessionRequired` 增加 `retry` 字段，由调用现场填（`push`/`ship` 填 `moonpub push <路径>`，`delete-draft` 填 `moonpub delete-draft <id>`）。④ 缺凭据提示改成不点名任何命令：「`moonpub login` 一次，把 `WECHAT_AUTH_METHOD=cookie` 写进 `.env`，再重跑这条命令」——`.env` 在启动时会被读成环境变量，所以这句话对每条命令都成立。⑤ 非数字 id 只能走 AppSecret，凭据也缺时直接说清两条出路，不再绕回缺变量提示。

**防复发：** ① `cookie_delete_outcome` 四个分支各一条断言（有凭据×选没选 cookie / 没凭据×选没选 cookie）；② `first_line` 只留第一行；③ 错误文案测试断言 `then: moonpub delete-draft 2247483648` 且**不含** `moonpub push`、不含 `retry push`；④ 提示测试遍历 `push` / `ship` / `delete-draft` 三个命令名，断言一个都不出现，并断言出现「run this command again」；⑤ `tests/json_error_contract.rs` 用真实二进制跑 `--json delete-draft`（`HOME` 指向临时目录，保证既找不到真实登录态也不碰跑测试那台机器），断言 `error.code == cookie_session_required`、`next_step == moonpub login`、消息里带正确的重试命令、stderr 既没有 `missing env var` 也没有「改用 AppSecret 通道」。受控变异：把 `cookie_delete_outcome` 改回无条件 `Fallback { warn: true }` → ① 与 ⑤ 红；把 `first_line` 换成 `&err.to_string()` → ② 红；重试命令改回写死 `moonpub push` → ③ 红。

**证据：** `src/push.rs`（`delete_draft`、`CookieDeleteOutcome` / `cookie_delete_outcome` / `first_line` 及三条测试）、`src/error.rs`（`CookieSessionRequired { retry }`、`missing_env_var_hint` 改命令无关并加两条测试）、`src/push_browser.rs`（七处构造改带 `retry`，`build_cookie_header` 传入重试命令）、`src/protocol/error.rs`、`tests/json_error_contract.rs`、`src/help.rs`。真实二进制（`/tmp/mp_del`）：cookie 模式无凭据 → `WeChat cookie session not ready … then: moonpub delete-draft 2247483648`；appsecret 模式有凭据 → 一行警告都没有，直接走 AppSecret；cookie 模式有凭据 → 一行警告 + AppSecret；`.env` 里写 `WECHAT_AUTH_METHOD=cookie` 确认会被读到。全量 875 → 880 passed。

**文章角度：** 回退是唯一一种会**销毁证据**的错误处理方式：它把 A 的失败改写成 B 的失败，而 B 失败的原因往往与问题无关。回退之前先问一句「退过去那条路，现在跑得起来吗」；跑不起来就别退，把 A 的原话交给用户。

## 2026-11-18 参数写错一个字，诊断却指向整条通道

**症状：** `moonpub delete-draft R1LkThumb`（把素材 id 当成了草稿 id）在 cookie 模式下报：

```
Error: push failed: cookie 会话只能删数字 AppMsgId，`R1LkThumb` 不是；appsecret 凭据又缺失 —— …
```

`--json` 里三处全错：`error.code` 是 `push_failed`；`error.first_check` 是「先跑 `moonpub wechat-health`：区分凭证/IP 白名单/登录态」；`next_step` 是 `moonpub wechat-health`；连 `error.lesson` 都给了一条「微信 IP 白名单失败可能来自公网出口漂移」。也就是说，用户只是把 id 抄错了，却被系统送去排查一条完全健康的通道——查完当然什么也查不到。

**根因：** `PushFailed` 被当成了「微信侧做不了」的通用桶。它确实能装下「接口返回错误」，但这次根本没有请求发出去，问题在参数。错误码不是标签，它带着一整套诊断：`first_check`（先查什么）、`lesson`（这个坑踩过吗）、`next_step`（下一步做什么）。桶越大，指错路的概率越高——`push_failed` 的这三样全是通道视角的，装参数错误必然指错。同一处还有第二个浪费：`delete_draft_cookie_session` 的数字校验在**启动浏览器之后**才可能触发（实际调用方已提前拦过，成了不可达分支），而如果真从那条路进来，用户会先等一个 Chrome 起来。

**修复：** 新增 `AppError::InvalidMediaId { media_id, reason }` 与 `invalid_media_id` 码（注册表第 26 个）。消息第一行只点名字面量（形如 `invalid media_id: R1LkThumb`），第二行说清 id 从哪来（`moonpub push` 输出的 `media_id:`、或 `moonpub list-drafts`），第三行说本次的具体限制（cookie 只认数字 AppMsgId / 没有 AppSecret 凭据时可去后台删）。`first_check` 改成「先确认 id 来源」，`next_step` 是 `moonpub help delete-draft`——参数错误能推进的动作只有一个，就是看清用法；`lesson` 不登记（`push_failed` 那条白名单教训与参数无关，注册了就是误导）。`push_browser::delete_draft_cookie_session` 的入口校验改用同一个码，并把校验留在启动浏览器之前。

**防复发：** ① 错误文案测试：第一行等于 ``invalid media_id: `R1LkThumb` ``、不以 `push failed` 开头、含 `moonpub list-drafts`、总行数 ≤ 3（参数错误必须短）。② `tests/json_error_contract.rs` 真实二进制跑 `--json delete-draft R1LkThumb`：`error.code == invalid_media_id`、`next_step == moonpub help delete-draft`、`first_check` 不含 `wechat-health`、`lesson` 为空（这条最关键：它锁的是「不要给参数错误挂通道教训」）。③ `push_browser::tests`：直接调 `delete_draft_cookie_session("R1LkThumb")` 断言返回 `InvalidMediaId` 而不是 `PushFailed`——不启浏览器即可完成，也证明确实是早退。受控变异：把 `reason` 换成 `"push failed: …"` → ① 红；把 `classify` 改回 `PushFailed` → ② 红（`first_check` 与 `next_step` 两处同时暴露）；把 `delete_draft_cookie_session` 的守卫挪进 `block_on` 之后 → ③ 仍绿（它只看返回值），说明这条测试锁的是「返回什么错误」，而「不启浏览器」要靠代码位置保证——两者一起才完整。

**证据：** `src/error.rs`（`InvalidMediaId` 与文案测试）、`src/protocol/error.rs`（`ErrorCode::InvalidMediaId`、`ALL` 26、`as_str` / `summary` / `example_next_step` / `first_check` / `classify` / `all_failures`）、`src/push.rs`（非数字且无凭据的分支）、`src/push_browser.rs`（入口校验 + 单元测试）、`tests/json_error_contract.rs`。真实二进制（`/tmp/mp_chain`）：`delete-draft R1LkThumb` 文本三行、`--json` 的 `code` 为 `invalid_media_id`、`next_step` 为 `moonpub help delete-draft`、`lesson` 字段省略。

**文章角度：** 错误码是给人指路的，桶越大越容易指错。判断标准很朴素：这个码的 `first_check` 和 `next_step`，与用户实际做错的事有关系吗？没有就说明码选错了，而不是文案没写清。

## 2026-11-19 转义不是一件事：HTML、CSS、TOML、HTTP 头各有一套字母表

**现象：** 三个此前从未被审计过的出口，都能被普通用户输入触达。① 某篇文章的 frontmatter 写成 `title: "x\n[extra]\ninjected = true\n#"`，`moonpub export` 生成的 Zola 文件里标题值在第一个真实换行处提前闭合，`[extra]` 与 `injected = true` 成了真正的 TOML 字段，等于改写了下游站点配置。② AI 封面 HTML 用 `background-image:url('{png_name}')`，`png_name` 来自文章文件名，一个叫 `a';background:url(//evil)#.md` 的文件就能闭合 CSS 字符串注入声明。③ `moonpub init` 只转义 `"`，用户输入 Windows 路径 `C:\Users\me\articles` 会写出 `root = "C:\Users\me\articles"`，其中 `\U` 在 TOML 里是非法转义——它写出的配置自己读不回来。

**根因：** 前七轮修的是「用户文本进 HTML」，修多了就把「转义」当成一个动作、一套字母表。但安全性只相对于出口成立：HTML 正文要 `&<>`，HTML 属性还要管引号，URL 要协议白名单，TOML 基础字符串要 `\` `"` **外加** `\n` `\r` 和控制字符（真实换行会闭合字符串并另起一个键），CSS `url('...')` 要 `\'`，multipart 的 `filename="..."` 要挡引号与 CRLF（它是 HTTP 头，不是消息体）。这三处被漏掉，恰恰因为都不像 HTML——其中两处根本不是标记语言。还有一个更安静的问题：`escape_toml_string` 在 `export.rs` 和 `config.rs` 各有一份，函数体一模一样，漏的东西也一模一样。重复的转义逻辑不会随时间各自漂移，而是**一起漂移、一起保持错误**，所以审阅任一份都会看到一个「看起来合理」的函数。

**修法：** 统一到 `html::escape_toml_string`，覆盖 `\` `"` `\n` `\r` `\t` 与其它控制字符（按 `\uXXXX`），`export` / `config` / `init` 全部委派。`init` 里 footer 字段那三处手工 `.replace('\n', "\\n")` 必须同批删掉，否则真实换行会被二次转义成字面反斜杠加 n。`cover::escape_css_url_string` 负责 CSS 字符串口径（`\\`、`\'`、`\a` 与 `\d` 分别代表换行和回车，CSS 十六进制转义需要尾随空格来收尾，其余控制字符同样按十六进制）。`wechat::escape_multipart_filename` 由两条上传路径共用，且必须放在 `mime_for(filename)` **之后**，这样按扩展名猜 MIME 仍然看原始文件名。`config::unescape_toml_string` 补上 `\r` / `\t`，保证写得进也读得回。

**预防：** ① 每种字母表一个单测，断言载荷失去闭合容器的能力、普通值原样通过。② 一条端到端 `export` 测试：真的导出、截出生成的 frontmatter、断言没有任何一行以 `injected` 开头，且换行以字面 `\n` 留在标题值里——真正证明「文件仍是合法 TOML」的是解析回来，不是转义函数自己的输出。③ 断言写不变量、不写实现：「每个引号前都有反斜杠」，不是「输出里没有引号」。第一版用了后者，测试直接红了，因为转义并不删除引号、只是让它失效——这个红本身就是教训。

**文章角度：** 大家把「转义」当一个动词用，好像只有一件事。其实是四件事：正文、属性、URL、结构化值；再加两个容易忘的非 HTML 出口：CSS 字符串和 HTTP 头。判断标准不是「我调转义函数了吗」，而是「这个出口按哪套字母表解析，我说的是那一套吗」。

**证据：** `src/html.rs`（`escape_toml_string` 与两条单测）、`src/export.rs`（委派 + `export_cannot_inject_toml_fields_via_title`）、`src/config.rs`（委派，`unescape_toml_string` 接受 `\r`/`\t`）、`src/init.rs`（`toml_prompt_value` 委派；删掉三处手工换行替换；测试改名为覆盖引号/反斜杠/换行）、`src/cover.rs`（`escape_css_url_string` 与两条单测）、`src/wechat.rs`（`escape_multipart_filename` 与一条单测）、`src/push_browser.rs`（cookie 上传路径复用同一 helper）。956 → 962 tests，clippy 零警告，`fmt` 干净。

## 相关记录

- `AGENTS.md`：开发时必须遵守的当前约束和模块边界。
- `CLAUDE.md`：较早期的详细排障笔记，保留作背景资料；新结论以本文为准。
- `PROGRESS.md`：按时间记录已完成的功能、验证和发布事实。
- `docs/WECHAT_REGRESSION_CHECKLIST_ZH.md`：微信真实回归执行清单，不替代根因记录。
