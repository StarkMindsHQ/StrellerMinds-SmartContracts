# Monitoring Procedures

## Overview

StrellerMinds smart contracts use an **events-based monitoring architecture**. Each contract emits standardized `Monitoring` events via the `StandardEvent` schema. Off-chain indexers (Mercury, custom Hubble sinks) capture and aggregate these events for dashboards and alerting.

**Key principle**: Health reports are emitted as events, not stored on-chain, keeping gas costs minimal.

## Architecture

```
Contracts (on-chain)         Off-chain Infrastructure
+------------------+        +---------------------+
| Token            |------->| Mercury/Hubble      |
| Analytics        |        | Indexer              |
| Certificate      |  emit  +----------+----------+
| Progress         |------->           |
| Assessment       |                   v
| Community        |        +---------------------+
| Gamification     |        | Dashboard           |
| Search           |        | (Grafana/custom)    |
| StudentTracker   |        +----------+----------+
| CrossChain       |                   |
+------------------+                   v
                             +---------------------+
                             | Alerting            |
                             | (PagerDuty/Slack)   |
                             +---------------------+
```

## Health Check Endpoints

Every contract exposes a `health_check()` function that returns a `ContractHealthReport`:

```rust
pub struct ContractHealthReport {
    pub contract_id: Symbol,
    pub status: ContractHealthStatus, // Healthy, Degraded, Unhealthy, Unknown
    pub timestamp: u64,
    pub initialized: bool,
    pub error_count: u32,
    pub custom_metrics: Vec<MetricSnapshot>,
}
```

### Calling Health Checks

Using the Soroban CLI:

```bash
# Check a single contract
soroban contract invoke \
  --id <CONTRACT_ID> \
  --source <IDENTITY> \
  --network testnet \
  -- health_check

# Contracts with health_check endpoints:
# token, analytics, progress, certificate, assessment,
# community, gamification, search, student-progress-tracker,
# cross-chain-credentials
```

### Health Status Meanings

| Status | Value | Meaning |
|--------|-------|---------|
| `Healthy` | 0 | Contract initialized and operating normally |
| `Degraded` | 1 | Contract functional but with issues (e.g., high error count) |
| `Unhealthy` | 2 | Contract experiencing critical issues |
| `Unknown` | 3 | Contract not initialized or status cannot be determined |

## Event Query Patterns

All monitoring events use the `StandardEvent` schema with topic structure:

```
("standard_event", <contract_id>, "monitoring", <event_type>, <actor>)
```

### Mercury/Hubble Indexer Queries

**Health check events:**
```sql
SELECT * FROM events
WHERE topic_0 = 'standard_event'
  AND topic_2 = 'monitoring'
  AND topic_3 = 'health_check'
ORDER BY timestamp DESC;
```

**Alert events:**
```sql
SELECT * FROM events
WHERE topic_0 = 'standard_event'
  AND topic_2 = 'monitoring'
  AND topic_3 = 'alert_triggered'
ORDER BY timestamp DESC;
```

**Metric events:**
```sql
SELECT * FROM events
WHERE topic_0 = 'standard_event'
  AND topic_2 = 'monitoring'
  AND topic_3 = 'metric_recorded'
ORDER BY timestamp DESC;
```

**Alert resolution events:**
```sql
SELECT * FROM events
WHERE topic_0 = 'standard_event'
  AND topic_2 = 'monitoring'
  AND topic_3 = 'alert_resolved'
ORDER BY timestamp DESC;
```

## Dashboard Panels

### 1. System Overview

A table showing the latest health status of each contract:

| Contract | Status | Last Check | Error Count | Initialized |
|----------|--------|------------|-------------|-------------|
| token | Healthy | 2024-01-15 12:00 | 0 | Yes |
| certificate | Healthy | 2024-01-15 12:00 | 0 | Yes |
| ... | ... | ... | ... | ... |

**Query**: Latest `health_check` event per `contract_id`.

### 2. Alert Timeline

Time-series chart showing alert events grouped by contract and severity level.

**Query**: All `alert_triggered` events, grouped by `alert_level` (2=Warning, 4=Critical).

