# Pull Request

## 改动类型

- [ ] feat：新功能
- [ ] fix：bug 修复
- [ ] docs：文档更新
- [ ] refactor：重构，行为不变
- [ ] test：测试补充或修复
- [ ] build：构建/依赖/工具链
- [ ] ci：CI/check 门禁
- [ ] chore：日常维护

## 改动说明

简要说明本次改动解决了什么问题、为什么这样改。避免营销式或空泛表达。

## 涉及模块

列出主要改动的文件/模块，例如 `src/app.rs`、`src/push.rs`、`docs/`、`obsidian-plugin/`。

## 验证命令

```bash
# 根据改动类型选择
cargo fmt --all -- --check
cargo clippy --all-targets --all-features --tests --benches -- -D warnings
cargo nextest run --all-features

# 如果改动 obsidian-plugin/
cd obsidian-plugin && npm ci && npm test && npm run build
```

## 文档同步

- [ ] 已更新相关 README / README_zh / docs / PROGRESS / 插件 README
- [ ] 不需要文档更新（纯格式或机械改动）

## 生产边界

如果改动触及以下任一边界，请说明风险点和回滚方式：

- [ ] 微信 API（push / ship / configure / login）
- [ ] 浏览器自动化（CDP / headless Chrome / 微信后台）
- [ ] AI provider（write / expand / polish / ship --ai / intake --analyze-images）
- [ ] 文件写回（Inbox / drafts / ready / published / 配置/凭据文件）
- [ ] 插件/浏览器端行为

## 凭据/隐私检查

- [ ] 没有将真实凭据（WECHAT_SECRET、DEEPSEEK_API_KEY、.env、session、cookie、pass_ticket、uin、token）写入 git

## 回滚说明（可选）

如果改动涉及 runtime 行为或配置格式变更，说明如何回滚。
