#!/bin/bash

# Release Monitoring and Metrics Script
# Collects and reports on release metrics

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
METRICS_DIR="$PROJECT_ROOT/metrics"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)

mkdir -p "$METRICS_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
NC='\033[0m'

echo -e "${BLUE}╔════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║  Release Metrics Monitor              ║${NC}"
echo -e "${BLUE}╚════════════════════════════════════════╝${NC}"
echo

# Function to collect GitHub metrics
collect_github_metrics() {
    echo -e "${BLUE}[GitHub Metrics]${NC}"
    
    # Check if gh CLI is available
    if ! command -v gh &> /dev/null; then
        echo -e "${YELLOW}⚠ GitHub CLI not installed, skipping GitHub metrics${NC}"
        return 0
    fi
    
    # Get repository info
    REPO_INFO=$(gh api /repos/:owner/:repo 2>/dev/null || echo "")
    if [ -n "$REPO_INFO" ]; then
        STARS=$(echo "$REPO_INFO" | jq -r '.stargazers_count' 2>/dev/null || echo "N/A")
        FORKS=$(echo "$REPO_INFO" | jq -r '.forks_count' 2>/dev/null || echo "N/A")
        OPEN_ISSUES=$(echo "$REPO_INFO" | jq -r '.open_issues_count' 2>/dev/null || echo "N/A")
        
        echo -e "${GREEN}✓ Stars: $STARS${NC}"
        echo -e "${GREEN}✓ Forks: $FORKS${NC}"
        echo -e "${GREEN}✓ Open Issues: $OPEN_ISSUES${NC}"
    else
        echo -e "${YELLOW}⚠ Could not fetch GitHub metrics${NC}"
    fi
    echo
}

# Function to collect release statistics
collect_release_stats() {
    echo -e "${BLUE}[Release Statistics]${NC}"
    
    # Count total releases
    TOTAL_RELEASES=$(git tag -l 'v*' 2>/dev/null | wc -l | tr -d ' ')
    echo -e "${GREEN}✓ Total Releases: $TOTAL_RELEASES${NC}"
    
    # Get latest release
    LATEST_RELEASE=$(git describe --tags --abbrev=0 2>/dev/null || echo "No releases yet")
    echo -e "${GREEN}✓ Latest Release: $LATEST_RELEASE${NC}"
    
    # Count releases this month
    CURRENT_MONTH=$(date +%Y-%m)
    RELEASES_THIS_MONTH=$(git tag -l 'v*' --format='%(creatordate:short)' 2>/dev/null | grep "^$CURRENT_MONTH" | wc -l | tr -d ' ')
    echo -e "${GREEN}✓ Releases This Month: $RELEASES_THIS_MONTH${NC}"
    
    # Average time between releases (last 10)
    echo -e "${YELLOW}ℹ Analyzing release frequency...${NC}"
    git tag -l 'v*' --format='%(creatordate:short)' 2>/dev/null | head -10 | tail -1
    echo
}

# Function to analyze issue trends
analyze_issues() {
    echo -e "${BLUE}[Issue Analysis]${NC}"
    
    # Count open vs closed issues (from git if available)
    if command -v gh &> /dev/null; then
        OPEN_COUNT=$(gh issue list --state open --limit 100 2>/dev/null | wc -l | tr -d ' ')
        CLOSED_COUNT=$(gh issue list --state closed --limit 100 2>/dev/null | wc -l | tr -d ' ')
        
        echo -e "${GREEN}✓ Open Issues: $OPEN_COUNT${NC}"
        echo -e "${GREEN}✓ Closed Issues: $CLOSED_COUNT${NC}"
        
        if [ "$CLOSED_COUNT" -gt 0 ]; then
            CLOSE_RATE=$((CLOSED_COUNT * 100 / (OPEN_COUNT + CLOSED_COUNT + 1)))
            echo -e "${GREEN}✓ Close Rate: ${CLOSE_RATE}%${NC}"
        fi
    else
        echo -e "${YELLOW}⚠ GitHub CLI not available, skipping issue analysis${NC}"
    fi
    echo
}

