#!/usr/bin/env bash

# Contract coverage gate for StrellerMinds Smart Contracts.
# Targets the core contracts called out in issue #382 and fails below 95%.

set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
COVERAGE_DIR="${ROOT_DIR}/target/coverage"
REPORT_DIR="${ROOT_DIR}/target/coverage-reports"
MIN_COVERAGE="${MIN_COVERAGE:-95}"
RUN_TESTS=1
GENERATE_HTML=0
GENERATE_LCOV=0

CONTRACT_PACKAGES=(certificate progress token analytics)
PACKAGE_ARGS=()
for package in "${CONTRACT_PACKAGES[@]}"; do
    PACKAGE_ARGS+=("--package" "$package")
done

usage() {
    cat <<EOF
Usage: $0 [OPTIONS]

Options:
  --gate <N>      Required line coverage percentage (default: 95)
  --html          Generate an HTML report in target/coverage/html
  --lcov          Generate an LCOV report in target/coverage/lcov.info
  --no-test-run   Skip the explicit cargo test pass before coverage
  -h, --help      Show this help message
EOF
}

while [[ $# -gt 0 ]]; do
    case "$1" in
        --gate)
            shift
            MIN_COVERAGE="${1:?--gate requires a numeric value}"
            shift
            ;;
        --html)
            GENERATE_HTML=1
            shift
            ;;
        --lcov)
            GENERATE_LCOV=1
            shift
            ;;
        --no-test-run)
            RUN_TESTS=0
            shift
            ;;
        -h|--help)
            usage
            exit 0
            ;;
        *)
            echo "Unknown option: $1" >&2
            usage >&2
            exit 1
            ;;
    esac
done

command_exists() {
    command -v "$1" >/dev/null 2>&1
}

ensure_coverage_tooling() {
    if ! command_exists cargo-llvm-cov; then
        echo "[coverage] cargo-llvm-cov not found; installing with cargo"
        cargo install cargo-llvm-cov --locked
    fi

    if ! rustup component list --installed | grep -q "llvm-tools"; then
        echo "[coverage] llvm-tools-preview not found; installing with rustup"
        rustup component add llvm-tools-preview
    fi
}

run_contract_tests() {
    echo "[coverage] Running focused contract tests"
    for package in "${CONTRACT_PACKAGES[@]}"; do
        cargo test --package "$package" --lib
    done

    echo "[coverage] Running token property-based tests"
    cargo test --package token --lib property_tests
}

extract_line_coverage() {
    local json_file="$1"
    python3 - "$json_file" <<'PY'
import json
import sys

with open(sys.argv[1], encoding="utf-8") as handle:
    data = json.load(handle)

def find_percent(node):
    if isinstance(node, dict):
        for key in ("percent", "percentage"):
            value = node.get(key)
            if isinstance(value, (int, float)):
                return float(value)
        for key in ("lines", "line"):
            value = find_percent(node.get(key))
            if value is not None:
                return value
        for key in ("totals", "summary"):
            value = find_percent(node.get(key))
            if value is not None:
                return value
    if isinstance(node, list):
        for item in node:
            value = find_percent(item)
            if value is not None:
                return value
    return None

coverage = find_percent(data)
if coverage is None:
    raise SystemExit("could not find line coverage percentage in cargo-llvm-cov JSON")

print(f"{coverage:.2f}")
PY
}

write_markdown_report() {
    local coverage="$1"
    local status="$2"
    local report_file="${REPORT_DIR}/coverage-summary.md"

    cat > "$report_file" <<EOF
# Contract Coverage Report

- Required line coverage: ${MIN_COVERAGE}%
- Actual line coverage: ${coverage}%
- Status: ${status}
- Packages: ${CONTRACT_PACKAGES[*]}

## Test Scope

- Focused unit tests for certificate, progress, token, and analytics
- Token property-based transfer invariants
- cargo-llvm-cov JSON metrics

## Artifacts

- JSON summary: target/coverage/contract-summary.json
- HTML report: target/coverage/html/index.html (when --html is used)
- LCOV report: target/coverage/lcov.info (when --lcov is used)
EOF
}

main() {
    cd "$ROOT_DIR"
    mkdir -p "$COVERAGE_DIR" "$REPORT_DIR"

    ensure_coverage_tooling

    if [[ "$RUN_TESTS" -eq 1 ]]; then
        run_contract_tests
    fi

    echo "[coverage] Generating JSON coverage summary"
    cargo llvm-cov \
        "${PACKAGE_ARGS[@]}" \
        --all-features \
        --lib \
        --tests \
        --json \
        --summary-only \
        --output-path "${COVERAGE_DIR}/contract-summary.json"

    if [[ "$GENERATE_HTML" -eq 1 ]]; then
        echo "[coverage] Generating HTML coverage report"
        cargo llvm-cov \
            "${PACKAGE_ARGS[@]}" \
            --all-features \
            --lib \
            --tests \
            --html \
            --output-dir "${COVERAGE_DIR}/html"
    fi

    if [[ "$GENERATE_LCOV" -eq 1 ]]; then
        echo "[coverage] Generating LCOV coverage report"
        cargo llvm-cov \
            "${PACKAGE_ARGS[@]}" \
            --all-features \
            --lib \
            --tests \
            --lcov \
            --output-path "${COVERAGE_DIR}/lcov.info"
    fi

    line_coverage="$(extract_line_coverage "${COVERAGE_DIR}/contract-summary.json")"
    gate_passed="$(awk "BEGIN { print (${line_coverage} >= ${MIN_COVERAGE}) ? \"yes\" : \"no\" }")"

    echo "[coverage] Required line coverage: ${MIN_COVERAGE}%"
    echo "[coverage] Actual line coverage:   ${line_coverage}%"

    if [[ "$gate_passed" == "yes" ]]; then
        write_markdown_report "$line_coverage" "PASS"
        echo "[coverage] Coverage gate PASSED"
    else
        write_markdown_report "$line_coverage" "FAIL"
        echo "[coverage] Coverage gate FAILED (${line_coverage}% < ${MIN_COVERAGE}%)"
        exit 1
    fi
}

main
