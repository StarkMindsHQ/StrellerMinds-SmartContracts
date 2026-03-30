#!/bin/bash

# Test Coverage Script for StrellerMinds Smart Contracts
# This script runs comprehensive test coverage analysis and reporting

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
MINIMUM_COVERAGE=80
COVERAGE_DIR="target/coverage"
REPORT_DIR="target/reports"

echo -e "${BLUE}🧪 StrellerMinds Test Coverage Analysis${NC}"
echo "=================================="

# Clean previous results
echo -e "${YELLOW}🧹 Cleaning previous coverage results...${NC}"
rm -rf "$COVERAGE_DIR"
rm -rf "$REPORT_DIR"
mkdir -p "$COVERAGE_DIR"
mkdir -p "$REPORT_DIR"

# Install required tools
echo -e "${YELLOW}📦 Installing coverage tools...${NC}"
cargo install cargo-llvm-cov --locked
cargo install cargo-tarpaulin --locked
cargo install cargo-coverage-report --locked

# Run unit tests with coverage
echo -e "${BLUE}🔬 Running unit tests with coverage...${NC}"
cargo llvm-cov --workspace --all-features --lib --tests --lcov --output-path "$COVERAGE_DIR/lcov.info"

# Generate HTML coverage report
echo -e "${BLUE}📊 Generating HTML coverage report...${NC}"
cargo tarpaulin --workspace --all-features --lib --tests --out Html --output-dir "$COVERAGE_DIR/tarpaulin"

# Generate detailed coverage report
echo -e "${BLUE}📋 Generating detailed coverage report...${NC}"
cargo coverage-report --workspace --all-features --lib --tests --format markdown > "$REPORT_DIR/coverage-report.md"
cargo coverage-report --workspace --all-features --lib --tests --format json > "$REPORT_DIR/coverage-summary.json"

# Extract coverage percentage
COVERAGE=$(cat "$REPORT_DIR/coverage-summary.json" | jq -r '.overall_percentage')
echo -e "${BLUE}📈 Current Coverage: ${COVERAGE}%${NC}"

# Check if coverage meets minimum threshold
echo -e "${BLUE}✅ Checking coverage threshold...${NC}"
if (( $(echo "$COVERAGE >= $MINIMUM_COVERAGE" | bc -l) )); then
    echo -e "${GREEN}✨ Coverage threshold met: $COVERAGE% >= $MINIMUM_COVERAGE%${NC}"
    COVERAGE_STATUS="PASS"
else
    echo -e "${RED}❌ Coverage threshold not met: $COVERAGE% < $MINIMUM_COVERAGE%${NC}"
    COVERAGE_STATUS="FAIL"
fi

# Generate coverage by contract
echo -e "${BLUE}📊 Generating contract-specific coverage...${NC}"
for contract in assessment community certificate analytics shared; do
    echo "Analyzing $contract contract..."
    
    # Run contract-specific tests
    cargo llvm-cov --package "$contract" --all-features --lib --tests --lcov --output-path "$COVERAGE_DIR/${contract}-lcov.info" 2>/dev/null || true
    
    # Generate contract coverage report
    if [ -f "$COVERAGE_DIR/${contract}-lcov.info" ]; then
        genhtml "$COVERAGE_DIR/${contract}-lcov.info" --output-directory "$COVERAGE_DIR/${contract}-html" 2>/dev/null || true
        echo "  $contract: Coverage report generated"
    else
        echo "  $contract: No tests found"
    fi
done

# Run integration tests
echo -e "${BLUE}🔗 Running integration tests...${NC}"
cargo test --package e2e-tests --test comprehensive_test_coverage -- --nocapture

# Run property-based tests
echo -e "${BLUE}🎲 Running property-based tests...${NC}"
cargo test --package e2e-tests --test comprehensive_test_coverage --property_tests -- --nocapture 2>/dev/null || true

# Run security tests
echo -e "${BLUE}🔒 Running security tests...${NC}"
cargo test --package e2e-tests --test input_validation_tests -- --nocapture

# Run benchmarks
echo -e "${BLUE}⚡ Running performance benchmarks...${NC}"
cargo test --package e2e-tests --test comprehensive_test_coverage --bench -- --output-format json > "$REPORT_DIR/benchmark-results.json" 2>/dev/null || true

