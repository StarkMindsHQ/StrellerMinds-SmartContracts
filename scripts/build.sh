#!/bin/bash
set -e

# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Usage/help message
if [[ "$1" == "-h" || "$1" == "--help" ]]; then
  echo -e "${YELLOW}Usage: bash scripts/build.sh [contract_name] [--production]${NC}"
  echo "If contract_name is provided, only that contract will be built and optimized."
  echo "Use --production to use the production profile (max optimization, slower build)."
  exit 0
fi

PROFILE="release"
SPECIFIC_CONTRACT=""
for arg in "$@"; do
  if [ "$arg" == "--production" ]; then
    PROFILE="production"
  elif [ -z "$SPECIFIC_CONTRACT" ]; then
    SPECIFIC_CONTRACT="$arg"
  fi
done

# Check for required tools
if ! command -v cargo &> /dev/null; then
  echo -e "${RED}Error: 'cargo' not found. Please install Rust: https://www.rust-lang.org/tools/install${NC}"
  exit 1
fi
if ! command -v soroban &> /dev/null; then
  echo -e "${RED}Error: 'soroban' CLI not found. Please install Soroban CLI: https://soroban.stellar.org/docs/getting-started/installation${NC}"
  exit 1
fi

# Print environment info
echo -e "${YELLOW}Environment Info:${NC}"
echo -n "Rust: " && rustc --version || echo "Not installed"
echo -n "Cargo: " && cargo --version || echo "Not installed"
echo -n "Soroban: " && soroban --version || echo "Not installed"

# Dependency Caching (sccache)
if command -v sccache &> /dev/null; then
  export RUSTC_WRAPPER=sccache
  echo -e "${GREEN}sccache detected and enabled for faster recompilation.${NC}"
fi

# Create target directory if it doesn't exist
mkdir -p target/wasm32-unknown-unknown/$PROFILE

# Logging setup
LOGFILE="build.log"
echo "--- Build started at $(date) ---" > "$LOGFILE"

START_TOTAL=$(date +%s)

# Build contracts
if [ -n "$SPECIFIC_CONTRACT" ]; then
  echo -e "${YELLOW}Building contract: $SPECIFIC_CONTRACT (Profile: $PROFILE)...${NC}"
  cargo build --target wasm32-unknown-unknown --profile "$PROFILE" -p "$SPECIFIC_CONTRACT" 2>&1 | tee -a "$LOGFILE"
else
  echo -e "${YELLOW}Building all contracts in workspace (Profile: $PROFILE)...${NC}"
  # Exclude non-contract workspace members if necessary, but cargo build --workspace --target wasm32-unknown-unknown 
  # will only build CDYLIB crates for that target anyway.
  cargo build --workspace --target wasm32-unknown-unknown --profile "$PROFILE" 2>&1 | tee -a "$LOGFILE"
fi

BUILD_END=$(date +%s)
echo -e "${GREEN}Cargo build completed in $((BUILD_END - START_TOTAL))s${NC}"

# Optimize WASM files in parallel
echo -e "${YELLOW}Optimizing WASM files in parallel...${NC}"
OPTIMIZE_START=$(date +%s)

success_contracts=()
failed_contracts=()

# Function to optimize a single contract
optimize_contract() {
  local wasm_path=$1
  local contract_name=$(basename "$wasm_path" .wasm)
  local optimized_wasm_path="target/wasm32-unknown-unknown/$PROFILE/$contract_name.optimized.wasm"
  
  echo -e "${YELLOW}Optimizing $contract_name...${NC}"
  if soroban contract optimize --wasm "$wasm_path" --wasm-out "$optimized_wasm_path" >> "$LOGFILE" 2>&1; then
    echo -e "${GREEN}✓ $contract_name optimized${NC}"
    return 0
  else
    # Fallback to wasm-opt if available
    if command -v wasm-opt &> /dev/null; then
      if wasm-opt -Oz "$wasm_path" -o "$optimized_wasm_path" >> "$LOGFILE" 2>&1; then
        echo -e "${GREEN}✓ $contract_name optimized (via wasm-opt)${NC}"
        return 0
      fi
    fi
    echo -e "${RED}✗ $contract_name optimization failed${NC}"
    return 1
  fi
}

# Find all generated WASM files (excluding already optimized ones)
wasm_files=$(find target/wasm32-unknown-unknown/$PROFILE -maxdepth 1 -name "*.wasm" ! -name "*.optimized.wasm")

# Run optimizations in parallel
pids=()
for wasm in $wasm_files; do
  optimize_contract "$wasm" &
  pids+=($!)
done

# Wait for all background jobs
exit_code=0
for pid in "${pids[@]}"; do
  if ! wait "$pid"; then
    exit_code=1
  fi
done

OPTIMIZE_END=$(date +%s)
TOTAL_END=$(date +%s)

echo -e "\n${GREEN}Optimization completed in $((OPTIMIZE_END - OPTIMIZE_START))s${NC}"
echo -e "${GREEN}Total time: $((TOTAL_END - START_TOTAL))s${NC}"

if [ $exit_code -ne 0 ]; then
  echo -e "${RED}Some optimizations failed. See $LOGFILE for details.${NC}"
  exit 4
fi

echo -e "${GREEN}Build and optimization successful!${NC}"
