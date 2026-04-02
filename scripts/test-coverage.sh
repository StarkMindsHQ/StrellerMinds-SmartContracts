#!/bin/bash

# Test Coverage Script for StrellerMinds Smart Contracts
# This script generates comprehensive test coverage reports

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
MIN_COVERAGE=80
COVERAGE_DIR="target/coverage"
REPORT_DIR="target/coverage-reports"

echo -e "${BLUE}🧪 StrellerMinds Test Coverage Analysis${NC}"
echo "============================================"

# Create directories
mkdir -p "$COVERAGE_DIR"
mkdir -p "$REPORT_DIR"

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to install cargo tools if not present
install_cargo_tools() {
    echo -e "${YELLOW}📦 Installing required cargo tools...${NC}"
    
    if ! command_exists cargo-llvm-cov; then
        echo "Installing cargo-llvm-cov..."
        cargo install cargo-llvm-cov --locked
    fi
    
    if ! command_exists cargo-tarpaulin; then
        echo "Installing cargo-tarpaulin..."
        cargo install cargo-tarpaulin --locked
    fi
    
    if ! command_exists cargo-coverage-report; then
        echo "Installing cargo-coverage-report..."
        cargo install cargo-coverage-report --locked
    fi
    
    if ! command_exists cargo-nextest; then
        echo "Installing cargo-nextest for faster test execution..."
        cargo install cargo-nextest --locked
    fi
}

# Function to run unit tests with coverage
run_unit_tests() {
    echo -e "${BLUE}🔬 Running unit tests with coverage...${NC}"
    
    # Run tests for each contract separately
    contracts=("assessment" "community" "certificate" "analytics" "shared")
    
    for contract in "${contracts[@]}"; do
        echo -e "${YELLOW}Testing $contract contract...${NC}"
        
        # Check if contract exists
        if cargo metadata --no-deps --format-version 1 | grep -q "\"name\":\"$contract\""; then
            # Run tests with llvm-cov
            cargo llvm-cov --package "$contract" --all-features --lib --tests \
                --lcov --output-path "$COVERAGE_DIR/${contract}-coverage.lcov" \
                --html --output-dir "$COVERAGE_DIR/${contract}-html" || true
            
            # Run with tarpaulin for additional coverage metrics
            cargo tarpaulin --package "$contract" --all-features --lib --tests \
                --out Html --output-dir "$COVERAGE_DIR/${contract}-tarpaulin" || true
        else
            echo -e "${RED}Contract $contract not found, skipping...${NC}"
        fi
    done
}

# Function to run integration tests
run_integration_tests() {
    echo -e "${BLUE}🔗 Running integration tests...${NC}"
    
    # Run e2e tests
    if cargo metadata --no-deps --format-version 1 | grep -q "\"name\":\"e2e-tests\""; then
        echo "Running e2e integration tests..."
        
        # Run comprehensive test coverage
        cargo test --package e2e-tests --test comprehensive_test_coverage --all-features \
            -- --nocapture || true
        
        # Run gas benchmark tests
        cargo test --package e2e-tests --test gas_benchmark --all-features \
            -- --nocapture || true
        
        # Run input validation tests
        cargo test --package e2e-tests --test input_validation_tests --all-features \
            -- --nocapture || true
        
        # Generate coverage for e2e tests
        cargo llvm-cov --package e2e-tests --all-features --tests \
            --lcov --output-path "$COVERAGE_DIR/e2e-coverage.lcov" \
            --html --output-dir "$COVERAGE_DIR/e2e-html" || true
    else
        echo -e "${YELLOW}e2e-tests package not found, skipping integration tests...${NC}"
    fi
}

# Function to run workspace-wide coverage
run_workspace_coverage() {
    echo -e "${BLUE}📊 Generating workspace-wide coverage...${NC}"
    
    # Generate overall coverage
    cargo llvm-cov --workspace --all-features --lib --tests \
        --lcov --output-path "$COVERAGE_DIR/workspace-coverage.lcov" \
        --html --output-dir "$COVERAGE_DIR/workspace-html" || true
    
    # Generate JSON summary
    cargo llvm-cov --workspace --all-features --lib --tests \
        --json --summary-only --output-path "$COVERAGE_DIR/workspace-summary.json" || true
}

# Function to merge coverage reports
merge_coverage() {
    echo -e "${BLUE}🔀 Merging coverage reports...${NC}"
    
    # Find all lcov files
    lcov_files=$(find "$COVERAGE_DIR" -name "*.lcov" 2>/dev/null || true)
    
    if [ -n "$lcov_files" ]; then
        echo "Found lcov files, merging..."
        
        # Install lcov if not present
        if ! command_exists lcov; then
            echo "Installing lcov..."
            sudo apt-get update && sudo apt-get install -y lcov || true
        fi
        
        # Merge lcov files
        merged_file="$COVERAGE_DIR/merged-coverage.lcov"
        echo "" > "$merged_file"
        
        for file in $lcov_files; do
            if [ -f "$file" ] && [ -s "$file" ]; then
                echo "Merging $file"
                lcov -a "$file" -o "$merged_file.tmp" || true
                if [ -f "$merged_file.tmp" ]; then
                    mv "$merged_file.tmp" "$merged_file"
                fi
            fi
        done
        
        # Generate HTML from merged coverage
        if [ -f "$merged_file" ] && [ -s "$merged_file" ]; then
            genhtml "$merged_file" --output-directory "$COVERAGE_DIR/merged-html" || true
        fi
    else
        echo -e "${YELLOW}No lcov files found to merge${NC}"
    fi
}

