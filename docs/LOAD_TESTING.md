# StrellerMinds Load Testing & Performance Benchmarks

This document outlines the load testing suite and performance benchmarks implemented for the StrellerMinds smart contracts.

## 🚀 Overview

The load testing suite is designed to evaluate how the system behaves under stress, specifically focusing on transaction throughput, resource consumption (CPU/Memory), and latency when multiple users interact with the contracts.

## 📁 Structure

-   **E2E Load Tests**: Located in `e2e-tests/tests/load_testing.rs`. These tests run against a live Soroban network (e.g., localnet).
-   **Internal Benchmarks**: Located in `contracts/token/src/benchmarks.rs`. These are unit tests that measure execution efficiency within the `soroban-sdk` test environment.

## 🧪 Load Scenarios

### 1. High Volume Analytics Recording
-   **Scenario**: Simulates multiple students recording learning sessions simultaneously.
-   **Component**: `analytics` contract.
-   **Goal**: Measure throughput (ops/sec) and ensure non-blocking transaction flow.
-   **Running**: `cargo test --test load_testing test_load_analytics_recording_stress -- --ignored`

### 2. Leaderboard Generation Performance
-   **Scenario**: Measures the time taken to generate leaderboards with a large set of student data.
-   **Component**: `analytics` contract.
-   **Goal**: Ensure leaderboard sorting and calculation remain efficient as the user base grows.
-   **Running**: `cargo test --test load_testing test_load_leaderboard_generation_performance -- --ignored`

### 3. Diagnostics Monitoring Overhead
-   **Scenario**: Compares transaction execution time with and without active diagnostics monitoring.
-   **Component**: `diagnostics` & `token` contracts.
-   **Goal**: Quantify the performance "cost" of real-time monitoring.
-   **Running**: `cargo test --test load_testing test_load_diagnostics_overhead -- --ignored`

### 4. Token Mint/Transfer Load
-   **Scenario**: Performs 100+ mint and transfer operations in a single test environment.
-   **Component**: `token` contract.
-   **Goal**: Benchmark gas efficiency and CPU budget utilization.
-   **Running**: `cargo test -p token benchmark_token_load`

## 📊 Success Criteria

| Metric | Target |
| :--- | :--- |
| Recording Throughput | > 10 ops/sec (Localnet) |
| Leaderboard Latency | < 500ms (100 users) |
| Diagnostics Overhead | < 15% CPU increase |
| Recovery Time | < 5s after surge |

## 🛠️ Monitoring Load Performance

We use the `diagnostics` contract to monitor performance during load tests. Key metrics tracked include:
-   `average_execution_time`: Average time per transaction.
-   `gas_used`: Resource consumption per operation.
-   `error_rate`: Percentage of failed transactions under load.

## 📝 Reviewing Effectiveness

Load tests should be reviewed:
1.  **After major refactors**: To ensure no performance regressions.
2.  **Before mainnet deployment**: To validate system capacity.
3.  **When adding new features**: To check impact on existing throughput.

---

*Verified on: 2026-03-27*
