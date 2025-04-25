#!/bin/bash
set -e

# Check if STELLAR_SECRET_KEY is set
if [ -z "$STELLAR_SECRET_KEY" ]; then
  echo "Error: STELLAR_SECRET_KEY environment variable is not set"
  echo "Please set it with your Stellar secret key for deployment"
  exit 1
fi

# Confirmation for mainnet deployment
echo "WARNING: You are about to deploy to MAINNET."
echo "This will use real funds and deploy production contracts."
read -p "Are you sure you want to continue? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
  echo "Deployment cancelled."
  exit 0
fi

# Deploy to mainnet
echo "Deploying contracts to mainnet..."

# Set Soroban network to mainnet
soroban config network add --global mainnet \
  --rpc-url https://soroban-rpc.stellar.org \
  --network-passphrase "Public Global Stellar Network ; September 2015"

# Deploy each contract
for wasm in target/wasm32-unknown-unknown/release/*.optimized.wasm; do
  contract_name=$(basename "$wasm" .optimized.wasm)
  echo "Deploying $contract_name..."
  
  # Deploy the contract
  contract_id=$(soroban contract deploy \
    --wasm "$wasm" \
    --source "$STELLAR_SECRET_KEY" \
    --network mainnet)
  
  echo "Deployed $contract_name with contract ID: $contract_id"
  echo "$contract_id" > "target/$contract_name.mainnet.id"
done

echo "Deployment to mainnet completed successfully!"
