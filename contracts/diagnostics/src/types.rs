use soroban_sdk::{contracttype, Address, BytesN, Map, String, Symbol, Vec};

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
    pub error_message: Option<String>,
    pub child_calls: Vec<Symbol>,
    pub events_emitted: u32,
}

/// Performance metric for profiling
#[contracttype]
#[derive(Clone, Debug)]
pub struct PerformanceMetric {
    pub metric_id: Symbol,
    pub contract_id: Address,
    pub operation: Symbol,
    pub timestamp: u64,
    pub execution_time_ms: u32,
    pub gas_consumed: u64,
    pub memory_peak_bytes: u64,
    pub cpu_instructions: u64,
    pub io_operations: u32,
    pub is_bottleneck: bool,
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

/// Types of anomalies that can be detected
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnomalyType {
    UnusualGasSpike,
    MemoryLeak,
    SlowExecution,
    HighErrorRate,
    StateInconsistency,
    UnexpectedBehavior,
}

/// Anomaly severity levels
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnomalySeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Real-time diagnostic session
#[contracttype]
#[derive(Clone, Debug)]
pub struct DiagnosticSession {
    pub session_id: Symbol,
    pub contract_id: Address,
    pub started_at: u64,
    pub ended_at: Option<u64>,
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