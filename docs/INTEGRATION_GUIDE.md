# Diagnostics Platform Integration Guide

This guide will help you integrate the debugging and diagnostics platform into your StrellerMinds smart contracts project.

## Files to Add

### 1. Diagnostics Contract

Copy the entire `contracts/diagnostics/` directory to your project:

```
StrellerMinds-SmartContracts/
└── contracts/
    └── diagnostics/          ← NEW
        ├── Cargo.toml
        ├── README.md
        └── src/
            ├── lib.rs
            ├── types.rs
            ├── state_tracker.rs
            ├── transaction_tracer.rs
            ├── performance_profiler.rs
            └── anomaly_detector.rs
```

### 2. Documentation

Add the documentation file:

```
StrellerMinds-SmartContracts/
└── docs/
    └── DIAGNOSTICS_PLATFORM.md    ← NEW
```

### 3. Enhanced CLI (Optional)

You can either:

**Option A: Replace the existing CLI**

```
StrellerMinds-SmartContracts/
└── utils/
    └── streller-cli/
        └── src/
            └── main.rs          ← REPLACE with enhanced version
```

**Option B: Add alongside existing CLI**

```
StrellerMinds-SmartContracts/
└── utils/
    ├── streller-cli/            ← Keep original
    └── streller-cli-enhanced/   ← NEW (add as separate tool)
```

## Integration Steps

### Step 1: Update Workspace Cargo.toml

Add the diagnostics contract to your workspace members in the root `Cargo.toml`:

```toml
[workspace]
members = [
    "contracts/*",
    "e2e-tests",
    "utils/streller-cli"
]
```

The wildcard `contracts/*` already includes it, so no change needed if you're using wildcards.

### Step 2: Build the Diagnostics Contract

```bash
cd contracts/diagnostics
cargo build --release --target wasm32-unknown-unknown
```

Or use the existing build script:

```bash
./scripts/build.sh
```

### Step 3: Deploy Diagnostics Contract

#### Option A: Add to existing deployment script

Edit `scripts/deploy.sh` or `scripts/deploy_testnet.sh` to include diagnostics:

```bash
# Add this line with other contract deployments
soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/diagnostics.wasm \
    --source $DEPLOYER_ACCOUNT \
    --network testnet
```

#### Option B: Deploy manually

```bash
soroban contract deploy \
    --wasm target/wasm32-unknown-unknown/release/diagnostics.wasm \
    --source admin \
    --network testnet
```

Save the returned contract ID to use in your code.

### Step 4: Initialize Diagnostics Contract

After deployment, initialize it:

```bash
soroban contract invoke \
    --id <DIAGNOSTICS_CONTRACT_ID> \
    --source admin \
    --network testnet \
    -- \
    initialize \
    --admin <ADMIN_ADDRESS>
```

### Step 5: Use in Your Contracts

Add diagnostics to your existing contracts:

```rust
// In your contract's Cargo.toml
[dependencies]
diagnostics = { path = "../diagnostics" }

// In your contract code
use diagnostics::{DiagnosticsContract, StateSnapshot, TransactionTrace};

// Example usage in a function
pub fn my_function(env: Env, params: MyParams) -> Result<(), Error> {
    // Start tracing
    let trace_id = diagnostics::start_trace(
        &env,
        &env.current_contract_address(),
        Symbol::new(&env, "my_function"),
        &env.invoker()
    );

    // Your existing logic
    let result = perform_operation(&env, params);

    // Complete trace
    diagnostics::complete_trace(
        &env,
        trace_id,
        &env.current_contract_address(),
        Symbol::new(&env, "my_function"),
        &env.invoker(),
        result.is_ok(),
        result.as_ref().err().map(|e| String::from_str(&env, &e.to_string())),
        Vec::new(&env),
        // Gas estimation - in production, get actual gas used
        estimate_gas_used()
    );

    result
}
```

### Step 6: Update Tests

Add diagnostic checks to your test suite:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_with_diagnostics() {
        let env = Env::default();

        // Deploy diagnostics
        let diagnostics_contract_id = deploy_diagnostics(&env);

        // Initialize
        let admin = Address::generate(&env);
        DiagnosticsContract::initialize(env.clone(), admin);

        // Start session
        let contract_id = env.current_contract_address();
        let session_id = DiagnosticsContract::start_session(
            env.clone(),
            contract_id.clone()
        );

        // Capture state before
        let snapshot_before = DiagnosticsContract::capture_state_snapshot(
            env.clone(),
            contract_id.clone()
        );

        // Run your test
        let result = my_contract_function(env.clone(), test_params);

        // Capture state after
        let snapshot_after = DiagnosticsContract::capture_state_snapshot(
            env.clone(),
            contract_id.clone()
        );

        // Check for issues
        let differences = DiagnosticsContract::compare_snapshots(
            env.clone(),
            snapshot_before,
            snapshot_after
        );

        // End session
        DiagnosticsContract::end_session(env, session_id);

        // Assertions
        assert!(result.is_ok());
        assert!(differences.len() == expected_changes);
    }
}
```

### Step 7: Configure Diagnostics

Set up your diagnostics configuration:

```rust
use diagnostics::DiagnosticConfig;

