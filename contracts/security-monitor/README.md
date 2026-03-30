# Security Monitor Contract

## Purpose

The Security Monitor contract is the threat intelligence and incident management layer of the StrellerMinds platform. It provides on-chain threat detection and storage, an oracle-based asynchronous analysis pattern for computationally intensive checks (anomaly detection, biometric verification, credential fraud detection), user risk scoring with decay on training completion, threat intelligence management, security training progress tracking, and formal incident report generation. The contract is designed around a request-callback model: analysis requests are emitted as events for off-chain oracles to pick up, with results delivered back via authorized callback functions.

## Architecture

| Module | Description |
|---|---|
| `lib.rs` | Contract entry point — all public functions including oracle callback handlers |
| `threat_detector.rs` | `ThreatDetector` — generates deterministic threat IDs, handles oracle result callbacks |
| `circuit_breaker.rs` | Circuit breaker state management for protecting downstream contracts |
| `recommendation_engine.rs` | Generates security recommendations from detected threat patterns |
| `security_scanner.rs` | Scanner interface for polling threat records |
| `storage.rs` | `SecurityStorage` — typed persistent storage for threats, risk scores, training records, oracles, incident reports |
| `types.rs` | `SecurityConfig`, `SecurityThreat`, `UserRiskScore`, `ThreatIntelligence`, `SecurityTrainingStatus`, `IncidentReport` |
| `interface.rs` | Trait definitions for the security monitor's public interface |
| `events.rs` | `SecurityEvents` — emits events for initialization, threat detection, oracle requests, training, and incidents |
| `errors.rs` | `SecurityError` — typed error variants across 8 categories |

## Public API

| Function | Description | Auth Required |
|---|---|---|
| `initialize(admin, config)` | One-time setup; sets admin and security configuration | Admin |
| `scan_for_threats(contract, window_seconds)` | Returns active threats for a contract within a time window | None |
| `get_threat(threat_id)` | Returns a single threat record by its 32-byte ID | None |
| `get_contract_threats(contract)` | Lists all threat IDs associated with a contract | None |
| `request_anomaly_analysis(actor, contract)` | Emits a request event for off-chain oracle anomaly analysis; returns request ID | User |
| `callback_anomaly_analysis(oracle, request_id, is_anomalous, risk_score)` | Delivers oracle analysis result back on-chain | Oracle |
| `verify_biometrics(actor, encrypted_payload)` | Emits a biometric verification request; returns request ID | User |
| `callback_biometrics_verification(oracle, request_id, is_valid)` | Delivers biometric verification result | Oracle |
| `verify_credential_fraud(actor, credential_hash)` | Emits a credential fraud check request; returns request ID | User |
| `callback_credential_fraud(oracle, request_id, is_fraudulent)` | Delivers fraud detection result | Oracle |
| `update_threat_intelligence(admin, intel)` | Stores or updates a threat intelligence indicator | Admin |
| `update_user_risk_score(admin, user, score, risk_factor)` | Updates a user's absolute risk score with a labeled factor | Admin / Oracle |
| `get_user_risk_score(user)` | Returns the current risk score record for a user | None |
| `record_security_training(admin, user, module, score)` | Records completion of a security training module; reduces risk score by 10 | Admin |
| `generate_incident_report(admin, threat_ids, impact_summary)` | Creates a formal incident report aggregating multiple threats | Admin |

## Usage Example

```
# 1. Admin initializes the contract
security_monitor.initialize(admin, {
    risk_threshold: 75,
    auto_lockout_threshold: 90,
    ...
})

# 2. A user requests anomaly analysis on a contract
request_id = security_monitor.request_anomaly_analysis(operator, certificate_contract_symbol)
# Off-chain oracle picks up the event and computes the analysis

# 3. Authorized oracle delivers the result
security_monitor.callback_anomaly_analysis(oracle, request_id, true, 82)

# 4. Admin updates the user's risk score based on the finding
security_monitor.update_user_risk_score(admin, suspicious_user, 82, "AnomalousBehavior")

# 5. User completes security training — reduces their risk score
security_monitor.record_security_training(admin, suspicious_user, "PhishingAwareness", 95)

# 6. Admin stores new threat intelligence and files an incident report
security_monitor.update_threat_intelligence(admin, {
    indicator_type: "IPAddress",
    indicator_value: "192.168.1.100",
    threat_level: "High",
    source: "ThreatFeedA"
})
incident_id = security_monitor.generate_incident_report(admin, [threat_id_1, threat_id_2], "Credential replay attack detected")
```

## Errors

| Error | Code | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | Contract has already been initialized |
| `NotInitialized` | 2 | Contract has not been initialized |
| `Unauthorized` | 3 | Caller is not authorized |
| `PermissionDenied` | 4 | Caller lacks the required permission |
| `InvalidConfiguration` | 5 | Configuration fields contain invalid values |
| `InvalidThreshold` | 6 | Alert or detection threshold is out of range |
| `InvalidTimeWindow` | 7 | Time window value is invalid |
| `ThreatNotFound` | 10 | No threat record found for the specified ID |
| `InvalidThreatData` | 11 | Supplied threat data failed validation |
| `ThreatAlreadyExists` | 12 | A threat with this identifier already exists |
| `CircuitBreakerOpen` | 20 | Circuit breaker is open and rejecting requests |
| `CircuitBreakerNotFound` | 21 | No circuit breaker found for the contract |
| `InvalidBreakerState` | 22 | Requested circuit breaker state transition is not allowed |
| `RateLimitExceeded` | 30 | Caller has exceeded the allowed request rate |
| `InvalidRateLimitConfig` | 31 | Rate-limit configuration is invalid |
| `EventReplayFailed` | 40 | Replaying a historical event sequence failed |
| `EventFilteringFailed` | 41 | Applying event filter criteria failed |
| `InsufficientEvents` | 42 | Not enough events for the requested analysis |
| `MetricsNotFound` | 50 | No metrics found for the target |
| `InvalidMetricsData` | 51 | Metrics data failed validation |
| `MetricsCalculationFailed` | 52 | Error computing derived metrics |
| `RecommendationNotFound` | 60 | No recommendation found for the specified ID |
| `InvalidRecommendation` | 61 | Recommendation contains invalid data |
| `StorageError` | 70 | Storage read/write operation failed |
| `DataNotFound` | 71 | Requested data record was not found |
| `InvalidInput` | 80 | Input value is invalid or out of range |
| `OperationFailed` | 81 | Requested operation failed to complete |

For cross-contract error conventions, see [ERROR_HANDLING.md](../../docs/ERROR_HANDLING.md).

## Integration

| Contract | Interaction |
|---|---|
| `assessment` | Provides plagiarism and integrity metadata to submitted assessments via `update_integrity_metadata` |
| `diagnostics` | Anomaly events from diagnostics feed into security monitor threat assessment |
| `certificate` | Credential fraud detection protects the certificate issuance pipeline |
| `proxy` | Circuit breaker patterns can be applied to protect proxy upgrade operations |
| `shared` | Uses shared event schema conventions |
