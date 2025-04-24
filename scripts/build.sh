#!/bin/bash
set -e

# Build all contracts
echo "Building all contracts..."

# Create target directory if it doesn't exist
mkdir -p target/wasm32-unknown-unknown/release

# Build each contract
for contract in contracts/*/; do
  contract_name=$(basename "$contract")
  echo "Building $contract_name contract..."
  
  cargo build --target wasm32-unknown-unknown --release -p "$contract_name"
  
  # Optimize the WASM file
  if [ -f "target/wasm32-unknown-unknown/release/$contract_name.wasm" ]; then
    echo "Optimizing $contract_name.wasm..."
    soroban contract optimize --wasm "target/wasm32-unknown-unknown/release/$contract_name.wasm" --wasm-out "target/wasm32-unknown-unknown/release/$contract_name.optimized.wasm"
  fi
done

echo "Build completed successfully!"
