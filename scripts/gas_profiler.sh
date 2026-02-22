#!/usr/bin/env bash
# scripts/gas_profiler.sh
set -euo pipefail

NETWORK="${NETWORK:-local}"
CONTRACT_FILTER="${CONTRACT_FILTER:-all}"
COMPARE_MODE=false
OUTPUT_DIR="target/gas-reports"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
SOROBAN="${SOROBAN_CLI:-soroban}"

FEE_PER_READ_ENTRY=6250
FEE_PER_WRITE_ENTRY=10000
FEE_PER_READ_BYTE=1750
FEE_PER_WRITE_BYTE=11800
STROOPS_PER_XLM=10000000

log() { echo "[gas-profiler] $*" >&2; }
sep() { echo "────────────────────────────────────────" >&2; }

while [[ $# -gt 0 ]]; do
    case "$1" in
        --network)  NETWORK="$2";         shift 2 ;;
        --contract) CONTRACT_FILTER="$2"; shift 2 ;;
        --compare)  COMPARE_MODE=true;    shift   ;;
        *) echo "Unknown: $1"; exit 1 ;;
    esac
done

mkdir -p "$OUTPUT_DIR"

estimate_fee() {
    local r="$1" w="$2" rb="$3" wb="$4"
    echo $(( r * FEE_PER_READ_ENTRY + w * FEE_PER_WRITE_ENTRY + rb * FEE_PER_READ_BYTE / 1024 + wb * FEE_PER_WRITE_BYTE / 1024 ))
}

build_contracts() {
    log "Building contracts..."
    cargo build --release --target wasm32-unknown-unknown 2>&1 | tail -5
    log "Build complete."
}

compare_reports() {
    command -v jq >/dev/null 2>&1 || { echo "jq required"; exit 1; }
    local reports=("${OUTPUT_DIR}"/gas_report_*.json)
    [[ ${#reports[@]} -lt 2 ]] && { echo "Need 2 reports to compare"; exit 1; }
    local before="${reports[-2]}" after="${reports[-1]}"
    log "Before: ${before} | After: ${after}"
    sep
    printf "%-40s %-15s %-15s %-10s\n" "Function" "Before" "After" "Savings"
    sep
}

main() {
    log "StrellerMinds Gas Profiler v1.0"
    sep
    if $COMPARE_MODE; then compare_reports; exit 0; fi
    build_contracts
    log "Done. Check ${OUTPUT_DIR} for reports."
    log "Run with --compare after optimizing to see savings."
}

main "$@"
