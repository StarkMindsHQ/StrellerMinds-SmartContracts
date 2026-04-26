# StrellerMinds Off-Chain Event Monitoring Guide

This guide explains how to monitor, filter, and index events emitted by StrellerMinds smart contracts using the standardized `shared` event schema.

## Event Structure

All events follow a unified topic structure for efficient indexing:

| Topic Index | Content | Description |
| --- | --- | --- |
| 0 | `category` | The broad functional area (e.g., `Certification`, `Gamification`). |
| 1 | `sub_type` | Specific action (e.g., `Issued`, `Staked`, `Verified`). |
| 2 | `actor` | (Optional) The `Address` of the user/admin who triggered the action. |
| 3 | `identifier` | (Optional) A specific ID like `course_id` or `certificate_id`. |

## Examples

### 1. Monitoring Course Progress

To track when students complete modules, monitor the `Progress` category:

- **Filter**:
  - `Topic[0] == symbol_short!("progress")`
  - `Topic[1] == symbol_short!("complete")`
- **Data**: Extract `student`, `course_id`, and `module_id` from the event data payload.

### 2. Auditing Certifications

To monitor all new certifications issued across the platform:

- **Filter**:
  - `Topic[0] == symbol_short!("certification")`
  - `Topic[1] == symbol_short!("issued")`
- **Data**: Retrieve the `student` address and `certificate_id` for on-chain verification.

## Indexing Strategy

It is recommended to use an indexer (like Mercury or a custom Hubble sink) to:

1. **Denormalize**: Map the `StandardEvent` wrapper fields (`timestamp`, `contract`, `actor`) into a Postgres/MongoDB collection.
2. **Alerting**: Set up webhooks for `Security` category events (e.g., `CircuitBreakerOpened`) to notify admins immediately.
3. **Analytics**: Aggregate `Gamification` events to build real-time leaderboards without re-querying contract state.

## Monitoring Events

The `Monitoring` event category provides standardized health check, metrics, and alerting events. These are emitted by each contract's `health_check()` endpoint and by the shared monitoring utilities.

### Event Types

| Event Type | Topic[3] | Description |
| --- | --- | --- |
| `HealthCheck` | `health_check` | Emitted when a contract's health is checked. Contains status, timestamp, and details. |
| `MetricRecorded` | `metric_recorded` | Emitted when a performance metric is recorded. Contains metric name and value. |
| `AlertTriggered` | `alert_triggered` | Emitted when a metric breaches a configured threshold. Contains alert level and values. |
| `AlertResolved` | `alert_resolved` | Emitted when a previously triggered alert is resolved. |

### Filtering Monitoring Events

All monitoring events use the `StandardEvent` topic structure:

```
("standard_event", <contract_id>, "monitoring", <event_type>, <actor>)
```

**Mercury query for health checks across all contracts:**
```sql
SELECT * FROM events
WHERE topic_0 = 'standard_event'
  AND topic_2 = 'monitoring'
  AND topic_3 = 'health_check'
ORDER BY ledger_sequence DESC;
```

**Mercury query for active alerts:**
```sql
SELECT * FROM events
WHERE topic_0 = 'standard_event'
  AND topic_2 = 'monitoring'
  AND topic_3 = 'alert_triggered'
  AND ledger_timestamp > NOW() - INTERVAL '1 hour'
ORDER BY ledger_sequence DESC;
```

### Contracts with Health Check Endpoints

All major contracts expose a `health_check()` function: `token`, `analytics`, `progress`, `certificate`, `assessment`, `community`, `gamification`, `search`, `student-progress-tracker`, and `cross-chain-credentials`.

For detailed monitoring setup, dashboard specifications, and escalation procedures, see [MONITORING_PROCEDURES.md](MONITORING_PROCEDURES.md).

## Schema Versioning

Events include an `EVENT_SCHEMA_VERSION` (currently `1`). Off-chain listeners should check this version to ensure data compatibility.
