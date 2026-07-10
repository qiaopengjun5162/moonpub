# WeChat Evidence

这个目录只放 v0.4.2 release gate 需要的真实微信回归证据。

必须文件：

- `wechat-draft-created.png`：微信草稿已创建
- `configure-headed.png`：`configure --headed` 可进入后台配置流程
- `preview-sent.png`：微信公众号后台预览发送结果

要求：

- 截图必须脱敏，不提交二维码、token、cookie、AppSecret、手机号、账号隐私或后台敏感 URL。
- 这些截图只能证明一次真实回归证据存在，不能替代人工复查截图内容是否安全。
- 缺文件时，`moonpub evidence-status` 会继续把 v0.4.2 证据状态标记为未通过。
