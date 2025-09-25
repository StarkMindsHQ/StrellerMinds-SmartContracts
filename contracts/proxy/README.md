# Proxy Contract

## Overview
A simple proxy contract implementation for upgradeable smart contracts on Stellar. This contract provides basic upgrade functionality with rollback capabilities, allowing for contract implementation updates while maintaining the same contract address.

## Interface

### Core Functions
```rust
// Initialize proxy with admin and implementation address
fn initialize(env: Env, admin: Address, implementation: Address)

// Upgrade implementation (admin only)
fn upgrade(env: Env, new_implementation: Address)

// Rollback to previous implementation (admin only)
fn rollback(env: Env)

// Get current implementation address
fn get_implementation(env: Env) -> Address

// Get admin address
fn get_admin(env: Env) -> Address
```

## Events

### Proxy Events
- `proxy_initialized`: Emitted when proxy is initialized with admin and implementation
- `proxy_upgraded`: Emitted when implementation is upgraded to a new version
- `proxy_rollback`: Emitted when implementation is rolled back to previous version

## Configuration

### Proxy Configuration
- `admin`: Address with upgrade and rollback permissions
- `implementation`: Current implementation contract address
- `rollback_stack`: Stack of previous implementations for rollback functionality

### Storage Keys
- `Implementation`: Current implementation address
- `Admin`: Admin address
- `RollbackStack`: Stack of previous implementations

## Testing

### Running Tests
```bash
# Run all tests for proxy contract
cargo test --package proxy

# Run specific test modules
cargo test --package proxy tests::test_initialization
cargo test --package proxy tests::test_upgrade_functionality
cargo test --package proxy tests::test_rollback_functionality
```

### Test Coverage
- **Initialization Tests**: Proxy setup and configuration
- **Upgrade Tests**: Implementation upgrade functionality
- **Rollback Tests**: Rollback to previous implementation
- **Authorization Tests**: Admin-only function access control
- **Storage Tests**: Persistent storage validation

## Deployment

### Prerequisites
- Admin address for proxy management
- Initial implementation contract address

### Deployment Steps
1. Deploy the proxy contract
2. Deploy the initial implementation contract
3. Initialize proxy with admin and implementation addresses
4. Test upgrade and rollback functionality
5. Begin using proxy for contract interactions

### Environment Setup
- Set admin address for proxy management
- Deploy initial implementation contract
- Configure rollback stack size limits
- Set up monitoring for upgrade events

## Usage Examples

### Initializing the Proxy
```rust
let admin = Address::generate(&env);
let implementation = Address::generate(&env);

client.initialize(&admin, &implementation);
```

### Upgrading Implementation
```rust
let new_implementation = Address::generate(&env);
client.upgrade(&new_implementation);
```

### Rolling Back Implementation
```rust
client.rollback();
```

### Getting Current Implementation
```rust
let current_impl = client.get_implementation();
let admin_addr = client.get_admin();
```

## Important Notes

### Soroban Host Integration
- Actual call delegation is handled by Soroban host, not in userland Rust
- For a real proxy, you would use Soroban's host functions to forward calls
- This implementation provides the storage and upgrade management framework

### Security Considerations
- Only admin can perform upgrades and rollbacks
- Rollback stack prevents infinite rollback loops
- Implementation addresses are validated before storage

### Upgrade Process
1. Deploy new implementation contract
2. Call `upgrade()` with new implementation address
3. Previous implementation is stored in rollback stack
4. New implementation becomes active immediately

### Rollback Process
1. Call `rollback()` to revert to previous implementation
2. Previous implementation is popped from rollback stack
3. Current implementation becomes the previous version
4. Rollback fails if no previous implementation exists

## Data Structures

### Rollback Stack
- **Type**: `Vec<Address>`
- **Purpose**: Stores previous implementation addresses
- **Behavior**: LIFO (Last In, First Out) stack
- **Limits**: No explicit size limit (limited by contract storage)

### Storage Organization
- **Instance Storage**: Admin and implementation addresses
- **Persistent Storage**: Rollback stack for implementation history

## Related Docs
- [Development Guide](../docs/development.md)
- [Security Documentation](../docs/security.md)