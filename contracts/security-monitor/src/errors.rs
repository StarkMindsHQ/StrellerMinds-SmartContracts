use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SecurityError {
    // Initialization errors
    AlreadyInitialized = 1,
    NotInitialized = 2,

    // Authorization errors
    Unauthorized = 3,
    PermissionDenied = 4,

    // Configuration errors
    InvalidConfiguration = 5,
    InvalidThreshold = 6,
    InvalidTimeWindow = 7,

    // Threat detection errors
    ThreatNotFound = 10,
    InvalidThreatData = 11,
    ThreatAlreadyExists = 12,

    // Circuit breaker errors
    CircuitBreakerOpen = 20,
    CircuitBreakerNotFound = 21,
    InvalidBreakerState = 22,

    // Rate limiting errors
    RateLimitExceeded = 30,
    InvalidRateLimitConfig = 31,

    // Event processing errors
    EventReplayFailed = 40,
    EventFilteringFailed = 41,
    InsufficientEvents = 42,

    // Metrics errors
    MetricsNotFound = 50,
    InvalidMetricsData = 51,
    MetricsCalculationFailed = 52,

    // Recommendation errors
    RecommendationNotFound = 60,
    InvalidRecommendation = 61,

    // Storage errors
    StorageError = 70,
    DataNotFound = 71,

    // General errors
    InvalidInput = 80,
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
