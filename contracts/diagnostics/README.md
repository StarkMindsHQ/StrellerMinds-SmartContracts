# Diagnostics Contract

Advanced debugging and diagnostics platform for Soroban smart contracts providing real-time monitoring, performance analysis, and automated issue detection.

## Overview

The Diagnostics Contract is a comprehensive debugging platform that helps developers:

- Monitor contract state in real-time
- Trace transaction flows and analyze execution patterns
- Detect performance bottlenecks automatically
- Identify anomalies and potential issues before they become critical
- Generate actionable optimization recommendations

## Features

### 1. Real-Time State Visualization

- Capture contract state snapshots at any point in time
- Compare snapshots to detect state changes
- Track state evolution over time
- Detect potential memory leaks automatically

### 2. Transaction Flow Tracing

- Start and complete transaction traces
- Build call tree visualizations
- Analyze transaction patterns
- Detect unusual execution behaviors
- Track success/failure rates

### 3. Performance Profiling

- Record detailed performance metrics
- Identify bottlenecks automatically
- Calculate efficiency scores
- Compare performance across time periods
- Generate optimization recommendations

### 4. Anomaly Detection

- Detect gas usage spikes
- Identify slow execution patterns
- Find potential memory leaks
- Alert on high error rates
- Provide root cause analysis

## Interface

### Initialization

```rust
// Initialize the diagnostics contract
diagnostics::initialize(env, admin_address)
```

### Diagnostic Sessions

```rust
// Start a new diagnostic session
let session_id = diagnostics::start_session(env, contract_id);

// ... perform monitored operations ...

// End the session
let session = diagnostics::end_session(env, session_id);
```

### State Monitoring

```rust
// Capture state snapshot
let snapshot = diagnostics::capture_state_snapshot(env, contract_id);

// Compare two snapshots
let differences = diagnostics::compare_snapshots(env, snapshot1, snapshot2);

// Detect memory leaks
let has_leak = diagnostics::detect_memory_leak(env, snapshots_vec);
```

### Transaction Tracing

```rust
// Start tracing
let trace_id = diagnostics::start_trace(env, contract_id, function_name, caller);

// ... execute operation ...

// Complete trace
let trace = diagnostics::complete_trace(
    env,
    trace_id,
    contract_id,
    function_name,
    caller,
    success,
    error_message,
    child_calls,
    gas_used
);

// Analyze patterns
let patterns = diagnostics::analyze_flow_patterns(env, traces_vec);
```

### Performance Analysis

```rust
// Record performance metric
let metric = diagnostics::record_performance_metric(
    env,
    contract_id,
    operation_name,
    execution_time_ms,
    gas_consumed,
    memory_peak_bytes,
    cpu_instructions,
    io_operations
);

// Identify bottlenecks
let bottlenecks = diagnostics::identify_bottlenecks(env, metrics_vec, operation_filter);

// Get recommendations
let recommendations = diagnostics::get_recommendations(env, bottleneck);

// Calculate efficiency score
let score = diagnostics::calculate_efficiency_score(env, metrics_vec);
```

### Anomaly Detection

```rust
// Detect anomalies
let anomalies = diagnostics::detect_anomalies(
    env,
    contract_id,
    recent_metrics,
    baseline_metrics
);
```

## Configuration

```rust
pub struct DiagnosticConfig {
    pub enable_state_tracking: bool,
    pub enable_transaction_tracing: bool,
    pub enable_performance_profiling: bool,
    pub enable_anomaly_detection: bool,
    pub trace_retention_days: u32,
    pub anomaly_threshold_multiplier: u32,
    pub max_traces_per_session: u32,
}
```

### Update Configuration (Admin Only)

```rust
diagnostics::update_config(env, admin_address, new_config)
```

## Events

The contract emits the following events:

- `DIAGNOSTIC:INIT` - Contract initialized
- `DIAGNOSTIC:SESSION_START` - Diagnostic session started
- `DIAGNOSTIC:SESSION_END` - Diagnostic session ended
- `DIAGNOSTIC:SNAPSHOT` - State snapshot captured
- `DIAGNOSTIC:CONFIG_UPDATE` - Configuration updated

## Usage Examples

### Basic Monitoring

```rust
// Initialize
diagnostics::initialize(env.clone(), admin);

// Start session
let session_id = diagnostics::start_session(env.clone(), contract_id);

// Capture initial state
let snapshot1 = diagnostics::capture_state_snapshot(env.clone(), contract_id);

// ... perform operations ...

// Capture final state
let snapshot2 = diagnostics::capture_state_snapshot(env.clone(), contract_id);

// Check for changes
let changes = diagnostics::compare_snapshots(env.clone(), snapshot1, snapshot2);

// End session
diagnostics::end_session(env, session_id);
```

### Performance Profiling

```rust
// Record metrics for an operation
let metric = diagnostics::record_performance_metric(
    env.clone(),
    contract_id,
    Symbol::new(&env, "process_data"),
    250, // execution time in ms
    150000, // gas consumed
    5000000, // memory peak bytes
    250000, // CPU instructions
    10 // IO operations
);

// Collect multiple metrics
let mut metrics = Vec::new(&env);
metrics.push_back(metric);

// Identify bottlenecks
let bottlenecks = diagnostics::identify_bottlenecks(env.clone(), metrics, None);

// Get recommendations
for bottleneck in bottlenecks {
    let recommendations = diagnostics::get_recommendations(env.clone(), bottleneck);
    // Apply recommendations...
}
```

### Anomaly Detection

```rust
// Collect baseline metrics (normal operations)
let baseline_metrics = Vec::new(&env);
// ... populate with normal metrics ...

// Collect recent metrics
let recent_metrics = Vec::new(&env);
// ... populate with recent metrics ...

// Detect anomalies
let anomalies = diagnostics::detect_anomalies(
    env,
    contract_id,
    recent_metrics,
    baseline_metrics
);

// Handle anomalies
for anomaly in anomalies {
    match anomaly.severity {
        AnomalySeverity::Critical => {
            // Take immediate action
        }
        AnomalySeverity::Error => {
            // Log and investigate
        }
        _ => {
            // Monitor
        }
    }
}
```

## Best Practices

1. **Use Sessions**: Always wrap diagnostic operations in sessions for better organization
2. **Regular Snapshots**: Capture state snapshots at key points in your workflow
3. **Baseline Establishment**: Collect baseline metrics during normal operation for accurate anomaly detection
4. **Review Recommendations**: Act on bottleneck recommendations promptly
5. **Monitor Trends**: Track metrics over time to identify gradual degradation

## Integration with CLI

Use the Streller CLI for interactive debugging:

```bash
# Start diagnostic session
streller diagnostics start --contract <contract-id>

# View real-time metrics
streller diagnostics metrics --session <session-id>

# Analyze bottlenecks
streller diagnostics bottlenecks --contract <contract-id>

# Detect anomalies
streller diagnostics anomalies --contract <contract-id>
```

## Testing

Run the test suite:

```bash
cargo test -p diagnostics
```

## Dependencies

- soroban-sdk: ^22.0.0

## License

See LICENSE file in the root of the repository.
