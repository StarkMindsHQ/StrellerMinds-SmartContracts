# StarkMinds-SmartContracts

StarkMinds-SmartContracts is the dedicated repository for all Stellar smart contracts powering StarkMinds—a pioneering blockchain education platform built on Stellar. Developed using Soroban, these smart contracts handle on-chain interactions such as course credentialing, token management, and secure data validation.

## Features

- Smart contract development using Soroban on Stellar
- Secure, efficient on-chain logic for education and credentialing
- Comprehensive testing suite for contract functionality
- Modular and scalable design for future enhancements

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Stellar CLI & Soroban CLI](https://soroban.stellar.org/docs/getting-started)
- Docker (optional, for running a local Stellar testnet)

### Installation

1. **Clone the Repository:**
   ```bash
   git clone https://github.com/your-username/starkminds-smartcontracts.git
   ```
2. **Navigate to the Repository:**
   ```bash
   cd starkminds-smartcontracts
   ```
3. **Build the Smart Contracts:**
   ```bash
   cargo build --release
   ```

### Testing

Run tests to ensure everything is functioning as expected:
```bash
cargo test
```

### Linting and Formatting

To maintain code quality and consistency, run the following commands locally before committing:

- **Format code:**
  ```bash
  cargo fmt
  ```

- **Check for linting issues:**
  ```bash
  cargo clippy
  ```

These checks are also enforced in CI and will fail the build if there are formatting issues or warnings.

### Deployment
Deployment instructions will be updated as integration with the Stellar network advances. For now, please refer to the [Soroban documentation](https://soroban.stellar.org/docs) for deployment details.

## Contribution Guidelines

We welcome contributions to improve our smart contracts!

1. Fork the repository.
2. Create a new branch:
   ```bash
   git checkout -b feature/your-feature-name
   ```
3. Make your changes with clear, descriptive commit messages.
4. Push your branch and open a pull request with a detailed description of your changes.

Ensure that your contributions adhere to our coding standards and include appropriate tests.

## Smart Contracts

This repository contains several smart contracts that power the StarkMinds educational platform:

### Core Contracts

- **[Analytics Contract](contracts/analytics/README.md)** - Comprehensive learning analytics and progress tracking
- **[Certificate Contract](contracts/certificate/README.md)** - Educational credential management with expiry and multi-signature support
- **[Mint Batch Certificates Contract](contracts/mint-batch-certificates/README.md)** - Efficient batch certificate issuance
- **[Mobile Optimizer Contract](contracts/mobile-optimizer/README.md)** - Mobile optimization with offline capabilities and gas optimization
- **[Progress Contract](contracts/progress/README.md)** - Simple course progress tracking with validation
- **[Proxy Contract](contracts/proxy/README.md)** - Upgradeable contract implementation with rollback support
- **[Search Contract](contracts/search/README.md)** - Advanced search system with saved searches and analytics
- **[Shared Contract](contracts/shared/README.md)** - Common utilities including RBAC and reentrancy protection
- **[Student Progress Tracker Contract](contracts/student-progress-tracker/README.md)** - Granular module-level progress tracking
- **[Token Contract](contracts/token/README.md)** - Token management with incentive system and staking

### Contract Documentation

Each contract includes comprehensive documentation covering:
- **Overview**: Purpose and main functionality
- **Interface**: Public functions and parameters
- **Configuration**: Settings and environment variables
- **Testing**: How to run tests and test coverage
- **Deployment**: Setup and deployment instructions
- **Usage Examples**: Code examples and integration patterns

  ### Documentation Standards

  All contracts follow a standardized documentation structure defined in [docs/README_TEMPLATE.md](docs/README_TEMPLATE.md). This ensures consistency and makes it easier for contributors and integrators to understand and use the contracts.
 
 
## Release Pipeline

Releases are automated and triggered by pushing a semantic version tag, e.g., `v1.2.3`.

Steps:

1. Ensure commits follow Conventional Commits (e.g., `feat(certificate): add expiry validation`).
2. Tag and push:

```bash
VERSION=vX.Y.Z
git tag -a "$VERSION" -m "Release $VERSION"
git push origin "$VERSION"
```

The workflow at `/.github/workflows/release.yml` will:

- Build all contracts for `wasm32-unknown-unknown`.
- Optimize WASM using `soroban contract optimize`.
- Package artifacts into `dist/` with filenames including the version tag.
- Generate release notes using `git-cliff` (Keep a Changelog style) from Conventional Commits.
- Create a GitHub Release and upload all WASM artifacts and `SHA256SUMS.txt`.

Pre-releases (e.g., `v1.2.3-rc.1`) are automatically marked as prereleases.

## Contact

  For any questions, issues, or suggestions, please open an issue or reach out to the maintainers.
