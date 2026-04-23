#!/bin/bash

# Test Coverage Dashboard Script for StrellerMinds Smart Contracts
# This script generates a comprehensive coverage dashboard

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
COVERAGE_DIR="target/coverage"
REPORT_DIR="target/coverage-reports"
DASHBOARD_DIR="target/coverage-dashboard"
MIN_COVERAGE=80

# Create directories
mkdir -p "$DASHBOARD_DIR"
mkdir -p "$REPORT_DIR"

echo -e "${BLUE}📊 StrellerMinds Test Coverage Dashboard${NC}"
echo "=================================================="

# Function to check if command exists
command_exists() {
    command -v "$1" >/dev/null 2>&1
}

# Function to extract coverage from lcov file
extract_coverage() {
    local lcov_file="$1"
    if [ -f "$lcov_file" ] && command_exists lcov; then
        lcov --summary "$lcov_file" 2>/dev/null | grep "lines......:" | awk '{print $2}' | sed 's/%//' || echo "0"
    else
        echo "0"
    fi
}

# Function to get contract coverage
get_contract_coverage() {
    local contract="$1"
    local lcov_file="$COVERAGE_DIR/${contract}-coverage.lcov"
    
    if [ -f "$lcov_file" ]; then
        extract_coverage "$lcov_file"
    else
        echo "0"
    fi
}

# Function to generate coverage bar
generate_coverage_bar() {
    local coverage="$1"
    local width=50
    local filled=$((coverage * width / 100))
    
    # Create the bar
    local bar=""
    for ((i=0; i<width; i++)); do
        if [ $i -lt $filled ]; then
            if [ $coverage -ge $MIN_COVERAGE ]; then
                bar="${bar}█"
            else
                bar="${bar}▓"
            fi
        else
            bar="${bar}░"
        fi
    done
    
    echo "$bar"
}

# Function to determine coverage status
coverage_status() {
    local coverage="$1"
    
    if [ "${coverage%.*}" -ge $MIN_COVERAGE ]; then
        echo -e "${GREEN}✅ PASS${NC}"
    elif [ "${coverage%.*}" -ge 60 ]; then
        echo -e "${YELLOW}⚠️  WARN${NC}"
    else
        echo -e "${RED}❌ FAIL${NC}"
    fi
}

# Function to generate contract summary
generate_contract_summary() {
    echo -e "${CYAN}📋 Contract Coverage Summary${NC}"
    echo "========================="
    
    contracts=("assessment" "community" "certificate" "analytics" "shared")
    
    printf "%-15s %8s %s %s\n" "Contract" "Coverage" "Status" "Progress"
    printf "%-15s %8s %s %s\n" "--------" "--------" "------" "--------"
    
    total_coverage=0
    contract_count=0
    
    for contract in "${contracts[@]}"; do
        coverage=$(get_contract_coverage "$contract")
        status=$(coverage_status "$coverage")
        bar=$(generate_coverage_bar "$coverage")
        
        printf "%-15s %8.1f%% %s %s\n" "$contract" "$coverage" "$status" "$bar"
        
        total_coverage=$(echo "$total_coverage + $coverage" | bc -l)
        contract_count=$((contract_count + 1))
    done
    
    # Calculate average
    if [ $contract_count -gt 0 ]; then
        average_coverage=$(echo "scale=1; $total_coverage / $contract_count" | bc -l)
        avg_status=$(coverage_status "$average_coverage")
        avg_bar=$(generate_coverage_bar "$average_coverage")
        
        echo ""
        printf "%-15s %8.1f%% %s %s\n" "AVERAGE" "$average_coverage" "$avg_status" "$avg_bar"
    fi
}

# Function to generate test type coverage
generate_test_type_coverage() {
    echo ""
    echo -e "${PURPLE}🧪 Test Type Coverage${NC}"
    echo "====================="
    
    # Check different test types
    test_types=(
        "Unit Tests:unit-test"
        "Integration Tests:integration-test"
        "Property Tests:property-test"
        "Edge Case Tests:edge-case-test"
        "Error Tests:error-test"
        "Performance Tests:performance-test"
    )
    
    for test_type_info in "${test_types[@]}"; do
        IFS=':' read -r display_name test_name <<< "$test_type_info"
        
        # Check if test results exist
        if [ -f "$REPORT_DIR/${test_name}-results.json" ]; then
            if command_exists jq; then
                coverage=$(jq -r '.coverage // 0' "$REPORT_DIR/${test_name}-results.json" 2>/dev/null || echo "0")
                status=$(coverage_status "$coverage")
                bar=$(generate_coverage_bar "$coverage")
                
                printf "%-20s %8.1f%% %s %s\n" "$display_name" "$coverage" "$status" "$bar"
            else
                printf "%-20s %8s %s\n" "$display_name" "N/A" "jq not available"
            fi
        else
            printf "%-20s %8s %s\n" "$display_name" "0.0%" "❌ NOT RUN"
        fi
    done
}

