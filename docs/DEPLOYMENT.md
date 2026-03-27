# Deployment Guide

Deploying the StarkMinds Smart Contracts to the Stellar network (Local, Testnet, or Mainnet) involves careful preparation and environment configuration. This guide outlines the comprehensive deployment strategy across all network environments.

## 1. Prerequisites

Before beginning the deployment process, verify your environment:
- **Rust target:** `wasm32-unknown-unknown`
- **Soroban CLI** & **Stellar CLI** (v21.5.0+)
- **Build Utilities:** Make sure you can execute `make build` and `cargo fmt`.

*Tip: Use `./scripts/setup.sh` as mentioned in the Developer Guide to establish a baseline.*

## 2. Compilation and Optimization

Compile all contracts to highly optimized WASM formats to save on deployment costs. Using the build script guarantees the appropriate flags are invoked:

```bash
cargo build --release --target wasm32-unknown-unknown
```

For extreme optimization (optional but recommended for mainnet):
```bash
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/<contract>.wasm
```

## 3. Environment Configuration

StarkMinds handles environment variables using `.env` configurations depending on the network.

Create variables for your specific context:
```bash
export STELLAR_SECRET_KEY="S..."
export SOROBAN_RPC_URL="https://soroban-testnet.stellar.org" # Example
export STELLAR_NETWORK_PASSPHRASE="Test SDF Network ; September 2015"
```

## 4. Using the Deployment Scripts

We provide managed scripts to abstract typical deployment complexities.

### Local Development Network
Start a local network for isolated testing before publishing live:
```bash
make localnet-start
# Wait for the network...
./scripts/deploy.sh --network local --contract shared --wasm target/wasm32-unknown-unknown/release/shared.wasm
```

### Testnet Deployment
The testnet simulates mainnet behavior with free network fees. Request Testnet XLM via Friendbot if your account is unfunded.
```bash
./scripts/deploy.sh --network testnet --contract analytics --wasm target/wasm32-unknown-unknown/release/analytics.wasm
```

### Mainnet Deployment
**(WARNING: Requires real XLM and is immutable)**  
Once you are confident in your contracts (audited, 100% test coverage):
```bash
./scripts/deploy.sh --network mainnet --contract token --wasm target/wasm32-unknown-unknown/release/token.wasm --verbose
```

## 5. Post-Deployment Verification

When a script completes successfully, the unique **Contract ID** is saved locally. Example:
```
target/analytics.testnet.id
```

**To verify the existence and state of the contract:**
```bash
CONTRACT_ID=$(cat target/analytics.testnet.id)

stellar contract info \
    --id $CONTRACT_ID \
    --network testnet
```

### Initializing the Contract
Almost all StarkMinds contracts require immediate initialization to bond an Admin structure or set standard parameters:
```bash
stellar contract invoke \
    --id $CONTRACT_ID \
    --source-account YOUR_ACCOUNT \
    --network testnet \
    -- initialize \
    --admin <YOUR_ADMIN_PUB_KEY>
```

Failure to initialize your contract post-deployment correctly could leave it vulnerable inside its unconfigured state.

## 6. Extending TTL (Time to Live)

Soroban leverages state archiving. It is paramount that active contracts and persistent storage are routinely restored or extended.
```bash
stellar contract extend \
    --id $CONTRACT_ID \
    --ledgers-to-extend 500000 \
    --network mainnet
```
Read more in the main development documentation on monitoring state health.
