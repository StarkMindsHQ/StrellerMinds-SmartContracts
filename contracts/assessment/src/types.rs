#![allow(clippy::too_many_arguments)]

use soroban_sdk::{contracttype, Address, BytesN, String, Symbol, Vec};

/// Supported question types for assessments.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
#[repr(u32)]
pub enum QuestionType {
    SingleChoice,
    MultipleChoice,
    Numeric,
    ShortText,
    Essay,
    Code,
}

/// Option for choice-based questions.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct QuestionOption {
    pub id: u32,
    pub label: String,
}

/// Canonical answer key representation for automated grading.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AnswerKey {
    /// Single correct option id.
    SingleChoice(u32),
    /// Set of correct option ids (order-insensitive).
    MultipleChoice(Vec<u32>),
    /// Numeric range (min, max).
    NumericRange(i64, i64),
    /// Whitelist of acceptable lowercased answers.
    ShortText(Vec<String>),
    /// No automatic grading; requires manual review.
    Manual,
}

/// Core question definition stored on-chain (content is referenced by hash).
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Question {
    pub question_id: u64,
    pub assessment_id: u64,
    pub question_type: QuestionType,
    pub max_score: u32,
    pub difficulty: u32, // 1-5 difficulty band
    pub content_hash: BytesN<32>,
    pub options: Vec<QuestionOption>,
    pub answer_key: AnswerKey,
}

/// Assessment-level configuration for timing, attempts, and integrity.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AssessmentConfig {
    pub time_limit_seconds: u64,
    pub max_attempts: u32,
    pub pass_score: u32,
    pub allow_review: bool,
    pub is_adaptive: bool,
    pub proctoring_required: bool,
}

/// Accessibility and accommodation configuration per student.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AccommodationConfig {
    /// Extra time percentage (e.g. 25 => +25% time limit).
    pub extra_time_percent: u32,
    /// Additional attempt allowance.
    pub extra_attempts: u32,
    /// Indicates that alternative formats are provisioned off-chain.
    pub alt_format_provided: bool,
}

/// High-level assessment metadata.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AssessmentMetadata {
    pub assessment_id: u64,
    pub course_id: Symbol,
    pub module_id: Symbol,
    pub instructor: Address,
    pub config: AssessmentConfig,
    pub published: bool,
}

/// Scheduling and proctoring configuration.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ScheduleConfig {
    pub assessment_id: u64,
    pub start_time: u64,
    pub end_time: u64,
    pub time_zone_offset_minutes: i32,
    pub proctoring_provider: Option<Symbol>,
}

/// Representation of a submitted answer.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum SubmittedAnswerValue {
    SingleChoice(u32),
    MultipleChoice(Vec<u32>),
    Numeric(i64),
    ShortText(String),
    Essay(String),
    Code(String),
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SubmittedAnswer {
    pub question_id: u64,
    pub value: SubmittedAnswerValue,
}

/// Submission status lifecycle.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
#[repr(u32)]
pub enum SubmissionStatus {
    InProgress,
    AutoGraded,
    RequiresManualReview,
    Finalized,
}

/// Security and integrity markers attached to a submission.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct IntegrityMetadata {
    pub plagiarism_score: u32, // 0-100; higher means more similar; 0 = not checked
    pub plagiarism_flag: bool,
    pub integrity_flags: Vec<Symbol>, // e.g. "MULTI_DEVICE", "TAB_SWITCH"
    pub has_proctoring_evidence: bool,
    pub proctoring_evidence_hash: BytesN<32>,
}

/// Full submission record including grading and integrity.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Submission {
    pub submission_id: BytesN<32>,
    pub assessment_id: u64,
    pub student: Address,
    pub attempt: u32,
    pub started_at: u64,
    pub submitted_at: u64,
    pub score: u32,
    pub max_score: u32,
    pub passed: bool,
    pub status: SubmissionStatus,
    pub answers: Vec<SubmittedAnswer>,
    pub integrity: IntegrityMetadata,
}

/// Per-student adaptive testing state.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AdaptiveState {
    pub current_difficulty: u32,
    pub completed_questions: Vec<u64>,
}

/// Cross-contract integration config for analytics, progress, and security.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct IntegrationConfig {
    pub analytics_contract: Option<Address>,
    pub progress_contract: Option<Address>,
    pub security_monitor_contract: Option<Address>,
}

/// Storage keys for the assessment contract.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DataKey {
    Admin,
    Integration,
    AssessmentCounter,
    QuestionCounter,
    Assessment(u64),
    AssessmentQuestions(u64), // assessment_id -> Vec<u64>
    Question(u64),
    Submission(BytesN<32>),
    StudentAssessmentSubmissions(Address, u64), // (student, assessment_id)
    Schedule(u64),
    Accommodation(Address),
    Adaptive(Address, u64), // (student, assessment_id)
}
