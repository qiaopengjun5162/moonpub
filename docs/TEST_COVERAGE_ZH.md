# 测试覆盖率与验证边界（2026-10-29 首测，2026-11-23 复测）

这份文档回答一个问题：**这个项目到底有多少东西是被真正验证过的？**
数字全部来自本机实测，命令见文末，可随时复现。

## 最新复测（2026-11-23）

`cargo llvm-cov report` 一次给三列百分比，**它们不是一回事**：门禁卡的是 `Lines` 那一列，而 PR 里最常被引用的却是排在最左边的 `Regions`。下表三列分开写，避免再混读（这条教训见 `docs/ENGINEERING_LESSONS_ZH.md` 的 2026-11-23 记录）。

| 指标 | 本机 macOS | CI（Ubuntu，run `45051259852`） |
| --- | --- | --- |
| **行覆盖（Lines，门禁卡这列）** | **83.93%**（26335 行，未覆盖 4232） | **83.92%**（未覆盖 4234） |
| 区域覆盖（Regions） | 81.45% | 81.45% |
| 函数覆盖（Functions） | 82.36% | 82.36% |
| 测试数 | **1022 passed** | 1022 passed |
| 门禁下限 | 78%（`COVERAGE_FLOOR`，可用环境变量覆盖） | 同左 |

> 两列都是 `ship.rs` 重构之后的实测值，行覆盖只差 0.01 个点（`cfg(target_os)` 覆盖差异）。

行覆盖低于平均线的仍集中在浏览器编排层：`publish.rs` 12.68%（未覆盖 473 行）、`cdp.rs` 25.11%、`push_browser.rs` 25.54%、`init.rs` 26.46%。其中 `cdp.rs` / `push_browser.rs` / `publish.rs` 按 `docs/BROWSER_AUTOMATION.md` 的规则不追求单测覆盖（要真 Chrome）。两个"编排层缺接缝"的判断已连着验证两次：`publish_steps.rs` 抽出 `StepIo` 后行覆盖 **13.58% → 87.01%**；`ship.rs` 把凭据解析 / 封面落盘 / 风格校验从 200 行单函数里剪出来后行覆盖 **24.05% → 57.09%**（`ship_article` 主流程本身仍要真微信后台，剩余缺口集中在那里）。补上接缝，覆盖率就上去了，说明缺的从来不是测试意愿。

> 下面「一句话结论」与「修复前后对比」两节是 **2026-10-29 那一轮**的快照，保留作历史记录（它记录的是从 72.0% 修到 74.2% 那次改造），不要拿其中的数字当今天的实测值。

## 一句话结论（2026-10-29 快照）

- 可自动验证的部分：**74.2% 行覆盖**，`cargo nextest` 共 **593 个测试**全绿。
- 真正发布到公众号的浏览器自动化部分：**7%–27%**，这不是遗漏，而是**无法用单元测试验证**，只能人工过闸。
- 本次修复前，发布链路的核心网络层 `wechat.rs` 只有 **40.1%**，`push.rs` 只有 **43.6%** —— 测试看着不少，实际没碰真正的代码路径。

## 修复前后对比（2026-10-29 快照）

| 文件 | 作用 | 修复前 | 修复后 |
| --- | --- | --- | --- |
| `src/wechat.rs` | 微信 API 客户端（取 token / 建草稿 / 传图 / 发布） | 40.1% | **93.6%** |
| `src/push.rs` | 推送草稿管线（渲染→传图→建草稿→归档） | 43.6% | **70.9%** |
| `src/publish.rs` | 浏览器步骤编排 | 1.6% | 6.9% |
| `src/publish_steps.rs` | 编辑器内单步操作（原创/赞赏/留言…） | 12.0% | 12.0% |
| `src/push_browser.rs` | Cookie 会话推送 | 22.2% | 22.2% |
| `src/cdp.rs` | CDP 原语 | 24.1% | 24.1% |
| **合计（行覆盖）** | | **72.0%** | **74.2%** |

## 之前为什么测不到发布链路

`WechatClient` 把 `https://api.weixin.qq.com` 硬编码在每个方法里：

```rust
const DRAFT_ADD_URL: &str = "https://api.weixin.qq.com/cgi-bin/draft/add";
```

没有注入点，测试只能写"把一段 JSON 字符串解析一下"这种**看起来像测试、实际没执行被测代码**的用例。
`wechat.rs` 原有 15 个测试，**没有一个调用过 `create_draft` / `upload_image`**。

## 修法：把出口变成数据

1. base URL 从常量变成字段，`WechatClient::with_base_url(appid, secret, base)` 可注入；
   `new()` 默认读 `WECHAT_API_BASE`，所以 staging 也能指向回放代理。
2. 新增 `src/mock_wechat.rs`：基于 `std::net::TcpListener` 的本地假微信服务，零依赖、按序应答、记录请求。
3. 顺手修了一个真实缺陷：`proxy_url_for` 之前把 `127.0.0.1` 也塞进 `HTTPS_PROXY`，
   本地请求会被代理打成 502。现在 loopback 一律绕过代理。

## 现在真的验证了什么

### 客户端方法级（18 个测试，直接打 mock 服务）

- `access_token`：凭据拼接、响应解析、`errcode 40164` → 带 IP 的 `PushFailed`
- `create_draft`：POST 到 `/cgi-bin/draft/add`、body 含标题与正文、返回 `media_id`、缺字段报错
- `update_draft`：只发第一篇、`index: 0`、带上 `media_id`
- `list_drafts`：标题/总数/更新时间解析
- `delete_draft`、`free_publish`：成功与 `errcode` 失败两条路
- `upload_image`：`/material/add_material?type=image`，multipart 含 filename、mime 和字节
- `upload_image_url`：`.gif` 和 >1 MB **在发请求之前**就被拒（断言 mock 收到 0 个请求）

