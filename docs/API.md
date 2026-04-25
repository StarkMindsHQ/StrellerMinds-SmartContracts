# StrellerMinds Smart Contracts API Reference

Complete API reference for the StrellerMinds educational platform smart contracts on Soroban.

## Contract Overview

| Contract | Description | Functions |
|----------|-------------|-----------|
| `token` | Token mint, transfer, and balance operations | 5 |
| `certificate` | Multi-sig certificate issuance, verification, and revocation | 24 |
| `assessment` | Assessment creation, submission, and grading | 24 |
| `progress` | Student progress tracking | 5 |
| `analytics` | Learning analytics and reporting | 15 |
| `gamification` | Achievements, challenges, guilds, seasons | 26 |
| `community` | Forum, events, mentorship | 23 |
| `marketplace` | Learning path marketplace | 8 |

## Quick Reference

### Token Contract

```rust
// Initialize
client.initialize(&admin);

// Mint tokens (admin only, rate limited: 50/day)
client.mint(&recipient, &1000u64);

// Transfer tokens (rate limited: 100/day per user)
client.transfer(&sender, &recipient, &500u64);

// Query balance (returns 0 - stub implementation)
let balance = client.balance(&account);
```

### Certificate Contract

```rust
// Initialize with admin
client.initialize(&admin);
# StrellerMinds Smart Contracts — API Reference

All contracts are deployed on the Stellar network and invoked via the Soroban SDK.
Authentication is handled by Stellar keypairs: the caller signs the transaction, and the
contract validates the signature with `address.require_auth()`.

## Table of Contents

1. [Analytics Contract](#1-analytics-contract)
2. [Certificate Contract](#2-certificate-contract)
3. [Token Contract](#3-token-contract)
4. [Progress Contract](#4-progress-contract)
5. [Community Contract](#5-community-contract)
6. [Assessment Contract](#6-assessment-contract)
7. [Shared Utilities](#7-shared-utilities)
8. [Error Codes](#8-error-codes)
9. [Common Types](#9-common-types)

---

## 1. Analytics Contract

Tracks learning sessions, computes per-student and per-course analytics, and generates
AI-powered learning path recommendations.

### `initialize`

```
initialize(admin: Address, config: AnalyticsConfig) -> Result<(), AnalyticsError>
```

Initialises the contract. Must be called exactly once before any other function.

| Parameter | Type              | Description                              |
|-----------|-------------------|------------------------------------------|
| `admin`   | `Address`         | Address granted administrative control.  |
| `config`  | `AnalyticsConfig` | Initial analytics configuration values. |

**Errors**: `AlreadyInitialized`

---

### `record_session`

```
record_session(session: LearningSession) -> Result<(), AnalyticsError>
```

Records the start of a new learning session. The `session.student` must sign the transaction.

| Parameter | Type              | Description                    |
|-----------|-------------------|--------------------------------|
| `session` | `LearningSession` | Full session data to persist.  |

**Errors**: `NotInitialized`, `SessionAlreadyExists`

---

### `complete_session`

```
complete_session(
    session_id:            BytesN<32>,
    end_time:              u64,
    final_score:           Option<u32>,
    completion_percentage: u32,
) -> Result<(), AnalyticsError>
```

Marks a session as completed and triggers progress analytics recalculation and
achievement checks.

| Parameter               | Type          | Description                                      |
|-------------------------|---------------|--------------------------------------------------|
| `session_id`            | `BytesN<32>`  | Unique session identifier.                       |
| `end_time`              | `u64`         | Unix timestamp when the session ended.           |
| `final_score`           | `Option<u32>` | Optional assessment score (0–100).               |
| `completion_percentage` | `u32`         | Percentage of module content completed (0–100).  |

**Errors**: `SessionNotFound`

---

### `get_session`

```
get_session(session_id: BytesN<32>) -> Option<LearningSession>
```

Returns a single learning session by ID.

---

### `get_student_sessions`

```
get_student_sessions(student: Address, course_id: Symbol) -> Vec<BytesN<32>>
```

Returns all session IDs for a student within a course.

---

### `get_progress_analytics`

```
get_progress_analytics(student: Address, course_id: Symbol)
    -> Result<ProgressAnalytics, AnalyticsError>
