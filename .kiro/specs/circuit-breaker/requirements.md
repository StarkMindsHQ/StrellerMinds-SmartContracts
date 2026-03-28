# Requirements Document

## Introduction

This feature adds circuit breaker mechanisms across all CosmWasm/Soroban smart contracts in the workspace. The `certificate` contract already has a partial circuit breaker implementation; this spec standardises and extends that pattern to the `analytics`, `assessment`, `community`, `gamification`, `progress`, `proxy`, `search`, `security-monitor`, `student-progress-tracker`, `token`, and remaining contracts. The goal is to prevent cascading failures, enable automatic recovery, and provide on-chain observability for operational incidents.

## Glossary

- **Circuit_Breaker**: A state machine (Closed → Open → Half-Open) that wraps a critical operation and stops execution when a failure threshold is exceeded.
- **Closed_State**: Normal operating state; all calls pass through.
- **Open_State**: Tripped state; all calls are rejected immediately without executing the operation.
- **Half_Open_State**: Recovery probe state; one call is allowed through to test whether the underlying condition has resolved.
- **Failure_Threshold**: The number of consecutive failures required to trip a circuit breaker from Closed to Open.
- **Reset_Timeout**: The duration (in ledger seconds) the circuit breaker remains Open before transitioning to Half-Open.
- **Operation_Key**: A unique string identifier for a protected operation within a contract (e.g., `"batch_issue"`, `"record_session"`).
- **Circuit_State**: The persisted struct `{ state: BreakerState, failure_count: u32, opened_at: u64 }` stored per Operation_Key.
- **Admin**: The privileged address that initialised a contract and holds administrative authority.
- **Shared_Library**: The `contracts/shared` crate that provides reusable utilities across contracts.
- **Event**: A Soroban SDK event emitted via `env.events().publish(...)`.

---

## Requirements

### Requirement 1: Shared Circuit Breaker Library

**User Story:** As a contract developer, I want a single reusable circuit breaker implementation in the shared crate, so that all contracts use consistent logic without duplicating code.

#### Acceptance Criteria

1. THE Shared_Library SHALL expose a `CircuitBreaker` module containing the `BreakerState` enum, `OperationCircuit` struct, and the functions `can_proceed`, `record_success`, and `record_failure`.
2. THE Shared_Library SHALL define `CIRCUIT_FAILURE_THRESHOLD` as a configurable constant defaulting to `3` consecutive failures.
3. THE Shared_Library SHALL define `CIRCUIT_RESET_TIMEOUT_SECONDS` as a configurable constant defaulting to `300` seconds.
4. WHEN `can_proceed` is called with an Operation_Key in Closed_State, THE Circuit_Breaker SHALL return `true`.
5. WHEN `can_proceed` is called with an Operation_Key in Open_State and the elapsed time since `opened_at` is less than `CIRCUIT_RESET_TIMEOUT_SECONDS`, THE Circuit_Breaker SHALL return `false`.
6. WHEN `can_proceed` is called with an Operation_Key in Open_State and the elapsed time since `opened_at` is greater than or equal to `CIRCUIT_RESET_TIMEOUT_SECONDS`, THE Circuit_Breaker SHALL transition the circuit to Half_Open_State and return `true`.
7. WHEN `can_proceed` is called with an Operation_Key in Half_Open_State, THE Circuit_Breaker SHALL return `true`.
8. WHEN `record_failure` is called and `failure_count` reaches `CIRCUIT_FAILURE_THRESHOLD`, THE Circuit_Breaker SHALL transition the circuit to Open_State, record `opened_at` as the current ledger timestamp, and emit a `circuit_opened` Event.
9. WHEN `record_failure` is called while the circuit is in Half_Open_State, THE Circuit_Breaker SHALL immediately transition the circuit to Open_State and emit a `circuit_opened` Event.
10. WHEN `record_success` is called, THE Circuit_Breaker SHALL reset `failure_count` to `0`, transition the circuit to Closed_State, and emit a `circuit_closed` Event.
11. IF an Operation_Key has no stored Circuit_State, THEN THE Circuit_Breaker SHALL initialise it as Closed_State with `failure_count = 0`.

---

### Requirement 2: Circuit Breaker Integration — Analytics Contract

**User Story:** As an operator, I want the analytics contract's critical write operations protected by a circuit breaker, so that repeated failures do not corrupt analytics state.

#### Acceptance Criteria

