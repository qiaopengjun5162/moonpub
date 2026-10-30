#!/usr/bin/env bash
# Lint Markdown files. Default: lint all tracked Markdown files.
# Pass --changed to lint only files changed against origin/main.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

MDLINT="markdownlint-cli2"
if ! command -v "$MDLINT" >/dev/null 2>&1; then
  MDLINT="npx markdownlint-cli2"
fi

if [[ "${1:-}" == "--changed" ]]; then
  BASE="${BASE_BRANCH:-origin/main}"
  CHANGED=$(
    {
      # core.quotepath=false：否则中文文件名会被转义成八进制并加引号，
      # 行尾不再是 .md，下面的 grep 会把它们全部漏掉。
      git -c core.quotepath=false diff --name-only --diff-filter=AM "$BASE...HEAD"
      # HEAD（不是默认的工作区 vs 暂存区）才能覆盖已 git add 但未 commit 的文件
      git -c core.quotepath=false diff --name-only --diff-filter=AM HEAD
    } | grep '\.md$' | sort -u || true
  )
  if [[ -z "$CHANGED" ]]; then
    echo "No Markdown files changed against $BASE."
    exit 0
  fi
  echo "Linting changed Markdown files:"
  echo "$CHANGED"
  echo "$CHANGED" | xargs -r "$MDLINT" --no-globs
else
  echo "Linting all Markdown files..."
  "$MDLINT"
fi
