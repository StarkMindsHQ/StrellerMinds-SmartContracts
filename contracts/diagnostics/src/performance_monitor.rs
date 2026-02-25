use crate::{
    errors::DiagnosticsError, events::DiagnosticsEvents, storage::DiagnosticsStorage, types::*,
};
use soroban_sdk::{Address, BytesN, Env, String};

/// Real-time performance monitoring and profiling engine
pub struct PerformanceMonitor;

impl PerformanceMonitor {
    /// Start monitoring performance for a contract
    pub fn start_monitoring(
        env: &Env,
        contract_address: &Address,
        config: MonitoringConfig,
    ) -> Result<BytesN<32>, DiagnosticsError> {
        // Validate configuration
        if config.metrics_collection_interval == 0 || config.max_metrics_history == 0 {
            return Err(DiagnosticsError::InvalidConfiguration);
        }

        // Store monitoring configuration
        DiagnosticsStorage::set_monitoring_config(env, contract_address, &config);

        // Add to monitored contracts list
        DiagnosticsStorage::add_monitored_contract(env, contract_address);

        // Generate monitoring ID
        let monitoring_id = Self::generate_monitoring_id(env, contract_address);

        // Emit event
        DiagnosticsEvents::emit_monitoring_started(env, contract_address, &monitoring_id);

        Ok(monitoring_id)
    }

    /// Stop monitoring performance for a contract
    pub fn stop_monitoring(env: &Env, contract_address: &Address) -> Result<(), DiagnosticsError> {
        // Remove from monitored contracts
        DiagnosticsStorage::remove_monitored_contract(env, contract_address);

        // Emit event
        DiagnosticsEvents::emit_monitoring_stopped(env, contract_address);

        Ok(())
    }

    /// Record performance metrics for a contract
    pub fn record_metrics(
        env: &Env,
        contract_address: &Address,
        metrics: PerformanceMetrics,
    ) -> Result<(), DiagnosticsError> {
        // Validate metrics
        Self::validate_metrics(&metrics)?;

        // Store metrics
        DiagnosticsStorage::store_performance_metrics(env, contract_address, &metrics);

        // Check for performance alerts
        Self::check_performance_alerts(env, contract_address, &metrics)?;

        // Emit event
        DiagnosticsEvents::emit_metrics_recorded(
            env,
            contract_address,
            metrics.timestamp,
            metrics.execution_time,
            metrics.gas_used,
        );

        Ok(())
    }

    /// Get current performance metrics for a contract
    pub fn get_current_metrics(
        env: &Env,
        contract_address: &Address,
    ) -> Result<PerformanceMetrics, DiagnosticsError> {
        DiagnosticsStorage::get_latest_performance_metrics(env, contract_address)
            .ok_or(DiagnosticsError::MetricsNotFound)
    }

    /// Generate historical performance report
    pub fn generate_performance_report(
        env: &Env,
        contract_address: &Address,
        start_time: u64,
        end_time: u64,
    ) -> Result<PerformanceReport, DiagnosticsError> {
        let mut total_transactions = 0;
        let mut total_execution_time = 0;
        let mut total_gas_used = 0;
        let mut total_errors = 0;
        let mut peak_memory = 0;
        let mut min_execution_time = u64::MAX;
        let mut max_execution_time = 0;

        // Collect metrics for the time period
        for timestamp in start_time..=end_time {
            if let Some(metrics) =
                DiagnosticsStorage::get_performance_metrics(env, contract_address, timestamp)
            {
                total_transactions += metrics.transaction_count;
                total_execution_time += metrics.average_execution_time;
                total_gas_used += metrics.gas_used;
                total_errors += metrics.error_count;
                peak_memory = peak_memory.max(metrics.peak_memory_usage);
                min_execution_time = min_execution_time.min(metrics.execution_time);
                max_execution_time = max_execution_time.max(metrics.execution_time);
            }
        }

        if total_transactions == 0 {
            return Err(DiagnosticsError::MetricsNotFound);
        }

        let average_execution_time = if total_transactions > 0 {
            total_execution_time / total_transactions as u64
        } else {
            0
        };

        let error_rate = if total_transactions > 0 {
            (total_errors * 100) / total_transactions
        } else {
            0
        };

        Ok(PerformanceReport {
            contract_address: contract_address.clone(),
            start_time,
            end_time,
            total_transactions,
            average_execution_time,
            min_execution_time: if min_execution_time == u64::MAX {
                0
            } else {
                min_execution_time
            },
            max_execution_time,
            total_gas_used,
            peak_memory_usage: peak_memory,
            error_rate,
            performance_score: Self::calculate_performance_score(
                average_execution_time,
                error_rate,
                total_gas_used,
            ),
        })
    }

    /// Profile a specific operation
    pub fn profile_operation(
        env: &Env,
        contract_address: &Address,
        operation_name: &str,
        start_time: u64,
        end_time: u64,
        gas_used: u64,
        memory_used: u32,
        success: bool,
    ) -> Result<OperationProfile, DiagnosticsError> {
        let execution_time = end_time - start_time;
        let timestamp = env.ledger().timestamp();

        let profile = OperationProfile {
            operation_name: String::from_str(env, operation_name),
            contract_address: contract_address.clone(),
            timestamp,
            execution_time,
            gas_used,
            memory_used,
            success,
            efficiency_score: Self::calculate_efficiency_score(
                execution_time,
                gas_used,
                memory_used,
            ),
        };

        Ok(profile)
    }

