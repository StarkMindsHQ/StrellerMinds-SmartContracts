# Per-Contract WASM Build Matrix

## Overview

This document describes the per-contract WASM build matrix implementation added to the CI pipeline to address issue #128.

## Problem Statement

Previously, the CI built the entire workspace at once using `cargo build --all`. When a build or test failure occurred, it was difficult to identify which specific contract was causing the failure without parsing through extensive build logs.

## Solution

We implemented a GitHub Actions matrix strategy that builds and tests each contract individually. This provides:

1. **Isolation**: Each contract gets its own CI job
2. **Clarity**: Failures are immediately visible per contract
3. **Parallelization**: Multiple contracts can build simultaneously
4. **Size Reporting**: WASM file sizes are reported for each contract

## Implementation Details

### Matrix Configuration

The `contract-matrix` job in `.github/workflows/ci.yml` defines a matrix with the following contracts:

- analytics
- certificate
- mint-batch-certificates
- mobile-optimizer
- progress
- proxy
- search
- student-progress-tracker
- token

Note: The `shared` library is excluded as it's a dependency library, not a deployable contract.

### Job Steps

Each contract job performs the following steps:

1. **Checkout**: Clone the repository
2. **Install Rust**: Setup Rust toolchain with wasm32-unknown-unknown target
3. **Cache**: Cache cargo dependencies per contract
4. **Test**: Run contract-specific tests with `cargo test --package <contract>`
5. **Build (Native)**: Build for native target with `cargo build --package <contract> --release`
6. **Build (WASM)**: Build for WASM target with `cargo build --package <contract> --target wasm32-unknown-unknown --release --lib`
7. **Size Check**: Report the size of the generated WASM file

### Fail-Fast Configuration

The matrix is configured with `fail-fast: false`, meaning:
- If one contract fails, other contracts continue building
- You can see all failing contracts in a single CI run
- No need to fix one contract and re-run to discover the next failure

## Benefits

### For Developers

- **Quick Identification**: Immediately see which contract is broken
- **Targeted Fixes**: Fix only the failing contracts
- **Better Feedback**: Clear, per-contract build status in PR checks

### For CI/CD

- **Parallel Execution**: Faster overall CI completion time
- **Granular Caching**: Per-contract caching improves cache hit rates
- **Resource Efficiency**: Failed contracts don't block successful ones

## Example Output

When viewing a CI run, you'll see individual jobs like:

```
✅ Contract analytics
✅ Contract certificate
❌ Contract mint-batch-certificates
✅ Contract progress
✅ Contract token
```

Instead of a single monolithic build job.

## Testing Locally

You can test individual contracts locally using the same commands the CI uses:

```bash
# Run tests for a specific contract
cargo test --package analytics --verbose

# Build for native target
cargo build --package analytics --release

# Build for WASM target
cargo build --package analytics --target wasm32-unknown-unknown --release --lib
```

## Known Issues

### Shared Library WASM Compatibility

The `shared` library currently has issues building for the `wasm32-unknown-unknown` target due to:

1. Use of standard library types (`Vec<char>`, `String`) in the validation module
2. Missing `#![no_std]` declaration
3. Dependency on features not available in WASM builds

This affects contracts that depend on `shared`. The issue exists in the main branch as well and is not introduced by this PR. Future work should focus on making the `shared` library fully `no_std` compatible.

### Workaround

The CI workflow includes error handling for WASM build failures:
```bash
cargo build ... || {
  echo "::warning::WASM build failed. This may indicate missing no_std compatibility."
  exit 1
}
```

## Future Improvements

1. **Fix Shared Library**: Make the `shared` library fully `no_std` and WASM-compatible
2. **Optimization Metrics**: Add WASM optimization step and track optimized sizes
3. **Performance Tracking**: Track build times and sizes over time
4. **Test Coverage**: Add per-contract test coverage reporting
5. **Contract Versioning**: Automatically tag and version contracts that pass all checks

## Integration with Existing CI

The per-contract matrix integrates with the existing CI pipeline:

- `ci-success` job now depends on `contract-matrix`
- All existing jobs (format, clippy, test, build, wasm-check, dependency-check) remain unchanged
- The matrix provides additional granularity without replacing workspace-level checks

## Related Issues

- Closes #128: Per-Contract WASM Build Matrix in CI

## References

- [GitHub Actions Matrix Strategy](https://docs.github.com/en/actions/using-jobs/using-a-matrix-for-your-jobs)
- [Cargo Workspaces](https://doc.rust-lang.org/book/ch14-03-cargo-workspaces.html)
- [Soroban Smart Contracts](https://soroban.stellar.org/docs)
