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

## Schema Versioning

Events include an `EVENT_SCHEMA_VERSION` (currently `1`). Off-chain listeners should check this version to ensure data compatibility.
