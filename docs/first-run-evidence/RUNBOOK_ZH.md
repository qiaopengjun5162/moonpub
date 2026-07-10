# MoonPub 首次体验取证 Runbook

这份 runbook 用来指导一次真实取证。

目标不是证明 MoonPub “已经完美”，而是留下可复查的证据：

- 用户能不能从入口进来
- 用户能不能看懂下一步
- 草稿和本地预览是否真实生成
- 哪些步骤仍然需要人工确认

## 取证前检查

取证前先确认这些条件：

- 当前仓库处于干净状态：`git status --short --branch`
- Obsidian 插件已经能找到本地 `moonpub` 可执行文件
- Obsidian 插件设置里已经配置 `Articles 根目录`
- 不打开 `.env`、`moonpub.toml`、浏览器 cookie、二维码、AppSecret 等敏感内容
- 截图前先确认屏幕上没有私人聊天、真实 token、私人照片细节或不可公开路径

如果截图里必须出现本地路径，优先裁切到只保留 MoonPub 工作台主体。

## 路径 A：首页工作台

目标：

证明插件首页已经能作为首次体验入口，而不是只是一条命令。

建议步骤：

1. 在 Obsidian 打开一篇 Markdown 文章。
2. 执行命令：`打开 MoonPub 首页`。
3. 截图保存为 `homepage/homepage-workspace.png`。
4. 确认截图里能看到：
   - `MoonPub 首页工作台`
   - 当前入口
   - drafts / ready / published 阶段数量
   - 首页快捷动作
5. 再切换到一张图片或没有打开文件的状态，重新打开首页。
6. 截图保存为 `homepage/homepage-context.png`。
7. 确认截图里能看到：
   - 当前上下文
   - 当前更推荐
   - 首次建议步骤

通过标准：

- 首页不只是显示状态，还能解释“当前更推荐做什么”
- 当前打开 Markdown 和当前打开图片时，推荐语义不同
- 截图不暴露敏感配置或私人内容

## 路径 B：飞书首次体验

目标：

证明用户可以从插件首页进入飞书入口，并停在可编辑草稿和本地预览。

建议步骤：

1. 打开插件首页。
2. 截图首页里的飞书入口，保存为 `feishu/feishu-home-entry.png`。
3. 点击或执行：`导入最近一条飞书妙记并生成草稿预览`。
4. 等待结果工作台出现。
5. 截图保存为 `feishu/feishu-result-modal.png`。
6. 点击或确认草稿已在 Obsidian 打开。
7. 截图保存为 `feishu/feishu-draft-opened.png`。

通过标准：

- 结果工作台能看到 Inbox / Draft / HTML 预览路径
- 本次没有自动进入最终发布
- 草稿能被用户继续编辑
- 如果飞书内容涉及隐私，截图只保留工作台结构，裁掉正文敏感内容

## 路径 C：照片首次体验

目标：

证明用户可以从当前图片所在目录生成照片草稿和本地预览。

建议准备：

- 2 到 5 张可公开或可裁切的测试照片
- 最好属于同一天或同一组生活素材
- 不包含身份证、车牌、地址、聊天记录、人脸隐私等内容

建议步骤：

1. 在 Obsidian 打开一张测试图片。
2. 截图保存为 `photos/photos-image-opened.png`。
3. 打开插件首页，确认当前上下文推荐照片路径。
4. 执行：`导入当前图片所在目录并生成照片草稿预览`。
5. 等待照片结果工作台出现。
6. 截图保存为 `photos/photos-result-modal.png`。
7. 点击或确认草稿已在 Obsidian 打开。
8. 截图保存为 `photos/photos-draft-opened.png`。

通过标准：

- 结果工作台能看到 Inbox / Draft / HTML 预览路径
- 草稿内容符合“实事求是记录照片信息”的预期
- 不把测试照片误写成夸张营销文案
- 截图不泄露私人照片细节

## 路径 D：真实微信回归

目标：

证明 v0.4.2 发布前至少有一次真实微信草稿和后台预览路径证据。

建议步骤：

1. 先运行 `moonpub wechat-health`，确认浏览器自动化登录态。
2. 对一篇可公开测试文章运行 `moonpub push <article.md> --render`。
3. 运行 `moonpub configure --headed --evidence-dir docs/first-run-evidence/wechat`。
4. 确认命令生成了 `wechat/wechat-draft-created.png`、`wechat/configure-headed.png`、`wechat/preview-sent.png`。
5. 人工打开三张图逐张脱敏检查，必要时裁剪或打码后再提交。

通过标准：

- 草稿确实进入微信公众号后台
- 原创、赞赏、留言、创作来源和预览发送至少有一次真实尝试记录
- 最终发表仍由人工确认，不能自动点击发表
- 截图不包含二维码、token、cookie、AppSecret、手机号、账号隐私或后台敏感 URL

## 取证后记录

每次补完证据，都要更新 `docs/first-run-evidence/NOTES.md`。

建议至少记录：

- 日期
- MoonPub 版本或 commit
- 入口类型
- 是否使用插件首页
- 是否成功生成草稿
- 是否成功打开草稿
- 是否成功拿到本地预览
- 证据文件列表
- 仍需人工确认的边界

如果某条路径失败，也要记录失败原因。

失败证据同样有价值，因为它能告诉后续优化该从哪里下手。
