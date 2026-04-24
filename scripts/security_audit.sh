#!/usr/bin/env bash
# =============================================================================
# Security Audit Runner for StrellerMinds Smart Contracts
#
# Runs all security-related checks:
#   1. cargo audit   — known CVE scan on dependencies
#   2. cargo clippy  — static analysis (deny warnings)
#   3. cargo test    — unit + security tests across all contracts
#   4. cargo deny    — license and dependency policy enforcement
#
# Exit codes:
#   0  All checks passed
#   1  One or more checks failed
# =============================================================================

set -euo pipefail

BOLD="\033[1m"
RED="\033[0;31m"
GREEN="\033[0;32m"
YELLOW="\033[0;33m"
RESET="\033[0m"

PASS=0
FAIL=0
RESULTS=()

run_check() {
    local name="$1"
    shift
    echo -e "\n${BOLD}[CHECK] ${name}${RESET}"
    if "$@"; then
        echo -e "${GREEN}[PASS] ${name}${RESET}"
        RESULTS+=("PASS: ${name}")
        ((PASS++)) || true
    else
        echo -e "${RED}[FAIL] ${name}${RESET}"
        RESULTS+=("FAIL: ${name}")
        ((FAIL++)) || true
    fi
}

# ─────────────────────────────────────────────────────────────────────────────
# 1. Dependency CVE Scan
# ─────────────────────────────────────────────────────────────────────────────
run_check "Dependency CVE Scan (cargo audit)" \
    cargo audit --deny warnings

# ─────────────────────────────────────────────────────────────────────────────
# 2. Static Analysis
# ─────────────────────────────────────────────────────────────────────────────
run_check "Static Analysis (cargo clippy)" \
    cargo clippy --all-targets --all-features -- \
        -D warnings \
        -D clippy::unwrap_used \
        -D clippy::expect_used \
        -D clippy::panic \
        -D clippy::integer_arithmetic \
        -A clippy::too_many_arguments

# ─────────────────────────────────────────────────────────────────────────────
# 3. Security Unit Tests — shared validation & access control
# ─────────────────────────────────────────────────────────────────────────────
run_check "Security Tests: shared/validation" \
    cargo test -p shared --lib -- validation::tests --nocapture

# ─────────────────────────────────────────────────────────────────────────────
# 4. Security Tests — reentrancy guards
# ─────────────────────────────────────────────────────────────────────────────
run_check "Security Tests: token reentrancy" \
    cargo test -p token --lib -- reentrancy --nocapture

# ─────────────────────────────────────────────────────────────────────────────
# 5. Security Tests — access control / RBAC
# ─────────────────────────────────────────────────────────────────────────────
run_check "Security Tests: certificate authorization" \
    cargo test -p certificate --lib -- test --nocapture

# ─────────────────────────────────────────────────────────────────────────────
# 6. Full Test Suite
# ─────────────────────────────────────────────────────────────────────────────
run_check "Full Test Suite (all contracts)" \
    cargo test --workspace --lib

# ─────────────────────────────────────────────────────────────────────────────
# 7. Dependency Policy (cargo deny)
# ─────────────────────────────────────────────────────────────────────────────
if command -v cargo-deny &>/dev/null; then
    run_check "Dependency Policy (cargo deny)" \
        cargo deny check
else
    echo -e "${YELLOW}[SKIP] cargo-deny not installed — run: cargo install cargo-deny${RESET}"
    RESULTS+=("SKIP: Dependency Policy (cargo deny)")
fi

# ─────────────────────────────────────────────────────────────────────────────
# Summary
# ─────────────────────────────────────────────────────────────────────────────
echo -e "\n${BOLD}═══════════════════════════════════════════════════════${RESET}"
echo -e "${BOLD}Security Audit Summary${RESET}"
echo -e "${BOLD}═══════════════════════════════════════════════════════${RESET}"
for result in "${RESULTS[@]}"; do
    if [[ "$result" == PASS* ]]; then
        echo -e "  ${GREEN}✓ ${result}${RESET}"
    elif [[ "$result" == FAIL* ]]; then
        echo -e "  ${RED}✗ ${result}${RESET}"
    else
        echo -e "  ${YELLOW}⊘ ${result}${RESET}"
    fi
done
echo -e "${BOLD}───────────────────────────────────────────────────────${RESET}"
echo -e "  Passed: ${GREEN}${PASS}${RESET}  Failed: ${RED}${FAIL}${RESET}"
echo -e "${BOLD}═══════════════════════════════════════════════════════${RESET}\n"

if [[ $FAIL -gt 0 ]]; then
    exit 1
fi
exit 0