let config = DiagnosticConfig {
    enable_state_tracking: true,
    enable_transaction_tracing: true,
    enable_performance_profiling: true,
    enable_anomaly_detection: true,
    trace_retention_days: 30,
    anomaly_threshold_multiplier: 2,
    max_traces_per_session: 1000,
};

DiagnosticsContract::update_config(env, admin_address, config);
```

## CLI Integration

### Option 1: Replace Existing CLI

If you want to replace the existing CLI with the enhanced version:

```bash
cd utils/streller-cli
# Backup original
cp src/main.rs src/main.rs.backup
# Copy enhanced version
cp ../../diagnostics-platform/utils/streller-cli-enhanced/src/main.rs src/main.rs
# Rebuild
cargo build --release
```

### Option 2: Add as Separate Tool

Add the enhanced CLI as a separate binary:

```bash
cd utils
mkdir streller-diagnostics
cd streller-diagnostics
# Copy Cargo.toml and src from enhanced CLI
cargo init --name streller-diagnostics
# Copy the enhanced main.rs
```

## Testing the Integration

### 1. Build Everything

```bash
./scripts/build.sh
```

### 2. Run Tests

```bash
# Run diagnostics contract tests
cargo test -p diagnostics

# Run all tests
cargo test
```

### 3. Deploy to Testnet

```bash
./scripts/deploy_testnet.sh
```

### 4. Test CLI

```bash
# If you replaced the CLI
./utils/streller-cli/target/release/streller-cli

# Or if you added it separately
./utils/streller-diagnostics/target/release/streller-diagnostics
```

## Example: Adding Diagnostics to Analytics Contract

Here's a concrete example of adding diagnostics to the existing analytics contract:

```rust
// In contracts/analytics/src/lib.rs

use diagnostics::{DiagnosticsContract, PerformanceMetric};

#[contractimpl]
impl AnalyticsContract {
    pub fn record_learning_session(
        env: Env,
        student: Address,
        course_id: Symbol,
        module_id: u32,
        duration_seconds: u32,
        score: u32,
    ) -> Result<(), AnalyticsError> {
        // Start performance tracking
        let start_time = env.ledger().timestamp();

        // Start transaction trace
        let trace_id = diagnostics::start_trace(
            &env,
            &env.current_contract_address(),
            Symbol::new(&env, "record_session"),
            &student
        );

        // Your existing logic
        let result = self.internal_record_session(
            env.clone(),
            student.clone(),
            course_id,
            module_id,
            duration_seconds,
            score
        );

        // Calculate metrics
        let end_time = env.ledger().timestamp();
        let execution_time_ms = ((end_time - start_time) * 1000) as u32;

        // Record performance metric
        diagnostics::record_performance_metric(
            &env,
            &env.current_contract_address(),
            Symbol::new(&env, "record_session"),
            execution_time_ms,
            50000, // estimated gas
            1000000, // estimated memory
            execution_time_ms as u64 * 1000,
            5 // IO operations
        );

        // Complete trace
        diagnostics::complete_trace(
            &env,
            trace_id,
            &env.current_contract_address(),
            Symbol::new(&env, "record_session"),
            &student,
            result.is_ok(),
            result.as_ref().err().map(|e| String::from_str(&env, &e.to_string())),
            Vec::new(&env),
            50000
        );

        result
    }
}
```

## Monitoring in Production

### Set Up Periodic Health Checks

```rust
// Add a cron job or scheduled task
pub fn check_contract_health(env: Env, contract_id: Address) {
    let recent_metrics = get_recent_metrics(env.clone(), 3600); // last hour
    let baseline_metrics = get_baseline_metrics(env.clone(), 86400); // last day

    let anomalies = DiagnosticsContract::detect_anomalies(
        env.clone(),
        contract_id,
        recent_metrics,
        baseline_metrics
    );

    // Alert if critical anomalies found
    for anomaly in anomalies {
        if anomaly.severity == AnomalySeverity::Critical {
            send_alert(anomaly);
        }
    }
}
```

## Troubleshooting

### Build Errors

If you get compilation errors:

1. Make sure you're using Rust 1.75+
2. Ensure soroban-sdk version matches across all contracts
3. Run `cargo clean` and rebuild

### Deployment Issues

If deployment fails:

1. Check your Soroban CLI version: `soroban --version`
2. Verify network configuration
3. Ensure sufficient balance in deployer account

### Runtime Errors

If diagnostics fail at runtime:

1. Verify contract was initialized
2. Check admin permissions
3. Review storage limits and retention settings

## Best Practices

1. **Start Small**: Begin with one contract, validate the integration, then expand
2. **Monitor Costs**: Track storage and gas costs from diagnostics
3. **Baseline First**: Collect baseline metrics before enabling anomaly detection
4. **Regular Reviews**: Check diagnostic reports weekly
5. **Archive Data**: Export and archive old diagnostic data regularly

## Next Steps

After successful integration:

1. Set up automated monitoring
2. Create custom dashboards (optional)
3. Define alerting thresholds
4. Train your team on using the CLI
5. Document your specific monitoring workflows

## Support

If you encounter issues:

- Check the main README.md
- Review DIAGNOSTICS_PLATFORM.md for detailed documentation
- Open an issue on GitHub
- Join the StarkMinds community

## Contributing

Improvements to the diagnostics platform are welcome! See CONTRIBUTING.md for guidelines.
