#!/bin/bash

# Soroban E2E Test Runner
# This script orchestrates the complete E2E testing workflow:
# 1. Starts Soroban localnet
# 2. Builds and deploys contracts
# 3. Runs E2E tests
# 4. Cleans up resources

set -e

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
LOCALNET_SCRIPT="$SCRIPT_DIR/start_localnet.sh"
LOCALNET_TIMEOUT=60
TEST_TIMEOUT=300

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m' # No Color

# Logging functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_step() {
    echo -e "${PURPLE}[STEP]${NC} $1"
}

# Function to show help
show_help() {
    cat << EOF
Soroban E2E Test Runner

Usage: $0 [OPTIONS] [TEST_FILTER]

OPTIONS:
    -h, --help          Show this help message
    -k, --keep-running  Keep localnet running after tests complete
    -b, --build-only    Only build contracts, don't run tests
    -t, --tests-only    Only run tests (assumes localnet is already running)
    -v, --verbose       Enable verbose output
    -c, --clean         Clean build artifacts before starting
    --quick             Run quick smoke tests only

TEST_FILTER:
    Optional regex pattern to filter which tests to run.
    Examples:
        $0 connectivity          # Run only connectivity tests
        $0 certificate           # Run only certificate-related tests
        $0 "test_end_to_end"     # Run specific test function

EXAMPLES:
    $0                          # Run all E2E tests
    $0 -k                       # Run tests and keep localnet running
    $0 -t certificate           # Run certificate tests only (localnet must be running)
    $0 --clean --verbose        # Clean build and run with verbose output
    $0 --quick                  # Run quick smoke tests

EOF
}

# Parse command line arguments
KEEP_RUNNING=false
BUILD_ONLY=false
TESTS_ONLY=false
VERBOSE=false
CLEAN=false
QUICK=false
TEST_FILTER=""

while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            show_help
            exit 0
            ;;
        -k|--keep-running)
            KEEP_RUNNING=true
            shift
            ;;
        -b|--build-only)
            BUILD_ONLY=true
            shift
            ;;
        -t|--tests-only)
            TESTS_ONLY=true
            shift
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        -c|--clean)
            CLEAN=true
            shift
            ;;
        --quick)
            QUICK=true
            shift
            ;;
        *)
            if [[ -z "$TEST_FILTER" ]]; then
                TEST_FILTER="$1"
            else
                log_error "Unknown option: $1"
                show_help
                exit 1
            fi
            shift
            ;;
    esac
done

# Function to check prerequisites
check_prerequisites() {
    log_step "Checking prerequisites..."
    
    # Check if we're in the right directory
    if [[ ! -f "$PROJECT_ROOT/Cargo.toml" ]]; then
        log_error "Not in project root directory. Please run from project root."
        exit 1
    fi
    
    # Check if Docker is running
    if ! docker info > /dev/null 2>&1; then
        log_error "Docker is not running. Please start Docker."
        exit 1
    fi
    
    # Check if Soroban CLI is installed
    if ! command -v soroban &> /dev/null; then
        log_error "Soroban CLI is not installed. Install with: cargo install --locked soroban-cli"
        exit 1
    fi
    
    # Check if localnet script exists
    if [[ ! -f "$LOCALNET_SCRIPT" ]]; then
        log_error "Localnet script not found at: $LOCALNET_SCRIPT"
        exit 1
    fi
    
    log_success "Prerequisites check passed"
}

# Function to clean build artifacts
clean_artifacts() {
    if [[ "$CLEAN" == true ]]; then
        log_step "Cleaning build artifacts..."
        cd "$PROJECT_ROOT"
        
        # Clean Cargo artifacts
        cargo clean
        
        # Remove any existing WASM files
        find target -name "*.wasm" -delete 2>/dev/null || true
        
        log_success "Build artifacts cleaned"
    fi
}

# Function to start localnet
start_localnet() {
    if [[ "$TESTS_ONLY" == false ]]; then
        log_step "Starting Soroban localnet..."
        
        # Start the localnet using our script
        "$LOCALNET_SCRIPT" start
        
        # Wait for localnet to be fully ready
        local attempt=0
        local max_attempts=$((LOCALNET_TIMEOUT / 2))
        
        while [ $attempt -lt $max_attempts ]; do
            if curl -s "http://localhost:8000/health" > /dev/null 2>&1; then
                break
            fi
            
            attempt=$((attempt + 1))
            sleep 2
            
            if [ $((attempt % 10)) -eq 0 ]; then
                log_info "Still waiting for localnet... (${attempt}/${max_attempts})"
            fi
        done
        
        if [ $attempt -eq $max_attempts ]; then
            log_error "Localnet failed to start within ${LOCALNET_TIMEOUT} seconds"
            "$LOCALNET_SCRIPT" logs
            exit 1
        fi
        
        log_success "Localnet is ready"
    else
        log_step "Checking if localnet is running..."
        
        if ! curl -s "http://localhost:8000/health" > /dev/null 2>&1; then
            log_error "Localnet is not running. Please start it first with: $LOCALNET_SCRIPT start"
            exit 1
        fi
        
        log_success "Localnet is running"
    fi
}

