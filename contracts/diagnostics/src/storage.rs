use crate::{errors::DiagnosticsError, types::*};
use soroban_sdk::{Address, BytesN, Env, String, Symbol, Vec};

/// Storage keys for the diagnostics platform
pub enum DataKey {
    Admin,
    Config,
    PerformanceMetrics(Address, u64), // (contract, timestamp)
    MonitoringConfig(Address),
    CapacityPrediction(Address, u64), // (contract, prediction_time)
    BehaviorAnalysis(Address, u64),   // (user, analysis_time)
    OptimizationRecommendations(Address),
    TraceData(BytesN<32>),             // trace_id
    BenchmarkResults(String),          // benchmark_name
    AnomalyEvents(Address, u64),       // (contract, timestamp)
    ResourceUtilization(Address, u64), // (contract, analysis_time)
    RegressionReports(String),         // test_name
    SystemHealth,
    MonitoredContracts,
}

impl DataKey {
    pub fn to_symbol(&self, env: &Env) -> Symbol {
        match self {
            DataKey::Admin => Symbol::new(env, "admin"),
            DataKey::Config => Symbol::new(env, "config"),
            DataKey::PerformanceMetrics(_addr, _ts) => Symbol::new(env, "perf"),
            DataKey::MonitoringConfig(_addr) => Symbol::new(env, "mon_cfg"),
            DataKey::CapacityPrediction(_addr, _ts) => Symbol::new(env, "cap_pred"),
            DataKey::BehaviorAnalysis(_addr, _ts) => Symbol::new(env, "behav"),
            DataKey::OptimizationRecommendations(_addr) => Symbol::new(env, "opt_rec"),
            DataKey::TraceData(_trace_id) => Symbol::new(env, "trace"),
            DataKey::BenchmarkResults(_name) => Symbol::new(env, "bench"),
            DataKey::AnomalyEvents(_addr, _ts) => Symbol::new(env, "anom"),
            DataKey::ResourceUtilization(_addr, _ts) => Symbol::new(env, "res_util"),
            DataKey::RegressionReports(_name) => Symbol::new(env, "reg_rep"),
            DataKey::SystemHealth => Symbol::new(env, "sys_health"),
            DataKey::MonitoredContracts => Symbol::new(env, "monitored"),
        }
    }
}

/// Storage operations for the diagnostics platform
pub struct DiagnosticsStorage;

impl DiagnosticsStorage {
    /// Set the admin address
    pub fn set_admin(env: &Env, admin: &Address) {
        env.storage()
            .instance()
            .set(&DataKey::Admin.to_symbol(env), admin);
    }

    /// Get the admin address
    pub fn get_admin(env: &Env) -> Result<Address, DiagnosticsError> {
        env.storage()
            .instance()
            .get(&DataKey::Admin.to_symbol(env))
            .ok_or(DiagnosticsError::AdminNotSet)
    }

    /// Set diagnostics configuration
    pub fn set_config(env: &Env, config: &DiagnosticsConfig) {
        env.storage()
            .persistent()
            .set(&DataKey::Config.to_symbol(env), config);
    }

    /// Get diagnostics configuration
    pub fn get_config(env: &Env) -> Result<DiagnosticsConfig, DiagnosticsError> {
        env.storage()
            .persistent()
            .get(&DataKey::Config.to_symbol(env))
            .ok_or(DiagnosticsError::ConfigNotSet)
    }

    /// Store performance metrics
    pub fn store_performance_metrics(
        env: &Env,
        contract_address: &Address,
        metrics: &PerformanceMetrics,
    ) {
        let key = DataKey::PerformanceMetrics(contract_address.clone(), metrics.timestamp);
        env.storage().persistent().set(&key.to_symbol(env), metrics);
    }

    /// Get performance metrics for a contract at a specific time
    pub fn get_performance_metrics(
        env: &Env,
        contract_address: &Address,
        timestamp: u64,
    ) -> Option<PerformanceMetrics> {
        let key = DataKey::PerformanceMetrics(contract_address.clone(), timestamp);
        env.storage().persistent().get(&key.to_symbol(env))
    }

    /// Get latest performance metrics for a contract
    pub fn get_latest_performance_metrics(
        env: &Env,
        contract_address: &Address,
    ) -> Option<PerformanceMetrics> {
        // In a real implementation, we would iterate through recent timestamps
        // For now, we'll use current timestamp minus some offset
        let current_time = env.ledger().timestamp();
        for i in 0..3600 {
            // Check last hour in second intervals
            let timestamp = current_time - i;
            if let Some(metrics) = Self::get_performance_metrics(env, contract_address, timestamp) {
                return Some(metrics);
            }
        }
        None
    }

    /// Set monitoring configuration for a contract
    pub fn set_monitoring_config(env: &Env, contract_address: &Address, config: &MonitoringConfig) {
        let key = DataKey::MonitoringConfig(contract_address.clone());
        env.storage().persistent().set(&key.to_symbol(env), config);
    }

