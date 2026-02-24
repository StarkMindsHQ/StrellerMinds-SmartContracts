use soroban_sdk::{contracttype, Address, BytesN, Map, String, Symbol, Vec};

/// Configuration for the diagnostics platform
#[derive(Clone, Debug)]
#[contracttype]
pub struct DiagnosticsConfig {
    pub admin: Address,
    pub monitoring_enabled: bool,
    pub anomaly_detection_enabled: bool,
    pub prediction_enabled: bool,
    pub max_trace_duration: u64,
    pub metrics_retention_period: u64,
    pub alert_threshold_cpu: u32,
    pub alert_threshold_memory: u32,
    pub alert_threshold_gas: u64,
}

/// Configuration for performance monitoring
#[derive(Clone, Debug)]
#[contracttype]
pub struct MonitoringConfig {
    pub metrics_collection_interval: u64,
    pub enable_real_time_alerts: bool,
    pub enable_predictive_analysis: bool,
    pub enable_behavior_tracking: bool,
    pub max_metrics_history: u32,
}

/// Real-time performance metrics
#[derive(Clone, Debug)]
#[contracttype]
pub struct PerformanceMetrics {
    pub timestamp: u64,
    pub contract_address: Address,
    pub execution_time: u64, // microseconds
    pub gas_used: u64,
    pub memory_usage: u32, // bytes
    pub storage_reads: u32,
    pub storage_writes: u32,
    pub cpu_utilization: u32,  // percentage
    pub cpu_instructions: u64, // total CPU instructions executed
    pub transaction_count: u32,
    pub error_count: u32,
    pub error_rate: u32, // percentage
    pub average_execution_time: u64,
    pub average_response_time: u64,
    pub network_bandwidth: u32,
    pub gas_consumption: u64,
    pub storage_usage: u32,
    pub peak_memory_usage: u32,
    pub network_latency: u32, // milliseconds
}

/// Predictive capacity planning results
#[derive(Clone, Debug)]
#[contracttype]
pub struct CapacityPrediction {
    pub prediction_id: BytesN<32>,
    pub contract_address: Address,
    pub prediction_horizon: u64,
    pub predicted_load: LoadPrediction,
    pub capacity_recommendations: Vec<String>,
    pub bottleneck_predictions: Vec<BottleneckPrediction>,
    pub cost_projections: CostProjection,
    pub confidence_score: u32, // percentage
    pub generated_at: u64,
}

/// Load prediction details
#[derive(Clone, Debug)]
#[contracttype]
pub struct LoadPrediction {
    pub predicted_tx_per_hour: u32,
    pub predicted_gas_usage: u64,
    pub predicted_storage_growth: u32,
    pub peak_load_times: Vec<u64>,
    pub resource_saturation_risk: RiskLevel,
    pub predicted_tx_hourly: u32,
}

/// Predicted bottlenecks
#[derive(Clone, Debug)]
#[contracttype]
pub struct BottleneckPrediction {
    pub bottleneck_type: BottleneckType,
    pub severity: RiskLevel,
    pub estimated_impact: String,
    pub recommended_actions: Vec<String>,
    pub estimated_occurrence_time: u64,
}

/// Types of system bottlenecks
#[derive(Clone, Debug)]
#[contracttype]
pub enum BottleneckType {
    CPU,
    Memory,
    Storage,
    Network,
    Gas,
    Throughput,
}

/// Risk assessment levels
#[derive(Clone, Debug, PartialEq, PartialOrd)]
#[contracttype]
pub enum RiskLevel {
    Low,
    Medium,
    High,
    Critical,
}

/// Cost projection analysis
#[derive(Clone, Debug)]
#[contracttype]
pub struct CostProjection {
    pub current_daily_cost: u64,
    pub projected_daily_cost: u64,
    pub cost_optimization_potential: u64,
    pub cost_trend: CostTrend,
}

/// Cost trend analysis
#[derive(Clone, Debug)]
#[contracttype]
pub enum CostTrend {
    Decreasing,
    Stable,
    Increasing,
    Exponential,
}

/// User behavior analysis results
#[derive(Clone, Debug)]
#[contracttype]
pub struct BehaviorAnalysis {
    pub analysis_id: BytesN<32>,
    pub user: Address,
    pub analysis_period: u64,
    pub behavior_patterns: Vec<BehaviorPattern>,
    pub learning_effectiveness: LearningEffectiveness,
    pub engagement_metrics: EngagementMetrics,
    pub optimization_suggestions: Vec<String>,
    pub risk_indicators: Vec<RiskIndicator>,
}