```

Returns pre-computed progress analytics for a student–course pair.

**Errors**: `StudentNotFound`

---

### `get_course_analytics`

```
get_course_analytics(course_id: Symbol) -> Result<CourseAnalytics, AnalyticsError>
```

Computes and returns analytics aggregated from **all** enrolled students. For courses
with more than ~500 students, prefer `get_course_analytics_paginated` to avoid
per-transaction instruction limits.

**Errors**: `CourseNotFound`

---

### `get_course_students_count`

```
get_course_students_count(course_id: Symbol) -> u32
```

Returns the total number of students enrolled in a course. Use this to determine how
many pages are needed before calling `get_course_analytics_paginated`.

---

### `get_course_analytics_paginated`

```
get_course_analytics_paginated(
    course_id: Symbol,
    offset:    u32,
    limit:     u32,
) -> Result<CourseAnalytics, AnalyticsError>
```

Computes course analytics over a slice of enrolled students defined by `[offset, offset+limit)`.
The internal page size is capped at **200** to keep instruction counts bounded.
The `total_students` field in the returned struct always reflects the full enrolment count
so the caller can detect whether more pages remain.

**Usage example** (client-side aggregation):

```rust
let total = client.get_course_students_count(&course_id);
let page_size = 100u32;
let mut page = 0u32;
while page * page_size < total {
    let analytics = client.get_course_analytics_paginated(&course_id, &(page * page_size), &page_size);
    // merge analytics into running aggregate
    page += 1;
}
```

**Errors**: `CourseNotFound`

---

### `get_module_analytics`

```
get_module_analytics(course_id: Symbol, module_id: Symbol)
    -> Result<ModuleAnalytics, AnalyticsError>
```

Returns analytics for a specific module within a course, computing them on-the-fly if
not already cached.

**Errors**: `ModuleNotFound`

---

### `generate_progress_report`

```
generate_progress_report(
    student:    Address,
    course_id:  Symbol,
    period:     ReportPeriod,
    start_date: u64,
    end_date:   u64,
) -> Result<ProgressReport, AnalyticsError>
```

Generates a progress report for a student over the specified date range.

**Errors**: `InvalidTimeRange`, `InsufficientData`

---

### `generate_leaderboard`

```
generate_leaderboard(course_id: Symbol, metric: LeaderboardMetric, limit: u32)
    -> Result<Vec<LeaderboardEntry>, AnalyticsError>
```

Generates a ranked leaderboard for a course on the given metric.

| `metric` value      | Description                     |
|---------------------|---------------------------------|
| `TotalScore`        | Ranked by cumulative score      |
| `CompletionSpeed`   | Ranked by time-to-completion    |
| `ConsistencyScore`  | Ranked by session regularity    |
| `TimeSpent`         | Ranked by total study time      |

---

### `get_struggling_students`

```
get_struggling_students(course_id: Symbol, threshold: u32) -> Vec<Address>
```

Returns addresses of students whose average score is below `threshold`.

---

### `get_learning_recommendations` *(Issue #370)*

```
get_learning_recommendations(student: Address, course_id: Symbol)
    -> Result<Vec<LearningRecommendation>, AnalyticsError>
```

Generates adaptive learning path recommendations for a student based on their recorded
performance trend, average score, and streak data.

Recommendation tiers:

| Condition                                           | Recommended path     |
|-----------------------------------------------------|----------------------|
| Average score < 70 **or** trend = `Declining`       | Remedial review      |
| Average score ≥ 85 **and** trend = `Improving`      | Advanced / accelerated |
| Otherwise                                           | Standard next module |
| Streak = 0 (additional recommendation)              | Re-engagement module |

The result is also persisted as an `MLInsight` with type `AdaptiveRecommendation`
and is retrievable via `get_ml_insight`.

**Errors**: `StudentNotFound`

---

### `get_learning_path_optimization` *(Issue #370)*

```
get_learning_path_optimization(student: Address, course_id: Symbol)
    -> Result<LearningPathOptimization, AnalyticsError>
