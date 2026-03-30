# Security Testing Guide

> Resolves issue **#273 – Missing Security Testing**

## Overview

This document describes the security testing strategy for StrellerMinds Smart Contracts. Security tests are organised into three layers: unit tests, integration tests, and automated scanning.

---

## Test Architecture

```
contracts/security-monitor/src/
├── tests.rs            ← Comprehensive unit test suite (30+ tests)
├── security_scanner.rs ← Runtime scanner + embedded unit tests
└── circuit_breaker.rs  ← Circuit-breaker logic (covered by tests.rs)

scripts/
└── security_scan.sh    ← Automated multi-layer scan script
```

---

## Running Security Tests

### Quick run (unit tests only)

```bash
cargo test --package security-monitor --lib
```

### Full security scan

```bash
make security-scan
```

This runs:
1. `cargo audit` – advisory database CVE scan  
2. `cargo clippy` – security-oriented lint rules  
3. Security unit tests  

### Full scan with semgrep (requires `pip install semgrep`)

```bash
make security-scan-full
```

### CI pipeline

```bash
make ci-security
```

---

## Test Categories

### 1. Initialisation Tests

| Test | Validates |
|------|-----------|
| `test_initialize_sets_admin` | Admin stored securely at init |
| `test_initialize_stores_config` | Config stored with valid defaults |
| `test_scan_for_threats_returns_empty_without_metrics` | No false positives on empty state |

### 2. Penetration / Authorization Tests

These tests simulate adversarial inputs – a non-admin or unregistered oracle attempting privileged operations:

| Test | Attack Scenario |
|------|----------------|
| `test_update_threat_intel_rejects_non_admin` | Threat intel injection by attacker |
| `test_update_user_risk_rejects_non_admin_non_oracle` | Risk score escalation |
| `test_record_training_rejects_non_admin` | Fake training completion |
| `test_generate_incident_report_rejects_non_admin` | Unauthorized report creation |
| `test_oracle_callback_rejects_unauthorized_caller` | Unregistered oracle callback |
| `test_biometrics_callback_rejects_unauthorized_caller` | Biometric spoofing |
| `test_credential_fraud_callback_rejects_unauthorized_caller` | Credential fraud bypass |

### 3. Threat Management Tests

Validate that the `SecurityStorage` correctly persists, indexes, and retrieves threat records.

### 4. User Risk Score Tests

Validate that:
- Risk scores require admin authorization to update
- Security training correctly reduces scores
- Score cannot underflow below zero

### 5. Circuit Breaker Tests

| Test | Validates |
|------|-----------|
| `test_circuit_breaker_starts_closed` | New breakers default to Closed |
| `test_circuit_breaker_opens_after_threshold_failures` | Opens after N failures |
| `test_open_circuit_blocks_new_calls` | Blocked in Open state |
| `test_circuit_breaker_transitions_to_half_open_after_timeout` | Recovery flow |

### 6. SecurityScanner Tests

`security_scanner.rs` embeds its own `#[cfg(test)]` module covering:
- Clean contract scores 100
- Stored threats are correctly counted per severity
- Health status transitions: Healthy → Degraded → Critical
- Mitigation detection  
- Score computation arithmetic

---

## Security Scanner

```rust
use security_monitor::security_scanner::{SecurityScanner, HealthStatus};

// Full scan
let report = SecurityScanner::scan(&env, &contract_sym)?;
println!("Score: {}", report.aggregate_score);
println!("Action: {:?}", report.recommendation);

// Quick health check
let status = SecurityScanner::health_check(&env, &contract_sym)?;
assert_eq!(status, HealthStatus::Healthy);
```

### Score Computation

| Severity | Penalty per threat |
|----------|--------------------|
| Critical | −25 points |
| High     | −10 points |
| Medium   | −5 points  |
| Low      | −1 point   |

Score starts at 100; minimum is 0.

### Recommended Actions

| Score range / condition | Action |
|------------------------|--------|
| Any critical threat present | `HaltOperations` |
| High > 2 or score < 30 | `TriggerMitigation` |
| High > 0 or score < 60 | `AlertOperators` |
| Score 60–79 | `IncreasedMonitoring` |
| Score ≥ 80, no high/critical | `NoAction` |

---

## Automated Scan Script

`scripts/security_scan.sh` produces a JSON report at `target/security_scan.json`:

```json
{
  "timestamp": "2026-03-27T10:00:00Z",
  "summary": { "passed": 3, "warnings": 1, "failed": 0 },
  "checks": [
    { "check": "cargo-audit",    "status": "PASS", "detail": "No known vulnerabilities found" },
    { "check": "clippy",         "status": "PASS", "detail": "No lint issues found" },
    { "check": "security-tests", "status": "PASS", "detail": "30 security tests passed" },
    { "check": "semgrep",        "status": "WARN", "detail": "skipped (pass --full to enable)" }
  ]
}
```

---

## CI Integration

Add to `.github/workflows/ci.yml`:

```yaml
- name: Security Scan
  run: make ci-security

- name: Upload Security Report
  uses: actions/upload-artifact@v4
  with:
    name: security-scan-report
    path: target/security_scan.json
```

---

## Security Monitoring

The `SecurityMonitor` contract records threats, incident reports, risk scores, and training status on-chain. Operators should:

1. Poll `get_contract_threats` regularly for new threat IDs  
2. Call `SecurityScanner::health_check` before critical operations  
3. Integrate the scan script into CI on every merge to `main`  
4. Run `make security-scan-full` weekly with semgrep enabled  

---

## Effectiveness Review

| Metric | Target | Measurement |
|--------|--------|-------------|
| Security test count | ≥ 30 | `cargo test --package security-monitor -- --list \| wc -l` |
| All auth checks exercised | 100% | Covered by penetration tests |
| Scanner false-positive rate | 0% | Verified by clean-contract tests |
| CI scan gate | must pass | `make ci-security` exit code |