/// Identified behavior patterns
#[derive(Clone, Debug)]
#[contracttype]
pub struct BehaviorPattern {
    pub pattern_type: PatternType,
    pub frequency: u32,
    pub confidence: u32,
    pub impact_on_learning: ImpactLevel,
    pub description: String,
}

/// Types of behavior patterns
#[derive(Clone, Debug)]
#[contracttype]
pub enum PatternType {
    LoginTiming,
    SessionDuration,
    ContentConsumption,
    InteractionFrequency,
    ProgressPacing,
    ErrorRecovery,
}

/// Impact assessment levels
#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub enum ImpactLevel {
    Positive,
    Neutral,
    Negative,
    Unknown,
}

/// Learning effectiveness metrics
#[derive(Clone, Debug)]
#[contracttype]
pub struct LearningEffectiveness {
    pub completion_rate: u32,
    pub knowledge_retention: u32,
    pub skill_acquisition: u32,
    pub engagement_score: u32,
    pub effectiveness_trend: EffectivenessTrend,
}

/// Effectiveness trend indicators
#[derive(Clone, Debug)]
#[contracttype]
pub enum EffectivenessTrend {
    Improving,
    Stable,
    Declining,
    Inconsistent,
}

/// User engagement metrics
#[derive(Clone, Debug)]
#[contracttype]
pub struct EngagementMetrics {
    pub daily_active_time: u32,
    pub session_frequency: u32,
    pub content_interaction_rate: u32,
    pub completion_velocity: u32,
    pub return_rate: u32,
}

/// Risk indicators for user behavior
#[derive(Clone, Debug)]
#[contracttype]
pub struct RiskIndicator {
    pub risk_type: RiskType,
    pub severity: RiskLevel,
    pub probability: u32,
    pub description: String,
    pub mitigation_suggestions: Vec<String>,
}

/// Types of behavioral risks
#[derive(Clone, Debug)]
#[contracttype]
pub enum RiskType {
    Dropout,
    LearningPlateau,
    Disengagement,
    SkillRegression,
    TechnicalDifficulties,
}

/// Automated optimization recommendations
#[derive(Clone, Debug)]
#[contracttype]
pub struct OptimizationRecommendation {
    pub recommendation_id: BytesN<32>,
    pub category: OptimizationCategory,
    pub priority: Priority,
    pub description: String,
    pub expected_impact: ImpactEstimate,
    pub implementation_complexity: Complexity,
    pub estimated_savings: SavingsEstimate,
    pub implementation_steps: Vec<String>,
    pub contract_address: Address,
    pub optimization_type: OptimizationType,
    pub title: String,
    pub expected_improvement: u64,
    pub estimated_effort: ImplementationEffort,
    pub cost_savings: u64,
    pub automated_fix_available: bool,
    pub monitoring_metrics: Vec<String>,
}

/// Categories of optimizations
#[derive(Clone, Debug)]
#[contracttype]
pub enum OptimizationCategory {
    Gas,
    Storage,
    Memory,
    Network,
    Algorithm,
    Architecture,
}

/// Optimization types for recommendations
#[derive(Clone, Debug)]
#[contracttype]
pub enum OptimizationType {
    Storage,
    Compute,
    Network,
    Memory,
    Gas,
    GasEfficiency,
    StorageEfficiency,
    MemoryEfficiency,
    NetworkEfficiency,
    AlgorithmEfficiency,
    ArchitectureImprovement,
}

/// Implementation effort levels
#[derive(Clone, Debug)]
#[contracttype]
pub enum ImplementationEffort {
    Low,
    Simple,
    Medium,
    Complex,
    High,
    VeryHigh,
    VeryComplex,
}

/// Implementation complexity (alias for backwards compatibility)
#[derive(Clone, Debug)]
#[contracttype]
pub enum ImplementationComplexity {
    Low,
    Medium,
    High,
}

/// Priority levels
#[derive(Clone, Debug)]
#[contracttype]
pub enum Priority {
    Low,
    Medium,
    High,
    Critical,
}

/// Optimization priority (alias for Priority)
#[derive(Clone, Debug)]
#[contracttype]
pub enum OptimizationPriority {
    Low,
    Medium,
    High,
    Critical,
}

/// Impact estimation
#[derive(Clone, Debug)]
#[contracttype]
pub struct ImpactEstimate {
    pub performance_improvement: u32, // percentage
    pub cost_reduction: u32,          // percentage
    pub user_experience_improvement: u32,
    pub reliability_improvement: u32,
}

/// Implementation complexity
#[derive(Clone, Debug)]
#[contracttype]
pub enum Complexity {
    Low,
    Medium,
    High,
    Simple,
    Moderate,
    Complex,
    VeryComplex,
}

