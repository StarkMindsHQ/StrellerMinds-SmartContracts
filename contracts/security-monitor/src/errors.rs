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
