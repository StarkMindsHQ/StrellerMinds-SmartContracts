# Error Handling Guide

This document describes the error handling conventions used across all StrellerMinds smart contracts.

---

## Standard Pattern

Every contract exposes a dedicated `<ContractName>Error` enum in its own `errors.rs` module:

```rust
use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TokenError {
    // Initialization (1-9)
    AlreadyInitialized = 1,
    NotInitialized = 2,
    // Authorization (10-19)
    Unauthorized = 10,
    // Validation (20-49)
    InvalidAmount = 20,
    // Not Found (50-79)
    // Business Logic (80-199)
    InsufficientBalance = 80,
    // Internal (200+)
    InternalError = 200,
}
```

All public contract functions return `Result<T, <ContractName>Error>`:

```rust
pub fn mint(env: Env, to: Address, amount: u64) -> Result<(), TokenError> { ... }
```

---

## Error Code Range Conventions

| Range | Category | Examples |
|---|---|---|
| 1–9 | Initialization | `AlreadyInitialized`, `NotInitialized` |
| 10–19 | Authorization | `Unauthorized`, `AdminNotSet` |
| 20–49 | Validation | `InvalidAmount`, `InvalidPercent` |
| 50–79 | Not Found | `CredentialNotFound`, `SessionNotFound` |
| 80–199 | Business Logic | `InsufficientBalance`, `CredentialNotActive` |
| 200+ | Internal | `InternalError` |

> **Note:** The `diagnostics` contract uses 1000+ ranges for sub-system grouping (e.g., Monitoring: 1100–1199, Prediction: 1200–1299). This is the exception; all other contracts use the 1–200 convention above.

---

## Required Derive Traits

All `#[contracterror]` enums **must** derive:

```rust
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
```

- `Copy` — required because Soroban passes errors across the WASM host boundary as raw `u32` values; the type must be trivially copyable.
- `PartialOrd + Ord` — enables range-based comparisons useful in tests and error categorization.
- `Debug` — enables readable error output in tests.

---

## Contract Error Reference

### shared — `AccessControlError`

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | Contract already set up |
| 2 | `NotInitialized` | Contract not yet initialized |
| 3 | `Unauthorized` | Caller lacks required permission |
| 4 | `RoleNotFound` | Requested role does not exist |
| 5 | `PermissionDenied` | Permission explicitly denied |
| 6 | `RoleAlreadyExists` | Role already registered |
| 7 | `CannotRevokeOwnRole` | Admin cannot revoke own role |
| 8 | `CannotTransferOwnRole` | Admin cannot transfer own role |
| 9 | `InvalidPermission` | Permission value invalid |
| 10 | `PermissionNotGranted` | Required permission not granted |
| 11 | `InvalidRoleHierarchy` | Role hierarchy violated |
| 12 | `CannotGrantHigherRole` | Cannot grant a role above own level |
| 13 | `InvalidAddress` | Address is malformed or zero |
| 14 | `InvalidRole` | Role identifier invalid |
| 15 | `TemplateNotFound` | Referenced template not found |

---

### analytics — `AnalyticsError`

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | Contract already initialized |
| 2 | `NotInitialized` | Contract not initialized |
| 3 | `Unauthorized` | Caller is not authorized |
| 4 | `InvalidSessionData` | Session data failed validation |
| 5 | `InvalidTimeRange` | Time range is invalid |
| 6 | `InvalidScore` | Score out of valid bounds |
| 7 | `InvalidPercentage` | Percentage out of 0–100 range |
| 8 | `SessionTooShort` | Session duration below minimum |
| 9 | `SessionTooLong` | Session duration above maximum |
| 10 | `SessionNotFound` | Session ID not found |
| 11 | `StudentNotFound` | Student address has no data |
| 12 | `CourseNotFound` | Course ID has no data |
| 13 | `ModuleNotFound` | Module ID has no data |
| 14 | `ReportNotFound` | Report ID has no data |
| 15 | `SessionAlreadyExists` | Session ID already recorded |
| 16 | `SessionNotCompleted` | Operation requires completed session |
| 17 | `InsufficientData` | Not enough data to compute result |
| 18 | `InvalidBatchSize` | Batch size out of valid range |
| 19 | `StorageError` | Storage read/write failure |
| 20 | `InvalidConfiguration` | Configuration value invalid |
| 21 | `UnauthorizedOracle` | Oracle address not authorized |
| 22 | `InvalidInsightData` | Insight data failed validation |
| 23 | `InsightNotFound` | Insight ID not found |

---

