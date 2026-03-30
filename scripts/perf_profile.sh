#!/usr/bin/env bash
# =============================================================================
# perf_profile.sh – Performance profiling for StrellerMinds Smart Contracts
#
# Issue #271 – Missing Performance Optimization
#
# Runs performance-focused tests and outputs a timing report.
#
# Usage:
#   ./scripts/perf_profile.sh [--report <path>] [--baseline] [--compare <path>]
#
#   --report <f>    Write JSON timing report to <f>
#                   (default: target/perf_report.json)
#   --baseline      Save current results as the new baseline
#   --compare <f>   Compare current run against baseline at <f>
#   --verbose       Print cargo test output
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

REPORT_FILE="${ROOT_DIR}/target/perf_report.json"
BASELINE_FLAG=0
COMPARE_FILE=""
VERBOSE=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --report)   shift; REPORT_FILE="${1:?}"; shift ;;
        --baseline) BASELINE_FLAG=1; shift ;;
        --compare)  shift; COMPARE_FILE="${1:?}"; shift ;;
        --verbose)  VERBOSE=1; shift ;;
        -h|--help)
            sed -n '2,/^set /p' "$0" | grep '^#' | sed 's/^# \{0,1\}//'
            exit 0 ;;
        *) echo "Unknown option: $1" >&2; exit 1 ;;
    esac
done

cd "${ROOT_DIR}"
mkdir -p "$(dirname "${REPORT_FILE}")"

run_test_timing() {   # run_test_timing <label> <cargo_args...>
    local label="$1"; shift
    local start end elapsed_ms test_count
    start=$(date +%s%N 2>/dev/null || date +%s)
    local output
    output=$(cargo test "$@" -- --test-threads=1 2>&1) || true
    end=$(date +%s%N 2>/dev/null || date +%s)

    # milliseconds (handle systems without %N)
    if [[ "${#start}" -gt 10 ]]; then
        elapsed_ms=$(( (end - start) / 1000000 ))
    else
        elapsed_ms=$(( (end - start) * 1000 ))
    fi

    test_count=$(echo "${output}" | grep -oP 'test result: ok\. \K[0-9]+' | tail -1 || echo "0")
    [[ "${VERBOSE}" -eq 1 ]] && echo "${output}"

    echo "  ${label}: ${elapsed_ms}ms (${test_count} tests)"
    echo "  {\"suite\":\"${label}\",\"elapsed_ms\":${elapsed_ms},\"tests\":${test_count}}"
}

echo "========================================="
echo " StrellerMinds – Performance Profile"
echo "========================================="
echo ""

TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || echo "unknown")
declare -a SUITE_RESULTS=()

# Helper that adds the JSON line to SUITE_RESULTS array
profile_suite() {
    local label="$1"; shift
    local json_line
    json_line=$(run_test_timing "${label}" "$@" | grep '^\s*{')
    SUITE_RESULTS+=("${json_line}")
}

echo "[perf] Profiling: shared (gas optimizer + performance tests)"
profile_suite "shared-gas-optimizer" --package shared --lib -- performance_tests 2>/dev/null || \
profile_suite "shared-gas-optimizer" --package shared --lib

echo "[perf] Profiling: progress contract"
profile_suite "progress-contract"    --package progress   --lib

echo "[perf] Profiling: security-monitor contract"
profile_suite "security-monitor"     --package security-monitor --lib

echo "[perf] Profiling: token contract"
profile_suite "token-contract"       --package token      --lib 2>/dev/null || true

echo "[perf] Profiling: certificate contract"
profile_suite "certificate-contract" --package certificate --lib 2>/dev/null || true

# ── Write JSON report ─────────────────────────────────────────────────────────
SUITES_JSON=$(printf '%s,' "${SUITE_RESULTS[@]}")
SUITES_JSON="[${SUITES_JSON%,}]"

cat > "${REPORT_FILE}" <<EOF
{
  "timestamp": "${TIMESTAMP}",
  "suites": ${SUITES_JSON}
}
EOF
echo ""
echo "[perf] Report written to: ${REPORT_FILE}"

# ── Baseline / comparison ─────────────────────────────────────────────────────
if [[ "${BASELINE_FLAG}" -eq 1 ]]; then
    cp "${REPORT_FILE}" "${ROOT_DIR}/target/perf_baseline.json"
    echo "[perf] Baseline saved to: target/perf_baseline.json"
fi

if [[ -n "${COMPARE_FILE}" ]]; then
    echo ""
    echo "[perf] Comparing against baseline: ${COMPARE_FILE}"
    if command -v python3 &>/dev/null; then
        python3 - "${COMPARE_FILE}" "${REPORT_FILE}" <<'PYEOF'
import json, sys
baseline = json.load(open(sys.argv[1]))
current  = json.load(open(sys.argv[2]))

b_map = {s["suite"]: s for s in baseline.get("suites", [])}
c_map = {s["suite"]: s for s in current.get("suites", [])}

regressions = 0
for suite, c in c_map.items():
    b = b_map.get(suite)
    if not b:
        print(f"  [NEW]  {suite}: {c['elapsed_ms']}ms")
        continue
    delta_pct = ((c["elapsed_ms"] - b["elapsed_ms"]) / max(b["elapsed_ms"], 1)) * 100
    flag = "⚠️ REGRESSION" if delta_pct > 20 else ("✅ ok" if delta_pct <= 5 else "⚠️  slower")
    print(f"  {flag}  {suite}: {b['elapsed_ms']}ms → {c['elapsed_ms']}ms  ({delta_pct:+.1f}%)")
    if delta_pct > 20:
        regressions += 1

if regressions:
    print(f"\n[perf] ❌ {regressions} regression(s) detected (>20% slower)")
    sys.exit(1)
else:
    print("\n[perf] ✅ No regressions detected")
PYEOF
    else
        echo "[WARN] python3 not available – skipping comparison"
    fi
fi

echo ""
echo "[perf] ✅ Profiling complete"