### 管线端到端（5 个测试）

`push_wechat_draft_with_client` 从磁盘上的文章一路走到归档：

- 渲染产物 → 取 token → 上传正文图并改写 `src` → 建草稿 → 写 `.media_id` → 移到 `Articles/ready`
- 自动发布：`verified` 账号走 `freepublish/submit`，成功后移到 `Articles/published`
- 覆盖旧草稿：先读旧 `.media_id`，建新草稿后删除旧的
- 凭据被拒（40164）：立即中止，不建草稿、不写 `media_id`、不动目录
- 自动发布失败（48001）：**只降级为提示**，草稿仍然归档到 `ready`

### 浏览器层里能纯函数化的部分（5 个测试）

`publish.rs` 的 `xpath_literal` 决定"按昵称找账号卡片"能不能选中人，
XPath 又没有转义字符，名字里带 `'` 或 `"` 必须拼成 `concat` 形式。已覆盖：

- 纯中文名 → 单引号字面量
- 只有一个单引号 → 换双引号
- 同时有两种引号 → `'a', "'", 'b"c'` 拼接
- 单独一个引号 / 空字符串

### 真实二进制冒烟（不是测试代码）

```bash
moonpub new "端到端验证"
moonpub render "Articles/drafts/端到端验证.md"
WECHAT_APPID=… WECHAT_SECRET=… WECHAT_API_BASE=http://127.0.0.1:18878 \
  moonpub push "Articles/drafts/端到端验证.md"
```

实测结果：mock 依次收到 `GET /cgi-bin/token`、`POST /cgi-bin/draft/add`，
输出 `media_id: MEDIA_E2E`，`Articles/ready/端到端验证.media_id` 落盘，`status.jsonl` 记为 `ready`。

## 无法自动验证的部分（明确边界）

`publish.rs`、`publish_steps.rs`、`cdp.rs`、`push_browser.rs` 靠 CDP 操作真实编辑器，
并且代码里就有 `wait_enter()` / `ask_ok("选中…了？")` 这种人机确认点。**它们不可能在 CI 里跑。**

上面的真实冒烟同时印证了这一点：CLI 走到浏览器那一步时真的连上了线上后台，列出了 10 篇历史草稿，
然后因为 mock 草稿在真实后台里不存在而输出 `⚠ automation: draft list edit button not found`。
这类行为只有在有登录态的机器上、由人看着才能判定对错。

人工验证入口：

```bash
moonpub step-test          # 逐步走编辑器，每步截图 + 人工确认
moonpub test-zanshang      # 赞赏
moonpub test-chuangzuo     # 创作声明
moonpub test-yulan --to <wxid>  # 预览发送
```

判定标准见 `docs/BROWSER_AUTOMATION.md` 与 `docs/RELEASE_GATE_v*.md`。
**发版前这些必须人工过一遍，覆盖率数字不能替代它。**

## 门槛策略

`scripts/coverage.sh` 设了行覆盖下限（当前 **78%**，实测行覆盖 macOS 83.93% / CI Ubuntu 83.92%，留出 macOS / Linux 平台分支的漂移余量），CI 的 `coverage` job 跑同一个脚本。

- 只升不降：覆盖率涨了就抬高 `COVERAGE_FLOOR`。
- 不许为了让红灯变绿而下调门槛；真要下调必须在 PR 里说明原因。
- 新增微信 API 端点必须同时加一条打 mock 的测试，否则这个端点等于没验证。

### 新增端点的测试配方

```rust
let mock = MockWechat::start([r#"{"errcode":0,"some_id":"X"}"#]);
let client = WechatClient::with_base_url("app", "secret", mock.base_url());

let id = client.some_new_endpoint("TOKEN", arg)?;

assert_eq!(id, "X");
let request = &mock.wait_for(1)[0];          // 断言真实发出的请求
assert_eq!(request.path(), "/cgi-bin/some/endpoint");
assert!(request.body_text().contains("expected"));
```

## 复现命令

```bash
./scripts/coverage.sh              # 跑测试 + 逐文件报告 + 卡门槛
./scripts/coverage.sh --report     # 复用上次结果，不重跑
./scripts/coverage.sh --html       # 额外产出 target/llvm-cov/html/index.html
```

**CI 实测（Ubuntu，run `45051259852`，2026-11-23）**：行覆盖 `83.92%`（26335 行，未覆盖 4234），`Coverage floor OK (lines >= 78%)`；本机 macOS 清掉旧插桩产物后行覆盖 `83.93%`（26335 行，未覆盖 4232）。两者差 0.01 个点，来自 `cfg(target_os)` 覆盖差异；门禁的 78% 因此有约 5.9 点余量。

**读数前先清产物。** 2026-11-22 本机测出过 `80.52%`，那是一次**误读**：`src/cover.rs` 被拆成 `src/cover/{mod,styles,ai,capture}.rs` 之后，旧文件在 `target/` 下的插桩产物仍在，llvm-cov 把它们和新文件一起合并进报告——同一份代码的总行数从 25616 虚增到 41791，覆盖率也随之上浮。`scripts/coverage.sh` 现在默认先跑 `cargo llvm-cov clean --workspace`；需要快速迭代时可用 `--keep-artifacts` 跳过，但那个数字只适合看趋势，不适合当结论。

本地想拿到 CI 等价结果，先清掉本机的真实凭据：

```bash
env -u WECHAT_APPID -u WECHAT_SECRET cargo nextest run
```

要求 `cargo-llvm-cov` 与 `cargo-nextest`：

```bash
cargo install cargo-llvm-cov --locked
cargo install cargo-nextest --locked
```