```

Returns an optimised module sequence with a difficulty progression curve and an
estimated time savings figure. Struggling students receive an easy-to-hard ordering;
high-performing students are directed straight to intermediate and advanced material.

Includes a `confidence` score (0–100) based on the performance trend:

| Trend          | Confidence |
|----------------|------------|
| `Improving`    | 85         |
| `Stable`       | 75         |
| `Declining`    | 60         |
| `Insufficient` | 50         |

**Errors**: `StudentNotFound`

---

### `predict_course_completion` *(Issue #370)*

```
predict_course_completion(student: Address, course_id: Symbol)
    -> Result<MLInsight, AnalyticsError>
```

Returns a completion-probability prediction derived from completion percentage, average
score, and streak length. The `confidence` field (0–100) represents the predicted
probability, and `data` contains a human-readable risk category:

- `"HIGH: on track to complete"` (≥ 75)
- `"MEDIUM: at risk, intervention recommended"` (50–74)
- `"LOW: high dropout risk, immediate support needed"` (< 50)

**Errors**: `StudentNotFound`

---

### `get_ml_insight`

```
get_ml_insight(student: Address, course_id: Symbol, insight_type: InsightType)
    -> Option<MLInsight>
```

Returns a previously stored ML insight for the given student–course–type triple.

---

### `update_config`

```
update_config(admin: Address, config: AnalyticsConfig) -> Result<(), AnalyticsError>
```

Updates the analytics configuration. Requires admin authorisation.

**Errors**: `Unauthorized`

---

### `transfer_admin`

```
transfer_admin(current_admin: Address, new_admin: Address) -> Result<(), AnalyticsError>
```

Transfers the admin role. Requires the current admin to sign.

**Errors**: `Unauthorized`

---

### `cleanup_old_data`

```
cleanup_old_data(admin: Address, before_date: u64) -> Result<u32, AnalyticsError>
```

Removes sessions created before `before_date`. Returns the number of sessions removed.

**Errors**: `Unauthorized`

---

### `health_check`

```
health_check() -> ContractHealthReport
```

Returns a health snapshot indicating whether the contract is initialised and responsive.

---

## 2. Certificate Contract

Issues, verifies, revokes, and governs on-chain certificates through a multi-signature
approval workflow.

### `initialize`

```
initialize(admin: Address) -> Result<(), CertificateError>
```

Initialises the contract with a default rate-limit configuration (10 requests/day).

**Errors**: `AlreadyInitialized`

---

### Multi-Signature Certificate Approval *(Issue #366)*

The certificate contract enforces a configurable multi-signature workflow before any
certificate can be issued, providing governance guarantees for high-stakes credentials.

#### `configure_multisig`

```
configure_multisig(admin: Address, config: MultiSigConfig) -> Result<(), CertificateError>
```

Sets the approval requirements for a course. Must be called before creating requests
for that course.

| `MultiSigConfig` field    | Type           | Description                                       |
|---------------------------|----------------|---------------------------------------------------|
| `course_id`               | `String`       | Course the config applies to.                     |
| `required_approvals`      | `u32`          | Number of approvals needed (must be ≤ approver count). |
| `authorized_approvers`    | `Vec<Address>` | List of addresses allowed to approve (max 10).    |
| `timeout_duration`        | `u64`          | Seconds before a pending request expires (1 h–30 d). |
| `priority`                | `CertificatePriority` | Priority tier (`Standard`, `Premium`, `Enterprise`, `Institutional`). |
| `auto_execute`            | `bool`         | Issue certificate automatically when threshold is met. |

**Errors**: `Unauthorized`, `InvalidApprovalThreshold`, `TooManyApprovers`,
`TimeoutTooShort`, `TimeoutTooLong`

---

#### `create_multisig_request`

```
create_multisig_request(
    requester: Address,
    params:    MintCertificateParams,
    reason:    String,
) -> Result<BytesN<32>, CertificateError>
```

Creates a pending certificate issuance request. Returns the `request_id` used in all
subsequent approval calls. Non-admin callers are subject to rate limiting.

**Errors**: `ConfigNotFound`, `RateLimitExceeded`

---

#### `process_multisig_approval`

```
process_multisig_approval(
    approver:       Address,
    request_id:     BytesN<32>,
    approved:       bool,
    comments:       String,
    signature_hash: Option<Bytes>,
) -> Result<(), CertificateError>
```

Records an approval (`approved = true`) or rejection (`approved = false`).
When the approval count reaches `required_approvals` and `auto_execute` is enabled,
the certificate is issued automatically.
A single rejection immediately moves the request to `Rejected` status.

**Errors**: `MultiSigRequestNotFound`, `RequestNotPending`, `MultiSigRequestExpired`,
`ApproverNotAuthorized`, `AlreadyApproved`

---

#### `execute_multisig_request`

```
execute_multisig_request(executor: Address, request_id: BytesN<32>)
    -> Result<(), CertificateError>
