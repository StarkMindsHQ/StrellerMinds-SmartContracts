# Token Contract

## Purpose

The Token contract is the on-chain incentive layer for the StrellerMinds platform. It manages the lifecycle of the platform's native learning token: minting rewards to students and instructors, transferring tokens between accounts, and querying balances. All significant state changes emit structured events consumed by the Analytics contract for reward auditing and reporting.

## Architecture

| Module | Description |
|---|---|
| `src/lib.rs` | Contract entrypoint — exposes the public API (`initialize`, `mint`, `transfer`, `balance`) and emits `TokensMinted` / `TokensTransferred` events via shared macros |
| `src/errors.rs` | `TokenError` enum with codes for initialization state, authorization, validation, and balance faults |
| `src/gas_optimized.rs` | Gas-optimized variants of core operations for high-throughput batch scenarios |

## Public API

| Function | Description | Auth Required |
|---|---|---|
| `initialize(admin)` | One-time setup; records the admin address | No (open, call once) |
| `mint(to, amount)` | Creates `amount` new tokens and credits `to` | Yes — admin |
| `transfer(from, to, amount)` | Moves `amount` tokens from `from` to `to` | Yes — `from` |
| `balance(account)` | Returns the current token balance of `account` | No |

## Usage Example

```text
# Deploy and initialize
token.initialize(admin_address)

# Admin mints 1000 tokens to a student who completed a course
token.mint(student_address, 1000)

# Student transfers 200 tokens to another user
token.transfer(student_address, recipient_address, 200)  # requires student auth

# Query balance
bal = token.balance(student_address)  # returns 800
```

## Errors

For the full error code reference and conventions, see [ERROR_HANDLING.md](../../docs/ERROR_HANDLING.md).

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | `initialize` has already been called |
| 2 | `NotInitialized` | Contract has not been initialized yet |
| 10 | `Unauthorized` | Caller is not the admin |
| 20 | `InvalidAmount` | Supplied amount is zero or negative |
| 21 | `InvalidAddress` | Target address is malformed |
| 80 | `InsufficientBalance` | Sender does not hold enough tokens |
| 81 | `TransferFailed` | Transfer operation could not be completed |

## Integration

| Contract | Relationship |
|---|---|
| `gamification` | Queries and mints tokens as XP-to-token conversion rewards |
| `analytics` | Consumes `TokensMinted` / `TokensTransferred` events for reward reporting |
| `shared` | Uses RBAC helpers, event schema macros (`emit_token_event!`, `emit_access_control_event!`) |