    /// Get monitoring configuration for a contract
    pub fn get_monitoring_config(
        env: &Env,
        contract_address: &Address,
    ) -> Option<MonitoringConfig> {
        let key = DataKey::MonitoringConfig(contract_address.clone());
        env.storage().persistent().get(&key.to_symbol(env))
    }

    /// Store capacity prediction
    pub fn store_capacity_prediction(
        env: &Env,
        contract_address: &Address,
        prediction: &CapacityPrediction,
    ) {
        let key = DataKey::CapacityPrediction(contract_address.clone(), prediction.generated_at);
        env.storage()
            .persistent()
            .set(&key.to_symbol(env), prediction);
    }

    /// Get capacity prediction
    pub fn get_capacity_prediction(
        env: &Env,
        contract_address: &Address,
        timestamp: u64,
    ) -> Option<CapacityPrediction> {
        let key = DataKey::CapacityPrediction(contract_address.clone(), timestamp);
        env.storage().persistent().get(&key.to_symbol(env))
    }

    /// Store behavior analysis
    pub fn store_behavior_analysis(env: &Env, user: &Address, analysis: &BehaviorAnalysis) {
        let key = DataKey::BehaviorAnalysis(user.clone(), analysis.analysis_period);
        env.storage()
            .persistent()
            .set(&key.to_symbol(env), analysis);
    }

    /// Get behavior analysis
    pub fn get_behavior_analysis(
        env: &Env,
        user: &Address,
        analysis_time: u64,
    ) -> Option<BehaviorAnalysis> {
        let key = DataKey::BehaviorAnalysis(user.clone(), analysis_time);
        env.storage().persistent().get(&key.to_symbol(env))
    }

    /// Store optimization recommendations
    pub fn store_optimization_recommendations(
        env: &Env,
        contract_address: &Address,
        recommendations: &Vec<OptimizationRecommendation>,
    ) {
        let key = DataKey::OptimizationRecommendations(contract_address.clone());
        env.storage()
            .persistent()
            .set(&key.to_symbol(env), recommendations);
    }

    /// Get optimization recommendations
    pub fn get_optimization_recommendations(
        env: &Env,
        contract_address: &Address,
    ) -> Option<Vec<OptimizationRecommendation>> {
        let key = DataKey::OptimizationRecommendations(contract_address.clone());
        env.storage().persistent().get(&key.to_symbol(env))
    }

    /// Store trace data
    pub fn store_trace_data(env: &Env, trace_id: &BytesN<32>, analysis: &TraceAnalysis) {
        let key = DataKey::TraceData(trace_id.clone());
        env.storage()
            .persistent()
            .set(&key.to_symbol(env), analysis);
    }

    /// Get trace data
    pub fn get_trace_data(env: &Env, trace_id: &BytesN<32>) -> Option<TraceAnalysis> {
        let key = DataKey::TraceData(trace_id.clone());
        env.storage().persistent().get(&key.to_symbol(env))
    }

    /// Store benchmark results
    pub fn store_benchmark_results(env: &Env, benchmark_name: &String, results: &BenchmarkResult) {
        let key = DataKey::BenchmarkResults(benchmark_name.clone());
        env.storage().persistent().set(&key.to_symbol(env), results);
    }

    /// Get benchmark results
    pub fn get_benchmark_results(env: &Env, benchmark_name: &String) -> Option<BenchmarkResult> {
        let key = DataKey::BenchmarkResults(benchmark_name.clone());
        env.storage().persistent().get(&key.to_symbol(env))
    }

    /// Store anomaly events
    pub fn store_anomaly_events(env: &Env, contract_address: &Address, events: &Vec<AnomalyEvent>) {
        let timestamp = env.ledger().timestamp();
        let key = DataKey::AnomalyEvents(contract_address.clone(), timestamp);
        env.storage().persistent().set(&key.to_symbol(env), events);
    }

    /// Get anomaly events
    pub fn get_anomaly_events(
        env: &Env,
        contract_address: &Address,
        timestamp: u64,
    ) -> Option<Vec<AnomalyEvent>> {
        let key = DataKey::AnomalyEvents(contract_address.clone(), timestamp);
        env.storage().persistent().get(&key.to_symbol(env))
    }

    /// Store resource utilization analysis
    pub fn store_resource_utilization(
        env: &Env,
        contract_address: &Address,
        utilization: &ResourceUtilization,
    ) {
        let key =
            DataKey::ResourceUtilization(contract_address.clone(), utilization.analysis_period);
        env.storage()
            .persistent()
            .set(&key.to_symbol(env), utilization);
    }

    /// Get resource utilization analysis
    pub fn get_resource_utilization(
        env: &Env,
        contract_address: &Address,
        analysis_time: u64,
    ) -> Option<ResourceUtilization> {
        let key = DataKey::ResourceUtilization(contract_address.clone(), analysis_time);
        env.storage().persistent().get(&key.to_symbol(env))
    }

    /// Store regression test report
    pub fn store_regression_report(env: &Env, test_name: &String, report: &RegressionReport) {
        let key = DataKey::RegressionReports(test_name.clone());
        env.storage().persistent().set(&key.to_symbol(env), report);
    }

