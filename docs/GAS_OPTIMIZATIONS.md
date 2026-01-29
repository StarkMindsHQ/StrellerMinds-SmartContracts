# Gas Optimization Guidelines

This document summarizes strategies and quick patterns to reduce gas usage across contracts in this repository.

Key recommendations
- Use compact storage: pack boolean flags into integers (see `contracts/shared/src/gas_utils.rs`).
- Prefer batched writes: write large collections in fixed-size chunks rather than many single-key writes.
- Use lazy loading: only read from storage when necessary; compute and cache derived values.
- Choose efficient data structures: prefer fixed-size arrays/bitmaps over growing vectors when possible.
- Avoid unnecessary contract-to-contract calls; prefer internal execution or batch endpoints.
- Cache frequently-read values in contract storage with explicit invalidation.
- Use iterators and chunking to minimize memory footprint and reduce repeated storage reads.

Patterns to implement
- Bit-packing: store up to 128 boolean flags in one `u128` value. Helpers in `contracts/shared/src/gas_utils.rs`.
- Chunked writes: split large `Vec` writes into pages of N items and write each page in one storage entry.
- Batch operations: add methods that accept arrays of inputs and perform a single transaction.
- Lazy initialization: compute and store derived state on first access instead of on every transaction.

Profiling
- Use `e2e/utils/gas_profiler.ts` to deploy/invoke contracts and capture transaction responses for resource usage.

Recommendations for reviewers
- Prefer small, targeted changes per contract: replace hot paths with packed storage and batch APIs.
- When optimizing, add unit tests verifying behavior and gas-profile snapshots to `contracts/*/test_snapshots`.
