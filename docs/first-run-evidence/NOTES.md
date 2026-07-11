# First-Run Evidence Notes

## Record Template

```text
日期：
MoonPub 版本：
Commit：
入口类型：首页 / 飞书 / 照片
是否使用插件首页：是 / 否
是否成功生成草稿：是 / 否
是否成功打开草稿：是 / 否
是否成功拿到本地预览：是 / 否
证据文件：
仍需人工确认的边界：
问题记录：
```

## Records

### Pending

- `homepage/homepage-context.png` 还未补真实截图
- `feishu/feishu-home-entry.png` 还未补真实截图
- `feishu/feishu-result-modal.png` 还未补真实截图
- `feishu/feishu-draft-opened.png` 还未补真实截图
- `photos/photos-image-opened.png` 还未补真实截图
- `photos/photos-result-modal.png` 还未补真实截图
- `photos/photos-draft-opened.png` 还未补真实截图

### Homepage

```text
日期：2026-07-10
MoonPub 版本：0.4.2 source build
入口类型：首页
证据目录：docs/first-run-evidence/homepage/
证据文件：
- homepage-workspace.png
仍需人工确认的边界：截图已裁剪并遮盖本机 Articles 绝对路径；仍缺当前 Markdown / 图片上下文截图。
问题记录：真实 vault 中其它插件异常会影响命令面板列表，因此新增并验证左侧 Ribbon 首页入口；同时发现 PATH 中旧版 moonpub 可通过 --help 检测但不支持 workspace，插件已改为要求 moonpub --json doctor 兼容性检查。
```

### Feishu

```text
日期：
MoonPub 版本：
入口类型：飞书
是否使用插件首页：是 / 否
证据目录：docs/first-run-evidence/feishu/
证据文件：
- feishu-home-entry.png
- feishu-result-modal.png
- feishu-draft-opened.png
问题记录：
```

### Photos

```text
日期：
MoonPub 版本：
入口类型：照片
是否使用插件首页：是 / 否
证据目录：docs/first-run-evidence/photos/
证据文件：
- photos-image-opened.png
- photos-result-modal.png
- photos-draft-opened.png
问题记录：
```

### WeChat

```text
日期：
MoonPub 版本：
入口类型：真实微信回归
证据目录：docs/first-run-evidence/wechat/
证据文件：
- wechat-draft-created.png
- configure-headed.png
- preview-sent.png
是否成功创建微信草稿：是 / 否
是否成功进入后台配置：是 / 否
是否成功发送微信公众号后台预览：是 / 否
是否已人工确认截图脱敏：是 / 否
问题记录：
```

```text
日期：2026-07-10
MoonPub 版本：v0.4.2 source build
入口类型：真实微信回归预检
证据目录：docs/first-run-evidence/wechat/
证据文件：
- 暂无截图文件，本轮只记录命令级健康检查结果
wechat-health：通过，返回 status=ready、profile_mode=persistent、session_file_exists=true
脱敏 current_url：https://mp.weixin.qq.com/cgi-bin/home
是否成功创建微信草稿：否，本轮未触发 push
是否成功进入后台配置：否，本轮未触发 configure
是否成功发送微信公众号后台预览：否，本轮未触发 preview-send
是否已人工确认截图脱敏：否，仍需补 3 张真实微信截图后再确认
问题记录：登录态可复用，下一步应选定一篇可公开测试文章，运行 push --render，再运行 configure --headed 并补齐 wechat-draft-created.png / configure-headed.png / preview-sent.png。
```

```text
日期：2026-07-10
MoonPub 版本：v0.4.2 source build
入口类型：真实微信回归
证据目录：docs/first-run-evidence/wechat/
证据文件：
- wechat-draft-created.png
- configure-headed.png
- preview-sent.png
测试文章：/private/tmp/moonpub-wechat-regression/Articles/drafts/moonpub-v042-wechat-regression.md
是否成功创建微信草稿：是，media_id 已脱敏不记录
是否成功进入后台配置：是，session 恢复后进入编辑器
是否成功发送微信公众号后台预览：是
是否已人工确认截图脱敏：是，已遮挡账号名、头像、预览微信号、操作人和赞赏账号等后台隐私信息
问题记录：原创声明、赞赏、留言、创作来源配置成功；模板未配置时软跳过；最终发表未点击。微信三张证据已补，首页、飞书、照片证据仍缺。
```