```

Manually issues the certificate for a request already in `Approved` status.
Use when `auto_execute` is disabled.

**Errors**: `MultiSigRequestNotFound`, `RequestAlreadyExecuted`, `InsufficientApprovals`

---

#### `cleanup_expired_requests`

```
cleanup_expired_requests() -> Result<u32, CertificateError>
```

Scans all pending requests and transitions any past their `expires_at` deadline to
`Expired` status. Returns the count of requests expired. Call this periodically to
keep the pending queue accurate and analytics up to date.

**Errors**: `NotInitialized`

---

#### `get_multisig_request`

```
get_multisig_request(request_id: BytesN<32>) -> Option<MultiSigCertificateRequest>
```

Returns the full request record including approval history and current status.

---

#### `get_pending_approvals`

```
get_pending_approvals(approver: Address) -> Vec<BytesN<32>>
```

Returns all request IDs currently awaiting the given approver's decision.

---

#### `get_multisig_audit_trail`

```
get_multisig_audit_trail(request_id: BytesN<32>) -> Vec<MultiSigAuditEntry>
```

Returns the chronological audit trail of actions taken on a request (Created,
ApprovalGranted, ApprovalRejected, Expired, Executed, etc.).

---

### `batch_issue_certificates`

```
batch_issue_certificates(admin: Address, params_list: Vec<MintCertificateParams>)
    -> Result<BatchResult, CertificateError>
