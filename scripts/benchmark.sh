#!/bin/bash

# Benchmark script for StrellerMinds Smart Contracts
# Runs performance benchmarks on compiled contracts

set -e

echo "Starting benchmarks..."

# Define color codes
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Get the directory where the script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

echo -e "${BLUE}=== StrellerMinds Contracts Benchmark ===${NC}"
echo "Project Root: $PROJECT_ROOT"

# Array to store results
declare -a CONTRACTS

# Find all contracts with a Cargo.toml
cd "$PROJECT_ROOT/contracts"
for contract_dir in */; do
    contract_name="${contract_dir%/}"
    if [ -f "$contract_dir/Cargo.toml" ]; then
        CONTRACTS+=("$contract_name")
    fi
done

echo ""
echo -e "${YELLOW}Found contracts:${NC}"
for contract in "${CONTRACTS[@]}"; do
    echo "  - $contract"
done
echo ""

# Build and measure each contract
for contract in "${CONTRACTS[@]}"; do
    echo -e "${BLUE}Processing: $contract${NC}"
    cd "$PROJECT_ROOT/contracts/$contract"
    
    # Get WASM file size
    cargo build --target wasm32-unknown-unknown --release 2>/dev/null || true
    
    WASM_FILE="target/wasm32-unknown-unknown/release/${contract}.wasm"
    if [ -f "$WASM_FILE" ]; then
        SIZE=$(stat -f%z "$WASM_FILE" 2>/dev/null || stat -c%s "$WASM_FILE" 2>/dev/null || echo "0")
        SIZE_KB=$((SIZE / 1024))
        echo -e "${GREEN}✓ $contract: ${SIZE_KB}KB${NC}"
    else
        echo -e "${YELLOW}⊘ $contract: WASM file not found${NC}"
    fi
    echo ""
done

echo -e "${GREEN}Benchmarks completed!${NC}"
