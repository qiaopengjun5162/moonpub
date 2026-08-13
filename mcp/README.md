# MoonPub MCP Server

把 [moonpub](https://github.com/...)（公众号发布 CLI）的子命令暴露为 MCP tools，
让 Claude Desktop / Cursor / 任意 MCP client 能直接驱动发文章，而不必手写 bash。

## 设计

薄壳：每个 tool 拼出 argv → 调用 `moonpub` 二进制 → 把 stdout 作为结构化结果返回。
所有调用统一在全局位追加 `--json`（见 `src/app.rs:453`，非结构化命令也会被包成
JSON 字符串），因此 tool 拿到的始终是合法 JSON，处理统一。不重复造 JSON 解析。

## 安装

```bash
cd mcp
pip3 install -r requirements.txt
```

要求 Python 3.10+（用了 `str | None` 语法）。

## 运行

```bash
python3 mcp/server.py          # stdio 传输，供 MCP client 连接
```

### 环境变量

| 变量 | 说明 | 默认 |
|------|------|------|
| `MOONPUB_BIN` | moonpub 可执行文件路径 | `moonpub`（需已在 PATH） |
| `MOONPUB_ARTICLES` | 默认 articles 根目录，作为 `--articles` 传入 | 未设置则依赖 moonpub 自动发现（从文章路径向上找 `moonpub.toml`） |
| `MOONPUB_TIMEOUT` | 单条命令超时秒数 | `300` |

## 接入 MCP client

把下面配置写进对应 client 的 MCP 配置文件（路径见下），注意改成你本机的绝对路径：

```json
{
  "mcpServers": {
    "moonpub": {
      "command": "python3",
      "args": ["/abs/path/to/moonpub/mcp/server.py"],
      "env": {
        "MOONPUB_BIN": "moonpub",
        "MOONPUB_ARTICLES": "/abs/path/to/articles"
      }
    }
  }
}
```

- **Claude Desktop**：`~/Library/Application Support/Claude/claude_desktop_config.json`
- **Cursor**：`~/.cursor/mcp.json`
- 也可直接参考 `moonpub-mcp.example.json`（已填本机路径）。

## 可用 tools

诊断：`doctor` / `workspace` / `status` / `capabilities` / `list_drafts`
发布闭环：`preflight` / `render` / `cover` / `push` / `preview` / `publish` / `ship` / `mark_ready` / `delete_draft`
浏览器自动化：`login` / `test_yulan`
创作：`new` / `write`
素材：`intake_feishu` / `intake_photos`
通用：`run`（escape hatch，执行任意子命令）

## 注意事项（踩坑点）

- **封面必须先 `--screenshot`**：`cover` 默认只生成预览，不会写回 frontmatter 的
  `cover` 字段；不写回会导致 `push` 报 `40007 invalid media_id`。
- **IP 白名单**：`push` 依赖微信公众号 API，出口 IP 需在公众号后台加白名单，
  否则报 `40164`。
- **`test_yulan` 需先 `login` 扫码**，且扫码后要关掉 MoonPub 自动化打开的 Chrome
  窗口以释放 profile 锁。
- 每个 tool 返回结构：`{ ok, exit_code, json, stdout, stderr }`。非结构化命令的
  `json` 为字符串；结构化命令的 `json` 为解析后的对象。失败时看 `exit_code`/`stderr`。

## 与 Skill 的关系

`moonpub-wechat-publish` skill 是「知识 / 流程 / 方法论」层（教 agent 怎么做、怎么
评估移植外部仓库）；本 MCP server 是「执行」层（让 agent 直接调用能力）。两者互补。
