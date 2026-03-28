#!/usr/bin/env bash
# =============================================================================
# coverage.sh – Code coverage reporting for StrellerMinds Smart Contracts
#
# Issue #274 – Inadequate Code Coverage
#
# Prerequisites:
#   cargo install cargo-llvm-cov
#   rustup component add llvm-tools-preview
#
# Usage:
#   ./scripts/coverage.sh [--html] [--lcov] [--json] [--open] [--gate <min_%>]
#
#   --html          Generate HTML report in target/coverage/html/
#   --lcov          Generate lcov.info for CI integration
#   --json          Generate JSON summary (default when no format given)
#   --open          Open HTML report in browser after generation
#   --gate <N>      Fail with exit-code 1 if line coverage < N  (default: 90)
#   --workspace     Include all workspace members (default: exclude e2e-tests)
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

# ── Defaults ──────────────────────────────────────────────────────────────────
FORMAT_HTML=0
FORMAT_LCOV=0
FORMAT_JSON=1          # always emit JSON summary
OPEN_BROWSER=0
COVERAGE_GATE=90       # minimum acceptable line-coverage percentage
WORKSPACE_ALL=0

# ── Argument parsing ──────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case "$1" in
        --html)        FORMAT_HTML=1; shift ;;
        --lcov)        FORMAT_LCOV=1; shift ;;
        --json)        FORMAT_JSON=1; shift ;;
        --open)        OPEN_BROWSER=1; shift ;;
        --workspace)   WORKSPACE_ALL=1; shift ;;
        --gate)
            shift
            COVERAGE_GATE="${1:?--gate requires a number argument}"
            shift ;;
        -h|--help)
            sed -n '2,/^set /p' "$0" | grep '^#' | sed 's/^# \{0,1\}//'
            exit 0 ;;
        *)
            echo "Unknown option: $1" >&2
            exit 1 ;;
    esac
done

# ── Dependency check ──────────────────────────────────────────────────────────
if ! command -v cargo-llvm-cov &>/dev/null; then
    echo "cargo-llvm-cov not found. Installing..."
    cargo install cargo-llvm-cov --quiet
fi

if ! rustup component list --installed | grep -q llvm-tools; then
    echo "llvm-tools-preview not found. Installing..."
    rustup component add llvm-tools-preview
fi

# ── Build coverage command ────────────────────────────────────────────────────
cd "${ROOT_DIR}"

COVERAGE_DIR="target/coverage"
mkdir -p "${COVERAGE_DIR}"

EXCL_ARGS=(
    --exclude e2e-tests
    --exclude streller-cli
    --exclude streller-cli-enhanced
)
if [[ "${WORKSPACE_ALL}" -eq 0 ]]; then
    EXCL_ARGS+=(--exclude-from-report "e2e")
fi

echo "========================================="
echo " StrellerMinds – Code Coverage Report"
echo "========================================="
echo ""

# ── JSON summary (always) ─────────────────────────────────────────────────────
echo "[coverage] Generating JSON summary..."
cargo llvm-cov \
    --workspace \
    "${EXCL_ARGS[@]}" \
    --json \
    --output-path "${COVERAGE_DIR}/coverage.json" \
    2>/dev/null

# Extract line coverage percentage from JSON
LINE_PCT=$(
    python3 -c "
import json, sys
data = json.load(open('${COVERAGE_DIR}/coverage.json'))
totals = data.get('data', [{}])[0].get('totals', {})
lines = totals.get('lines', {})
pct = lines.get('percent', 0)
print(f'{pct:.1f}')
" 2>/dev/null || echo "0.0"
)
echo "[coverage] Line coverage: ${LINE_PCT}%"

# ── LCOV ─────────────────────────────────────────────────────────────────────
if [[ "${FORMAT_LCOV}" -eq 1 ]]; then
    echo "[coverage] Generating lcov.info..."
    cargo llvm-cov \
        --workspace \
        "${EXCL_ARGS[@]}" \
        --lcov \
        --output-path "${COVERAGE_DIR}/lcov.info" \
        2>/dev/null
    echo "[coverage] lcov report: ${COVERAGE_DIR}/lcov.info"
fi

# ── HTML ─────────────────────────────────────────────────────────────────────
if [[ "${FORMAT_HTML}" -eq 1 ]]; then
    echo "[coverage] Generating HTML report..."
    cargo llvm-cov \
        --workspace \
        "${EXCL_ARGS[@]}" \
        --html \
        --output-dir "${COVERAGE_DIR}/html" \
        2>/dev/null
    echo "[coverage] HTML report: ${COVERAGE_DIR}/html/index.html"

    if [[ "${OPEN_BROWSER}" -eq 1 ]]; then
        if command -v xdg-open &>/dev/null; then
            xdg-open "${COVERAGE_DIR}/html/index.html"
        elif command -v open &>/dev/null; then
            open "${COVERAGE_DIR}/html/index.html"
        fi
    fi
fi

# ── Coverage gate ─────────────────────────────────────────────────────────────
echo ""
echo "==========================================="
echo " Coverage Gate: ${COVERAGE_GATE}%"
echo " Actual:        ${LINE_PCT}%"
echo "==========================================="

# Compare using awk for float comparison
GATE_PASSED=$(awk "BEGIN { print (${LINE_PCT} >= ${COVERAGE_GATE}) ? \"yes\" : \"no\" }")

if [[ "${GATE_PASSED}" == "yes" ]]; then
    echo "[coverage] ✅ Coverage gate PASSED"
    exit 0
else
    echo "[coverage] ❌ Coverage gate FAILED (${LINE_PCT}% < ${COVERAGE_GATE}%)"
    echo ""
    echo "To see which lines are uncovered, run:"
    echo "  ./scripts/coverage.sh --html --open"
    exit 1
fi
