use soroban_sdk::contracterror;

/// Analytics contract errors
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AnalyticsError {
    /// Contract has already been initialized; `initialize` may only be called once.
    AlreadyInitialized = 1,
    /// Contract has not been initialized; call `initialize` first.
    NotInitialized = 2,
    /// Caller does not have the required admin privileges.
    Unauthorized = 3,
    /// Provided session data is malformed or missing required fields.
    InvalidSessionData = 4,
    /// Provided time range is invalid or out of order.
    InvalidTimeRange = 5,
    /// Score value is outside the valid range.
    InvalidScore = 6,
    /// Percentage value is outside the 0–100 range.
    InvalidPercentage = 7,
    /// Session duration is below the minimum allowed threshold.
    SessionTooShort = 8,
    /// Session duration exceeds the maximum allowed threshold.
    SessionTooLong = 9,
    /// No session was found with the given ID.
    SessionNotFound = 10,
    /// No student record was found for the given address.
    StudentNotFound = 11,
    /// No course was found with the given ID.
    CourseNotFound = 12,
    /// No module was found with the given ID.
    ModuleNotFound = 13,
    /// No report was found with the given ID.
    ReportNotFound = 14,
    /// A session with this ID already exists; use a unique session ID.
    SessionAlreadyExists = 15,
    /// Operation requires the session to be completed first.
    SessionNotCompleted = 16,
    /// Not enough data is available to perform the requested analytics operation.
    InsufficientData = 17,
    /// Batch size is zero or exceeds the maximum allowed value.
    InvalidBatchSize = 18,
    /// An internal storage read or write operation failed.
    StorageError = 19,
    /// Contract configuration is missing required values or contains invalid settings.
    InvalidConfiguration = 20,
    /// Oracle address is not registered as a trusted source.
    UnauthorizedOracle = 21,
    /// Provided insight data is malformed or missing required fields.
    InvalidInsightData = 22,
    /// No insight record was found with the given ID.
    InsightNotFound = 23,
    /// Timestamp is not a valid UTC Unix epoch second (Issue #442: DST fix).
    InvalidTimestamp = 24,
}