### token — `TokenError`

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | Contract already initialized |
| 2 | `NotInitialized` | Contract not initialized |
| 10 | `Unauthorized` | Caller is not authorized |
| 20 | `InvalidAmount` | Amount is zero or negative |
| 21 | `InvalidAddress` | Target address is invalid |
| 80 | `InsufficientBalance` | Sender has insufficient balance |
| 81 | `TransferFailed` | Transfer operation failed |

---

### certificate — `CertificateError`

| Range | Variants |
|---|---|
| 1–3 | Initialization and authorization |
| 10–17 | Multi-signature workflow |
| 20–24 | Certificate lifecycle |
| 30–33 | Template management |
| 40–44 | Configuration |
| 50–51 | Batch operations |
| 60–61 | Compliance |
| 70 | Sharing limits |
| 80, 99 | General errors |

---

### assessment — `AssessmentError`

| Range | Variants |
|---|---|
| 1–3 | Initialization and authorization |
| 10–13 | Configuration and scheduling |
| 20–26 | Questions and submissions |
| 30–31 | Adaptive/accessibility |
| 40 | Security integration |

---

### gamification — `GamificationError`

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | Already initialized |
| 2 | `NotInitialized` | Not initialized |
| 3 | `Unauthorized` | Not authorized |
| 4 | `InvalidAmount` | Invalid XP/token amount |
| 5 | `InvalidInput` | Input validation failed |
| 6 | `NotFound` | Entity not found |
| 7 | `AlreadyExists` | Entity already exists |
| 8–11 | Challenge state errors | Full, inactive, expired, not started |
| 12–14 | Guild membership errors | Full, already in guild, not in guild |
| 15–16 | Season errors | Already active, inactive |
| 17–22 | Social/endorsement errors | Self-endorsement, limit reached, etc. |
| 23–25 | Misc business logic | Guild name too long, season not ended, insufficient XP |

---

### community — `CommunityError`

| Range | Variants |
|---|---|
| 1–5 | Initialization, auth, not found, invalid input |
| 10–14 | Forum: posts and replies |
| 20–24 | Mentorship |
| 30–32 | Contributions |
| 40–43 | Events |
| 50–53 | Moderation |
| 60–63 | Governance |

---

### cross-chain-credentials — `CrossChainError`

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | Contract already initialized |
| 2 | `NotInitialized` | Contract not initialized |
| 10 | `Unauthorized` | Caller not authorized |
| 50 | `CredentialNotFound` | Credential ID not found in storage |
| 51 | `ProofNotFound` | Cross-chain proof not found |
| 52 | `VerificationRequestNotFound` | Verification request not found |
| 80 | `CredentialNotActive` | Credential must be Active for this operation |
| 81 | `CredentialRevoked` | Credential has been revoked |
| 82 | `CredentialSuspended` | Credential is currently suspended |

---

### student-progress-tracker — `StudentProgressError`

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | Contract already initialized |
| 2 | `NotInitialized` | Contract not initialized |
| 10 | `Unauthorized` | Caller not authorized |
| 11 | `AdminNotSet` | Admin address not set (contract not initialized) |
| 20 | `InvalidPercent` | Percentage value exceeds 100 |

---

### progress — `ProgressError`

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | Contract already initialized |
| 2 | `NotInitialized` | Contract not initialized |
| 10 | `Unauthorized` | Caller not authorized |
| 20 | `InvalidProgress` | Progress value is invalid |
| 21 | `InvalidCourseId` | Course ID is invalid |
| 50 | `ProgressNotFound` | No progress recorded for this student/course |

---

### security-monitor — `SecurityError`

| Range | Variants |
|---|---|
| 1–7 | Initialization, auth, configuration |
| 10–12 | Threat management |
| 20–22 | Circuit breaker |
| 30–31 | Rate limiting |
| 40–42 | Event processing |
| 50–52 | Metrics |
| 60–61 | Recommendations |
| 70–71 | Storage |
| 80–81 | General |

---

### diagnostics — `DiagnosticsError`

Uses 1000+ range grouping:

| Range | Sub-system |
|---|---|
| 1001–1099 | Configuration |
| 1101–1199 | Monitoring |
| 1201–1299 | Prediction |
| 1301–1399 | Behavior analysis |
| 1401–1499 | Optimization |
| 1501–1599 | Tracing |
| 1601–1699 | Benchmarking |
| 1701–1799 | Anomaly detection |
| 1801–1899 | Resource management |
| 1901–1999 | Regression testing |
| 2101–2199 | Storage |
| 2301–2399 | General |

