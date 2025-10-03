#!/bin/bash
set -e


# Color codes
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Build all contracts
echo -e "${YELLOW}Building all contracts...${NC}"

# Check for required tools
if ! command -v cargo &> /dev/null; then
  echo -e "${RED}Error: 'cargo' not found. Please install Rust: https://www.rust-lang.org/tools/install${NC}"
  exit 1
fi
if ! command -v soroban &> /dev/null; then
  echo -e "${RED}Error: 'soroban' CLI not found. Please install Soroban CLI: https://soroban.stellar.org/docs/getting-started/installation${NC}"
  exit 1
fi
if ! command -v wasm-opt &> /dev/null; then
  echo -e "${YELLOW}Warning: 'wasm-opt' not found. Fallback optimization will not be available. Install Binaryen: https://github.com/WebAssembly/binaryen${NC}"
fi
# Create target directory if it doesn't exist

mkdir -p target/wasm32-unknown-unknown/release

# Build each contract
echo "Build completed successfully!"

success_contracts=()
failed_contracts=()

for contract in contracts/*/; do
  contract_name=$(basename "$contract")
  echo -e "${YELLOW}Building $contract_name contract...${NC}"
  start_time=$(date +%s)

  if cargo build --target wasm32-unknown-unknown --release -p "$contract_name"; then
    if [ -f "target/wasm32-unknown-unknown/release/$contract_name.wasm" ]; then
      echo -e "${YELLOW}Optimizing $contract_name.wasm...${NC}"
      if soroban contract optimize --wasm "target/wasm32-unknown-unknown/release/$contract_name.wasm" --wasm-out "target/wasm32-unknown-unknown/release/$contract_name.optimized.wasm"; then
        echo -e "${GREEN}Optimization succeeded for $contract_name${NC}"
        success_contracts+=("$contract_name")
      else
        echo -e "${YELLOW}Warning: 'soroban contract optimize' failed for $contract_name. Attempting fallback with 'wasm-opt -Oz'...${NC}"
        if command -v wasm-opt &> /dev/null; then
          if wasm-opt -Oz "target/wasm32-unknown-unknown/release/$contract_name.wasm" -o "target/wasm32-unknown-unknown/release/$contract_name.optimized.wasm"; then
            echo -e "${GREEN}Fallback optimization succeeded for $contract_name${NC}"
            success_contracts+=("$contract_name")
          else
            echo -e "${RED}Error: Both 'soroban contract optimize' and 'wasm-opt -Oz' failed for $contract_name. Please check your WASM file and tool installations.${NC}"
            failed_contracts+=("$contract_name")
          fi
        else
          echo -e "${RED}Error: 'wasm-opt' not found. Please install Binaryen (https://github.com/WebAssembly/binaryen) for fallback optimization.${NC}"
          failed_contracts+=("$contract_name")
        fi
      fi
    else
      echo -e "${RED}Error: WASM file not found for $contract_name after build.${NC}"
      failed_contracts+=("$contract_name")
    fi
  else
    echo -e "${RED}Error: Build failed for $contract_name${NC}"
    failed_contracts+=("$contract_name")
  fi
  end_time=$(date +%s)
  duration=$((end_time - start_time))
  echo -e "${YELLOW}Time taken for $contract_name: ${duration}s${NC}"
done

# Print summary
echo -e "\n${GREEN}Build completed!${NC}"
if [ ${#success_contracts[@]} -gt 0 ]; then
  echo -e "${GREEN}Contracts built and optimized successfully:${NC} ${success_contracts[*]}"
fi
if [ ${#failed_contracts[@]} -gt 0 ]; then
  echo -e "${RED}Contracts with errors:${NC} ${failed_contracts[*]}"
  exit 4
fi
