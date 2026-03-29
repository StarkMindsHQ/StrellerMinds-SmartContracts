# Security Monitor Contract

## Overview

Security monitoring, threat tracking, circuit breaking, and rate limiting for
StrellerMinds contracts.

## Key Features

- Per-contract threat storage and scanning
- Circuit-breaker state tracking
- Oracle-assisted anomaly, biometrics, and fraud workflows
- Configurable per-actor rate limiting

## Rate Limiting

Rate limiting is configured through `SecurityConfig`:

```rust
pub struct SecurityConfig {
    pub rate_limit_per_window: u32,
    pub rate_limit_window: u64,
    // ...
}
```

The contract tracks a temporary per-actor, per-contract bucket and:

- increments the bucket on each guarded request
- resets the bucket when the configured window expires
- emits a `security/rate_limit` event when the limit is exceeded
- stores a `ThreatType::RateLimitExceeded` threat for monitoring

The guarded entrypoints currently include:

- `request_anomaly_analysis`
- `verify_biometrics`
- `verify_credential_fraud`

## Testing

```bash
cargo test -p security-monitor
```

Rate-limit coverage includes:

- within-window allowance
- threshold exceedance
- window reset behavior
- guarded entrypoint enforcement