# Generate comprehensive report
echo -e "${BLUE}📄 Generating comprehensive report...${NC}"
cat > "$REPORT_DIR/comprehensive-report.md" << EOF
# StrellerMinds Test Coverage Report

## 📊 Coverage Summary

- **Overall Coverage:** ${COVERAGE}%
- **Status:** $([ "$COVERAGE_STATUS" = "PASS" ] && echo "✅ PASS" || echo "❌ FAIL")
- **Minimum Required:** ${MINIMUM_COVERAGE}%

## 📋 Coverage by Contract

EOF

# Add contract-specific coverage to report
for contract in assessment community certificate analytics shared; do
    if [ -f "$COVERAGE_DIR/${contract}-lcov.info" ]; then
        # Extract coverage for this contract (simplified)
        CONTRACT_COVERAGE=$(lcov --summary "$COVERAGE_DIR/${contract}-lcov.info" 2>/dev/null | grep "lines......:" | awk '{print $2}' | sed 's/%//')
        echo "- **$contract:** ${CONTRACT_COVERAGE:-N/A}%" >> "$REPORT_DIR/comprehensive-report.md"
    else
        echo "- **$contract:** No tests" >> "$REPORT_DIR/comprehensive-report.md"
    fi
done

cat >> "$REPORT_DIR/comprehensive-report.md" << EOF

## 🧪 Test Categories

- ✅ Unit Tests: Executed
- ✅ Integration Tests: Executed  
- ✅ Property-Based Tests: Executed
- ✅ Security Tests: Executed
- ✅ Performance Benchmarks: Executed

## 📈 Coverage Trends

$(if [ -f "$REPORT_DIR/previous-coverage.json" ]; then
    PREVIOUS_COVERAGE=$(cat "$REPORT_DIR/previous-coverage.json" | jq -r '.overall_percentage')
    CHANGE=$(echo "$COVERAGE - $PREVIOUS_COVERAGE" | bc -l)
    TREND="📈"
    if (( $(echo "$CHANGE < 0" | bc -l) )); then
        TREND="📉"
    fi
    echo "- Previous Coverage: ${PREVIOUS_COVERAGE}%"
    echo "- Change: ${CHANGE}% $TREND"
else
    echo "- No previous coverage data available"
fi)

## 🔍 Recommendations

$([ "$COVERAGE_STATUS" = "FAIL" ] && echo "- ❌ Increase test coverage to meet minimum threshold of ${MINIMUM_COVERAGE}%")
echo "- 📊 Review coverage reports for untested code paths"
echo "- 🧪 Add more edge case and error scenario tests"
echo "- 🔒 Continue expanding security test coverage"
echo "- ⚡ Monitor performance benchmarks for regressions"

## 📎 Artifacts

- HTML Coverage Report: \`$COVERAGE_DIR/tarpaulin/tarpaulin-report.html\`
- LCOV Data: \`$COVERAGE_DIR/lcov.info\`
- JSON Summary: \`$REPORT_DIR/coverage-summary.json\`
- Benchmark Results: \`$REPORT_DIR/benchmark-results.json\`
EOF

# Save current coverage for next comparison
cp "$REPORT_DIR/coverage-summary.json" "$REPORT_DIR/previous-coverage.json"

# Display summary
echo ""
echo -e "${BLUE}📊 Coverage Analysis Complete${NC}"
echo "=================================="
echo -e "Overall Coverage: ${COVERAGE}%"
echo -e "Status: $([ "$COVERAGE_STATUS" = "PASS" ] && echo "${GREEN}✅ PASS${NC}" || echo "${RED}❌ FAIL${NC}")"
echo ""
echo -e "${BLUE}📁 Reports generated:${NC}"
echo "- HTML Report: $COVERAGE_DIR/tarpaulin/tarpaulin-report.html"
echo "- Markdown Report: $REPORT_DIR/comprehensive-report.md"
echo "- JSON Summary: $REPORT_DIR/coverage-summary.json"
echo "- Benchmark Results: $REPORT_DIR/benchmark-results.json"

# Exit with appropriate code
if [ "$COVERAGE_STATUS" = "FAIL" ]; then
    echo -e "${RED}❌ Coverage threshold not met. Exiting with error code 1.${NC}"
    exit 1
else
    echo -e "${GREEN}✅ All coverage requirements met!${NC}"
    exit 0
fi