### 3. Circuit Breaker Status

Current state of all circuit breakers from the error handling system.

**Query**: Latest `SecurityEventData` events with circuit breaker state changes.

### 4. Error Rate

Line chart showing error events per contract over a sliding time window.

**Query**: Count of `ErrorEventData` events per contract per 5-minute window.

### 5. Security Threat Score

From the security-monitor contract's threat detection events.

**Query**: Latest `SecurityEventData::UserRiskScoreUpdated` events.

## Alert Routing Rules

| Alert Level | Value | Routing | Response Time |
|-------------|-------|---------|---------------|
| Critical | 4 | PagerDuty / on-call engineer | Immediate |
| High | 3 | Slack `#alerts` channel | < 30 minutes |
| Warning | 2 | Slack `#monitoring` channel | < 4 hours |
| Info | 1 | Log only | Review in daily standup |

## Configuring Alert Rules

Alert rules evaluate per-metric thresholds. Configure them in your contract's admin setup:

```rust
use shared::monitoring::{AlertRule, ThresholdComparison};
use shared::error_handling::AlertConfig;

// Create config with custom rules
let mut rules = Vec::new(&env);
rules.push_back(AlertRule {
    metric_name: symbol_short!("errors"),
    warning_threshold: 10,
    critical_threshold: 50,
    comparison: ThresholdComparison::GreaterThan,
});
rules.push_back(AlertRule {
    metric_name: symbol_short!("uptime"),
    warning_threshold: 90,    // Alert if uptime drops below 90%
    critical_threshold: 50,   // Critical if below 50%
    comparison: ThresholdComparison::LessThan,
});

let config = AlertConfig::new(&env, admin).with_rules(rules);
```

### Threshold Evaluation

Use `Monitor::check_thresholds()` to evaluate rules against collected metrics:

```rust
use shared::monitoring::Monitor;

let alerts = Monitor::check_thresholds(&env, &config.alert_rules, &report.custom_metrics);
for alert in alerts.iter() {
    Monitor::emit_alert(
        &env,
        contract_id.clone(),
        alert.level,
        alert.metric_name.clone(),
        alert.current_value,
        alert.threshold_value,
    );
}
```

## Escalation Procedures

1. **Critical Alert Triggered**
   - On-call engineer paged immediately
   - Check circuit breaker state: if open, follow [Circuit Breaker Runbook](CIRCUIT_BREAKER_RUNBOOK.md)
   - Call `health_check()` on affected contract to get current status
   - If contract reports `Unhealthy`: investigate recent error events, check for failed transactions

2. **Warning Alert Triggered**
   - Post to `#monitoring` channel with contract ID and metric details
   - Review recent error events for the affected contract
   - If error count trending upward: investigate root cause before it becomes critical
   - Consider preemptively degrading non-essential features via `GracefulDegradation`

3. **Health Check Returns Unknown**
   - Contract may not be initialized — check deployment status
   - Verify contract address is correct
   - Re-deploy if necessary

4. **Multiple Contracts Unhealthy**
   - Potential network-wide issue — check Stellar network status
   - Review recent deployments or configuration changes
   - Escalate to engineering lead

## Integration with Existing Systems

- **Circuit Breaker Runbook**: See [CIRCUIT_BREAKER_RUNBOOK.md](CIRCUIT_BREAKER_RUNBOOK.md) for circuit breaker recovery procedures
- **Security Monitor**: The `security-monitor` contract provides additional threat detection and risk scoring
- **Diagnostics**: The `diagnostics` contract offers deeper performance profiling and anomaly detection
- **Error Handling**: The shared `error_handling` module integrates with monitoring via `HealthCheck::emit_health_status()`

## Testing Monitoring Effectiveness

Run the monitoring test suite to verify the infrastructure:

```bash
# Run all monitoring-related tests
cargo test --workspace monitoring
cargo test --workspace health_check

# Run the shared monitoring unit tests
cargo test -p shared monitoring_tests

# Run contract-level health check integration tests
cargo test -p token test_health_check
cargo test -p certificate test_health_check
cargo test -p cross-chain-credentials test_health_check
```
