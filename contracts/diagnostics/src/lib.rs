pub mod types;
pub mod storage;
pub mod events;
pub mod errors;
pub mod performance_monitor;
pub mod predictive_engine;
pub mod behavior_analyzer;
pub mod optimization_engine;
pub mod distributed_tracer;
pub mod benchmark_engine;
pub mod anomaly_detector;
pub mod resource_optimizer;
pub mod regression_tester;

use crate::{
    errors::DiagnosticsError,
    events::DiagnosticsEvents,
    storage::DiagnosticsStorage,
    types::*,
    performance_monitor::PerformanceMonitor,
    predictive_engine::PredictiveEngine,
    behavior_analyzer::BehaviorAnalyzer,
    optimization_engine::OptimizationEngine,
    distributed_tracer::DistributedTracer,
    benchmark_engine::BenchmarkEngine,
    anomaly_detector::AnomalyDetector,
    resource_optimizer::ResourceOptimizer,
    regression_tester::RegressionTester,
};

use soroban_sdk::{contract, contractimpl, Address, Env, Symbol, Vec, BytesN, String};

#[contract]
pub struct Diagnostics;

#[contractimpl]
impl Diagnostics {
    /// Initialize the comprehensive diagnostics platform
    pub fn initialize(env: Env, admin: Address, config: DiagnosticsConfig) -> Result<(), DiagnosticsError> {
        admin.require_auth();
        
        DiagnosticsStorage::set_admin(&env, &admin);
        DiagnosticsStorage::set_config(&env, &config);
        DiagnosticsEvents::emit_initialized(&env, &admin);
        
        Ok(())
    }

    /// Start real-time performance monitoring for a contract
    pub fn start_performance_monitoring(
        env: Env,
        contract_address: Address,
        monitoring_config: MonitoringConfig,
    ) -> Result<BytesN<32>, DiagnosticsError> {
        let admin = DiagnosticsStorage::get_admin(&env)?;
        admin.require_auth();

        PerformanceMonitor::start_monitoring(&env, &contract_address, monitoring_config)
    }

    /// Stop performance monitoring for a contract
    pub fn stop_performance_monitoring(
        env: Env,
        contract_address: Address,
    ) -> Result<(), DiagnosticsError> {
        let admin = DiagnosticsStorage::get_admin(&env)?;
        admin.require_auth();

        PerformanceMonitor::stop_monitoring(&env, &contract_address)
    }

    /// Record performance metrics
    pub fn record_performance_metrics(
        env: Env,
        contract_address: Address,
        metrics: PerformanceMetrics,
    ) -> Result<(), DiagnosticsError> {
        PerformanceMonitor::record_metrics(&env, &contract_address, metrics)
    }

    /// Get current performance metrics for a contract
    pub fn get_performance_metrics(
        env: Env,
        contract_address: Address,
    ) -> Result<PerformanceMetrics, DiagnosticsError> {
        PerformanceMonitor::get_current_metrics(&env, &contract_address)
    }

    /// Generate predictive capacity planning analysis
    pub fn generate_capacity_prediction(
        env: Env,
        contract_address: Address,
        prediction_horizon: u64, // seconds into the future
    ) -> Result<CapacityPrediction, DiagnosticsError> {
        PredictiveEngine::predict_capacity(&env, &contract_address, prediction_horizon)
    }

    /// Analyze user behavior patterns
    pub fn analyze_user_behavior(
        env: Env,
        user: Address,
        analysis_period: u64, // seconds of historical data to analyze
    ) -> Result<BehaviorAnalysis, DiagnosticsError> {
        BehaviorAnalyzer::analyze_behavior(&env, &user, analysis_period)
    }

    /// Generate automated optimization recommendations
    pub fn generate_opt_recommendations(
        env: Env,
        contract_address: Address,
    ) -> Result<Vec<OptimizationRecommendation>, DiagnosticsError> {
        OptimizationEngine::generate_recommendations(&env, &contract_address)
    }

