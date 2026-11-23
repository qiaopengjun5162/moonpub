#!/usr/bin/env bash
# Gate a pending release-please PR before it is merged.
#
# Why this exists: release-please opens its PR with `GITHUB_TOKEN`, and GitHub
# does not start the workflow runs for it until a human approves them - the run
# exists but has **zero jobs**. `gh pr checks` sees nothing to report, prints
# "no checks reported" and exits 1, and nothing else stops the merge: the green
# Merge button does not look at CI (this repo has no required checks), and
# `gh pr merge` does not either. Nine releases (v0.4.15 through v0.4.23) were
# merged with zero CI jobs executed; v0.4.24 was the first that ran its gate
# (5 jobs, after approving run 45099669061).
#
# So this script answers three questions that a merge button cannot:
#   1. is this really a release PR (only version metadata touched)?
#   2. did CI actually run - and if it is still held, what is the command?
#   3. is every check green? An empty list is NOT a pass: exit 3.
#
# Usage:
#   ./scripts/release-pr-check.sh            # inspect only, never approves
#   ./scripts/release-pr-check.sh --approve  # approve held runs, then wait
#   ./scripts/release-pr-check.sh --pr 243   # inspect a specific PR (debug)
#
# Exit codes:
#   0  ready to merge: pure release PR, CI ran and passed
#   1  CI failed
#   2  not a pure release PR: something other than version metadata changed
#   3  not verifiable by a script: runs still held, still running, or never ran
#
# Requires `gh` authenticated with write access for --approve.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# The only files a release PR may touch. Kept here as the single source of
# truth: docs/RELEASE_CHECKLIST.md points at this script instead of restating
# the list, so the two cannot drift apart.
ALLOWED_FILES=(
  .release-please-manifest.json
  CHANGELOG.md
  Cargo.toml
  Cargo.lock
  ROADMAP.md
)

POLL_SECONDS="${RELEASE_PR_POLL_SECONDS:-20}"
TIMEOUT_SECONDS="${RELEASE_PR_TIMEOUT_SECONDS:-1800}"

PR=""
DO_APPROVE=0
while [ $# -gt 0 ]; do
  case "$1" in
    --approve) DO_APPROVE=1 ;;
    --pr)
      shift
      PR="${1:-}"
      if [ -z "$PR" ]; then
        echo "--pr needs a PR number" >&2
        exit 64
      fi
      ;;
    -h | --help)
      # Print the header comment (everything before `set -euo pipefail`).
      sed -n '2,/^set -euo/p' "${BASH_SOURCE[0]}" | sed '$d' | sed 's/^# \{0,1\}//'
      exit 0
      ;;
    *)
      echo "unknown option: $1" >&2
      exit 64
      ;;
  esac
  shift
done

if ! command -v gh >/dev/null 2>&1; then
  echo "gh is required: https://cli.github.com" >&2
  exit 1
fi

if [ -z "$PR" ]; then
  PR="$(gh pr list --state open --limit 50 \
    --json number,headRefName \
    --jq '[.[] | select(.headRefName | startswith("release-please--"))] | first | .number // empty')"
  if [ -z "$PR" ]; then
    echo "OK: 没有待合并的发版 PR（head 分支前缀 release-please--），无需检查。"
    exit 0
  fi
fi

# Tab-separated on purpose: the title contains spaces, and plain `read` would
# split on those and silently shift every later field by one.
IFS=$'\t' read -r title head_branch pr_state head_sha < <(
  gh pr view "$PR" --json title,headRefName,state,headRefOid \
    --jq '"\(.title)\t\(.headRefName)\t\(.state)\t\(.headRefOid)"'
)

echo "PR #$PR  $title"
echo "  branch: $head_branch   state: $pr_state"
echo "  head:   $head_sha"

# --- 1. is it really a release PR? -----------------------------------------
extra=""
while IFS= read -r path; do
  [ -z "$path" ] && continue
  found=0
  for allowed in "${ALLOWED_FILES[@]}"; do
    if [ "$path" = "$allowed" ]; then
      found=1
      break
    fi
  done
  if [ "$found" -eq 0 ]; then
    extra="$extra $path"
  fi
done <<<"$(gh pr diff "$PR" --name-only)"

if [ -n "$extra" ]; then
  echo "FAIL: 这不是纯版本 PR，除版本元数据外还改了:$extra" >&2
  echo "      发版 PR 只应动 ${ALLOWED_FILES[*]}" >&2
  exit 2
fi
echo "  files: 只改版本元数据 ✓"

