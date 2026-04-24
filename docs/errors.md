# Error Message Standard

This document standardizes user-facing error wording across the contract suite without changing the underlying Soroban error behavior.

## Canonical Format

Use this pattern whenever an error is surfaced in logs, docs, SDKs, CLIs, or dashboards:

```text
[ERR_CODE]: message | action
```

Example:

```text
[TKN-080]: Account balance is too low for this transfer | Reduce the amount or fund the source account before retrying
```

## Rules

- `ERR_CODE` must be stable and contract-scoped.
- `message` must describe what failed in plain language.
- `action` must tell the operator or caller what to do next.
- Formatting changes must not change contract error enum values or logic flow.

## Code Prefixes

- `SHR-*` shared utilities
- `TKN-*` token
- `CERT-*` certificate
- `DOC-*` documentation
- `SRCH-*` search
- `SEC-*` security monitor

## Implementation Notes

- Keep Soroban `#[contracterror]` enums unchanged for compatibility.
- Add standardized descriptors or helper methods alongside existing error enums.
- Prefer deterministic wording so off-chain tools can group related failures.
- Reuse the same wording in dashboards, alert annotations, SDK exceptions, and troubleshooting docs.

## Current Coverage

This repository now includes:

- shared `ErrorDescriptor` and `StandardizedError` primitives in `contracts/shared/src/error_codes.rs`
- standardized code and action helpers for core contract error enums
- a dedicated reference document for downstream tooling and SDK usage
