# MoonPub 新手上手指南

> 这是精简版本地快速开始。完整用户指南（含飞书、照片、Obsidian 插件、微信凭证、一键发布和常见问题）见 [USER_ONBOARDING_ZH.md](USER_ONBOARDING_ZH.md) 中文 / [USER_ONBOARDING.md](USER_ONBOARDING.md) English。

## 目标

先在没有微信凭证的情况下，走完本地渲染和预览，确认 MoonPub 能工作。

## 安装

**macOS Apple Silicon（预编译二进制）：**

```bash
curl -L https://github.com/qiaopengjun5162/moonpub/releases/latest/download/moonpub-macos-arm64.tar.gz | tar xz
sudo mv moonpub /usr/local/bin/
```

链接里的 `latest` 会自动指向当前最新 release（不用改版本号）。macOS Intel 换 `moonpub-macos-amd64.tar.gz`；Linux x86_64 换 `moonpub-linux-amd64.tar.gz`；Linux ARM64 换 `moonpub-linux-arm64.tar.gz`。

每个压缩包都有同名 `.sha256`，建议先校验再安装：

```bash
curl -L -O https://github.com/qiaopengjun5162/moonpub/releases/latest/download/moonpub-macos-arm64.tar.gz.sha256
shasum -a 256 -c moonpub-macos-arm64.tar.gz.sha256
```

**Windows：** 从 [Releases](https://github.com/qiaopengjun5162/moonpub/releases) 下载 `moonpub-windows-amd64.zip`，解压后把 `moonpub.exe` 加入 PATH。

**Cargo：**

```bash
cargo install --git https://github.com/qiaopengjun5162/moonpub
```

验证：

```bash
moonpub --help              # 全量命令，按任务分组
moonpub help push           # 单个命令的用法、说明和全局参数
moonpub help errors         # 全部 error/v1 错误码与示例下一步
```

## 30 秒本地体验

```bash
cd /path/to/your/articles
moonpub init --yes    # 直接写默认配置；去掉 --yes 会走进交互式配置向导
moonpub new "我为什么开始每天读书"
# 编辑 Articles/drafts/我为什么开始每天读书.md
moonpub render Articles/drafts/我为什么开始每天读书.md
moonpub preview Articles/drafts/我为什么开始每天读书.md
moonpub cover Articles/drafts/我为什么开始每天读书.md --style literary
```

这一步只读写本地文件，不调用微信 API，不发布任何内容。

## 下一步

- 想继续推到微信草稿、配置飞书/照片入口、用 Obsidian 插件：看完整指南 [USER_ONBOARDING_ZH.md](USER_ONBOARDING_ZH.md) 中文 / [USER_ONBOARDING.md](USER_ONBOARDING.md) English
- 想理解三条工作流路径：[RECOMMENDED_WORKFLOWS_ZH.md](RECOMMENDED_WORKFLOWS_ZH.md) 中文 / [RECOMMENDED_WORKFLOWS.md](RECOMMENDED_WORKFLOWS.md) English
- 想从 Obsidian 插件首页开始：[FIRST_RUN_WALKTHROUGH_ZH.md](FIRST_RUN_WALKTHROUGH_ZH.md) 中文 / [FIRST_RUN_WALKTHROUGH.md](FIRST_RUN_WALKTHROUGH.md) English
- 想排障：[ENGINEERING_LESSONS_ZH.md](ENGINEERING_LESSONS_ZH.md) 中文 / [ENGINEERING_LESSONS.md](ENGINEERING_LESSONS.md) English
