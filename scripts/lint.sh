#!/bin/bash

# Comprehensive linting script for StrellerMinds Smart Contracts
# This script runs quality checks and can be used in CI/CD pipelines

set -e

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

# Function to run a check and report results
run_check() {
    local name="$1"
    local command="$2"
    local allow_failure="${3:-false}"
    
    print_status "Running $name..."
    
    if eval "$command"; then
        print_success "$name passed"
        return 0
    else
        if [ "$allow_failure" = "true" ]; then
            print_warning "$name failed (allowed)"
            return 0
        else
            print_error "$name failed"
            return 1
        fi
    fi
}

# Main linting function
main() {
    print_status "Starting linting checks..."
    
    # Check if we're in the right directory
    if [ ! -f "Cargo.toml" ]; then
        print_error "Not in a Cargo project directory"
        exit 1
    fi
    
    # Run checks in order of importance
    checks=(
        "cargo-fmt:cargo fmt --all -- --check:false"
        "cargo-clippy:cargo clippy --workspace --all-targets --all-features -- -D warnings:false"
        "cargo-build:cargo build --workspace --release:false"
        "cargo-test:cargo test --workspace:false"
    )
    
    failed_checks=()
    
    for check in "${checks[@]}"; do
        IFS=':' read -r name command allow_failure <<< "$check"
        
        if ! run_check "$name" "$command" "$allow_failure"; then
            failed_checks+=("$name")
        fi
        
        echo ""
    done
    
    # Report final results
    if [ ${#failed_checks[@]} -eq 0 ]; then
        print_success "All checks passed! 🎉"
        exit 0
    else
        print_error "The following checks failed:"
        for check in "${failed_checks[@]}"; do
            echo "  - $check"
        done
        print_error "Please fix the issues before proceeding"
        exit 1
    fi
}

# Handle command line arguments
case "${1:-}" in
    --fix)
        print_status "Running auto-fix checks..."
        cargo fmt --all
        print_success "Code formatted"
        ;;
    --fast)
        print_status "Running fast checks only..."
        run_check "cargo-fmt" "cargo fmt --all -- --check"
        run_check "cargo-clippy" "cargo clippy --workspace --all-targets --all-features -- -D warnings"
        ;;
    --help|-h)
        echo "Usage: $0 [OPTION]"
        echo "Options:"
        echo "  --fix    Run auto-fix checks (formatting only)"
        echo "  --fast   Run only fast checks (fmt, clippy)"
        echo "  --help   Show this help message"
        echo ""
        echo "Default: Run all comprehensive checks"
        ;;
    *)
        main
        ;;
esac
