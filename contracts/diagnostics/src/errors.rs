use soroban_sdk::contracterror;

/// Error types for the diagnostics platform
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum DiagnosticsError {
    // Configuration (1000-1099)
    /// Admin address has not been set during initialization.
    AdminNotSet = 1001,
    /// Diagnostics configuration has not been set.
    ConfigNotSet = 1002,
    /// The provided configuration values are invalid.
    InvalidConfig = 1003,
    /// Caller is not authorized to perform this operation.
    Unauthorized = 1004,
    /// One or more configuration fields contain invalid values.
    InvalidConfiguration = 1005,

    // Monitoring (1100-1199)
    /// Performance monitoring is disabled in the current configuration.
    MonitoringDisabled = 1101,
    /// The supplied metrics data failed validation.
    InvalidMetrics = 1102,
    /// No metrics record found for the specified contract.
    MetricsNotFound = 1103,
    /// Stored metrics data is corrupted or unreadable.
    DataCorrupt = 1105,
    /// The requested anomaly detection period is out of the allowed range.
    InvalidDetectionPeriod = 1106,

    // Prediction (1200-1299)
    /// Predictive capacity analysis is disabled in the current configuration.
    PredictionDisabled = 1201,
    /// Not enough historical data to produce a prediction.
    InsufficientData = 1202,
    /// The prediction model encountered an internal error.
    ModelError = 1203,
    /// The requested prediction horizon exceeds the allowed maximum.
    InvalidPredictionHorizon = 1204,
    /// Insufficient historical data points to generate a capacity prediction.
    InsufficientDataForPrediction = 1205,

    // Behavior (1300-1399)
    /// Behavior analysis is disabled in the current configuration.
    BehaviorDisabled = 1301,
    /// No behavior data found for the specified user.
    UserDataNotFound = 1304,
    /// The requested analysis period is invalid or out of range.
    InvalidAnalysisPeriod = 1305,
    /// Not enough behavior events recorded to complete the analysis.
    InsufficientBehaviorData = 1306,

    // Optimization (1400-1499)
    /// An error occurred while generating optimization recommendations.
    OptimizationError = 1401,
    /// No optimization opportunities were found for the contract.
    NoOptimizations = 1402,

    // Tracing (1500-1599)
    /// Distributed tracing is disabled in the current configuration.
    TracingDisabled = 1501,
    /// No trace found for the specified trace ID.
    TraceNotFound = 1503,
    /// The provided trace span data is invalid.
    InvalidTraceSpan = 1504,
    /// The trace has already been completed and cannot be modified.
    TraceAlreadyCompleted = 1505,

    // Benchmark (1600-1699)
    /// Benchmarking is disabled in the current configuration.
    BenchmarkDisabled = 1601,
    /// The benchmark run failed to complete successfully.
    BenchmarkFailed = 1603,
    /// The provided benchmark configuration contains invalid values.
    InvalidBenchmarkConfig = 1604,

    // Anomaly (1700-1799)
    /// Anomaly detection is disabled in the current configuration.
    AnomalyDisabled = 1701,
    /// No anomalies were detected during the specified period.
    NoAnomalies = 1702,

    // Resource (1800-1899)
    /// An error occurred during resource utilization analysis.
    ResourceError = 1801,

    // Regression (1900-1999)
    /// Regression testing is disabled in the current configuration.
    RegressionDisabled = 1901,
    /// No baseline metrics found for the specified contract.
    BaselineNotFound = 1903,
    /// The provided test scenario is invalid or malformed.
    InvalidTestScenario = 1904,
    /// The regression test configuration is missing required fields or is invalid.
    InvalidRegressionConfig = 1905,

    // Storage (2100-2199)
    /// A storage read or write operation failed.
    StorageError = 2101,
    /// The requested data record was not found in storage.
    DataNotFound = 2102,

    // General (2300-2399)
    /// An unexpected internal error occurred.
    InternalError = 2301,
    /// The provided input value is invalid or out of range.
    InvalidInput = 2302,
}
