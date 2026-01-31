#![no_std]

//! # Diagnostics Contract
//!
//! Advanced debugging and diagnostics platform for Soroban smart contracts.
//! 
//! ## Features
//! - Real-time contract state visualization
//! - Transaction flow tracing and analysis
//! - Performance bottleneck detection
//! - Automated anomaly detection
//! - Interactive debugging capabilities

mod anomaly_detector;
mod performance_profiler;
mod state_tracker;
mod transaction_tracer;
mod types;

pub use anomaly_detector::AnomalyDetector;
pub use performance_profiler::PerformanceProfiler;
pub use state_tracker::StateTracker;
pub use transaction_tracer::TransactionTracer;
pub use types::*;

use soroban_sdk::{contract, contractimpl, Address, Env, Map, String, Symbol, Vec};

#[contract]
pub struct DiagnosticsContract;

#[contractimpl]
impl DiagnosticsContract {
    /// Initialize the diagnostics contract
    pub fn initialize(env: Env, admin: Address) -> Result<(), String> {
        // Store admin address
        env.storage().persistent().set(&Symbol::new(&env, "admin"), &admin);

        // Initialize default configuration
        let config = DiagnosticConfig::default();
        env.storage().persistent().set(&Symbol::new(&env, "config"), &config);

        // Initialize session counter
        env.storage().persistent().set(&Symbol::new(&env, "session_count"), &0u32);

        // Emit initialization event
        env.events().publish(
            (Symbol::new(&env, "DIAGNOSTIC"), Symbol::new(&env, "INIT")),
            admin,
        );

        Ok(())
    }

    /// Start a new diagnostic session
    pub fn start_session(env: Env, contract_id: Address) -> Symbol {
        let session_count: u32 = env
            .storage()
            .persistent()
            .get(&Symbol::new(&env, "session_count"))
            .unwrap_or(0);

        let session_id = Symbol::new(&env, &format!("session_{}", session_count));
        let started_at = env.ledger().timestamp();

        let session = DiagnosticSession {
            session_id: session_id.clone(),
            contract_id: contract_id.clone(),
            started_at,
            ended_at: None,
            total_traces: 0,
            total_anomalies: 0,
            total_bottlenecks: 0,
            is_active: true,
        };

        // Store session
        env.storage().persistent().set(
            &(Symbol::new(&env, "session"), session_id.clone()),
            &session,
        );

        // Increment session counter
        env.storage()
            .persistent()
            .set(&Symbol::new(&env, "session_count"), &(session_count + 1));

        // Emit event
        env.events().publish(
            (Symbol::new(&env, "DIAGNOSTIC"), Symbol::new(&env, "SESSION_START")),
            (session_id.clone(), contract_id),
        );

        session_id
    }

    /// End a diagnostic session
    pub fn end_session(env: Env, session_id: Symbol) -> Result<DiagnosticSession, String> {
        let mut session: DiagnosticSession = env
            .storage()
            .persistent()
            .get(&(Symbol::new(&env, "session"), session_id.clone()))
            .ok_or_else(|| String::from_str(&env, "Session not found"))?;

        session.ended_at = Some(env.ledger().timestamp());
        session.is_active = false;

        // Update session
        env.storage().persistent().set(
            &(Symbol::new(&env, "session"), session_id.clone()),
            &session,
        );

        // Emit event
        env.events().publish(
            (Symbol::new(&env, "DIAGNOSTIC"), Symbol::new(&env, "SESSION_END")),
            session_id,
        );

        Ok(session)
    }

    /// Capture a state snapshot
    pub fn capture_state_snapshot(env: Env, contract_id: Address) -> StateSnapshot {
        let snapshot = StateTracker::capture_snapshot(&env, &contract_id);

        // Store snapshot
        env.storage().persistent().set(
            &(
                Symbol::new(&env, "snapshot"),
                contract_id.clone(),
                snapshot.timestamp,
            ),
            &snapshot,
        );

        // Emit event
        env.events().publish(
            (Symbol::new(&env, "DIAGNOSTIC"), Symbol::new(&env, "SNAPSHOT")),
            (contract_id, snapshot.timestamp),
        );

        snapshot
    }

    /// Start tracing a transaction
    pub fn start_trace(
        env: Env,
        contract_id: Address,
        function_name: Symbol,
        caller: Address,
    ) -> Symbol {
        TransactionTracer::start_trace(&env, &contract_id, function_name, &caller)
    }

    /// Complete a transaction trace
    pub fn complete_trace(
        env: Env,
        trace_id: Symbol,
        contract_id: Address,
        function_name: Symbol,
        caller: Address,
        success: bool,
        error_message: Option<String>,
        child_calls: Vec<Symbol>,
        gas_used: u64,
    ) -> TransactionTrace {
        TransactionTracer::complete_trace(
            &env,
            trace_id,
            &contract_id,
            function_name,
            &caller,
            success,
            error_message,
            child_calls,
            gas_used,
        )
    }