---

### mobile-optimizer — `MobileOptimizerError`

| Code | Variant | Code | Variant |
|---|---|---|---|
| 1 | `NotInitialized` | 20 | `AdminNotSet` |
| 2 | `AlreadyInitialized` | 21 | `UnauthorizedAdmin` |
| 3 | `SessionCreationFailed` | 22 | `Unauthorized` |
| 4 | `SessionUpdateFailed` | 23 | `CacheError` |
| 5 | `SessionNotFound` | 24 | `CacheFull` |
| 6 | `SessionExpired` | 25 | `DeviceNotRegistered` |
| 7 | `BatchExecutionFailed` | 26 | `MaxDevicesReached` |
| 8 | `BatchNotFound` | 27 | `SyncFailed` |
| 9 | `BatchExpired` | 28 | `SecurityViolation` |
| 10 | `GasEstimationFailed` | 29 | `BiometricAuthFailed` |
| 11 | `OptimizationFailed` | 30 | `AccountLocked` |
| 12 | `InteractionFailed` | 31 | `NotificationError` |
| 13 | `OfflineOperationFailed` | 32 | `PwaError` |
| 14 | `OfflineSyncFailed` | 33 | `InvalidInput` |
| 15 | `OfflineQueueFull` | 34 | `InternalError` |
| 16 | `ConflictResolutionFailed` | 35 | `ContentError` |
| 17 | `PreferenceUpdateFailed` | 36 | `CollaborationError` |
| 18 | `AnalyticsNotAvailable` | 37 | `UserExperienceError` |
| 19 | `ConfigNotFound` | | |

---

### search — `SearchError`

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | Contract already initialized |
| 2 | `NotInitialized` | Contract not initialized |
| 3 | `Unauthorized` | Caller not authorized |
| 4 | `InvalidQuery` | Search query is invalid |
| 5 | `ContentNotFound` | Content ID not found |
| 6 | `InvalidMetadata` | Metadata failed validation |
| 7 | `InvalidScore` | Score out of valid range |
| 8 | `SessionExpired` | Conversation session expired |
| 9 | `InvalidLanguage` | Language code not supported |
| 10 | `OracleNotAuthorized` | Oracle address not authorized |

---

### documentation — `DocumentationError`

| Code | Variant | Meaning |
|---|---|---|
| 1 | `NotInitialized` | Contract not initialized |
| 2 | `AlreadyInitialized` | Contract already initialized |
| 3 | `Unauthorized` | Caller not authorized |
| 4 | `DocumentNotFound` | Document ID not found |
| 5 | `InvalidDocument` | Document data is invalid |
| 6 | `VersionNotFound` | Document version not found |
| 7 | `ContributionNotFound` | Contribution ID not found |
| 8 | `InvalidContribution` | Contribution data invalid |
| 9 | `TranslationNotFound` | Translation not found |
| 10 | `InvalidLanguage` | Language code not supported |
| 11 | `DocumentTooLarge` | Document exceeds size limit |
| 12 | `InvalidStatus` | Status transition invalid |
| 13 | `AlreadyExists` | Entity already registered |

---

### proxy — `ProxyError`

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | Proxy already configured |
| 2 | `NotInitialized` | Proxy not configured |
| 10 | `Unauthorized` | Caller not authorized |
| 80 | `UpgradeFailed` | Implementation upgrade failed |
| 81 | `RollbackFailed` | Rollback to previous implementation failed |

---

## Adding a New Error Variant

1. Open `contracts/<name>/src/errors.rs`.
2. Choose a code in the appropriate range (see ranges above).
3. Add the variant with `#[repr(u32)]` code — never reuse or change existing codes.
4. The error is automatically available in the contract's `Result<T, NameError>` return type.

```rust
// Example: adding a new validation error to TokenError
InvalidDecimals = 22,  // fits in the 20-49 validation range
```

---

## Testing Error Paths

Use the Soroban test client's `try_*` methods to capture errors without panicking:

```rust
// Returns Ok(Err(CrossChainError::CredentialNotFound)) when credential is missing
let result = client.try_get_credential(&nonexistent_id);
assert_eq!(result, Ok(Err(CrossChainError::CredentialNotFound)));

// Returns Ok(Err(StudentProgressError::InvalidPercent)) for percent > 100
let result = client.try_update_progress(&student, &course, &module, &150u32);
assert_eq!(result, Ok(Err(StudentProgressError::InvalidPercent)));
```

The outer `Ok` means the call reached the contract (no host-level error). The inner `Err(E)` is the contract's own error.