/// Savings estimation
#[derive(Clone, Debug)]
#[contracttype]
pub struct SavingsEstimate {
    pub daily_cost_savings: u64,
    pub monthly_cost_savings: u64,
    pub annual_cost_savings: u64,
    pub performance_gains: u32,
}

/// Distributed tracing span
#[derive(Clone, Debug)]
#[contracttype]
pub struct TraceSpan {
    pub span_id: BytesN<32>,
    pub has_parent: bool,
    pub parent_span_id: BytesN<32>,
    pub operation_name: String,
    pub start_time: u64,
    pub end_time: u64,
    pub contract_address: Address,
    pub function_name: Symbol,
    pub gas_used: u64,
    pub success: bool,
    pub has_error: bool,
    pub error_message: String,
    pub metadata: Vec<(String, String)>,
}

/// Trace analysis results
#[derive(Clone, Debug)]
#[contracttype]
pub struct TraceAnalysis {
    pub trace_id: BytesN<32>,
    pub total_duration: u64,
    pub total_gas_used: u64,
    pub span_count: u32,
    pub bottlenecks: Vec<PerformanceBottleneck>,
    pub optimization_opportunities: Vec<String>,
    pub call_graph: Vec<ContractCall>,
    pub has_errors: bool,
    pub error_count: u32,
    pub error_analysis: String,
}

/// Performance bottleneck identification
#[derive(Clone, Debug)]
#[contracttype]
pub struct PerformanceBottleneck {
    pub span_id: BytesN<32>,
    pub bottleneck_type: BottleneckType,
    pub severity: RiskLevel,
    pub duration: u64,
    pub impact_percentage: u32,
    pub description: String,
}

/// Contract call in trace
#[derive(Clone, Debug)]
#[contracttype]
pub struct ContractCall {
    pub from_contract: Address,
    pub to_contract: Address,
    pub function: Symbol,
    pub duration: u64,
    pub gas_used: u64,
    pub success: bool,
}

/// Error analysis in traces
#[derive(Clone, Debug)]
#[contracttype]
pub struct ErrorAnalysis {
    pub error_type: String,
    pub error_count: u32,
    pub root_cause: String,
    pub affected_spans: Vec<BytesN<32>>,
    pub resolution_suggestions: Vec<String>,
}

/// Benchmark configuration
#[derive(Clone, Debug)]
#[contracttype]
pub struct BenchmarkConfig {
    pub benchmark_name: String,
    pub target_contracts: Vec<Address>,
    pub test_scenarios: Vec<TestScenario>,
    pub duration: u64,
    pub concurrent_users: u32,
    pub has_baseline: bool,
    pub baseline_version: String,
}

/// Test scenario for benchmarking
#[derive(Clone, Debug)]
#[contracttype]
pub struct TestScenario {
    pub scenario_name: String,
    pub function_calls: Vec<FunctionCall>,
    pub expected_performance: PerformanceExpectation,
    pub load_pattern: LoadPattern,
}

/// Function call definition
#[derive(Clone, Debug)]
#[contracttype]
pub struct FunctionCall {
    pub contract_address: Address,
    pub function_name: Symbol,
    pub call_frequency: u32,
    pub expected_duration: u64,
}

/// Performance expectations
#[derive(Clone, Debug)]
#[contracttype]
pub struct PerformanceExpectation {
    pub max_execution_time: u64,
    pub max_gas_usage: u64,
    pub max_memory_usage: u32,
    pub min_success_rate: u32,
}

/// Load testing patterns
#[derive(Clone, Debug)]
#[contracttype]
pub enum LoadPattern {
    Constant,
    Ramp,
    Spike,
    Wave,
}

/// Benchmark results
#[derive(Clone, Debug)]
#[contracttype]
pub struct BenchmarkResult {
    pub benchmark_id: BytesN<32>,
    pub benchmark_name: String,
    pub execution_time: u64,
    pub scenario_results: Vec<ScenarioResult>,
    pub has_comparison: bool,
    pub comparison_baseline_time: u64,
    pub comparison_improvement_pct: i32,
    pub bottlenecks: Vec<PerformanceBottleneck>,
    pub recommendations: Vec<String>,
    pub overall_score: u32,
    pub performance_comparison: String,
}

/// Individual scenario results
#[derive(Clone, Debug)]
#[contracttype]
pub struct ScenarioResult {
    pub scenario_name: String,
    pub success_rate: u32,
    pub average_execution_time: u64,
    pub total_gas_used: u64,
    pub peak_memory_usage: u32,
    pub error_count: u32,
    pub performance_score: u32,
}