# --- 2. did CI actually run? ------------------------------------------------
# Held runs live outside the check rollup, so look at the runs of *this commit*.
# `--branch` would be wrong here: every release-please PR shares the same head
# branch name (`release-please--branches--main--components--<pkg>`), so it also
# returns the held runs of every earlier release.
held_ids=""
held_lines=""
while IFS=$'\t' read -r run_id workflow status conclusion; do
  [ -z "$run_id" ] && continue
  if [ "$conclusion" = "action_required" ] || [ "$status" = "waiting" ]; then
    held_ids="$held_ids $run_id"
    held_lines="$held_lines
  - $workflow  (run $run_id)"
  fi
done <<<"$(gh run list --commit "$head_sha" --limit 30 \
  --json databaseId,workflowName,status,conclusion \
  --jq '.[] | "\(.databaseId)\t\(.workflowName)\t\(.status)\t\(.conclusion)"' \
  2>/dev/null || true)"

if [ -n "$held_ids" ]; then
  count=$(echo "$held_ids" | wc -w | tr -d ' ')
  echo "  runs: $count 个被 GitHub 扣住（action_required），CI 从未执行：$held_lines"
  if [ "$DO_APPROVE" -eq 0 ]; then
    echo
    echo "需要人工批准（GitHub 对 bot 创建的 PR 默认要求批准）。任选其一：" >&2
    echo "  ./scripts/release-pr-check.sh --approve" >&2
    for id in $held_ids; do
      echo "  gh api -X POST repos/\$(gh repo view --json nameWithOwner --jq .nameWithOwner)/actions/runs/$id/approve" >&2
    done
    echo
    echo "注意：这些 run 不会出现在 \`gh pr checks\` 里，那里只给一行 no checks reported（退出码 1）。" >&2
    exit 3
  fi
  repo="$(gh repo view --json nameWithOwner --jq .nameWithOwner)"
  for id in $held_ids; do
    echo "  approving run $id ..."
    if ! gh api -X POST "repos/$repo/actions/runs/$id/approve" >/dev/null 2>&1; then
      echo "  ✗ 批准 run $id 失败：token 需要仓库 write 权限（Actions: write）。" >&2
      exit 3
    fi
  done
  echo "  已批准，等待完成（超时 ${TIMEOUT_SECONDS}s）..."
  for id in $held_ids; do
    gh run watch "$id" --interval "$POLL_SECONDS" --exit-status >/dev/null || true
  done
fi

# --- 3. tally the checks ----------------------------------------------------
wait_for_checks() {
  local deadline=$((SECONDS + TIMEOUT_SECONDS))
  while :; do
    local pending=0
    while IFS=$'\t' read -r _name state; do
      [ -z "$state" ] && continue
      case "$state" in
        PENDING | QUEUED | IN_PROGRESS) pending=$((pending + 1)) ;;
      esac
    done <<<"$(gh pr checks "$PR" --json name,state \
      --jq '.[] | "\(.name)\t\(.state)"' 2>/dev/null || true)"
    if [ "$pending" -eq 0 ] || [ "$SECONDS" -ge "$deadline" ]; then
      return 0
    fi
    sleep "$POLL_SECONDS"
  done
}

wait_for_checks

passed=0
failed=0
pending=0
failed_lines=""
while IFS=$'\t' read -r name state; do
  [ -z "$state" ] && continue
  case "$state" in
    SUCCESS | SKIPPED | NEUTRAL) passed=$((passed + 1)) ;;
    PENDING | QUEUED | IN_PROGRESS) pending=$((pending + 1)) ;;
    *)
      failed=$((failed + 1))
      failed_lines="$failed_lines
  ✗ $name  $state"
      ;;
  esac
done <<<"$(gh pr checks "$PR" --json name,state \
  --jq '.[] | "\(.name)\t\(.state)"' 2>/dev/null || true)"

if [ "$failed" -gt 0 ]; then
  echo "FAIL: $failed 个检查失败$failed_lines" >&2
  exit 1
fi

if [ "$pending" -gt 0 ]; then
  echo "WAIT: 还有 $pending 个检查在跑，等完再看。" >&2
  exit 3
fi

if [ "$passed" -eq 0 ]; then
  echo "FAIL: 这个 PR 一个检查都没有——CI 从未跑过，不能当成通过。" >&2
  echo "      先确认 build.yml 是否被触发；必要时重新 push 一次让 workflow 起来。" >&2
  exit 3
fi

echo "  checks: $passed 个全部通过 ✓"
echo "OK: 可以合并（gh pr merge $PR --squash）。"
