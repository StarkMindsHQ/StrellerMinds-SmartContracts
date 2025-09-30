# Development Guide

## Supported Versions

This project maintains compatibility with the following versions:

| Component | Version | Notes |
|-----------|---------|-------|
| Rust Toolchain | stable | See `rust-toolchain.toml` for exact configuration |
| Soroban SDK | 22.0.0 | Defined in workspace `Cargo.toml` |
| Soroban CLI | Latest compatible with SDK 22.0.0 | Install via `cargo install soroban-cli` |
| Stellar Strkey | 0.0.7 | Workspace dependency |
| Ed25519 Dalek | 2.0.0 | Workspace dependency |
| Rand | 0.8.5 | Workspace dependency |

**Important**: All contracts use workspace dependencies to ensure version consistency. Do not override these versions in individual contract `Cargo.toml` files.

## Prerequisites

- Rust (see rust-toolchain.toml for version)
- Soroban CLI: Latest version compatible with `soroban-sdk` v22.0.0
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


### Hardened Deployment Script

Use the new deployment script for all environments:

```bash
./scripts/deploy.sh --network <local|testnet|mainnet> --contract <name> --wasm <path> [--dry-run]
```

#### Flags
| Flag                   | Description                                                                 |
|------------------------|-----------------------------------------------------------------------------|
| `--network <network>`  | Specify which network to deploy to (`local`, `testnet`, or `mainnet`).        |
| `--contract <name>`    | Name of the contract to deploy.                                              |
| `--wasm <path>`        | Path to the WASM file to deploy.                                             |
| `--dry-run`            | Simulate the deployment steps without executing them.                        |

#### Environment Configuration
Create environment files named `.env.local`, `.env.testnet`, or `.env.mainnet` to store network-specific variables (e.g., URLs, keys). The script loads the appropriate file based on the `--network` flag.

#### Example
```bash
./scripts/deploy.sh --network testnet --contract certificate --wasm contracts/certificate/target/wasm32-unknown-unknown/release/certificate.optimized.wasm --dry-run
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