    /// Start distributed tracing for cross-contract operations
    pub fn start_distributed_trace(
        env: Env,
        trace_name: String,
        contract_address: Address,
    ) -> Result<BytesN<32>, DiagnosticsError> {
        DistributedTracer::start_trace(&env, trace_name, &contract_address)
    }

    /// Add span to distributed trace
    pub fn add_trace_span(
        env: Env,
        trace_id: BytesN<32>,
        span: TraceSpan,
    ) -> Result<(), DiagnosticsError> {
        DistributedTracer::add_span(&env, trace_id, span)
    }

    /// Complete distributed trace and analyze
    pub fn complete_trace(
        env: Env,
        trace_id: BytesN<32>,
    ) -> Result<TraceAnalysis, DiagnosticsError> {
        DistributedTracer::complete_trace(&env, trace_id)
    }

    /// Run performance benchmark
    pub fn run_benchmark(
        env: Env,
        benchmark_config: BenchmarkConfig,
    ) -> Result<BenchmarkResult, DiagnosticsError> {
        let admin = DiagnosticsStorage::get_admin(&env)?;
        admin.require_auth();

        BenchmarkEngine::run_benchmark(&env, benchmark_config)
    }

    /// Detect system anomalies
    pub fn detect_anomalies(
        env: Env,
        contract_address: Address,
        detection_period: u64,
    ) -> Result<Vec<AnomalyEvent>, DiagnosticsError> {
        AnomalyDetector::detect_anomalies(&env, &contract_address, detection_period)
    }

    /// Analyze resource utilization and costs
    pub fn analyze_resource_utilization(
        env: Env,
        contract_address: Address,
        analysis_period: u64,
    ) -> Result<ResourceUtilization, DiagnosticsError> {
        ResourceOptimizer::analyze_resource_utilization(&env, &contract_address, analysis_period)
    }

    /// Generate resource optimization recommendations
    pub fn generate_resource_opt_recs(
        env: Env,
        contract_address: Address,
        resource_data: ResourceUtilization,
    ) -> Result<Vec<OptimizationRecommendation>, DiagnosticsError> {
        ResourceOptimizer::generate_optimization_recommendations(&env, &contract_address, &resource_data)
    }

    /// Monitor optimization implementation progress
    pub fn monitor_optimization_progress(
        env: Env,
        contract_address: Address,
        recommendation_id: BytesN<32>,
        baseline_metrics: ResourceUtilization,
    ) -> Result<OptimizationProgress, DiagnosticsError> {
        ResourceOptimizer::monitor_optimization_progress(&env, &contract_address, &recommendation_id, &baseline_metrics)
    }

    /// Run comprehensive regression tests
    pub fn run_regression_tests(
        env: Env,
        contract_address: Address,
        test_configuration: RegressionTestConfig,
    ) -> Result<RegressionTestResult, DiagnosticsError> {
        let admin = DiagnosticsStorage::get_admin(&env)?;
        admin.require_auth();

        RegressionTester::run_regression_tests(&env, &contract_address, &test_configuration)
    }

    /// Setup continuous performance monitoring
    pub fn setup_continuous_monitoring(
        env: Env,
        contract_address: Address,
        monitoring_config: ContinuousMonitorConfig,
    ) -> Result<MonitoringSession, DiagnosticsError> {
        let admin = DiagnosticsStorage::get_admin(&env)?;
        admin.require_auth();

        RegressionTester::setup_continuous_monitoring(&env, &contract_address, &monitoring_config)
    }

    /// Check real-time performance for regressions
    pub fn check_real_time_performance(
        env: Env,
        contract_address: Address,
        session_id: BytesN<32>,
        current_metrics: PerformanceMetrics,
    ) -> Result<Vec<PerformanceAlert>, DiagnosticsError> {
        RegressionTester::check_real_time_performance(&env, &contract_address, &session_id, &current_metrics)
    }

