# Developer Onboarding Guide

> **Version:** 2.0 | **Last Updated:** April 2026 | **Maintainer:** @LaGodxy

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
9. [Best Practices](#best-practices)
10. [Common Issues & Troubleshooting](#common-issues--troubleshooting)
11. [Getting Help](#getting-help)
12. [Verification Checklist](#verification-checklist)

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

## Best Practices

### Security-First Development

Always consider security implications in smart contract development:

```rust
// ✅ GOOD: Validate all inputs
fn process_transfer(from: Address, to: Address, amount: u32) -> Result<(), Error> {
    // Validate addresses are not zero
    if from == Address::from_bytes([0u8; 32]) {
        return Err(Error::InvalidFromAddress);
    }
    if to == Address::from_bytes([0u8; 32]) {
        return Err(Error::InvalidToAddress);
    }
    
    // Validate amount
    if amount == 0 {
        return Err(Error::ZeroAmount);
    }
    
    // Check permissions via RBAC
    if !Shared::has_role(from, Role::Admin) && !Shared::has_role(from, Role::Minter) {
        return Err(Error::Unauthorized);
    }
    
    // Proceed with transfer
    Ok(())
}

// ❌ BAD: Missing validation
fn process_transfer(from: Address, to: Address, amount: u32) -> Result<(), Error> {
    // No validation - vulnerable to attacks!
    Ok(())
}
```

### Reentrancy Protection

Always use the reentrancy guard from the Shared contract:

```rust
use crate::storage::ReentrancyGuard;

fn sensitive_operation(&self) -> Result<(), Error> {
    // Acquire lock at start
    self.reentrancy_guard.assert_not_entered()?;
    
    // ... perform operations ...
    
    // Lock automatically released when guard goes out of scope
    Ok(())
}
```

### Error Handling

Use descriptive error types:

```rust
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum Error {
    // Authentication errors
    NotAuthorized = 1,
    InvalidSignature = 2,
    ExpiredSession = 3,
    
    // Validation errors
    InvalidInput = 10,
    OutOfBounds = 11,
    DuplicateEntry = 12,
    
    // State errors
    ContractNotInitialized = 20,
    StorageOverflow = 21,
    LedgerSequenceTooOld = 22,
}
```

### Gas Optimization

Follow these patterns to minimize gas costs:

```rust
// ✅ GOOD: Use u32 instead of u64 when possible
struct Config {
    max_supply: u32,  // Sufficient for most token contracts
    min_stake: u32,
}

// ✅ GOOD: Batch storage operations
fn batch_update(&self, updates: Vec<Update>) -> Result<(), Error> {
    // Single storage commit for multiple changes
    let mut storage = self.storage.bump_mut();
    for update in updates {
        storage.set(&update.key, &update.value)?;
    }
    // Single commit at end
    storage.commit()?;  // One ledger write instead of N
    Ok(())
}

// ❌ BAD: Commit after each update
fn individual_update(&self, updates: Vec<Update>) -> Result<(), Error> {
    for update in updates {
        self.storage.set(&update.key, &update.value)?;
        self.storage.commit()?;  // N ledger writes - expensive!
    }
    Ok(())
}
```

### Testing Best Practices

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    // ✅ Use setup functions for common test state
    fn setup_test_env() -> (Env, Address, Address) {
        let env = Env::default();
        env.ledger().set().sequence(100);
        let admin = Address::from_array([1u8; 32]);
        let user = Address::from_array([2u8; 32]);
        (env, admin, user)
    }
    
    #[test]
    fn test_transfer_success() {
        let (env, admin, user) = setup_test_env();
        let contract = Contract::new(&env, admin);
        
        // Act
        contract.transfer(&admin, &user, 100).unwrap();
        
        // Assert
        assert_eq!(contract.balance(&user), 100);
    }
    
    #[test]
    fn test_transfer_insufficient_balance() {
        let (env, admin, user) = setup_test_env();
        let contract = Contract::new(&env, admin);
        
        // Act & Assert
        let result = contract.transfer(&admin, &user, 1000);
        assert!(result.is_err());
    }
}
```

### Documentation Standards

Every public function should have:

```rust
/// Transfers tokens from one address to another.
///
/// # Arguments
/// * `from` - The source address (must have sufficient balance)
/// * `to` - The destination address (cannot be zero address)
/// * `amount` - Number of tokens to transfer (must be > 0)
///
/// # Returns
/// * `Ok(())` - Transfer successful
/// * `Err(Error::NotAuthorized)` - Caller not authorized
/// * `Err(Error::InsufficientBalance)` - Source has insufficient balance
/// * `Err(Error::InvalidInput)` - Invalid address or amount
///
/// # Notes
/// - This function emits a `Transfer` event
/// - Reentrancy is protected via the Shared contract
/// - Gas cost: ~1000-1500 gas units depending on storage state
pub fn transfer(&self, from: &Address, to: &Address, amount: u32) -> Result<(), Error> {
    // Implementation
}
```

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

## Verification Checklist

Use this checklist to verify you've successfully completed your onboarding. Each step includes commands to run and expected outputs.

### Phase 1: Environment Setup ✅

| Step | Task | Verification Command | Expected Output |
|------|------|---------------------|-----------------|
| 1.1 | Clone repository | `git clone https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts.git` | Directory created |
| 1.2 | Navigate to project | `cd StrellerMinds-SmartContracts && ls` | Shows `Cargo.toml`, `Makefile`, `contracts/` |
| 1.3 | Install Rust | `rustc --version` | Version v1.75+ |
| 1.4 | Add WASM target | `rustup target add wasm32-unknown-unknown` | Success message |
| 1.5 | Install Soroban CLI | `soroban --version` | v21.5.0 |
| 1.6 | Install Stellar CLI | `stellar --version` | v21.5.0 |
| 1.7 | Run setup script | `./scripts/setup.sh` | "Environment ready" |

### Phase 2: Build & Test ✅

| Step | Task | Verification Command | Expected Output |
|------|------|---------------------|-----------------|
| 2.1 | Build contracts | `make build` or `./scripts/build.sh` | "Finished" + WASM files in `target/` |
| 2.2 | Run unit tests | `make unit-test` or `cargo test` | "test result: ok" |
| 2.3 | Format code | `cargo fmt --all` | No diff output |
| 2.4 | Run clippy | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | No warnings |

### Phase 3: Development Workflow ✅

| Step | Task | Verification Command | Expected Output |
|------|------|---------------------|-----------------|
| 3.1 | Create feature branch | `git checkout -b feature/test-onboarding` | Switched to new branch |
| 3.2 | Make a small change | Edit any test file | File modified |
| 3.3 | Run tests on change | `cargo test` | Tests pass |
| 3.4 | Commit change | `git commit -m "test: verify onboarding"` | Commit created |
| 3.5 | Clean up | `git checkout main && git branch -d feature/test-onboarding` | Branch deleted |

### Phase 4: E2E Testing (Optional) ✅

| Step | Task | Verification Command | Expected Output |
|------|------|---------------------|-----------------|
| 4.1 | Start localnet | `make localnet-start` | "Started" + RPC at port 8000 |
| 4.2 | Run E2E tests | `make e2e-test-quick` | "passed" |
| 4.3 | Stop localnet | `make localnet-stop` | "Stopped" |

---

## Step-by-Step Process to Complete Your Assignment

Follow these steps in order to complete your onboarding assignment:

### Step 1: Clone and Setup (10 minutes)

```bash
# Clone the repository
git clone https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts.git
cd StrellerMinds-SmartContracts

# Run automated setup
./scripts/setup.sh
```

**Verification:**
```bash
# Confirm setup completed
soroban --version  # Should show v21.5.0
stellar --version  # Should show v21.5.0
```

### Step 2: Build Contracts (5 minutes)

```bash
# Build all contracts
make build

# Verify build output
ls -la target/wasm32-unknown-unknown/release/*.wasm
```

**Expected Output:**
```
analytics.wasm    token.wasm    shared.wasm    mobile_optimizer.wasm
progress.wasm     proxy.wasm     search.wasm    student_progress_tracker.wasm
```

### Step 3: Run Tests (10 minutes)

```bash
# Run all unit tests
cargo test --workspace --exclude e2e-tests

# Verify no failures
# Look for: "test result: ok. 0 failed; 0 ignored; 0 measured; 0 filtered out"
```

### Step 4: Code Quality Checks (5 minutes)

```bash
# Format code
cargo fmt --all

# Run linter
cargo clippy --workspace --all-targets --all-features -- -D warnings -D nonstandard-style
```

**Expected:** No output means success.

### Step 5: Explore Contract Structure (15 minutes)

```bash
# List all contracts
ls -la contracts/

# Explore a specific contract (e.g., token)
ls -la contracts/token/src/
cat contracts/token/README.md
```

### Step 6: Make Your First Contribution (20 minutes)

```bash
# Create a new branch
git checkout -b feature/onboarding-verification

# Add a simple test verification comment
echo "# Onboarding Verification - $(date)" >> VERIFICATION.md

# Commit
git add VERIFICATION.md
git commit -m "docs: add onboarding verification marker"

# Push
git push -u origin feature/onboarding-verification
```

### Step 7: Review Architecture (15 minutes)

```bash
# Read architecture overview
cat docs/ARCHITECTURE.md

# Read code style guide
cat docs/CODE_STYLE.md
```

---

## Quick Reference Card

Print or save this quick reference:

```bash
# === DAILY DEVELOPMENT COMMANDS ===

# Build
make build

# Test (fast)
make unit-test

# Test (full including E2E)
make test

# Format + Lint
cargo fmt --all && cargo clippy -- -D warnings

# Start local network
make localnet-start

# Stop local network
make localnet-stop

# Deploy to testnet
./scripts/deploy.sh --network testnet --contract <name> --wasm <path>

# === FILE LOCATIONS ===

# Contracts:     contracts/<name>/
# Tests:         contracts/<name>/src/test.rs
# Docs:          docs/
# Scripts:       scripts/
# Build output:  target/wasm32-unknown-unknown/release/
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