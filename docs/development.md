# Development Guide

## Prerequisites

- Rust (see rust-toolchain.toml for version)
- Soroban CLI: Latest version `soroban-sdk` v22.0.0.
- Stellar account for testing

## Environment Setup

1. Install Rust:
\`\`\`
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
\`\`\`

2. Install Soroban CLI:
\`\`\`
cargo install --locked soroban-cli
\`\`\`

3. Add WebAssembly target:
\`\`\`
rustup target add wasm32-unknown-unknown
\`\`\`

## Building Contracts

To build all contracts:

\`\`\`
./scripts/build.sh
\`\`\`

## Testing

Run all tests:

\`\`\`
cargo test
\`\`\`

## Deployment

For testnet deployment:

\`\`\`
./scripts/deploy_testnet.sh
\`\`\`

For mainnet deployment (requires authorization):

\`\`\`
./scripts/deploy_mainnet.sh
