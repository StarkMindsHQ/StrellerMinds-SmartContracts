use soroban_sdk::contracterror;

/// Error types for the diagnostics platform
#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum DiagnosticsError {
    // Configuration (1000-1099)
    AdminNotSet = 1001,
    ConfigNotSet = 1002,
    InvalidConfig = 1003,
    Unauthorized = 1004,
    InvalidConfiguration = 1005,

    // Monitoring (1100-1199)
    MonitoringDisabled = 1101,
    InvalidMetrics = 1102,
    MetricsNotFound = 1103,
    DataCorrupt = 1105,
    InvalidDetectionPeriod = 1106,

    // Prediction (1200-1299)
    PredictionDisabled = 1201,
    InsufficientData = 1202,
    ModelError = 1203,
    InvalidPredictionHorizon = 1204,
    InsufficientDataForPrediction = 1205,

    // Behavior (1300-1399)
    BehaviorDisabled = 1301,
    UserDataNotFound = 1304,
    InvalidAnalysisPeriod = 1305,
    InsufficientBehaviorData = 1306,

    // Optimization (1400-1499)
    OptimizationError = 1401,
    NoOptimizations = 1402,

    // Tracing (1500-1599)
    TracingDisabled = 1501,
    TraceNotFound = 1503,
    InvalidTraceSpan = 1504,
    TraceAlreadyCompleted = 1505,

    // Benchmark (1600-1699)
    BenchmarkDisabled = 1601,
    BenchmarkFailed = 1603,
    InvalidBenchmarkConfig = 1604,

    // Anomaly (1700-1799)
    AnomalyDisabled = 1701,
    NoAnomalies = 1702,

    // Resource (1800-1899)
    ResourceError = 1801,

    // Regression (1900-1999)
    RegressionDisabled = 1901,
    BaselineNotFound = 1903,
    InvalidTestScenario = 1904,
    InvalidRegressionConfig = 1905,

    // Storage (2100-2199)
    StorageError = 2101,
    DataNotFound = 2102,

    // General (2300-2399)
    InternalError = 2301,
    InvalidInput = 2302,
}