#!/usr/bin/env bash
# Measure line coverage and enforce the documented floor.
#
# Usage:
#   ./scripts/coverage.sh                  # clean, measure, report, enforce floor
#   ./scripts/coverage.sh --report         # reuse the last run, no re-execution
#   ./scripts/coverage.sh --html           # also write target/llvm-cov/html
#   ./scripts/coverage.sh --keep-artifacts # skip the clean step (faster, may over-count)
#
# The floor exists so the verifiable part of the codebase cannot silently rot.
# It is deliberately below the measured value: raise it when coverage rises,
# never lower it to make a red build green.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Measured 2026-11-22 with a clean workspace: 79.79% lines on macOS, 79.68% on
# the Ubuntu CI runner (the same day's earlier 80.52% came from stale profdata
# left behind by a file split - see the clean step below).
# The floor trails the CI number by ~1.7 points so `cfg(target_os)` divergence
# between the two platforms cannot turn a real pass into a flaky red, while still
# catching a real slide. History: 74.2% measured / floor 73 when the gate landed.
# Raise it as coverage grows; never lower it to make a red build green.
FLOOR="${COVERAGE_FLOOR:-78}"

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "cargo-llvm-cov is required: cargo install cargo-llvm-cov --locked" >&2
  exit 1
fi

mode="run"
want_html=0
keep_artifacts=0
for arg in "$@"; do
  case "$arg" in
    --report) mode="report" ;;
    --html) want_html=1 ;;
    --keep-artifacts) keep_artifacts=1 ;;
    *) echo "unknown argument: $arg" >&2; exit 2 ;;
  esac
done

if [[ "$mode" == "run" ]]; then
  # 先清掉上一次的插桩产物。llvm-cov 的报告会合并 target/ 下所有 .profraw，
  # 源文件被改名/拆分成多个模块后，旧文件对应的插桩产物仍然躺在 target 里，
  # 会被合并进报告——同一份代码测出两个不同的总行数（2026-11-22 实测：
  # 拆完 cover 模块后不清是 41791 行 / 80.49%，清了是 25616 行 / 79.79%）。
  # 虚高只会掩盖滑落，所以默认清，`--keep-artifacts` 用于快速迭代时省掉重编。
  if [[ "$keep_artifacts" != "1" ]]; then
    cargo llvm-cov clean --workspace
  fi
  if command -v cargo-nextest >/dev/null 2>&1; then
    cargo llvm-cov nextest --no-report
  else
    cargo llvm-cov --no-report --no-fail-fast
  fi
fi

echo
echo "== Per-file line coverage =="
cargo llvm-cov report --summary-only

if [[ "$want_html" == "1" ]]; then
  cargo llvm-cov report --html
  echo
  echo "HTML report: target/llvm-cov/html/index.html"
fi

echo
cargo llvm-cov report --summary-only --fail-under-lines "$FLOOR" >/dev/null
echo "Coverage floor OK (lines >= ${FLOOR}%)."
