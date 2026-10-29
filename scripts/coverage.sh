#!/usr/bin/env bash
# Measure line coverage and enforce the documented floor.
#
# Usage:
#   ./scripts/coverage.sh              # measure, report, enforce floor
#   ./scripts/coverage.sh --report     # reuse the last run, no re-execution
#   ./scripts/coverage.sh --html       # also write target/coverage/html
#
# The floor exists so the verifiable part of the codebase cannot silently rot.
# It is deliberately below the measured value: raise it when coverage rises,
# never lower it to make a red build green.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

FLOOR="${COVERAGE_FLOOR:-74}"

if ! command -v cargo-llvm-cov >/dev/null 2>&1; then
  echo "cargo-llvm-cov is required: cargo install cargo-llvm-cov --locked" >&2
  exit 1
fi

mode="run"
want_html=0
for arg in "$@"; do
  case "$arg" in
    --report) mode="report" ;;
    --html) want_html=1 ;;
    *) echo "unknown argument: $arg" >&2; exit 2 ;;
  esac
done

if [[ "$mode" == "run" ]]; then
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
