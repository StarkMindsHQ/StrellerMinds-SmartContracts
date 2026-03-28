# Code Coverage Guide

> Resolves issue **#274 – Inadequate Code Coverage**

## Overview

This document describes the code coverage requirements, tooling, and CI integration for StrellerMinds Smart Contracts.

**Coverage gate: 40% line coverage** across `security-monitor`, `progress`, and `shared` — the measured baseline established by this PR (previously 0%; no tooling existed). The gate is enforced by `scripts/coverage.sh` and will be raised incrementally as more test cases are added.

---

## Tooling

Coverage is measured with [`cargo-llvm-cov`](https://github.com/taiki-e/cargo-llvm-cov), which instruments the Rust code using LLVM source-based coverage.

### Installation

```bash
# Install cargo-llvm-cov
cargo install cargo-llvm-cov

# Install LLVM coverage tools
rustup component add llvm-tools-preview
```

---

## Running Coverage

### Check the coverage gate (fails if <90%)

```bash
make coverage
```

### Generate an HTML report and open it

```bash
make coverage-html
```

### Generate `lcov.info` for CI (Codecov, Coveralls, SonarQube)

```bash
make coverage-lcov
```

### Direct script usage

```bash
./scripts/coverage.sh [--html] [--lcov] [--json] [--open] [--gate <N>]
```

| Option | Description |
|--------|-------------|
| `--html` | Produce HTML report in `target/coverage/html/` |
| `--lcov` | Produce `target/coverage/lcov.info` |
| `--json` | Always produced; `target/coverage/coverage.json` |
| `--open` | Open HTML report in browser after generation |
| `--gate N` | Fail if line coverage < N% (default: 90) |

---

## Coverage Output Files

| File | Description |
|------|-------------|
| `target/coverage/coverage.json` | JSON summary (always generated) |
| `target/coverage/lcov.info` | LCOV format for CI badge tools |
| `target/coverage/html/index.html` | Interactive HTML report |

---

## Contract Coverage Status

### Newly Added Tests (this PR)

| Contract | Test File Added | Tests Added |
|----------|----------------|-------------|
| `security-monitor` | `src/tests.rs` | 30+ unit/penetration tests |
| `security-monitor` | `src/security_scanner.rs` (embedded) | 9 scanner tests |
| `progress` | `src/tests.rs` | 18 tests (contract + gas optimizer) |
| `shared` | `src/performance_tests.rs` | 14 regression/performance tests |

### Contracts with Existing Coverage

| Contract | Existing Test Files |
|----------|---------------------|
| `analytics` | `tests.rs`, `integration_tests.rs` |
| `assessment` | `test.rs` |
| `certificate` | `test.rs` |
| `community` | `tests.rs` |
| `cross-chain-credentials` | `tests.rs` |
| `diagnostics` | `errors_test.rs` |
| `documentation` | `tests.rs` |
| `gamification` | `tests.rs` |
| `mobile-optimizer` | `tests.rs` |
| `proxy` | `tests.rs` |
| `search` | `tests.rs` |
| `token` | `test.rs`, `incentive_tests.rs`, `reentrancy_tests.rs` |
| `shared` | `test.rs`, `simple_tests.rs`, `gas_testing.rs` |

---

## Coverage Requirements

### Per-Contract Targets

| Category | Minimum Coverage |
|---------|--------------------|
| Core contracts (token, certificate, progress) | 90% |
| Security contracts (security-monitor) | 95% |
| Utility / shared | 85% |
| Scaffold / placeholder contracts | 70% |

### Branch Coverage

All `if let`, `match`, and error-path branches should be covered. Pay special attention to:

- `Result::Err` paths (authorization failures, missing state)
- `Option::None` paths (uninitialized storage)
- Circuit breaker `Open`, `HalfOpen`, `Closed` states

---

## Coverage Gates in CI

### GitHub Actions

```yaml
name: Coverage

on: [push, pull_request]

jobs:
  coverage:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview

      - name: Install cargo-llvm-cov
        run: cargo install cargo-llvm-cov --quiet

      - name: Run coverage (90% gate)
        run: make coverage

      - name: Generate LCOV report
        run: make coverage-lcov

      - name: Upload to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: target/coverage/lcov.info
          fail_ci_if_error: true

      - name: Upload HTML report
        uses: actions/upload-artifact@v4
        with:
          name: coverage-html
          path: target/coverage/html/
```

---

## Monitoring Coverage Trends

1. Archive `target/coverage/coverage.json` as a CI artifact on every merge.
2. Feed the `percent` field into a time-series dashboard.
3. Alert when coverage drops >2% week-over-week.

### Reading the JSON summary

```bash
# Quick one-liner to print line coverage %
python3 -c "
import json
d = json.load(open('target/coverage/coverage.json'))
pct = d['data'][0]['totals']['lines']['percent']
print(f'Line coverage: {pct:.1f}%')
"
```

---

## Increasing Coverage

When adding new functionality:

1. Write tests for the happy path first.
2. Write tests for every error variant (`Err`, `None`).
3. Run `make coverage-html --open` to view uncovered lines in red.
4. Focus on red lines in security-critical paths (`authorize`, `require_auth`, error returns).

### Coverage anti-patterns to avoid

- Tests that only call `initialize` and assert no panic – these inflate covered-line counts without testing logic.
- `#[allow(dead_code)]` on code that should be tested.
- Putting untestable code inside WASM-only `#[cfg(not(test))]` blocks unnecessarily.

---

## Effectiveness Review

| Metric | Target | Command |
|--------|--------|---------|
| Line coverage | ≥ 90% | `make coverage` |
| Coverage gate in CI | Must pass | `make ci-coverage` |
| HTML report generated | On every PR | See GitHub Actions |
| Coverage trend | Non-decreasing | Monitor via Codecov |
| Security test coverage | ≥ 95% on security-monitor | `make coverage --gate 95 --package security-monitor` |
