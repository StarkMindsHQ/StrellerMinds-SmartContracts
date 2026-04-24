# Migration Guide: v1 to v2

This guide provides comprehensive instructions for migrating from StrellerMinds Smart Contracts v1 to v2. Please read this entire document before beginning the migration process.

## Table of Contents

- [Overview](#overview)
- [Breaking Changes](#breaking-changes)
- [Data Migration Steps](#data-migration-steps)
- [API Changes](#api-changes)
- [Configuration Updates](#configuration-updates)
- [Rollback Procedures](#rollback-procedures)
- [Migration Checklist](#migration-checklist)
- [Troubleshooting](#troubleshooting)

## Overview

Version 2 introduces significant improvements to the StrellerMinds smart contracts ecosystem, including:

- Enhanced security features with improved RBAC implementation
- Optimized gas usage and storage patterns
- New analytics and tracking capabilities
- Improved error handling and validation
- Updated Soroban SDK compatibility (v22.0.0)
- Enhanced mobile optimization features

### Migration Timeline

- **Estimated Duration**: 2-4 hours for small deployments, 4-8 hours for enterprise deployments
- **Downtime Required**: Minimal (contract upgrade pattern used)
- **Testing Required**: Full integration testing recommended

## Breaking Changes

### 1. Soroban SDK Version Update

**v1**: Used Soroban SDK v20.x.x
**v2**: Uses Soroban SDK v22.0.0

**Impact**: All contracts must be recompiled with the new SDK version.

**Action Required**:
```bash
# Update workspace dependencies
cargo update --package soroban-sdk --precise 22.0.0
```

### 2. Storage Layout Changes

**v1**: Used individual storage instances per contract
**v2**: Implements unified storage optimization with shared storage patterns

**Impact**: Existing data storage locations have changed.

**Action Required**: Run data migration script (see [Data Migration Steps](#data-migration-steps))

### 3. Access Control System Overhaul

**v1**: Basic role-based access control
**v2**: Enhanced RBAC with granular permissions and hierarchical roles

**Impact**: Existing permission structures need to be migrated.

**Breaking Changes**:
- `AccessControl::initialize()` now requires additional parameters
- Role definitions have changed from enum-based to struct-based
- Permission checks are more strict

### 4. Contract Interface Changes

#### Analytics Contract
```rust
// v1
pub fn record_session(_env: Env, _session_id: BytesN<32>) -> Result<(), Error>

// v2
pub fn record_session(_env: Env, _session_id: BytesN<32>, _metadata: SessionMetadata) -> Result<(), Error>
```

#### Token Contract
```rust
// v1
pub fn mint(_env: Env, _to: Address, _amount: u64) -> Result<(), Error>

// v2
pub fn mint(_env: Env, _to: Address, _amount: u64, _reason: MintReason) -> Result<(), Error>
```

### 5. Error Handling Updates

**v1**: Basic error types
**v2**: Comprehensive error system with error codes and detailed messages

**Impact**: Error handling in client code needs to be updated.

### 6. Event Structure Changes

**v1**: Simple event emissions
**v2**: Structured events with enhanced metadata

**Impact**: Event parsing logic needs to be updated.

## Data Migration Steps

### Prerequisites

1. **Backup Current Data**
   ```bash
   # Export current contract state
   ./scripts/export-contract-state.sh --network <network> --contract <contract_id>
   ```

2. **Verify Backup Integrity**
   ```bash
   ./scripts/verify-backup.sh --backup-file <backup_file>
   ```

### Migration Process

#### Step 1: Environment Preparation

```bash
# 1.1 Clone v2 branch or tag
git checkout v2

# 1.2 Update dependencies
cargo update

# 1.3 Build new contracts
./scripts/build.sh

# 1.4 Run tests to ensure functionality
cargo test
```

#### Step 2: Deploy New Contracts

```bash
# 2.1 Deploy new contract instances
./scripts/deploy.sh --network <network> --contract analytics --wasm target/analytics.wasm
./scripts/deploy.sh --network <network> --contract token --wasm target/token.wasm
./scripts/deploy.sh --network <network> --contract shared --wasm target/shared.wasm

# 2.2 Initialize new contracts with migrated admin
./scripts/initialize-contracts.sh --network <network> --admin <admin_address>
```

#### Step 3: Data Migration

```bash
# 3.1 Run automated data migration
./scripts/migrate-data.sh --from-v1 --to-v2 --network <network>

# 3.2 Verify data integrity
./scripts/verify-migration.sh --network <network>
```

#### Step 4: Contract Upgrade

```bash
# 4.1 Update proxy contracts to point to new implementations
./scripts/upgrade-proxy.sh --network <network> --contract <contract_name>

# 4.2 Verify upgrade success
./scripts/verify-upgrade.sh --network <network>
```

### Manual Data Migration (if needed)

For custom data structures or complex migrations:

```rust
// Example: Migrating user permissions
use shared::access_control::{AccessControl, Role, Permission};

// v1 data structure
struct OldUserPermission {
    user: Address,
    role: String,  // String-based role
}

// v2 data structure
struct NewUserPermission {
    user: Address,
    role: Role,     // Enum-based role
    permissions: Vec<Permission>,
}

// Migration function
pub fn migrate_user_permissions(env: &Env, old_permissions: Vec<OldUserPermission>) {
    for old_perm in old_permissions {
        let new_role = match old_perm.role.as_str() {
            "admin" => Role::Admin,
            "instructor" => Role::Instructor,
            "student" => Role::Student,
            _ => Role::Student, // Default fallback
        };
        
        let new_permissions = match new_role {
            Role::Admin => vec![Permission::All],
            Role::Instructor => vec![Permission::CreateCourse, Permission::GradeStudents],
            Role::Student => vec![Permission::ViewCourse, Permission::SubmitAssignment],
        };
        
        AccessControl::grant_role(env, &old_perm.user, new_role, new_permissions);
    }
}
```

## API Changes

### Analytics Contract API Changes

#### New Methods
```rust
// Enhanced session tracking
pub fn record_session(env: Env, session_id: BytesN<32>, metadata: SessionMetadata) -> Result<(), Error>

// Advanced analytics
pub fn get_learning_metrics(env: Env, user: Address, timeframe: TimeFrame) -> LearningMetrics

// Engagement tracking
pub fn track_engagement(env: Env, user: Address, activity: Activity) -> Result<(), Error>
```

#### Modified Methods
```rust
// Updated with additional parameters
pub fn complete_session(env: Env, session_id: BytesN<32>, completion_data: CompletionData) -> Result<(), Error>

// Enhanced return types
pub fn get_session(env: Env, session_id: BytesN<32>) -> Option<SessionDetails>
```

#### Deprecated Methods
```rust
// Deprecated - use record_session with metadata instead
pub fn record_session(env: Env, session_id: BytesN<32>) -> Result<(), Error>
```

### Token Contract API Changes

#### New Methods
```rust
// Enhanced minting with reason tracking
pub fn mint(env: Env, to: Address, amount: u64, reason: MintReason) -> Result<(), Error>

// Staking functionality
pub fn stake(env: Env, user: Address, amount: u64, duration: u64) -> Result<(), Error>

// Reward distribution
pub fn distribute_rewards(env: Env, recipients: Vec<Address>, amounts: Vec<u64>) -> Result<(), Error>
```

#### Modified Methods
```rust
// Enhanced transfer with metadata
pub fn transfer(env: Env, from: Address, to: Address, amount: u64, metadata: TransferMetadata) -> Result<(), Error>

// Detailed balance information
pub fn balance(env: Env, account: Address) -> BalanceInfo
```

### Shared Contract API Changes

#### Access Control Enhancements
```rust
// Enhanced initialization
pub fn initialize(env: &Env, admin: &Address, config: AccessControlConfig) -> Result<(), Error>

// Granular permissions
pub fn grant_permission(env: &Env, user: &Address, permission: Permission) -> Result<(), Error>

// Role hierarchy support
pub fn create_role(env: &Env, role_name: Symbol, permissions: Vec<Permission>) -> Result<(), Error>
```

## Configuration Updates

### Environment Variables

#### New Variables
```bash
# Enhanced security
STRELLER_SECURITY_LEVEL=high
STRELLER_AUDIT_MODE=true

# Performance optimization
STRELLER_GAS_OPTIMIZATION=true
STRELLER_STORAGE_OPTIMIZATION=true

# Mobile optimization
STRELLER_MOBILE_MODE=true
STRELLER_OFFLINE_SUPPORT=true
```

#### Updated Variables
```bash
# Updated with additional options
SOROBAN_RPC_URL=https://horizon-testnet.stellar.org  # Updated endpoint
STELLAR_NETWORK_PASSPHRASE=Test SDF Network ; September 2015  # Updated passphrase
```

### Configuration Files

#### New Configuration Structure
```toml
# streller-config.toml
[network]
rpc_url = "https://horizon-testnet.stellar.org"
passphrase = "Test SDF Network ; September 2015"

[security]
audit_mode = true
security_level = "high"
require_multi_sig = true

[optimization]
gas_optimization = true
storage_optimization = true
mobile_optimization = true

[migration]
v1_contract_ids = ["old_analytics_id", "old_token_id"]
v2_contract_ids = ["new_analytics_id", "new_token_id"]
```

### Contract Configuration Updates

#### Analytics Contract
```rust
// v1 initialization
Analytics::initialize(env, admin_address)?;

// v2 initialization
let config = AnalyticsConfig {
    enable_advanced_tracking: true,
    retention_period: 365, // days
    privacy_level: PrivacyLevel::Standard,
};
Analytics::initialize(env, admin_address, config)?;
```

#### Token Contract
```rust
// v1 initialization
Token::initialize(env, admin_address)?;

// v2 initialization
let config = TokenConfig {
    enable_staking: true,
    reward_rate: 100, // basis points
    max_supply: 1_000_000_000,
};
Token::initialize(env, admin_address, config)?;
```

## Rollback Procedures

### Immediate Rollback (Emergency)

#### Step 1: Emergency Stop
```bash
# Pause all contract operations
./scripts/emergency-pause.sh --network <network>
```

#### Step 2: Restore Previous Version
```bash
# Switch to v1 branch
git checkout v1

# Rebuild v1 contracts
./scripts/build.sh

# Redeploy v1 contracts
./scripts/deploy.sh --network <network> --contract analytics --wasm target/analytics_v1.wasm
./scripts/deploy.sh --network <network> --contract token --wasm target/token_v1.wasm
```

#### Step 3: Restore Data
```bash
# Restore from backup
./scripts/restore-backup.sh --network <network> --backup-file <v1_backup_file>
```

#### Step 4: Verify Operations
```bash
# Verify contract functionality
./scripts/verify-rollback.sh --network <network>
```

### Planned Rollback

#### Step 1: Prepare Rollback Environment
```bash
# Create rollback branch
git checkout -b rollback-to-v1

# Ensure v1 contracts are available
./scripts/build-v1.sh
```

#### Step 2: Notify Users
```bash
# Send notification (implement based on your notification system)
./scripts/notify-users.sh --message "Scheduled maintenance: Rolling back to v1"
```

#### Step 3: Execute Rollback
```bash
# Graceful shutdown of v2 services
./scripts/graceful-shutdown.sh

# Restore v1 contracts
./scripts/rollback-to-v1.sh --network <network>
```

#### Step 4: Post-Rollback Verification
```bash
# Comprehensive testing
./scripts/post-rollback-tests.sh --network <network>

# Monitor system health
./scripts/monitor-health.sh --duration 3600  # 1 hour
```

## Migration Checklist

### Pre-Migration Checklist

- [ ] **Backup Creation**: Create complete backup of current contract states
- [ ] **Environment Setup**: Prepare v2 deployment environment
- [ ] **Dependency Update**: Update all dependencies to v2 versions
- [ ] **Testing**: Run full test suite on v2 contracts
- [ ] **Documentation Review**: Review all migration documentation
- [ ] **Team Training**: Ensure team is trained on v2 features
- [ ] **Communication Plan**: Prepare user communication plan

### Migration Day Checklist

- [ ] **Final Backup**: Create final backup before migration
- [ ] **User Notification**: Notify users of upcoming migration
- [ ] **Contract Deployment**: Deploy v2 contracts
- [ ] **Data Migration**: Execute data migration scripts
- [ ] **Contract Upgrade**: Upgrade proxy contracts
- [ ] **Verification**: Verify all contracts are working
- [ ] **Testing**: Run integration tests
- [ ] **Monitoring**: Begin monitoring system health

### Post-Migration Checklist

- [ ] **Functionality Testing**: Test all contract functions
- [ ] **Performance Testing**: Verify performance meets expectations
- [ ] **Security Testing**: Run security validation
- [ ] **User Acceptance**: Confirm users can access services
- [ ] **Documentation Update**: Update all documentation
- [ ] **Monitoring Setup**: Ensure monitoring is active
- [ ] **Backup Verification**: Verify backups are working
- [ ] **Rollback Plan Test**: Test rollback procedure

## Troubleshooting

### Common Issues

#### Issue 1: Contract Deployment Fails
**Symptoms**: Deployment script fails with "contract already exists" error

**Solutions**:
```bash
# Check if contract exists
./scripts/check-contract-exists.sh --contract-id <contract_id>

# If exists, use upgrade instead of deploy
./scripts/upgrade-contract.sh --contract-id <contract_id> --new-wasm <wasm_file>
```

#### Issue 2: Data Migration Fails
**Symptoms**: Migration script fails with "data validation error"

**Solutions**:
```bash
# Check data integrity
./scripts/validate-data.sh --source v1 --target v2

# Run migration with validation disabled (emergency only)
./scripts/migrate-data.sh --skip-validation --network <network>
```

#### Issue 3: Permission Errors
**Symptoms**: "Access denied" errors after migration

**Solutions**:
```bash
# Check current permissions
./scripts/check-permissions.sh --user <user_address>

# Reset permissions if needed
./scripts/reset-permissions.sh --network <network> --admin <admin_address>
```

#### Issue 4: Performance Degradation
**Symptoms**: Slow response times after migration

**Solutions**:
```bash
# Check gas optimization settings
./scripts/check-gas-optimization.sh --network <network>

# Re-optimize contracts
./scripts/optimize-contracts.sh --network <network>
```

### Getting Help

If you encounter issues not covered in this guide:

1. **Check Logs**: Review contract and migration logs
2. **Community Support**: Post issues on GitHub Discussions
3. **Emergency Contact**: For critical issues, contact the StarkMinds team

### Support Resources

- **GitHub Issues**: [StrellerMinds-SmartContracts Issues](https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts/issues)
- **Documentation**: [StrellerMinds Documentation](https://starkmindshq.github.io/StrellerMinds-SmartContracts/)
- **Community**: [GitHub Discussions](https://github.com/StarkMindsHQ/StrellerMinds-SmartContracts/discussions)

---

## Migration Scripts Reference

### Core Migration Scripts

- `scripts/migrate-data.sh` - Main data migration script
- `scripts/deploy.sh` - Contract deployment script
- `scripts/upgrade-contract.sh` - Contract upgrade script
- `scripts/verify-migration.sh` - Migration verification script
- `scripts/rollback-to-v1.sh` - Rollback script
- `scripts/emergency-pause.sh` - Emergency pause script

### Utility Scripts

- `scripts/export-contract-state.sh` - Export contract state
- `scripts/verify-backup.sh` - Verify backup integrity
- `scripts/check-permissions.sh` - Check user permissions
- `scripts/monitor-health.sh` - System health monitoring

---

**Last Updated**: 2024-04-24
**Version**: 2.0.0
**Compatibility**: Soroban SDK v22.0.0+
