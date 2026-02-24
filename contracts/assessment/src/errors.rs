use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AssessmentError {
    // Initialization / auth
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,

    // Configuration / scheduling
    InvalidConfig = 10,
    InvalidSchedule = 11,
    AssessmentNotFound = 12,
    AssessmentNotPublished = 13,

    // Question / submission
    QuestionNotFound = 20,
    InvalidQuestionType = 21,
    InvalidAnswer = 22,
    MaxAttemptsReached = 23,
    AssessmentClosed = 24,
    SubmissionNotFound = 25,
    SubmissionAlreadyFinalized = 26,

    // Adaptive / accessibility
    AdaptiveNotEnabled = 30,
    AccommodationNotFound = 31,

    // Integrity / security
    SecurityIntegrationMissing = 40,
}

