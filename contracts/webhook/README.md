# Webhook Contract

## Purpose

The Webhook contract enables third-party integrations by dispatching signed event notifications to registered HTTP endpoints. It supports three event types — **CertificateIssued**, **StudentProgress**, and **AchievementUnlocked** — with built-in retry logic and HMAC-SHA256 payload signing so consumers can verify authenticity.

## Architecture

| Module | Description |
|---|---|
| `src/lib.rs` | Contract entrypoint — exposes registration, dispatch, retry, and signing functions |
| `src/types.rs` | Data types: `WebhookEndpoint`, `PendingDelivery`, event payloads, `DataKey`, and constants |
| `src/errors.rs` | `WebhookError` enum covering all failure modes |
| `src/tests.rs` | Unit test suite (30+ tests) |

## Public API

| Function | Description | Auth Required |
|---|---|---|
| `initialize(admin)` | One-time setup; records the admin address | No (call once) |
| `register(owner, url, secret, event_types)` | Register a new webhook endpoint; returns the webhook ID | Yes — `owner` |
| `unregister(owner, webhook_id)` | Deactivate a webhook | Yes — `owner` |
| `dispatch_certificate_issued(caller, payload)` | Fire a `CertificateIssued` event to all matching active webhooks | Yes — `caller` |
| `dispatch_student_progress(caller, payload)` | Fire a `StudentProgress` event to all matching active webhooks | Yes — `caller` |
| `dispatch_achievement_unlocked(caller, payload)` | Fire an `AchievementUnlocked` event to all matching active webhooks | Yes — `caller` |
| `retry_delivery(webhook_id, delivery_seq)` | Retry a pending delivery after the backoff window has elapsed | No (keeper-callable) |
| `compute_signature(webhook_id, owner, message)` | Compute the HMAC-SHA256 signature for a message using the webhook's secret | Yes — `owner` |
| `get_webhook(webhook_id)` | Fetch a registered webhook by ID | No |
| `get_owner_webhooks(owner)` | List all webhook IDs owned by an address | No |
| `get_pending_delivery(webhook_id, delivery_seq)` | Fetch a pending delivery record | No |

## Event Types

| Variant | Payload Fields | Description |
|---|---|---|
| `CertificateIssued` | `certificate_id`, `student`, `course_id`, `issued_at` | Fired when a certificate is issued to a student |
| `StudentProgress` | `student`, `course_id`, `progress_pct`, `updated_at` | Fired when a student's course progress is updated |
| `AchievementUnlocked` | `student`, `achievement_id`, `unlocked_at` | Fired when a student unlocks an achievement |

## Retry Mechanism

When a webhook is dispatched, a `PendingDelivery` record is stored on-chain with:

- `attempts` — number of delivery attempts so far (starts at 1)
- `next_attempt_ledger` — earliest ledger at which a retry is allowed
- `payload_hash` — SHA-256 hash of the event payload

Retries use **exponential backoff**: each retry doubles the wait window (capped at 16×):

```
next_attempt_ledger = current_ledger + RETRY_BACKOFF_LEDGERS × 2^min(attempts, 4)
```

Constants (defined in `types.rs`):

| Constant | Value | Meaning |
|---|---|---|
| `MAX_RETRY_ATTEMPTS` | 3 | Maximum delivery attempts before the record is dropped |
| `RETRY_BACKOFF_LEDGERS` | 12 | Base backoff (~1 minute at 5s/ledger) |
| `MAX_WEBHOOKS_PER_OWNER` | 10 | Maximum webhooks per owner address |

After `MAX_RETRY_ATTEMPTS` the delivery record is removed and a `wh_fail` event is emitted.

## Webhook Signing

Every delivery is signed with **HMAC-SHA256** using the 32-byte secret provided at registration. The signature is computed over the SHA-256 hash of the event payload.

### Verification (receiver side)

```python
import hmac, hashlib

def verify_webhook(secret: bytes, payload_hash: bytes, signature: bytes) -> bool:
    expected = hmac.new(secret, payload_hash, hashlib.sha256).digest()
    return hmac.compare_digest(expected, signature)
```