    /// Get regression test report
    pub fn get_regression_report(env: &Env, test_name: &String) -> Option<RegressionReport> {
        let key = DataKey::RegressionReports(test_name.clone());
        env.storage().persistent().get(&key.to_symbol(env))
    }

    /// Store system health report
    pub fn store_system_health(env: &Env, health_report: &SystemHealthReport) {
        env.storage()
            .persistent()
            .set(&DataKey::SystemHealth.to_symbol(env), health_report);
    }

    /// Get system health report
    pub fn get_system_health(env: &Env) -> Option<SystemHealthReport> {
        env.storage()
            .persistent()
            .get(&DataKey::SystemHealth.to_symbol(env))
    }

    /// Add a contract to the monitored contracts list
    pub fn add_monitored_contract(env: &Env, contract_address: &Address) {
        let mut monitored = Self::get_monitored_contracts(env);
        if !monitored.iter().any(|addr| addr == *contract_address) {
            monitored.push_back(contract_address.clone());
            env.storage()
                .persistent()
                .set(&DataKey::MonitoredContracts.to_symbol(env), &monitored);
        }
    }

    /// Remove a contract from the monitored contracts list
    pub fn remove_monitored_contract(env: &Env, contract_address: &Address) {
        let mut monitored = Self::get_monitored_contracts(env);
        if let Some(index) = monitored.iter().position(|addr| addr == *contract_address) {
            monitored.remove(index as u32);
            env.storage()
                .persistent()
                .set(&DataKey::MonitoredContracts.to_symbol(env), &monitored);
        }
    }

    /// Get the list of monitored contracts
    pub fn get_monitored_contracts(env: &Env) -> Vec<Address> {
        env.storage()
            .persistent()
            .get(&DataKey::MonitoredContracts.to_symbol(env))
            .unwrap_or(Vec::new(env))
    }

    // Additional storage methods for new diagnostic features

    /// Store regression test result
    pub fn store_regression_test_result(
        env: &Env,
        _contract_address: &Address,
        result: &RegressionTestResult,
    ) {
        let key = DataKey::RegressionReports(result.regression_report.test_name.clone());
        env.storage().persistent().set(&key.to_symbol(env), result);
    }

    /// Store monitoring session
    pub fn store_monitoring_session(
        env: &Env,
        _contract_address: &Address,
        session: &MonitoringSession,
    ) {
        let key = Symbol::new(env, "mon_session");
        env.storage().persistent().set(&key, session);
    }

    /// Get monitoring session
    pub fn get_monitoring_session(
        env: &Env,
        _contract_address: &Address,
        _session_id: &BytesN<32>,
    ) -> Result<MonitoringSession, DiagnosticsError> {
        let key = Symbol::new(env, "mon_session");
        env.storage()
            .persistent()
            .get(&key)
            .ok_or(DiagnosticsError::DataNotFound)
    }

    /// Store performance alert
    pub fn store_performance_alert(
        env: &Env,
        _contract_address: &Address,
        alert: &PerformanceAlert,
    ) {
        let key = Symbol::new(env, "perf_alert");
        env.storage().persistent().set(&key, alert);
    }

    /// Get regression test results in period
    pub fn get_regression_test_results_in_period(
        env: &Env,
        _contract_address: &Address,
        _start_time: u64,
        _end_time: u64,
    ) -> Result<Vec<RegressionTestResult>, DiagnosticsError> {
        let results = Vec::new(env);
        // In a real implementation, this would query based on timestamps
        // For now, return empty vector as placeholder
        Ok(results)
    }

    /// Get performance alerts in period
    pub fn get_performance_alerts_in_period(
        env: &Env,
        _contract_address: &Address,
        _start_time: u64,
        _end_time: u64,
    ) -> Result<Vec<PerformanceAlert>, DiagnosticsError> {
        let alerts = Vec::new(env);
        // In a real implementation, this would query based on timestamps
        // For now, return empty vector as placeholder
        Ok(alerts)
    }

    /// Get historical regression results
    pub fn get_historical_regression_results(
        env: &Env,
        _contract_address: &Address,
        _days: u32,
    ) -> Result<Vec<RegressionTestResult>, DiagnosticsError> {
        let results = Vec::new(env);
        // In a real implementation, this would query historical data
        // For now, return empty vector as placeholder
        Ok(results)
    }

    /// Get recent performance metrics
    pub fn get_recent_performance_metrics(
        env: &Env,
        _contract_address: &Address,
        _hours: u32,
    ) -> Result<Vec<PerformanceMetrics>, DiagnosticsError> {
        let metrics = Vec::new(env);
        // In a real implementation, this would query recent metrics
        // For now, return empty vector as placeholder
        Ok(metrics)
    }

    /// Get anomaly events in period
    pub fn get_anomaly_events_in_period(
        env: &Env,
        _contract_address: &Address,
        _period: u64,
    ) -> Result<Vec<AnomalyEvent>, DiagnosticsError> {
        let anomalies = Vec::new(env);
        // In a real implementation, this would query based on timestamps
        // For now, return empty vector as placeholder
        Ok(anomalies)
    }
}
