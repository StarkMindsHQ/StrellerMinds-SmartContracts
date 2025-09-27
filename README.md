# StrellerMinds-SmartContracts

StrellerMinds-SmartContracts is the dedicated repository for all Stellar smart contracts powering StrellerMinds—a pioneering blockchain education platform built on Stellar. Developed using Soroban, these smart contracts handle on-chain interactions such as course credentialing, token management, and secure data validation.

## Documentation

For detailed documentation on our smart contracts, system architecture, and contribution guidelines, please visit our [documentation site](https://your-documentation-site.com).

## Features

- Smart contract development using Soroban on Stellar
- Secure, efficient on-chain logic for education and credentialing
- Comprehensive testing suite for contract functionality
- Modular and scalable design for future enhancements

## Supported Versions

| Toolchain         | Version                               |
| ----------------- | ------------------------------------- |
| Rust              | stable                                |
| Soroban CLI       | v0.8.2 (placeholder, please verify)   |
| Soroban SDK       | 0.8.4 (placeholder, please verify)    |

## Getting Started

### Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- [Stellar CLI & Soroban CLI](https://soroban.stellar.org/docs/getting-started)
- Docker (optional, for running a local Stellar testnet)

### Installation

1. **Clone the Repository:**
   ```bash
   git clone https://github.com/your-github-username/StrellerMinds-SmartContracts.git
   ```
2. **Navigate to the Repository:**
   ```bash
   cd StrellerMinds-SmartContracts
   ```
3. **Build the Smart Contracts:
   ```bash
   cargo build --release
   ```

### Localnet Quickstart

To get a local network up and running for testing or development, follow these steps:

1. **Start the Local Network:**
   ```bash
   docker run --rm -it -p 8000:8000 --name stellar stellar/quickstart:latest --local
   ```
2. **Deploy to Testnet:**
   ```bash
   ./scripts/deploy_testnet.sh
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

This repository contains several smart contracts that power the StrellerMinds educational platform:

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
- **Events**: Emitted events and their schemas
- **Configuration**: Settings and environment variables
- **Testing**: How to run tests and test coverage
- **Deployment**: Setup and deployment instructions
- **Usage Examples**: Code examples and integration patterns

### Documentation Standards

All contracts follow a standardized documentation structure defined in [docs/README_TEMPLATE.md](docs/README_TEMPLATE.md). This ensures consistency and makes it easier for contributors and integrators to understand and use the contracts.

## Contact

For any questions, issues, or suggestions, please open an issue or reach out to the maintainers.