# Function to generate coverage trends
generate_coverage_trends() {
    echo ""
    echo -e "${BLUE}📈 Coverage Trends${NC}"
    echo "=================="
    
    # Look for historical coverage data
    trend_file="$REPORT_DIR/coverage-trends.json"
    
    if [ -f "$trend_file" ] && command_exists jq; then
        echo "Recent coverage history:"
        
        # Get last 7 entries
        jq -r '.trends[-7:][] | "- \(.date): \(.coverage)%"' "$trend_file" 2>/dev/null || echo "No trend data available"
        
        # Calculate trend
        latest=$(jq -r '.trends[-1].coverage // 0' "$trend_file" 2>/dev/null || echo "0")
        previous=$(jq -r '.trends[-2].coverage // 0' "$trend_file" 2>/dev/null || echo "0")
        
        if [ "$latest" != "0" ] && [ "$previous" != "0" ]; then
            change=$(echo "scale=1; $latest - $previous" | bc -l)
            
            if (( $(echo "$change > 0" | bc -l) )); then
                echo -e "${GREEN}📈 Trend: +${change}% (improving)${NC}"
            elif (( $(echo "$change < 0" | bc -l) )); then
                echo -e "${RED}📉 Trend: ${change}% (declining)${NC}"
            else
                echo -e "${YELLOW}➡️  Trend: No change${NC}"
            fi
        fi
    else
        echo "No historical coverage data available"
        echo "Run coverage analysis regularly to build trend data"
    fi
}

# Function to generate quality metrics
generate_quality_metrics() {
    echo ""
    echo -e "${CYAN}🎯 Quality Metrics${NC}"
    echo "=================="
    
    # Check various quality indicators
    metrics=()
    
    # Test execution time
    if [ -f "$REPORT_DIR/test-execution-time.txt" ]; then
        exec_time=$(cat "$REPORT_DIR/test-execution-time.txt")
        metrics+=("Test Execution Time:$exec_time seconds")
    fi
    
    # Number of tests
    if command_exists cargo; then
        test_count=$(cargo test --all-features --no-run --quiet 2>/dev/null | grep -c "test" || echo "0")
        metrics+=("Total Tests:$test_count")
    fi
    
    # Code complexity (if available)
    if command_exists cargo-audit; then
        security_issues=$(cargo audit 2>/dev/null | grep -c "Vulnerability" || echo "0")
        metrics+=("Security Issues:$security_issues")
    fi
    
    # Display metrics
    for metric_info in "${metrics[@]}"; do
        IFS=':' read -r label value <<< "$metric_info"
        printf "%-25s %s\n" "$label" "$value"
    done
}

