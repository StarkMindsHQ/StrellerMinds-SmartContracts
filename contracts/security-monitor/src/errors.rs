use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SecurityError {
    // Initialization errors
    /// The contract has already been initialized and cannot be re-initialized.
    AlreadyInitialized = 1,
    /// The contract has not been initialized yet.
    NotInitialized = 2,

    // Authorization errors
    /// Caller is not authorized to perform this operation.
    Unauthorized = 3,
    /// Caller lacks the required permission for the requested action.
    PermissionDenied = 4,

    // Configuration errors
    /// One or more configuration fields contain invalid values.
    InvalidConfiguration = 5,
    /// The provided alert or detection threshold is out of the allowed range.
    InvalidThreshold = 6,
    /// The provided time window value is invalid or out of range.
    InvalidTimeWindow = 7,

    // Threat detection errors
    /// No threat record found for the specified threat ID.
    ThreatNotFound = 10,
    /// The supplied threat data failed validation.
    InvalidThreatData = 11,
    /// A threat with the same identifier already exists.
    ThreatAlreadyExists = 12,

    // Circuit breaker errors
    /// The circuit breaker is open and rejecting requests to the protected contract.
    CircuitBreakerOpen = 20,
    /// No circuit breaker found for the specified contract identifier.
    CircuitBreakerNotFound = 21,
    /// The requested state transition for the circuit breaker is not allowed.
    InvalidBreakerState = 22,

    // Rate limiting errors
    /// The caller has exceeded the allowed request rate and must wait before retrying.
    RateLimitExceeded = 30,
    /// The provided rate-limit configuration contains invalid values.
    InvalidRateLimitConfig = 31,

    // Event processing errors
    /// Replaying a historical event sequence failed.
    EventReplayFailed = 40,
    /// Applying event filter criteria failed.
    EventFilteringFailed = 41,
    /// Not enough events available to complete the requested analysis.
    InsufficientEvents = 42,

    // Metrics errors
    /// No metrics record found for the specified target.
    MetricsNotFound = 50,
    /// The supplied metrics data failed validation.
    InvalidMetricsData = 51,
    /// An error occurred while computing derived metrics values.
    MetricsCalculationFailed = 52,

    // Recommendation errors
    /// No security recommendation found for the specified ID.
    RecommendationNotFound = 60,
    /// The provided security recommendation contains invalid data.
    InvalidRecommendation = 61,

    // Storage errors
    /// A storage read or write operation failed.
    StorageError = 70,
    /// The requested data record was not found in storage.
    DataNotFound = 71,

    // General errors
    /// The provided input value is invalid or out of range.
    InvalidInput = 80,
    /// The requested operation failed to complete.
    OperationFailed = 81,
}

impl SecurityError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::AlreadyInitialized => "SEC-001",
            Self::NotInitialized => "SEC-002",
            Self::Unauthorized => "SEC-003",
            Self::PermissionDenied => "SEC-004",
            Self::InvalidConfiguration => "SEC-005",
            Self::InvalidThreshold => "SEC-006",
            Self::InvalidTimeWindow => "SEC-007",
            Self::ThreatNotFound => "SEC-010",
            Self::InvalidThreatData => "SEC-011",
            Self::ThreatAlreadyExists => "SEC-012",
            Self::CircuitBreakerOpen => "SEC-020",
            Self::CircuitBreakerNotFound => "SEC-021",
            Self::InvalidBreakerState => "SEC-022",
            Self::RateLimitExceeded => "SEC-030",
            Self::InvalidRateLimitConfig => "SEC-031",
            Self::EventReplayFailed => "SEC-040",
            Self::EventFilteringFailed => "SEC-041",
            Self::InsufficientEvents => "SEC-042",
            Self::MetricsNotFound => "SEC-050",
            Self::InvalidMetricsData => "SEC-051",
            Self::MetricsCalculationFailed => "SEC-052",
            Self::RecommendationNotFound => "SEC-060",
            Self::InvalidRecommendation => "SEC-061",
            Self::StorageError => "SEC-070",
            Self::DataNotFound => "SEC-071",
            Self::InvalidInput => "SEC-080",
            Self::OperationFailed => "SEC-081",
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            Self::AlreadyInitialized => "Security monitor is already initialized",
            Self::NotInitialized => "Security monitor is not initialized",
            Self::Unauthorized => "Caller is not authorized for this security operation",
            Self::PermissionDenied => "Permission was denied for this security action",
            Self::InvalidConfiguration => "Security configuration is invalid",
            Self::InvalidThreshold => "Security threshold value is invalid",
            Self::InvalidTimeWindow => "Security time window is invalid",
            Self::ThreatNotFound => "Threat record was not found",
            Self::InvalidThreatData => "Threat payload is invalid",
            Self::ThreatAlreadyExists => "Threat record already exists",
            Self::CircuitBreakerOpen => "Circuit breaker is open",
            Self::CircuitBreakerNotFound => "Circuit breaker was not found",
            Self::InvalidBreakerState => "Circuit breaker state is invalid",
            Self::RateLimitExceeded => "Rate limit has been exceeded",
            Self::InvalidRateLimitConfig => "Rate limit configuration is invalid",
            Self::EventReplayFailed => "Event replay failed",
            Self::EventFilteringFailed => "Event filtering failed",
            Self::InsufficientEvents => {
                "Not enough events were available to complete the operation"
            }
            Self::MetricsNotFound => "Requested metrics were not found",
            Self::InvalidMetricsData => "Metrics data is invalid",
            Self::MetricsCalculationFailed => "Metrics calculation failed",
            Self::RecommendationNotFound => "Recommendation was not found",
            Self::InvalidRecommendation => "Recommendation payload is invalid",
            Self::StorageError => "Security monitor storage operation failed",
            Self::DataNotFound => "Requested security data was not found",
            Self::InvalidInput => "Input payload is invalid",
            Self::OperationFailed => "Security operation failed",
        }
    }
}
