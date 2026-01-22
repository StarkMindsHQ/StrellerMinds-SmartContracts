#!/bin/bash

# Analytics Integration Test Runner
# This script runs the comprehensive analytics integration test suite locally

set -e

echo "🚀 Starting Analytics Integration Test Suite"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check prerequisites
check_prerequisites() {
    print_status "Checking prerequisites..."
    
    # Check if Soroban CLI is installed
    if ! command -v soroban &> /dev/null; then
        print_error "Soroban CLI is not installed. Please install it first."
        exit 1
    fi
    
    # Check if Rust is installed
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo is not installed. Please install it first."
        exit 1
    fi
    
    # Check if local network is running
    if ! soroban network info 2>/dev/null | grep -q "standalone"; then
        print_warning "Soroban local network is not running. Starting it..."
        start_local_network
    fi
    
    print_success "Prerequisites check passed"
}

# Start Soroban local network
start_local_network() {
    print_status "Starting Soroban local network..."
    soroban network standalone &
    sleep 5
    soroban config network standalone --global --rpc-url http://localhost:8000
    print_success "Local network started"
}

# Build contracts
build_contracts() {
    print_status "Building contracts..."
    
    if [ -f "Makefile" ]; then
        make build
    else
        print_status "Building contracts manually..."
        cd contracts/analytics
        cargo build --release --target wasm32-unknown-unknown
        cd ../..
    fi
    
    print_success "Contracts built successfully"
}

# Run unit tests
run_unit_tests() {
    print_status "Running analytics contract unit tests..."
    
    cd contracts/analytics
    
    # Run library tests
    print_status "Running library tests..."
    cargo test --lib -- --nocapture
    
    # Run integration tests
    print_status "Running contract integration tests..."
    cargo test --integration -- --nocapture
    
    cd ../..
    print_success "Unit tests passed"
}

# Run E2E integration tests
run_e2e_tests() {
    print_status "Running E2E integration tests..."
    
    cd e2e-tests
    
    # Build test dependencies
    print_status "Building test dependencies..."
    cargo build --tests
    
    # Run specific test suites
    test_suites=(
        "test_learning_session_tracking_e2e"
        "test_progress_analytics_calculations"
        "test_leaderboard_generation"
        "test_performance_metrics_aggregation"
        "test_data_consistency_validation"
        "test_edge_cases_and_error_conditions"
        "test_cicd_pipeline_integration"
    )
    
    for test_suite in "${test_suites[@]}"; do
        print_status "Running $test_suite..."
        if cargo test --test analytics_integration "$test_suite" -- --nocapture; then
            print_success "$test_suite passed"
        else
            print_error "$test_suite failed"
            exit 1
        fi
    done
    
    cd ../..
    print_success "All E2E tests passed"
}

# Run performance benchmarks
run_performance_tests() {
    print_status "Running performance benchmarks..."
    
    cd e2e-tests
    
    # Run metrics aggregation test with performance tracking
    cargo test --test analytics_integration test_performance_metrics_aggregation -- --nocapture
    
    cd ../..
    print_success "Performance tests completed"
}

# Run security checks
run_security_checks() {
    print_status "Running security checks..."
    
    cd contracts/analytics
    
    # Check for common security issues
    if command -v cargo-audit &> /dev/null; then
        print_status "Running cargo audit..."
        cargo audit
    else
        print_warning "cargo-audit not installed, skipping security audit"
    fi
    
    # Run clippy with strict checks
    print_status "Running clippy with strict checks..."
    cargo clippy -- -D warnings -W clippy::all
    
    cd ../..
    print_success "Security checks completed"
}

# Generate test report
generate_report() {
    print_status "Generating test report..."
    
    # Create report directory
    mkdir -p reports/analytics
    
    # Generate coverage report if available
    cd contracts/analytics
    if command -v cargo-tarpaulin &> /dev/null; then
        print_status "Generating coverage report..."
        cargo tarpaulin --out Html --output-dir ../reports/analytics
        print_success "Coverage report generated at reports/analytics/tarpaulin-report.html"
    else
        print_warning "cargo-tarpaulin not installed, skipping coverage report"
    fi
    cd ../..
    
    # Generate documentation
    cd contracts/analytics
    print_status "Generating documentation..."
    cargo doc --no-deps --target-dir ../reports/analytics/docs
    print_success "Documentation generated at reports/analytics/docs/index.html"
    cd ../..
    
    print_success "Test report generated"
}

# Cleanup function
cleanup() {
    print_status "Cleaning up..."
    
    # Stop Soroban network
    pkill -f soroban || true
    
    # Clean up test artifacts
    rm -rf target/wasm32-unknown-unknown/release/build/
    rm -rf target/wasm32-unknown-unknown/release/deps/
    
    print_success "Cleanup completed"
}

# Main execution
main() {
    echo "=========================================="
    echo "  Analytics Integration Test Suite"
    echo "=========================================="
    
    # Set up error handling
    trap cleanup EXIT
    
    # Run test pipeline
    check_prerequisites
    build_contracts
    run_unit_tests
    run_e2e_tests
    run_performance_tests
    run_security_checks
    generate_report
    
    echo "=========================================="
    print_success "All tests completed successfully!"
    echo "=========================================="
    
    # Print summary
    echo ""
    echo "Test Summary:"
    echo "✅ Unit Tests: PASSED"
    echo "✅ E2E Integration Tests: PASSED"
    echo "✅ Performance Tests: PASSED"
    echo "✅ Security Checks: PASSED"
    echo ""
    echo "Reports generated in: reports/analytics/"
    echo "- Coverage Report: reports/analytics/tarpaulin-report.html"
    echo "- Documentation: reports/analytics/docs/index.html"
    echo ""
}

# Handle script arguments
case "${1:-run}" in
    "run")
        main
        ;;
    "unit")
        check_prerequisites
        run_unit_tests
        ;;
    "e2e")
        check_prerequisites
        build_contracts
        run_e2e_tests
        ;;
    "performance")
        check_prerequisites
        build_contracts
        run_performance_tests
        ;;
    "security")
        run_security_checks
        ;;
    "cleanup")
        cleanup
        ;;
    "help"|"-h"|"--help")
        echo "Analytics Integration Test Runner"
        echo ""
        echo "Usage: $0 [COMMAND]"
        echo ""
        echo "Commands:"
        echo "  run         Run full test suite (default)"
        echo "  unit        Run unit tests only"
        echo "  e2e         Run E2E integration tests only"
        echo "  performance Run performance tests only"
        echo "  security    Run security checks only"
        echo "  cleanup     Clean up test artifacts"
        echo "  help        Show this help message"
        echo ""
        ;;
    *)
        print_error "Unknown command: $1"
        echo "Use '$0 help' for usage information"
        exit 1
        ;;
esac
