# Assessment Contract

## Overview

The `assessment` contract provides a comprehensive, on-chain assessment system for educational platforms built on Soroban. It supports rich question types, automated grading, plagiarism and integrity metadata, adaptive testing, scheduling, accessibility accommodations, and integration points for progress tracking and analytics.

## Features

- **Multiple Question Types**: Single choice, multiple choice, numeric, short text, essay, and code questions via `QuestionType`.
- **Automated Grading**: Configurable answer keys (`AnswerKey`) with automatic scoring for objective questions and manual-review routing for essay/code responses.
- **Plagiarism & Integrity Metadata**: `IntegrityMetadata` tracks plagiarism scores, flags, integrity events, and proctoring evidence.
- **Adaptive Testing**: Simple adaptive algorithm that adjusts difficulty by tracking per-student `AdaptiveState`.
- **Scheduling & Proctoring**: Time-windowed assessments and optional proctoring provider via `ScheduleConfig`.
- **Accessibility & Accommodations**: Per-student `AccommodationConfig` that adjusts time limits and attempts.
- **Progress & Analytics Hooks**: Helper API to summarize assessment performance per course and student, designed to feed into the existing `progress` and `analytics` contracts.

## Core Data Structures

- `AssessmentMetadata`: High-level definition of an assessment (course, module, instructor, config, published state).
- `AssessmentConfig`: Time limit, attempts, passing score, adaptive and proctoring flags.
- `Question`: Question metadata (type, max score, difficulty band, off-chain content hash, options, answer key).
- `Submission`: Full submission record including answers, scores, status, and integrity metadata.
- `AdaptiveState`: Per-student adaptive difficulty and completed questions.
- `IntegrationConfig`: Addresses of `analytics`, `progress`, and `security-monitor` contracts (optional).

## Contract Interface (High-Level)

### Initialization & Integration

- `initialize(env, admin)`: One-time setup with admin.
- `set_integration(env, admin, analytics, progress, security_monitor)`: Configure integration addresses.

### Authoring Assessments

- `create_assessment(env, instructor, course_id, module_id, config) -> assessment_id`
- `publish_assessment(env, admin, assessment_id)`
- `add_question(env, admin, assessment_id, question_type, max_score, difficulty, content_hash, options, answer_key)`
- `get_assessment_metadata(env, assessment_id) -> Option<AssessmentMetadata>`

### Scheduling & Accessibility

- `set_schedule(env, admin, assessment_id, start_time, end_time, time_zone_offset_minutes, proctoring_provider)`
- `set_accommodation(env, admin, student, config)`
- `get_accommodation_for_student(env, student) -> Option<AccommodationConfig>`

### Adaptive Testing

- `get_next_question(env, student, assessment_id) -> Result<Option<Question>, AssessmentError>`
- `update_adaptive_state(env, student, assessment_id, question_id, was_correct)`

### Submissions & Grading

- `start_submission(env, student, assessment_id) -> submission_id`
- `submit_answers(env, student, submission_id, answers) -> Submission`
- `get_submission_details(env, submission_id) -> Option<Submission>`

Automated grading is delegated to `GradingEngine::grade_submission`, which:

- Scores single/multiple-choice, numeric-range, and short-text questions using the stored `AnswerKey`.
- Marks essay and code questions as `RequiresManualReview`.
- Sets `SubmissionStatus` to `AutoGraded` or `RequiresManualReview` based on whether any manual grading is needed.

### Integrity & Plagiarism

- `update_integrity_metadata(env, oracle_or_admin, submission_id, plagiarism_score, plagiarism_flag, integrity_flags, proctoring_evidence_hash)`

This function is restricted to the configured `security-monitor` contract or the assessment admin. It updates plagiarism metrics and emits `plagiarism` and `integrity` events, which can be correlated with the existing `security-monitor` contract.

### Progress & Analytics Helper

- `get_course_assessment_progress(env, student, course_id) -> Map<u64, (u32, u32, bool)>`

Returns the latest `(score, max_score, passed)` per `assessment_id` for a given course and student. This is intended to:

- Feed **course-level progress** into the `progress` and `student-progress-tracker` contracts.
- Provide structured input to the `analytics` contract (e.g. as part of learning sessions of type `SessionType::Assessment`).

## Usage Scenarios

- **Objective Quiz**: Use `SingleChoice`, `MultipleChoice`, `Numeric`, or `ShortText` questions with appropriate `AnswerKey` variants, then call `submit_answers` for automated grading.
- **Essay / Code Assignments**: Use `Essay` or `Code` with `AnswerKey::Manual`; submissions are flagged for manual review.
- **Timed Exams**: Configure `AssessmentConfig.time_limit_seconds` and `set_schedule` with an exam window.
- **Proctored Exams**: Set `proctoring_required = true` and supply a `proctoring_provider` symbol; store off-chain proctoring evidence and reference it via `update_integrity_metadata`.
- **Accessibility Accommodations**: Configure `AccommodationConfig` for eligible students to provide extra time and attempts.
- **Adaptive Practice Tests**: Set `is_adaptive = true` and call `get_next_question` / `update_adaptive_state` between questions to adjust difficulty.

## Testing

The `src/test.rs` module includes scenarios for:

- Creating and publishing assessments.
- Adding questions and performing single-choice automated grading.
- Basic adaptive state updates.

Additional tests can be added to cover:

- Multi-format grading (multiple choice, numeric range, short text).
- Integrity metadata updates and authorization rules.
- Scheduling and time-limit enforcement.
- Progress helper behaviour across multiple assessments.

> Note: Running tests on Windows requires a Rust MSVC toolchain with the Visual C++ build tools installed so that `link.exe` is available.

