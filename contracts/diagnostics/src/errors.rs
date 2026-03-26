use soroban_sdk::contracterror;

/// Re-export standardized errors for backward compatibility
pub use crate::standardized_errors::StandardError;

/// Diagnostics-specific errors that extend the standard error set
#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DiagnosticsError {
    // Configuration specific errors (7000-7099)
    AdminNotSet = 7000,
    ConfigNotSet = 7001,
    InvalidDetectionPeriod = 7002,

    // Monitoring specific errors (7100-7199)
    MonitoringDisabled = 7100,
    InvalidMetrics = 7101,
    MetricsNotFound = 7102,
    DataCorrupt = 7103,

    // Prediction specific errors (7200-7299)
    PredictionDisabled = 7200,
    ModelError = 7201,
    InvalidPredictionHorizon = 7202,
    InsufficientDataForPrediction = 7203,

    // Behavior specific errors (7300-7399)
    BehaviorDisabled = 7300,
    UserDataNotFound = 7301,
    InvalidAnalysisPeriod = 7302,
    InsufficientBehaviorData = 7303,

    // Optimization specific errors (7400-7499)
    OptimizationError = 7400,
    NoOptimizations = 7401,

    // Tracing specific errors (7500-7599)
    TracingDisabled = 7500,
    TraceNotFound = 7501,
    InvalidTraceSpan = 7502,
    TraceAlreadyCompleted = 7503,

    // Benchmark specific errors (7600-7699)
    BenchmarkDisabled = 7600,
    BenchmarkFailed = 7601,
    InvalidBenchmarkConfig = 7602,

    // Anomaly specific errors (7700-7799)
    AnomalyDisabled = 7700,
    NoAnomalies = 7701,

    // Resource specific errors (7800-7899)
    ResourceError = 7800,

    // Regression specific errors (7900-7999)
    RegressionDisabled = 7900,
    BaselineNotFound = 7901,
    InvalidTestScenario = 7902,
    InvalidRegressionConfig = 7903,
}

/// Error context for diagnostics operations
pub type DiagnosticsErrorContext = crate::standardized_errors::ErrorContext;

/// Helper macro for diagnostics errors with context
#[macro_export]
macro_rules! diagnostics_error {
    ($error:expr, $operation:expr, $info:expr) => {
        $crate::standardized_errors::ErrorContext::new(
            $crate::standardized_errors::StandardError::from($error),
            $operation,
            "DiagnosticsContract",
            $info,
        )
    };
}
