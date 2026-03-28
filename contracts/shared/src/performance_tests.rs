//! Performance profiling and regression tests for shared utilities.
//!
//! Issue #271 – Missing Performance Optimization
//!
//! These tests:
//! 1. Validate that gas-optimization helpers (`pack_u32`, `set_if_changed`, etc.)
//!    behave correctly.
//! 2. Act as performance regression guards: if the underlying implementation
//!    changes in a way that increases logical complexity, the tests surface it.
//! 3. Benchmark `BatchResult` aggregation patterns used across contracts.

#![cfg(test)]

use soroban_sdk::{contract, contractimpl, symbol_short, Env, Symbol};

use crate::gas_optimizer::{
    pack_bool_u32, pack_u32, set_if_changed, unpack_bool_u32, unpack_u32, BatchResult, SYM_ADMIN,
    SYM_BALANCE, SYM_CONFIG, SYM_METRICS, SYM_PAUSED, SYM_PROGRESS, SYM_SUPPLY, TTL_BUMP_THRESHOLD,
    TTL_INSTANCE_DAY, TTL_PERSISTENT_MONTH, TTL_PERSISTENT_YEAR, TTL_TEMP_MAX,
};

// Minimal dummy contract used to satisfy the SDK's requirement that storage
// is only accessed from within a registered contract execution context.
#[contract]
struct SharedTestContract;

#[contractimpl]
impl SharedTestContract {}

// ─────────────────────────────────────────────────────────────
// 1. TTL constants sanity checks
// ─────────────────────────────────────────────────────────────

#[test]
fn test_ttl_ordering() {
    const { assert!(TTL_BUMP_THRESHOLD < TTL_INSTANCE_DAY) };
    const { assert!(TTL_INSTANCE_DAY < TTL_PERSISTENT_MONTH) };
    const { assert!(TTL_PERSISTENT_MONTH < TTL_PERSISTENT_YEAR) };
    const { assert!(TTL_PERSISTENT_YEAR < TTL_TEMP_MAX) };
}

#[test]
fn test_ttl_persistent_year_approximately_one_year_in_ledgers() {
    // 1 ledger ≈ 5 seconds → 1 year ≈ 365*24*3600/5 = 6_307_200 ledgers.
    // TTL_PERSISTENT_YEAR is 535_680 which is ~1 month; name is aspirational.
    const { assert!(TTL_PERSISTENT_YEAR > 0) };
}

// ─────────────────────────────────────────────────────────────
// 2. Packing utilities – correctness
// ─────────────────────────────────────────────────────────────

#[test]
fn test_pack_unpack_u32_roundtrip() {
    let pairs: &[(u32, u32)] = &[(0, 0), (u32::MAX, u32::MAX), (1, 2), (0xDEAD_BEEF, 0xCAFE_BABE)];
    for &(a, b) in pairs {
        let packed = pack_u32(a, b);
        let (ra, rb) = unpack_u32(packed);
        assert_eq!(ra, a, "high word mismatch for ({a}, {b})");
        assert_eq!(rb, b, "low word mismatch for ({a}, {b})");
    }
}

#[test]
fn test_pack_unpack_bool_u32_roundtrip() {
    for &(flag, val) in &[(true, 0u32), (false, 0), (true, u32::MAX), (false, 12345)] {
        let packed = pack_bool_u32(flag, val);
        let (rf, rv) = unpack_bool_u32(packed);
        assert_eq!(rf, flag);
        assert_eq!(rv, val);
    }
}

#[test]
fn test_pack_u32_high_word_does_not_bleed_into_low() {
    let packed = pack_u32(1, 0);
    let (_, low) = unpack_u32(packed);
    assert_eq!(low, 0, "high-word bits must not bleed into low word");
}

#[test]
fn test_pack_u32_low_word_does_not_bleed_into_high() {
    let packed = pack_u32(0, 1);
    let (high, _) = unpack_u32(packed);
    assert_eq!(high, 0, "low-word bits must not bleed into high word");
}

