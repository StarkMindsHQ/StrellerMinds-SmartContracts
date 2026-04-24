# Security Review Process

## Overview

Every change to a smart contract must pass through this review process before merging.
The process has four gates: automated checks, peer review, security testing, and sign-off.

---

## Gate 1 — Automated Checks (CI)

Run on every PR automatically via `scripts/security_audit.sh`:

| Check | Tool | Failure Action |
|---|---|---|
| Known CVEs in dependencies | `cargo audit` | Block merge |
| Static analysis / lint | `cargo clippy -D warnings` | Block merge |
| Unwrap/expect/panic usage | `clippy::unwrap_used` etc. | Block merge |
| Integer arithmetic | `clippy::integer_arithmetic` | Block merge |
| All unit + security tests | `cargo test --workspace` | Block merge |
| Dependency policy | `cargo deny check` | Block merge |

Run locally before pushing:
```bash
bash scripts/security_audit.sh
```

---

## Gate 2 — Peer Security Review

Every PR touching a contract must be reviewed by at least one team member who checks:

### Authorization
- [ ] Every state-changing function calls `require_auth()` on the appropriate caller
- [ ] Admin-only functions verify the caller matches the stored admin address
- [ ] No function allows privilege escalation (role hierarchy enforced)
- [ ] `initialize()` is idempotent — double-call returns `AlreadyInitialized`

### Input Validation
- [ ] All string inputs validated with `CoreValidator` (length, forbidden chars, quality)
- [ ] All URI inputs validated with `CoreValidator::validate_uri()`
- [ ] Batch sizes checked against `ValidationConfig::MAX_BATCH_SIZE` (100) or contract-specific limit
- [ ] Expiry dates validated with `CoreValidator::validate_expiry_date()`
- [ ] Zero/null values handled explicitly (not silently accepted)

### Arithmetic Safety
- [ ] All arithmetic uses `saturating_add`, `saturating_sub`, `saturating_mul`
- [ ] No raw `+`, `-`, `*` on user-supplied numeric values
- [ ] Division by zero guarded with `.max(1)` or explicit check

### Reentrancy
- [ ] All user-callable state-changing functions use `ReentrancyLock::new(&env)`
- [ ] Admin-only functions that don't call external contracts may omit the guard (document why)

### Storage
- [ ] Storage keys are unique and documented
- [ ] No unbounded storage growth (collections have size limits)
- [ ] Sensitive data is not stored in `instance` storage (use `persistent`)

### Events
- [ ] All state changes emit an event
- [ ] Events do not expose sensitive data (private keys, answers, etc.)

---

## Gate 3 — Security Test Coverage

New contract functions require security tests covering:

1. Unauthorized caller is rejected
2. Double-initialization is rejected
3. Invalid inputs are rejected (empty, oversized, forbidden chars)
4. Arithmetic edge cases (max values, zero values)
5. Reentrancy guard is in place for state-changing functions

Add tests to the contract's `test.rs` or `tests.rs` file following the pattern:

```rust
#[test]
#[should_panic]
fn security_unauthorized_caller_rejected() {
    let env = Env::default();
    env.mock_all_auths();
    let attacker = Address::generate(&env);
    // Call without being admin — must panic or return Err
    ContractClient::new(&env, &contract_id).admin_fn(&attacker, ...);
}
```

---

## Gate 4 — Sign-off

Before merging any contract change:

- [ ] All CI checks green
- [ ] At least one peer review approval
- [ ] Security test coverage added for new functions
- [ ] `CHANGELOG.md` updated with security-relevant changes
- [ ] If a vulnerability was fixed: severity, impact, and fix documented in PR description

---

## Vulnerability Reporting

Do NOT open a public GitHub issue for security vulnerabilities.

Email: security@strellerminds.com

Include:
- Contract name and function affected
- Steps to reproduce
- Estimated severity (Critical / High / Medium / Low)
- Suggested fix (optional)

Response SLA: 48 hours acknowledgement, 7 days for patch on Critical/High.

---

## Regular Security Assessments

| Frequency | Activity |
|---|---|
| Every PR | Automated checks (Gate 1) |
| Every PR | Peer review (Gate 2) |
| Monthly | Run `cargo audit` on main branch, review new CVEs |
| Quarterly | Full manual review of access control and reentrancy coverage |
| Before major release | External security audit engagement |

---

## Security Test Locations

| Module | Test File | Coverage |
|---|---|---|
| Input validation | `contracts/shared/src/validation_core.rs` | XSS, injection, overflow, expiry, URI |
| Access control / RBAC | `contracts/shared/src/test.rs` | Role hierarchy, privilege escalation, expiry |
| Reentrancy | `contracts/token/src/reentrancy_tests.rs` | Guard enter/exit, RAII release |
| Certificate auth | `contracts/certificate/src/test.rs` | Unauthorized mint/revoke/transfer |
| Assessment auth | `contracts/assessment/src/test.rs` | Unauthorized publish/grade |

Run all security tests:
```bash
cargo test --workspace --lib 2>&1 | grep -E "security_|reentrancy|unauthorized"
```
