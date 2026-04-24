# Authentication Guide

Soroban smart contracts use the Stellar blockchain's authentication model. This guide explains how authentication works with StrellerMinds contracts.

## Soroban Authentication Model

Soroban uses an **address-based authentication** model where:

1. Every account has a **public address** (similar to an Ethereum address)
2. Actions require **authorization** from the account owner
3. Authorization is granted via **signed invocations**

## Address Types

### User Addresses

Regular users have Stellar accounts with public addresses:

```
GDRXE2BURVHYV6KBB2SSE5FXQPCS7K4LN5YSMC4T3JNB2CECZ7RLLOA
```

### Contract Addresses

Contracts also have addresses, allowing them to interact with each other:

```
CDLZFC3SYJBDYDAJSKJWZCF7HGH4GZ3C6DBJJTYWA7WZ4WMZBFHCNPOB
```

## Requiring Authorization

Contracts require authorization using `require_auth()`:

```rust
pub fn transfer(env: Env, from: Address, to: Address, amount: u64) -> Result<(), TokenError> {
    from.require_auth();  // Requires `from` to have authorized this call
    // ... transfer logic
}
```

## Authorizing Transactions

### Using Soroban CLI

```bash
# Invoke contract with authorization
soroban contract invoke \
  --source GDRXE2BURVHYV6KBB2SSE5FXQPCS7K4LN5YSMC4T3JNB2CECZ7RLLOA \
  --network testnet \
  -- contract_id CDLZFC3SYJBDYDAJSKJWZCF7HGH4GZ3C6DBJJTYWA7WZ4WMZBFHCNPOB \
  -- transfer \
  --from GDRXE2BURVHYV6KBB2SSE5FXQPCS7K4LN5YSMC4T3JNB2CECZ7RLLOA \
  --to GBMLX7PJDDHGNDBLWDMMH3CJQDS5CZOSCF4GBOV6JIH5BBG6CNF3F5XX \
  --amount 1000
```

### Using JavaScript/TypeScript SDK

```typescript
import { SorobanRpc, Keypair, Contract } from '@stellar/stellar-sdk';

const keypair = Keypair.fromSecret('SDRXE2BURVHYV6KBB2SSE5FXQPCS7K4LN5YSMC4T3JNB2CECZ7RLLOA');
const contract = new Contract('CDLZFC3SYJBDYDAJSKJWZCF7HGH4GZ3C6DBJJTYWA7WZ4WMZBFHCNPOB');

const tx = contract.call('transfer',
  keypair.publicKey(),  // from
  'GBMLX7PJDDHGNDBLWDMMH3CJQDS5CZOSCF4GBOV6JIH5BBG6CNF3F5XX',  // to
  1000n  // amount
);

// Sign and submit
```

## Authentication Patterns in StrellerMinds

### User-Authorized Actions

These functions require the user to authorize:

```rust
// Token transfer - sender must authorize
pub fn transfer(env: Env, from: Address, to: Address, amount: u64)

// Progress recording - student must authorize
pub fn record_progress(env: Env, student: Address, course_id: Symbol, progress: u32)

// Activity recording - user must authorize
pub fn record_activity(env: Env, user: Address, activity: ActivityRecord)
```

### Admin-Only Actions

These functions require admin authorization:

```rust
// Initialize - caller becomes admin
pub fn initialize(env: Env, admin: Address)

// Create achievement - admin must authorize
pub fn create_achievement(env: Env, admin: Address, achievement: Achievement)

// Batch issue certificates - admin must authorize
pub fn batch_issue_certificates(env: Env, admin: Address, params_list: Vec<MintCertificateParams>)
```

## Multi-Signature Authentication

The Certificate contract uses multi-signature (multi-sig) for certificate issuance:

```rust
// Configure multi-sig requirements for a course
client.configure_multisig(&admin, &multisig_config);

// Create issuance request (requires multi-sig approval)
let request_id = client.create_multisig_request(&requester, &params, &reason);

// Multiple approvers must approve
client.process_multisig_approval(&approver1, &request_id, true, &comments1, None);
client.process_multisig_approval(&approver2, &request_id, true, &comments2, None);

// Execute after threshold is reached
client.execute_multisig_request(&executor, &request_id);
```

## Verifying Authorization

Contracts can verify authorization programmatically:

```rust
fn require_admin(env: &Env, caller: &Address) -> Result<(), CertificateError> {
    caller.require_auth();
    let admin = storage::get_admin(env);
    if *caller != admin {
        return Err(CertificateError::Unauthorized);
    }
    Ok(())
}
```

## Best Practices

1. **Always require auth for sensitive operations** - Use `require_auth()` on all user-specific actions
2. **Separate admin and user functions** - Keep admin functions separate from user-facing ones
3. **Use multi-sig for high-value operations** - Certificate issuance should require multiple approvals
4. **Store and verify admin addresses** - Use proper storage, not constants
5. **Log authorization events** - Emit events for off-chain monitoring of authorization patterns

## Error Handling

Common authorization errors:

| Error | Cause |
|-------|-------|
| `Unauthorized` | Caller is not authorized for this action |
| `NotInitialized` | Contract has not been initialized |
| `RateLimitExceeded` | Too many operations in time window |

## Related

- [Getting Started Guide](getting-started.md)
- [Code Examples](code-examples/)