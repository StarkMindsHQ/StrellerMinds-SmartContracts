# Troubleshooting Guide

This guide maps common error symptoms to their causes and recovery steps. Error codes are the `u32` values from each contract's error enum — see [ERROR_HANDLING.md](ERROR_HANDLING.md) for full code tables.

---

## Reading a Contract Error

When a Soroban contract call fails, the host returns a `HostError` containing the contract's error code. In the Rust test client, use `try_*` methods:

```rust
match client.try_initialize(&admin) {
    Ok(Ok(())) => { /* success */ }
    Ok(Err(TokenError::AlreadyInitialized)) => { /* already set up */ }
    Err(host_err) => { /* host-level failure (auth, budget, etc.) */ }
}
```

In JavaScript/TypeScript (via Stellar SDK), the error appears as `invokeHostFunctionError` with a `contractError` field carrying the `u32` code.

---

## Common Errors by Symptom

### "Contract is not responding / returns error code 2"

**Cause:** `NotInitialized` — the contract's `initialize` function has not been called.

**Fix:** Call `initialize(admin)` once before any other operation. Ensure the transaction is signed by the intended admin.

**Affected contracts:** All contracts.

---

### "Got error code 1 after calling initialize a second time"

**Cause:** `AlreadyInitialized` — `initialize` can only be called once per contract instance.

**Fix:** Do not call `initialize` again. If you need to reset the contract, deploy a new instance.

**Affected contracts:** All contracts.

---

### "Got error code 3 (or 10 depending on contract)"

**Cause:** `Unauthorized` — the transaction signer does not have the required role or is not the admin.

**Fix:**
1. Confirm you are signing with the admin/owner address.
2. For RBAC-protected operations, verify the calling address has been granted the required role via the `shared` access control module.
3. Check that `require_auth()` is satisfied in the transaction envelope.

**Affected contracts:** All contracts.

---

### "Error code 11 — AdminNotSet" (student-progress-tracker)

**Cause:** `AdminNotSet` — `get_admin` or `update_progress` was called before `initialize`.

**Fix:** Call `initialize(admin)` first.

---

### "Error code 20 — InvalidPercent" (student-progress-tracker)

**Cause:** `update_progress` was called with a `percent` value greater than 100.

**Fix:** Ensure the percentage value is in the range 0–100 (inclusive).

---

### "Error code 50 — CredentialNotFound" (cross-chain-credentials)

**Cause:** The `credential_id` passed to `get_credential`, `revoke_credential`, `suspend_credential`, `reactivate_credential`, or `verify_cross_chain` does not exist.

**Fix:**
1. Verify the `credential_id` was returned by a successful `issue_credential` call.
2. Confirm the credential was issued on this contract instance (not a different deployment).

---

### "Error code 80 — CredentialNotActive" (cross-chain-credentials)

**Cause:** `verify_cross_chain` was called on a credential that is `Revoked` or `Suspended`.

**Fix:**
- If revoked: credentials cannot be reactivated after revocation. Issue a new credential.
- If suspended: call `reactivate_credential(credential_id)` as admin, then retry verification.

---

### "Error code 80 — InsufficientBalance" (token)

**Cause:** The sender's balance is lower than the transfer amount.

**Fix:** Ensure the sender account has been minted sufficient tokens before initiating the transfer.

---

### "Error code 2 — AlreadyInitialized" (mobile-optimizer)

**Cause:** `initialize` was called on a contract that is already set up.

**Fix:** Do not call `initialize` again. Connect to the existing instance.

---

### "Error code 5 — SessionNotFound" (mobile-optimizer)

**Cause:** A session operation references a session ID that does not exist or has expired.

**Fix:**
1. Confirm the `session_id` was returned by `create_session`.
2. Check if the session has expired (exceeds `session_timeout_seconds` from config).
3. Create a new session if necessary.

---

### "Error code 1 — AlreadyInitialized" (search)

**Cause:** `initialize` was called more than once.

**Fix:** Do not call `initialize` again. Use the existing contract instance.

---

### "Error code 5 — ContentNotFound" (search)

**Cause:** The `content_id` passed to a search or retrieval function has no metadata stored.

**Fix:**
1. Ensure `store_semantic_metadata` or `store_content_analysis` was called for this content first.
2. Verify the content ID string matches exactly (case-sensitive).

---

### "Error code 4 — DocumentNotFound" (documentation)

**Cause:** The `doc_id` passed to `get_document`, `publish_document`, or `view_document` was never created.

**Fix:**
1. Call `create_document` first.
2. Confirm the document ID string is identical to what was returned by `create_document`.

---

### "Error code 13 — AlreadyExists" (documentation)

**Cause:** A `create_*` function was called with an ID that already exists.

**Fix:** Use the existing entity or choose a unique identifier.

---

### Diagnostics: Error in the 1000+ range

**Cause:** A sub-system in the `diagnostics` contract is disabled or misconfigured.

| Range | Sub-system | Common Fix |
|---|---|---|
| 1101–1199 | Monitoring | Enable monitoring in config |
| 1201–1299 | Prediction | Ensure sufficient historical data |
| 1501–1599 | Tracing | Enable tracing in config |
| 1601–1699 | Benchmarking | Enable benchmarking in config |
| 1701–1799 | Anomaly detection | Enable anomaly detection |
| 2101–2199 | Storage | Check persistent storage budget |

---

## Host-Level Errors (Not Contract Errors)

These are not from the contract's error enum — they come from the Soroban host itself:

| HostError | Cause | Fix |
|---|---|---|
| `Error(Auth, InvalidAction)` | `require_auth()` failed — missing or wrong signature | Sign transaction with the correct keypair |
| `Error(Budget, CpuLimitExceeded)` | Contract computation exceeded the CPU budget | Reduce batch size or split operations |
| `Error(Value, InvalidInput)` | Wrong argument type passed to contract | Check argument types match the function signature |

---

## Running Error Scenario Tests

```bash
# Run all unit tests including error path tests
make unit-test

# Run tests for a specific contract
cargo test -p cross-chain-credentials
cargo test -p student-progress-tracker
cargo test -p analytics
cargo test -p gamification
```

---

## Reporting a New Error

If you encounter an error code not listed here:

1. Check the contract's `src/errors.rs` for the numeric code.
2. Open an issue at the repository with the contract name, function called, and error code.
3. See [ERROR_HANDLING.md](ERROR_HANDLING.md) for the complete per-contract code tables.
