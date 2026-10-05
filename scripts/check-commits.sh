#!/usr/bin/env bash
# Check that all commits in a range follow Conventional Commits.
# Usage: ./scripts/check-commits.sh [base-branch]
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

BASE="${1:-origin/main}"
ALLOWED_TYPES="build|chore|ci|docs|feat|fix|perf|refactor|revert|style|test"
PATTERN="^(${ALLOWED_TYPES})(\(.+\))?!?: .+"

COMMITS=$(git rev-list --no-merges "${BASE}..HEAD" 2>/dev/null || true)
if [[ -z "$COMMITS" ]]; then
  echo "No commits to check between $BASE and HEAD."
  exit 0
fi

INVALID=0
for commit in $COMMITS; do
  MSG=$(git log -1 --format=%s "$commit")
  if [[ ! "$MSG" =~ $PATTERN ]]; then
    echo "Invalid commit message at $commit: $MSG" >&2
    INVALID=1
  fi
done

if [[ "$INVALID" -ne 0 ]]; then
  echo "" >&2
  echo "Commit messages must follow Conventional Commits." >&2
  echo "Allowed types: ${ALLOWED_TYPES//|/ }" >&2
  exit 1
fi

echo "All commits follow Conventional Commits."
