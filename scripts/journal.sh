#!/usr/bin/env bash
# 生成一次操作的记录骨架，或者列出已有记录。
#
#   ./scripts/journal.sh "覆盖率实证"    # 新建今天的记录
#   ./scripts/journal.sh --list          # 列出已有记录
#
# 只生成骨架，不写内容。它省掉的是新建文件、敲日期、翻 git log 这几步摩擦。

set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

OPS_DIR="writing/operations"
mkdir -p "$OPS_DIR"

if [[ "${1:-}" == "--list" || "${1:-}" == "-l" ]]; then
  echo "已有记录（$OPS_DIR）："
  found=0
  for f in "$OPS_DIR"/*.md; do
    [[ -e "$f" ]] || continue
    [[ "$(basename "$f")" == "TEMPLATE.md" ]] && continue
    printf '  %s  (%s 行)\n' "$(basename "$f")" "$(wc -l < "$f" | tr -d ' ')"
    found=1
  done
  [[ $found -eq 1 ]] || echo "  （还没有，用 ./scripts/journal.sh \"标题\" 建一个）"
  exit 0
fi

TITLE="${1:-}"
if [[ -z "$TITLE" ]]; then
  echo "用法: ./scripts/journal.sh \"短标题\"" >&2
  echo "      ./scripts/journal.sh --list" >&2
  exit 1
fi

DATE="$(date +%Y-%m-%d)"
# 文件名里的斜杠和空格会破坏路径，统一换掉
SLUG="$(printf '%s' "$TITLE" | tr ' /' '--')"
OUT="$OPS_DIR/${DATE}-${SLUG}.md"

if [[ -e "$OUT" ]]; then
  echo "已存在：$OUT"
  echo "（不会覆盖已有记录，换个标题，或直接编辑它）"
  exit 0
fi

BRANCH="$(git rev-parse --abbrev-ref HEAD 2>/dev/null || echo unknown)"

# 当天提交（按仓库本地时区），最多 20 条
TODAY_LOG="$(git log --since="${DATE} 00:00" --format='- `%h` %s' 2>/dev/null | head -20 || true)"
[[ -n "$TODAY_LOG" ]] || TODAY_LOG="（今天还没有提交）"

DIRTY="$(git status --porcelain 2>/dev/null | wc -l | tr -d ' ')"

cat > "$OUT" <<EOF
# ${DATE} ${TITLE}

**上下文**：一句话说明当时在做什么、为什么现在做这件事。

## 意图

想达成什么。

## 输入

涉及文件、命令、前提条件。

\`\`\`bash
# 实际跑的命令
\`\`\`

## 输出

实际发生了什么，原样贴关键输出。

\`\`\`text

\`\`\`

## 解释

为什么会有这个输出。

## 问题

> 现象：

## 怎么解决的

1.
2.
3.

## 证据

- 提交：
- CI run：
- 截图：
- 复算命令：

## 能写成什么

一句话：这段经历可以支撑哪篇文章的哪个部分。

## 还有没解决的

无

---

<details>
<summary>当天提交（自动抓取，供参考，可删）</summary>

当前分支：\`${BRANCH}\`　未提交改动：${DIRTY} 个文件

${TODAY_LOG}

</details>
EOF

echo "已生成：$OUT"
echo
echo "接着做三件事："
echo "  1. 填「意图」和「问题」两栏（写不动就先只填这两栏）"
echo "  2. 命令和输出原样贴，不要复述"
echo "  3. 有截图的，按 writing/PROTOCOL.md 的命名规范放进 writing/assets/"
