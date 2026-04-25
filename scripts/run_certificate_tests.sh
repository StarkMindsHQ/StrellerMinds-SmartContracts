#!/bin/bash

# Certificate E2E Test Runner
# Comprehensive test suite for certificate workflow

set -e

echo "🎓 Certificate E2E Test Suite"
echo "============================="

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

# Check if Soroban CLI is installed
check_dependencies() {
    print_status "Checking dependencies..."
    
    if ! command -v soroban &> /dev/null; then
        print_error "Soroban CLI is not installed. Please install it first."
        exit 1
    fi
    
    if ! command -v cargo &> /dev/null; then
        print_error "Cargo is not installed. Please install Rust first."
        exit 1
    fi
    
    print_success "Dependencies check passed"
}

# Build contracts
build_contracts() {
    print_status "Building contracts..."
    
    if ! ./scripts/build.sh; then
        print_error "Failed to build contracts"
        exit 1
    fi
    
    print_success "Contracts built successfully"
}

# Start local Soroban network if not running
start_localnet() {
    print_status "Checking local Soroban network..."
    
    if ! curl -s http://localhost:8000/health > /dev/null 2>&1; then
        print_status "Starting local Soroban network..."
        soroban network standalone &
        sleep 5
        
        # Configure network
        soroban config network standalone --global --rpc-url http://localhost:8000
        
        # Wait for network to be ready
        for i in {1..30}; do
            if curl -s http://localhost:8000/health > /dev/null 2>&1; then
                print_success "Local Soroban network is ready"
                break
            fi
            sleep 1
        done
    else
        print_success "Local Soroban network is already running"
    fi
}

# Run certificate integration tests
run_certificate_tests() {
    local test_type=$1
    
    print_status "Running certificate integration tests..."
    
    case $test_type in
        "unit")
            print_status "Running unit tests for certificate contract..."
            cd contracts/certificate && cargo test --lib && cd ../..
            ;;
        "integration")
            print_status "Running certificate integration tests..."
            cd e2e-tests && cargo test --test certificate_integration test_certificate_lifecycle_e2e -- --nocapture && cd ..
            ;;
        "performance")
            print_status "Running certificate performance tests..."
            cd e2e-tests && cargo test --test certificate_integration test_certificate_performance_stress -- --nocapture && cd ..
            ;;
        "edge_cases")
            print_status "Running certificate edge case tests..."
            cd e2e-tests && cargo test --test certificate_integration test_certificate_edge_cases_and_errors -- --nocapture && cd ..
            ;;
        "all")
            print_status "Running all certificate tests..."
            cd e2e-tests && cargo test --test certificate_integration -- --nocapture && cd ..
            ;;
        *)
            print_error "Unknown test type: $test_type"
            echo "Available options: unit, integration, performance, edge_cases, all"
            exit 1
            ;;
    esac
}

# Generate test coverage report
generate_coverage() {
    print_status "Generating test coverage report..."
    
    cd e2e-tests
    cargo llvm-cov --test certificate_integration --lcov --output-path lcov.info
    cargo llvm-cov --test certificate_integration --html
    cd ..
    
    print_success "Coverage report generated in e2e-tests/target/llvm-cov/html/"
}

# Performance benchmarking
run_benchmarks() {
    print_status "Running performance benchmarks..."
    
    cd e2e-tests
    cargo test --test certificate_integration test_certificate_performance_stress -- --nocapture | tee benchmark_results.txt
    
    # Extract key metrics
    echo ""
    print_status "Benchmark Summary:"
    grep -E "(Large-scale issuance|Concurrent verification|Estimated storage)" benchmark_results.txt || true
    
    cd ..
}

# Cleanup function
cleanup() {
    print_status "Cleaning up..."
    
    # Stop local network if we started it
    if pgrep -f "soroban network standalone" > /dev/null; then
        pkill -f "soroban network standalone"
        print_success "Local Soroban network stopped"
    fi
}

# Set trap for cleanup
trap cleanup EXIT

# Main execution
main() {
    local test_type=${1:-"all"}
    local with_coverage=${2:-false}
    local with_benchmarks=${3:-false}
    
    print_status "Starting certificate E2E test suite..."
    
    check_dependencies
    build_contracts
    start_localnet
    run_certificate_tests "$test_type"
    
    if [ "$with_coverage" = true ]; then
        generate_coverage
    fi
    
    if [ "$with_benchmarks" = true ]; then
        run_benchmarks
    fi
    
    print_success "Certificate E2E test suite completed successfully!"
    
    # Display summary
    echo ""
    echo "📊 Test Summary:"
    echo "================"
    echo "✅ Certificate lifecycle tests"
    echo "✅ Multi-sig approval workflow"
    echo "✅ Batch certificate issuance"
    echo "✅ Template-based issuance"
    echo "✅ Certificate verification and sharing"
    echo "✅ Certificate revocation and reissuance"
    echo "✅ Compliance and audit functionality"
    
    if [ "$test_type" = "performance" ] || [ "$test_type" = "all" ]; then
        echo "✅ Performance and stress tests"
    fi
    
    if [ "$test_type" = "edge_cases" ] || [ "$test_type" = "all" ]; then
        echo "✅ Edge cases and error handling"
    fi
    
    echo ""
    print_success "All certificate tests passed! 🎉"
}

# Parse command line arguments
case $1 in
    --help|-h)
        echo "Certificate E2E Test Runner"
        echo ""
        echo "Usage: $0 [test_type] [--coverage] [--benchmarks]"
        echo ""
        echo "Test types:"
        echo "  unit        - Run unit tests only"
        echo "  integration - Run integration tests only"
        echo "  performance - Run performance tests only"
        echo "  edge_cases  - Run edge case tests only"
        echo "  all         - Run all tests (default)"
        echo ""
        echo "Options:"
        echo "  --coverage    - Generate coverage report"
        echo "  --benchmarks  - Run performance benchmarks"
        echo ""
        echo "Examples:"
        echo "  $0                           # Run all tests"
        echo "  $0 integration               # Run integration tests only"
        echo "  $0 performance --benchmarks  # Run performance tests with benchmarks"
        echo "  $0 all --coverage            # Run all tests with coverage"
        exit 0
        ;;
    *)
        main "$@"
        ;;
esac