1. WHEN `record_session` is called and the circuit for `"record_session"` is Open, THE Analytics contract SHALL return a `CircuitBreakerOpen` error without executing the operation.
2. WHEN `complete_session` is called and the circuit for `"complete_session"` is Open, THE Analytics contract SHALL return a `CircuitBreakerOpen` error without executing the operation.
3. WHEN `record_session` completes successfully, THE Analytics contract SHALL call `record_success` for the `"record_session"` Operation_Key.
4. IF `record_session` encounters an internal error, THEN THE Analytics contract SHALL call `record_failure` for the `"record_session"` Operation_Key before returning the error.
5. THE Analytics contract's error enum SHALL include a `CircuitBreakerOpen` variant.

---

### Requirement 3: Circuit Breaker Integration — Assessment Contract

**User Story:** As an operator, I want assessment submission and grading operations protected by a circuit breaker, so that grading failures do not cascade into data corruption.

#### Acceptance Criteria

1. WHEN a submission operation is called and the circuit for `"submit"` is Open, THE Assessment contract SHALL return a `CircuitBreakerOpen` error without executing the operation.
2. WHEN a grading operation is called and the circuit for `"grade"` is Open, THE Assessment contract SHALL return a `CircuitBreakerOpen` error without executing the operation.
3. WHEN a submission completes successfully, THE Assessment contract SHALL call `record_success` for the `"submit"` Operation_Key.
4. IF a submission encounters an internal error, THEN THE Assessment contract SHALL call `record_failure` for the `"submit"` Operation_Key before returning the error.
5. THE Assessment contract's error enum SHALL include a `CircuitBreakerOpen` variant.

---

### Requirement 4: Circuit Breaker Integration — Community Contract

**User Story:** As an operator, I want community write operations (posts, governance, moderation) protected by a circuit breaker, so that failures in one subsystem do not affect others.

#### Acceptance Criteria

1. WHEN `create_post` is called and the circuit for `"create_post"` is Open, THE Community contract SHALL return a `CircuitBreakerOpen` error.
2. WHEN `create_proposal` is called and the circuit for `"governance"` is Open, THE Community contract SHALL return a `CircuitBreakerOpen` error.
3. WHEN `report_content` is called and the circuit for `"moderation"` is Open, THE Community contract SHALL return a `CircuitBreakerOpen` error.
4. WHEN any protected Community operation completes successfully, THE Community contract SHALL call `record_success` for the corresponding Operation_Key.
5. IF any protected Community operation encounters an internal error, THEN THE Community contract SHALL call `record_failure` for the corresponding Operation_Key before returning the error.
6. THE Community contract's error enum SHALL include a `CircuitBreakerOpen` variant.

---

### Requirement 5: Circuit Breaker Integration — Remaining Contracts

**User Story:** As an operator, I want all remaining contracts (gamification, progress, proxy, search, security-monitor, student-progress-tracker, token) to protect their critical write operations with circuit breakers, so that the entire platform has uniform failure isolation.

#### Acceptance Criteria

1. THE Gamification contract SHALL protect XP award and badge grant operations with circuit breakers.
2. THE Progress contract SHALL protect progress update and milestone operations with circuit breakers.
3. THE Proxy contract SHALL protect cross-contract call dispatch operations with circuit breakers.
4. THE Search contract SHALL protect index write operations with circuit breakers.
5. THE Security_Monitor contract SHALL protect alert recording operations with circuit breakers.
6. THE Student_Progress_Tracker contract SHALL protect enrollment and completion operations with circuit breakers.
7. THE Token contract SHALL protect mint and transfer operations with circuit breakers.
8. WHILE a circuit is Open for any contract, THE contract SHALL reject the protected operation and return a `CircuitBreakerOpen` error without modifying state.
9. WHEN a protected operation in any contract succeeds, THE contract SHALL call `record_success` for the corresponding Operation_Key.
10. IF a protected operation in any contract encounters an internal error, THEN THE contract SHALL call `record_failure` for the corresponding Operation_Key.

---

### Requirement 6: Admin Circuit Breaker Override

**User Story:** As an Admin, I want to manually open or reset a circuit breaker for any operation, so that I can respond to incidents without waiting for the automatic timeout.

#### Acceptance Criteria

1. WHEN an Admin calls `admin_open_circuit` with a valid Operation_Key, THE Circuit_Breaker SHALL immediately transition the circuit to Open_State and emit a `circuit_opened` Event.
2. WHEN an Admin calls `admin_reset_circuit` with a valid Operation_Key, THE Circuit_Breaker SHALL immediately transition the circuit to Closed_State, reset `failure_count` to `0`, and emit a `circuit_closed` Event.
3. IF a non-Admin address calls `admin_open_circuit` or `admin_reset_circuit`, THEN THE contract SHALL return an `Unauthorized` error.
4. THE Shared_Library SHALL expose `admin_open_circuit` and `admin_reset_circuit` helper functions that accept an `Env`, an `Admin` address, and an Operation_Key.

