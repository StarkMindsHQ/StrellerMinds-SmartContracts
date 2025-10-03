# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Overview

StarkMinds-SmartContracts is a Stellar blockchain education platform built using Soroban smart contracts. The project features a modular architecture with comprehensive RBAC (Role-Based Access Control), reentrancy protection, and advanced educational features including certificate management, progress tracking, and multi-signature workflows.

## Architecture

### Workspace Structure
- **Cargo workspace** with 10 smart contracts under `contracts/`
- **Shared library** (`contracts/shared/`) provides common functionality across all contracts
- **Soroban SDK v22** for Stellar smart contract development
- **WASM target** compilation for blockchain deployment

### Contract Types
1. **Core Educational Contracts**:
   - `certificate` - Educational credential management with expiry/renewal
   - `progress` - Simple course progress tracking
   - `student-progress-tracker` - Granular module-level progress
   - `analytics` - Learning analytics and tracking

2. **Infrastructure Contracts**:
   - `shared` - RBAC system, reentrancy guards, and common utilities
   - `proxy` - Upgradeable contract implementation with rollback
   - `token` - Token management with incentive/staking systems
   - `search` - Advanced search with saved searches and analytics

3. **Advanced Features**:
   - `mint-batch-certificates` - Efficient batch certificate issuance
   - `mobile-optimizer` - Mobile optimization with offline capabilities

### Shared Module Architecture
The `shared` contract provides:
- **AccessControl**: OpenZeppelin-style RBAC with hierarchical permissions
- **ReentrancyGuard**: Protection against reentrancy attacks
- **Roles/Permissions**: Four-tier role system (SuperAdmin, Admin, Instructor, Student)
- **Events/Storage**: Common event emission and storage patterns

All contracts integrate the shared RBAC system using:
```rust
use shared::{
    access_control::AccessControl,
    roles::{Permission, RoleLevel},
    reentrancy_guard::ReentrancyLock,
};
```

## Development Commands

### Building Contracts
```bash
# Build all contracts with WASM optimization
./scripts/build.sh

# Build specific contract
cargo build --target wasm32-unknown-unknown --release -p <contract-name>

# Standard Rust build (for development)
cargo build --release
```

### Testing
```bash
# Run all tests
cargo test

# Run tests for specific contract
cargo test -p <contract-name>

# Run specific test
cargo test <test-name>

# Run tests with output
cargo test -- --nocapture
```

### Code Quality
```bash
# Format code
cargo fmt

# Run linter
cargo clippy

# Check formatting (CI-friendly)
cargo fmt --check

# Run clippy with all targets
cargo clippy --all-targets --all-features
```

### Deployment
```bash
# Deploy to testnet (requires STELLAR_SECRET_KEY env var)
./scripts/deploy_testnet.sh

# Deploy to mainnet (interactive confirmation required)
./scripts/deploy_mainnet.sh
```

## Testing Patterns

### Contract Testing Structure
Each contract follows this test organization:
- `test.rs` - Basic unit tests
- `*_tests.rs` - Feature-specific test modules
- `integration_tests.rs` - Cross-contract integration tests

### Running Single Test Files
```bash
# Run tests in a specific test file
cargo test --test integration_tests

# Run tests matching a pattern
cargo test certificate_mint
```

### Test Environment Setup
Tests use Soroban test environment:
```rust
use soroban_sdk::testutils::{Address as _, AuthorizedFunction, MockAuth, MockAuthInvoke};
use soroban_sdk::{Address, Env};

let env = Env::default();
env.mock_all_auths();  // For testing authorization flows
```

## Smart Contract Specifics

### Role-Based Access Control (RBAC)
All contracts implement hierarchical permissions:
- **SuperAdmin**: Full system control, can grant/revoke any role
- **Admin**: Contract administration, can manage most operations
- **Instructor**: Can issue/revoke certificates, manage courses
- **Student**: Basic read access, can view own certificates

### Reentrancy Protection
All state-modifying functions use reentrancy guards:
```rust
fn mint_certificate(env: Env, ...) -> Result<(), CertificateError> {
    let _guard = ReentrancyLock::new(&env);
    // Function implementation
}
```

### Certificate Contract Features
- **Multi-signature workflows** for high-value certificates
- **Expiry management** with renewal requests and notifications
- **Prerequisite system** with learning path generation
- **Batch operations** for efficient mass certificate issuance
- **Metadata validation** with comprehensive URI and content checks

### Storage Patterns
Contracts use Soroban's persistent storage:
```rust
env.storage().persistent().set(&key, &value);
let value: Option<T> = env.storage().persistent().get(&key);
```

### Event Emission
All contracts emit structured events for off-chain monitoring:
```rust
CertificateEvents::emit_certificate_minted(&env, &issuer, &student, &metadata);
```

## Environment Configuration

### Required Dependencies
- Rust stable (specified in `rust-toolchain.toml`)
- Soroban CLI: `cargo install --locked soroban-cli`
- WASM target: `rustup target add wasm32-unknown-unknown`

### Environment Variables
- `STELLAR_SECRET_KEY` - Required for deployment scripts
- Test contracts use mock environments, no external dependencies needed

### Network Configuration
The deployment scripts automatically configure:
- **Testnet**: `https://soroban-testnet.stellar.org`
- **Mainnet**: `https://soroban-rpc.stellar.org`

Contract IDs are saved to `target/<contract-name>.<network>.id` files after deployment.

## Common Development Workflows

### Adding New Contract Features
1. Implement the interface in `contracts/<contract>/src/interface.rs`
2. Add the implementation in `contracts/<contract>/src/lib.rs`
3. Update storage patterns in `contracts/<contract>/src/storage.rs`
4. Add comprehensive tests following existing patterns
5. Update contract documentation

### Integrating with Shared RBAC
1. Import shared modules: `use shared::access_control::AccessControl;`
2. Initialize RBAC in contract initialization: `AccessControl::initialize(&env, &admin)`
3. Use permission checks: `AccessControl::require_permission(&env, &user, &permission)`
4. Add reentrancy protection: `let _guard = ReentrancyLock::new(&env);`

### Cross-Contract Integration
Contracts can interact using Soroban's contract invocation:
```rust
let progress_client = ProgressContractClient::new(&env, &progress_contract_address);
let completion_status = progress_client.get_completion_status(&student, &course_id);
```

This is extensively used in the certificate contract's prerequisite system to validate student progress before certificate issuance.