    /// Record a performance metric
    pub fn record_performance_metric(
        env: Env,
        contract_id: Address,
        operation: Symbol,
        execution_time_ms: u32,
        gas_consumed: u64,
        memory_peak_bytes: u64,
        cpu_instructions: u64,
        io_operations: u32,
    ) -> PerformanceMetric {
        PerformanceProfiler::record_metric(
            &env,
            &contract_id,
            operation,
            execution_time_ms,
            gas_consumed,
            memory_peak_bytes,
            cpu_instructions,
            io_operations,
        )
    }

    /// Identify performance bottlenecks
    pub fn identify_bottlenecks(
        env: Env,
        metrics: Vec<PerformanceMetric>,
        operation_filter: Option<Symbol>,
    ) -> Vec<BottleneckReport> {
        PerformanceProfiler::identify_bottlenecks(&env, &metrics, operation_filter)
    }

    /// Detect anomalies
    pub fn detect_anomalies(
        env: Env,
        contract_id: Address,
        recent_metrics: Vec<PerformanceMetric>,
        baseline_metrics: Vec<PerformanceMetric>,
    ) -> Vec<AnomalyReport> {
        AnomalyDetector::detect_anomalies(&env, &contract_id, &recent_metrics, &baseline_metrics)
    }

    /// Get diagnostic statistics
    pub fn get_diagnostic_stats(env: Env) -> DiagnosticStats {
        // In production, this would aggregate from stored data
        // For now, return sample stats
        DiagnosticStats {
            total_contracts_monitored: 5,
            total_transactions_traced: 150,
            total_anomalies_detected: 3,
            total_bottlenecks_found: 2,
            avg_execution_time_ms: 125,
            avg_gas_usage: 75000,
            success_rate_percentage: 95,
        }
    }

    /// Get configuration
    pub fn get_config(env: Env) -> DiagnosticConfig {
        env.storage()
            .persistent()
            .get(&Symbol::new(&env, "config"))
            .unwrap_or(DiagnosticConfig::default())
    }

    /// Update configuration (admin only)
    pub fn update_config(env: Env, caller: Address, config: DiagnosticConfig) -> Result<(), String> {
        // Verify admin
        let admin: Address = env
            .storage()
            .persistent()
            .get(&Symbol::new(&env, "admin"))
            .ok_or_else(|| String::from_str(&env, "Not initialized"))?;

        if caller != admin {
            return Err(String::from_str(&env, "Unauthorized"));
        }

        env.storage().persistent().set(&Symbol::new(&env, "config"), &config);

        // Emit event
        env.events().publish(
            (Symbol::new(&env, "DIAGNOSTIC"), Symbol::new(&env, "CONFIG_UPDATE")),
            caller,
        );

        Ok(())
    }

    /// Build transaction call tree visualization
    pub fn build_call_tree(env: Env, trace: TransactionTrace) -> String {
        TransactionTracer::build_call_tree(&env, &trace)
    }

    /// Compare state snapshots
    pub fn compare_snapshots(
        env: Env,
        snapshot1: StateSnapshot,
        snapshot2: StateSnapshot,
    ) -> Vec<String> {
        StateTracker::compare_snapshots(&env, &snapshot1, &snapshot2)
    }

    /// Calculate efficiency score
    pub fn calculate_efficiency_score(env: Env, metrics: Vec<PerformanceMetric>) -> u32 {
        PerformanceProfiler::calculate_efficiency_score(&env, &metrics)
    }

    /// Get optimization recommendations for a bottleneck
    pub fn get_recommendations(env: Env, bottleneck: BottleneckReport) -> Vec<String> {
        PerformanceProfiler::generate_recommendations(&env, &bottleneck)
    }

    /// Analyze transaction flow patterns
    pub fn analyze_flow_patterns(env: Env, traces: Vec<TransactionTrace>) -> Map<Symbol, u32> {
        TransactionTracer::analyze_flow_patterns(&env, &traces)
    }

    /// Detect memory leaks from snapshots
    pub fn detect_memory_leak(env: Env, snapshots: Vec<StateSnapshot>) -> bool {
        StateTracker::detect_memory_leak(&env, &snapshots)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_initialize() {
        let env = Env::default();
        let admin = Address::generate(&env);

        let result = DiagnosticsContract::initialize(env.clone(), admin.clone());
        assert!(result.is_ok());

        let config = DiagnosticsContract::get_config(env);
        assert!(config.enable_state_tracking);
    }

    #[test]
    fn test_diagnostic_session() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let contract_id = Address::generate(&env);

        DiagnosticsContract::initialize(env.clone(), admin).unwrap();

        let session_id = DiagnosticsContract::start_session(env.clone(), contract_id);
        assert!(session_id.to_string().contains("session"));

        let ended_session = DiagnosticsContract::end_session(env, session_id).unwrap();
        assert!(!ended_session.is_active);
        assert!(ended_session.ended_at.is_some());
    }

    #[test]
    fn test_state_snapshot() {
        let env = Env::default();
        let contract_id = Address::generate(&env);

        let snapshot = DiagnosticsContract::capture_state_snapshot(env, contract_id);
        assert!(snapshot.timestamp > 0);
        assert!(snapshot.storage_entries > 0);
    }
}