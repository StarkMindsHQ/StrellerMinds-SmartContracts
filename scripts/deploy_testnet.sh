#!/bin/bash
set -e

# Check if STELLAR_SECRET_KEY is set
if [ -z "$STELLAR_SECRET_KEY" ]; then
  echo "Error: STELLAR_SECRET_KEY environment variable is not set"
  echo "Please set it with your Stellar secret key for deployment"
  exit 1
fi

# Deploy to testnet
echo "Deploying contracts to testnet..."

# Set Soroban network to testnet
soroban config network add --global testnet \
  --rpc-url https://soroban-testnet.stellar.org \
  --network-passphrase "Test SDF Network ; September 2015"

# Deploy each contract
for wasm in target/wasm32-unknown-unknown/release/*.optimized.wasm; do
  contract_name=$(basename "$wasm" .optimized.wasm)
  echo "Deploying $contract_name..."
  
  # Deploy the contract
  contract_id=$(soroban contract deploy \
    --wasm "$wasm" \
    --source "$STELLAR_SECRET_KEY" \
    --network testnet)
  
  echo "Deployed $contract_name with contract ID: $contract_id"
  echo "$contract_id" > "target/$contract_name.testnet.id"
done

echo "Deployment to testnet completed successfully!"
