# Certificate Contract

## Overview

The `certificate` contract manages educational certificate issuance, approval workflows, revocation, reissue handling, template management, and shareable verification flows for the StrellerMinds platform.

## Quick Start

```bash
cargo test -p certificate
```

Initialize the contract with an admin and multisig configuration, then create templates and issue or approve certificates through the public contract interface in [lib.rs](./src/lib.rs).

## Usage Examples

```rust
// Initialize certificate settings
CertificateContract::initialize(env, admin, approvers, threshold, timeout_secs)?;

// Create a reusable template
CertificateContract::create_template(env, admin, template_id, name, description, fields)?;

// Submit a certificate request and execute approvals
CertificateContract::approve_request(env, approver, request_id)?;
CertificateContract::execute_request(env, approver, request_id)?;
```

## Contribution Guide

- Keep changes focused on certificate lifecycle, template management, approvals, or verification.
- Preserve current error codes and event semantics unless the change explicitly targets them.
- Add or update tests in `src/test.rs` for new approval, template, or sharing behavior.

## Troubleshooting

- `NotInitialized`: initialize the contract before creating templates or certificates.
- `InvalidApprovalThreshold`: ensure the threshold is greater than zero and no larger than the approver set.
- `MultiSigRequestExpired`: recreate the request or increase timeout settings if the workflow legitimately needs more time.

## Related Files

- `src/lib.rs`: contract entrypoints
- `src/types.rs`: certificate and multisig types
- `src/events.rs`: emitted lifecycle events
- `src/test.rs`: unit and workflow tests
