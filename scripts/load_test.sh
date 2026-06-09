#!/usr/bin/env bash

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
REPORT_PATH="${ROOT_DIR}/target/load-test-report.json"
SUMMARY_PATH="${ROOT_DIR}/target/load-test-summary.md"
CI_MODE=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --report)
            shift
            REPORT_PATH="${1:?missing report path}"
            shift
            ;;
        --summary)
            shift
            SUMMARY_PATH="${1:?missing summary path}"
            shift
            ;;
        --ci)
            CI_MODE=1
            shift
            ;;
        -h|--help)
            cat <<'EOF'
StrellerMinds load-test wrapper

Usage:
  ./scripts/load_test.sh [--ci] [--report <path>] [--summary <path>]

Environment:
  STRELLER_PEAK_LOAD
  STRELLER_LOAD_MULTIPLIER
  STRELLER_STUDENT_POOL
  STRELLER_COURSE_COUNT
  STRELLER_READ_MULTIPLIER
EOF
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1
            ;;
    esac
done

if [[ "${CI_MODE}" -eq 1 ]]; then
    export STRELLER_PEAK_LOAD="${STRELLER_PEAK_LOAD:-5}"
    export STRELLER_LOAD_MULTIPLIER="${STRELLER_LOAD_MULTIPLIER:-10}"
    export STRELLER_STUDENT_POOL="${STRELLER_STUDENT_POOL:-20}"
    export STRELLER_COURSE_COUNT="${STRELLER_COURSE_COUNT:-4}"
    export STRELLER_READ_MULTIPLIER="${STRELLER_READ_MULTIPLIER:-2}"
    RUNNER_ARGS=(--ci)
else
    export STRELLER_PEAK_LOAD="${STRELLER_PEAK_LOAD:-25}"
    export STRELLER_LOAD_MULTIPLIER="${STRELLER_LOAD_MULTIPLIER:-10}"
    export STRELLER_STUDENT_POOL="${STRELLER_STUDENT_POOL:-50}"
    export STRELLER_COURSE_COUNT="${STRELLER_COURSE_COUNT:-8}"
    export STRELLER_READ_MULTIPLIER="${STRELLER_READ_MULTIPLIER:-3}"
    RUNNER_ARGS=()
fi

mkdir -p "$(dirname "${REPORT_PATH}")"
mkdir -p "$(dirname "${SUMMARY_PATH}")"

cd "${ROOT_DIR}"
cargo run -p load-test-runner -- "${RUNNER_ARGS[@]}" --report "${REPORT_PATH}" --summary "${SUMMARY_PATH}"
