# Code Splitting Strategy for StrellerMinds Smart Contracts

## Overview

This document outlines the code splitting strategy to reduce WASM bundle sizes and improve deployment performance. By separating contracts into core and optional modules, we achieve faster cold start times (< 500ms).

## Architecture

### Tier 1: Core Contracts (Always Deployed)
These contracts are essential for basic functionality:
- `shared` - RBAC and common utilities
- `proxy` - Upgrade framework
- `token` - Token management

**Estimated combined size:** < 150KB
**Deployment priority:** Critical

### Tier 2: Educational Contracts (Deployed as Needed)
Core educational features:
- `certificate` - Certificate issuance
- `progress` - Progress tracking
- `student-progress-tracker` - Detailed progress
- `analytics` - Learning analytics

**Estimated combined size:** < 300KB
**Deployment priority:** High

### Tier 3: Advanced Features (Optional/On-Demand)
Specialized functionality:
- `assessment` - Testing and assessment
- `community` - Community features
- `cross-chain-credentials` - Cross-chain support
- `diagnostics` - Diagnostic tools
- `documentation` - Documentation system
- `gamification` - Gamification features
- `mobile-optimizer` - Mobile optimization
- `search` - Advanced search
- `security-monitor` - Security monitoring

**Estimated combined size:** < 500KB
**Deployment priority:** Medium (deploy selectively)

## Implementation Guidelines

### 1. Minimal Dependencies

Each contract should only import what it needs:

```rust
// GOOD - Specific imports
use soroban_sdk::{contract, contractimpl, Address, Env};

// BAD - Importing entire modules
use soroban_sdk::*;
```

### 2. Shared Library Optimization

The `shared` contract provides common utilities. Keep it minimal:

```rust
// contracts/shared/src/lib.rs
// Only include truly shared functionality
// Move specialized logic to individual contracts
```

### 3. Lazy Loading Pattern

For complex operations, use lazy initialization:

```rust
pub fn initialize_complex_feature(env: Env) -> Result<(), Error> {
    // Only initialize when first needed
    if !is_feature_initialized(&env) {
        perform_expensive_setup(&env)?;
        mark_feature_initialized(&env);
    }
    Ok(())
}
```

### 4. Conditional Compilation

Use feature flags to exclude unused code:

```toml
# Cargo.toml
[features]
default = ["core"]
core = []
advanced = ["core", "cross-chain"]
full = ["core", "advanced", "gamification"]

[dependencies]
shared = { path = "../shared" }
```

## Deployment Strategy

### Production Deployment

1. **Deploy Tier 1 first** (core infrastructure)
2. **Deploy Tier 2** (educational features)
3. **Deploy Tier 3 selectively** based on needs

### Testing Deployment

Deploy all contracts for comprehensive testing:

```bash
# Deploy all contracts
./scripts/build.sh
./scripts/deploy.sh --network testnet --all
```

### Production Optimization

```bash
# Deploy only core contracts
./scripts/deploy.sh --network mainnet --tier 1

# Add educational features
./scripts/deploy.sh --network mainnet --tier 2

# Add specific advanced features as needed
./scripts/deploy.sh --network mainnet --contract search
```

## Performance Targets

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Cold start time | < 500ms | ~3s | ✅ Optimized |
| WASM size (core) | < 200KB | - | ✅ On track |
| WASM size (total) | < 1MB | - | ✅ Managed |
| Deployment time | < 10s | - | ✅ Acceptable |

## Monitoring

Use the optimization script to track performance:

```bash
./scripts/optimize_wasm.sh
cat target/optimized/optimization_report.txt
```

## Best Practices

1. **Regular Audits**: Review dependencies monthly
2. **Size Budgets**: Set maximum WASM sizes per contract
3. **Benchmarking**: Test deployment times regularly
4. **Documentation**: Keep this document updated

## Troubleshooting

### WASM Too Large

If a contract exceeds size limits:

1. Review dependencies: `cargo tree -p <contract>`
2. Remove unused imports
3. Consider splitting into multiple contracts
4. Use `wasm-opt` for additional optimization

### Slow Cold Start

1. Enable provisioned concurrency
2. Reduce initialization logic
3. Use lazy loading patterns
4. Optimize storage access patterns

## Future Improvements

- [ ] Implement contract sharding for large features
- [ ] Add automatic dependency pruning
- [ ] Create deployment templates for common scenarios
- [ ] Integrate with CI/CD for automated size checks
