# Incident Runbook: Circuit Breaker and Failure Recovery

## Severity Matrix

- Sev1: Multiple critical contract circuits open or active exploit indicators.
- Sev2: Single critical circuit open with production impact.
- Sev3: Repeated failures approaching threshold with no user-visible outage.

## Ownership and Roles

- Incident Commander: on-call smart-contract engineer.
- Operations Lead: monitors alerts and coordinates triage.
- Recovery Approver: contract admin/multisig owner for manual reset.

## Monitoring and Alert Conditions

- Alert when any `circuit_opened` event is emitted.
- Alert when one operation emits 2+ failures within 5 minutes.
- Alert when a circuit remains open longer than reset timeout + 2 minutes.
- Alert when a circuit repeatedly flaps between `open` and `half_open`.

## Immediate Response

1. Identify affected contract and operation from emitted event topics.
2. Validate if failure source is auth abuse, dependency failure, or invalid input burst.
3. Pause related operational workflows and notify stakeholders.
4. Confirm if manual intervention is required (admin/multisig approval).

## Recovery Procedure

1. Confirm root cause is removed or mitigated.
2. Execute a low-risk validation transaction for the affected operation.
3. Observe transition to `half_open` then `closed` with successful execution.
4. If failure repeats, keep circuit open and escalate incident severity.

## Verification Checklist Before Re-enabling

- Root cause documented.
- At least one successful validation transaction completed.
- No new failure spikes in the last 15 minutes.
- Alert stream stable and no flapping observed.
- Post-incident notes shared with engineering and operations.

## Post-Incident Actions

- Add or refine tests for the exact failure mode.
- Update thresholds/timeouts if false positives occurred.
- Track corrective actions in the reliability backlog.