```typescript
import { createHmac, timingSafeEqual } from "crypto";

function verifyWebhook(secret: Buffer, payloadHash: Buffer, signature: Buffer): boolean {
  const expected = createHmac("sha256", secret).update(payloadHash).digest();
  return timingSafeEqual(expected, signature);
}
```

You can also call `compute_signature(webhook_id, owner, message)` on-chain to obtain the expected signature for any message.

## On-Chain Events

| Symbol | Trigger | Data |
|---|---|---|
| `wh_init` | `initialize` | admin address |
| `wh_reg` | `register` | `(owner, webhook_id)` |
| `wh_unreg` | `unregister` | `(owner, webhook_id)` |
| `wh_cert` | `dispatch_certificate_issued` | `(student, certificate_id)` |
| `wh_prog` | `dispatch_student_progress` | `(student, progress_pct)` |
| `wh_ach` | `dispatch_achievement_unlocked` | `(student, achievement_id)` |
| `wh_retry` | `retry_delivery` (success) | `(delivery_seq, attempts)` |
| `wh_fail` | `retry_delivery` (exhausted) | `delivery_seq` |

## Usage Example

```rust
// 1. Initialize
client.initialize(&admin);

// 2. Register a webhook
let secret = BytesN::from_array(&env, &[0x42u8; 32]);
let url = Bytes::from_slice(&env, b"https://my-app.example.com/hooks");
let mut events = Vec::new(&env);
events.push_back(WebhookEventType::CertificateIssued);
events.push_back(WebhookEventType::StudentProgress);
let webhook_id = client.register(&owner, &url, &secret, &events).unwrap();

// 3. Dispatch an event (called by the certificate contract)
let payload = CertificateIssuedPayload {
    certificate_id: BytesN::from_array(&env, &cert_id),
    student: student_address,
    course_id: String::from_str(&env, "RUST101"),
    issued_at: env.ledger().timestamp(),
};
let delivery_seqs = client.dispatch_certificate_issued(&caller, &payload).unwrap();

// 4. Retry a failed delivery (called by a keeper after backoff)
client.retry_delivery(&webhook_id, &delivery_seqs.get(0).unwrap()).unwrap();

// 5. Verify a signature on the receiver side
let sig = client.compute_signature(&webhook_id, &owner, &message).unwrap();
```

## Errors

| Code | Variant | Meaning |
|---|---|---|
| 1 | `NotInitialized` | Contract has not been initialized |
| 2 | `AlreadyInitialized` | `initialize` has already been called |
| 3 | `Unauthorized` | Caller is not the webhook owner |
| 4 | `WebhookNotFound` | No webhook with the given ID exists |
| 5 | `WebhookInactive` | Webhook has been deactivated |
| 6 | `TooManyWebhooks` | Owner has reached `MAX_WEBHOOKS_PER_OWNER` |
| 7 | `InvalidUrl` | URL bytes are empty |
| 8 | `InvalidEventType` | Reserved for future use |
| 9 | `DeliveryNotFound` | No pending delivery with the given sequence number |
| 10 | `RetryLimitExceeded` | Delivery has exhausted all retry attempts |
| 11 | `RetryTooEarly` | Retry called before the backoff window has elapsed |
| 12 | `NoEventTypesSpecified` | `event_types` list is empty |

## Integration

| Contract | Relationship |
|---|---|
| `certificate` | Calls `dispatch_certificate_issued` when a certificate is issued |
| `student-progress-tracker` | Calls `dispatch_student_progress` on progress updates |
| `gamification` | Calls `dispatch_achievement_unlocked` when achievements are unlocked |
| `shared` | Uses `soroban-sdk` primitives; no direct shared-contract dependency |

## Testing

```bash
# Run all webhook unit tests
cargo test -p webhook

# Run with output
cargo test -p webhook -- --nocapture
```

The test suite covers:
- Initialization and double-initialization guard
- Registration: success, empty URL, empty event types, per-owner limit
- Unregistration: success, wrong owner, nonexistent webhook
- Event dispatch for all three event types
- Filtering: inactive webhooks skipped, wrong event type skipped
- Multiple subscribers receiving the same event
- Retry: too-early rejection, backoff success, attempt counter, exhaustion cleanup
- HMAC signing: determinism, message sensitivity, wrong-owner rejection
- Query functions: missing webhook, missing delivery, empty owner list
