# Changelog

All notable changes to MoonPub.

## [0.4.21](https://github.com/qiaopengjun5162/moonpub/compare/v0.4.20...v0.4.21) (2026-11-19)


### Features

* **cli:** 帮助系统改为命令注册表驱动 ([#150](https://github.com/qiaopengjun5162/moonpub/issues/150)) ([3d73291](https://github.com/qiaopengjun5162/moonpub/commit/3d73291d3aba6a9b57fcc25f8a2e73358cd0f906))
* **cover:** fit cover text to the canvas and add series tag ([3c6cba2](https://github.com/qiaopengjun5162/moonpub/commit/3c6cba21eaad916af6810088353fe36fdd075a29))
* **cover:** read cover fields from frontmatter and size titles by length ([1a456b7](https://github.com/qiaopengjun5162/moonpub/commit/1a456b7bde1d0908bfefd9a4d03d55747712de57))
* **cover:** 封面风格可发现，并把写错的 cover_style 拦在发布前 ([#157](https://github.com/qiaopengjun5162/moonpub/issues/157)) ([e93e6ea](https://github.com/qiaopengjun5162/moonpub/commit/e93e6ea1d9f01c6aea64ec3dd86657b8449d6d98))
* **cover:** 新增 frontmatter cover_style，cover 与 ship 共用风格解析链 ([#154](https://github.com/qiaopengjun5162/moonpub/issues/154)) ([7c6378c](https://github.com/qiaopengjun5162/moonpub/commit/7c6378c3ef5ec54ca31cbdf2e809ab7f185fdd6c))
* **cover:** 新增 swiss-dark 风格，swiss 封面把「N 字节」提成数据锚点 ([#156](https://github.com/qiaopengjun5162/moonpub/issues/156)) ([0d28ce3](https://github.com/qiaopengjun5162/moonpub/commit/0d28ce3fbc0826697c007f3a9b5d2e15584b3380))
* **help:** `--json help` 输出命令注册表，不再把整段帮助文本塞进 {"output"} ([#155](https://github.com/qiaopengjun5162/moonpub/issues/155)) ([d41053e](https://github.com/qiaopengjun5162/moonpub/commit/d41053ea5b5e3e2eb57b02691496158cf576edfc))
* **help:** 让 error/v1 错误码在运行时可枚举 ([#164](https://github.com/qiaopengjun5162/moonpub/issues/164)) ([a6b6b37](https://github.com/qiaopengjun5162/moonpub/commit/a6b6b372da189dcbd48fae4fa5d350461ac57ce9))
* **init:** add --yes flag for non-interactive setup ([d9f88c0](https://github.com/qiaopengjun5162/moonpub/commit/d9f88c08eed46376a76becb0d556e43229edf397))
* **preflight:** flag missing cover image before push ([be24d2a](https://github.com/qiaopengjun5162/moonpub/commit/be24d2af78b5fd0fa50c0ac02ac263672663dcc9))
* **preflight:** 新增 --strict，让发布前质量门能卡住退出码 ([#161](https://github.com/qiaopengjun5162/moonpub/issues/161)) ([a747940](https://github.com/qiaopengjun5162/moonpub/commit/a7479407dfc6c46374a3f58fbe96663f7367a462))
* **protocol:** --json 失败路径补 error/v1 结构化错误对象 ([#163](https://github.com/qiaopengjun5162/moonpub/issues/163)) ([2e232de](https://github.com/qiaopengjun5162/moonpub/commit/2e232de0bd543326373997372a491fb1ee585d87))
* **protocol:** 失败对象补齐 first_check 与 lesson 指针 ([a5de1ba](https://github.com/qiaopengjun5162/moonpub/commit/a5de1babf240ddfb120b94662120a723442820c7))
* **protocol:** 失败对象补齐 first_check 与 lesson 指针 ([#191](https://github.com/qiaopengjun5162/moonpub/issues/191)) ([a5de1ba](https://github.com/qiaopengjun5162/moonpub/commit/a5de1babf240ddfb120b94662120a723442820c7))
* **protocol:** 给全部命令级 JSON 载荷补 schema_version ([#167](https://github.com/qiaopengjun5162/moonpub/issues/167)) ([712cf5b](https://github.com/qiaopengjun5162/moonpub/commit/712cf5bc7fb05749703bdb39dd1b419d64ee6081))
* **protocol:** 给诊断/状态类 JSON 载荷补 schema_version ([#166](https://github.com/qiaopengjun5162/moonpub/issues/166)) ([21e567c](https://github.com/qiaopengjun5162/moonpub/commit/21e567c1b41d0162fc8608bf83ad99ead41afdc7))
* **protocol:** 错误码带诊断与教训指针，工作流带命令白名单 ([f3a559d](https://github.com/qiaopengjun5162/moonpub/commit/f3a559d80687e496f5187eb4fd5c0288269ee0e4))
* **protocol:** 错误码带诊断与教训指针，工作流带命令白名单 ([#189](https://github.com/qiaopengjun5162/moonpub/issues/189)) ([f3a559d](https://github.com/qiaopengjun5162/moonpub/commit/f3a559d80687e496f5187eb4fd5c0288269ee0e4))


### Bug Fixes

* **article:** BOM 不再吞掉 frontmatter，写反的方括号不再让整篇崩溃 ([#186](https://github.com/qiaopengjun5162/moonpub/issues/186)) ([6f7217b](https://github.com/qiaopengjun5162/moonpub/commit/6f7217bd1f4bd755397bace4907a74e9b66507c2))
* **build:** 声明真实 MSRV 为 1.88，并在 CI 验证 ([#151](https://github.com/qiaopengjun5162/moonpub/issues/151)) ([a46a921](https://github.com/qiaopengjun5162/moonpub/commit/a46a921edab16f321fe1ba4fcd883d594d1445e6))
* **cli:** configure 拼错步骤名不再静默跳过 ([#179](https://github.com/qiaopengjun5162/moonpub/issues/179)) ([c081277](https://github.com/qiaopengjun5162/moonpub/commit/c0812777d52c909c744cc5c6833bb6874235eab7))
* **cli:** cookie 模式没覆盖的命令不再把人送回起点 ([#207](https://github.com/qiaopengjun5162/moonpub/issues/207)) ([c021a04](https://github.com/qiaopengjun5162/moonpub/commit/c021a04dabc1a792505f90fb651a4b1638366f17))
* **cli:** delete-draft 不再退进一条跑不起来的通道 ([#208](https://github.com/qiaopengjun5162/moonpub/issues/208)) ([2ac4e54](https://github.com/qiaopengjun5162/moonpub/commit/2ac4e54e90ae81aebdf5d30b43ece376822708aa))
* **cli:** init 拒绝第二个位置参数，子命令 --help 扫描所有位置 ([#178](https://github.com/qiaopengjun5162/moonpub/issues/178)) ([147a147](https://github.com/qiaopengjun5162/moonpub/commit/147a147dc1413f14435e3d0087fbc164dd69a585))
* **cli:** intake 的 flag 与位置参数顺序无关，带值 selector 不再吞开关 ([#175](https://github.com/qiaopengjun5162/moonpub/issues/175)) ([8cd7a84](https://github.com/qiaopengjun5162/moonpub/commit/8cd7a84d06f9a2db2b408d1f67d1be6a1a904a2c))
* **cli:** radar 六个子命令统一收口，带值 flag 不再吞开关 ([#177](https://github.com/qiaopengjun5162/moonpub/issues/177)) ([1e44aa8](https://github.com/qiaopengjun5162/moonpub/commit/1e44aa8082e6ccc1a2ff9490aa000bf445a39582))
* **cli:** status 打印的路径能直接当下一条命令用，README 首屏一句话说清 ([#203](https://github.com/qiaopengjun5162/moonpub/issues/203)) ([deaeba3](https://github.com/qiaopengjun5162/moonpub/commit/deaeba38d995d7e71a4167cfca59acfd240342df))
* **cli:** 位置参数与布尔 flag 顺序无关 ([#168](https://github.com/qiaopengjun5162/moonpub/issues/168)) ([2b76a57](https://github.com/qiaopengjun5162/moonpub/commit/2b76a5745cf402c6c751e9c0daeffe2f95f89bf2))
* **cli:** 同一个文件不再有两种写法，工具印出的路径都能粘回自己的命令 ([#215](https://github.com/qiaopengjun5162/moonpub/issues/215)) ([2ea283f](https://github.com/qiaopengjun5162/moonpub/commit/2ea283fd9bf4b4b54e5c9d100906521427b2a1b0))
* **cli:** 后置 --json 在 help / version 上不再静默失效 ([#162](https://github.com/qiaopengjun5162/moonpub/issues/162)) ([d652527](https://github.com/qiaopengjun5162/moonpub/commit/d6525277a6ffb692e35b49e7b4cb931a302fb5d1))
* **cli:** 固定参数个数命令不再静默吞掉多余参数 ([#173](https://github.com/qiaopengjun5162/moonpub/issues/173)) ([7f0b903](https://github.com/qiaopengjun5162/moonpub/commit/7f0b90380b1f0ebc034886e1818cb503cf0e0a2e))
* **cli:** 带值 flag 命令支持 flag 前置且不再静默吞参数 ([#171](https://github.com/qiaopengjun5162/moonpub/issues/171)) ([dd34102](https://github.com/qiaopengjun5162/moonpub/commit/dd3410211a1ef32414a61933f3fc85529e1b6625))
* **cli:** 推荐的下一步必须真能推进，check/new/render 都以 next 收尾 ([#204](https://github.com/qiaopengjun5162/moonpub/issues/204)) ([f138b53](https://github.com/qiaopengjun5162/moonpub/commit/f138b53dc13ab2d3cdefd99577ed0164d65f61b8))
* **cli:** 缺微信凭据不再是一句死胡同，doctor 终于看凭据 ([#206](https://github.com/qiaopengjun5162/moonpub/issues/206)) ([#206](https://github.com/qiaopengjun5162/moonpub/issues/206)) ([e18450d](https://github.com/qiaopengjun5162/moonpub/commit/e18450d4e658710fb42a3f6d448cfb25d3606176))
* **cli:** 自由文本命令不再把标题砍成第一个词，手写循环也不再吞参数 ([#174](https://github.com/qiaopengjun5162/moonpub/issues/174)) ([2f1962d](https://github.com/qiaopengjun5162/moonpub/commit/2f1962d5291695e7b2b9cf33c9012908a3199246))
* **cli:** 裸跑 moonpub 的第一行就是该跑的命令 ([#205](https://github.com/qiaopengjun5162/moonpub/issues/205)) ([69c65da](https://github.com/qiaopengjun5162/moonpub/commit/69c65da3f5297b9ffeda4ab24500061a92446d21))
* **cli:** 配置发现不再按「第一个非 flag token」猜路径 ([#176](https://github.com/qiaopengjun5162/moonpub/issues/176)) ([a2fa930](https://github.com/qiaopengjun5162/moonpub/commit/a2fa9307053212332c4f5a52dd6ce57f58855781))
* **cli:** 零 flag 命令不再静默吞掉多余参数 ([#169](https://github.com/qiaopengjun5162/moonpub/issues/169)) ([7410fae](https://github.com/qiaopengjun5162/moonpub/commit/7410fae889f0122590dd09ad4dfb9be568aba503))
* **cli:** 非数字 media_id 是参数错误，不是通道错误 ([#209](https://github.com/qiaopengjun5162/moonpub/issues/209)) ([6881b26](https://github.com/qiaopengjun5162/moonpub/commit/6881b2693e8d056e43e02167900c50c185cdaccf))
* **codeblock:** 代码块不再依赖 white-space，公众号后台里不再一个 token 一行 ([#217](https://github.com/qiaopengjun5162/moonpub/issues/217)) ([ac54d9f](https://github.com/qiaopengjun5162/moonpub/commit/ac54d9ff0f1cd3132149906099c90cf11d3fa148))
* **codeblock:** 代码块里的空格改成 &nbsp;，公众号成品不再连成一片 ([#218](https://github.com/qiaopengjun5162/moonpub/issues/218)) ([d407a09](https://github.com/qiaopengjun5162/moonpub/commit/d407a0970ef88ed8a512cac39948ae6daffaee8a))
* **codeblock:** 空白不再单独成节点，词间空格不会被后台删掉 ([#231](https://github.com/qiaopengjun5162/moonpub/issues/231)) ([c2da0d2](https://github.com/qiaopengjun5162/moonpub/commit/c2da0d200a34f38ca560e543d2f8928cc32e20ef))
* **config:** init 生成的配置能被自己读回来，行尾注释不再污染主题名 ([#197](https://github.com/qiaopengjun5162/moonpub/issues/197)) ([9228cd0](https://github.com/qiaopengjun5162/moonpub/commit/9228cd02668dd7b1b11c7190fdcc29348c3b10af))
* **cover:** use generated cover.png when frontmatter has no cover field ([b837ac0](https://github.com/qiaopengjun5162/moonpub/commit/b837ac03e1392a04da35811565f6271b21f58b88))
* **cover:** 中文长标题生成 ai-lab 封面不再 panic（UTF-8 char boundary） ([#170](https://github.com/qiaopengjun5162/moonpub/issues/170)) ([fc9aac2](https://github.com/qiaopengjun5162/moonpub/commit/fc9aac2985be1f92cf277386243fbc8fd517d7f6))
* **cover:** 显式 cover_subtitle 不被 cover_title 逗号覆盖 ([c1608c2](https://github.com/qiaopengjun5162/moonpub/commit/c1608c217de383c415161fe1880b56b4f823e992))
* **docs:** 安装链接改用 latest，版本口径钉成契约测试 ([07ede1e](https://github.com/qiaopengjun5162/moonpub/commit/07ede1ebef50e52dc42fa58d526d28f4347cb5cf))
* **docs:** 安装链接改用 latest，版本口径钉成契约测试 ([#190](https://github.com/qiaopengjun5162/moonpub/issues/190)) ([07ede1e](https://github.com/qiaopengjun5162/moonpub/commit/07ede1ebef50e52dc42fa58d526d28f4347cb5cf))
* **doctor:** 警告逐条成行，并把 first-run 的 next_step 改成可执行动作 ([#153](https://github.com/qiaopengjun5162/moonpub/issues/153)) ([c4a14c9](https://github.com/qiaopengjun5162/moonpub/commit/c4a14c934e4559194934a641127063a9ee4d53b2))
* **footer:** 配置文案与二维码地址不再注入 HTML ([#223](https://github.com/qiaopengjun5162/moonpub/issues/223)) ([60b30aa](https://github.com/qiaopengjun5162/moonpub/commit/60b30aa4f97475e6b402fc0a7f36ee28c347eff2))
* **help:** radar 帮助页与报错提示补全 suggest / scrape 子命令 ([#158](https://github.com/qiaopengjun5162/moonpub/issues/158)) ([6922240](https://github.com/qiaopengjun5162/moonpub/commit/6922240ce487e7ba101f2189e9c7e64969f5056f))
* **humanize:** 中英混排文章不再 panic，30 字节回退窗口对齐字符边界 ([#182](https://github.com/qiaopengjun5162/moonpub/issues/182)) ([6accdbc](https://github.com/qiaopengjun5162/moonpub/commit/6accdbc555295fc1d6f64910ca55eb7ac39bf6ba))
* **illustrate:** 代码块词间保留真实空格，避免从单词中间硬断 ([6b943f7](https://github.com/qiaopengjun5162/moonpub/commit/6b943f770fcd8b5d421b69fb776cc472f260ada0))
* **layout:** 代码行显式归零 white-space，预检拦下 pre 系列旧产物 ([#228](https://github.com/qiaopengjun5162/moonpub/issues/228)) ([40421c5](https://github.com/qiaopengjun5162/moonpub/commit/40421c5f19bc0cf0f93a66a6362fb63832ff029f))
* **layout:** 排版审计仅匹配带引号的真实属性 ([69668ed](https://github.com/qiaopengjun5162/moonpub/commit/69668ed95314ea68c54a9d0bb9c002721628bbcd))
* **markdown:** ::: 块的正文也走块解析器，卡片里的代码块不再是行内 span ([#216](https://github.com/qiaopengjun5162/moonpub/issues/216)) ([e1f4c3c](https://github.com/qiaopengjun5162/moonpub/commit/e1f4c3cfbbceaaf24eb8e9978de5719fff83c940))
* **markdown:** 同步有序列表测试断言，修复 main 上失败的用例 ([7eddfbb](https://github.com/qiaopengjun5162/moonpub/commit/7eddfbb9fb9c37d9e5da57c80a7273925b8784e9))
* **markdown:** 孤立 | 行不再让整篇文章渲染崩溃 ([#185](https://github.com/qiaopengjun5162/moonpub/issues/185)) ([f389f49](https://github.com/qiaopengjun5162/moonpub/commit/f389f49edfcab672a3fdfe5a20421b165e75515b))
* **markdown:** 无空格井号标题与表格管道符不再泄漏到微信正文 ([#184](https://github.com/qiaopengjun5162/moonpub/issues/184)) ([a229d94](https://github.com/qiaopengjun5162/moonpub/commit/a229d94fd46738d97fbe7c72f059b2d9ddeb3138))
* **preflight:** 提取正文标题时跳过代码围栏，不再把 shell 注释当 H1 ([2876c89](https://github.com/qiaopengjun5162/moonpub/commit/2876c895bde2569f818f423cfd66626969f1409a))
* **protocol:** error/v1 的 next_step 不再泄漏散文 ([#165](https://github.com/qiaopengjun5162/moonpub/issues/165)) ([44ccda1](https://github.com/qiaopengjun5162/moonpub/commit/44ccda19c262eb3cc88cd973dd2ffc1aa1fe2e21))
* **push:** check for draft.json before reading credentials ([e5ef3a4](https://github.com/qiaopengjun5162/moonpub/commit/e5ef3a4a99b7f8d5a44539c8824a341874d09c1f))
* **push:** render the draft before reading credentials ([191d2f7](https://github.com/qiaopengjun5162/moonpub/commit/191d2f798e07226954e6b1f56d83f30ef19fc927))
* **push:** 推送前拦下过期产物，旧 white-space 或源比产物新都拒绝 ([ac799e6](https://github.com/qiaopengjun5162/moonpub/commit/ac799e68ed2de0c60350f126473417bd7aafd22a))
* **radar:** radar import 非法互动数值不再静默变空，失败时整批零写入 ([#181](https://github.com/qiaopengjun5162/moonpub/issues/181)) ([e8c1e6d](https://github.com/qiaopengjun5162/moonpub/commit/e8c1e6d89c5778fa9da698c4ad75efafe79cc437))
* **radar:** 修复抓取畸形 HTML 时 panic（字节窗口未对齐字符边界） ([#172](https://github.com/qiaopengjun5162/moonpub/issues/172)) ([df19581](https://github.com/qiaopengjun5162/moonpub/commit/df1958184c55a4e4709a41556e1b1df8673a27e6))
* **render:** href/src 里的危险协议换成 #，本地预览不再执行 javascript: ([#221](https://github.com/qiaopengjun5162/moonpub/issues/221)) ([3e367c6](https://github.com/qiaopengjun5162/moonpub/commit/3e367c63d29dab152a4929c69640347c2dd9f659))
* **render:** setext 标题不再印成空 &lt;mark&gt; 和半个 == ([#210](https://github.com/qiaopengjun5162/moonpub/issues/210)) ([b673f51](https://github.com/qiaopengjun5162/moonpub/commit/b673f514553e5bde94628e82bb45967283d40a54))
* **render:** 一段话不再被印成三段 ([#213](https://github.com/qiaopengjun5162/moonpub/issues/213)) ([5c9b014](https://github.com/qiaopengjun5162/moonpub/commit/5c9b014a0cf5896a7d9c432019c88b7217ab404e))
* **render:** 三级强调不再漏裸星号，五六级标题不再印出井号 ([#196](https://github.com/qiaopengjun5162/moonpub/issues/196)) ([8f1e463](https://github.com/qiaopengjun5162/moonpub/commit/8f1e463ad08b08cbc3622df9793e6869a131967f))
* **render:** 代码块配色跟随主题明暗，浅色主题不再出现黑色终端块 ([ab30fad](https://github.com/qiaopengjun5162/moonpub/commit/ab30fad540e65f672ae51381b85430ed699aa5c2))
* **render:** 列表项的缩进续行不再掉成顶格段落 ([#212](https://github.com/qiaopengjun5162/moonpub/issues/212)) ([cd6309b](https://github.com/qiaopengjun5162/moonpub/commit/cd6309b4c646fa4da9596bb2b7392d8746bbbe2b))
* **render:** 嵌套列表保住层级，引用里的列表不再漏 - 号 ([#194](https://github.com/qiaopengjun5162/moonpub/issues/194)) ([ccd78ff](https://github.com/qiaopengjun5162/moonpub/commit/ccd78ffe93bcdd14df742ab1c37eddd8bb99efa9))
* **render:** 引用块里的块级结构交回同一个块解析器，不再漏标记 ([#200](https://github.com/qiaopengjun5162/moonpub/issues/200)) ([7972114](https://github.com/qiaopengjun5162/moonpub/commit/79721144a2e649548cd79f7e100065ad4292ba36))
* **render:** 引用式链接不再把定义行印成正文，徽章按方括号嵌套配对 ([#202](https://github.com/qiaopengjun5162/moonpub/issues/202)) ([391cbf1](https://github.com/qiaopengjun5162/moonpub/commit/391cbf1ccd64894e678c52370632d8e634d87ccc))
* **render:** 微信摘要按字符截断，preflight 的 pass 不再与 draft.json 分叉 ([#183](https://github.com/qiaopengjun5162/moonpub/issues/183)) ([06e979f](https://github.com/qiaopengjun5162/moonpub/commit/06e979f45ad23e780c6c21010744bd7aa22dc9a0))
* **render:** 括号链接不再截成死链，段中图片不再嵌套 &lt;p&gt; ([929702a](https://github.com/qiaopengjun5162/moonpub/commit/929702a8eb28c44cfe3f5ab4442dc347bb6377cf))
* **render:** 括号链接不再截成死链，段中图片不再嵌套 &lt;p&gt; ([#193](https://github.com/qiaopengjun5162/moonpub/issues/193)) ([929702a](https://github.com/qiaopengjun5162/moonpub/commit/929702a8eb28c44cfe3f5ab4442dc347bb6377cf))
* **render:** 排版修复——卡片长串换行 + 表格/列表行式重写 ([39b276d](https://github.com/qiaopengjun5162/moonpub/commit/39b276dfec3777dd1b20147b6b5d581277091fec))
* **render:** 无边框 Markdown 表格不再整块漏成正文 ([7addbcc](https://github.com/qiaopengjun5162/moonpub/commit/7addbcc93fb42c23ac9a59cd9dd8bff6819d3832))
* **render:** 无边框 Markdown 表格不再整块漏成正文 ([#192](https://github.com/qiaopengjun5162/moonpub/issues/192)) ([7addbcc](https://github.com/qiaopengjun5162/moonpub/commit/7addbcc93fb42c23ac9a59cd9dd8bff6819d3832))
* **render:** 正文里的 `<div>` 不再变成真标签，用户文本一律转义 ([#219](https://github.com/qiaopengjun5162/moonpub/issues/219)) ([88ea7a5](https://github.com/qiaopengjun5162/moonpub/commit/88ea7a5095d644d837e1ef010a1c4afb5b48c476))
* **render:** 波浪线围栏不再把代码煮成正文 ([#211](https://github.com/qiaopengjun5162/moonpub/issues/211)) ([5f91437](https://github.com/qiaopengjun5162/moonpub/commit/5f91437b6632ac53aed65596e2aba0ace0a25b94))
* **render:** 空主题名回落到默认主题，别再让渲染整条失败 ([#230](https://github.com/qiaopengjun5162/moonpub/issues/230)) ([868359c](https://github.com/qiaopengjun5162/moonpub/commit/868359cb1d08650ec0f69b9f6cb2cbc280b3a02e))
* **render:** 行内代码改用 span + 内联样式，不再输出 code 标签 ([67722ff](https://github.com/qiaopengjun5162/moonpub/commit/67722ffc3605ff7c4d97af0a4d0bd458a6bfb48e))
* **render:** 行内元素之间的空格落成 &nbsp;，成品里文字不再粘在一起 ([#220](https://github.com/qiaopengjun5162/moonpub/issues/220)) ([bdbb2e0](https://github.com/qiaopengjun5162/moonpub/commit/bdbb2e079493efe4c37c1c312a8edffdc4a5cbf7))
* **render:** 链接标题不再塞进 href，&lt;url&gt; 自动链接不再拆坏 ([#195](https://github.com/qiaopengjun5162/moonpub/issues/195)) ([9fd9587](https://github.com/qiaopengjun5162/moonpub/commit/9fd9587dadc509c1a9f1b748a055e5e57410bb1e))
* **security:** 转义按出口分口径，TOML / CSS / HTTP 头不再可注入 ([#225](https://github.com/qiaopengjun5162/moonpub/issues/225)) ([e1336e0](https://github.com/qiaopengjun5162/moonpub/commit/e1336e050fb031609abfb6baf3e29a98aefe6a14))
* **status:** 已推送的文章不再被当成「只渲染过」 ([#214](https://github.com/qiaopengjun5162/moonpub/issues/214)) ([22dc76b](https://github.com/qiaopengjun5162/moonpub/commit/22dc76b5eeef00a3323535bd36728fb48c0fce7c))
* **theme:** 拼错的主题名不再静默降级，check 不再谎报 publishable ([#180](https://github.com/qiaopengjun5162/moonpub/issues/180)) ([14e70ad](https://github.com/qiaopengjun5162/moonpub/commit/14e70ad32a510f8a8ae943ac16c7ad928178c9e6))

## [0.4.20](https://github.com/qiaopengjun5162/moonpub/compare/v0.4.19...v0.4.20) (2026-11-19)


### Bug Fixes

* **push:** 推送前拦下过期产物，旧 white-space 或源比产物新都拒绝 ([ac799e6](https://github.com/qiaopengjun5162/moonpub/commit/ac799e68ed2de0c60350f126473417bd7aafd22a))

## [0.4.19](https://github.com/qiaopengjun5162/moonpub/compare/v0.4.18...v0.4.19) (2026-11-19)


### Bug Fixes

* **codeblock:** 空白不再单独成节点，词间空格不会被后台删掉 ([#231](https://github.com/qiaopengjun5162/moonpub/issues/231)) ([c2da0d2](https://github.com/qiaopengjun5162/moonpub/commit/c2da0d200a34f38ca560e543d2f8928cc32e20ef))
* **layout:** 代码行显式归零 white-space，预检拦下 pre 系列旧产物 ([#228](https://github.com/qiaopengjun5162/moonpub/issues/228)) ([40421c5](https://github.com/qiaopengjun5162/moonpub/commit/40421c5f19bc0cf0f93a66a6362fb63832ff029f))
* **render:** 空主题名回落到默认主题，别再让渲染整条失败 ([#230](https://github.com/qiaopengjun5162/moonpub/issues/230)) ([868359c](https://github.com/qiaopengjun5162/moonpub/commit/868359cb1d08650ec0f69b9f6cb2cbc280b3a02e))

## [0.4.18](https://github.com/qiaopengjun5162/moonpub/compare/v0.4.17...v0.4.18) (2026-11-19)


### Bug Fixes

* **security:** 转义按出口分口径，TOML / CSS / HTTP 头不再可注入 ([#225](https://github.com/qiaopengjun5162/moonpub/issues/225)) ([e1336e0](https://github.com/qiaopengjun5162/moonpub/commit/e1336e050fb031609abfb6baf3e29a98aefe6a14))

## [0.4.17](https://github.com/qiaopengjun5162/moonpub/compare/v0.4.16...v0.4.17) (2026-11-19)


### Bug Fixes

* **footer:** 配置文案与二维码地址不再注入 HTML ([#223](https://github.com/qiaopengjun5162/moonpub/issues/223)) ([60b30aa](https://github.com/qiaopengjun5162/moonpub/commit/60b30aa4f97475e6b402fc0a7f36ee28c347eff2))

## [0.4.16](https://github.com/qiaopengjun5162/moonpub/compare/v0.4.15...v0.4.16) (2026-11-19)


### Bug Fixes

* **cli:** cookie 模式没覆盖的命令不再把人送回起点 ([#207](https://github.com/qiaopengjun5162/moonpub/issues/207)) ([c021a04](https://github.com/qiaopengjun5162/moonpub/commit/c021a04dabc1a792505f90fb651a4b1638366f17))
* **cli:** delete-draft 不再退进一条跑不起来的通道 ([#208](https://github.com/qiaopengjun5162/moonpub/issues/208)) ([2ac4e54](https://github.com/qiaopengjun5162/moonpub/commit/2ac4e54e90ae81aebdf5d30b43ece376822708aa))
* **cli:** status 打印的路径能直接当下一条命令用，README 首屏一句话说清 ([#203](https://github.com/qiaopengjun5162/moonpub/issues/203)) ([deaeba3](https://github.com/qiaopengjun5162/moonpub/commit/deaeba38d995d7e71a4167cfca59acfd240342df))
* **cli:** 同一个文件不再有两种写法，工具印出的路径都能粘回自己的命令 ([#215](https://github.com/qiaopengjun5162/moonpub/issues/215)) ([2ea283f](https://github.com/qiaopengjun5162/moonpub/commit/2ea283fd9bf4b4b54e5c9d100906521427b2a1b0))
* **cli:** 推荐的下一步必须真能推进，check/new/render 都以 next 收尾 ([#204](https://github.com/qiaopengjun5162/moonpub/issues/204)) ([f138b53](https://github.com/qiaopengjun5162/moonpub/commit/f138b53dc13ab2d3cdefd99577ed0164d65f61b8))
* **cli:** 缺微信凭据不再是一句死胡同，doctor 终于看凭据 ([#206](https://github.com/qiaopengjun5162/moonpub/issues/206)) ([#206](https://github.com/qiaopengjun5162/moonpub/issues/206)) ([e18450d](https://github.com/qiaopengjun5162/moonpub/commit/e18450d4e658710fb42a3f6d448cfb25d3606176))
* **cli:** 裸跑 moonpub 的第一行就是该跑的命令 ([#205](https://github.com/qiaopengjun5162/moonpub/issues/205)) ([69c65da](https://github.com/qiaopengjun5162/moonpub/commit/69c65da3f5297b9ffeda4ab24500061a92446d21))
* **cli:** 非数字 media_id 是参数错误，不是通道错误 ([#209](https://github.com/qiaopengjun5162/moonpub/issues/209)) ([6881b26](https://github.com/qiaopengjun5162/moonpub/commit/6881b2693e8d056e43e02167900c50c185cdaccf))
* **codeblock:** 代码块不再依赖 white-space，公众号后台里不再一个 token 一行 ([#217](https://github.com/qiaopengjun5162/moonpub/issues/217)) ([ac54d9f](https://github.com/qiaopengjun5162/moonpub/commit/ac54d9ff0f1cd3132149906099c90cf11d3fa148))
* **codeblock:** 代码块里的空格改成 &nbsp;，公众号成品不再连成一片 ([#218](https://github.com/qiaopengjun5162/moonpub/issues/218)) ([d407a09](https://github.com/qiaopengjun5162/moonpub/commit/d407a0970ef88ed8a512cac39948ae6daffaee8a))
* **markdown:** ::: 块的正文也走块解析器，卡片里的代码块不再是行内 span ([#216](https://github.com/qiaopengjun5162/moonpub/issues/216)) ([e1f4c3c](https://github.com/qiaopengjun5162/moonpub/commit/e1f4c3cfbbceaaf24eb8e9978de5719fff83c940))
* **render:** href/src 里的危险协议换成 #，本地预览不再执行 javascript: ([#221](https://github.com/qiaopengjun5162/moonpub/issues/221)) ([3e367c6](https://github.com/qiaopengjun5162/moonpub/commit/3e367c63d29dab152a4929c69640347c2dd9f659))
* **render:** setext 标题不再印成空 &lt;mark&gt; 和半个 == ([#210](https://github.com/qiaopengjun5162/moonpub/issues/210)) ([b673f51](https://github.com/qiaopengjun5162/moonpub/commit/b673f514553e5bde94628e82bb45967283d40a54))
* **render:** 一段话不再被印成三段 ([#213](https://github.com/qiaopengjun5162/moonpub/issues/213)) ([5c9b014](https://github.com/qiaopengjun5162/moonpub/commit/5c9b014a0cf5896a7d9c432019c88b7217ab404e))
* **render:** 列表项的缩进续行不再掉成顶格段落 ([#212](https://github.com/qiaopengjun5162/moonpub/issues/212)) ([cd6309b](https://github.com/qiaopengjun5162/moonpub/commit/cd6309b4c646fa4da9596bb2b7392d8746bbbe2b))
* **render:** 引用块里的块级结构交回同一个块解析器，不再漏标记 ([#200](https://github.com/qiaopengjun5162/moonpub/issues/200)) ([7972114](https://github.com/qiaopengjun5162/moonpub/commit/79721144a2e649548cd79f7e100065ad4292ba36))
* **render:** 引用式链接不再把定义行印成正文，徽章按方括号嵌套配对 ([#202](https://github.com/qiaopengjun5162/moonpub/issues/202)) ([391cbf1](https://github.com/qiaopengjun5162/moonpub/commit/391cbf1ccd64894e678c52370632d8e634d87ccc))
* **render:** 正文里的 `<div>` 不再变成真标签，用户文本一律转义 ([#219](https://github.com/qiaopengjun5162/moonpub/issues/219)) ([88ea7a5](https://github.com/qiaopengjun5162/moonpub/commit/88ea7a5095d644d837e1ef010a1c4afb5b48c476))
* **render:** 波浪线围栏不再把代码煮成正文 ([#211](https://github.com/qiaopengjun5162/moonpub/issues/211)) ([5f91437](https://github.com/qiaopengjun5162/moonpub/commit/5f91437b6632ac53aed65596e2aba0ace0a25b94))
* **render:** 行内元素之间的空格落成 &nbsp;，成品里文字不再粘在一起 ([#220](https://github.com/qiaopengjun5162/moonpub/issues/220)) ([bdbb2e0](https://github.com/qiaopengjun5162/moonpub/commit/bdbb2e079493efe4c37c1c312a8edffdc4a5cbf7))
* **status:** 已推送的文章不再被当成「只渲染过」 ([#214](https://github.com/qiaopengjun5162/moonpub/issues/214)) ([22dc76b](https://github.com/qiaopengjun5162/moonpub/commit/22dc76b5eeef00a3323535bd36728fb48c0fce7c))

## [0.4.15](https://github.com/qiaopengjun5162/moonpub/compare/v0.4.14...v0.4.15) (2026-11-18)


### Features

* **cli:** 帮助系统改为命令注册表驱动 ([#150](https://github.com/qiaopengjun5162/moonpub/issues/150)) ([3d73291](https://github.com/qiaopengjun5162/moonpub/commit/3d73291d3aba6a9b57fcc25f8a2e73358cd0f906))
* **cover:** fit cover text to the canvas and add series tag ([3c6cba2](https://github.com/qiaopengjun5162/moonpub/commit/3c6cba21eaad916af6810088353fe36fdd075a29))
* **cover:** read cover fields from frontmatter and size titles by length ([1a456b7](https://github.com/qiaopengjun5162/moonpub/commit/1a456b7bde1d0908bfefd9a4d03d55747712de57))
* **cover:** 封面风格可发现，并把写错的 cover_style 拦在发布前 ([#157](https://github.com/qiaopengjun5162/moonpub/issues/157)) ([e93e6ea](https://github.com/qiaopengjun5162/moonpub/commit/e93e6ea1d9f01c6aea64ec3dd86657b8449d6d98))
* **cover:** 新增 frontmatter cover_style，cover 与 ship 共用风格解析链 ([#154](https://github.com/qiaopengjun5162/moonpub/issues/154)) ([7c6378c](https://github.com/qiaopengjun5162/moonpub/commit/7c6378c3ef5ec54ca31cbdf2e809ab7f185fdd6c))
* **cover:** 新增 swiss-dark 风格，swiss 封面把「N 字节」提成数据锚点 ([#156](https://github.com/qiaopengjun5162/moonpub/issues/156)) ([0d28ce3](https://github.com/qiaopengjun5162/moonpub/commit/0d28ce3fbc0826697c007f3a9b5d2e15584b3380))
* **help:** `--json help` 输出命令注册表，不再把整段帮助文本塞进 {"output"} ([#155](https://github.com/qiaopengjun5162/moonpub/issues/155)) ([d41053e](https://github.com/qiaopengjun5162/moonpub/commit/d41053ea5b5e3e2eb57b02691496158cf576edfc))
* **help:** 让 error/v1 错误码在运行时可枚举 ([#164](https://github.com/qiaopengjun5162/moonpub/issues/164)) ([a6b6b37](https://github.com/qiaopengjun5162/moonpub/commit/a6b6b372da189dcbd48fae4fa5d350461ac57ce9))
* **init:** add --yes flag for non-interactive setup ([d9f88c0](https://github.com/qiaopengjun5162/moonpub/commit/d9f88c08eed46376a76becb0d556e43229edf397))
* **preflight:** flag missing cover image before push ([be24d2a](https://github.com/qiaopengjun5162/moonpub/commit/be24d2af78b5fd0fa50c0ac02ac263672663dcc9))
* **preflight:** 新增 --strict，让发布前质量门能卡住退出码 ([#161](https://github.com/qiaopengjun5162/moonpub/issues/161)) ([a747940](https://github.com/qiaopengjun5162/moonpub/commit/a7479407dfc6c46374a3f58fbe96663f7367a462))
* **protocol:** --json 失败路径补 error/v1 结构化错误对象 ([#163](https://github.com/qiaopengjun5162/moonpub/issues/163)) ([2e232de](https://github.com/qiaopengjun5162/moonpub/commit/2e232de0bd543326373997372a491fb1ee585d87))
* **protocol:** 失败对象补齐 first_check 与 lesson 指针 ([a5de1ba](https://github.com/qiaopengjun5162/moonpub/commit/a5de1babf240ddfb120b94662120a723442820c7))
* **protocol:** 失败对象补齐 first_check 与 lesson 指针 ([#191](https://github.com/qiaopengjun5162/moonpub/issues/191)) ([a5de1ba](https://github.com/qiaopengjun5162/moonpub/commit/a5de1babf240ddfb120b94662120a723442820c7))
* **protocol:** 给全部命令级 JSON 载荷补 schema_version ([#167](https://github.com/qiaopengjun5162/moonpub/issues/167)) ([712cf5b](https://github.com/qiaopengjun5162/moonpub/commit/712cf5bc7fb05749703bdb39dd1b419d64ee6081))
* **protocol:** 给诊断/状态类 JSON 载荷补 schema_version ([#166](https://github.com/qiaopengjun5162/moonpub/issues/166)) ([21e567c](https://github.com/qiaopengjun5162/moonpub/commit/21e567c1b41d0162fc8608bf83ad99ead41afdc7))
* **protocol:** 错误码带诊断与教训指针，工作流带命令白名单 ([f3a559d](https://github.com/qiaopengjun5162/moonpub/commit/f3a559d80687e496f5187eb4fd5c0288269ee0e4))
* **protocol:** 错误码带诊断与教训指针，工作流带命令白名单 ([#189](https://github.com/qiaopengjun5162/moonpub/issues/189)) ([f3a559d](https://github.com/qiaopengjun5162/moonpub/commit/f3a559d80687e496f5187eb4fd5c0288269ee0e4))


### Bug Fixes

* **article:** BOM 不再吞掉 frontmatter，写反的方括号不再让整篇崩溃 ([#186](https://github.com/qiaopengjun5162/moonpub/issues/186)) ([6f7217b](https://github.com/qiaopengjun5162/moonpub/commit/6f7217bd1f4bd755397bace4907a74e9b66507c2))
* **build:** 声明真实 MSRV 为 1.88，并在 CI 验证 ([#151](https://github.com/qiaopengjun5162/moonpub/issues/151)) ([a46a921](https://github.com/qiaopengjun5162/moonpub/commit/a46a921edab16f321fe1ba4fcd883d594d1445e6))
* **cli:** configure 拼错步骤名不再静默跳过 ([#179](https://github.com/qiaopengjun5162/moonpub/issues/179)) ([c081277](https://github.com/qiaopengjun5162/moonpub/commit/c0812777d52c909c744cc5c6833bb6874235eab7))
* **cli:** init 拒绝第二个位置参数，子命令 --help 扫描所有位置 ([#178](https://github.com/qiaopengjun5162/moonpub/issues/178)) ([147a147](https://github.com/qiaopengjun5162/moonpub/commit/147a147dc1413f14435e3d0087fbc164dd69a585))
* **cli:** intake 的 flag 与位置参数顺序无关，带值 selector 不再吞开关 ([#175](https://github.com/qiaopengjun5162/moonpub/issues/175)) ([8cd7a84](https://github.com/qiaopengjun5162/moonpub/commit/8cd7a84d06f9a2db2b408d1f67d1be6a1a904a2c))
* **cli:** radar 六个子命令统一收口，带值 flag 不再吞开关 ([#177](https://github.com/qiaopengjun5162/moonpub/issues/177)) ([1e44aa8](https://github.com/qiaopengjun5162/moonpub/commit/1e44aa8082e6ccc1a2ff9490aa000bf445a39582))
* **cli:** 位置参数与布尔 flag 顺序无关 ([#168](https://github.com/qiaopengjun5162/moonpub/issues/168)) ([2b76a57](https://github.com/qiaopengjun5162/moonpub/commit/2b76a5745cf402c6c751e9c0daeffe2f95f89bf2))
* **cli:** 后置 --json 在 help / version 上不再静默失效 ([#162](https://github.com/qiaopengjun5162/moonpub/issues/162)) ([d652527](https://github.com/qiaopengjun5162/moonpub/commit/d6525277a6ffb692e35b49e7b4cb931a302fb5d1))
* **cli:** 固定参数个数命令不再静默吞掉多余参数 ([#173](https://github.com/qiaopengjun5162/moonpub/issues/173)) ([7f0b903](https://github.com/qiaopengjun5162/moonpub/commit/7f0b90380b1f0ebc034886e1818cb503cf0e0a2e))
* **cli:** 带值 flag 命令支持 flag 前置且不再静默吞参数 ([#171](https://github.com/qiaopengjun5162/moonpub/issues/171)) ([dd34102](https://github.com/qiaopengjun5162/moonpub/commit/dd3410211a1ef32414a61933f3fc85529e1b6625))
* **cli:** 自由文本命令不再把标题砍成第一个词，手写循环也不再吞参数 ([#174](https://github.com/qiaopengjun5162/moonpub/issues/174)) ([2f1962d](https://github.com/qiaopengjun5162/moonpub/commit/2f1962d5291695e7b2b9cf33c9012908a3199246))
* **cli:** 配置发现不再按「第一个非 flag token」猜路径 ([#176](https://github.com/qiaopengjun5162/moonpub/issues/176)) ([a2fa930](https://github.com/qiaopengjun5162/moonpub/commit/a2fa9307053212332c4f5a52dd6ce57f58855781))
* **cli:** 零 flag 命令不再静默吞掉多余参数 ([#169](https://github.com/qiaopengjun5162/moonpub/issues/169)) ([7410fae](https://github.com/qiaopengjun5162/moonpub/commit/7410fae889f0122590dd09ad4dfb9be568aba503))
* **config:** init 生成的配置能被自己读回来，行尾注释不再污染主题名 ([#197](https://github.com/qiaopengjun5162/moonpub/issues/197)) ([9228cd0](https://github.com/qiaopengjun5162/moonpub/commit/9228cd02668dd7b1b11c7190fdcc29348c3b10af))
* **cover:** use generated cover.png when frontmatter has no cover field ([b837ac0](https://github.com/qiaopengjun5162/moonpub/commit/b837ac03e1392a04da35811565f6271b21f58b88))
* **cover:** 中文长标题生成 ai-lab 封面不再 panic（UTF-8 char boundary） ([#170](https://github.com/qiaopengjun5162/moonpub/issues/170)) ([fc9aac2](https://github.com/qiaopengjun5162/moonpub/commit/fc9aac2985be1f92cf277386243fbc8fd517d7f6))
* **cover:** 显式 cover_subtitle 不被 cover_title 逗号覆盖 ([c1608c2](https://github.com/qiaopengjun5162/moonpub/commit/c1608c217de383c415161fe1880b56b4f823e992))
* **docs:** 安装链接改用 latest，版本口径钉成契约测试 ([07ede1e](https://github.com/qiaopengjun5162/moonpub/commit/07ede1ebef50e52dc42fa58d526d28f4347cb5cf))
* **docs:** 安装链接改用 latest，版本口径钉成契约测试 ([#190](https://github.com/qiaopengjun5162/moonpub/issues/190)) ([07ede1e](https://github.com/qiaopengjun5162/moonpub/commit/07ede1ebef50e52dc42fa58d526d28f4347cb5cf))
* **doctor:** 警告逐条成行，并把 first-run 的 next_step 改成可执行动作 ([#153](https://github.com/qiaopengjun5162/moonpub/issues/153)) ([c4a14c9](https://github.com/qiaopengjun5162/moonpub/commit/c4a14c934e4559194934a641127063a9ee4d53b2))
* **help:** radar 帮助页与报错提示补全 suggest / scrape 子命令 ([#158](https://github.com/qiaopengjun5162/moonpub/issues/158)) ([6922240](https://github.com/qiaopengjun5162/moonpub/commit/6922240ce487e7ba101f2189e9c7e64969f5056f))
* **humanize:** 中英混排文章不再 panic，30 字节回退窗口对齐字符边界 ([#182](https://github.com/qiaopengjun5162/moonpub/issues/182)) ([6accdbc](https://github.com/qiaopengjun5162/moonpub/commit/6accdbc555295fc1d6f64910ca55eb7ac39bf6ba))
* **illustrate:** 代码块词间保留真实空格，避免从单词中间硬断 ([6b943f7](https://github.com/qiaopengjun5162/moonpub/commit/6b943f770fcd8b5d421b69fb776cc472f260ada0))
* **layout:** 排版审计仅匹配带引号的真实属性 ([69668ed](https://github.com/qiaopengjun5162/moonpub/commit/69668ed95314ea68c54a9d0bb9c002721628bbcd))
* **markdown:** 同步有序列表测试断言，修复 main 上失败的用例 ([7eddfbb](https://github.com/qiaopengjun5162/moonpub/commit/7eddfbb9fb9c37d9e5da57c80a7273925b8784e9))
* **markdown:** 孤立 | 行不再让整篇文章渲染崩溃 ([#185](https://github.com/qiaopengjun5162/moonpub/issues/185)) ([f389f49](https://github.com/qiaopengjun5162/moonpub/commit/f389f49edfcab672a3fdfe5a20421b165e75515b))
* **markdown:** 无空格井号标题与表格管道符不再泄漏到微信正文 ([#184](https://github.com/qiaopengjun5162/moonpub/issues/184)) ([a229d94](https://github.com/qiaopengjun5162/moonpub/commit/a229d94fd46738d97fbe7c72f059b2d9ddeb3138))
* **preflight:** 提取正文标题时跳过代码围栏，不再把 shell 注释当 H1 ([2876c89](https://github.com/qiaopengjun5162/moonpub/commit/2876c895bde2569f818f423cfd66626969f1409a))
* **protocol:** error/v1 的 next_step 不再泄漏散文 ([#165](https://github.com/qiaopengjun5162/moonpub/issues/165)) ([44ccda1](https://github.com/qiaopengjun5162/moonpub/commit/44ccda19c262eb3cc88cd973dd2ffc1aa1fe2e21))
* **push:** check for draft.json before reading credentials ([e5ef3a4](https://github.com/qiaopengjun5162/moonpub/commit/e5ef3a4a99b7f8d5a44539c8824a341874d09c1f))
* **push:** render the draft before reading credentials ([191d2f7](https://github.com/qiaopengjun5162/moonpub/commit/191d2f798e07226954e6b1f56d83f30ef19fc927))
* **radar:** radar import 非法互动数值不再静默变空，失败时整批零写入 ([#181](https://github.com/qiaopengjun5162/moonpub/issues/181)) ([e8c1e6d](https://github.com/qiaopengjun5162/moonpub/commit/e8c1e6d89c5778fa9da698c4ad75efafe79cc437))
* **radar:** 修复抓取畸形 HTML 时 panic（字节窗口未对齐字符边界） ([#172](https://github.com/qiaopengjun5162/moonpub/issues/172)) ([df19581](https://github.com/qiaopengjun5162/moonpub/commit/df1958184c55a4e4709a41556e1b1df8673a27e6))
* **render:** 三级强调不再漏裸星号，五六级标题不再印出井号 ([#196](https://github.com/qiaopengjun5162/moonpub/issues/196)) ([8f1e463](https://github.com/qiaopengjun5162/moonpub/commit/8f1e463ad08b08cbc3622df9793e6869a131967f))
* **render:** 代码块配色跟随主题明暗，浅色主题不再出现黑色终端块 ([ab30fad](https://github.com/qiaopengjun5162/moonpub/commit/ab30fad540e65f672ae51381b85430ed699aa5c2))
* **render:** 嵌套列表保住层级，引用里的列表不再漏 - 号 ([#194](https://github.com/qiaopengjun5162/moonpub/issues/194)) ([ccd78ff](https://github.com/qiaopengjun5162/moonpub/commit/ccd78ffe93bcdd14df742ab1c37eddd8bb99efa9))
* **render:** 微信摘要按字符截断，preflight 的 pass 不再与 draft.json 分叉 ([#183](https://github.com/qiaopengjun5162/moonpub/issues/183)) ([06e979f](https://github.com/qiaopengjun5162/moonpub/commit/06e979f45ad23e780c6c21010744bd7aa22dc9a0))
* **render:** 括号链接不再截成死链，段中图片不再嵌套 &lt;p&gt; ([929702a](https://github.com/qiaopengjun5162/moonpub/commit/929702a8eb28c44cfe3f5ab4442dc347bb6377cf))
* **render:** 括号链接不再截成死链，段中图片不再嵌套 &lt;p&gt; ([#193](https://github.com/qiaopengjun5162/moonpub/issues/193)) ([929702a](https://github.com/qiaopengjun5162/moonpub/commit/929702a8eb28c44cfe3f5ab4442dc347bb6377cf))
* **render:** 排版修复——卡片长串换行 + 表格/列表行式重写 ([39b276d](https://github.com/qiaopengjun5162/moonpub/commit/39b276dfec3777dd1b20147b6b5d581277091fec))
* **render:** 无边框 Markdown 表格不再整块漏成正文 ([7addbcc](https://github.com/qiaopengjun5162/moonpub/commit/7addbcc93fb42c23ac9a59cd9dd8bff6819d3832))
* **render:** 无边框 Markdown 表格不再整块漏成正文 ([#192](https://github.com/qiaopengjun5162/moonpub/issues/192)) ([7addbcc](https://github.com/qiaopengjun5162/moonpub/commit/7addbcc93fb42c23ac9a59cd9dd8bff6819d3832))
* **render:** 行内代码改用 span + 内联样式，不再输出 code 标签 ([67722ff](https://github.com/qiaopengjun5162/moonpub/commit/67722ffc3605ff7c4d97af0a4d0bd458a6bfb48e))
* **render:** 链接标题不再塞进 href，&lt;url&gt; 自动链接不再拆坏 ([#195](https://github.com/qiaopengjun5162/moonpub/issues/195)) ([9fd9587](https://github.com/qiaopengjun5162/moonpub/commit/9fd9587dadc509c1a9f1b748a055e5e57410bb1e))
* **theme:** 拼错的主题名不再静默降级，check 不再谎报 publishable ([#180](https://github.com/qiaopengjun5162/moonpub/issues/180)) ([14e70ad](https://github.com/qiaopengjun5162/moonpub/commit/14e70ad32a510f8a8ae943ac16c7ad928178c9e6))

## [0.4.14](https://github.com/qiaopengjun5162/moonpub/compare/v0.4.13...v0.4.14) (2026-11-18)


### Features

* **help:** 让 error/v1 错误码在运行时可枚举 ([#164](https://github.com/qiaopengjun5162/moonpub/issues/164)) ([a6b6b37](https://github.com/qiaopengjun5162/moonpub/commit/a6b6b372da189dcbd48fae4fa5d350461ac57ce9))
* **preflight:** 新增 --strict，让发布前质量门能卡住退出码 ([#161](https://github.com/qiaopengjun5162/moonpub/issues/161)) ([a747940](https://github.com/qiaopengjun5162/moonpub/commit/a7479407dfc6c46374a3f58fbe96663f7367a462))
* **protocol:** --json 失败路径补 error/v1 结构化错误对象 ([#163](https://github.com/qiaopengjun5162/moonpub/issues/163)) ([2e232de](https://github.com/qiaopengjun5162/moonpub/commit/2e232de0bd543326373997372a491fb1ee585d87))
* **protocol:** 失败对象补齐 first_check 与 lesson 指针 ([a5de1ba](https://github.com/qiaopengjun5162/moonpub/commit/a5de1babf240ddfb120b94662120a723442820c7))
* **protocol:** 失败对象补齐 first_check 与 lesson 指针 ([#191](https://github.com/qiaopengjun5162/moonpub/issues/191)) ([a5de1ba](https://github.com/qiaopengjun5162/moonpub/commit/a5de1babf240ddfb120b94662120a723442820c7))
* **protocol:** 给全部命令级 JSON 载荷补 schema_version ([#167](https://github.com/qiaopengjun5162/moonpub/issues/167)) ([712cf5b](https://github.com/qiaopengjun5162/moonpub/commit/712cf5bc7fb05749703bdb39dd1b419d64ee6081))
* **protocol:** 给诊断/状态类 JSON 载荷补 schema_version ([#166](https://github.com/qiaopengjun5162/moonpub/issues/166)) ([21e567c](https://github.com/qiaopengjun5162/moonpub/commit/21e567c1b41d0162fc8608bf83ad99ead41afdc7))
* **protocol:** 错误码带诊断与教训指针，工作流带命令白名单 ([f3a559d](https://github.com/qiaopengjun5162/moonpub/commit/f3a559d80687e496f5187eb4fd5c0288269ee0e4))
* **protocol:** 错误码带诊断与教训指针，工作流带命令白名单 ([#189](https://github.com/qiaopengjun5162/moonpub/issues/189)) ([f3a559d](https://github.com/qiaopengjun5162/moonpub/commit/f3a559d80687e496f5187eb4fd5c0288269ee0e4))


### Bug Fixes

* **article:** BOM 不再吞掉 frontmatter，写反的方括号不再让整篇崩溃 ([#186](https://github.com/qiaopengjun5162/moonpub/issues/186)) ([6f7217b](https://github.com/qiaopengjun5162/moonpub/commit/6f7217bd1f4bd755397bace4907a74e9b66507c2))
* **cli:** configure 拼错步骤名不再静默跳过 ([#179](https://github.com/qiaopengjun5162/moonpub/issues/179)) ([c081277](https://github.com/qiaopengjun5162/moonpub/commit/c0812777d52c909c744cc5c6833bb6874235eab7))
* **cli:** init 拒绝第二个位置参数，子命令 --help 扫描所有位置 ([#178](https://github.com/qiaopengjun5162/moonpub/issues/178)) ([147a147](https://github.com/qiaopengjun5162/moonpub/commit/147a147dc1413f14435e3d0087fbc164dd69a585))
* **cli:** intake 的 flag 与位置参数顺序无关，带值 selector 不再吞开关 ([#175](https://github.com/qiaopengjun5162/moonpub/issues/175)) ([8cd7a84](https://github.com/qiaopengjun5162/moonpub/commit/8cd7a84d06f9a2db2b408d1f67d1be6a1a904a2c))
* **cli:** radar 六个子命令统一收口，带值 flag 不再吞开关 ([#177](https://github.com/qiaopengjun5162/moonpub/issues/177)) ([1e44aa8](https://github.com/qiaopengjun5162/moonpub/commit/1e44aa8082e6ccc1a2ff9490aa000bf445a39582))
* **cli:** 位置参数与布尔 flag 顺序无关 ([#168](https://github.com/qiaopengjun5162/moonpub/issues/168)) ([2b76a57](https://github.com/qiaopengjun5162/moonpub/commit/2b76a5745cf402c6c751e9c0daeffe2f95f89bf2))
* **cli:** 后置 --json 在 help / version 上不再静默失效 ([#162](https://github.com/qiaopengjun5162/moonpub/issues/162)) ([d652527](https://github.com/qiaopengjun5162/moonpub/commit/d6525277a6ffb692e35b49e7b4cb931a302fb5d1))
* **cli:** 固定参数个数命令不再静默吞掉多余参数 ([#173](https://github.com/qiaopengjun5162/moonpub/issues/173)) ([7f0b903](https://github.com/qiaopengjun5162/moonpub/commit/7f0b90380b1f0ebc034886e1818cb503cf0e0a2e))
* **cli:** 带值 flag 命令支持 flag 前置且不再静默吞参数 ([#171](https://github.com/qiaopengjun5162/moonpub/issues/171)) ([dd34102](https://github.com/qiaopengjun5162/moonpub/commit/dd3410211a1ef32414a61933f3fc85529e1b6625))
* **cli:** 自由文本命令不再把标题砍成第一个词，手写循环也不再吞参数 ([#174](https://github.com/qiaopengjun5162/moonpub/issues/174)) ([2f1962d](https://github.com/qiaopengjun5162/moonpub/commit/2f1962d5291695e7b2b9cf33c9012908a3199246))
* **cli:** 配置发现不再按「第一个非 flag token」猜路径 ([#176](https://github.com/qiaopengjun5162/moonpub/issues/176)) ([a2fa930](https://github.com/qiaopengjun5162/moonpub/commit/a2fa9307053212332c4f5a52dd6ce57f58855781))
* **cli:** 零 flag 命令不再静默吞掉多余参数 ([#169](https://github.com/qiaopengjun5162/moonpub/issues/169)) ([7410fae](https://github.com/qiaopengjun5162/moonpub/commit/7410fae889f0122590dd09ad4dfb9be568aba503))
* **cover:** 中文长标题生成 ai-lab 封面不再 panic（UTF-8 char boundary） ([#170](https://github.com/qiaopengjun5162/moonpub/issues/170)) ([fc9aac2](https://github.com/qiaopengjun5162/moonpub/commit/fc9aac2985be1f92cf277386243fbc8fd517d7f6))
* **docs:** 安装链接改用 latest，版本口径钉成契约测试 ([07ede1e](https://github.com/qiaopengjun5162/moonpub/commit/07ede1ebef50e52dc42fa58d526d28f4347cb5cf))
* **docs:** 安装链接改用 latest，版本口径钉成契约测试 ([#190](https://github.com/qiaopengjun5162/moonpub/issues/190)) ([07ede1e](https://github.com/qiaopengjun5162/moonpub/commit/07ede1ebef50e52dc42fa58d526d28f4347cb5cf))
* **help:** radar 帮助页与报错提示补全 suggest / scrape 子命令 ([#158](https://github.com/qiaopengjun5162/moonpub/issues/158)) ([6922240](https://github.com/qiaopengjun5162/moonpub/commit/6922240ce487e7ba101f2189e9c7e64969f5056f))
* **humanize:** 中英混排文章不再 panic，30 字节回退窗口对齐字符边界 ([#182](https://github.com/qiaopengjun5162/moonpub/issues/182)) ([6accdbc](https://github.com/qiaopengjun5162/moonpub/commit/6accdbc555295fc1d6f64910ca55eb7ac39bf6ba))
* **markdown:** 孤立 | 行不再让整篇文章渲染崩溃 ([#185](https://github.com/qiaopengjun5162/moonpub/issues/185)) ([f389f49](https://github.com/qiaopengjun5162/moonpub/commit/f389f49edfcab672a3fdfe5a20421b165e75515b))
* **markdown:** 无空格井号标题与表格管道符不再泄漏到微信正文 ([#184](https://github.com/qiaopengjun5162/moonpub/issues/184)) ([a229d94](https://github.com/qiaopengjun5162/moonpub/commit/a229d94fd46738d97fbe7c72f059b2d9ddeb3138))
* **protocol:** error/v1 的 next_step 不再泄漏散文 ([#165](https://github.com/qiaopengjun5162/moonpub/issues/165)) ([44ccda1](https://github.com/qiaopengjun5162/moonpub/commit/44ccda19c262eb3cc88cd973dd2ffc1aa1fe2e21))
* **radar:** radar import 非法互动数值不再静默变空，失败时整批零写入 ([#181](https://github.com/qiaopengjun5162/moonpub/issues/181)) ([e8c1e6d](https://github.com/qiaopengjun5162/moonpub/commit/e8c1e6d89c5778fa9da698c4ad75efafe79cc437))
* **radar:** 修复抓取畸形 HTML 时 panic（字节窗口未对齐字符边界） ([#172](https://github.com/qiaopengjun5162/moonpub/issues/172)) ([df19581](https://github.com/qiaopengjun5162/moonpub/commit/df1958184c55a4e4709a41556e1b1df8673a27e6))
* **render:** 三级强调不再漏裸星号，五六级标题不再印出井号 ([#196](https://github.com/qiaopengjun5162/moonpub/issues/196)) ([8f1e463](https://github.com/qiaopengjun5162/moonpub/commit/8f1e463ad08b08cbc3622df9793e6869a131967f))
* **render:** 嵌套列表保住层级，引用里的列表不再漏 - 号 ([#194](https://github.com/qiaopengjun5162/moonpub/issues/194)) ([ccd78ff](https://github.com/qiaopengjun5162/moonpub/commit/ccd78ffe93bcdd14df742ab1c37eddd8bb99efa9))
* **render:** 微信摘要按字符截断，preflight 的 pass 不再与 draft.json 分叉 ([#183](https://github.com/qiaopengjun5162/moonpub/issues/183)) ([06e979f](https://github.com/qiaopengjun5162/moonpub/commit/06e979f45ad23e780c6c21010744bd7aa22dc9a0))
* **render:** 括号链接不再截成死链，段中图片不再嵌套 &lt;p&gt; ([929702a](https://github.com/qiaopengjun5162/moonpub/commit/929702a8eb28c44cfe3f5ab4442dc347bb6377cf))
* **render:** 括号链接不再截成死链，段中图片不再嵌套 &lt;p&gt; ([#193](https://github.com/qiaopengjun5162/moonpub/issues/193)) ([929702a](https://github.com/qiaopengjun5162/moonpub/commit/929702a8eb28c44cfe3f5ab4442dc347bb6377cf))
* **render:** 无边框 Markdown 表格不再整块漏成正文 ([7addbcc](https://github.com/qiaopengjun5162/moonpub/commit/7addbcc93fb42c23ac9a59cd9dd8bff6819d3832))
* **render:** 无边框 Markdown 表格不再整块漏成正文 ([#192](https://github.com/qiaopengjun5162/moonpub/issues/192)) ([7addbcc](https://github.com/qiaopengjun5162/moonpub/commit/7addbcc93fb42c23ac9a59cd9dd8bff6819d3832))
* **render:** 链接标题不再塞进 href，&lt;url&gt; 自动链接不再拆坏 ([#195](https://github.com/qiaopengjun5162/moonpub/issues/195)) ([9fd9587](https://github.com/qiaopengjun5162/moonpub/commit/9fd9587dadc509c1a9f1b748a055e5e57410bb1e))
* **theme:** 拼错的主题名不再静默降级，check 不再谎报 publishable ([#180](https://github.com/qiaopengjun5162/moonpub/issues/180)) ([14e70ad](https://github.com/qiaopengjun5162/moonpub/commit/14e70ad32a510f8a8ae943ac16c7ad928178c9e6))

## [0.4.13](https://github.com/qiaopengjun5162/moonpub/compare/v0.4.12...v0.4.13) (2026-11-06)


### Features

* **cli:** 帮助系统改为命令注册表驱动 ([#150](https://github.com/qiaopengjun5162/moonpub/issues/150)) ([3d73291](https://github.com/qiaopengjun5162/moonpub/commit/3d73291d3aba6a9b57fcc25f8a2e73358cd0f906))
* **cover:** 封面风格可发现，并把写错的 cover_style 拦在发布前 ([#157](https://github.com/qiaopengjun5162/moonpub/issues/157)) ([e93e6ea](https://github.com/qiaopengjun5162/moonpub/commit/e93e6ea1d9f01c6aea64ec3dd86657b8449d6d98))
* **cover:** 新增 frontmatter cover_style，cover 与 ship 共用风格解析链 ([#154](https://github.com/qiaopengjun5162/moonpub/issues/154)) ([7c6378c](https://github.com/qiaopengjun5162/moonpub/commit/7c6378c3ef5ec54ca31cbdf2e809ab7f185fdd6c))
* **cover:** 新增 swiss-dark 风格，swiss 封面把「N 字节」提成数据锚点 ([#156](https://github.com/qiaopengjun5162/moonpub/issues/156)) ([0d28ce3](https://github.com/qiaopengjun5162/moonpub/commit/0d28ce3fbc0826697c007f3a9b5d2e15584b3380))
* **help:** `--json help` 输出命令注册表，不再把整段帮助文本塞进 {"output"} ([#155](https://github.com/qiaopengjun5162/moonpub/issues/155)) ([d41053e](https://github.com/qiaopengjun5162/moonpub/commit/d41053ea5b5e3e2eb57b02691496158cf576edfc))


### Bug Fixes

* **build:** 声明真实 MSRV 为 1.88，并在 CI 验证 ([#151](https://github.com/qiaopengjun5162/moonpub/issues/151)) ([a46a921](https://github.com/qiaopengjun5162/moonpub/commit/a46a921edab16f321fe1ba4fcd883d594d1445e6))
* **doctor:** 警告逐条成行，并把 first-run 的 next_step 改成可执行动作 ([#153](https://github.com/qiaopengjun5162/moonpub/issues/153)) ([c4a14c9](https://github.com/qiaopengjun5162/moonpub/commit/c4a14c934e4559194934a641127063a9ee4d53b2))

## [0.4.12](https://github.com/qiaopengjun5162/moonpub/compare/v0.4.11...v0.4.12) (2026-11-06)


### Bug Fixes

* **render:** 行内代码改用 span + 内联样式，不再输出 code 标签 ([67722ff](https://github.com/qiaopengjun5162/moonpub/commit/67722ffc3605ff7c4d97af0a4d0bd458a6bfb48e))

## [0.4.11](https://github.com/qiaopengjun5162/moonpub/compare/v0.4.10...v0.4.11) (2026-11-06)


### Bug Fixes

* **markdown:** 同步有序列表测试断言，修复 main 上失败的用例 ([7eddfbb](https://github.com/qiaopengjun5162/moonpub/commit/7eddfbb9fb9c37d9e5da57c80a7273925b8784e9))
* **preflight:** 提取正文标题时跳过代码围栏，不再把 shell 注释当 H1 ([2876c89](https://github.com/qiaopengjun5162/moonpub/commit/2876c895bde2569f818f423cfd66626969f1409a))
* **render:** 代码块配色跟随主题明暗，浅色主题不再出现黑色终端块 ([ab30fad](https://github.com/qiaopengjun5162/moonpub/commit/ab30fad540e65f672ae51381b85430ed699aa5c2))
* **render:** 排版修复——卡片长串换行 + 表格/列表行式重写 ([39b276d](https://github.com/qiaopengjun5162/moonpub/commit/39b276dfec3777dd1b20147b6b5d581277091fec))

## [0.4.10](https://github.com/qiaopengjun5162/moonpub/compare/v0.4.9...v0.4.10) (2026-11-01)


### Bug Fixes

* **illustrate:** 代码块词间保留真实空格，避免从单词中间硬断 ([6b943f7](https://github.com/qiaopengjun5162/moonpub/commit/6b943f770fcd8b5d421b69fb776cc472f260ada0))

## [0.4.9](https://github.com/qiaopengjun5162/moonpub/compare/v0.4.8...v0.4.9) (2026-10-31)


### Features

* **cover:** fit cover text to the canvas and add series tag ([3c6cba2](https://github.com/qiaopengjun5162/moonpub/commit/3c6cba21eaad916af6810088353fe36fdd075a29))
* **cover:** read cover fields from frontmatter and size titles by length ([1a456b7](https://github.com/qiaopengjun5162/moonpub/commit/1a456b7bde1d0908bfefd9a4d03d55747712de57))
* **init:** add --yes flag for non-interactive setup ([d9f88c0](https://github.com/qiaopengjun5162/moonpub/commit/d9f88c08eed46376a76becb0d556e43229edf397))
* **preflight:** flag missing cover image before push ([be24d2a](https://github.com/qiaopengjun5162/moonpub/commit/be24d2af78b5fd0fa50c0ac02ac263672663dcc9))


### Bug Fixes

* **cover:** use generated cover.png when frontmatter has no cover field ([b837ac0](https://github.com/qiaopengjun5162/moonpub/commit/b837ac03e1392a04da35811565f6271b21f58b88))
* **cover:** 显式 cover_subtitle 不被 cover_title 逗号覆盖 ([c1608c2](https://github.com/qiaopengjun5162/moonpub/commit/c1608c217de383c415161fe1880b56b4f823e992))
* **layout:** 排版审计仅匹配带引号的真实属性 ([69668ed](https://github.com/qiaopengjun5162/moonpub/commit/69668ed95314ea68c54a9d0bb9c002721628bbcd))

## [0.4.8](https://github.com/qiaopengjun5162/moonpub/compare/v0.4.7...v0.4.8) (2026-10-29)


### Bug Fixes

* **push:** check for draft.json before reading credentials ([e5ef3a4](https://github.com/qiaopengjun5162/moonpub/commit/e5ef3a4a99b7f8d5a44539c8824a341874d09c1f))
* **push:** render the draft before reading credentials ([191d2f7](https://github.com/qiaopengjun5162/moonpub/commit/191d2f798e07226954e6b1f56d83f30ef19fc927))

## [0.4.7](https://github.com/qiaopengjun5162/moonpub/compare/v0.4.6...v0.4.7) (2026-10-29)


### Refactoring

* **cli:** split Options::parse into staged helpers ([3043433](https://github.com/qiaopengjun5162/moonpub/commit/30434333af9d50aea7568cf7154031be1e1f9174))
* **html:** share one HTML escaper across renderers ([ff64d47](https://github.com/qiaopengjun5162/moonpub/commit/ff64d476a3740b4d3820383a61b479db07e396b7))

## [0.4.6](https://github.com/qiaopengjun5162/moonpub/compare/v0.4.5...v0.4.6) (2026-10-29)


### CI / Deployment

* **release:** hide docs commits from version bumps ([7abde7b](https://github.com/qiaopengjun5162/moonpub/commit/7abde7bfb752b454d45eb04c7a62aa96d872b496))

## [0.4.5](https://github.com/qiaopengjun5162/moonpub/compare/v0.4.4...v0.4.5) (2026-10-28)


### Features

* add cards + wechat-checklist commands and cover/theme metadata ([dff8525](https://github.com/qiaopengjun5162/moonpub/commit/dff8525ae40829a10e1e31ec7b45bdb93390d10d))
* **cover:** AI 封面生成能力（cover --style ai-art/cartoon/anime） ([55db2e0](https://github.com/qiaopengjun5162/moonpub/commit/55db2e0459eb1a5919b0d37a78bb130739bf1109))
* **cover:** 新增 5 套高级海报封面风格（swiss/aurora/riso/noir/bauhaus） ([317a1ef](https://github.com/qiaopengjun5162/moonpub/commit/317a1ef04fa8c92b096d35e96a5fe3a2611d83d5))
* **cover:** 新增 Editorial 内容驱动少文字封面风格 ([329b9cb](https://github.com/qiaopengjun5162/moonpub/commit/329b9cb0380873a4492e17dae7d6cbbf842d595e))
* **mcp:** add MoonPub MCP server and companion publish skill ([2e7c2b0](https://github.com/qiaopengjun5162/moonpub/commit/2e7c2b0912dfefcbd20402a7cd71f6adb58bb61d))
* **ship:** 在 ship 命令集成 AI 封面（--style ai-art） ([2d9e74f](https://github.com/qiaopengjun5162/moonpub/commit/2d9e74fd6e8d02154f272e4acb4274c6b7bf26f0))
* **ship:** 封面风格校验防静默fallback + 输出带style ([4c9d8dc](https://github.com/qiaopengjun5162/moonpub/commit/4c9d8dc03a42d696c11b2943ecbe1b8c6b198e33))
* **theme:** 新增 5 套编辑器配色主题（monokai/dracula/nord/one-dark/gruvbox） ([e9bf1f8](https://github.com/qiaopengjun5162/moonpub/commit/e9bf1f8907ab8dc67ba3ee6616af7578ce9d44ee))


### Bug Fixes

* **cover:** AI 封面 provider 守卫前置，避免误导性报错 ([9c3790a](https://github.com/qiaopengjun5162/moonpub/commit/9c3790a58d61474995493fea07764212f82be8df))
* **cover:** replace internal build signature in geek-black cover with neutral text ([58be536](https://github.com/qiaopengjun5162/moonpub/commit/58be5361d515835dcd99f3220d673e60220cb502))
* **cover:** replace internal build signature in geek-black cover with neutral text ([cd83b7f](https://github.com/qiaopengjun5162/moonpub/commit/cd83b7fcfb3ff30fba35611bae4e5fed98f46484))
* **footer:** inline local follow_image as data URI like qrcode ([5bfaa1d](https://github.com/qiaopengjun5162/moonpub/commit/5bfaa1d8566a47bbbab2103029fc85dbfeb50db5))
* **release:** derive release gate doc path from CARGO_PKG_VERSION ([9b11a91](https://github.com/qiaopengjun5162/moonpub/commit/9b11a91fc3852b52b2e6fcd28e2207861094762c))
* **render:** blockquote text uses text_color for contrast on tinted backgrounds ([1d2d800](https://github.com/qiaopengjun5162/moonpub/commit/1d2d80026b8565d2930826744be4adced4f0b493))
* **render:** render list markers inline instead of 34px leading column (empty column on WeChat mobile) ([b758b99](https://github.com/qiaopengjun5162/moonpub/commit/b758b99431fd37a5ad67cda00dd719743e9add11))
* **render:** 封面HTML不再嵌入微信正文, 修复构建tag泄漏(WEB3·DEV) D17-D19已发布文章实测 ([3d67d91](https://github.com/qiaopengjun5162/moonpub/commit/3d67d91cbe12d59ee5b6d970d285ef036e5844d0))
* **render:** 微信兼容代码块——弃用 &lt;pre&gt; 改逐行 &lt;p&gt;，Xcode Dark 主题，去语言标签 ([f415809](https://github.com/qiaopengjun5162/moonpub/commit/f4158095145729a5f94c51244b8a4066781d785d))


### Refactoring

* **protocol:** 将单文件 protocol.rs 抽取为 protocol/ 模块 ([c4561ef](https://github.com/qiaopengjun5162/moonpub/commit/c4561ef49220d1249a9d7a6d5d5fb9e21491e769))


### Documentation

* **lessons:** footer follow_image data URI 内联修复记录 ([9dc8d17](https://github.com/qiaopengjun5162/moonpub/commit/9dc8d1758b88d01116ab5e78688216a9c2a6d3df))
* **readme:** 同步封面风格文档与代码默认风格 ([c59f310](https://github.com/qiaopengjun5162/moonpub/commit/c59f3103eb42f3da72307bb66e115432c582bcef))
* **release:** correct v0.4.4 publication facts in gate doc and ROADMAP ([b834ca4](https://github.com/qiaopengjun5162/moonpub/commit/b834ca405659eac355428895b177907bf2ca85f5))


### Maintenance

* **deps:** bump rustls ecosystem to patch security releases ([c2caae0](https://github.com/qiaopengjun5162/moonpub/commit/c2caae0917b968f7db97eb956c8f3c3db412f1d7))
* 忽略生成物目录与覆盖率产物 ([c42f595](https://github.com/qiaopengjun5162/moonpub/commit/c42f59521f46cf27f3a2b86e982e15df569a1931))
* 收口并行会话在途改动并闭合测试套件 ([a022fdb](https://github.com/qiaopengjun5162/moonpub/commit/a022fdb1205e0ff4f3058d9365c6dfc0bbd64419))
* 补充仓库基础工程配置与文档 ([5dc9417](https://github.com/qiaopengjun5162/moonpub/commit/5dc94178bcad61352f58d34003963b4395d5182e))

## [0.4.1] — 2026-06-23

### Added
- `moonpub --version` / `moonpub -V` prints the current CLI version for install checks and support requests.
- Release workflow now builds a native macOS ARM64 asset (`moonpub-macos-arm64.tar.gz`) in addition to macOS x86_64, Linux, and Windows assets.

### Fixed
- Non-interactive `moonpub init` now writes the current directory as `[articles].root`, so the first-run local flow works from a clean directory instead of writing `/path/to/ObsidianMain`.
- The hand-written TOML parser now unescapes basic string sequences used by generated config paths.

### Verified
- v0.4.0 macOS amd64 asset download and sha256 verification passed, but first-run smoke test failed because `moonpub init` wrote the placeholder articles root. v0.4.1 is the intended first broadly shareable release candidate.

## [0.4.0] — 2026-06-17

### Added
- **AI 写作**: `write`, `expand`, `polish` 三个命令，通过 DeepSeek API 实现
- **Obsidian 插件**: Cmd+P 输入"发布"即可推送到微信
- **去 AI 味**: `humanize` 命令，6 阶段规则处理
- **自动加载 .env**: 启动时自动加载 `.env` 和 `~/.moonpub.env`
- **封面风格**: 新增 `ink`(水墨)、`sunset`(日落)、`forest`(森林) 三种风格，共 10 种
- **封面自动下载**: frontmatter 中 `cover: https://...` URL 自动下载上传为微信封面图
- **用户使用说明书**: `docs/USER_GUIDE.md`
- **新手上手指南**: `docs/GETTING_STARTED.md`
- **项目首页**: `docs/index.html`

### Changed
- **Footer 模块化**: `[footer]` TOML section 可配置结尾模板，未配置则不渲染
- **ship --ai**: 润色后发布，一步到位
- **首段摘要**: 不再自动填充 digest，由微信自行抓取

### Fixed
- 微读导入笔记的 Obsidian callout `[!abstract]` 不再渲染为超长 blockquote
- TOML 解析器按 section 区分同名 key，root 不再冲突
- 配置自动发现：不传 `--config` 时自动检测 `moonpub.toml`

### Infrastructure
- GitHub Actions release 构建 (macOS)
- CODE_OF_CONDUCT.md, CONTRIBUTING.md
- 129 个测试全过

## [0.3.2] — 2026-06-16

### Added
- **浏览器自动化全通**: 原创声明、赞赏、留言、创作来源、预览全部稳定
- **创作来源 radio value 选择器**: 精确到 `input[value="4"]`
- **预览发送**: headless 下成功发送预览到手机
- **文章状态追踪**: `status` / `check` 命令
- **阅读量数据采集**: `radar` 命令组

### Fixed
- headless 下 target="_blank" 新 tab 检测不可靠，改用 `page.goto(url)` 直接导航
- 创作来源弹窗文本匹配不稳定，改用 DOM 结构标记
- 赞赏 toggle offsetParent 不可见，改用 JS `.click()` 绕过
- `<blockquote>` 样式被微信剥离，改用 `<section>` 标签

## [0.3.0] — 2026-06-15

### Changed
- monolithic `lib.rs` 按职责拆分为 13 个模块
- `wechat.rs` 与 `app.rs` 循环依赖修复
- geek 主题从纯黑改为 GitHub light 配色
- `ship` 每次截图封面 PNG → 上传微信 → 新 media_id

## [0.2.0] — 2026-06-14

### Added
- `ship` 一键发布命令
- 封面生成 (`cover`) — 6 种风格
- 浏览器自动化 (CDP) — 原创声明/赞赏/留言/预览
- WeChat API 客户端 — ureq, 零 SDK
- Zola 博客导出
- 4 种主题: default, warm, dark, geek

## [0.1.0] — 2026-06-13

### Added
- Markdown → WeChat HTML 渲染
- Block 模板系统 (intro, callout, steps, summary 等)
- 命令行解析和配置管理
- 手写 TOML 解析器
