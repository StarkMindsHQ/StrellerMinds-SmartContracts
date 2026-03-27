# Property-Based Testing for StrellerMinds Smart Contracts

This repository uses **Property-Based Testing (PBT)** via the `proptest` crate to ensure the robustness and correctness of our smart contracts beyond simple unit tests.

## Why Property-Based Testing?

Unlike traditional tests that use fixed inputs, property-based tests generate hundreds of random inputs to verify that certain "invariant properties" always hold true. This is particularly useful for:
- Financial logic (token transfers, reward calculations)
- State transitions
- Mathematical edge cases (overflow/underflow)

## Current Implementations

### Token Contract Invariants

We have implemented property tests in `contracts/token/src/property_tests.rs` covering:

1.  **Reward Calculation Consistency**:
    - Ensures that `IncentiveManager::reward_course_completion` correctly applies rewards based on completion percentage and user streaks.
    - Verified property: `final_reward == base * percentage_multiplier * streak_multiplier`.
    
2.  **Conservation of Balance (Transfer)**:
    - Verifies that `transfer_optimized` preserves the total balance of the contract.
    - Invariant: `sum(balances_before) == sum(balances_after)`.
    - Verifies individual balance updates: `sender_after == sender_before - amount` and `recipient_after == recipient_before + amount`.

## Test Generators

We use `proptest` strategies to generate:
- **Reward percentages**: `0..100u32`
- **User streaks**: `0..100u32` (tested up to max multiplier)
- **Token amounts**: `0..5000u64` for transfers
- **Initial balances**: Varied ranges to ensure coverage of both low and high liquidity accounts.

## How to Run Property Tests

You can run the property tests using standard `cargo test`:

```bash
cargo test -p token --lib property_tests
```

By default, `proptest` runs 256 iterations for each test. You can increase this by setting the `PROPTEST_CASES` environment variable:

```bash
PROPTEST_CASES=1000 cargo test -p token --lib property_tests
```

## Future Work / Test Scenarios to Add

- [ ] **Batched Transfers**: Ensure that the total amount deducted in a batch transfer matches the sum of distributed amounts.
- [ ] **Staking Reward Invariants**: Verify that staking rewards never exceed the total supply and are correctly proportional to time and amount.
- [ ] **Reentrancy Guard Invariants**: Property tests that attempt to trigger state changes through callback hooks (if applicable).
- [ ] **Cross-Contract Invariants**: Ensure that token supply in the `token` contract matches data tracked in the `analytics` contract.

## Monitoring Results

Property test failures produce "shrinking" logs, which attempt to find the smallest possible input that triggers a failure. Always check the first failure in the output for the most minimal reproducible case.
