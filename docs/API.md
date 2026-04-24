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