    /// Generate monitoring ID
    fn generate_monitoring_id(env: &Env, _contract_address: &Address) -> BytesN<32> {
        let timestamp = env.ledger().timestamp();
        let sequence = env.ledger().sequence();
        let mut data = [0u8; 32];

        // Use timestamp and sequence to create unique ID
        let ts_bytes = timestamp.to_be_bytes();
        let seq_bytes = sequence.to_be_bytes();

        for i in 0..8 {
            data[i] = ts_bytes[i];
            data[i + 8] = seq_bytes[i];
        }

        BytesN::from_array(env, &data)
    }

    /// Validate performance metrics
    fn validate_metrics(metrics: &PerformanceMetrics) -> Result<(), DiagnosticsError> {
        if metrics.execution_time == 0 && metrics.transaction_count > 0 {
            return Err(DiagnosticsError::InvalidMetrics);
        }

        if metrics.error_rate > 100 {
            return Err(DiagnosticsError::InvalidMetrics);
        }

        if metrics.cpu_utilization > 100 {
            return Err(DiagnosticsError::InvalidMetrics);
        }

        Ok(())
    }

    /// Check for performance alerts
    fn check_performance_alerts(
        env: &Env,
        contract_address: &Address,
        metrics: &PerformanceMetrics,
    ) -> Result<(), DiagnosticsError> {
        if let Ok(config) = DiagnosticsStorage::get_config(env) {
            // Check CPU utilization alert
            if metrics.cpu_utilization > config.alert_threshold_cpu {
                DiagnosticsEvents::emit_performance_alert(
                    env,
                    contract_address,
                    &String::from_str(env, "CPU_UTILIZATION_HIGH"),
                    config.alert_threshold_cpu as u64,
                    metrics.cpu_utilization as u64,
                );
            }

            // Check memory usage alert
            if metrics.memory_usage > config.alert_threshold_memory {
                DiagnosticsEvents::emit_performance_alert(
                    env,
                    contract_address,
                    &String::from_str(env, "MEMORY_USAGE_HIGH"),
                    config.alert_threshold_memory as u64,
                    metrics.memory_usage as u64,
                );
            }

            // Check gas usage alert
            if metrics.gas_used > config.alert_threshold_gas {
                DiagnosticsEvents::emit_performance_alert(
                    env,
                    contract_address,
                    &String::from_str(env, "GAS_USAGE_HIGH"),
                    config.alert_threshold_gas,
                    metrics.gas_used,
                );
            }

            // Check error rate alert (above 5%)
            if metrics.error_rate > 5 {
                DiagnosticsEvents::emit_performance_alert(
                    env,
                    contract_address,
                    &String::from_str(env, "ERROR_RATE_HIGH"),
                    5,
                    metrics.error_rate as u64,
                );
            }
        }

        Ok(())
    }

    /// Calculate performance score (0-100)
    fn calculate_performance_score(avg_execution_time: u64, error_rate: u32, gas_used: u64) -> u32 {
        let mut score = 100u32;

        // Penalize high execution times (above 1000ms)
        if avg_execution_time > 1000 {
            score = score.saturating_sub((avg_execution_time / 100) as u32);
        }

        // Penalize error rates
        score = score.saturating_sub(error_rate * 5);

        // Penalize high gas usage (above 1M gas)
        if gas_used > 1_000_000 {
            score = score.saturating_sub((gas_used / 100_000) as u32);
        }

        score.min(100)
    }

    /// Calculate efficiency score for operations
    fn calculate_efficiency_score(execution_time: u64, gas_used: u64, memory_used: u32) -> u32 {
        let mut score = 100u32;

        // Efficiency decreases with higher resource usage
        if execution_time > 500 {
            score = score.saturating_sub((execution_time / 50) as u32);
        }

        if gas_used > 100_000 {
            score = score.saturating_sub((gas_used / 10_000) as u32);
        }

        if memory_used > 1024 {
            score = score.saturating_sub(memory_used / 100);
        }

        score.min(100)
    }
}

/// Additional types for performance monitoring
use soroban_sdk::contracttype;

#[derive(Clone, Debug)]
#[contracttype]
pub struct PerformanceReport {
    pub contract_address: Address,
    pub start_time: u64,
    pub end_time: u64,
    pub total_transactions: u32,
    pub average_execution_time: u64,
    pub min_execution_time: u64,
    pub max_execution_time: u64,
    pub total_gas_used: u64,
    pub peak_memory_usage: u32,
    pub error_rate: u32,
    pub performance_score: u32,
}

#[derive(Clone, Debug)]
#[contracttype]
pub struct OperationProfile {
    pub operation_name: String,
    pub contract_address: Address,
    pub timestamp: u64,
    pub execution_time: u64,
    pub gas_used: u64,
    pub memory_used: u32,
    pub success: bool,
    pub efficiency_score: u32,
}