# Function to build contracts
build_contracts() {
    if [[ "$TESTS_ONLY" == false ]]; then
        log_step "Building contracts..."
        cd "$PROJECT_ROOT"
        
        # Use the existing build script
        if [[ -f "scripts/build.sh" ]]; then
            ./scripts/build.sh
        else
            # Fallback to direct cargo build
            cargo build --target wasm32-unknown-unknown --release
        fi
        
        log_success "Contracts built successfully"
    fi
}

# Function to run E2E tests
run_e2e_tests() {
    if [[ "$BUILD_ONLY" == false ]]; then
        log_step "Running E2E tests..."
        cd "$PROJECT_ROOT"
        
        # Prepare test command
        local test_cmd="cargo test -p e2e-tests"
        
        # Add verbose flag if requested
        if [[ "$VERBOSE" == true ]]; then
            test_cmd="$test_cmd -- --nocapture"
        fi
        
        # Add test filter if provided
        if [[ -n "$TEST_FILTER" ]]; then
            if [[ "$VERBOSE" == true ]]; then
                test_cmd="$test_cmd $TEST_FILTER"
            else
                test_cmd="$test_cmd $TEST_FILTER -- --nocapture"
            fi
        fi
        
        # Run quick tests only if requested
        if [[ "$QUICK" == true ]]; then
            if [[ "$VERBOSE" == true ]]; then
                test_cmd="$test_cmd test_localnet_connectivity test_account_setup"
            else
                test_cmd="$test_cmd test_localnet_connectivity test_account_setup -- --nocapture"
            fi
        fi
        
        log_info "Executing: $test_cmd"
        
        # Set timeout for tests
        timeout $TEST_TIMEOUT bash -c "$test_cmd" || {
            local exit_code=$?
            if [ $exit_code -eq 124 ]; then
                log_error "E2E tests timed out after ${TEST_TIMEOUT} seconds"
            else
                log_error "E2E tests failed with exit code: $exit_code"
            fi
            return $exit_code
        }
        
        log_success "E2E tests completed successfully"
    fi
}

# Function to stop localnet
stop_localnet() {
    if [[ "$KEEP_RUNNING" == false && "$TESTS_ONLY" == false ]]; then
        log_step "Stopping localnet..."
        "$LOCALNET_SCRIPT" stop
        log_success "Localnet stopped"
    else
        log_info "Leaving localnet running as requested"
        log_info "To stop manually, run: $LOCALNET_SCRIPT stop"
    fi
}

# Function to show test results summary
show_summary() {
    echo
    echo "=========================================="
    log_info "E2E Test Summary"
    echo "=========================================="
    
    if [[ "$BUILD_ONLY" == true ]]; then
        echo "✅ Contracts built successfully"
        echo "📋 Use '$0 -t' to run tests against running localnet"
    elif [[ "$TESTS_ONLY" == true ]]; then
        echo "✅ Tests completed against running localnet"
    else
        echo "✅ Complete E2E test cycle completed"
        echo "   - Localnet started"
        echo "   - Contracts built"
        echo "   - Tests executed"
        if [[ "$KEEP_RUNNING" == true ]]; then
            echo "   - Localnet left running"
        else
            echo "   - Localnet stopped"
        fi
    fi
    
    if [[ -n "$TEST_FILTER" ]]; then
        echo "🎯 Test filter applied: $TEST_FILTER"
    fi
    
    echo
    log_info "Useful commands:"
    echo "  View localnet status:    $LOCALNET_SCRIPT status"
    echo "  View localnet logs:      $LOCALNET_SCRIPT logs"
    echo "  Stop localnet:           $LOCALNET_SCRIPT stop"
    echo "  Run specific test:       $0 test_name"
    echo "  Run with verbose output: $0 -v"
}

# Cleanup function for graceful shutdown
cleanup() {
    local exit_code=$?
    
    if [[ $exit_code -ne 0 ]]; then
        log_error "E2E test runner failed"
        
        # Show localnet logs if available
        if [[ "$TESTS_ONLY" == false ]]; then
            log_info "Localnet logs (last 20 lines):"
            "$LOCALNET_SCRIPT" logs 2>/dev/null | tail -20 || echo "No logs available"
        fi
    fi
    
    # Always try to stop localnet if we started it and not keeping it running
    if [[ "$KEEP_RUNNING" == false && "$TESTS_ONLY" == false ]]; then
        "$LOCALNET_SCRIPT" stop 2>/dev/null || true
    fi
    
    exit $exit_code
}

# Set up signal handlers
trap cleanup EXIT INT TERM

# Main execution
main() {
    echo "🚀 Soroban E2E Test Runner"
    echo "=========================="
    
    check_prerequisites
    clean_artifacts
    start_localnet
    build_contracts
    run_e2e_tests
    stop_localnet
    show_summary
    
    log_success "E2E test runner completed successfully! 🎉"
}

# Execute main function
main "$@"