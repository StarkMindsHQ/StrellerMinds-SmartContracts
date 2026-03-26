use soroban_sdk::contracterror;

/// Re-export standardized errors for backward compatibility
pub use crate::standardized_errors::StandardError;

/// Analytics-specific errors that extend the standard error set
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AnalyticsError {
    // Data validation specific errors (6000-6099)
    InvalidSessionData = 6000,
    InvalidTimeRange = 6001,
    InvalidScore = 6002,
    InvalidPercentage = 6003,
    SessionTooShort = 6004,
    SessionTooLong = 6005,

    // Data not found specific errors (6100-6199)
    StudentNotFound = 6100,
    CourseNotFound = 6101,
    ModuleNotFound = 6102,
    ReportNotFound = 6103,

    // Business logic specific errors (6200-6299)
    SessionAlreadyExists = 6200,
    SessionNotCompleted = 6201,
    InsufficientData = 6202,
    InvalidBatchSize = 6203,

    // Configuration specific errors (6300-6399)
    UnauthorizedOracle = 6300,
    InvalidInsightData = 6301,
    InsightNotFound = 6302,
}

/// Error context for analytics operations
pub type AnalyticsErrorContext = crate::standardized_errors::ErrorContext;

/// Helper macro for analytics errors with context
#[macro_export]
macro_rules! analytics_error {
    ($error:expr, $operation:expr, $info:expr) => {
        $crate::standardized_errors::ErrorContext::new(
            $crate::standardized_errors::StandardError::from($error),
            $operation,
            "AnalyticsContract",
            $info,
        )
    };
}
