# MoonPub 真实微信草稿回归清单

这份清单用于验证 v0.4.1 在真实微信公众号环境中的草稿推送和浏览器辅助配置。它不是自动发布流程，也不要求 MoonPub 点击最终发表。

## 前置条件

- [ ] 已准备一个可测试的微信公众号账号。
- [ ] 本机 IP 已加入微信公众平台 IP 白名单。
- [ ] `WECHAT_APPID` 已通过环境变量或本地 env 文件提供。
- [ ] `WECHAT_SECRET` 已通过环境变量或本地 env 文件提供，未写入仓库。
- [ ] 本机已安装 Chrome 或 Chromium。
- [ ] 已准备 v0.4.1 release 二进制。
- [ ] 已理解 `push` / `ship` 会调用真实微信 API 并创建草稿。

## 安全边界

- 不读取、打印或提交 `.env`、`moonpub.toml` 中的真实凭据。
- 不记录 AppSecret、access token、cookie、二维码内容或登录态文件。
- 不绕过扫码、验证码、平台审核、账号权限或最终人工确认。
- 不默认点击“发表”。
- 若微信 UI 变化导致 `configure` 失败，记录失败步骤；不要把失败包装成成功。

## 推荐验证顺序

先使用已经生成的演示文章，或复制一篇低风险测试文章到草稿目录。

```bash
moonpub --version
moonpub login
moonpub push "Articles/drafts/文章名.md" --render
moonpub configure --headed
moonpub status
moonpub check "Articles/ready/文章名.md"
```

## 期望结果

- [ ] `moonpub --version` 输出 `moonpub 0.4.1`。
- [ ] `moonpub login` 由用户自己扫码完成。
- [ ] `moonpub push --render` 创建微信草稿。
- [ ] 草稿创建后本地文章包进入 `Articles/ready/`，不是 `Articles/published/`。
- [ ] `.media_id` 文件存在，但不对外公开具体值。
- [ ] `moonpub configure --headed` 可见模式能尝试配置原创、赞赏、留言、创作来源和预览。
- [ ] 自动化失败时不影响已创建的微信草稿。
- [ ] 用户在微信后台人工检查草稿。
- [ ] 最终发表仍由用户自己决定。

## 回归记录模板

```text
日期：
MoonPub 版本：
平台：
微信账号类型：
IP 白名单：已配置 / 未配置
login：通过 / 未通过
push --render：通过 / 未通过
configure --headed：通过 / 部分通过 / 未通过
进入 Articles/ready：是 / 否
是否点击最终发表：否
问题记录：
```

## 可接受的首版结论

首版对外发布可以写：

> v0.4.1 已通过本地无凭证 release smoke test；真实微信草稿回归仍在持续补充。MoonPub 当前适合技术用户试用，不承诺无人值守自动发布。

只有在完整跑过本清单后，才可以把文案升级为“已完成真实微信草稿回归”。
