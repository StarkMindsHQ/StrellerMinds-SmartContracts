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
      if ! soroban contract optimize --wasm "target/wasm32-unknown-unknown/release/$contract_name.wasm" --wasm-out "target/wasm32-unknown-unknown/release/$contract_name.optimized.wasm"; then
        echo "Warning: 'soroban contract optimize' failed for $contract_name. Attempting fallback with 'wasm-opt -Oz'..."
        if command -v wasm-opt &> /dev/null; then
          if ! wasm-opt -Oz "target/wasm32-unknown-unknown/release/$contract_name.wasm" -o "target/wasm32-unknown-unknown/release/$contract_name.optimized.wasm"; then
            echo "Error: Both 'soroban contract optimize' and 'wasm-opt -Oz' failed for $contract_name. Please check your WASM file and tool installations."
            exit 2
          fi
        else
          echo "Error: 'wasm-opt' not found. Please install Binaryen (https://github.com/WebAssembly/binaryen) for fallback optimization."
          exit 3
        fi
      fi
  fi
done

echo "Build completed successfully!"