```

Issues up to 100 certificates in a single transaction. Duplicates within the batch or
in storage are skipped and counted as `failed`.

**Errors**: `BatchEmpty`, `BatchTooLarge`, `Unauthorized`

---

### `verify_certificate`

```
verify_certificate(certificate_id: BytesN<32>) -> Result<bool, CertificateError>
```

Returns `true` if the certificate is `Active`, not expired, and has a blockchain anchor.

**Errors**: `CertificateNotFound`

---

### `revoke_certificate`

```
revoke_certificate(
    admin:                Address,
    certificate_id:       BytesN<32>,
    reason:               String,
    reissuance_eligible:  bool,
) -> Result<(), CertificateError>
```

Revokes an active certificate. Set `reissuance_eligible = true` to allow the student
to request a replacement via `reissue_certificate`.

**Errors**: `CertificateNotFound`, `CertificateRevoked`, `Unauthorized`

---

### `reissue_certificate`

```
reissue_certificate(
    admin:              Address,
    old_certificate_id: BytesN<32>,
    new_params:         MintCertificateParams,
) -> Result<BytesN<32>, CertificateError>
```

Issues a replacement certificate. The original is marked `Reissued`; the new one
receives an incremented `version` number.

**Errors**: `CertificateNotFound`, `CertificateNotEligibleForReissue`, `Unauthorized`

---

### `verify_compliance`

```
verify_compliance(
    verifier:       Address,
    certificate_id: BytesN<32>,
    standard:       ComplianceStandard,
    notes:          String,
) -> Result<bool, CertificateError>
```

Records a compliance check result for a certificate against a standard such as
`Iso17024` or `Iso9001`.

| `ComplianceStandard` values |
|-----------------------------|
| `Iso17024`                  |
| `Iso9001`                   |
| `Gdpr`                      |
| `Custom`                    |

**Errors**: `CertificateNotFound`

---

### `share_certificate`

```
share_certificate(
    owner:            Address,
    certificate_id:   BytesN<32>,
    platform:         String,
    verification_url: String,
) -> Result<(), CertificateError>
```

Records a social share of a certificate. Only the certificate owner may share it.
Share count is capped at 100 per certificate.

**Errors**: `CertificateNotFound`, `CertificateRevoked`, `Unauthorized`, `ShareLimitReached`

---

### `get_student_certificates`

```
get_student_certificates(student: Address) -> Vec<BytesN<32>>
```

Returns IDs of **active, unexpired** certificates for the student.

### `get_all_student_certificates`

```
get_all_student_certificates(student: Address) -> Vec<BytesN<32>>
```

Returns IDs of **all** certificates (including revoked and expired) for the student.

---

## 3. Token Contract

Manages the STRM token: minting, transfers, staking, and incentive distribution.

### `mint`

```
mint(to: Address, amount: i128) -> Result<(), TokenError>
```

Mints `amount` tokens to `to`. Requires admin authorisation.

### `transfer`

```
transfer(from: Address, to: Address, amount: i128) -> Result<(), TokenError>
```

Transfers `amount` tokens from `from` to `to`. `from` must sign.

### `stake`

```
stake(user: Address, amount: i128) -> Result<(), TokenError>
```

Stakes `amount` tokens for `user`. Staked tokens accrue rewards over time.

### `unstake`

```
unstake(user: Address, amount: i128) -> Result<(), TokenError>
```

Unstakes `amount` tokens, returning them to the user's spendable balance.

### `claim_rewards`

```
claim_rewards(user: Address) -> Result<i128, TokenError>
```

Claims accrued staking rewards for `user`. Returns the amount claimed.

### `balance`

```
balance(user: Address) -> i128
```

Returns the current spendable token balance of `user`.

---

## 4. Progress Contract

Records module-level course progress.

### `record_progress`

```
record_progress(student: Address, course_id: Symbol, module_id: Symbol, progress: u32)
    -> Result<(), ProgressError>
```

Records `progress` (0–100) for `student` in `module_id` of `course_id`.

### `get_progress`

```
get_progress(student: Address, course_id: Symbol, module_id: Symbol) -> Option<u32>
```

Returns the recorded progress percentage, or `None` if not found.

### `get_course_completion`

```
get_course_completion(student: Address, course_id: Symbol) -> u32
```

Returns the overall course completion percentage averaged across all modules.

---

## 5. Community Contract

Provides forum, mentorship, governance, and community moderation features.

### Forum

| Function              | Signature                                                   | Description               |
|-----------------------|-------------------------------------------------------------|---------------------------|
| `create_post`         | `(author, title, content, tags) -> Result<u64, ...>`        | Creates a forum post.     |
| `reply_to_post`       | `(author, post_id, content) -> Result<u64, ...>`            | Adds a reply to a post.   |
| `upvote_post`         | `(voter, post_id) -> Result<(), ...>`                       | Upvotes a post.           |

### Mentorship

| Function                  | Signature                                           | Description                      |
|---------------------------|-----------------------------------------------------|----------------------------------|
| `register_mentor`         | `(mentor, expertise) -> Result<(), ...>`            | Registers a mentor.              |
| `request_mentorship`      | `(student, mentor, topic) -> Result<u64, ...>`      | Sends a mentorship request.      |
| `accept_mentorship`       | `(mentor, request_id) -> Result<(), ...>`           | Accepts a pending request.       |

### Governance

| Function               | Signature                                                  | Description                    |
|------------------------|------------------------------------------------------------|--------------------------------|
| `create_proposal`      | `(proposer, title, description, duration) -> Result<...>`  | Creates a governance proposal. |
| `vote`                 | `(voter, proposal_id, in_favor) -> Result<(), ...>`        | Casts a vote on a proposal.    |
| `execute_proposal`     | `(proposal_id) -> Result<(), ...>`                         | Executes a passed proposal.    |

---

## 6. Assessment Contract

Manages quizzes, assignments, and automated grading.

### `create_assessment`

```
create_assessment(admin, course_id, module_id, questions, passing_score)
    -> Result<BytesN<32>, AssessmentError>