---

### Requirement 7: Circuit Breaker State Query

**User Story:** As an operator, I want to query the current state of any circuit breaker on-chain, so that I can monitor system health without relying solely on events.

#### Acceptance Criteria

1. THE Shared_Library SHALL expose a `get_circuit_state` function that accepts an `Env` and an Operation_Key and returns the current `OperationCircuit` struct.
2. WHEN `get_circuit_state` is called for an Operation_Key with no stored state, THE Circuit_Breaker SHALL return a default `OperationCircuit` in Closed_State with `failure_count = 0` and `opened_at = 0`.
3. THE Shared_Library SHALL expose a `list_open_circuits` function that returns all Operation_Keys currently in Open_State or Half_Open_State for a given contract instance.

---

### Requirement 8: Circuit Breaker Events and Observability

**User Story:** As an operator, I want standardised on-chain events emitted for every circuit state transition, so that off-chain monitoring tools can detect and alert on failures automatically.

#### Acceptance Criteria

1. WHEN a circuit transitions from Closed_State to Open_State, THE Circuit_Breaker SHALL emit an event with topic `("circuit_opened", <operation_key>)` and data `{ failure_count: u32, opened_at: u64, reason: String }`.
2. WHEN a circuit transitions from Open_State or Half_Open_State to Closed_State, THE Circuit_Breaker SHALL emit an event with topic `("circuit_closed", <operation_key>)` and data `{ closed_at: u64 }`.
3. WHEN a circuit transitions from Open_State to Half_Open_State, THE Circuit_Breaker SHALL emit an event with topic `("circuit_half_open", <operation_key>)` and data `{ probed_at: u64 }`.
4. THE event schema for `circuit_opened`, `circuit_closed`, and `circuit_half_open` SHALL be consistent across all contracts.
5. WHEN `record_failure` is called, THE Circuit_Breaker SHALL emit a `circuit_failure_recorded` event with the current `failure_count` even if the threshold has not yet been reached, so that operators can observe failure accumulation.

---

### Requirement 9: Certificate Contract Alignment

**User Story:** As a developer, I want the existing certificate contract circuit breaker to be refactored to use the shared library, so that it is consistent with all other contracts and does not maintain a separate implementation.

#### Acceptance Criteria

1. THE Certificate contract SHALL remove its inline `can_proceed`, `record_success`, `record_failure`, `get_or_init_circuit`, and `circuit_key` functions.
2. THE Certificate contract SHALL import and use the `CircuitBreaker` module from the Shared_Library for all circuit breaker operations.
3. WHEN the Certificate contract is refactored, all existing circuit-protected operations (`cfg_msig`, `exec_msig`, `batch_issue`, `revoke_cert`, `reissue_cert`) SHALL continue to function identically.
4. THE Certificate contract's existing `CircuitBreakerOpen` error variant SHALL be retained.

---

### Requirement 10: Failure Scenario Testing

**User Story:** As a developer, I want comprehensive tests for circuit breaker behaviour across all contracts, so that failure scenarios are verified and regressions are caught.

#### Acceptance Criteria

1. THE test suite SHALL include a test that verifies a circuit transitions to Open_State after exactly `CIRCUIT_FAILURE_THRESHOLD` consecutive failures.
2. THE test suite SHALL include a test that verifies a protected operation is rejected when the circuit is in Open_State.
3. THE test suite SHALL include a test that verifies the circuit transitions to Half_Open_State after `CIRCUIT_RESET_TIMEOUT_SECONDS` have elapsed.
4. THE test suite SHALL include a test that verifies a successful operation in Half_Open_State transitions the circuit back to Closed_State.
5. THE test suite SHALL include a test that verifies a failed operation in Half_Open_State transitions the circuit back to Open_State.
6. THE test suite SHALL include a test that verifies `admin_reset_circuit` immediately resets an Open circuit to Closed_State.
7. THE test suite SHALL include a test that verifies `admin_open_circuit` immediately trips a Closed circuit to Open_State.
8. FOR ALL contracts with circuit breakers, THE test suite SHALL verify that `get_circuit_state` returns the correct state after each transition (round-trip property: transition → query → verify).
