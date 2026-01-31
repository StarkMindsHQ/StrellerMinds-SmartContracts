# StrellerMinds-SmartContracts

[![CI](https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts/actions/workflows/ci.yml/badge.svg?branch=main)](https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts/actions/workflows/ci.yml)
[![Release](https://img.shields.io/github/v/release/StarkMindsHQ/StrellerMinds-SmartContracts?sort=semver)](https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts/releases)
[![License](https://img.shields.io/github/license/StarkMindsHQ/StrellerMinds-SmartContracts)](https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts/blob/main/LICENSE)
[![Codecov](https://codecov.io/gh/StarkMindsHQ/StrellerMinds-SmartContracts/branch/main/graph/badge.svg)](https://codecov.io/gh/StarkMindsHQ/StrellerMinds-SmartContracts)
[![Soroban](https://img.shields.io/badge/Soroban-Stellar-purple.svg)](https://developers.stellar.org/)

StrellerMinds-SmartContracts is a comprehensive suite of Stellar smart contracts built with Soroban, powering the StarkMinds blockchain education platform. This repository provides secure, efficient on-chain logic for educational credentialing, learning analytics, token incentives, and progress tracking. Designed for educational institutions, online learning platforms, and EdTech developers who need reliable blockchain infrastructure for verifiable learning achievements and decentralized education ecosystems.

> 📖 **Documentation:** See the [docs/](docs/) directory and run `mkdocs serve` for local preview

## 🚀 Quick Start

Get up and running in under 5 minutes:

```bash
# 1. Clone the repository
git clone https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts.git
cd StrellerMinds-SmartContracts

# 2. Build all contracts
./scripts/build.sh

# 3. Run tests
cargo test
```

That's it! Your contracts are built and tested. See [Getting Started](#getting-started) for detailed installation instructions.

## ✨ Features

- Smart contract development using Soroban on Stellar
- Secure, efficient on-chain logic for education and credentialing
- Comprehensive testing suite for contract functionality
- Modular and scalable design for future enhancements
- Role-Based Access Control (RBAC) across all contracts
- Advanced analytics and progress tracking
- Multi-signature certificate issuance
- Token incentive system with staking capabilities

## 📋 Prerequisites

Before you begin, ensure you have the following installed:

- [Rust](https://www.rust-lang.org/tools/install) (v1.75 or later)
- [Stellar CLI & Soroban CLI](https://developers.stellar.org/docs/build/smart-contracts/getting-started)
- Docker (optional, for running a local Stellar testnet)
- Node.js v18+ (for E2E tests)

**Quick Setup:** Use our automated setup script to install all prerequisites:
```bash
./scripts/setup.sh
```

This script will automatically:
- Install Rust target `wasm32-unknown-unknown`
- Install Soroban CLI (pinned version 21.5.0)
- Install Stellar CLI (pinned version 21.5.0)
- Verify all installations
- Optionally install Binaryen (wasm-opt) for WASM optimization

### Required Environment Variables

- `STELLAR_SECRET_KEY` - Your Stellar secret key for deployment

## 🛠️ Getting Started

### Installation

1. **Clone the Repository:**
   ```bash
   git clone https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts.git
   cd StrellerMinds-SmartContracts
   ```

2. **Run the Setup Script (Recommended):**
   ```bash
   ./scripts/setup.sh
   ```
   
   This automated script will set up your entire development environment, including:
   - Rust toolchain with WASM target
   - Soroban CLI (v21.5.0)
   - Stellar CLI (v21.5.0)
   - Optional: Binaryen for WASM optimization
   
   The script will guide you through the installation process and verify all components.

3. **Build the Smart Contracts:**
   ```bash
   ./scripts/build.sh
   # Or manually:
   cargo build --release --target wasm32-unknown-unknown
   ```

### Testing

#### Unit Tests

Run unit tests to ensure everything is functioning as expected:

```bash
cargo test
```

#### End-to-End (E2E) Tests

Run the complete E2E test suite against a local Soroban network:

```bash
./scripts/run-e2e-tests.sh
```

This will:
- Build all contracts
- Start a local Soroban network
- Deploy contracts
- Run integration tests
- Clean up resources

For more details, see the [E2E Test Documentation](e2e/README.md).

### Linting and Formatting

To maintain code quality and consistency, run the following commands locally before committing:

- **Format code:**
  ```bash
  cargo fmt
  ```

- **Check for linting issues:**
  ```bash
  cargo clippy -- -D warnings
  ```

These checks are also enforced in CI and will fail the build if there are formatting issues or warnings.

## 🚢 Deployment

The repository includes a comprehensive deployment script for Soroban smart contracts supporting multiple networks:

### Deployment Script Usage

```bash
./scripts/deploy.sh --network <local|testnet|mainnet> --contract <name> --wasm <path> [--dry-run]
```

#### Available Flags

| Flag                  | Description                                                            |
| --------------------- | ---------------------------------------------------------------------- |
| `--network <network>` | Specify which network to deploy to (`local`, `testnet`, or `mainnet`) |
| `--contract <name>`   | Name of the contract to deploy                                         |
| `--wasm <path>`       | Path to the WASM file to deploy                                        |
| `--dry-run`           | Simulate the deployment steps without executing them                   |

#### Quick Deployment Examples

```bash
# Dry-run deployment to testnet (preview only)
./scripts/deploy_testnet.sh --dry-run

# Deploy specific contract to testnet
./scripts/deploy_testnet.sh --contract certificate

# Verbose deployment with custom profile
./scripts/deploy_mainnet.sh --contract certificate --profile myprofile --verbose
```

#### Environment Configuration

Network-specific settings are managed via `.env.<network>` files. The script automatically loads the correct configuration based on the `--network` flag.

#### Before Deploying

1. **Build and optimize contracts:**
   ```bash
   cargo build --release --target wasm32-unknown-unknown
   ```

2. **Set required environment variables:**
   ```bash
   export STELLAR_SECRET_KEY='your_secret_key_here'
   ```

3. **Optional: Set custom RPC endpoint:**
   ```bash
   export SOROBAN_RPC_URL='https://your-rpc-endpoint.com'
   ```

After successful deployment, contract IDs are saved to:
```
target/<contract_name>.<network>.id
```

For advanced configurations, see the [Soroban documentation](https://soroban.stellar.org/docs).

## 📦 Smart Contracts

This repository contains several smart contracts that power the StarkMinds educational platform:

### Core Contracts

- **[Analytics Contract](contracts/analytics/README.md)** - Comprehensive learning analytics and progress tracking with performance metrics and engagement insights
- **[Token Contract](contracts/token/README.md)** - Token management with incentive system, staking capabilities, and reward mechanisms
- **[Shared Contract](contracts/shared/README.md)** - Common utilities including RBAC, reentrancy protection, and validation functions

### Supporting Contracts

- **[Mobile Optimizer Contract](contracts/mobile-optimizer/README.md)** - Mobile optimization with offline capabilities and gas optimization
- **[Progress Contract](contracts/progress/README.md)** - Simple course progress tracking with validation
- **[Proxy Contract](contracts/proxy/README.md)** - Upgradeable contract implementation with rollback support
- **[Search Contract](contracts/search/README.md)** - Advanced search system with saved searches and analytics
- **[Student Progress Tracker Contract](contracts/student-progress-tracker/README.md)** - Granular module-level progress tracking

### Contract Documentation

Each contract includes comprehensive documentation covering:
- **Overview**: Purpose and main functionality
- **Interface**: Public functions and parameters
- **Events**: Emitted events and their schemas
- **Configuration**: Settings and environment variables
- **Testing**: Unit and integration test examples

> **Note**: Some contracts were removed during cleanup to ensure project stability. See the [cleanup notes](#cleanup-notes) for details.

## 🤝 Contributing

We welcome contributions to improve our smart contracts!

1. Fork the repository
2. Create a new branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. Make your changes with clear, descriptive commit messages
4. Ensure tests pass:
   ```bash
   cargo test
   cargo fmt
   cargo clippy
   ```
5. Push your branch and open a pull request with a detailed description

Please read our [Contributing Guidelines](docs/contributing.md) for more details.

## 📚 Documentation

- [Development Guide](docs/development.md)
- [Security Guidelines](docs/security.md)
- [RBAC Implementation](docs/RBAC_IMPLEMENTATION.md)
- [Mobile Optimizer System](docs/MOBILE_OPTIMIZER_SYSTEM.md)
- [Token Incentive System](docs/TOKEN_INCENTIVE_SYSTEM.md)
- [Security Audit Report](docs/SECURITY_AUDIT_REPORT.md)

### 📖 Quickstart: Contributing to Documentation

1. **Install dependencies** (Python 3.x):
   ```bash
   pip install mkdocs mkdocs-material
   ```
2. **Run local preview** from the repo root:
   ```bash
   mkdocs serve
   ```
   Open http://127.0.0.1:8000 to view the docs.
3. **Edit content** in `docs/`. The homepage is `docs/index.md`.
4. **Update navigation** in `mkdocs.yml` under the `nav:` section.
5. **Submit a PR**. The site auto-deploys to GitHub Pages on merges to `main`.

## 📝 License

This project is licensed under the terms specified in the [LICENSE](LICENSE) file.

## 🧹 Cleanup Notes

This repository has undergone significant cleanup to ensure stability and maintainability:

### Removed Components
- **Certificate Contract** - Had extensive compilation errors and missing trait implementations
- **Mint Batch Certificates Contract** - Type conversion issues and incomplete implementation
- **Unused files** - `.DS_Store`, `node_modules`, and other temporary files

### Current Status
- ✅ All remaining contracts compile successfully
- ✅ Core functionality preserved (analytics, token, shared utilities)
- ✅ Tests passing for shared contract
- ⚠️ Minor warnings remain (non-critical unused variables)

### Repository Structure

```
StrellerMinds-SmartContracts/
├── contracts/
│   ├── analytics/          # Learning analytics and progress tracking
│   ├── token/              # Token management with incentives
│   ├── shared/             # Common utilities and RBAC
│   ├── mobile-optimizer/   # Mobile optimization features
│   ├── progress/           # Simple progress tracking
│   ├── proxy/              # Upgradeable contract pattern
│   ├── search/             # Search functionality
│   └── student-progress-tracker/  # Granular progress tracking
├── e2e-tests/            # End-to-end integration tests
├── docs/                  # Comprehensive documentation
├── scripts/               # Build, deploy, and utility scripts
└── Cargo.toml            # Workspace configuration
```

## 🔗 Helpful Links

### Development & Documentation
- **[Development Guide](docs/development.md)** - Detailed setup and development workflow
- **[Security Guidelines](docs/security.md)** - Security best practices and audit reports
- **[RBAC Implementation](docs/RBAC_IMPLEMENTATION.md)** - Role-based access control documentation
- **[Mobile Optimizer System](docs/MOBILE_OPTIMIZER_SYSTEM.md)** - Mobile optimization architecture
- **[Token Incentive System](docs/TOKEN_INCENTIVE_SYSTEM.md)** - Token economics and incentives

### Platform & Community
- **[Stellar Documentation](https://stellar.org/developers)** - Stellar network documentation
- **[Soroban Documentation](https://developers.stellar.org/)** - Smart contract development guide
- **[Issue Tracker](https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts/issues)** - Bug reports and feature requests

### Contribution Resources
- **[Contributing Guidelines](docs/contributing.md)** - Detailed contribution process
- **[Code of Conduct](CODE_OF_CONDUCT.md)** - Community guidelines
- **[Release Process](docs/RELEASE_PROCESS.md)** - How releases are managed
- **[Architecture Overview](docs/ARCHITECTURE.md)** - High-level system design

## 📧 Support

For questions or support, please:
- Open an [issue](https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts/issues)
- Check our [documentation](docs/)
- Join our community discussions

---
