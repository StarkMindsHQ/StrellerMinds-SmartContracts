# Certificate Contract

## Purpose

The Certificate contract manages the full lifecycle of on-chain educational credentials for the StrellerMinds platform. It provides hierarchical multi-signature approval workflows for certificate issuance, batch minting, tamper-evident revocation with reissuance paths, compliance record keeping against multiple standards, a template system for standardized credential types, and verifiable sharing records. Every certificate is anchored to a deterministic blockchain hash, making it independently verifiable by any third party.

## Architecture

| Module | Description |
|---|---|
| `lib.rs` | Contract entry point — 25 public functions covering the full certificate lifecycle |
| `storage.rs` | All persistent state access: certificates, multi-sig requests, revocations, templates, analytics, audit log |
| `types.rs` | `contracttype`-derived structs: `Certificate`, `MultiSigCertificateRequest`, `CertificateTemplate`, `ComplianceRecord`, `RevocationRecord`, `ShareRecord`, `BatchResult`, `CertificateAnalytics` |
| `events.rs` | Typed event emitters for issuance, approval, revocation, verification, and template events |
| `errors.rs` | `CertificateError` — 30+ typed error variants across 7 categories |

## Public API

| Function | Description | Auth Required |
|---|---|---|
| `initialize(admin)` | One-time setup; sets the admin address | Admin |
| `configure_multisig(admin, config)` | Sets multi-sig approval rules for a course (approvers, threshold, timeout) | Admin |
| `get_multisig_config(course_id)` | Returns the multi-sig config for a course | None |
| `create_multisig_request(requester, params, reason)` | Queues a certificate issuance request for multi-sig approval | User |
| `process_multisig_approval(approver, request_id, approved, comments, sig_hash)` | Records an approver's decision; auto-executes if threshold is met | Approver |
| `get_multisig_request(request_id)` | Returns a multi-sig request by ID | None |
| `get_pending_requests()` | Lists all pending multi-sig requests | None |
| `execute_multisig_request(executor, request_id)` | Manually executes an approved request | Admin |
| `batch_issue_certificates(admin, params_list)` | Issues up to 25 certificates in a single transaction | Admin |
| `verify_certificate(certificate_id)` | Verifies a certificate is active, unexpired, and anchored | None |
| `revoke_certificate(admin, certificate_id, reason, reissuance_eligible)` | Revokes an active certificate and records the reason | Admin |
| `reissue_certificate(admin, old_certificate_id, new_params)` | Issues a replacement for a revoked, eligible certificate | Admin |
| `create_template(admin, template_id, name, description, fields)` | Defines a reusable certificate template | Admin |
| `get_template(template_id)` | Returns a certificate template by ID | None |
| `issue_with_template(admin, template_id, params, field_values)` | Issues a certificate validated against a template's required fields | Admin |
| `get_certificate(certificate_id)` | Returns a certificate by ID | None |
| `get_student_certificates(student)` | Lists all certificate IDs belonging to a student | None |
| `record_compliance(admin, certificate_id, standard, record)` | Attaches a compliance record to a certificate | Admin |
| `get_compliance_record(certificate_id, standard)` | Returns the compliance record for a certificate against a given standard | None |
| `share_certificate(student, certificate_id, recipient, platform)` | Records that a student shared their certificate with a recipient | User |
| `get_share_records(certificate_id)` | Lists all share records for a certificate | None |
| `get_analytics()` | Returns aggregate analytics: total issued, revoked, verified, pending requests | None |
| `get_audit_log(request_id)` | Returns the audit trail for a multi-sig request | None |
| `get_revocation_record(certificate_id)` | Returns the revocation details for a revoked certificate | None |
| `get_admin()` | Returns the stored admin address | None |

## Usage Example

