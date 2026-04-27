# Performance Optimization Guide

> Resolves issue **#271 – Missing Performance Optimization**

## Overview

This document covers the performance optimization strategy, profiling tooling, and test suite for StrellerMinds Smart Contracts. The primary performance concern in Soroban contracts is **instruction-budget consumption** and **storage I/O cost**.

For certificate verification API query-backend optimization, see `docs/DATABASE_QUERY_OPTIMIZATION.md`.

---

## Implemented Optimizations

### 1. Bit-Packed Progress Storage (`contracts/progress/src/gas_optimized.rs`)

`PackedProgress` stores all per-student progress metadata in two `u64` fields instead of multiple separate keys, reducing storage reads/writes per operation.

```
module_flags    u64   bit-set of completed module indices (up to 64 modules)
score_and_meta  u64   [score_x10:16][completion_pct:8][reserved:8][started_ledger:32]
```

**Gas savings:** Replacing `n` persistent reads with 1 achieves **O(1)** completion checks regardless of module count.

### 2. TTL-Aware Storage Bump (`contracts/shared/src/gas_optimizer.rs`)

`extend_persistent_if_needed` only extends TTL when the remaining TTL drops below `TTL_BUMP_THRESHOLD`, avoiding wasted rent payments:

```rust
pub fn extend_persistent_if_needed(env: &Env, key: &impl IntoVal<Env, Val>) {
    env.storage().persistent()
       .extend_ttl(&key_val, TTL_BUMP_THRESHOLD, TTL_PERSISTENT_YEAR);
}
```

### 3. Write Deduplication (`set_if_changed`)

`set_if_changed` compares the new value against the existing one before writing. If unchanged, the write is skipped entirely.

```rust
pub fn set_if_changed<K, V>(env: &Env, key: &K, new_val: &V) -> bool
```

**Returns** `true` when a write occurred, `false` when skipped. This is used in hot paths (token balances, progress percentages) to avoid charging for redundant storage mutations.

### 4. Packed Integer Utilities

`pack_u32 / unpack_u32` and `pack_bool_u32 / unpack_bool_u32` allow two values to be stored in a single `u64` slot, halving storage cost for frequently-read pairs.

### 5. Pre-Computed Symbol Constants

All storage key symbols (`SYM_ADMIN`, `SYM_SUPPLY`, etc.) are declared as `const Symbol` values, evaluated at compile time rather than allocated on every invocation.

---

## Performance Test Suite

### Location

```
contracts/progress/src/tests.rs           ← progress + PackedProgress perf tests
contracts/shared/src/performance_tests.rs ← gas optimizer regression tests
```

### Key tests

| Test | Validates |
|------|-----------|
| `test_record_progress_bulk_50_updates_within_budget` | 50 sequential writes within budget |
| `test_record_progress_20_students_concurrent_simulation` | 20 students × 1 write within budget |
| `test_packed_progress_bulk_operations_performance` | 64-module mark completes in O(1) |
| `test_pack_unpack_high_throughput` | 100,000 pack/unpack cycles without regression |
| `test_set_if_changed_skips_identical_value` | No-op on duplicate write |

---

## Profiling Tool

### Run a full profile

```bash
make perf-profile
```

This runs each contract's unit tests with timing instrumentation and writes results to `target/perf_report.json`.

### Save a baseline

```bash
make perf-baseline
```

### Compare against baseline

```bash
make perf-compare
```

Example output:

```
  ✅ ok  shared-gas-optimizer: 120ms → 115ms  (-4.2%)
  ✅ ok  progress-contract:    45ms  → 48ms   (+6.7%)
  ⚠️  slower  certificate-contract: 90ms → 115ms (+27.8%)
```

Any suite >20% slower than its baseline is flagged as a **regression**.

### Script usage

```bash
./scripts/perf_profile.sh [--report <path>] [--baseline] [--compare <path>] [--verbose]
```

---

## Profiling Report Format

`target/perf_report.json`:

```json
{
  "timestamp": "2026-03-27T10:00:00Z",
  "suites": [
    { "suite": "shared-gas-optimizer", "elapsed_ms": 115, "tests": 14 },
    { "suite": "progress-contract",    "elapsed_ms":  48, "tests": 18 },
    { "suite": "security-monitor",     "elapsed_ms": 210, "tests": 31 }
  ]
}
```

---

## Optimization Checklist

Before merging contract changes:

- [ ] Run `make perf-compare` to verify no regressions >20%
- [ ] Verify `set_if_changed` is used for any value that may not change on update
- [ ] Use `PackedProgress` or similar bit-packing for multi-field records
- [ ] Prefer `symbol_short!` over heap-allocated strings for storage keys
- [ ] Use `extend_persistent_if_needed` instead of unconditional `extend_ttl`
- [ ] Run `make perf-baseline` after intentional improvements

---

## CI Integration

```yaml
- name: Performance Profile
  run: make ci-perf

- name: Upload Perf Report
  uses: actions/upload-artifact@v4
  with:
    name: perf-report
    path: target/perf_report.json
```

---

## Monitoring

Track `elapsed_ms` trends across CI runs by archiving `target/perf_report.json` as a CI artifact and feeding it into a dashboard (Grafana, DataDog, etc.).

Alert thresholds:
- **Warning:** any suite >15% slower than 30-day rolling average  
- **Failure:** any suite >30% slower than 30-day rolling average  

---

## Effectiveness Review

| Metric | Target | How to Measure |
|--------|--------|----------------|
| No perf regressions on merge | 0 | `make perf-compare` in CI |
| Bulk write tests pass | 100% | `cargo test -- perf` |
| Pack/unpack 100k ops | < 10ms native | `test_pack_unpack_high_throughput` |
| set_if_changed skip rate | > 50% in token hot-path | Instrument with `BatchResult` |
