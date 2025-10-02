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

- Soroban CLI
- Docker (for E2E testing with localnet)
- Stellar account for testnet/mainnet deployment


## Environment Setup

1. Install Rust:
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

2. Install Soroban CLI:
```bash
cargo install --locked soroban-cli
```

3. Add WebAssembly target:
```bash
rustup target add wasm32-unknown-unknown
```

4. Install Docker (for E2E testing):
- macOS: Download Docker Desktop from https://docker.com
- Linux: Follow instructions at https://docs.docker.com/engine/install/
- Ensure Docker daemon is running before E2E tests

## Quick Start with Makefile

The project includes a comprehensive Makefile for easy development workflows:

```bash
# Show all available commands
make help

# Check prerequisites
make check

# Build all contracts
make build

# Run all tests (unit + E2E)
make test

# Run E2E tests only
make e2e-test

# Development workflow: clean, build, and test
make dev-test
```

## Building Contracts

To build all contracts:

```bash
./scripts/build.sh
# or
make build
```

## Testing

### Unit Tests

Run unit tests only (faster, no localnet required):

```bash
cargo test --workspace --exclude e2e-tests
# or
make unit-test
```

### End-to-End (E2E) Tests

The E2E test harness provides comprehensive integration testing using Soroban localnet:

#### Prerequisites for E2E Testing

- Docker installed and running
- Soroban CLI installed
- No other processes using ports 8000 (RPC) and 6379 (Redis)

#### Running E2E Tests

**Full E2E test cycle (recommended):**
```bash
# Starts localnet, runs tests, stops localnet
make e2e-test
# or
./scripts/run_e2e_tests.sh
```

**Quick smoke tests:**
```bash
# Fast connectivity and basic deployment tests
make e2e-test-quick
# or
./scripts/run_e2e_tests.sh --quick
```

**Keep localnet running for debugging:**
```bash
# Useful for multiple test runs or debugging
make e2e-test-keep
# or
./scripts/run_e2e_tests.sh --keep-running
```

**Run specific test patterns:**
```bash
./scripts/run_e2e_tests.sh --filter "certificate"
./scripts/run_e2e_tests.sh --filter "analytics" --verbose
```

#### Manual Localnet Management

For advanced development workflows:

```bash
# Start localnet manually
make localnet-start
# or
./scripts/start_localnet.sh start

# Check localnet status
make localnet-status
# or
./scripts/start_localnet.sh status

# View localnet logs
make localnet-logs
# or
./scripts/start_localnet.sh logs

# Stop localnet
make localnet-stop
# or
./scripts/start_localnet.sh stop
```

#### E2E Test Structure

The E2E tests are organized as follows:

- `e2e-tests/` - Dedicated test crate
- `e2e-tests/src/lib.rs` - Test utilities and contract management
- `e2e-tests/tests/integration.rs` - Integration test cases
- `scripts/start_localnet.sh` - Localnet lifecycle management
- `scripts/run_e2e_tests.sh` - Unified test runner

#### Test Accounts

The localnet automatically sets up test accounts:

- **admin**: System administrator with SuperAdmin role
- **alice**: Test user (instructor/student)
- **bob**: Test user (student)
- **charlie**: Test user (student)

All accounts are funded and ready for testing.

#### Troubleshooting E2E Tests


**Docker issues:**
```bash
# Check if Docker is running
docker info

# Check for port conflicts
lsof -i :8000  # Soroban RPC port
lsof -i :6379  # Redis port
```

**Localnet issues:**
```bash
# Reset localnet completely
make localnet-stop
docker system prune -f
make localnet-start
```

**Test failures:**
```bash
# Run with verbose output for debugging
./scripts/run_e2e_tests.sh --verbose

# Run specific test with cargo directly
cargo test --package e2e-tests test_certificate_flow -- --nocapture
```

## Code Quality

### Formatting and Linting

```bash
# Format all code
make fmt
# or
cargo fmt --all

# Run linter
make lint
# or
cargo clippy --all-targets --all-features

# Both together
make check-code
```

### Gas Testing and Optimization

The project includes gas regression testing:

```bash
# Run gas regression tests
cargo test gas_regression

# View gas optimization analysis
cat docs/gas_optimization_analysis.md

```

## Deployment


### Testnet Deployment

```bash
# Requires STELLAR_SECRET_KEY environment variable
export STELLAR_SECRET_KEY="your-testnet-secret-key"
./scripts/deploy_testnet.sh
# or
make deploy-testnet
```

### Mainnet Deployment

```bash
# Requires authorization and confirmation
./scripts/deploy_mainnet.sh
# or
make deploy-mainnet
```

### Contract IDs

After deployment, contract IDs are saved to:
- `target/<contract-name>.testnet.id`
- `target/<contract-name>.mainnet.id`

## Development Workflows

### Daily Development

```bash
# 1. Pull latest changes and check prerequisites
git pull
make check

# 2. Clean build and run all tests
make dev-test

# 3. Code formatting and linting before commit
make check-code
```

### Feature Development

```bash
# 1. Create feature branch
git checkout -b feature/your-feature

# 2. Develop with fast feedback
make unit-test  # Fast feedback during development

# 3. Full validation before commit
make test       # Unit + E2E tests

# 4. Commit and push
git add .
git commit -m "feat: your feature description"
git push origin feature/your-feature
```

### Debugging Contract Issues

```bash
# 1. Start localnet for interactive testing
make localnet-start

# 2. Deploy contracts manually
make build
# Deploy individual contracts as needed...

# 3. Use Soroban CLI for debugging
soroban contract invoke --id <contract-id> --fn <function> --arg <arg>

# 4. View logs
make localnet-logs
```

## Project Structure

```
StrellerMinds-SmartContracts/
├── contracts/              # Smart contract source code
│   ├── shared/            # Common RBAC and utilities
│   ├── certificate/       # Certificate management
│   ├── analytics/         # Learning analytics
│   └── token/             # Token and incentives
├── e2e-tests/             # End-to-end test suite
│   ├── src/lib.rs         # Test utilities
│   └── tests/             # Integration tests
├── scripts/               # Build and deployment scripts
├── docs/                  # Documentation
├── Cargo.toml            # Workspace configuration
└── Makefile              # Unified development commands
```

## Contributing

1. Ensure all tests pass: `make test`
2. Format and lint code: `make check-code`
3. Update documentation for new features
4. Follow existing code patterns and RBAC integration
5. Add appropriate tests for new functionality