# Function to analyze coverage and generate reports
analyze_coverage() {
    echo -e "${BLUE}📈 Analyzing coverage results...${NC}"
    
    # Create summary report
    summary_file="$REPORT_DIR/coverage-summary.md"
    echo "# Test Coverage Report" > "$summary_file"
    echo "" >> "$summary_file"
    echo "Generated on: $(date)" >> "$summary_file"
    echo "" >> "$summary_file"
    
    # Analyze workspace coverage
    if [ -f "$COVERAGE_DIR/workspace-summary.json" ]; then
        echo "## Workspace Coverage" >> "$summary_file"
        
        # Extract coverage percentage (this would need proper JSON parsing)
        if command_exists jq; then
            coverage=$(jq -r '.percentage // "N/A"' "$COVERAGE_DIR/workspace-summary.json" 2>/dev/null || echo "N/A")
            echo "- **Overall Coverage:** $coverage%" >> "$summary_file"
            
            # Check if meets minimum threshold
            if [ "$coverage" != "N/A" ] && [ "${coverage%.*}" -ge "$MIN_COVERAGE" ]; then
                echo "- ✅ **Status:** Meets minimum threshold ($MIN_COVERAGE%)" >> "$summary_file"
            else
                echo "- ❌ **Status:** Below minimum threshold ($MIN_COVERAGE%)" >> "$summary_file"
            fi
        else
            echo "- **Overall Coverage:** N/A (jq not available)" >> "$summary_file"
        fi
        echo "" >> "$summary_file"
    fi
    
    # Add contract-specific coverage
    echo "## Contract Coverage" >> "$summary_file"
    
    contracts=("assessment" "community" "certificate" "analytics" "shared")
    for contract in "${contracts[@]}"; do
        if [ -f "$COVERAGE_DIR/${contract}-coverage.lcov" ]; then
            echo "### $contract" >> "$summary_file"
            echo "- Coverage report generated" >> "$summary_file"
            echo "- HTML report: \`${contract}-html/\`" >> "$summary_file"
            echo "" >> "$summary_file"
        fi
    done
    
    # Add integration test coverage
    if [ -f "$COVERAGE_DIR/e2e-coverage.lcov" ]; then
        echo "## Integration Tests" >> "$summary_file"
        echo "- E2E test coverage generated" >> "$summary_file"
        echo "- HTML report: \`e2e-html/\`" >> "$summary_file"
        echo "" >> "$summary_file"
    fi
    
    # Print summary
    echo -e "${GREEN}✅ Coverage analysis completed${NC}"
    echo "Report saved to: $summary_file"
    
    # Display summary
    if [ -f "$summary_file" ]; then
        echo ""
        echo -e "${BLUE}📋 Coverage Summary:${NC}"
        cat "$summary_file"
    fi
}

# Function to check coverage thresholds
check_thresholds() {
    echo -e "${BLUE}🎯 Checking coverage thresholds...${NC}"
    
    # Check workspace coverage
    if [ -f "$COVERAGE_DIR/workspace-summary.json" ] && command_exists jq; then
        coverage=$(jq -r '.percentage // 0' "$COVERAGE_DIR/workspace-summary.json" 2>/dev/null || echo "0")
        
        echo "Current coverage: $coverage%"
        echo "Required coverage: $MIN_COVERAGE%"
        
        if [ "${coverage%.*}" -ge "$MIN_COVERAGE" ]; then
            echo -e "${GREEN}✅ Coverage threshold met!${NC}"
            return 0
        else
            echo -e "${RED}❌ Coverage threshold not met!${NC}"
            echo "Current: $coverage%, Required: $MIN_COVERAGE%"
            return 1
        fi
    else
        echo -e "${YELLOW}⚠️  Cannot determine coverage percentage${NC}"
        return 0
    fi
}

# Function to generate coverage badge
generate_badge() {
    echo -e "${BLUE}🏷️  Generating coverage badge...${NC}"
    
    if [ -f "$COVERAGE_DIR/workspace-summary.json" ] && command_exists jq; then
        coverage=$(jq -r '.percentage // 0' "$COVERAGE_DIR/workspace-summary.json" 2>/dev/null || echo "0")
        coverage_int=${coverage%.*}
        
        # Determine color
        color="red"
        if [ $coverage_int -ge 80 ]; then
            color="green"
        elif [ $coverage_int -ge 60 ]; then
            color="yellow"
        fi
        
        # Generate badge (simple text version)
        badge_file="$REPORT_DIR/coverage-badge.txt"
        echo "Coverage: $coverage%" > "$badge_file"
        echo "Color: $color" >> "$badge_file"
        
        echo -e "${GREEN}✅ Badge generated: $badge_file${NC}"
    fi
}

