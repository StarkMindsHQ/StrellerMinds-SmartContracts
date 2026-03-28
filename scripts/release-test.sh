#!/bin/bash

# Release Testing Framework for Soroban Smart Contracts
# This script performs comprehensive testing before a release

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOG_DIR="$PROJECT_ROOT/logs"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
LOG_FILE="$LOG_DIR/release_test_$TIMESTAMP.log"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

# Test counters
TOTAL_TESTS=0
PASSED_TESTS=0
FAILED_TESTS=0
SKIPPED_TESTS=0

# Create log directory
mkdir -p "$LOG_DIR"

# Logging functions
log() {
    echo "[$(date '+%Y-%m-%d %H:%M:%S')] $1" | tee -a "$LOG_FILE"
}

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1" | tee -a "$LOG_FILE"
}

log_success() {
    echo -e "${GREEN}[PASS]${NC} $1" | tee -a "$LOG_FILE"
}

log_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1" | tee -a "$LOG_FILE"
}

log_error() {
    echo -e "${RED}[FAIL]${NC} $1" | tee -a "$LOG_FILE"
}

log_step() {
    echo -e "${PURPLE}[STEP]${NC} $1" | tee -a "$LOG_FILE"
}

# Test result tracking
record_test() {
    local test_name="$1"
    local result="$2"
    local message="${3:-}"
    
    TOTAL_TESTS=$((TOTAL_TESTS + 1))
    
    if [ "$result" = "PASS" ]; then
        PASSED_TESTS=$((PASSED_TESTS + 1))
        log_success "$test_name - $message"
    elif [ "$result" = "SKIP" ]; then
        SKIPPED_TESTS=$((SKIPPED_TESTS + 1))
        log_warning "$test_name - SKIPPED: $message"
    else
        FAILED_TESTS=$((FAILED_TESTS + 1))
        log_error "$test_name - $message"
    fi
}

# Show help
show_help() {
    cat << EOF
Release Testing Framework

Usage: $0 [OPTIONS] [TEST_SUITE]

OPTIONS:
    -h, --help          Show this help message
    -v, --verbose       Enable verbose output
    -q, --quick         Run quick tests only (smoke tests)
    -c, --critical      Run critical tests only
    --skip-build        Skip build tests
    --skip-unit         Skip unit tests
    --skip-e2e          Skip E2E tests
    --skip-security     Skip security tests

TEST_SUITES:
    all                 Run all tests (default)
    build               Run build and compilation tests
    unit                Run unit tests
    e2e                 Run end-to-end tests
    security            Run security audits
    smoke               Run smoke tests
    performance         Run performance tests

EXAMPLES:
    $0                  # Run all tests
    $0 --quick          # Run smoke tests only
    $0 build            # Run only build tests
    $0 --skip-e2e       # Skip E2E tests

EXIT CODES:
    0   All tests passed
    1   One or more tests failed
    2   Critical test failure
EOF
}

# Parse arguments
VERBOSE=false
QUICK=false
CRITICAL=false
SKIP_BUILD=false
SKIP_UNIT=false
SKIP_E2E=false
SKIP_SECURITY=false
TEST_SUITE="all"

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -q|--quick)
            QUICK=true
            shift
            ;;
        -c|--critical)
            CRITICAL=true
            shift
            ;;
        --skip-build)
            SKIP_BUILD=true
            shift
            ;;
        --skip-unit)
            SKIP_UNIT=true
            shift
            ;;
        --skip-e2e)
            SKIP_E2E=true
            shift
            ;;
        --skip-security)
            SKIP_SECURITY=true
            shift
            ;;
        build|unit|e2e|security|smoke|performance)
            TEST_SUITE="$1"
            shift
            ;;
        *)
            log_error "Unknown option: $1"
            show_help
            exit 1
            ;;
    esac
done

# Test functions

