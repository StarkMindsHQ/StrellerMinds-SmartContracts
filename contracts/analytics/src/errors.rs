use soroban_sdk::contracterror;

/// Analytics contract errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AnalyticsError {
    // Initialization errors
    AlreadyInitialized = 1,
    NotInitialized = 2,

    // Authorization errors
    Unauthorized = 3,

    // Data validation errors
    InvalidSessionData = 4,
    InvalidTimeRange = 5,
    InvalidScore = 6,
    InvalidPercentage = 7,
    SessionTooShort = 8,
    SessionTooLong = 9,

    // Data not found errors
    SessionNotFound = 10,
    StudentNotFound = 11,
    CourseNotFound = 12,
    ModuleNotFound = 13,
    ReportNotFound = 14,

    // Business logic errors
    SessionAlreadyExists = 15,
    SessionNotCompleted = 16,
    InsufficientData = 17,
    InvalidBatchSize = 18,

    // Storage errors
    StorageError = 19,

    // Configuration errors
    InvalidConfiguration = 20,
    UnauthorizedOracle = 21,
    InvalidInsightData = 22,
    InsightNotFound = 23,
}
