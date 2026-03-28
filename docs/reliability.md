### Reliability: Error Recovery, Retries, Fallbacks, and Monitoring

This document describes the recovery mechanisms added across SDKs, e2e tooling, and deployment scripts.

### Retries with Exponential Backoff
- **TypeScript SDK**: `sdks/typescript/src/retry.ts`
- **Python SDK**: `sdks/python/src/strellerminds_sdk/retry.py`
- **Go SDK**: `sdks/go/retry.go`
- **E2E Utilities**: `e2e/utils/retry.ts`

Common features:
- Configurable `retries`, `initialDelayMs`, `maxDelayMs`, `multiplier`, and `jitter`
- `isRetryable` predicate to target transient failures only
- `onRetry` hook for logging/metrics

Transient errors are heuristically detected (timeouts, connection resets, 5xx, 429, rate-limit, not_found polling races).

### Integration Points
- E2E Soroban client (`e2e/utils/soroban-client.ts`) now wraps network calls:
  - `getAccount`, `sendTransaction`, `getTransaction` (polling), `getLedgerEntries`, and friendbot funding now use retry with backoff.

### Fallback and Rollback Procedures
- Deployment script: `scripts/deploy.sh`
  - Supports `--retries` and `--initial-delay` to retry deploy commands
  - Supports `--rollback-wasm <path>` to attempt an automatic rollback if deployment fails
  - Uses `retry_cmd` from `scripts/common.sh` to provide exponential backoff for shell commands

Usage example:
```bash
./scripts/deploy.sh --network local --contract analytics --wasm target/wasm32-unknown-unknown/release/analytics.wasm \
  --retries 3 --initial-delay 2 \
  --rollback-wasm releases/analytics_prev.wasm
```

### Monitoring Recovery Effectiveness
- E2E retry metrics: `e2e/utils/retry.ts` exports `retryMetrics` with:
  - `attempts`, `successes`, `failures`, `retries`
- You can print metrics at the end of test runs:
```ts
import { retryMetrics } from '../utils/retry.js';
afterAll(() => {
  console.log('Retry metrics:', retryMetrics);
});
```
- Hooks (`onRetry`) are used to log retry attempts with backoff delays.

### Testing Recovery Scenarios
- Unit tests should simulate transient failures (e.g., throw on first N attempts, then succeed) and assert:
  - The task eventually succeeds
  - The number of attempts matches expectations
  - Non-transient errors fail fast without retry loops

See `e2e/tests/retry.spec.ts` (added) for examples.

### Notes and Caveats
- Smart contracts must remain deterministic; retries and fallbacks are implemented off-chain (SDKs, tooling, scripts).
- For critical on-chain operations, ensure idempotency (e.g., by using unique operation IDs) to make retries safe.

