# StellarMinds-SmartContracts

StellarMinds-SmartContracts is the dedicated repository for all Stellar smart contracts powering StarkMinds—a pioneering blockchain education platform built on Stellar. Developed using Soroban, these smart contracts handle on-chain interactions such as course credentialing, token management, and secure data validation.

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
- Required environment variable:  
  - `STELLAR_SECRET_KEY` (your Stellar secret key for deployment)

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
  cargo clippy
  ```

These checks are also enforced in CI and will fail the build if there are formatting issues or warnings.

---

## Deployment

The repository now includes a hardened deployment script for Soroban smart contracts:

- `scripts/deploy.sh` (supports local, testnet, and mainnet)

### Deployment Script Usage

Run the script from the project root:

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
Network-specific settings (URLs, keys, etc.) are managed via environment files named `.env.<network>`. The script will automatically load the correct file based on the `--network` flag.

#### Dry-Run Mode
When `--dry-run` is specified, the script prints the deployment steps but does not execute them.

#### Example
```bash
./scripts/deploy.sh --network testnet --contract certificate --wasm contracts/certificate/target/wasm32-unknown-unknown/release/certificate.optimized.wasm --dry-run
```

#### Cross-Platform Compatibility
The scripts are linted and compatible with macOS and Linux.

### Before Deploying

1. Ensure contracts are built and optimized:  
   ```bash
   cargo build --release --target wasm32-unknown-unknown
   ```
2. Set `STELLAR_SECRET_KEY` in your environment:  
   ```bash
   export STELLAR_SECRET_KEY='your_secret_key_here'
   ```
3. Optionally set `SOROBAN_RPC_URL` for a custom RPC endpoint.  

The scripts automatically validate:  
- Installed tools: `soroban`, `stellar`, `jq`  
- Environment variables: `STELLAR_SECRET_KEY`  
- Presence of optimized WASM files in:  
  ```
  target/wasm32-unknown-unknown/release/*.optimized.wasm
  ```

### Script Usage

Run the scripts from the project root:

- **Testnet Deployment**  
  ```bash
  ./scripts/deploy_testnet.sh [OPTIONS]
  ```

- **Mainnet Deployment**  
  ```bash
  ./scripts/deploy_mainnet.sh [OPTIONS]
  ```

### Available Flags

| Flag                 | Description                                                                                  | Required | Default        |
|----------------------|----------------------------------------------------------------------------------------------|----------|----------------|
| `--contract <name>`  | Deploy a specific contract (e.g., certificate). If omitted, deploys all contracts.           | No       | All contracts  |
| `--dry-run`          | Preview the deployment steps without executing any soroban commands. Useful for verification.| No       | Disabled       |
| `--profile <name>`   | Specify a Soroban profile (e.g., for AWS or custom config).                                   | No       | None           |
| `--verbose`          | Enable detailed output, including file paths and network details.                            | No       | Disabled       |
| `--help`             | Display usage information and exit.                                                          | No       | N/A            |

#### Notes:
- Mainnet deployment requires explicit confirmation by typing **YES** (in all caps).  
- In `--dry-run` mode, commands are echoed but not executed, and simulated contract IDs are generated.  
- Contract names must match the basename of your `.optimized.wasm` files (e.g., `certificate.optimized.wasm`).  

### Usage Examples

- Dry-run all contracts to testnet (preview only):  
  ```bash
  ./scripts/deploy_testnet.sh --dry-run
  ```

- Deploy a specific contract to testnet:  
  ```bash
  ./scripts/deploy_testnet.sh --contract certificate
  ```

- Verbose dry-run for a specific contract to mainnet:  
  ```bash
  ./scripts/deploy_mainnet.sh --contract certificate --dry-run --verbose
  ```

- Deploy all contracts to mainnet with a custom profile:  
  ```bash
  ./scripts/deploy_mainnet.sh --profile myprofile
  ```

After successful deployment, contract IDs are saved to:  
```
target/<contract_name>.<network>.id
```

For advanced configurations or troubleshooting, refer to the [Soroban documentation](https://soroban.stellar.org/docs).

---

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

---

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
- **Events**: Emitted events and their schemas
- **Configuration**: Settings and environment variables