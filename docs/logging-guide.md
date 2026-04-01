# Logging Guide

## Overview

StrellerMinds contracts use two complementary event systems:

| System | Purpose | Example |
|--------|---------|---------|
| **Domain events** (`emit_token_event!`, etc.) | Business logic events for indexers and UIs | "Token minted", "Certificate issued" |
| **Logging** (`log_info!`, `log_error!`, etc.) | Operational telemetry for debugging and monitoring | "Function entered", "Unauthorized access attempt" |

Both emit through `env.events().publish()` but use different topic prefixes:
- Domain events: `"std_event"` topic prefix
- Logging: `"LOG"` topic prefix

Off-chain indexers can filter by topic[0] to separate the two streams.

## Quick Start

```rust
use shared::logger::{Logger, LogLevel};
use shared::{log_info, log_error};
use soroban_sdk::symbol_short;

// In your contract's initialize function:
pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
    Logger::init(&env, LogLevel::Info);
    log_info!(&env, symbol_short!("myctrt"), symbol_short!("init_ok"));
    Ok(())
}

// In other functions:
pub fn do_something(env: Env) -> Result<(), Error> {
    log_info!(&env, symbol_short!("myctrt"), symbol_short!("action"));
    // ... business logic ...
    Ok(())
}
```

## Log Levels

| Level | Value | Use For |
|-------|-------|---------|
| `Debug` | 0 | Detailed execution tracing (development only) |
| `Info` | 1 | Normal operational events (default) |
| `Warn` | 2 | Unexpected but recoverable situations |
| `Err` | 3 | Errors and failed operations |
| `Metric` | 4 | Performance measurements |

**Recommendation:** Use `Info` in production, `Debug` during development. Set the level via `Logger::init()` during contract initialization or `Logger::set_level()` at runtime.

## Available Macros

### Logging macros

```rust
log_debug!(env, contract_symbol, message_symbol);
log_debug!(env, contract_symbol, message_symbol, payload_string);

log_info!(env, contract_symbol, message_symbol);
log_info!(env, contract_symbol, message_symbol, payload_string);

log_warn!(env, contract_symbol, message_symbol);
log_warn!(env, contract_symbol, message_symbol, payload_string);

log_error!(env, contract_symbol, message_symbol);
log_error!(env, contract_symbol, message_symbol, payload_string);

log_metric!(env, metric_name_symbol, value_i128);
```

### Context macro

For more detailed logging with function name and correlation IDs:

```rust
use shared::{log_ctx};
use shared::logger::Logger;

let ctx = log_ctx!(symbol_short!("token"), symbol_short!("mint"));
// or with correlation ID:
let ctx = log_ctx!(symbol_short!("token"), symbol_short!("mint"), 42);

Logger::log(&env, LogLevel::Info, &ctx, symbol_short!("started"), None);
```

### Function tracing (test/debug builds only)

```rust
use shared::trace_fn;

// Wraps a block with enter/exit log events
let result = trace_fn!(&env, symbol_short!("token"), symbol_short!("mint"), {
    // ... function body ...
    do_mint(&env, &to, amount)
});
```

Note: `trace_fn!` and `DebugUtils` are only available when compiled with the `testutils` feature or in `#[cfg(test)]`.

## Log Aggregation

The `LogAggregator` maintains lightweight counters in temporary storage:

```rust
use shared::log_aggregator::LogAggregator;

// Get current stats
let stats = LogAggregator::get_stats(&env);
// stats.info_count, stats.error_count, stats.total_count, etc.

// Get error rate as percentage (0-100)
let rate = LogAggregator::get_error_rate(&env);

// Reset counters (admin operation)
LogAggregator::reset(&env);
```

Counters auto-expire via temporary storage TTL (~30 days). Full log history is NOT stored on-chain; use off-chain event indexers for that.

## Debugging Utilities

Available in test and `testutils` builds only:

```rust
use shared::debug_utils::DebugUtils;

// Check if a storage key exists across all storage types
DebugUtils::inspect_storage_key(&env, &key, symbol_short!("mykey"), symbol_short!("myctrt"));

// Emit current ledger state
DebugUtils::emit_ledger_snapshot(&env, symbol_short!("myctrt"));
```

## Gas Cost Considerations

- **Level filtering is cheap**: `should_log()` reads one instance storage value. Suppressed logs cost ~200 gas (just the storage read).
- **Use `symbol_short!`** for messages: max 9 ASCII characters, stored inline, much cheaper than `String`.
- **`payload` is optional**: pass `None` when you don't need extra data.
- **Debug logs skip aggregation**: `LogAggregator::record()` is not called for `Debug`-level logs to save gas.
- **Debug utilities are zero-cost in production**: they are compiled out when not using `testutils` feature.

## Best Practices

1. **Initialize logging early**: Call `Logger::init()` in your contract's `initialize` function.
2. **Keep messages short**: Use `symbol_short!` (max 9 chars). Put details in the optional `payload`.
3. **Log security events**: Always log unauthorized access attempts, revocations, and admin actions at `Warn` or `Err` level.
4. **Don't log in tight loops**: If processing a batch, log once before/after, not per item.
5. **Use domain events for business logic**: `log_info!` is for operational telemetry. Use `emit_token_event!` etc. for events that indexers and UIs consume.
6. **Use correlation IDs for complex flows**: Pass `env.ledger().sequence() as u64` as the correlation ID to group related log entries.

## Off-Chain Integration

To filter log events from a Soroban RPC event stream:

```
topic[0] = "LOG"        -> operational logs
topic[1] = contract name -> filter by contract
topic[2] = log level     -> filter by severity
topic[3] = function name -> filter by function
```

Data payload contains: `(timestamp, message, optional_payload, optional_correlation_id)`