# Function to generate recommendations
generate_recommendations() {
    echo ""
    echo -e "${YELLOW}💡 Recommendations${NC}"
    echo "=================="
    
    # Analyze coverage and provide recommendations
    recommendations=()
    
    # Check overall coverage
    if [ -f "$COVERAGE_DIR/workspace-summary.json" ] && command_exists jq; then
        overall_coverage=$(jq -r '.percentage // 0' "$COVERAGE_DIR/workspace-summary.json" 2>/dev/null || echo "0")
        
        if [ "${overall_coverage%.*}" -lt $MIN_COVERAGE ]; then
            recommendations+=("❌ Overall coverage ($overall_coverage%) is below threshold ($MIN_COVERAGE%)")
            recommendations+=("   → Focus on increasing test coverage for low-coverage areas")
        else
            recommendations+=("✅ Overall coverage ($overall_coverage%) meets threshold")
        fi
    fi
    
    # Check individual contracts
    contracts=("assessment" "community" "certificate" "analytics" "shared")
    for contract in "${contracts[@]}"; do
        coverage=$(get_contract_coverage "$contract")
        
        if [ "${coverage%.*}" -lt $MIN_COVERAGE ]; then
            recommendations+=("❌ $contract contract coverage ($coverage%) is below threshold")
            recommendations+=("   → Add more unit tests for $contract functions")
        fi
    done
    
    # Check for missing test types
    missing_tests=()
    test_types=("unit-test" "integration-test" "property-test" "edge-case-test")
    
    for test_type in "${test_types[@]}"; do
        if [ ! -f "$REPORT_DIR/${test_type}-results.json" ]; then
            missing_tests+=("$test_type")
        fi
    done
    
    if [ ${#missing_tests[@]} -gt 0 ]; then
        recommendations+=("⚠️  Missing test types: ${missing_tests[*]}")
        recommendations+=("   → Implement comprehensive test suite for all test types")
    fi
    
    # Performance recommendations
    if [ -f "$REPORT_DIR/performance-regression.json" ] && command_exists jq; then
        regression=$(jq -r '.has_regression // false' "$REPORT_DIR/performance-regression.json" 2>/dev/null || echo "false")
        
        if [ "$regression" = "true" ]; then
            recommendations+=("⚠️  Performance regression detected")
            recommendations+=("   → Review recent changes for performance impact")
        fi
    fi
    
    # Display recommendations
    if [ ${#recommendations[@]} -eq 0 ]; then
        echo -e "${GREEN}✅ All quality metrics look good!${NC}"
        echo "Keep up the great work!"
    else
        for recommendation in "${recommendations[@]}"; do
            echo "$recommendation"
        done
    fi
}

# Function to generate HTML dashboard
generate_html_dashboard() {
    echo ""
    echo -e "${BLUE}🌐 Generating HTML Dashboard...${NC}"
    
    html_file="$DASHBOARD_DIR/index.html"
    
    cat > "$html_file" << 'EOF'
<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>StrellerMinds Test Coverage Dashboard</title>
    <style>
        body {
            font-family: 'Segoe UI', Tahoma, Geneva, Verdana, sans-serif;
            margin: 0;
            padding: 20px;
            background-color: #f5f5f5;
            color: #333;
        }
        .container {
            max-width: 1200px;
            margin: 0 auto;
            background-color: white;
            padding: 30px;
            border-radius: 10px;
            box-shadow: 0 2px 10px rgba(0,0,0,0.1);
        }
        .header {
            text-align: center;
            margin-bottom: 40px;
            border-bottom: 2px solid #e0e0e0;
            padding-bottom: 20px;
        }
        .header h1 {
            color: #2c3e50;
            margin: 0;
            font-size: 2.5em;
        }
        .header p {
            color: #7f8c8d;
            margin: 10px 0 0 0;
            font-size: 1.2em;
        }
        .section {
            margin-bottom: 30px;
            padding: 20px;
            border: 1px solid #e0e0e0;
            border-radius: 8px;
            background-color: #fafafa;
        }
        .section h2 {
            color: #2c3e50;
            margin-top: 0;
            border-bottom: 1px solid #ddd;
            padding-bottom: 10px;
        }
        .coverage-table {
            width: 100%;
            border-collapse: collapse;
            margin-top: 15px;
        }
        .coverage-table th,
        .coverage-table td {
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }
        .coverage-table th {
            background-color: #3498db;
            color: white;
            font-weight: bold;
        }
        .coverage-bar {
            width: 100%;
            height: 20px;
            background-color: #ecf0f1;
            border-radius: 10px;
            overflow: hidden;
        }
        .coverage-fill {
            height: 100%;
            transition: width 0.3s ease;
        }
        .coverage-high { background-color: #27ae60; }
        .coverage-medium { background-color: #f39c12; }
        .coverage-low { background-color: #e74c3c; }
        .status-pass { color: #27ae60; font-weight: bold; }
        .status-warn { color: #f39c12; font-weight: bold; }
        .status-fail { color: #e74c3c; font-weight: bold; }
        .metrics-grid {
            display: grid;
            grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
            gap: 20px;
            margin-top: 15px;
        }
        .metric-card {
            background-color: white;
            padding: 20px;
            border-radius: 8px;
            border: 1px solid #ddd;
            text-align: center;
        }
        .metric-value {
            font-size: 2em;
            font-weight: bold;
            color: #2c3e50;
        }
        .metric-label {
            color: #7f8c8d;
            margin-top: 5px;
        }
        .recommendations {
            background-color: #fff3cd;
            border: 1px solid #ffeaa7;
            border-radius: 8px;
            padding: 15px;
        }
        .recommendations h3 {
            color: #856404;
            margin-top: 0;
        }
        .recommendation-item {
            margin: 10px 0;
            padding: 10px;
            background-color: white;
            border-radius: 5px;
            border-left: 4px solid #f39c12;
        }
        .last-updated {
            text-align: center;
            color: #7f8c8d;
            margin-top: 30px;
            font-size: 0.9em;
        }
    </style>
</head>
<body>
    <div class="container">
        <div class="header">
            <h1>🧪 StrellerMinds Test Coverage Dashboard</h1>
            <p>Comprehensive test coverage analysis and quality metrics</p>
        </div>
        
        <div class="section">
            <h2>📊 Contract Coverage Summary</h2>
            <table class="coverage-table" id="contract-table">
                <thead>
                    <tr>
                        <th>Contract</th>
                        <th>Coverage</th>
                        <th>Status</th>
                        <th>Progress</th>
                    </tr>
                </thead>
                <tbody>
                    <!-- Contract data will be inserted here -->
                </tbody>
            </table>
        </div>
        
        <div class="section">
            <h2>🎯 Quality Metrics</h2>
            <div class="metrics-grid" id="metrics-grid">
                <!-- Metrics will be inserted here -->
            </div>
        </div>
        
        <div class="section">
            <h2>💡 Recommendations</h2>
            <div class="recommendations" id="recommendations">
                <!-- Recommendations will be inserted here -->
            </div>
        </div>
        
        <div class="last-updated">
            Last updated: <span id="last-updated-time"></span>
        </div>
    </div>
    
    <script>
        // This would be populated by the script with actual data
        // For now, showing static structure
        document.getElementById('last-updated-time').textContent = new Date().toLocaleString();
    </script>
</body>
</html>
EOF
    
    echo -e "${GREEN}✅ HTML dashboard generated: $html_file${NC}"
}

# Function to update trend data
update_trend_data() {
    if [ -f "$COVERAGE_DIR/workspace-summary.json" ] && command_exists jq; then
        current_coverage=$(jq -r '.percentage // 0' "$COVERAGE_DIR/workspace-summary.json" 2>/dev/null || echo "0")
        current_date=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
        
        trend_file="$REPORT_DIR/coverage-trends.json"
        
        # Create trend file if it doesn't exist
        if [ ! -f "$trend_file" ]; then
            echo '{"trends": []}' > "$trend_file"
        fi
        
        # Add new trend entry
        temp_file=$(mktemp)
        jq --arg date "$current_date" --arg coverage "$current_coverage" \
            '.trends += [{"date": $date, "coverage": ($coverage | tonumber)}]' \
            "$trend_file" > "$temp_file" && mv "$temp_file" "$trend_file"
        
        # Keep only last 30 entries
        jq '.trends = (.trends | length > 30) ? (.trends[-30:]) : .trends' "$trend_file" > "$temp_file" && mv "$temp_file" "$trend_file"
    fi
}

# Main execution
main() {
    echo "Generating comprehensive coverage dashboard..."
    
    # Update trend data
    update_trend_data
    
    # Generate all sections
    generate_contract_summary
    generate_test_type_coverage
    generate_coverage_trends
    generate_quality_metrics
    generate_recommendations
    generate_html_dashboard
    
    echo ""
    echo -e "${GREEN}🎉 Dashboard generation complete!${NC}"
    echo ""
    echo "Files generated:"
    echo "- HTML Dashboard: $DASHBOARD_DIR/index.html"
    echo "- Coverage Reports: $REPORT_DIR/"
    echo "- Coverage Data: $COVERAGE_DIR/"
    echo ""
    echo "To view the dashboard:"
    if command -v xdg-open >/dev/null 2>&1; then
        echo "  xdg-open $DASHBOARD_DIR/index.html"
    elif command -v open >/dev/null 2>&1; then
        echo "  open $DASHBOARD_DIR/index.html"
    else
        echo "  Open $DASHBOARD_DIR/index.html in your browser"
    fi
}

# Parse command line arguments
case "${1:-}" in
    --contracts-only)
        generate_contract_summary
        ;;
    --trends-only)
        generate_coverage_trends
        ;;
    --recommendations-only)
        generate_recommendations
        ;;
    --html-only)
        generate_html_dashboard
        ;;
    --help|-h)
        echo "Usage: $0 [OPTION]"
        echo "Options:"
        echo "  --contracts-only     Show contract coverage only"
        echo "  --trends-only        Show coverage trends only"
        echo "  --recommendations-only  Show recommendations only"
        echo "  --html-only          Generate HTML dashboard only"
        echo "  --help, -h           Show this help message"
        echo ""
        echo "Default: Generate complete dashboard"
        ;;
    *)
        main
        ;;
esac