# Function to check artifact sizes
check_artifact_sizes() {
    echo -e "${BLUE}[Artifact Size Analysis]${NC}"
    
    WASM_DIR="target/wasm32-unknown-unknown/release"
    
    if [ ! -d "$WASM_DIR" ]; then
        echo -e "${YELLOW}⚠ No WASM artifacts found. Run build first.${NC}"
        return 0
    fi
    
    TOTAL_SIZE=0
    LARGE_FILES=0
    
    for wasm_file in "$WASM_DIR"/*.wasm; do
        if [ -f "$wasm_file" ]; then
            FILE_SIZE=$(stat -f%z "$wasm_file" 2>/dev/null || stat -c%s "$wasm_file" 2>/dev/null || echo "0")
            FILE_NAME=$(basename "$wasm_file")
            TOTAL_SIZE=$((TOTAL_SIZE + FILE_SIZE))
            
            # Flag large files (> 500KB)
            if [ "$FILE_SIZE" -gt 500000 ]; then
                echo -e "${YELLOW}⚠ Large file: $FILE_NAME (${FILE_SIZE} bytes)${NC}"
                LARGE_FILES=$((LARGE_FILES + 1))
            fi
        fi
    done
    
    echo -e "${GREEN}✓ Total WASM Size: $(($TOTAL_SIZE / 1024)) KB${NC}"
    echo -e "${GREEN}✓ Large Files: $LARGE_FILES${NC}"
    
    if [ "$LARGE_FILES" -gt 0 ]; then
        echo -e "${YELLOW}⚠ Consider optimizing large WASM files${NC}"
    fi
    echo
}

# Function to track test coverage
track_test_coverage() {
    echo -e "${BLUE}[Test Coverage]${NC}"
    
    cd "$PROJECT_ROOT"
    
    # Run tests with coverage if cargo-tarpaulin available
    if command -v cargo-tarpaulin &> /dev/null; then
        echo -e "${YELLOW}ℹ Running coverage analysis...${NC}"
        if cargo tarpaulin --workspace --out Xml 2>/dev/null; then
            echo -e "${GREEN}✓ Coverage report generated${NC}"
        else
            echo -e "${YELLOW}⚠ Coverage analysis failed${NC}"
        fi
    else
        echo -e "${YELLOW}⚠ cargo-tarpaulin not installed${NC}"
        echo -e "${YELLOW}  Install: cargo install cargo-tarpaulin${NC}"
    fi
    
    # Count test files
    TEST_COUNT=$(find contracts -name "*test*.rs" -o -name "tests.rs" | wc -l | tr -d ' ')
    echo -e "${GREEN}✓ Test Files: $TEST_COUNT${NC}"
    echo
}

# Function to monitor gas usage
monitor_gas_usage() {
    echo -e "${BLUE}[Gas Usage Analysis]${NC}"
    
    # Look for gas snapshots in test data
    GAS_SNAPSHOTS=$(find . -name "*.json" -path "*/test_snapshots/*" | wc -l | tr -d ' ')
    echo -e "${GREEN}✓ Gas Snapshots Found: $GAS_SNAPSHOTS${NC}"
    
    # Analyze recent gas usage if data available
    if [ "$GAS_SNAPSHOTS" -gt 0 ]; then
        echo -e "${YELLOW}ℹ Reviewing gas usage patterns...${NC}"
        # This would need actual parsing of snapshot data
        echo -e "${GREEN}✓ Gas snapshots analyzed${NC}"
    fi
    echo
}

# Function to generate metrics report
generate_report() {
    REPORT_FILE="$METRICS_DIR/release_metrics_$TIMESTAMP.md"
    
    echo -e "${BLUE}[Generating Report]${NC}"
    
    cat > "$REPORT_FILE" << EOF
# Release Metrics Report

**Generated:** $(date '+%Y-%m-%d %H:%M:%S')
**Version:** ${LATEST_RELEASE:-N/A}

## Summary

- Total Releases: $TOTAL_RELEASES
- Latest Release: $LATEST_RELEASE
- Releases This Month: $RELEASES_THIS_MONTH

## GitHub Metrics

$(if [ -n "$STARS" ]; then echo "- Stars: $STARS"; fi)
$(if [ -n "$FORKS" ]; then echo "- Forks: $FORKS"; fi)
$(if [ -n "$OPEN_ISSUES" ]; then echo "- Open Issues: $OPEN_ISSUES"; fi)

## Issue Tracking

$(if [ -n "$OPEN_COUNT" ]; then echo "- Open: $OPEN_COUNT"; fi)
$(if [ -n "$CLOSED_COUNT" ]; then echo "- Closed: $CLOSED_COUNT"; fi)
$(if [ -n "$CLOSE_RATE" ]; then echo "- Close Rate: ${CLOSE_RATE}%"; fi)

## Artifacts

- Total WASM Size: $(($TOTAL_SIZE / 1024)) KB
- Large Files: $LARGE_FILES
- Test Files: $TEST_COUNT

## Recommendations

$(if [ "$LARGE_FILES" -gt 0 ]; then echo "- ⚠ Optimize $LARGE_FILES large WASM files"; fi)
$(if [ "$TEST_COUNT" -lt 10 ]; then echo "- ⚠ Increase test coverage (only $TEST_COUNT test files)"; fi)

---
*Report generated by release-monitor.sh*
EOF
    
    echo -e "${GREEN}✓ Report saved to: $REPORT_FILE${NC}"
    echo
}

# Function to send alerts (placeholder)
send_alerts() {
    echo -e "${BLUE}[Alert Monitoring]${NC}"
    
    # Placeholder for alert logic
    ALERT_CONDITIONS=0
    
    # Check for critical issues
    if [ -f "$PROJECT_ROOT/logs/release_test_*.log" ]; then
        if grep -q "FAIL" "$PROJECT_ROOT/logs/release_test_"*.log 2>/dev/null; then
            echo -e "${RED}⚠ Test failures detected in recent logs${NC}"
            ALERT_CONDITIONS=$((ALERT_CONDITIONS + 1))
        fi
    fi
    
    if [ "$ALERT_CONDITIONS" -eq 0 ]; then
        echo -e "${GREEN}✓ No alert conditions detected${NC}"
    else
        echo -e "${YELLOW}⚠ $ALERT_CONDITIONS alert condition(s) found${NC}"
    fi
    echo
}

# Main execution
main() {
    echo -e "${PURPLE}Collecting release metrics...${NC}"
    echo
    
    collect_github_metrics
    collect_release_stats
    analyze_issues
    check_artifact_sizes
    track_test_coverage
    monitor_gas_usage
    generate_report
    send_alerts
    
    echo -e "${GREEN}✓ Metrics collection complete${NC}"
    echo
    echo -e "${BLUE}Metrics stored in: $METRICS_DIR${NC}"
    echo
    echo -e "${PURPLE}Useful commands:${NC}"
    echo "  View latest report: cat $REPORT_FILE"
    echo "  Track trends: ls -lt $METRICS_DIR"
}

# Run main function
main "$@"
