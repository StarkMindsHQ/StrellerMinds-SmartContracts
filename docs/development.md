# Development Guide

## Prerequisites

- Rust (see rust-toolchain.toml for version)
- Soroban CLI
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

```
cargo test
```

## Deployment

For testnet deployment:

```
./scripts/deploy_testnet.sh
```

For mainnet deployment (requires authorization):

```
./scripts/deploy_mainnet.sh
```

## Release Process

This repository uses an automated release pipeline triggered by semantic version tags (e.g., `v1.2.3`).

Steps to cut a release:

1. Ensure commits follow Conventional Commits (e.g., `feat(certificate): add expiry validation`).
2. Create and push a version tag:

```bash
VERSION=vX.Y.Z
git tag -a "$VERSION" -m "Release $VERSION"
git push origin "$VERSION"
```

The GitHub Action at `/.github/workflows/release.yml` will:

- Build all contracts for `wasm32-unknown-unknown`.
- Optimize WASM using `soroban contract optimize`.
- Package artifacts into `dist/` with the tag in filenames.
- Generate release notes using `git-cliff` (Keep a Changelog format) from Conventional Commits.
- Create a GitHub Release and upload all WASM artifacts and `SHA256SUMS.txt`.

Pre-releases (e.g., `v1.2.3-rc.1`) are marked as prerelease automatically.