/// Performance comparison with baseline
#[derive(Clone, Debug)]
#[contracttype]
pub struct PerformanceComparison {
    pub baseline_version: String,
    pub current_version: String,
    pub performance_delta: i32, // percentage change
    pub gas_delta: i64,
    pub memory_delta: i32,
    pub throughput_delta: i32,
    pub regression_detected: bool,
    pub baseline_duration: u64,
    pub improvement_percentage: i32,
}

/// Anomaly event detection
#[derive(Clone, Debug)]
#[contracttype]
pub struct AnomalyEvent {
    pub anomaly_id: BytesN<32>,
    pub contract_address: Address,
    pub anomaly_type: AnomalyType,
    pub severity: RiskLevel,
    pub detected_at: u64,
    pub description: String,
    pub affected_metrics: Vec<String>,
    pub root_cause_analysis: String,
    pub mitigation_steps: Vec<String>,
    pub auto_resolved: bool,
}

/// Types of anomalies
#[derive(Clone, Debug, PartialEq, Eq)]
#[contracttype]
pub enum AnomalyType {
    PerformanceDegradation,
    MemoryLeak,
    GasSpike,
    ErrorRateSpike,
    ThroughputDrop,
    LatencyIncrease,
    ResourceExhaustion,
    UnusualPatterns,
    StateInconsistency,
}

/// Resource utilization analysis
#[derive(Clone, Debug)]
#[contracttype]
pub struct ResourceUtilization {
    pub analysis_id: BytesN<32>,
    pub resource_id: BytesN<32>,
    pub contract_address: Address,
    pub analysis_period: u64,
    pub timestamp: u64,
    pub cpu_utilization: ResourceMetrics,
    pub memory_utilization: ResourceMetrics,
    pub storage_utilization: ResourceMetrics,
    pub network_utilization: ResourceMetrics,
    pub cost_analysis: CostAnalysis,
    pub optimization_opportunities: Vec<OptimizationOpportunity>,
    pub overall_efficiency_score: u64,
    pub gas_utilization: ResourceMetrics,
    pub optimization_priority: Priority,
}

/// Resource metrics
#[derive(Clone, Debug)]
#[contracttype]
pub struct ResourceMetrics {
    pub average_usage: u32,
    pub peak_usage: u32,
    pub minimum_usage: u32,
    pub utilization_trend: UtilizationTrend,
    pub efficiency_score: u32,
    pub storage_cost_per_operation: u64,
    pub cpu_efficiency_score: u64,
    pub optimization_opportunities: Vec<String>,
    pub gas_optimization_potential: u64,
    pub memory_leak_risk: RiskLevel,
    pub memory_opt_recommendations: Vec<String>,
    pub storage_efficiency_score: u32,
    pub storage_opt_suggestions: Vec<String>,
}

/// CPU utilization metrics
#[derive(Clone, Debug)]
#[contracttype]
pub struct CpuUtilization {
    pub average_usage: u32,
    pub peak_usage: u32,
    pub minimum_usage: u32,
    pub utilization_trend: UtilizationTrend,
    pub efficiency_score: u32,
    pub cpu_efficiency_score: u64,
    pub optimization_opportunities: Vec<String>,
    pub avg_instructions_per_tx: u64,
    pub avg_instr_per_tx: u64,
    pub computational_bottlenecks: Vec<String>,
}

/// Memory utilization metrics
#[derive(Clone, Debug)]
#[contracttype]
pub struct MemoryUtilization {
    pub average_usage: u32,
    pub peak_usage: u32,
    pub peak_memory_usage: u32,
    pub minimum_usage: u32,
    pub utilization_trend: UtilizationTrend,
    pub efficiency_score: u32,
    pub memory_efficiency_score: u64,
    pub average_memory_usage: u32,
    pub memory_leak_risk: RiskLevel,
    pub memory_opt_recommendations: Vec<String>,
}

/// Storage utilization metrics
#[derive(Clone, Debug)]
#[contracttype]
pub struct StorageUtilization {
    pub average_usage: u32,
    pub peak_usage: u32,
    pub minimum_usage: u32,
    pub utilization_trend: UtilizationTrend,
    pub efficiency_score: u32,
    pub storage_cost_per_operation: u64,
    pub storage_efficiency_score: u64,
    pub reads_per_transaction: u32,
    pub writes_per_transaction: u32,
    pub storage_opt_suggestions: Vec<String>,
}

/// Gas utilization metrics
#[derive(Clone, Debug)]
#[contracttype]
pub struct GasUtilization {
    pub average_usage: u32,
    pub peak_usage: u32,
    pub peak_gas_usage: u32,
    pub minimum_usage: u32,
    pub average_gas_per_transaction: u32,
    pub utilization_trend: UtilizationTrend,
    pub efficiency_score: u32,
    pub gas_optimization_potential: u64,
    pub gas_cost_analysis: u64,
    pub gas_efficiency_score: u64,
}

