# Monitoring Guide

This repository now includes a modular monitoring baseline for smart contracts and supporting infrastructure. The implementation is intentionally additive so we can improve observability without changing unrelated contract behavior.

## What Was Added

- `shared::monitoring` for reusable counter and gauge metrics
- event-based metric hooks that can be scraped or forwarded by an off-chain collector
- a Grafana dashboard definition in `monitoring/grafana/contracts-overview.json`
- Prometheus alert rules in `monitoring/alerts/contract-observability.yml`

## Metrics Model

The shared monitoring module records:

- counters for total contract calls
- counters for success and failure totals
- gauges for the latest observed latency
- gauges for infrastructure backlog values

Metrics are stored in contract storage and also emitted as Soroban events under the `monitoring` topic. This gives us two integration options:

1. read snapshots directly from contract state for diagnostics and tests
2. stream events into Prometheus-compatible exporters or log pipelines

## Integration Pattern

Use the helper in any contract or supporting monitor:

```rust
use shared::monitoring::Monitoring;
use soroban_sdk::Symbol;

let contract = Symbol::new(&env, "token");
Monitoring::record_contract_call(&env, &contract, true, 28);
```

For infrastructure jobs:

```rust
let pipeline = Symbol::new(&env, "indexer");
Monitoring::record_infra_backlog(&env, &pipeline, 12);
```

## Dashboard

The supplied Grafana dashboard focuses on:

- contract throughput
- failure rate
- latency
- ledger lag
- queue backlog

Import `monitoring/grafana/contracts-overview.json` into Grafana and point the panels at your Prometheus datasource.

## Alerts

The default Prometheus rules cover:

- elevated contract failure rate
- sustained RPC latency
- backlog growth for off-chain processing
- ledger synchronization lag

These are starter thresholds and should be tuned per environment.

## Operational Notes

- Keep metric names short enough for Soroban `Symbol` keys.
- Prefer recording metrics at contract boundaries and off-chain workers, not inside deep business logic branches.
- Mirror important production thresholds in dashboard annotations and runbooks.
