# Developer Onboarding Guide

Welcome to the StrellerMinds development team. This comprehensive guide will take you from zero to productive contributor in the StrellerMinds-SmartContracts codebase.

## Table of Contents

1. [Project Overview](#project-overview)
2. [Prerequisites](#prerequisites)
3. [Environment Setup](#environment-setup)
4. [Project Structure](#project-structure)
5. [Development Workflow](#development-workflow)
6. [Testing Guidelines](#testing-guidelines)
7. [Deployment Procedures](#deployment-procedures)
8. [Code Standards](#code-standards)
9. [Common Issues & Troubleshooting](#common-issues--troubleshooting)
10. [Getting Help](#getting-help)

---

## Project Overview

StrellerMinds-SmartContracts is a comprehensive suite of **Soroban smart contracts** on the **Stellar network** that powers the StarkMinds blockchain education platform.

### What We Build

| Component | Description |
|-----------|-------------|
| **Smart Contracts** | On-chain logic for educational credentialing, learning analytics, token incentives, and progress tracking |
| **Token System** | Token management with staking capabilities and reward mechanisms |
| **Analytics** | Comprehensive learning analytics and progress tracking |
| **RBAC** | Role-Based Access Control across all contracts |
| **Certificate System** | Multi-signature certificate issuance and verification |

### Technology Stack

| Technology | Version | Purpose |
|------------|---------|---------|
| Rust | stable (see `rust-toolchain.toml`) | Smart contract development |
| Soroban SDK | 22.0.0 | Stellar smart contract framework |
| Soroban CLI | 21.5.0 | Contract deployment and interaction |
| Stellar CLI | 21.5.0 | Network management |
| Docker | Latest | Local network for E2E testing |
| Node.js | 18+ | E2E test execution |

---

## Prerequisites

Before you begin, ensure you have the following installed:

### Required Software

```bash
# 1. Rust (v1.75 or later)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Soroban CLI (pinned version 21.5.0)
cargo install --locked soroban-cli --version 21.5.0

# 3. Docker (for E2E testing)
# macOS: Download Docker Desktop from https://docker.com
# Linux: Follow instructions at https://docs.docker.com/engine/install/

# 4. Node.js v18+ (for E2E tests)
# https://nodejs.org/
```

### Required Environment Variables

```bash
# Set your Stellar secret key for deployment
export STELLAR_SECRET_KEY="your_secret_key_here"

# Optional: Custom RPC endpoint
export SOROBAN_RPC_URL="https://your-rpc-endpoint.com"
```

---

## Environment Setup

### Automated Setup (Recommended)

The easiest way to set up your development environment:

```bash
# 1. Clone the repository
git clone https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts.git
cd StrellerMinds-SmartContracts

# 2. Run the automated setup script
./scripts/setup.sh
```

The setup script will automatically:
- ✅ Install Rust target `wasm32-unknown-unknown`
- ✅ Install Soroban CLI (pinned version 21.5.0)
- ✅ Install Stellar CLI (pinned version 21.5.0)
- ✅ Verify all installations
- ✅ Optionally install Binaryen (wasm-opt) for WASM optimization

### Manual Setup

If you prefer manual setup:

```bash
# 1. Add WebAssembly target
rustup target add wasm32-unknown-unknown

# 2. Verify installation
soroban --version
stellar --version

# 3. Build the project
cargo build --release --target wasm32-unknown-unknown

# 4. Run tests
cargo test
```

### Using the Makefile

The project includes a comprehensive Makefile for common workflows:

```bash
# Show all available commands
make help

# Check prerequisites
make check

# Build all contracts
make build

# Run all tests (unit + E2E)
make test

# Run unit tests only (faster)
make unit-test

# Run E2E tests only
make e2e-test

# Development workflow: clean, build, and test
make dev-test
```

---

## Project Structure

```
StrellerMinds-SmartContracts/
├── contracts/              # Smart contract source code
│   ├── analytics/         # Learning analytics contract
│   ├── token/             # Token management contract
│   ├── shared/            # RBAC and common utilities
│   ├── mobile-optimizer/  # Mobile optimization features
│   ├── progress/          # Progress tracking
│   ├── proxy/             # Upgradeable contract pattern
│   ├── search/            # Search functionality
│   └── student-progress-tracker/  # Module-level tracking
├── src/                   # Rust source files
├── scripts/               # Build, deploy, and utility scripts
├── docs/                  # Documentation
├── e2e-tests/            # End-to-end integration tests
├── Makefile              # Build automation
└── Cargo.toml            # Workspace configuration
```

### Contract Descriptions

| Contract | Directory | Purpose |
|----------|-----------|---------|
| **Analytics** | `contracts/analytics/` | Learning analytics and progress tracking with performance metrics |
| **Token** | `contracts/token/` | Token management with incentive system and staking |
| **Shared** | `contracts/shared/` | Common utilities including RBAC and reentrancy protection |
| **Mobile Optimizer** | `contracts/mobile-optimizer/` | Mobile optimization with offline capabilities |
| **Progress** | `contracts/progress/` | Simple course progress tracking |
| **Proxy** | `contracts/proxy/` | Upgradeable contract implementation with rollback |
| **Search** | `contracts/search/` | Advanced search system with analytics |
| **Student Progress Tracker** | `contracts/student-progress-tracker/` | Granular module-level progress tracking |

### Key Scripts

| Script | Purpose |
|--------|---------|
| `./scripts/setup.sh` | Automated environment setup |
| `./scripts/build.sh` | Build all smart contracts |
| `./scripts/run-e2e-tests.sh` | Run end-to-end tests |
| `./scripts/deploy.sh` | Deploy contracts to networks |
| `./scripts/start_localnet.sh` | Start local Soroban network |

---

## Development Workflow

### Making Your First Change

```bash
# 1. Create a new branch
git checkout -b feature/your-feature-name

# 2. Make your changes

# 3. Run fast feedback tests
make unit-test

# 4. Format and lint your code
cargo fmt --all
cargo clippy --workspace --all-targets --all-features -- -D warnings -D nonstandard-style

# 5. Commit with conventional commits format
git commit -m "feat(scope): add new feature"

# 6. Push to remote
git push origin feature/your-feature-name

# 7. Open a PR and link to the issue
```

### Code Review Checklist

Before opening a PR:

- [ ] `cargo fmt --all` produces no diffs
- [ ] Naming conventions followed (see [CODE_STYLE.md](CODE_STYLE.md))
- [ ] Clippy checks pass with no warnings
- [ ] All tests pass (`cargo test`)
- [ ] Documentation updated if needed
- [ ] PR links to the related issue

### Pre-commit Hooks

Install pre-commit hooks for automatic formatting:

```bash
# Install pre-commit
pip install pre-commit

# Install hooks
pre-commit install

# Run hooks manually
pre-commit run --all-files
```

---

## Testing Guidelines

### Unit Tests

Run unit tests for fast feedback (no localnet required):

```bash
# Run all unit tests
cargo test --workspace --exclude e2e-tests

# Or use Makefile
make unit-test

# Run tests for a specific contract
cargo test -p analytics
cargo test -p token
```

### Property-Based Tests

We use proptest to verify contract invariants with random inputs:

```bash
cargo test -p token --lib property_tests
```

For more details, see [PROPERTY_TESTING.md](PROPERTY_TESTING.md).

### End-to-End (E2E) Tests

Run the complete E2E test suite against a local Soroban network:

```bash
# Full E2E test cycle (recommended)
make e2e-test
# or
./scripts/run-e2e-tests.sh

# Quick smoke tests
make e2e-test-quick
# or
./scripts/run-e2e-tests.sh --quick

# Keep localnet running for debugging
make e2e-test-keep
# or
./scripts/run-e2e-tests.sh --keep-running

# Run specific test patterns
./scripts/run-e2e-tests.sh --filter "analytics"
./scripts/run-e2e-tests.sh --filter "token" --verbose
```

### E2E Prerequisites

- Docker installed and running
- Soroban CLI installed
- No other processes using ports 8000 (RPC) and 6379 (Redis)

### Manual Localnet Management

```bash
# Start localnet manually
make localnet-start
# or
./scripts/start_localnet.sh start

# Check localnet status
make localnet-status
# or
./scripts/start_localnet.sh status

# Stop localnet
make localnet-stop
# or
./scripts/start_localnet.sh stop
```

---

## Deployment Procedures

### Building Contracts

```bash
# Build all contracts
./scripts/build.sh
# or
make build

# Build with optimization
cargo build --release --target wasm32-unknown-unknown

# Optional: Extreme optimization for mainnet
soroban contract optimize --wasm target/wasm32-unknown-unknown/release/<contract>.wasm
```

### Network Options

| Network | Use Case | Command |
|---------|----------|---------|
| **Local** | Development & debugging | `./scripts/deploy.sh --network local` |
| **Testnet** | Staging & testing | `./scripts/deploy.sh --network testnet` |
| **Mainnet** | Production | `./scripts/deploy.sh --network mainnet` |

### Deployment Examples

```bash
# Dry-run deployment to testnet (preview only)
./scripts/deploy_testnet.sh --dry-run

# Deploy specific contract to testnet
./scripts/deploy_testnet.sh --contract analytics

# Deploy to mainnet (requires authorization)
export STELLAR_SECRET_KEY="your-secret-key"
./scripts/deploy_mainnet.sh --contract token --verbose
```

### Post-Deployment

After successful deployment, contract IDs are saved to:
```
target/<contract_name>.<network>.id
```

**Verify the contract:**
```bash
CONTRACT_ID=$(cat target/analytics.testnet.id)
stellar contract info --id $CONTRACT_ID --network testnet
```

**Initialize the contract:**
```bash
stellar contract invoke \
    --id $CONTRACT_ID \
    --source-account YOUR_ACCOUNT \
    --network testnet \
    -- initialize \
    --admin <YOUR_ADMIN_PUB_KEY>
```

**Extend TTL (Time to Live):**
```bash
stellar contract extend \
    --id $CONTRACT_ID \
    --ledgers-to-extend 500000 \
    --network mainnet
```

---

## Code Standards

### Naming Conventions

| Element | Convention | Example |
|---------|------------|---------|
| Variables/Functions | `snake_case` | `get_user_id`, `total_supply` |
| Types (`struct`, `enum`, `trait`) | `PascalCase` | `UserProfile`, `TokenBalance` |
| Constants | `SCREAMING_SNAKE_CASE` | `MAX_SUPPLY`, `DEFAULT_FEE` |
| Modules/Files | `snake_case` | `user_auth.rs`, `mod.rs` |

### Formatting and Linting

```bash
# Format code
cargo fmt --all

# Check for linting issues
cargo clippy --workspace --all-targets --all-features -- -D warnings -D nonstandard-style
```

### Commit Message Format

Follow conventional commits:

```
<type>(<scope>): <description>

feat(auth): add OAuth2 support
fix(token): correct balance calculation
docs(readme): update installation steps
test(analytics): add property-based tests
chore(deps): update Soroban SDK version
```

### Documentation Requirements

When creating or updating contracts:

1. **README.md**: Each contract directory must have a README.md
2. **Function Documentation**: All public functions must have rustdoc comments
3. **Event Documentation**: Document all emitted events and their schemas
4. **Usage Examples**: Include code examples for common operations

See [contributing.md](contributing.md) for the full documentation checklist.

---

## Common Issues & Troubleshooting

### wasm32 Target Not Found

```bash
# Solution
rustup target add wasm32-unknown-unknown
```

### E2E Tests Failing

```bash
# Clean up Docker
docker system prune -f

# Restart localnet
make localnet-stop
make localnet-start
```

### Build Errors After Pulling

```bash
# Clean and rebuild
make clean && make build
```

### Soroban CLI Version Mismatch

```bash
# Check version
soroban --version

# Reinstall correct version
cargo install --locked soroban-cli --version 21.5.0
```

### Port Conflicts

```bash
# Check what's using port 8000
lsof -i :8000

# Kill the process
kill <PID>
```

### Contract Initialization Errors

```bash
# Verify the contract exists
stellar contract info --id <CONTRACT_ID> --network <network>

# Check contract source
stellar contract inspect --id <CONTRACT_ID> --network <network>
```

---

## Getting Help

### Documentation Map

| Document | Purpose |
|----------|---------|
| [ARCHITECTURE.md](ARCHITECTURE.md) | System design and high-level architecture |
| [development.md](development.md) | Dev environment and workflows |
| [contributing.md](contributing.md) | Contribution standards |
| [CODE_STYLE.md](CODE_STYLE.md) | Naming and formatting rules |
| [DEPLOYMENT.md](DEPLOYMENT.md) | Deployment procedures |
| [API.md](API.md) | API reference |
| [SECURITY_AUDIT_REPORT.md](SECURITY_AUDIT_REPORT.md) | Security guidelines |

### Resources

- **Documentation Site**: https://starkmindshq.github.io/StrellerMinds-SmartContracts
- **Stellar Documentation**: https://www.stellar.org/developers
- **Soroban Documentation**: https://soroban.stellar.org/docs
- **GitHub Issues**: https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts/issues
- **Discussions**: https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts/discussions

### Quick Reference Commands

```bash
# Development
make help              # Show all commands
make check             # Verify prerequisites
make build             # Build all contracts
make test              # Run all tests
make unit-test         # Fast unit tests
make e2e-test          # Integration tests

# Localnet
make localnet-start   # Start local network
make localnet-status  # Check status
make localnet-stop    # Stop local network

# Deployment
./scripts/deploy.sh --network testnet --contract <name> --wasm <path>
```

---

## Next Steps

After completing this onboarding:

1. ✅ Review the [Architecture Overview](ARCHITECTURE.md)
2. ✅ Read the [Contributing Guidelines](contributing.md)
3. ✅ Understand the [Code Style](CODE_STYLE.md)
4. ✅ Explore the [Deployment Guide](DEPLOYMENT.md)
5. ✅ Make your first contribution!

Welcome to the team! 🚀