/// Utilization trends
#[derive(Clone, Debug)]
#[contracttype]
pub enum UtilizationTrend {
    Increasing,
    Stable,
    Decreasing,
    Volatile,
}

/// Cost analysis details
#[derive(Clone, Debug)]
#[contracttype]
pub struct CostAnalysis {
    pub total_cost: u64,
    pub cost_per_transaction: u64,
    pub cost_breakdown: Vec<CostComponent>,
    pub cost_efficiency: u32,
    pub cost_trend: CostTrend,
    pub total_cost_per_transaction: u64,
    pub gas_cost_percentage: u64,
    pub storage_cost_percentage: u64,
    pub optimization_potential_savings: u64,
    pub cost_efficiency_score: u64,
}

/// Cost component breakdown
#[derive(Clone, Debug)]
#[contracttype]
pub struct CostComponent {
    pub component_name: String,
    pub cost: u64,
    pub percentage_of_total: u32,
}

/// Optimization opportunities
#[derive(Clone, Debug)]
#[contracttype]
pub struct OptimizationOpportunity {
    pub opportunity_type: OptimizationCategory,
    pub potential_savings: u64,
    pub implementation_effort: Complexity,
    pub description: String,
}

/// Regression test configuration
#[derive(Clone, Debug)]
#[contracttype]
pub struct RegressionTestConfig {
    pub test_name: String,
    pub baseline_version: String,
    pub current_version: String,
    pub test_contracts: Vec<Address>,
    pub performance_thresholds: PerformanceThresholds,
    pub test_duration: u64,
    pub test_scenarios: Vec<RegressionTestScenario>,
}

/// Regression test scenario
#[derive(Clone, Debug)]
#[contracttype]
pub struct RegressionTestScenario {
    pub scenario_id: BytesN<32>,
    pub test_name: String,
    pub target_contract: Address,
    pub expected_performance_score: u64,
    pub test_parameters: Vec<String>,
    pub expected_execution_time: u64,
    pub expected_gas_usage: u64,
    pub expected_memory_usage: u32,
    pub expected_error_rate: u32,
    pub expected_network_latency: u32,
    pub expected_transaction_count: u32,
    pub performance_thresholds: PerformanceThresholds,
}

/// Performance thresholds for regression testing
#[derive(Clone, Debug)]
#[contracttype]
pub struct PerformanceThresholds {
    pub max_execution_time_regression: u32, // percentage
    pub max_gas_usage_regression: u32,
    pub max_memory_usage_regression: u32,
    pub min_throughput_threshold: u32,
    pub max_error_rate_increase: u32,
    pub max_execution_time: u64,
    pub max_gas_usage: u64,
    pub max_memory_usage: u64,
    pub max_error_rate: u64,
    pub max_network_latency: u64,
}

/// Regression test report
#[derive(Clone, Debug)]
#[contracttype]
pub struct RegressionReport {
    pub report_id: BytesN<32>,
    pub test_name: String,
    pub execution_time: u64,
    pub regression_detected: bool,
    pub performance_changes: Vec<PerformanceChange>,
    pub failed_thresholds: Vec<String>,
    pub recommendations: Vec<String>,
    pub overall_verdict: TestVerdict,
    pub contract_address: Address,
    pub report_period_start: u64,
    pub report_period_end: u64,
    pub total_tests_executed: u32,
    pub total_regressions_detected: u32,
    pub critical_regressions: u32,
    pub high_severity_regressions: u32,
    pub performance_trend: TrendDirection,
    pub stability_score: u64,
    pub most_problematic_areas: Vec<String>,
    pub improvement_recommendations: Vec<String>,
    pub testing_coverage_analysis: String,
    pub risk_assessment: RiskLevel,
}

/// Performance change details
#[derive(Clone, Debug)]
#[contracttype]
pub struct PerformanceChange {
    pub metric_name: String,
    pub baseline_value: u64,
    pub current_value: u64,
    pub change_percentage: i32,
    pub threshold_exceeded: bool,
}

/// Test verdict
#[derive(Clone, Debug)]
#[contracttype]
pub enum TestVerdict {
    Pass,
    Warning,
    Fail,
    Inconclusive,
}

/// Testing coverage analysis
#[derive(Clone, Debug)]
#[contracttype]
pub struct TestingCoverageAnalysis {
    pub coverage_percentage: u64,
    pub covered_functions: Vec<String>,
    pub uncovered_functions: Vec<String>,
    pub coverage_gaps: Vec<String>,
}

