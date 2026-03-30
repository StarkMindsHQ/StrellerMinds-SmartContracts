# Diagnostics Contract

## Purpose

The Diagnostics contract is the observability backbone of the StrellerMinds platform. It provides a comprehensive suite of on-chain monitoring, performance analysis, and automated diagnostics tools for all other contracts in the workspace. Key capabilities include real-time performance metric collection, predictive capacity planning, user behavior analysis, automated optimization recommendations, distributed cross-contract tracing with span aggregation, performance benchmarking, statistical anomaly detection, resource utilization analysis with cost-benefit modeling, and continuous regression testing. All analysis is contract-address-scoped, making it possible to track and compare any contract in the ecosystem.

## Architecture

| Module | Description |
|---|---|
| `lib.rs` | Contract entry point — 25 public functions delegating to specialist engines |
| `performance_monitor.rs` | `PerformanceMonitor` — starts/stops monitoring sessions, records and retrieves metrics |
| `predictive_engine.rs` | `PredictiveEngine` — generates capacity predictions from historical metric trends |
| `behavior_analyzer.rs` | `BehaviorAnalyzer` — analyzes user interaction patterns over configurable time windows |
| `optimization_engine.rs` | `OptimizationEngine` — generates automated optimization recommendations per contract |
| `distributed_tracer.rs` | `DistributedTracer` — manages multi-span distributed traces across contract boundaries |
| `benchmark_engine.rs` | `BenchmarkEngine` — executes configurable performance benchmark suites |
| `anomaly_detector.rs` | `AnomalyDetector` — detects statistical anomalies and surfaces trend analysis |
| `resource_optimizer.rs` | `ResourceOptimizer` — analyzes resource utilization, generates recommendations, models cost-benefit |
| `regression_tester.rs` | `RegressionTester` — runs regression test suites, sets up continuous monitoring sessions, generates reports |
| `storage.rs` | `DiagnosticsStorage` — typed storage access for admin, config, metrics, and monitored contract registry |
| `types.rs` | Core types: `DiagnosticsConfig`, `MonitoringConfig`, `PerformanceMetrics`, `CapacityPrediction`, `BehaviorAnalysis`, `OptimizationRecommendation`, `TraceSpan`, `TraceAnalysis`, `BenchmarkConfig`, `BenchmarkResult`, `AnomalyEvent`, `AnomalyTrends`, `ResourceUtilization`, `CostBenefitAnalysis`, `RegressionTestConfig`, `RegressionTestResult`, `SystemHealthReport` |
| `events.rs` | `DiagnosticsEvents` — emits initialization and monitoring lifecycle events |
| `errors.rs` | `DiagnosticsError` — 40+ typed error variants organized by subsystem category |

## Public API

| Function | Description | Auth Required |
|---|---|---|
| `initialize(admin, config)` | One-time setup; sets admin and diagnostics configuration | Admin |
| `start_performance_monitoring(contract_address, monitoring_config)` | Starts a real-time monitoring session for a contract; returns session ID | Admin |
| `stop_performance_monitoring(contract_address)` | Stops the active monitoring session for a contract | Admin |
| `record_performance_metrics(contract_address, metrics)` | Records a metrics snapshot for a monitored contract | None |
| `get_performance_metrics(contract_address)` | Returns the current performance metrics for a contract | None |
| `generate_capacity_prediction(contract_address, prediction_horizon)` | Generates a capacity prediction for the given future time horizon (seconds) | None |
| `analyze_user_behavior(user, analysis_period)` | Analyzes user behavior patterns over the specified historical period | None |
| `generate_opt_recommendations(contract_address)` | Generates automated optimization recommendations for a contract | None |
| `start_distributed_trace(trace_name, contract_address)` | Starts a distributed trace; returns the trace ID | None |
| `add_trace_span(trace_id, span)` | Adds a span to an existing distributed trace | None |
| `complete_trace(trace_id)` | Completes a trace and returns the analysis | None |
| `run_benchmark(benchmark_config)` | Runs a performance benchmark suite | Admin |
| `detect_anomalies(contract_address, detection_period)` | Detects anomalies within the given period (seconds) | None |
| `analyze_resource_utilization(contract_address, analysis_period)` | Analyzes resource usage and costs for a contract | None |
| `generate_resource_opt_recs(contract_address, resource_data)` | Generates resource optimization recommendations from utilization data | None |
| `monitor_optimization_progress(contract_address, recommendation_id, baseline_metrics)` | Tracks implementation progress of a recommendation | None |
| `run_regression_tests(contract_address, test_configuration)` | Runs a full regression test suite against a contract | Admin |
| `setup_continuous_monitoring(contract_address, monitoring_config)` | Configures a continuous monitoring session | Admin |
| `check_real_time_performance(contract_address, session_id, current_metrics)` | Checks current metrics against baselines; returns performance alerts | None |
| `generate_regression_report(contract_address, time_period)` | Generates a comprehensive regression report for a time window | Admin |
| `get_anomaly_trends(contract_address, period)` | Returns anomaly trends and statistics for a period | None |
| `analyze_opt_cost_benefit(recommendation)` | Performs cost-benefit analysis for an optimization recommendation | None |
| `run_regression_test(test_config)` | Runs a regression test using the first contract in the config | Admin |
| `get_system_health_report()` | Returns an aggregate health report across all monitored contracts | None |
| `run_diagnostic(contract_address)` | Returns a comprehensive diagnostic report (performance, anomalies, recommendations) | None |