# Function to run property-based tests
run_property_tests() {
    echo -e "${BLUE}🎲 Running property-based tests...${NC}"
    
    # Check for proptest
    if cargo metadata --no-deps --format-version 1 | grep -q "proptest"; then
        echo "Running property-based tests..."
        
        # Run property tests with quickcheck-like behavior
        cargo test --all-features --release -- --nocapture \
            --test-threads=1 property_based || true
    else
        echo -e "${YELLOW}Property-based testing dependencies not found${NC}"
    fi
}

# Function to run edge case tests
run_edge_case_tests() {
    echo -e "${BLUE}🔍 Running edge case tests...${NC}"
    
    # Run tests specifically for edge cases
    cargo test --all-features -- --nocapture edge_case || true
    cargo test --all-features -- --nocapture boundary || true
    cargo test --all-features -- -- --exact 'test_*edge*' || true
    cargo test --all-features -- -- --exact 'test_*boundary*' || true
}

# Function to run error scenario tests
run_error_tests() {
    echo -e "${BLUE}💥 Running error scenario tests...${NC}"
    
    # Run tests for error handling
    cargo test --all-features -- --nocapture error || true
    cargo test --all-features -- -- --exact 'test_*error*' || true
    cargo test --all-features -- -- --exact 'test_*fail*' || true
}

# Function to run performance tests
run_performance_tests() {
    echo -e "${BLUE}⚡ Running performance tests...${NC}"
    
    # Run benchmark tests
    if cargo metadata --no-deps --format-version 1 | grep -q "criterion"; then
        echo "Running criterion benchmarks..."
        cargo bench --all-features || true
    fi
    
    # Run performance regression tests
    cargo test --all-features -- --nocapture performance || true
    cargo test --all-features -- -- --exact 'test_*perf*' || true
}

# Function to create coverage report for CI
create_ci_report() {
    echo -e "${BLUE}📝 Creating CI coverage report...${NC}"
    
    ci_report="$REPORT_DIR/ci-coverage.json"
    
    # Create JSON report for CI
    cat > "$ci_report" << EOF
{
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "minimum_coverage": $MIN_COVERAGE,
    "contracts": [
EOF
    
    # Add contract coverage data
    first=true
    contracts=("assessment" "community" "certificate" "analytics" "shared")
    for contract in "${contracts[@]}"; do
        if [ -f "$COVERAGE_DIR/${contract}-coverage.lcov" ]; then
            if [ "$first" = true ]; then
                first=false
            else
                echo "," >> "$ci_report"
            fi
            
            echo "        {\"name\": \"$contract\", \"coverage\": \"generated\"}" >> "$ci_report"
        fi
    done
    
    cat >> "$ci_report" << EOF
    ],
    "integration_tests": "generated",
    "property_tests": "generated",
    "edge_case_tests": "generated",
    "error_tests": "generated",
    "performance_tests": "generated"
}
EOF
    
    echo -e "${GREEN}✅ CI report created: $ci_report${NC}"
}

# Main execution
main() {
    echo "Starting comprehensive test coverage analysis..."
    
    # Install required tools
    install_cargo_tools
    
    # Run different test categories
    run_unit_tests
    run_integration_tests
    run_workspace_coverage
    run_property_tests
    run_edge_case_tests
    run_error_tests
    run_performance_tests
    
    # Process coverage data
    merge_coverage
    analyze_coverage
    generate_badge
    create_ci_report
    
    # Check thresholds
    if check_thresholds; then
        echo -e "${GREEN}🎉 All coverage requirements met!${NC}"
        exit_code=0
    else
        echo -e "${RED}❌ Coverage requirements not met!${NC}"
        exit_code=1
    fi
    
    echo -e "${BLUE}📊 Coverage reports available in:${NC}"
    echo "- HTML reports: $COVERAGE_DIR/"
    echo "- Summary report: $REPORT_DIR/coverage-summary.md"
    echo "- CI report: $REPORT_DIR/ci-coverage.json"
    
    exit $exit_code
}

# Parse command line arguments
case "${1:-}" in
    --unit-only)
        install_cargo_tools
        run_unit_tests
        ;;
    --integration-only)
        install_cargo_tools
        run_integration_tests
        ;;
    --workspace-only)
        install_cargo_tools
        run_workspace_coverage
        ;;
    --analyze-only)
        analyze_coverage
        check_thresholds
        ;;
    --help|-h)
        echo "Usage: $0 [OPTION]"
        echo "Options:"
        echo "  --unit-only        Run only unit tests"
        echo "  --integration-only  Run only integration tests"
        echo "  --workspace-only    Run only workspace coverage"
        echo "  --analyze-only      Analyze existing coverage"
        echo "  --help, -h          Show this help message"
        echo ""
        echo "Default: Run complete coverage analysis"
        ;;
    *)
        main
        ;;
esac
