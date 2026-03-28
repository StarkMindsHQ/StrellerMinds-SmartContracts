# Circuit Breaker Operations Runbook

This runbook defines failure handling, recovery flow, and incident response for contract-level circuit breakers.

## Scope

- `contracts/analytics`
- `contracts/student-progress-tracker`

Both contracts implement:
- Closed/Open/Half-Open states
- Configurable failure thresholds and cooldown windows
- Admin reset and configuration endpoints
- On-chain monitoring events for transitions and blocked calls

## Failure Detection Model

### Signal Sources

- **Automated operation failures**: business logic failures are surfaced to callers via contract errors.
- **Persisted breaker failure counts**: operators call `report_operation_failure` after confirmed upstream/downstream failure conditions.

### Why explicit failure reporting exists

Soroban rolls back state changes on reverted transactions. For this reason, failure counters that must persist are recorded through successful management calls (`report_operation_failure`) by authorized operators.

## Monitoring & Alerting

Monitor these event topics from both contracts:

- `(circuit, config)` - breaker configuration changed
- `(circuit, failure)` - operational failure explicitly reported
- `(circuit, open)` - breaker opened (alert)
- `(circuit, halfopen)` - cooldown expired, probe mode active
- `(circuit, closed)` - service recovered
- `(circuit, blocked)` - requests blocked while open or half-open saturated
- `(circuit, reset)` - operator forced breaker reset

## Incident Severity Guide

- **SEV-2**: breaker open for under 15 minutes, partial service degradation
- **SEV-1**: breaker open for 15+ minutes or repeated reopen cycles

## Triage Checklist

1. Confirm current breaker state via `get_circuit_breaker_status`.
2. Check latest `(circuit, open)` and `(circuit, blocked)` events.
3. Validate external dependencies and upstream contract health.
4. If root cause persists, keep circuit open and continue mitigation.
5. If root cause is fixed, allow cooldown or perform controlled reset.

## Recovery Procedures

### Automatic Recovery

1. Wait for configured `recovery_timeout_seconds`.
2. Contract transitions Open -> Half-Open on next allowed call.
3. Successful probe calls close the circuit automatically.

### Manual Recovery

1. Verify dependency health and error budget status.
2. Execute `reset_circuit_breaker(admin)`.
3. Monitor for reopen events over the next 10-30 minutes.

### If Recovery Fails

1. Re-open incident timeline.
2. Increase cooldown and/or lower traffic via operational controls.
3. Re-run dependency diagnostics.
4. Escalate to platform incident commander.

## Contract Operations Reference

### Analytics

- `configure_circuit_breaker(admin, failure_threshold, recovery_timeout_seconds, half_open_max_calls, half_open_success_threshold)`
- `report_operation_failure(admin)`
- `reset_circuit_breaker(admin)`
- `get_circuit_breaker_status()`

### Student Progress Tracker

- `configure_circuit_breaker(admin, failure_threshold, recovery_timeout_seconds, half_open_max_calls, half_open_success_threshold)`
- `report_operation_failure(admin)`
- `reset_circuit_breaker(admin)`
- `get_circuit_breaker_status()`

## Post-Incident Actions

1. Record failure pattern and recovery latency.
2. Tune breaker thresholds if false positives were observed.
3. Add/expand test cases for reproduced scenario.
4. Update this runbook with lessons learned.