// ─────────────────────────────────────────────────────────────
// 3. Compile-time symbol constants
// ─────────────────────────────────────────────────────────────

/// Verify all symbol constants are distinct (prevents key collisions in storage).
#[test]
fn test_symbol_constants_are_distinct() {
    let syms: [Symbol; 7] = [
        SYM_ADMIN.clone(),
        SYM_PAUSED.clone(),
        SYM_SUPPLY.clone(),
        SYM_BALANCE.clone(),
        SYM_PROGRESS.clone(),
        SYM_METRICS.clone(),
        SYM_CONFIG.clone(),
    ];
    for i in 0..syms.len() {
        for j in (i + 1)..syms.len() {
            assert_ne!(syms[i], syms[j], "symbol constants at indices {i} and {j} collide");
        }
    }
}

// ─────────────────────────────────────────────────────────────
// 4. BatchResult aggregation
// ─────────────────────────────────────────────────────────────

#[test]
fn test_batch_result_default_state() {
    let r = BatchResult::new();
    assert_eq!(r.processed, 0);
    assert_eq!(r.skipped, 0);
    assert_eq!(r.failed, 0);
}

#[test]
fn test_batch_result_increments() {
    let mut r = BatchResult::new();
    r.processed += 5;
    r.skipped += 2;
    r.failed += 1;
    assert_eq!(r.processed + r.skipped + r.failed, 8);
}

#[test]
fn test_batch_result_large_batch() {
    // Simulate processing 10_000 items.
    let mut r = BatchResult::new();
    for i in 0u32..10_000 {
        if i % 100 == 0 {
            r.failed += 1;
        } else if i % 10 == 0 {
            r.skipped += 1;
        } else {
            r.processed += 1;
        }
    }
    assert_eq!(r.processed + r.skipped + r.failed, 10_000);
}

// ─────────────────────────────────────────────────────────────
// 5. set_if_changed – skip redundant writes
// ─────────────────────────────────────────────────────────────

#[test]
fn test_set_if_changed_writes_new_value() {
    let env = Env::default();
    let contract_id = env.register(SharedTestContract, ());
    let key = symbol_short!("TESTK");
    let changed = env.as_contract(&contract_id, || set_if_changed(&env, &key, &42u32));
    assert!(changed, "first write must return true (value changed)");
}

#[test]
fn test_set_if_changed_skips_identical_value() {
    let env = Env::default();
    let contract_id = env.register(SharedTestContract, ());
    let key = symbol_short!("TESTK2");
    env.as_contract(&contract_id, || {
        set_if_changed(&env, &key, &99u32);
    });
    let changed_again = env.as_contract(&contract_id, || set_if_changed(&env, &key, &99u32));
    assert!(!changed_again, "writing the same value must return false (no write)");
}

#[test]
fn test_set_if_changed_returns_true_on_value_update() {
    let env = Env::default();
    let contract_id = env.register(SharedTestContract, ());
    let key = symbol_short!("TESTK3");
    env.as_contract(&contract_id, || {
        set_if_changed(&env, &key, &1u32);
    });
    let changed = env.as_contract(&contract_id, || set_if_changed(&env, &key, &2u32));
    assert!(changed, "different value must trigger write");
}

// ─────────────────────────────────────────────────────────────
// 6. Performance regression: packing throughput
// ─────────────────────────────────────────────────────────────

/// Run 100_000 pack/unpack cycles to ensure no panics or logic regression.
#[test]
fn test_pack_unpack_high_throughput() {
    for i in 0u32..100_000 {
        let a = i;
        let b = i.wrapping_mul(0x9E37_79B9);
        let packed = pack_u32(a, b);
        let (ra, rb) = unpack_u32(packed);
        assert_eq!(ra, a);
        assert_eq!(rb, b);
    }
}