```

Creates an assessment for a module. Returns the assessment ID.

### `submit_response`

```
submit_response(student, assessment_id, answers) -> Result<u32, AssessmentError>
```

Submits student answers and returns the computed score. Triggers grading immediately.

### `get_result`

```
get_result(student, assessment_id) -> Option<AssessmentResult>
```

Returns the stored result for a student–assessment pair.

---

## 7. Shared Utilities

### Role-Based Access Control

```
grant_role(admin: Address, grantee: Address, role: Role) -> Result<(), SharedError>
require_role(env: &Env, caller: &Address, role: Role) -> Result<(), SharedError>
revoke_role(admin: Address, grantee: Address, role: Role) -> Result<(), SharedError>
```

| `Role` values  |
|----------------|
| `Admin`        |
| `Instructor`   |
| `Verifier`     |
| `Moderator`    |

### Rate Limiter

```
enforce_rate_limit(env: &Env, key: &DataKey, config: &RateLimitConfig)
    -> Result<(), RateLimitError>
```

Enforces a sliding-window rate limit keyed on an arbitrary storage key.

### Reentrancy Guard

```
acquire_lock(env: &Env) -> Result<(), SharedError>
release_lock(env: &Env)
```

Prevents reentrant calls to sensitive contract functions.

---

## 8. Error Codes

### Analytics

| Error                  | Description                                         |
|------------------------|-----------------------------------------------------|
| `NotInitialized`       | Contract has not been initialised.                  |
| `AlreadyInitialized`   | `initialize` called more than once.                 |
| `Unauthorized`         | Caller is not the admin.                            |
| `SessionNotFound`      | No session with the given ID.                       |
| `SessionAlreadyExists` | A session with that ID already exists.              |
| `StudentNotFound`      | No analytics record for the student–course pair.   |
| `CourseNotFound`       | No students enrolled in the course.                 |
| `ModuleNotFound`       | No analytics for the module.                        |
| `InvalidTimeRange`     | `start_date >= end_date`.                           |
| `InsufficientData`     | Not enough sessions in the requested range.         |

### Certificate

| Error                          | Description                                               |
|--------------------------------|-----------------------------------------------------------|
| `NotInitialized`               | Contract has not been initialised.                        |
| `AlreadyInitialized`           | `initialize` called more than once.                       |
| `Unauthorized`                 | Caller is not the admin or certificate owner.             |
| `CertificateNotFound`          | No certificate with the given ID.                         |
| `CertificateRevoked`           | Certificate has been revoked.                             |
| `CertificateNotEligibleForReissue` | Certificate is not revoked or not marked eligible.    |
| `ConfigNotFound`               | No multi-sig config for the course.                       |
| `MultiSigRequestNotFound`      | No request with the given ID.                             |
| `RequestNotPending`            | Request is not in `Pending` status.                       |
| `MultiSigRequestExpired`       | Request deadline has passed.                              |
| `ApproverNotAuthorized`        | Address is not in the authorised approver list.           |
| `AlreadyApproved`              | Approver has already submitted a decision.                |
| `InsufficientApprovals`        | Request has not yet reached `Approved` status.            |
| `RequestAlreadyExecuted`       | Request has already been executed.                        |
| `TooManyApprovers`             | Approver list exceeds the maximum of 10.                  |
| `InvalidApprovalThreshold`     | `required_approvals` is 0 or exceeds available approvers. |
| `TimeoutTooShort`              | `timeout_duration` < 1 hour.                              |
| `TimeoutTooLong`               | `timeout_duration` > 30 days.                             |
| `RateLimitExceeded`            | Caller has exceeded the per-day request limit.            |
| `BatchEmpty`                   | Batch list is empty.                                      |
| `BatchTooLarge`                | Batch exceeds 100 items.                                  |
| `ShareLimitReached`            | Certificate has been shared the maximum number of times.  |
| `TemplateNotFound`             | No template with the given ID.                            |
| `TemplateAlreadyExists`        | A template with that ID already exists.                   |
| `TemplateInactive`             | Template is no longer active.                             |
| `MissingRequiredField`         | Fewer field values supplied than required fields.         |
| `InvalidProof`                 | ZKP proof is too short or malformed.                      |
| `VerificationFailed`           | ZKP verification did not succeed.                         |

---

## 9. Common Types

### `LearningSession`

```
{
  session_id:             BytesN<32>,
  student:                Address,
  course_id:              Symbol,
  module_id:              Symbol,
  start_time:             u64,       // Unix timestamp
  end_time:               u64,       // 0 until complete_session is called
  completion_percentage:  u32,       // 0–100
  time_spent:             u64,       // seconds
  interactions:           u32,
  score:                  Option<u32>,
  session_type:           SessionType, // Study | Assessment | Practice | Review
}
```

### `ProgressAnalytics`

```
{
  student:               Address,
  course_id:             Symbol,
  total_modules:         u32,
  completed_modules:     u32,
  completion_percentage: u32,
  total_time_spent:      u64,
  average_session_time:  u64,
  total_sessions:        u32,
  last_activity:         u64,
  first_activity:        u64,
  average_score:         Option<u32>,
  streak_days:           u32,
  performance_trend:     PerformanceTrend, // Improving | Stable | Declining | Insufficient
}
```

### `LearningRecommendation`

```
{
  target_module:        Symbol,
  reason:               String,
  priority:             u32,       // 1 = highest
  estimated_difficulty: u32,       // 1–10
  prerequisites:        Vec<Symbol>,
  learning_resources:   Vec<String>,
  adaptive_path:        bool,
}
```

### `LearningPathOptimization`

```
{
  student:                Address,
  course_id:              Symbol,
  optimized_path:         Vec<Symbol>,  // ordered module sequence
  estimated_time_savings: u32,          // minutes saved
  difficulty_progression: Vec<u32>,     // difficulty score per module
  adaptation_reason:      String,
  confidence:             u32,          // 0–100
}
```

### `MLInsight`

```
{
  insight_id:    BytesN<32>,
  student:       Address,
  course_id:     Symbol,
  insight_type:  InsightType,
  data:          String,
  confidence:    u32,       // 0–100
  timestamp:     u64,
  model_version: u32,
  metadata:      Vec<(String, String)>,
}
```

### `MultiSigCertificateRequest`

```
{
  request_id:           BytesN<32>,
  certificate_params:   MintCertificateParams,
  requester:            Address,
  required_approvals:   u32,
  current_approvals:    u32,
  approvers:            Vec<Address>,
  approval_records:     Vec<ApprovalRecord>,
  status:               MultiSigRequestStatus, // Pending | Approved | Rejected | Executed | Expired
  created_at:           u64,
  expires_at:           u64,
  reason:               String,
  priority:             CertificatePriority,
}
```

### `CourseAnalytics`

```
{
  course_id:                Symbol,
  total_students:           u32,
  active_students:          u32,   // active within last 30 days
  completion_rate:          u32,   // percentage
  average_completion_time:  u64,   // seconds
  average_score:            Option<u32>,
  dropout_rate:             u32,
  most_difficult_module:    Option<Symbol>,
  easiest_module:           Option<Symbol>,
  total_time_invested:      u64,   // sum of all student time in seconds
}
```

// Configure multi-sig for a course
client.configure_multisig(&admin, &multisig_config);

// Create issuance request (requires multi-sig approval)
let request_id = client.create_multisig_request(&requester, &params, &reason);

// Approve/reject request
client.process_multisig_approval(&approver, &request_id, true, &comments, None);

// Execute approved request
client.execute_multisig_request(&executor, &request_id);

// Batch issue certificates
let result = client.batch_issue_certificates(&admin, &params_list);

// Verify certificate
let is_valid = client.verify_certificate(&certificate_id);

// Revoke certificate
client.revoke_certificate(&admin, &certificate_id, &reason, true);
```

