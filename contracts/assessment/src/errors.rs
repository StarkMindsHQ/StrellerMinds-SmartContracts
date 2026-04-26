use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AssessmentError {
    // Initialization / auth
    /// Contract has already been initialized.
    AlreadyInitialized = 1,
    /// Contract has not been initialized yet.
    NotInitialized = 2,
    /// Caller is not authorized to perform this operation.
    Unauthorized = 3,

    // Configuration / scheduling
    /// The provided assessment configuration contains invalid values.
    InvalidConfig = 10,
    /// The schedule is invalid, for example `end_time` is not after `start_time`.
    InvalidSchedule = 11,
    /// No assessment was found with the given ID.
    AssessmentNotFound = 12,
    /// The assessment has not been published and is not yet available to students.
    AssessmentNotPublished = 13,

    // Question / submission
    /// No question was found with the given ID.
    QuestionNotFound = 20,
    /// The question type is invalid or not supported for this operation.
    InvalidQuestionType = 21,
    /// The submitted answer format is invalid.
    InvalidAnswer = 22,
    /// The student has used all allowed attempts for this assessment.
    MaxAttemptsReached = 23,
    /// The assessment is currently closed or the student's time limit has been exceeded.
    AssessmentClosed = 24,
    /// No submission was found with the given ID.
    SubmissionNotFound = 25,
    /// The submission has already been finalized and cannot be modified.
    SubmissionAlreadyFinalized = 26,

    // Adaptive / accessibility
    /// The assessment does not have adaptive testing mode enabled.
    AdaptiveNotEnabled = 30,
    /// No accommodation configuration was found for the specified student.
    AccommodationNotFound = 31,

    // Integrity / security
    /// The caller is not the admin or a registered security monitor contract.
    SecurityIntegrationMissing = 40,

    // Rate limiting
    RateLimitExceeded = 50,
}
