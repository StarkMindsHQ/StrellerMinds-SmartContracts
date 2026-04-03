# Proxy Contract

## Purpose

The Proxy contract is the upgrade gateway for the StrellerMinds smart contract suite. It holds a pointer to a live implementation contract and allows an authorized admin to swap that pointer to a new implementation without migrating stored state. This upgradeable pattern lets the platform ship logic fixes and feature additions while keeping all on-chain data intact and contract addresses stable for integrators.

## Architecture

| Module | Description |
|---|---|
| `src/lib.rs` | Contract entrypoint — exposes `initialize`, `upgrade`, `get_admin`, and `get_implementation`; guards all mutations behind admin authorization |
| `src/errors.rs` | `ProxyError` enum covering initialization state, authorization, and upgrade/rollback failure modes |

## Public API

| Function | Description | Auth Required |
|---|---|---|
| `initialize(admin, implementation)` | One-time setup; sets the admin address and the initial implementation contract address | No (open, call once) |
| `upgrade(new_implementation)` | Replaces the current implementation pointer with `new_implementation`; takes effect immediately | Yes — admin |
| `get_admin()` | Returns the current admin address | No |
| `get_implementation()` | Returns the address of the active implementation contract | No |

## Usage Example

```text
# Deploy proxy and point it at v1 of the certificate contract
proxy.initialize(admin_address, certificate_v1_address)

# Query the current implementation
impl_addr = proxy.get_implementation()  # returns certificate_v1_address

# Admin deploys v2 and upgrades the proxy
proxy.upgrade(certificate_v2_address)  # requires admin auth

# All future calls routed through proxy now hit v2 logic
impl_addr = proxy.get_implementation()  # returns certificate_v2_address
```

## Errors

For the full error code reference and conventions, see [ERROR_HANDLING.md](../../docs/ERROR_HANDLING.md).

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | `initialize` has already been called |
| 2 | `NotInitialized` | Contract has not been initialized; queries cannot proceed |
| 10 | `Unauthorized` | Caller is not the admin |
| 80 | `UpgradeFailed` | The upgrade operation could not be applied |
| 81 | `RollbackFailed` | Rollback to the previous implementation failed |

## Integration

| Contract | Relationship |
|---|---|
| `certificate` | Primary candidate for proxied upgrades — certificate logic evolves as compliance requirements change |
| `shared` | All contracts in the suite can be placed behind a proxy instance; RBAC from `shared` governs admin identity |