/// Risk assessment for regression testing
#[derive(Clone, Debug)]
#[contracttype]
pub struct RiskAssessment {
    pub overall_risk: RiskLevel,
    pub risk_factors: Vec<String>,
    pub mitigation_strategies: Vec<String>,
}

/// System health report
#[derive(Clone, Debug)]
#[contracttype]
pub struct SystemHealthReport {
    pub timestamp: u64,
    pub overall_health: HealthStatus,
    pub contract_health: Vec<ContractHealth>,
    pub system_metrics: SystemMetrics,
    pub anomalies: Vec<AnomalyEvent>,
    pub recommendations: Vec<String>,
}

/// Health status levels
#[derive(Clone, Debug)]
#[contracttype]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Individual contract health
#[derive(Clone, Debug)]
#[contracttype]
pub struct ContractHealth {
    pub contract_address: Address,
    pub health_status: HealthStatus,
    pub performance_score: u32,
    pub error_rate: u32,
    pub last_activity: u64,
}

/// System-wide metrics
#[derive(Clone, Debug)]
#[contracttype]
pub struct SystemMetrics {
    pub total_contracts: u32,
    pub active_contracts: u32,
    pub total_transactions: u32,
    pub average_response_time: u64,
    pub error_rate: u32,
    pub resource_utilization: u32,
}

/// Comprehensive diagnostic report
#[derive(Clone, Debug)]
#[contracttype]
pub struct DiagnosticReport {
    pub contract_address: Address,
    pub timestamp: u64,
    pub performance_metrics: PerformanceMetrics,
    pub anomalies: Vec<AnomalyEvent>,
    pub recommendations: Vec<OptimizationRecommendation>,
    pub health_status: HealthStatus,
}

// Legacy types for backward compatibility
/// Diagnostic event types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DiagnosticEventType {
    StateSnapshot,
    TransactionTrace,
    PerformanceMetric,
    AnomalyDetected,
    ErrorOccurred,
}

/// Contract state snapshot for visualization
#[contracttype]
#[derive(Clone, Debug)]
pub struct StateSnapshot {
    pub contract_id: Address,
    pub timestamp: u64,
    pub ledger_sequence: u32,
    pub storage_entries: u32,
    pub memory_usage_bytes: u64,
    pub state_hash: BytesN<32>,
    pub key_value_pairs: Map<Symbol, String>,
}

/// Transaction flow trace
#[contracttype]
#[derive(Clone, Debug)]
pub struct TransactionTrace {
    pub trace_id: Symbol,
    pub contract_id: Address,
    pub function_name: Symbol,
    pub caller: Address,
    pub timestamp: u64,
    pub execution_time_ms: u32,
    pub gas_used: u64,
    pub success: bool,
    pub has_error_msg: bool,
    pub error_message: String,
    pub child_calls: Vec<Symbol>,
    pub events_emitted: u32,
}

/// Performance bottleneck detection result
#[contracttype]
#[derive(Clone, Debug)]
pub struct BottleneckReport {
    pub contract_id: Address,
    pub operation: Symbol,
    pub severity: BottleneckSeverity,
    pub avg_execution_time: u32,
    pub max_execution_time: u32,
    pub avg_gas_usage: u64,
    pub max_gas_usage: u64,
    pub occurrence_count: u32,
    pub recommendations: Vec<String>,
}

/// Bottleneck severity levels
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Anomaly detection result
#[contracttype]
#[derive(Clone, Debug)]
pub struct AnomalyReport {
    pub anomaly_id: Symbol,
    pub contract_id: Address,
    pub detected_at: u64,
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub description: String,
    pub affected_operations: Vec<Symbol>,
    pub root_cause_analysis: String,
    pub suggested_fixes: Vec<String>,
}

/// Anomaly severity levels
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnomalySeverity {
    Info,
    Warning,
    Severe,
    Critical,
}

/// Real-time diagnostic session
#[contracttype]
#[derive(Clone, Debug)]
pub struct DiagnosticSession {
    pub session_id: Symbol,
    pub contract_id: Address,
    pub started_at: u64,
    pub has_ended: bool,
    pub ended_at: u64,
    pub total_traces: u32,
    pub total_anomalies: u32,
    pub total_bottlenecks: u32,
    pub is_active: bool,
}

/// Statistics for diagnostic dashboard
#[contracttype]
#[derive(Clone, Debug)]
pub struct DiagnosticStats {
    pub total_contracts_monitored: u32,
    pub total_transactions_traced: u32,
    pub total_anomalies_detected: u32,
    pub total_bottlenecks_found: u32,
    pub avg_execution_time_ms: u32,
    pub avg_gas_usage: u64,
    pub success_rate_percentage: u32,
}

