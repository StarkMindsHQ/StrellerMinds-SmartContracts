#!/usr/bin/env bash
# =============================================================================
# security_scan.sh – Automated security scanning for StrellerMinds Smart Contracts
#
# Issue #273 – Missing Security Testing
#
# This script runs a layered security audit:
#   1. cargo audit    – known CVE / advisory database scan
#   2. cargo clippy   – lint-based security checks with extra deny flags
#   3. cargo test     – run the security test suite
#   4. semgrep        – pattern-based static analysis (optional)
#
# Usage:
#   ./scripts/security_scan.sh [--full] [--fix] [--report <path>]
#
#   --full         Also run semgrep (requires semgrep in PATH)
#   --fix          Pass --fix to clippy (CAUTION: modifies source files)
#   --report <f>   Write a JSON summary to <f>  (default: target/security_scan.json)
#   --fail-on-warn Exit non-zero if any warnings are found (CI strict mode)
# =============================================================================
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT_DIR="$(cd "${SCRIPT_DIR}/.." && pwd)"

# ── Defaults ──────────────────────────────────────────────────────────────────
FULL_SCAN=0
FIX_MODE=0
REPORT_FILE="${ROOT_DIR}/target/security_scan.json"
FAIL_ON_WARN=0

PASS_COUNT=0
FAIL_COUNT=0
WARN_COUNT=0
declare -a RESULTS=()

# ── Argument parsing ──────────────────────────────────────────────────────────
while [[ $# -gt 0 ]]; do
    case "$1" in
        --full)         FULL_SCAN=1; shift ;;
        --fix)          FIX_MODE=1; shift ;;
        --fail-on-warn) FAIL_ON_WARN=1; shift ;;
        --report)       shift; REPORT_FILE="${1:?--report requires a path}"; shift ;;
        -h|--help)
            sed -n '2,/^set /p' "$0" | grep '^#' | sed 's/^# \{0,1\}//'
            exit 0 ;;
        *)
            echo "Unknown option: $1" >&2; exit 1 ;;
    esac
done

cd "${ROOT_DIR}"
mkdir -p "$(dirname "${REPORT_FILE}")"

record() {   # record <check> <status> <detail>
    local check="$1" status="$2" detail="$3"
    RESULTS+=("{\"check\":\"${check}\",\"status\":\"${status}\",\"detail\":\"${detail//\"/\\\"}\"}")
    case "${status}" in
        PASS) PASS_COUNT=$((PASS_COUNT + 1)) ;;
        WARN) WARN_COUNT=$((WARN_COUNT + 1)) ;;
        FAIL) FAIL_COUNT=$((FAIL_COUNT + 1)) ;;
    esac
}

section() { echo ""; echo "══════════════════════════════════════════"; echo "  $1"; echo "══════════════════════════════════════════"; }

# ── 1. cargo audit ────────────────────────────────────────────────────────────
section "1/4  Advisory Database Scan (cargo audit)"
if command -v cargo-audit &>/dev/null || cargo audit --version &>/dev/null 2>&1; then
    if cargo audit 2>&1 | tee /tmp/audit_out.txt; then
        record "cargo-audit" "PASS" "No known vulnerabilities found"
    else
        VULN_COUNT=$(grep -c "Vulnerability found" /tmp/audit_out.txt 2>/dev/null || echo "?")
        record "cargo-audit" "FAIL" "${VULN_COUNT} vulnerable dependencies detected"
    fi
else
    echo "[WARN] cargo-audit not installed. Skipping. Install: cargo install cargo-audit"
    record "cargo-audit" "WARN" "cargo-audit not installed"
fi

# ── 2. clippy (security lints) ────────────────────────────────────────────────
section "2/4  Static Lint Analysis (clippy)"
CLIPPY_ARGS=(
    --workspace
    --exclude e2e-tests
    --all-targets
    --all-features
    --
    -W clippy::all
    -W clippy::pedantic
    -W clippy::cargo
    -D clippy::mem-forget
    -D clippy::unwrap-used
    -D clippy::expect-used
    -D clippy::panic
    -D clippy::exhaustive-enums
    -A clippy::module_name_repetitions
    -A clippy::missing_errors_doc
    -A clippy::missing_panics_doc
    -A clippy::must_use_candidate
)
FIX_FLAG=""
if [[ "${FIX_MODE}" -eq 1 ]]; then FIX_FLAG="--fix"; fi