### Progress Contract

```rust
// Record progress (rate limited: 100 updates/day per student)
client.record_progress(&student, &course_id, &75u32);

// Query progress
let progress = client.get_progress(&student, &course_id);

// Get all courses for a student
let courses = client.get_student_courses(&student);
```

### Assessment Contract

```rust
// Initialize
client.initialize(&admin);

// Create assessment
let assessment_id = client.create_assessment(&admin, &metadata, &config, &Vec::new(&env));

// Add questions
client.add_questions(&admin, assessment_id, questions);

// Start submission
let submission_id = client.start_submission(&student, assessment_id);

// Submit answers
client.submit_answers(&student, submission_id, answers);

// Grade submission
client.grade_submission(&admin, submission_id, &grades);
```

### Gamification Contract

```rust
// Initialize and seed achievements
client.initialize(&admin);

// Record activity (returns newly earned achievement IDs)
let achievements = client.record_activity(&user, &activity);

// Get user profile
let profile = client.get_user_profile(&user);

// Create challenge
let challenge_id = client.create_challenge(&admin, &challenge);

// Join challenge
client.join_challenge(&user, challenge_id);

// Create guild
let guild_id = client.create_guild(&creator, &name, &description, &max_members, &is_public);

// Endorse peer
client.endorse_peer(&endorser, &endorsee, &skill);

// Recognize peer
client.recognize_peer(&from, &to, &recognition_type, &message);

// Get leaderboard
let leaderboard = client.get_leaderboard(&category, 50);
```