```
# 1. Admin initializes and configures multi-sig for a course
certificate.initialize(admin)
certificate.configure_multisig(admin, {
    course_id: "RUST101",
    authorized_approvers: [approver1, approver2, approver3],
    required_approvals: 2,
    timeout_duration: 86400,  # 24 hours
    priority: "Standard"
})

# 2. Instructor requests certificate issuance
request_id = certificate.create_multisig_request(instructor, {
    certificate_id: cert_hash,
    course_id: "RUST101",
    student: student_address,
    title: "Rust Fundamentals",
    ...
}, "Course completion verified")

# 3. Two approvers approve — certificate is auto-issued
certificate.process_multisig_approval(approver1, request_id, true, "Verified", None)
certificate.process_multisig_approval(approver2, request_id, true, "Confirmed", None)

# 4. Anyone can verify the certificate
is_valid = certificate.verify_certificate(cert_hash)

# 5. Student shares it, admin can revoke if needed
certificate.share_certificate(student, cert_hash, "LinkedIn", "linkedin")
certificate.revoke_certificate(admin, cert_hash, "Policy violation", false)
```

## Errors

| Error | Code | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | Contract has already been initialized |
| `NotInitialized` | 2 | Contract has not been initialized |
| `Unauthorized` | 3 | Caller is not the admin |
| `MultiSigRequestNotFound` | 10 | Specified multi-sig request does not exist |
| `MultiSigRequestExpired` | 11 | Multi-sig request has passed its deadline |
| `ApproverNotAuthorized` | 12 | Caller is not in the authorized approvers list |
| `InsufficientApprovals` | 13 | Request does not have enough approvals to execute |
| `InvalidApprovalThreshold` | 14 | Threshold is zero or exceeds approver count |
| `AlreadyApproved` | 15 | This approver has already submitted a decision |
| `RequestNotPending` | 16 | Request is not in the pending state |
| `RequestAlreadyExecuted` | 17 | Request has already been executed |
| `CertificateNotFound` | 20 | No certificate found with the given ID |
| `CertificateAlreadyExists` | 21 | A certificate with this ID was already issued |
| `CertificateRevoked` | 22 | Certificate has been revoked |
| `CertificateExpired` | 23 | Certificate has passed its expiry date |
| `CertificateNotEligibleForReissue` | 24 | Certificate is not revoked or not marked eligible |
| `TemplateNotFound` | 30 | No template found with the given ID |
| `TemplateAlreadyExists` | 31 | A template with this ID already exists |
| `TemplateInactive` | 32 | Template is inactive and cannot be used |
| `MissingRequiredField` | 33 | Not all required template fields were provided |
| `InvalidConfig` | 40 | Configuration contains invalid values |
| `ConfigNotFound` | 41 | No multi-sig config found for the course |
| `TooManyApprovers` | 42 | Approver list exceeds the maximum of 10 |
| `TimeoutTooShort` | 43 | Timeout is below the 1-hour minimum |
| `TimeoutTooLong` | 44 | Timeout exceeds the 30-day maximum |
| `BatchTooLarge` | 50 | Batch exceeds the 25-certificate maximum |
| `BatchEmpty` | 51 | Batch list is empty |
| `ComplianceCheckFailed` | 60 | Compliance check could not be completed |
| `UnsupportedStandard` | 61 | Compliance standard is not supported |
| `ShareLimitReached` | 70 | Certificate has reached maximum share records (100) |
| `InvalidInput` | 80 | One or more input values are invalid |
| `InternalError` | 99 | Unexpected internal error |

For cross-contract error conventions, see [ERROR_HANDLING.md](../../docs/ERROR_HANDLING.md).

## Integration

| Contract | Interaction |
|---|---|
| `cross-chain-credentials` | Exports issued certificates as cross-chain verifiable credentials |
| `assessment` | Assessment pass results trigger certificate issuance requests |
| `analytics` | Certificate issuance, verification, and revocation events feed analytics |
| `token` | Future integration for certificate-gated token rewards |
| `shared` | Uses shared RBAC and event schema conventions |
