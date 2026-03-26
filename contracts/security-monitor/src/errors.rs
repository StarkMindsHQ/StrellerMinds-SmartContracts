use soroban_sdk::contracterror;

/// Re-export standardized errors for backward compatibility
pub use crate::standardized_errors::StandardError;

/// Security-specific errors that extend the standard error set
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SecurityError {
    // Configuration specific errors (9000-9099)
    InvalidThreshold = 9000,
    InvalidTimeWindow = 9001,

    // Threat detection specific errors (9100-9199)
    ThreatNotFound = 9100,
    InvalidThreatData = 9101,
    ThreatAlreadyExists = 9102,

    // Circuit breaker specific errors (9200-9299)
    CircuitBreakerOpen = 9200,
    CircuitBreakerNotFound = 9201,
    InvalidBreakerState = 9202,

    // Rate limiting specific errors (9300-9399)
    InvalidRateLimitConfig = 9300,

    // Event processing specific errors (9400-9499)
    EventReplayFailed = 9400,
    EventFilteringFailed = 9401,
    InsufficientEvents = 9402,

    // Metrics specific errors (9500-9599)
    MetricsNotFound = 9500,
    InvalidMetricsData = 9501,
    MetricsCalculationFailed = 9502,

    // Recommendation specific errors (9600-9699)
    RecommendationNotFound = 9600,
    InvalidRecommendation = 9601,

    // General specific errors (9700-9799)
    OperationFailed = 9700,
}

/// Error context for security operations
pub type SecurityErrorContext = crate::standardized_errors::ErrorContext;

/// Helper macro for security errors with context
#[macro_export]
macro_rules! security_error {
    ($error:expr, $operation:expr, $info:expr) => {
        $crate::standardized_errors::ErrorContext::new(
            $crate::standardized_errors::StandardError::from($error),
            $operation,
            "SecurityContract",
            $info,
        )
    };
}