## Error Codes

| Prefix | Contract | Description |
|--------|----------|-------------|
| `CERT-*` | Certificate | Certificate operations |
| `TKN-*` | Token | Token operations |
| `ASSESS-*` | Assessment | Assessment operations |
| `PROG-*` | Progress | Progress tracking |
| `ANAL-*` | Analytics | Analytics operations |
| `GAM-*` | Gamification | Gamification operations |
| `COMM-*` | Community | Community operations |
| `MKP-*` | Marketplace | Marketplace operations |
| `SHR-*` | Shared | Cross-contract errors |

## Rate Limits

| Contract | Operation | Limit |
|----------|-----------|-------|
| Token | Mint | 50/day per user |
| Token | Transfer | 100/day per user |
| Progress | Record progress | 100/day per student |
| Assessment | Start submission | 3/day per student |
| Assessment | Submit answers | 5/day per student |
| Certificate | Multi-sig request | 10/day per requester |
| Gamification | Record activity | 100/day per user |
| Gamification | Recognition | 10/day per user |

## Health Checks

All contracts implement `health_check()` returning a `ContractHealthReport`:

```json
{
  "status": "healthy",
  "contract": "token",
  "version": "1.0.0",
  "timestamp": 1714000000
}
```

## Events

Contracts emit events for off-chain monitoring:

| Event | Contract | Purpose |
|-------|----------|---------|
| `ContractInitialized` | All | Contract initialization |
| `TokensMinted` | Token | Token minting |
| `TokensTransferred` | Token | Token transfers |
| `CertificateIssued` | Certificate | Certificate issuance |
| `CertificateRevoked` | Certificate | Certificate revocation |
| `ProgressUpdated` | Progress | Progress updates |
| `AchievementEarned` | Gamification | Achievement awards |

## OpenAPI Specifications

Detailed OpenAPI 3.0 specifications for each contract are available in the `openapi/` directory:

- [Certificate OpenAPI](openapi/certificate.yaml)
- [Token OpenAPI](openapi/token.yaml)
- [Assessment OpenAPI](openapi/assessment.yaml)
- [Analytics OpenAPI](openapi/analytics.yaml)
- [Community OpenAPI](openapi/community.yaml)
- [Gamification OpenAPI](openapi/gamification.yaml)
- [Marketplace OpenAPI](openapi/marketplace.yaml)
- [Progress OpenAPI](openapi/progress.yaml)
*For SDK integration examples, see the [Development Guide](development.md).*
*For deployment instructions, see the [Deployment Guide](DEPLOYMENT.md).*
