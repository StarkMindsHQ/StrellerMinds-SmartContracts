# Security Monitor Contract

## Overview

The `security-monitor` contract tracks threats, metrics, circuit-breaker state, threat intelligence, user risk scores, and recommendations for operational and contract-level security monitoring.

## Quick Start

```bash
cargo test -p security-monitor
```

Initialize the contract with an admin and `SecurityConfig`, then record events, scan for threats, and manage threat intelligence through the entrypoints in [lib.rs](./src/lib.rs).

## Usage Examples

```rust
// Initialize the security monitor
SecurityMonitor::initialize(env, admin, config)?;

// Request deeper anomaly analysis
SecurityMonitor::request_anomaly_analysis(env, actor, contract_symbol)?;

// Update threat intelligence
SecurityMonitor::update_threat_intelligence(env, admin, intel)?;
```

## Contribution Guide

- Keep changes focused on detection, monitoring, recommendations, storage, or incident reporting.
- Avoid changing public threat data formats without updating tests and related docs.
- Add or update tests in `src/tests.rs` for new threat types, metrics handling, or mitigation flows.

## Troubleshooting

- `NotInitialized`: initialize the contract before storing security configuration or metrics.
- `MetricsNotFound`: confirm the expected contract symbol and time window were written before querying.
- `RateLimitExceeded`: back off the caller or tune the security configuration for the affected environment.

## Related Files

- `src/lib.rs`: contract entrypoints
- `src/threat_detector.rs`: threat detection logic
- `src/security_scanner.rs`: scoring and scan support
- `src/tests.rs`: unit and integration-style tests