if cargo clippy ${FIX_FLAG} "${CLIPPY_ARGS[@]}" 2>&1 | tee /tmp/clippy_out.txt; then
    WARN_LINES=$(grep -c "^warning" /tmp/clippy_out.txt 2>/dev/null || echo "0")
    if [[ "${WARN_LINES}" -gt 0 ]]; then
        record "clippy" "WARN" "${WARN_LINES} lint warning(s) found"
    else
        record "clippy" "PASS" "No lint issues found"
    fi
else
    record "clippy" "FAIL" "Clippy found deny-level lint violations"
fi

# ── 3. Security unit tests ────────────────────────────────────────────────────
section "3/4  Security Test Suite"
if cargo test \
    --package security-monitor \
    --lib \
    -- \
    --test-threads=4 \
    2>&1 | tee /tmp/sec_test_out.txt; then
    TEST_COUNT=$(grep -oP 'test result: ok\. \K[0-9]+' /tmp/sec_test_out.txt | tail -1 || echo "?")
    record "security-tests" "PASS" "${TEST_COUNT} security tests passed"
else
    FAILED=$(grep -oP '\K[0-9]+ failed' /tmp/sec_test_out.txt | tail -1 || echo "unknown")
    record "security-tests" "FAIL" "Security tests failed: ${FAILED}"
fi

# ── 4. Semgrep (optional) ─────────────────────────────────────────────────────
section "4/4  Pattern-Based Static Analysis (semgrep)"
if [[ "${FULL_SCAN}" -eq 1 ]]; then
    if command -v semgrep &>/dev/null; then
        SEMGREP_RULES="p/rust p/secrets p/command-injection"
        if semgrep --config "${SEMGREP_RULES}" \
                   --include "*.rs" \
                   --error \
                   contracts/ \
                   2>&1 | tee /tmp/semgrep_out.txt; then
            record "semgrep" "PASS" "No semgrep findings"
        else
            FINDING_COUNT=$(grep -c "^>" /tmp/semgrep_out.txt 2>/dev/null || echo "?")
            record "semgrep" "WARN" "${FINDING_COUNT} semgrep finding(s)"
        fi
    else
        echo "[WARN] semgrep not installed. Install: pip install semgrep"
        record "semgrep" "WARN" "semgrep not installed – skipped"
    fi
else
    echo "[SKIP] semgrep skipped (use --full to enable)"
    record "semgrep" "WARN" "skipped (pass --full to enable)"
fi

# ── Write JSON report ─────────────────────────────────────────────────────────
RESULTS_JSON=$(printf '%s,' "${RESULTS[@]}")
RESULTS_JSON="[${RESULTS_JSON%,}]"
TIMESTAMP=$(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || echo "unknown")

cat > "${REPORT_FILE}" <<EOF
{
  "timestamp": "${TIMESTAMP}",
  "summary": {
    "passed": ${PASS_COUNT},
    "warnings": ${WARN_COUNT},
    "failed": ${FAIL_COUNT}
  },
  "checks": ${RESULTS_JSON}
}
EOF
echo ""
echo "[scan] Report written to: ${REPORT_FILE}"

# ── Final summary ─────────────────────────────────────────────────────────────
echo ""
echo "==========================================="
echo " Security Scan Summary"
echo "  ✅ Passed:   ${PASS_COUNT}"
echo "  ⚠️  Warnings: ${WARN_COUNT}"
echo "  ❌ Failed:   ${FAIL_COUNT}"
echo "==========================================="

if [[ "${FAIL_COUNT}" -gt 0 ]]; then
    echo "[scan] ❌ SCAN FAILED"
    exit 1
elif [[ "${FAIL_ON_WARN}" -eq 1 && "${WARN_COUNT}" -gt 0 ]]; then
    echo "[scan] ❌ SCAN FAILED (warnings treated as errors)"
    exit 1
else
    echo "[scan] ✅ SCAN PASSED"
    exit 0
fi