## Usage Example

```
# 1. Admin initializes with a full configuration
diagnostics.initialize(admin, {
    enable_performance_monitoring: true,
    enable_predictions: true,
    enable_behavior_analysis: true,
    enable_tracing: true,
    enable_benchmarking: true,
    enable_anomaly_detection: true,
    enable_regression_testing: true,
    max_prediction_horizon: 2592000
})

# 2. Start monitoring a specific contract
session_id = diagnostics.start_performance_monitoring(certificate_contract, {
    sample_interval_seconds: 60,
    alert_thresholds: {...}
})

# 3. Record metrics periodically (called by the monitored contract or off-chain)
diagnostics.record_performance_metrics(certificate_contract, {
    transaction_count: 150,
    average_execution_time: 250,
    error_rate: 1,
    ...
})

# 4. Run diagnostics and get optimization recommendations
report = diagnostics.run_diagnostic(certificate_contract)
recommendations = diagnostics.generate_opt_recommendations(certificate_contract)

# 5. Set up distributed tracing across contracts
trace_id = diagnostics.start_distributed_trace("CertIssuance", certificate_contract)
diagnostics.add_trace_span(trace_id, {contract: assessment_contract, operation: "grade", ...})
analysis = diagnostics.complete_trace(trace_id)

# 6. Get system-wide health
health = diagnostics.get_system_health_report()
```

## Errors

| Error | Code | Description |
|---|---|---|
| `AdminNotSet` | 1001 | Admin address not set during initialization |
| `ConfigNotSet` | 1002 | Diagnostics configuration not set |
| `InvalidConfig` | 1003 | Configuration values are invalid |
| `Unauthorized` | 1004 | Caller is not authorized |
| `MonitoringDisabled` | 1101 | Performance monitoring is disabled |
| `InvalidMetrics` | 1102 | Supplied metrics data failed validation |
| `MetricsNotFound` | 1103 | No metrics found for the specified contract |
| `DataCorrupt` | 1105 | Stored metrics data is corrupted |
| `InvalidDetectionPeriod` | 1106 | Anomaly detection period is out of range |
| `PredictionDisabled` | 1201 | Predictive capacity analysis is disabled |
| `InsufficientData` | 1202 | Not enough historical data for prediction |
| `InvalidPredictionHorizon` | 1204 | Prediction horizon exceeds allowed maximum |
| `BehaviorDisabled` | 1301 | Behavior analysis is disabled |
| `UserDataNotFound` | 1304 | No behavior data found for user |
| `OptimizationError` | 1401 | Error generating optimization recommendations |
| `TracingDisabled` | 1501 | Distributed tracing is disabled |
| `TraceNotFound` | 1503 | No trace found for the specified ID |
| `TraceAlreadyCompleted` | 1505 | Trace has already been completed |
| `BenchmarkDisabled` | 1601 | Benchmarking is disabled |
| `BenchmarkFailed` | 1603 | Benchmark run failed to complete |
| `AnomalyDisabled` | 1701 | Anomaly detection is disabled |
| `ResourceError` | 1801 | Resource utilization analysis error |
| `RegressionDisabled` | 1901 | Regression testing is disabled |
| `BaselineNotFound` | 1903 | No baseline metrics found for the contract |
| `InvalidRegressionConfig` | 1905 | Regression test configuration is missing required fields |
| `StorageError` | 2101 | Storage read/write operation failed |
| `InternalError` | 2301 | Unexpected internal error |
| `InvalidInput` | 2302 | Provided input value is invalid or out of range |

For cross-contract error conventions, see [ERROR_HANDLING.md](../../docs/ERROR_HANDLING.md).

## Integration

| Contract | Interaction |
|---|---|
| `analytics` | Diagnostics data can be correlated with learning analytics for platform-wide insights |
| `security-monitor` | Anomaly events from diagnostics feed into the security monitor's threat assessment |
| `proxy` | The proxy contract upgrade lifecycle can be tracked via distributed traces |
| All contracts | Any contract in the workspace can be monitored, traced, and benchmarked by this contract |
