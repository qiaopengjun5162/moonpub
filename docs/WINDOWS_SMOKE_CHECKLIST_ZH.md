# MoonPub Windows Smoke Checklist

这份清单用于验证发布给用户的 `moonpub-windows-amd64.zip` 是否真的可用。

目标不是证明“Windows 全链路发布已经稳定”，而是确认 Windows release 资产至少能跑通无凭证本地首跑路径。

## 当前结论

- PR CI 已在 `windows-latest` 上通过源码构建二进制 smoke：
  - `moonpub.exe --version`
  - `moonpub.exe --help`
  - `moonpub.exe init moonpub.toml`
  - `moonpub.exe new "Windows Smoke"`
  - `moonpub.exe render "Articles\drafts\windows-smoke.md"`
  - `moonpub.exe check "Articles\drafts\windows-smoke.md"`
- 这份清单补的是另一层证据：
  - 用户实际下载的 release zip 是否也能跑通同一路径

## 前置条件

- Windows 10/11
- PowerShell
- 已从 Releases 下载 `moonpub-windows-amd64.zip`
- 不需要微信凭证
- 不需要 DeepSeek API Key

## 验证步骤

1. 解压 `moonpub-windows-amd64.zip`
2. 在解压目录打开 PowerShell
3. 创建一个临时空目录并切进去

```powershell
New-Item -ItemType Directory -Force -Path "$env:TEMP\moonpub-windows-smoke" | Out-Null
Set-Location "$env:TEMP\moonpub-windows-smoke"
```

4. 运行以下命令

```powershell
\path\to\moonpub.exe --version
\path\to\moonpub.exe --help
\path\to\moonpub.exe init moonpub.toml
\path\to\moonpub.exe new "Windows Smoke"
\path\to\moonpub.exe render "Articles\drafts\windows-smoke.md"
\path\to\moonpub.exe check "Articles\drafts\windows-smoke.md"
```

如果已经把 `moonpub.exe` 加进 `PATH`，也可以直接写：

```powershell
moonpub.exe --version
moonpub.exe --help
moonpub.exe init moonpub.toml
moonpub.exe new "Windows Smoke"
moonpub.exe render "Articles\drafts\windows-smoke.md"
moonpub.exe check "Articles\drafts\windows-smoke.md"
```

## 期望结果

- `--version` 输出当前版本号
- `--help` 正常显示命令帮助
- `init` 生成可直接使用的 `moonpub.toml`
- `new` 创建 `Articles\drafts\windows-smoke.md`
- `render` 生成对应 `.html` 和 `.draft.json`
- `check` 正常输出文章包状态

## 失败时先看什么

- `moonpub.exe` 无法运行：先确认 zip 是否完整解压、PowerShell 当前目录是否正确
- `new` 失败：检查 `moonpub.toml` 是否成功生成，当前目录是否可写
- `render` 失败：检查 `Articles\drafts\windows-smoke.md` 是否存在
- `check` 失败：优先保留终端输出，再回仓库提 issue

## 记录方式

建议把以下信息记进 release 验证记录：

- Windows 版本
- MoonPub 版本
- 是否直接运行 zip 内二进制，还是已加入 `PATH`
- 六条命令是否全部通过
- 若失败，失败命令和终端输出摘要