/// Configuration for diagnostics
#[contracttype]
#[derive(Clone, Debug)]
pub struct DiagnosticConfig {
    pub enable_state_tracking: bool,
    pub enable_transaction_tracing: bool,
    pub enable_performance_profiling: bool,
    pub enable_anomaly_detection: bool,
    pub trace_retention_days: u32,
    pub anomaly_threshold_multiplier: u32,
    pub max_traces_per_session: u32,
}

impl Default for DiagnosticConfig {
    fn default() -> Self {
        Self {
            enable_state_tracking: true,
            enable_transaction_tracing: true,
            enable_performance_profiling: true,
            enable_anomaly_detection: true,
            trace_retention_days: 30,
            anomaly_threshold_multiplier: 2,
            max_traces_per_session: 1000,
        }
    }
}

// Additional types needed for compilation

/// Trend direction enumeration
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

/// Test execution status
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum TestExecutionStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
    Cancelled,
}

/// Regression severity levels
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum RegressionSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Monitoring status
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum MonitoringStatus {
    Inactive,
    Active,
    Paused,
    Stopped,
}

/// Regression types
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum RegressionType {
    PerformanceDegradation,
    MemoryLeakage,
    ResourceConsumption,
    ErrorRateIncrease,
    LatencyIncrease,
    FunctionalRegression,
}

/// Business impact levels
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum BusinessImpact {
    Low,
    Medium,
    High,
}

/// Implementation status
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum ImplementationStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed,
}

/// Regression test result
#[derive(Clone, Debug)]
#[contracttype]
pub struct RegressionTestResult {
    pub test_run_id: BytesN<32>,
    pub contract_address: Address,
    pub timestamp: u64,
    pub test_timestamp: u64,
    pub test_scenarios: Vec<RegressionTestScenario>,
    pub scenario_results: Vec<TestScenarioResult>,
    pub overall_status: TestStatus,
    pub regression_report: RegressionReport,
    pub test_summary: String,
    pub test_configuration: TestParameters,
    pub regressions_detected: Vec<PerformanceRegression>,
    pub performance_trends: PerformanceTrends,
    pub recommendations: Vec<String>,
    pub overall_performance_score: u32,
    pub next_test_schedule: u64,
}

/// Monitoring session
#[derive(Clone, Debug)]
#[contracttype]
pub struct MonitoringSession {
    pub session_id: BytesN<32>,
    pub contract_address: Address,
    pub start_time: u64,
    pub has_ended: bool,
    pub end_time: u64,
    pub metrics_collected: Vec<PerformanceMetrics>,
    pub status: SessionStatus,
    pub monitoring_status: SessionStatus,
    pub monitoring_config: ContinuousMonitorConfig,
    pub baseline_metrics: PerformanceMetrics,
    pub alert_thresholds: Vec<AlertThreshold>,
    pub active_alerts: Vec<PerformanceAlert>,
}

/// Session status
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum SessionStatus {
    Active,
    Completed,
    Cancelled,
}

/// Performance alert
#[derive(Clone, Debug)]
#[contracttype]
pub struct PerformanceAlert {
    pub alert_id: BytesN<32>,
    pub contract_address: Address,
    pub timestamp: u64,
    pub triggered_at: u64,
    pub severity: RiskLevel,
    pub alert_type: AlertType,
    pub description: String,
    pub alert_message: String,
    pub metric_name: String,
    pub threshold_value: u64,
    pub actual_value: u64,
    pub baseline_value: u64,
    pub escalation_level: u32,
    pub auto_resolved: bool,
    pub metrics: PerformanceMetrics,
}

/// Alert type
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum AlertType {
    GasUsageHigh,
    MemoryUsageHigh,
    ExecutionTimeSlow,
    StorageInefficient,
    ErrorRateHigh,
    AnomalyDetected,
    PerformanceRegression,
}

/// Test status
#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Running,
}

/// Test scenario result
#[derive(Clone, Debug)]
#[contracttype]
pub struct TestScenarioResult {
    pub scenario_id: BytesN<32>,
    pub scenario_name: String,
    pub test_name: String,
    pub current_metrics: PerformanceMetrics,
    pub baseline_metrics: PerformanceMetrics,
    pub performance_metrics: PerformanceMetrics,
    pub deviation_percentage: i32,
    pub status: TestStatus,
    pub execution_status: TestStatus,
    pub execution_time: u64,
    pub performance_score: u32,
    pub has_error: bool,
    pub error_details: String,
    pub has_baseline_comparison: bool,
    pub baseline_comparison: BaselineComparison,
    pub recommendations: Vec<String>,
}

