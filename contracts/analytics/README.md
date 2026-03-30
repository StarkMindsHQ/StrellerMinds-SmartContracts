# Analytics Contract

## Purpose

The Analytics contract is the on-chain event ledger for learning session data on the StrellerMinds platform. It records the start and completion of individual study sessions identified by unique 32-byte session IDs, emitting structured events on each transition. These events feed off-chain pipelines and the platform's reporting dashboards, providing auditable evidence of learner engagement that complements the progress and student-progress-tracker contracts.

## Architecture

| Module | Description |
|---|---|
| `src/lib.rs` | Contract entrypoint — exposes `initialize`, `record_session`, `complete_session`, `get_session`, and `get_admin`; emits `SessionRecorded` / `SessionCompleted` events via shared macros |
| `src/errors.rs` | `AnalyticsError` enum covering initialization state, authorization, session validation, data quality, and oracle trust |
| `src/gas_optimized.rs` | Gas-optimized batch session recording utilities |

## Public API

| Function | Description | Auth Required |
|---|---|---|
| `initialize(admin)` | One-time setup; records the admin address | No (open, call once) |
| `record_session(user, session_id)` | Records the start of a 32-byte-identified learning session for `user`; emits `SessionRecorded` | Yes — `user` |
| `complete_session(user, session_id)` | Marks a previously recorded session as complete; emits `SessionCompleted` | Yes — `user` |
| `get_session(session_id)` | Returns the session data for a given session ID, or `None` | No |
| `get_admin()` | Returns the admin address, or `None` if not initialized | No |

## Usage Example

```text
# Initialize
analytics.initialize(admin_address)

# Student starts a study session (session_id is a 32-byte unique identifier)
session_id = generate_unique_session_id()
analytics.record_session(student_address, session_id)

# Student finishes the session
analytics.complete_session(student_address, session_id)

# Verify the session exists
session_data = analytics.get_session(session_id)
```

## Reliability: Circuit Breaker

- `record_session` and `complete_session` are protected by a circuit breaker.
- Breaker states: `Closed`, `Open`, `HalfOpen`.
- Emitted monitoring topics: `config`, `failure`, `open`, `halfopen`, `closed`, `blocked`, `reset`.
- Because reverted Soroban transactions roll back state, persisted failure increments are handled by explicit `report_operation_failure` calls from authorized operators.

## Errors

For the full error code reference and conventions, see [ERROR_HANDLING.md](../../docs/ERROR_HANDLING.md).

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | `initialize` has already been called |
| 2 | `NotInitialized` | Contract has not been initialized yet |
| 3 | `Unauthorized` | Caller is not authorized |
| 4 | `InvalidSessionData` | Session data is malformed or missing required fields |
| 5 | `InvalidTimeRange` | Time range is invalid or out of order |
| 6 | `InvalidScore` | Score value is out of valid bounds |
| 7 | `InvalidPercentage` | Percentage value is outside 0–100 |
| 8 | `SessionTooShort` | Session duration is below the minimum threshold |
| 9 | `SessionTooLong` | Session duration exceeds the maximum threshold |
| 10 | `SessionNotFound` | No session found for the given ID |
| 11 | `StudentNotFound` | No data found for the given student address |
| 12 | `CourseNotFound` | No data found for the given course ID |
| 13 | `ModuleNotFound` | No data found for the given module ID |
| 14 | `ReportNotFound` | No report found for the given ID |
| 15 | `SessionAlreadyExists` | A session with this ID has already been recorded |
| 16 | `SessionNotCompleted` | Operation requires the session to be in completed state |
| 17 | `InsufficientData` | Not enough data to compute the requested result |
| 18 | `InvalidBatchSize` | Batch size is zero or exceeds the allowed maximum |
| 19 | `StorageError` | An internal storage read or write failed |
| 20 | `InvalidConfiguration` | Contract configuration is missing or invalid |
| 21 | `UnauthorizedOracle` | Oracle address is not registered as a trusted source |
| 22 | `InvalidInsightData` | Insight data is malformed or missing required fields |
| 23 | `InsightNotFound` | No insight record found for the given ID |

## Integration

| Contract | Relationship |
|---|---|
| `progress` | Emits `ProgressUpdated` events that analytics pipelines correlate with session data |
| `student-progress-tracker` | Emits `ProgressUpdated` events at module granularity for analytics aggregation |
| `token` | Emits `TokensMinted` / `TokensTransferred` events that analytics may include in reward reports |
| `shared` | Uses event schema macros (`emit_analytics_event!`, `emit_access_control_event!`) |
