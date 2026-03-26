use soroban_sdk::contracterror;

/// Re-export standardized errors for backward compatibility
pub use crate::standardized_errors::StandardError;

/// Assessment-specific errors that extend the standard error set
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AssessmentError {
    // Configuration specific errors (5000-5099)
    InvalidSchedule = 5000,
    AssessmentNotPublished = 5001,

    // Question specific errors (5100-5199)
    QuestionNotFound = 5100,
    InvalidQuestionType = 5101,
    InvalidAnswer = 5102,
    MaxAttemptsReached = 5103,
    AssessmentClosed = 5104,
    SubmissionNotFound = 5105,
    SubmissionAlreadyFinalized = 5106,

    // Adaptive specific errors (5200-5299)
    AdaptiveNotEnabled = 5200,
    AccommodationNotFound = 5201,

    // Security specific errors (5300-5399)
    SecurityIntegrationMissing = 5300,
}

/// Error context for assessment operations
pub type AssessmentErrorContext = crate::standardized_errors::ErrorContext;

/// Helper macro for assessment errors with context
#[macro_export]
macro_rules! assessment_error {
    ($error:expr, $operation:expr, $info:expr) => {
        $crate::standardized_errors::ErrorContext::new(
            $crate::standardized_errors::StandardError::from($error),
            $operation,
            "AssessmentContract",
            $info,
        )
    };
}