/// Performance regression
#[derive(Clone, Debug)]
#[contracttype]
pub struct PerformanceRegression {
    pub regression_id: BytesN<32>,
    pub test_name: String,
    pub scenario_id: BytesN<32>,
    pub metric_name: String,
    pub baseline_value: u64,
    pub current_value: u64,
    pub regression_percentage: i32,
    pub severity: RiskLevel,
    pub regression_type: String,
    pub detected_at: u64,
    pub performance_impact: u32,
    pub affected_operations: Vec<String>,
    pub root_cause_analysis: String,
    pub mitigation_steps: Vec<String>,
    pub rollback_recommendation: String,
    pub monitoring_alerts: Vec<String>,
}

/// Regression impact
#[derive(Clone, Debug)]
#[contracttype]
pub struct RegressionImpact {
    pub overall_severity: RiskLevel,
    pub affected_scenarios: Vec<String>,
    pub estimated_user_impact: u32,
    pub recommendation: String,
    pub execution_time_change: i32,
    pub gas_consumption_change: i32,
    pub memory_usage_change: i32,
    pub throughput_change: i32,
    pub error_rate_change: i32,
}

/// Anomaly trends
#[derive(Clone, Debug)]
#[contracttype]
pub struct AnomalyTrends {
    pub trend_direction: TrendDirection,
    pub frequency_increase: bool,
    pub severity_escalation: bool,
    pub has_prediction: bool,
    pub predicted_next_anomaly: u64,
    pub total_anomalies: u32,
    pub critical_anomalies: u32,
    pub high_severity_anomalies: u32,
    pub most_common_type: String,
    pub improvement_rate: i32,
}

/// Cost benefit analysis
#[derive(Clone, Debug)]
#[contracttype]
pub struct CostBenefitAnalysis {
    pub recommendation_id: BytesN<32>,
    pub optimization_cost: u64,
    pub implementation_cost: u64,
    pub expected_savings: u64,
    pub annual_cost_savings: u64,
    pub payback_period_days: u32,
    pub payback_period_months: u32,
    pub roi_percentage: u32,
    pub three_year_roi: u32,
    pub risk_assessment: String,
    pub business_impact: String,
    pub performance_impact: i32,
    pub scalability_benefit: u32,
    pub technical_debt_reduction: u32,
}

/// Optimization progress
#[derive(Clone, Debug)]
#[contracttype]
pub struct OptimizationProgress {
    pub recommendation_id: BytesN<32>,
    pub contract_address: Address,
    pub total_optimizations: u32,
    pub completed: u32,
    pub in_progress: u32,
    pub pending: u32,
    pub success_rate: u32,
    pub implementation_status: ImplementationStatus,
    pub progress_percentage: u32,
    pub cost_savings_achieved: u64,
    pub gas_efficiency_change: i32,
    pub memory_efficiency_change: i32,
    pub cpu_efficiency_change: i32,
    pub storage_efficiency_change: i32,
    pub performance_impact: i32,
    pub monitoring_recommendations: Vec<String>,
    pub next_steps: Vec<String>,
}

/// Continuous monitoring config
#[derive(Clone, Debug)]
#[contracttype]
pub struct ContinuousMonitorConfig {
    pub enabled: bool,
    pub check_interval_seconds: u64,
    pub alert_threshold: u32,
    pub auto_remediation: bool,
}

/// Baseline comparison
#[derive(Clone, Debug)]
#[contracttype]
pub struct BaselineComparison {
    pub baseline_metrics: PerformanceMetrics,
    pub current_metrics: PerformanceMetrics,
    pub deviation: i32,
    pub is_regression: bool,
    pub baseline_score: u64,
    pub current_score: u64,
    pub performance_delta: i32,
    pub regression_detected: bool,
    pub significance_level: u32,
    pub baseline_duration: u64,
    pub current_duration: u64,
    pub improvement_percentage: i32,
    pub has_regression: bool,
}

/// Performance trends
#[derive(Clone, Debug)]
#[contracttype]
pub struct PerformanceTrends {
    pub trend_direction: TrendDirection,
    pub performance_change_pct: i32,
    pub performance_score_trend: TrendDirection,
    pub stability_trend: TrendDirection,
    pub predicted_performance: u64,
    pub prediction: String,
    pub regression_frequency_trend: TrendDirection,
    pub confidence_level: u32,
}

/// Test parameters
#[derive(Clone, Debug)]
#[contracttype]
pub struct TestParameters {
    pub test_name: String,
    pub iterations: u32,
    pub timeout_seconds: u64,
    pub acceptable_deviation_pct: u32,
}

/// Alert threshold
#[derive(Clone, Debug)]
#[contracttype]
pub struct AlertThreshold {
    pub metric_name: String,
    pub warning_threshold: u64,
    pub critical_threshold: u64,
    pub enabled: bool,
}
