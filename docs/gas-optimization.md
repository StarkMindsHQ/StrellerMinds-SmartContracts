# Gas Optimization Guide

## Fee Dimensions

| Resource | Rate |
|---|---|
| Ledger entry read | 6,250 stroops |
| Ledger entry write | 10,000 stroops |
| Bytes read | 1,750 stroops/KB |
| Bytes written | 11,800 stroops/KB |

## Strategies Applied

### 1. Packed Structs
All counters grouped into one struct = 1 read + 1 write instead of N.

### 2. Integer Packing
Two u32 values packed into one u64 slot, halving storage entries.

### 3. Bitfield Module Tracking
Up to 64 module flags in a single u64. Completion derived via count_ones().

### 4. Storage Tier Selection
- instance: config, aggregates, supply counters
- persistent: per-user data, course progress
- temporary: event logs, auto-expiring data

### 5. Lazy TTL Bumping
TTL extended only when near expiry, not on every operation.

### 6. Write Guard
Skip write if value unchanged, saves ledger write fee.

### 7. Batch Operations
All contracts expose batch variants: 1 transaction for N operations.

## Files Added

| File | Purpose |
|---|---|
| contracts/shared/src/gas_optimizer.rs | Shared utilities |
| contracts/analytics/src/gas_optimized.rs | PackedMetrics, batch events |
| contracts/token/src/gas_optimized.rs | PackedAccount, batch transfers |
| contracts/progress/src/gas_optimized.rs | Bitfield progress tracking |
| contracts/student-progress-tracker/src/gas_optimized.rs | Student aggregates |
| scripts/gas_profiler.sh | Profiling tool |

## Profiling Tool
```bash
./scripts/gas_profiler.sh
./scripts/gas_profiler.sh --compare
./scripts/gas_profiler.sh --contract token
```

## Wire Up in lib.rs

Add to contracts/shared/src/lib.rs:
pub mod gas_optimizer;

Add to each other contract's lib.rs:
pub mod gas_optimized;
