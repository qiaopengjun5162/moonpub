# MoonPub 首次体验取证清单

这份清单不是功能说明，而是：

**当我们已经把首页、飞书、照片、当前文章这些首次入口做出来后，接下来应该怎样补“真实使用证据”。**

目标是避免后面再次出现这种情况：

- 代码已经实现
- 文档已经写了
- 测试也通过了
- 但真实用户证据仍然很弱

## 取证目标

本轮优先补和“首次体验”直接相关的证据；v0.4.2 release gate 还需要单独补真实微信回归证据。

需要补的核心证据有 4 类：

1. 插件首页证据
2. 飞书首次体验证据
3. 照片首次体验证据
4. 真实微信回归证据

## 总原则

- 证据优先来自真实本地运行，而不是脑补流程图
- 截图或录屏里不要暴露 `.env`、`moonpub.toml`、token、cookie、二维码等敏感信息
- 如果某条路径只是“代码和测试已到位”，不要在证据里写成“真实已完全打通”
- 首次体验证据的重点是：
  - 能不能顺利进入入口
  - 能不能生成草稿
  - 能不能看到本地预览
  - 用户能不能看懂下一步

## 证据 A：插件首页

### 目标

证明 `打开 MoonPub 首页` 已经是一个真实可用的首页入口，而不只是一个命令名。

### 建议截图

至少准备 2 张：

1. `homepage-workspace.png`
   内容：
   - `MoonPub 首页工作台`
   - 当前推荐入口
   - drafts / ready / published
   - 首页快捷动作
2. `homepage-context.png`
   内容：
   - 当前上下文
   - 当前更推荐
   - 第一次建议步骤

### 验收点

- [ ] 首页命令名清楚可见：`打开 MoonPub 首页`
- [ ] 首页弹窗标题清楚可见：`MoonPub 首页工作台`
- [ ] 至少能看见一条上下文推荐
- [ ] 至少能看见一组首次建议步骤

## 证据 B：飞书首次体验

### 目标

证明用户可以从插件首页进入飞书入口，再顺着走到草稿和结果页。

### 建议截图或短录屏

至少准备下面 3 个节点：

1. `feishu-home-entry.png`
   内容：
   - 从首页看到飞书入口
2. `feishu-result-modal.png`
   内容：
   - 飞书结果工作台
   - Inbox / Draft / HTML 预览
   - 推荐下一步
3. `feishu-draft-opened.png`
   内容：
   - 生成出来的草稿已在 Obsidian 中打开

如果录屏，更推荐录一段 20-40 秒短视频，把下面节奏录下来：

`打开首页 -> 点飞书入口 -> 等待生成 -> 看到结果工作台 -> 打开草稿`

### 验收点

- [ ] 首页能进入飞书入口
- [ ] 飞书入口执行后确实生成草稿
- [ ] 结果工作台能展示 `inbox_path` / `draft_path` / `html_path`
- [ ] 草稿能在 vault 中被打开

### 备注

飞书路线已经有比较强的真实 CLI 闭环证据，所以这轮更重点补的是：

**插件首页到结果页的用户证据。**

## 证据 C：照片首次体验

### 目标

证明用户可以从当前图片出发，把图片所在目录整理成草稿。

### 建议准备一组低风险样例

优先选择：

- 2 到 5 张同一天的生活照片
- 不包含敏感隐私
- 能直观看出它们属于同一组素材

### 建议截图或短录屏

至少准备下面 3 个节点：

1. `photos-image-opened.png`
   内容：
   - 当前在 Obsidian 中打开一张图片
2. `photos-result-modal.png`
   内容：
   - 照片结果工作台
   - Inbox / Draft / HTML 预览
3. `photos-draft-opened.png`
   内容：
   - 生成出来的照片草稿

如果录屏，更推荐录一段 20-40 秒短视频，把下面节奏录下来：

`打开图片 -> 打开首页 -> 点导入当前图片目录 -> 看到结果工作台 -> 打开草稿`

### 验收点

- [ ] 当前图片目录入口能被触发
- [ ] 结果工作台能展示 `inbox_path` / `draft_path` / `html_path`
- [ ] 生成出来的草稿能在 vault 中打开
- [ ] 草稿内容基本符合“按真实信息先归档”的预期

### 备注

照片路线当前最缺的不是代码，而是：

**真实样例证据。**

## 证据 D：真实微信回归

### 目标

证明 v0.4.2 发布前至少有一次真实微信公众号草稿和后台预览路径证据。

### 建议截图或短录屏

至少准备下面 3 个节点：

1. `wechat-draft-created.png`
   内容：
   - 微信草稿创建成功
2. `configure-headed.png`
   内容：
   - `configure --headed` 进入后台配置流程
3. `preview-sent.png`
   内容：
   - 微信公众号后台预览发送结果

### 验收点

- [ ] 微信草稿创建成功或失败原因已记录
- [ ] 原创声明 / 赞赏 / 留言 / 创作来源至少有真实尝试记录
- [ ] 后台预览发送成功或失败原因已记录
- [ ] 最终发表仍由人工确认，没有自动点击发表
- [ ] 截图不包含二维码、token、cookie、AppSecret、手机号、账号隐私或后台敏感 URL

## 证据存放建议

建议新建一个统一目录，例如：

`/private/tmp/moonpub-first-run-evidence/`

可以按下面结构组织：

```text
moonpub-first-run-evidence/
  homepage/
    homepage-workspace.png
    homepage-context.png
  feishu/
    feishu-home-entry.png
    feishu-result-modal.png
    feishu-draft-opened.png
  photos/
    photos-image-opened.png
    photos-result-modal.png
    photos-draft-opened.png
  wechat/
    wechat-draft-created.png
    configure-headed.png
    preview-sent.png
  NOTES.md
```

如果你希望把最终留档也放回仓库，而不是只放在临时目录，现在仓库里已经补了统一归档位和记录模板：

- `docs/first-run-evidence/README.md`
- `docs/first-run-evidence/NOTES.md`
- `docs/first-run-evidence/homepage/README.md`
- `docs/first-run-evidence/feishu/README.md`
- `docs/first-run-evidence/photos/README.md`
- `docs/first-run-evidence/wechat/README.md`

可以用下面的本地只读命令快速检查缺哪些证据文件：

```bash
moonpub evidence-status
moonpub --json evidence-status
```

`evidence-status` 会显示已归档 / 必需 / 缺失数量和缺失路径清单；它只检查文件是否存在，不打开图片、不读取图片内容，也不替代人工脱敏审查。

## 记录模板

每次补证据时，建议顺手填一份简单记录：

```text
日期：
MoonPub 版本：
入口类型：首页 / 飞书 / 照片 / 微信回归
是否使用插件首页：是 / 否
是否成功生成草稿：是 / 否
是否成功打开草稿：是 / 否
是否成功拿到本地预览：是 / 否
问题记录：
```

## 完成标准

如果这轮证据补齐，可以认为：

- 插件首页证据：`通过`
- 飞书插件首次体验证据：`通过`
- 照片插件首次体验证据：`通过`
- 真实微信回归证据：`通过`

在那之前，更准确的说法仍然应该是：

**入口、文档、测试已收口，但真实首次体验证据仍在继续补。**
