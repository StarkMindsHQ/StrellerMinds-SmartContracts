# Assessment Contract

## Purpose

The Assessment contract provides a full on-chain examination engine for the StrellerMinds educational platform. It supports the complete lifecycle of a quiz or test: creating assessments, adding heterogeneous question types, scheduling availability windows, granting per-student accessibility accommodations, running adaptive testing sessions that adjust difficulty in real time, auto-grading submissions, recording academic integrity metadata from external oracles, and aggregating per-student progress across a course. All state mutations emit structured events for consumption by the analytics and progress-tracking contracts.

## Architecture

| Module | Description |
|---|---|
| `lib.rs` | Contract entry point — all 20 public functions and private helpers; no sub-manager pattern (single-file design) |
| `grading.rs` | `GradingEngine` — auto-grades submitted answers for all supported question types; determines whether manual review is required |
| `types.rs` | `contracttype`-derived structs: `AssessmentMetadata`, `AssessmentConfig`, `Question`, `QuestionOption`, `AnswerKey`, `Submission`, `SubmittedAnswer`, `ScheduleConfig`, `AccommodationConfig`, `AdaptiveState`, `IntegrationConfig`, `IntegrityMetadata` |
| `events.rs` | `AssessmentEvents` — typed event emitters for initialization, assessment lifecycle, submission events, and integrity flags |
| `errors.rs` | `AssessmentError` — typed error variants across 5 categories |

## Public API

| Function | Description | Auth Required |
|---|---|---|
| `initialize(admin)` | One-time setup; sets the admin address and empty integration config | Admin |
| `set_integration(admin, analytics, progress, security_monitor)` | Configures addresses of integrated external contracts | Admin |
| `create_assessment(instructor, course_id, module_id, config)` | Creates an unpublished assessment; returns its generated ID | Instructor |
| `publish_assessment(admin, assessment_id)` | Marks an assessment as published and available to students | Admin |
| `add_question_single_choice(admin, assessment_id, max_score, difficulty, content_hash, options, correct_option_id)` | Adds a single-choice question | Admin |
| `add_question_multiple_choice(admin, assessment_id, max_score, difficulty, content_hash, options, correct_option_ids)` | Adds a multiple-choice question | Admin |
| `add_question_numeric_range(admin, assessment_id, max_score, difficulty, content_hash, options, min, max)` | Adds a numeric range question | Admin |
| `add_question_short_text(admin, assessment_id, max_score, difficulty, content_hash, options, accepted_answers)` | Adds a short-text question with accepted answer variants | Admin |
| `add_question(admin, assessment_id, question_type, max_score, difficulty, content_hash, options, answer_key)` | Generic question addition for any supported type | Admin |
| `get_assessment_metadata(assessment_id)` | Returns assessment metadata, or `None` if not found | None |
| `set_schedule(admin, assessment_id, start_time, end_time, tz_offset, proctoring_provider)` | Sets the availability window for an assessment | Admin |
| `set_accommodation(admin, student, config)` | Grants a student extra time and/or extra attempts | Admin |
| `get_accommodation_for_student(student)` | Returns a student's accommodation config | None |
| `get_next_question(student, assessment_id)` | Returns the next adaptive question for a student (adaptive mode only) | User |
| `update_adaptive_state(student, assessment_id, question_id, was_correct)` | Updates the student's adaptive difficulty level after answering | User |
| `start_submission(student, assessment_id)` | Opens a new in-progress submission; returns the submission ID | User |
| `submit_answers(student, submission_id, answers)` | Finalizes a submission, auto-grades it, and returns the result | User |
| `get_submission_details(submission_id)` | Returns a submission by ID | None |
| `update_integrity_metadata(oracle_or_admin, submission_id, plagiarism_score, plagiarism_flag, integrity_flags, has_proctoring_evidence, proctoring_evidence_hash)` | Attaches integrity/plagiarism data from an oracle | Admin / Oracle |
| `get_course_assessment_progress(student, course_id)` | Returns the latest score for each assessment in a course for a student | None |

## Usage Example

```
# 1. Admin initializes and links external contracts
assessment.initialize(admin)
assessment.set_integration(admin, analytics_addr, progress_addr, security_monitor_addr)

# 2. Instructor creates an assessment for a module
assessment_id = assessment.create_assessment(instructor, "RUST101", "M1", {
    pass_score: 70,
    max_attempts: 3,
    time_limit_seconds: 3600,
    is_adaptive: false
})

# 3. Admin adds questions
q1 = assessment.add_question_single_choice(admin, assessment_id, 10, 2, content_hash, options, correct_id)
q2 = assessment.add_question_multiple_choice(admin, assessment_id, 20, 3, content_hash, options, [1, 3])

# 4. Admin schedules and publishes
assessment.set_schedule(admin, assessment_id, open_time, close_time, 0, None)
assessment.publish_assessment(admin, assessment_id)

# 5. Student takes the assessment
submission_id = assessment.start_submission(student, assessment_id)
result = assessment.submit_answers(student, submission_id, [
    {question_id: q1, answer: SingleChoice(2)},
    {question_id: q2, answer: MultipleChoice([1, 3])}
])
# result.passed == true if score >= pass_score

# 6. Security monitor attaches proctoring evidence
assessment.update_integrity_metadata(security_monitor, submission_id, 0, false, [], true, evidence_hash)
```

## Errors

| Error | Code | Description |
|---|---|---|
| `AlreadyInitialized` | 1 | Contract has already been initialized |
| `NotInitialized` | 2 | Contract has not been initialized |
| `Unauthorized` | 3 | Caller is not authorized |
| `InvalidConfig` | 10 | Assessment configuration contains invalid values |
| `InvalidSchedule` | 11 | Schedule is invalid (end_time not after start_time) |
| `AssessmentNotFound` | 12 | No assessment found with the given ID |
| `AssessmentNotPublished` | 13 | Assessment has not been published yet |
| `QuestionNotFound` | 20 | No question found with the given ID |
| `InvalidQuestionType` | 21 | Question type is invalid for this operation |
| `InvalidAnswer` | 22 | Submitted answer format is invalid |
| `MaxAttemptsReached` | 23 | Student has used all allowed attempts |
| `AssessmentClosed` | 24 | Assessment is closed or student's time limit was exceeded |
| `SubmissionNotFound` | 25 | No submission found with the given ID |
| `SubmissionAlreadyFinalized` | 26 | Submission has already been graded and cannot be modified |
| `AdaptiveNotEnabled` | 30 | Assessment does not have adaptive mode enabled |
| `AccommodationNotFound` | 31 | No accommodation config found for the student |
| `SecurityIntegrationMissing` | 40 | Caller is neither the admin nor the registered security monitor |

For cross-contract error conventions, see [ERROR_HANDLING.md](../../docs/ERROR_HANDLING.md).

## Integration

| Contract | Interaction |
|---|---|
| `analytics` | Submission and grading events are forwarded to the analytics contract |
| `progress` | Passing scores trigger progress updates in the student progress tracker |
| `security-monitor` | Integrity metadata is submitted by the security monitor contract acting as an oracle |
| `certificate` | Passing an assessment can trigger a certificate issuance request |
| `gamification` | Assessment completion events feed gamification activity records |
