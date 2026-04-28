# StrellerMinds Load Testing

This repository includes a contract load-testing suite for scalability and performance benchmarking. The suite runs in Soroban's host-side test environment, which keeps CI stable while still stressing contract execution paths, storage writes, and repeated reads at a configurable 10x peak multiplier.

## What the suite measures

- Latency and duration per scenario
- Throughput in operations per second
- Failed operations
- Emitted contract events
- Estimated persistent state writes
- Process RSS memory delta on Linux runners when available
- Bottleneck notes generated from p95 latency, throughput, and write intensity heuristics

## Scenarios

-   **E2E Load Tests**: Located in `e2e-tests/tests/load_testing.rs`. These tests run against a live Soroban network (e.g., localnet).
-   **Internal Benchmarks**: Located in `contracts/token/src/benchmarks.rs`. These are unit tests that measure execution efficiency within the `soroban-sdk` test environment.
-   **CI/CD Integration**: Automated daily runs via GitHub Actions (`.github/workflows/load-testing.yml`) to track baseline metrics and alert on performance regressions.

1. `progress-write-hot-path`
   Runs repeated `record_progress` calls against a tight course set to stress hot writes.
2. `progress-write-multi-course`
   Spreads `record_progress` calls across more courses to grow student course lists and storage churn.
3. `progress-read-heavy`
   Seeds progress data, then benchmarks `get_progress` and `get_student_courses` under sustained reads.

### 1. High Volume Analytics Recording
-   **Scenario**: Simulates multiple students recording learning sessions simultaneously.
-   **Component**: `analytics` contract.
-   **Goal**: Measure throughput (ops/sec) and ensure non-blocking transaction flow.
-   **Running**: `LOAD_TEST_REQUESTS=10000 cargo test --test load_testing test_load_analytics_recording_stress -- --ignored`

The runner simulates:

- `peak load * load multiplier` write operations
- `(peak load * load multiplier) * read multiplier` read operations

By default, the multiplier is `10`, which gives the required 10x peak simulation. All values are configurable through environment variables.

## Local usage

| Metric | Target |
| :--- | :--- |
| Recording Throughput | > 50 ops/sec (Localnet for 10k users) |
| Leaderboard Latency | < 2000ms (10,000 users) |
| Diagnostics Overhead | < 15% CPU increase |
| Recovery Time | < 5s after surge |

```bash
./scripts/load_test.sh
```

Or through cargo:

```bash
cargo load-test -- --report target/load-test-report.json --summary target/load-test-summary.md
```

Or through Make:

```bash
make load-test
```

## CI-safe usage

The CI workflow uses bounded defaults to keep the job deterministic:

```bash
./scripts/load_test.sh --ci
```

Or:

```bash
make load-test-ci
```

## Environment variables

| Variable | Purpose | Local Default | CI Default |
| --- | --- | ---: | ---: |
| `STRELLER_PEAK_LOAD` | Baseline peak load before multiplying | `25` | `5` |
| `STRELLER_LOAD_MULTIPLIER` | Stress multiplier | `10` | `10` |
| `STRELLER_STUDENT_POOL` | Number of simulated students | `50` | `20` |
| `STRELLER_COURSE_COUNT` | Number of simulated courses | `8` | `4` |
| `STRELLER_READ_MULTIPLIER` | Read operations per seeded write | `3` | `2` |

Example:

```bash
STRELLER_PEAK_LOAD=40 \
STRELLER_LOAD_MULTIPLIER=10 \
STRELLER_STUDENT_POOL=80 \
STRELLER_COURSE_COUNT=12 \
./scripts/load_test.sh
```

## Reports

Each run produces:

- `target/load-test-report.json` - machine-readable metrics for CI artifacts
- `target/load-test-summary.md` - human-readable scenario summary and bottleneck notes

You can override both paths with:

```bash
./scripts/load_test.sh --report target/custom-load-report.json --summary target/custom-load-summary.md
```

## Reading the output

Focus on these fields first:

- `throughput_ops_per_sec`
- `avg_latency_ms`
- `p95_latency_ms`
- `failed_operations`
- `estimated_state_writes`

If the generated bottleneck notes mention tail latency or write intensity, the usual next step is to inspect storage-heavy contract code paths, symbol creation patterns, and list growth in persistent storage.

## CI integration

The main CI workflow includes a dedicated `Load Testing` job that:

1. Uses bounded environment defaults
2. Runs `./scripts/load_test.sh --ci`
3. Uploads the JSON and Markdown reports as workflow artifacts

This keeps the suite safe for pull requests while still catching regressions in latency, throughput, and storage-heavy paths.
