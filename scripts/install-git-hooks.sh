#!/usr/bin/env bash
# Install project git hooks into the local repository.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

GIT_DIR=$(git rev-parse --git-common-dir 2>/dev/null || git rev-parse --git-dir)
HOOKS_DIR="$GIT_DIR/hooks"

mkdir -p "$HOOKS_DIR"
cp -v .githooks/commit-msg "$HOOKS_DIR/"
echo "Git hooks installed to $HOOKS_DIR"
