#!/bin/bash
# Deployment Performance Metrics & Verification Script
# Monitors cold start times and deployment performance

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
METRICS_DIR="$PROJECT_ROOT/target/metrics"
METRICS_FILE="$METRICS_DIR/deployment_metrics.json"
HISTORY_FILE="$METRICS_DIR/deployment_history.csv"

mkdir -p "$METRICS_DIR"

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${GREEN}=== Deployment Performance Monitor ===${NC}"
echo ""

# Initialize history file if it doesn't exist
if [ ! -f "$HISTORY_FILE" ]; then
    echo "timestamp,contract,network,wasm_size_bytes,deploy_time_ms,cold_start_ms,status" > "$HISTORY_FILE"
fi

# Function to measure deployment time
measure_deployment() {
    local contract_name=$1
    local network=$2
    local wasm_file="$PROJECT_ROOT/target/optimized/${contract_name}.wasm"
    
    if [ ! -f "$wasm_file" ]; then
        echo -e "${YELLOW}Warning: Optimized WASM not found for $contract_name${NC}"
        echo -e "${YELLOW}Running optimization first...${NC}"
        bash "$SCRIPT_DIR/optimize_wasm.sh"
    fi
    
    local wasm_size=$(stat -f%z "$wasm_file" 2>/dev/null || stat -c%s "$wasm_file" 2>/dev/null)
    echo "Contract: $contract_name"
    echo "  WASM size: $wasm_size bytes"
    
    # Measure deployment time
    echo "  Measuring deployment time..."
    local start_time=$(date +%s%N)
    
    # Simulate deployment (replace with actual soroban deploy command)
    # soroban contract deploy --wasm "$wasm_file" --network "$network"
    sleep 0.1  # Placeholder for actual deployment
    
    local end_time=$(date +%s%N)
    local deploy_time_ms=$(( (end_time - start_time) / 1000000 ))
    
    echo "  Deployment time: ${deploy_time_ms}ms"
    
    # Estimate cold start time based on WASM size
    # Rule of thumb: ~10ms per KB for WASM loading
    local cold_start_ms=$(( (wasm_size * 10) / 1024 ))
    
    # Add overhead for initialization
    cold_start_ms=$(( cold_start_ms + 100 ))
    
    echo "  Estimated cold start: ${cold_start_ms}ms"
    
    # Verify performance target
    local status="PASS"
    if [ "$cold_start_ms" -gt 500 ]; then
        status="FAIL"
        echo -e "  ${RED}✗ Exceeds 500ms target${NC}"
    else
        echo -e "  ${GREEN}✓ Within 500ms target${NC}"
    fi
    
    # Record metrics
    local timestamp=$(date -u +"%Y-%m-%dT%H:%M:%SZ")
    echo "$timestamp,$contract_name,$network,$wasm_size,$deploy_time_ms,$cold_start_ms,$status" >> "$HISTORY_FILE"
    
    echo ""
}

# Generate JSON metrics report
generate_json_report() {
    echo "{" > "$METRICS_FILE"
    echo "  \"generated_at\": \"$(date -u +"%Y-%m-%dT%H:%M:%SZ")\"," >> "$METRICS_FILE"
    echo "  \"performance_targets\": {" >> "$METRICS_FILE"
    echo "    \"cold_start_max_ms\": 500," >> "$METRICS_FILE"
    echo "    \"wasm_size_max_bytes\": 131072" >> "$METRICS_FILE"
    echo "  }," >> "$METRICS_FILE"
    echo "  \"contracts\": [" >> "$METRICS_FILE"
    
    local first=true
    while IFS=',' read -r timestamp contract network wasm_size deploy_time cold_start status; do
        # Skip header
        if [ "$timestamp" = "timestamp" ]; then
            continue
        fi
        
        if [ "$first" = true ]; then
            first=false
        else
            echo "," >> "$METRICS_FILE"
        fi
        
        cat >> "$METRICS_FILE" << EOF
    {
      "contract": "$contract",
      "network": "$network",
      "wasm_size_bytes": $wasm_size,
      "deploy_time_ms": $deploy_time,
      "cold_start_ms": $cold_start,
      "status": "$status",
      "timestamp": "$timestamp"
    }
EOF
    done < "$HISTORY_FILE"
    
    echo "" >> "$METRICS_FILE"
    echo "  ]" >> "$METRICS_FILE"
    echo "}" >> "$METRICS_FILE"
    
    echo -e "${GREEN}Metrics saved to: $METRICS_FILE${NC}"
}

# Main execution
echo "Enter network (testnet/mainnet):"
read -r network
network=${network:-testnet}

echo ""
echo "Testing deployment performance for all contracts..."
echo ""

contracts=("certificate" "analytics" "token" "shared" "proxy")

for contract in "${contracts[@]}"; do
    measure_deployment "$contract" "$network"
done

# Generate report
echo ""
generate_json_report

# Show summary
echo ""
echo -e "${GREEN}=== Performance Summary ===${NC}"
echo ""

total=0
passed=0
failed=0

while IFS=',' read -r timestamp contract network wasm_size deploy_time cold_start status; do
    if [ "$timestamp" = "timestamp" ]; then
        continue
    fi
    total=$((total + 1))
    if [ "$status" = "PASS" ]; then
        passed=$((passed + 1))
    else
        failed=$((failed + 1))
    fi
done < "$HISTORY_FILE"

echo "Total contracts tested: $total"
echo "Passed (< 500ms): $passed"
echo "Failed (> 500ms): $failed"
echo ""

if [ "$failed" -gt 0 ]; then
    echo -e "${RED}Warning: $failed contracts exceed performance targets${NC}"
    echo "Consider:"
    echo "  1. Running WASM optimization"
    echo "  2. Enabling provisioned concurrency"
    echo "  3. Reviewing code splitting strategy"
    exit 1
else
    echo -e "${GREEN}All contracts meet performance targets!${NC}"
fi
