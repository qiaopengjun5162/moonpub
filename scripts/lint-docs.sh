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
      git diff --name-only --diff-filter=AM "$BASE...HEAD"
      git diff --name-only --diff-filter=AM
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