    /// Generate comprehensive regression report
    pub fn generate_regression_report(
        env: Env,
        contract_address: Address,
        time_period: u64,
    ) -> Result<RegressionReport, DiagnosticsError> {
        let admin = DiagnosticsStorage::get_admin(&env)?;
        admin.require_auth();

        RegressionTester::generate_regression_report(&env, &contract_address, time_period)
    }

    /// Analyze anomaly trends
    pub fn get_anomaly_trends(
        env: Env,
        contract_address: Address,
        period: u64,
    ) -> Result<AnomalyTrends, DiagnosticsError> {
        AnomalyDetector::get_anomaly_trends(&env, &contract_address, period)
    }

    /// Perform cost-benefit analysis for optimizations
    pub fn analyze_opt_cost_benefit(
        env: Env,
        recommendation: OptimizationRecommendation,
    ) -> Result<CostBenefitAnalysis, DiagnosticsError> {
        let analysis = ResourceOptimizer::analyze_optimization_cost_benefit(&env, &recommendation);
        Ok(analysis)
    }

    /// Run performance regression test
    pub fn run_regression_test(
        env: Env,
        test_config: RegressionTestConfig,
    ) -> Result<RegressionTestResult, DiagnosticsError> {
        let admin = DiagnosticsStorage::get_admin(&env)?;
        admin.require_auth();

        // Extract contract address from test config for the new implementation
        let contract_address = if test_config.test_contracts.is_empty() {
            return Err(DiagnosticsError::InvalidRegressionConfig);
        } else {
            test_config.test_contracts.get(0).unwrap().clone()
        };
        RegressionTester::run_regression_tests(&env, &contract_address, &test_config)
    }

    /// Get comprehensive system health report
    pub fn get_system_health_report(
        env: Env,
    ) -> Result<SystemHealthReport, DiagnosticsError> {
        let contracts = DiagnosticsStorage::get_monitored_contracts(&env);
        let mut health_report = SystemHealthReport {
            timestamp: env.ledger().timestamp(),
            overall_health: HealthStatus::Healthy,
            contract_health: Vec::new(&env),
            system_metrics: SystemMetrics {
                total_contracts: contracts.len(),
                active_contracts: 0,
                total_transactions: 0,
                average_response_time: 0,
                error_rate: 0,
                resource_utilization: 0,
            },
            anomalies: Vec::new(&env),
            recommendations: Vec::new(&env),
        };

        // Aggregate health data from all monitored contracts
        for contract in contracts.iter() {
            if let Ok(metrics) = PerformanceMonitor::get_current_metrics(&env, &contract) {
                health_report.system_metrics.total_transactions += metrics.transaction_count;
                health_report.system_metrics.average_response_time += metrics.average_execution_time;
                
                if metrics.error_rate < 5 && metrics.average_execution_time < 1000 {
                    health_report.system_metrics.active_contracts += 1;
                }
            }
        }

        if health_report.system_metrics.active_contracts > 0 {
            health_report.system_metrics.average_response_time /= health_report.system_metrics.active_contracts as u64;
        }

        Ok(health_report)
    }

    /// Legacy diagnostic function - now enhanced
    pub fn run_diagnostic(env: Env, contract_address: Address) -> Result<DiagnosticReport, DiagnosticsError> {
        let performance = PerformanceMonitor::get_current_metrics(&env, &contract_address)?;
        let anomalies = AnomalyDetector::detect_anomalies(&env, &contract_address, 3600)?; // Last hour
        let recommendations = OptimizationEngine::generate_recommendations(&env, &contract_address)?;
        
        Ok(DiagnosticReport {
            contract_address: contract_address.clone(),
            timestamp: env.ledger().timestamp(),
            performance_metrics: performance.clone(),
            anomalies: anomalies.clone(),
            recommendations,
            health_status: if anomalies.is_empty() && performance.error_rate < 5 {
                HealthStatus::Healthy
            } else {
                HealthStatus::Warning
            },
        })
    }
}