test_prerequisites() {
    log_step "Testing prerequisites..."
    
    # Check Rust toolchain
    if command -v rustc &> /dev/null; then
        record_test "rust_installed" "PASS" "Rust is installed"
    else
        record_test "rust_installed" "FAIL" "Rust is not installed"
        return 2
    fi
    
    # Check Cargo
    if command -v cargo &> /dev/null; then
        record_test "cargo_installed" "PASS" "Cargo is installed"
    else
        record_test "cargo_installed" "FAIL" "Cargo is not installed"
        return 2
    fi
    
    # Check Soroban CLI
    if command -v soroban &> /dev/null; then
        record_test "soroban_installed" "PASS" "Soroban CLI is installed"
    else
        record_test "soroban_installed" "SKIP" "Soroban CLI not found (optional)"
    fi
    
    # Check wasm-opt
    if command -v wasm-opt &> /dev/null; then
        record_test "wasm_opt_installed" "PASS" "wasm-opt is installed"
    else
        record_test "wasm_opt_installed" "SKIP" "wasm-opt not found (optional)"
    fi
}

test_build() {
    if [ "$SKIP_BUILD" = true ]; then
        log_info "Skipping build tests"
        return 0
    fi
    
    log_step "Running build tests..."
    cd "$PROJECT_ROOT"
    
    # Test clean build
    log_info "Testing clean build..."
    if cargo check --workspace --target wasm32-unknown-unknown --release 2>&1 | tee -a "$LOG_FILE"; then
        record_test "clean_build" "PASS" "Workspace builds successfully"
    else
        record_test "clean_build" "FAIL" "Workspace build failed"
        return 1
    fi
    
    # Test individual contracts
    log_info "Testing individual contract builds..."
    local contract_failures=0
    for contract_dir in contracts/*/; do
        contract_name=$(basename "$contract_dir")
        if [ -f "$contract_dir/Cargo.toml" ]; then
            if cargo check --target wasm32-unknown-unknown --release -p "$contract_name" 2>&1 | tee -a "$LOG_FILE"; then
                record_test "build_$contract_name" "PASS" "$contract_name builds"
            else
                record_test "build_$contract_name" "FAIL" "$contract_name build failed"
                contract_failures=$((contract_failures + 1))
            fi
        fi
    done
    
    if [ $contract_failures -gt 0 ]; then
        return 1
    fi
}

test_unit_tests() {
    if [ "$SKIP_UNIT" = true ]; then
        log_info "Skipping unit tests"
        return 0
    fi
    
    log_step "Running unit tests..."
    cd "$PROJECT_ROOT"
    
    # Run workspace tests
    if cargo test --workspace --exclude e2e-tests --lib 2>&1 | tee -a "$LOG_FILE"; then
        record_test "unit_tests" "PASS" "All unit tests passed"
    else
        record_test "unit_tests" "FAIL" "Unit tests failed"
        return 1
    fi
}

test_code_quality() {
    log_step "Running code quality checks..."
    cd "$PROJECT_ROOT"
    
    # Check formatting
    log_info "Checking code format..."
    if cargo fmt --all -- --check 2>&1 | tee -a "$LOG_FILE"; then
        record_test "code_format" "PASS" "Code is properly formatted"
    else
        record_test "code_format" "FAIL" "Code formatting issues detected"
        return 1
    fi
    
    # Run Clippy
    log_info "Running Clippy lints..."
    if cargo clippy --workspace --all-targets -- -D warnings 2>&1 | tee -a "$LOG_FILE"; then
        record_test "clippy_lints" "PASS" "No Clippy warnings"
    else
        record_test "clippy_lints" "FAIL" "Clippy warnings detected"
        return 1
    fi
}

test_security() {
    if [ "$SKIP_SECURITY" = true ]; then
        log_info "Skipping security tests"
        return 0
    fi
    
    log_step "Running security checks..."
    cd "$PROJECT_ROOT"
    
    # Run cargo audit
    if command -v cargo-audit &> /dev/null; then
        if cargo audit 2>&1 | tee -a "$LOG_FILE"; then
            record_test "security_audit" "PASS" "No security vulnerabilities found"
        else
            record_test "security_audit" "FAIL" "Security vulnerabilities detected"
            return 1
        fi
    else
        record_test "security_audit" "SKIP" "cargo-audit not installed"
    fi
    
    # Check for reentrancy guards
    log_info "Checking for reentrancy protection..."
    local reentrancy_count=$(grep -r "reentrant" contracts/*/src/*.rs 2>/dev/null | wc -l)
    if [ "$reentrancy_count" -gt 0 ]; then
        record_test "reentrancy_protection" "PASS" "Reentrancy guards found ($reentrancy_count instances)"
    else
        record_test "reentrancy_protection" "SKIP" "No explicit reentrancy guards found"
    fi
}

test_e2e() {
    if [ "$SKIP_E2E" = true ]; then
        log_info "Skipping E2E tests"
        return 0
    fi
    
    log_step "Running E2E tests..."
    cd "$PROJECT_ROOT"
    
    # Check if localnet is running
    if curl -s "http://localhost:8000/health" > /dev/null 2>&1; then
        if cargo test -p e2e-tests 2>&1 | tee -a "$LOG_FILE"; then
            record_test "e2e_tests" "PASS" "E2E tests passed"
        else
            record_test "e2e_tests" "FAIL" "E2E tests failed"
            return 1
        fi
    else
        record_test "e2e_tests" "SKIP" "Localnet not running"
    fi
}

test_smoke() {
    log_step "Running smoke tests..."
    cd "$PROJECT_ROOT"
    
    # Quick build check
    if cargo check --workspace --target wasm32-unknown-unknown 2>&1 | tee -a "$LOG_FILE"; then
        record_test "smoke_build" "PASS" "Quick build check passed"
    else
        record_test "smoke_build" "FAIL" "Quick build check failed"
        return 1
    fi
    
    # Check WASM files exist
    local wasm_count=$(find target/wasm32-unknown-unknown/release -name "*.wasm" 2>/dev/null | wc -l)
    if [ "$wasm_count" -gt 0 ]; then
        record_test "wasm_artifacts" "PASS" "WASM artifacts found ($wasm_count files)"
    else
        record_test "wasm_artifacts" "FAIL" "No WASM artifacts found"
        return 1
    fi
}

test_performance() {
    log_step "Running performance checks..."
    cd "$PROJECT_ROOT"
    
    # Check WASM file sizes
    log_info "Checking WASM file sizes..."
    local large_files=0
    for wasm_file in target/wasm32-unknown-unknown/release/*.wasm; do
        if [ -f "$wasm_file" ]; then
            file_size=$(stat -f%z "$wasm_file" 2>/dev/null || stat -c%s "$wasm_file" 2>/dev/null || echo "0")
            if [ "$file_size" -gt 500000 ]; then
                log_warning "Large WASM file: $(basename "$wasm_file") (${file_size} bytes)"
                large_files=$((large_files + 1))
            fi
        fi
    done
    
    if [ "$large_files" -eq 0 ]; then
        record_test "wasm_size_check" "PASS" "All WASM files are within size limits"
    else
        record_test "wasm_size_check" "FAIL" "$large_files WASM files exceed size limit"
        return 1
    fi
}

show_summary() {
    echo
    echo "=========================================="
    log_info "Release Test Summary"
    echo "=========================================="
    echo "Total Tests:    $TOTAL_TESTS"
    echo -e "${GREEN}Passed:         $PASSED_TESTS${NC}"
    echo -e "${RED}Failed:         $FAILED_TESTS${NC}"
    echo -e "${YELLOW}Skipped:        $SKIPPED_TESTS${NC}"
    echo "=========================================="
    echo "Log file: $LOG_FILE"
    echo "=========================================="
    
    if [ "$FAILED_TESTS" -gt 0 ]; then
        echo -e "${RED}❌ Release tests FAILED${NC}"
        return 1
    else
        echo -e "${GREEN}✅ Release tests PASSED${NC}"
        return 0
    fi
}

# Main execution
main() {
    log_info "Starting Release Testing Framework"
    log_info "Timestamp: $TIMESTAMP"
    log_info "Test Suite: $TEST_SUITE"
    
    # Always run prerequisites
    test_prerequisites
    
    # Run test suites based on selection
    case $TEST_SUITE in
        all)
            if [ "$QUICK" = true ]; then
                test_smoke
            else
                test_build
                test_code_quality
                test_unit_tests
                test_security
                test_e2e
                test_performance
            fi
            ;;
        build)
            test_build
            ;;
        unit)
            test_unit_tests
            ;;
        e2e)
            test_e2e
            ;;
        security)
            test_security
            ;;
        smoke)
            test_smoke
            ;;
        performance)
            test_performance
            ;;
    esac
    
    # Show summary and exit
    show_summary
    exit_code=$?
    
    if [ $exit_code -ne 0 ]; then
        log_error "Release testing failed. Review log file: $LOG_FILE"
    else
        log_success "Release testing completed successfully!"
    fi
    
    exit $exit_code
}

# Run main function
main "$@"
