# Shared Library

## Purpose

The `shared` crate is the foundational library that all 16 contracts in the StrellerMinds workspace depend on. It is not a deployable contract — it is a Rust library crate (`#![no_std]`) compiled into each consuming contract at build time. It provides the cross-cutting concerns that would otherwise be duplicated: role-based access control (RBAC), reentrancy protection, circuit breaker state management, standardized event schemas and emission utilities, input validation helpers, and gas optimization utilities. Any logic that is common across two or more contracts belongs here rather than being inlined.

## Architecture

| Module | Description |
|---|---|
| `lib.rs` | Crate root — re-exports all public sub-modules |
| `access_control` (inline in `lib.rs`) | `AccessControl` struct — `initialize(env, admin)` initializes role-based access control |
| `reentrancy_guard` (inline in `lib.rs`) | `ReentrancyLock` struct — guards against re-entrant calls within a single contract invocation |
| `roles` (inline in `lib.rs`) | `Permission` struct — represents a discrete permission token used in role checking |
| `error_handling` (inline in `lib.rs`) | `CircuitBreakerState` struct — tracks open/closed/half-open circuit breaker state |
| `validation` (inline in `lib.rs`) | Free functions: `validate_course_id`, `validate_symbol`, `validate_string`, `sanitize_text` |
| `event_schema.rs` | `StandardEvent`, `EventCategory`, `EventData` — the canonical event envelope used by all contracts; defines `EVENT_SCHEMA_VERSION = 1` |
| `event_utils.rs` | Helper utilities for constructing and emitting `StandardEvent` wrappers |
| `gas_optimizer.rs` | Gas optimization utilities shared across contracts |
| `errors.rs` | `AccessControlError` — 15 typed error variants for RBAC operations |

## Key Components

### Access Control (`access_control`)

Provides the `AccessControl` struct used to initialize RBAC state. Consuming contracts call `AccessControl::initialize(env, admin)` during their own `initialize` function to bind an admin address.

```rust
AccessControl::initialize(&env, &admin)?;
```

### Reentrancy Guard (`reentrancy_guard`)

`ReentrancyLock::new(env)` creates a scoped lock. Contracts that expose fund-transferring or state-mutating functions wrap critical sections with this guard to prevent re-entrant exploit patterns.

```rust
let _lock = ReentrancyLock::new(&env);
// critical section
```

### Circuit Breaker (`error_handling`)

`CircuitBreakerState` tracks whether a contract or integration point is in the `Open`, `Closed`, or `HalfOpen` state. The security monitor and proxy contracts use this to halt operations when error thresholds are breached.

### Event Schema (`event_schema`)

Defines the `StandardEvent` envelope that all contracts use when emitting events. This ensures every event carries a consistent set of fields:

| Field | Type | Description |
|---|---|---|
| `version` | `u32` | Schema version (`EVENT_SCHEMA_VERSION = 1`) |
| `contract` | `Symbol` | Identifier of the emitting contract |
| `actor` | `Address` | Address that triggered the event |
| `timestamp` | `u64` | Ledger timestamp at emission |
| `tx_hash` | `BytesN<32>` | Derived transaction identifier |
| `sequence` | `Option<u32>` | Event sequence number for ordering |
| `event_data` | `EventData` | Contract-specific event payload |

The `EventCategory` enum groups events by domain: `AccessControl`, `Certificate`, `Analytics`, `Token`, `Progress`, `System`, `Assessment`, `Community`, `Mentorship`, `Governance`, `Security`, `Certification`, `Gamification`, `CrossChain`, `Search`, `Failure`.

### Validation (`validation`)

| Function | Description |
|---|---|
| `validate_course_id(env, course_id)` | Validates a course ID symbol is well-formed |
| `validate_symbol(env, symbol)` | Validates a generic symbol value |
| `validate_string(env, text)` | Validates and converts a raw string slice to a Soroban `String` |
| `sanitize_text(env, text)` | Sanitizes and converts a raw string slice to a Soroban `String` |

## Public API

This crate has no deployable contract entry point. It is consumed as a Rust dependency. The table below lists the symbols other contracts import from `shared`:

| Symbol | Kind | Description |
|---|---|---|
| `access_control::AccessControl` | Struct | RBAC initialization |
| `reentrancy_guard::ReentrancyLock` | Struct | Reentrancy protection guard |
| `roles::Permission` | Struct | Discrete permission token |
| `error_handling::CircuitBreakerState` | Struct | Circuit breaker state holder |
| `validation::validate_course_id` | Function | Course ID validation |
| `validation::validate_symbol` | Function | Symbol validation |
| `validation::validate_string` | Function | String validation |
| `validation::sanitize_text` | Function | Text sanitization |
| `event_schema::StandardEvent` | Struct | Canonical event envelope |
| `event_schema::EventCategory` | Enum | Event domain categories |
| `event_schema::EventData` | Enum | Contract-specific event payload |
| `event_schema::EVENT_SCHEMA_VERSION` | Constant | Current schema version (`1`) |
| `errors::AccessControlError` | Enum | RBAC error variants |

## Usage Example

```rust
// In any contract's Cargo.toml:
// [dependencies]
// shared = { path = "../../contracts/shared" }

use shared::access_control::AccessControl;
use shared::reentrancy_guard::ReentrancyLock;
use shared::event_schema::{StandardEvent, EventCategory, EVENT_SCHEMA_VERSION};
use shared::validation::validate_string;
use shared::errors::AccessControlError;

// In initialize():
AccessControl::initialize(&env, &admin)?;

// In a state-mutating function:
let _lock = ReentrancyLock::new(&env);
let title = validate_string(&env, "My Course")?;
```

## Errors

| Error | Code | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | Access control module has already been initialized |
| `NotInitialized` | 2 | Access control module has not been initialized |
| `Unauthorized` | 3 | Caller does not have the required authority |
| `RoleNotFound` | 4 | Specified role does not exist |
| `PermissionDenied` | 5 | Caller's role does not grant the required permission |
| `RoleAlreadyExists` | 6 | A role with this identifier has already been registered |
| `CannotRevokeOwnRole` | 7 | An admin cannot revoke their own role |
| `CannotTransferOwnRole` | 8 | An admin cannot transfer their own role |
| `InvalidPermission` | 9 | Permission identifier is not recognized |
| `PermissionNotGranted` | 10 | Requested permission has not been granted |
| `InvalidRoleHierarchy` | 11 | Specified role hierarchy relationship is not valid |
| `CannotGrantHigherRole` | 12 | A role cannot grant permissions to a higher-ranked role |
| `InvalidAddress` | 13 | Provided address is invalid or zero |
| `InvalidRole` | 14 | Provided role identifier is invalid or empty |
| `TemplateNotFound` | 15 | Requested role template was not found |

For cross-contract error conventions, see [ERROR_HANDLING.md](../../docs/ERROR_HANDLING.md).

## Integration

The `shared` library is a dependency of every contract in the workspace. It does not call out to other contracts — it is a compile-time dependency providing primitives. The contracts that make heaviest use of it are:

| Contract | Usage |
|---|---|
| All contracts | `event_schema` — `StandardEvent` envelope and `EventCategory` for consistent event emission |
| `security-monitor` | `circuit_breaker` / `error_handling` — `CircuitBreakerState` for open/closed state tracking |
| `proxy` | `access_control` and `reentrancy_guard` — admin-gated upgrades with re-entrancy protection |
| `gamification`, `community`, `assessment` | `validation` helpers for input sanitization |
