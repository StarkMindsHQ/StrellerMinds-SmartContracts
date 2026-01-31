use crate::types::*;
use soroban_sdk::{Address, Env, Map, String, Symbol, Vec};

/// Automated anomaly detection system
pub struct AnomalyDetector;

impl AnomalyDetector {
    /// Detect anomalies in performance metrics
    pub fn detect_anomalies(
        env: &Env,
        contract_id: &Address,
        recent_metrics: &Vec<PerformanceMetric>,
        baseline_metrics: &Vec<PerformanceMetric>,
    ) -> Vec<AnomalyReport> {
        let mut anomalies = Vec::new(env);

        // Calculate baseline statistics
        let baseline_stats = Self::calculate_baseline_stats(env, baseline_metrics);

        // Check for gas usage spikes
        if let Some(gas_anomaly) = Self::detect_gas_spike(env, contract_id, recent_metrics, &baseline_stats) {
            anomalies.push_back(gas_anomaly);
        }

        // Check for slow execution
        if let Some(slow_anomaly) = Self::detect_slow_execution(env, contract_id, recent_metrics, &baseline_stats) {
            anomalies.push_back(slow_anomaly);
        }

        // Check for memory leaks
        if let Some(memory_anomaly) = Self::detect_memory_pattern_anomaly(env, contract_id, recent_metrics) {
            anomalies.push_back(memory_anomaly);
        }

        // Check for high error rates
        if let Some(error_anomaly) = Self::detect_high_error_rate(env, contract_id) {
            anomalies.push_back(error_anomaly);
        }

        anomalies
    }

    /// Detect unusual gas consumption spikes
    fn detect_gas_spike(
        env: &Env,
        contract_id: &Address,
        recent_metrics: &Vec<PerformanceMetric>,
        baseline_stats: &Map<Symbol, u64>,
    ) -> Option<AnomalyReport> {
        if recent_metrics.is_empty() {
            return None;
        }

        let baseline_avg_gas = baseline_stats
            .get(Symbol::new(env, "avg_gas"))
            .unwrap_or(50000);

        // Calculate recent average
        let mut total_gas = 0u64;
        let mut spike_operations = Vec::new(env);

        for metric in recent_metrics {
            total_gas += metric.gas_consumed;
            
            // Check if individual metric is a spike (3x baseline)
            if metric.gas_consumed > baseline_avg_gas * 3 {
                spike_operations.push_back(metric.operation.clone());
            }
        }

        let recent_avg_gas = total_gas / recent_metrics.len() as u64;

        // If recent average is 2x+ baseline, it's an anomaly
        if recent_avg_gas > baseline_avg_gas * 2 {
            let severity = if recent_avg_gas > baseline_avg_gas * 5 {
                AnomalySeverity::Critical
            } else if recent_avg_gas > baseline_avg_gas * 3 {
                AnomalySeverity::Error
            } else {
                AnomalySeverity::Warning
            };

            let mut suggested_fixes = Vec::new(env);
            suggested_fixes.push_back(String::from_str(
                env,
                "Review recent code changes for gas optimization",
            ));
            suggested_fixes.push_back(String::from_str(
                env,
                "Check for unnecessary storage operations",
            ));
            suggested_fixes.push_back(String::from_str(
                env,
                "Consider implementing caching for frequently accessed data",
            ));

            Some(AnomalyReport {
                anomaly_id: Symbol::new(env, "gas_spike"),
                contract_id: contract_id.clone(),
                detected_at: env.ledger().timestamp(),
                anomaly_type: AnomalyType::UnusualGasSpike,
                severity,
                description: String::from_str(
                    env,
                    &format!(
                        "Gas consumption increased from {} to {} ({}% increase)",
                        baseline_avg_gas,
                        recent_avg_gas,
                        ((recent_avg_gas - baseline_avg_gas) * 100) / baseline_avg_gas
                    ),
                ),
                affected_operations: spike_operations,
                root_cause_analysis: String::from_str(
                    env,
                    "Possible causes: increased storage operations, inefficient loops, or new features",
                ),
                suggested_fixes,
            })
        } else {
            None
        }
    }

    /// Detect slow execution patterns
    fn detect_slow_execution(
        env: &Env,
        contract_id: &Address,
        recent_metrics: &Vec<PerformanceMetric>,
        baseline_stats: &Map<Symbol, u64>,
    ) -> Option<AnomalyReport> {
        if recent_metrics.is_empty() {
            return None;
        }

        let baseline_avg_time = baseline_stats
            .get(Symbol::new(env, "avg_time"))
            .unwrap_or(100) as u32;

        let mut total_time = 0u64;
        let mut slow_operations = Vec::new(env);

        for metric in recent_metrics {
            total_time += metric.execution_time_ms as u64;
            
            // Check if individual metric is slow (3x baseline)
            if metric.execution_time_ms > baseline_avg_time * 3 {
                slow_operations.push_back(metric.operation.clone());
            }
        }

        let recent_avg_time = (total_time / recent_metrics.len() as u64) as u32;

        // If recent average is 2x+ baseline, it's an anomaly
        if recent_avg_time > baseline_avg_time * 2 {
            let severity = if recent_avg_time > baseline_avg_time * 5 {
                AnomalySeverity::Critical
            } else if recent_avg_time > baseline_avg_time * 3 {
                AnomalySeverity::Error
            } else {
                AnomalySeverity::Warning
            };

            let mut suggested_fixes = Vec::new(env);
            suggested_fixes.push_back(String::from_str(
                env,
                "Profile code to identify performance bottlenecks",
            ));
            suggested_fixes.push_back(String::from_str(
                env,
                "Optimize database queries and storage access",
            ));
            suggested_fixes.push_back(String::from_str(
                env,
                "Consider parallel processing for independent operations",
            ));

            Some(AnomalyReport {
                anomaly_id: Symbol::new(env, "slow_exec"),
                contract_id: contract_id.clone(),
                detected_at: env.ledger().timestamp(),
                anomaly_type: AnomalyType::SlowExecution,
                severity,
                description: String::from_str(
                    env,
                    &format!(
                        "Execution time increased from {}ms to {}ms",
                        baseline_avg_time, recent_avg_time
                    ),
                ),
                affected_operations: slow_operations,
                root_cause_analysis: String::from_str(
                    env,
                    "Possible causes: inefficient algorithms, blocking operations, or resource contention",
                ),
                suggested_fixes,
            })
        } else {
            None
        }
    }

    /// Detect potential memory leaks
    fn detect_memory_pattern_anomaly(
        env: &Env,
        contract_id: &Address,
        recent_metrics: &Vec<PerformanceMetric>,
    ) -> Option<AnomalyReport> {
        if recent_metrics.len() < 5 {
            return None;
        }

        // Check for consistently increasing memory usage
        let mut consecutive_increases = 0u32;
        
        for i in 1..recent_metrics.len() {
            let prev = recent_metrics.get(i - 1).unwrap();
            let current = recent_metrics.get(i).unwrap();
            
            if current.memory_peak_bytes > prev.memory_peak_bytes {
                consecutive_increases += 1;
            } else {
                consecutive_increases = 0;
            }
        }

        // If memory increased 4+ times consecutively, potential leak
        if consecutive_increases >= 4 {
            let mut suggested_fixes = Vec::new(env);
            suggested_fixes.push_back(String::from_str(
                env,
                "Review data structures for proper cleanup",
            ));
            suggested_fixes.push_back(String::from_str(
                env,
                "Check for unbounded collections or caches",
            ));
            suggested_fixes.push_back(String::from_str(
                env,
                "Implement proper resource disposal",
            ));

            Some(AnomalyReport {
                anomaly_id: Symbol::new(env, "memory_leak"),
                contract_id: contract_id.clone(),
                detected_at: env.ledger().timestamp(),
                anomaly_type: AnomalyType::MemoryLeak,
                severity: AnomalySeverity::Error,
                description: String::from_str(
                    env,
                    "Memory usage shows consistent upward trend indicating potential leak",
                ),
                affected_operations: Vec::new(env),
                root_cause_analysis: String::from_str(
                    env,
                    "Memory is not being released properly after operations",
                ),
                suggested_fixes,
            })
        } else {
            None
        }
    }

    /// Detect high error rates
    fn detect_high_error_rate(
        env: &Env,
        contract_id: &Address,
    ) -> Option<AnomalyReport> {
        // In production, this would check actual error rates from transaction traces
        // For now, we'll use a simplified check
        
        // Simulating error rate detection
        let error_rate = Self::get_recent_error_rate(env);
        
        if error_rate > 20 { // More than 20% error rate
            let severity = if error_rate > 50 {
                AnomalySeverity::Critical
            } else if error_rate > 30 {
                AnomalySeverity::Error
            } else {
                AnomalySeverity::Warning
            };

            let mut suggested_fixes = Vec::new(env);
            suggested_fixes.push_back(String::from_str(
                env,
                "Review error logs to identify common failure patterns",
            ));
            suggested_fixes.push_back(String::from_str(
                env,
                "Add input validation and error handling",
            ));
            suggested_fixes.push_back(String::from_str(
                env,
                "Implement circuit breakers for failing operations",
            ));

            Some(AnomalyReport {
                anomaly_id: Symbol::new(env, "high_errors"),
                contract_id: contract_id.clone(),
                detected_at: env.ledger().timestamp(),
                anomaly_type: AnomalyType::HighErrorRate,
                severity,
                description: String::from_str(
                    env,
                    &format!("Error rate at {}% (threshold: 20%)", error_rate),
                ),
                affected_operations: Vec::new(env),
                root_cause_analysis: String::from_str(
                    env,
                    "Multiple operations failing - check logs for patterns",
                ),
                suggested_fixes,
            })
        } else {
            None
        }
    }

    /// Generate root cause analysis for an anomaly
    pub fn analyze_root_cause(
        env: &Env,
        anomaly_type: AnomalyType,
        metrics: &Vec<PerformanceMetric>,
    ) -> String {
        match anomaly_type {
            AnomalyType::UnusualGasSpike => {
                String::from_str(
                    env,
                    "Analysis: Gas spike likely caused by increased storage operations or computation complexity",
                )
            }
            AnomalyType::MemoryLeak => {
                String::from_str(
                    env,
                    "Analysis: Memory not being freed - check for unbounded data structures",
                )
            }
            AnomalyType::SlowExecution => {
                String::from_str(
                    env,
                    "Analysis: Performance degradation - review algorithm efficiency",
                )
            }
            AnomalyType::HighErrorRate => {
                String::from_str(
                    env,
                    "Analysis: High failure rate - validate inputs and error handling",
                )
            }
            AnomalyType::StateInconsistency => {
                String::from_str(
                    env,
                    "Analysis: State changes not matching expected patterns",
                )
            }
            AnomalyType::UnexpectedBehavior => {
                String::from_str(
                    env,
                    "Analysis: Behavior deviating from normal patterns",
                )
            }
        }
    }

    /// Get anomaly severity score (0-100)
    pub fn calculate_severity_score(anomaly: &AnomalyReport) -> u32 {
        match anomaly.severity {
            AnomalySeverity::Info => 25,
            AnomalySeverity::Warning => 50,
            AnomalySeverity::Error => 75,
            AnomalySeverity::Critical => 100,
        }
    }

    // Helper functions

    fn calculate_baseline_stats(
        env: &Env,
        baseline_metrics: &Vec<PerformanceMetric>,
    ) -> Map<Symbol, u64> {
        let mut stats = Map::new(env);

        if baseline_metrics.is_empty() {
            stats.set(Symbol::new(env, "avg_gas"), 50000);
            stats.set(Symbol::new(env, "avg_time"), 100);
            return stats;
        }

        let mut total_gas = 0u64;
        let mut total_time = 0u64;

        for metric in baseline_metrics {
            total_gas += metric.gas_consumed;
            total_time += metric.execution_time_ms as u64;
        }

        let count = baseline_metrics.len() as u64;
        stats.set(Symbol::new(env, "avg_gas"), total_gas / count);
        stats.set(Symbol::new(env, "avg_time"), total_time / count);

        stats
    }

    fn get_recent_error_rate(_env: &Env) -> u32 {
        // In production, calculate from actual transaction traces
        // For now, return a simulated value
        15 // 15% error rate
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_detect_gas_spike() {
        let env = Env::default();
        let contract_id = Address::generate(&env);

        // Create baseline metrics (normal gas usage)
        let mut baseline = Vec::new(&env);
        for i in 0..10 {
            baseline.push_back(PerformanceMetric {
                metric_id: Symbol::new(&env, &format!("baseline_{}", i)),
                contract_id: contract_id.clone(),
                operation: Symbol::new(&env, "normal_op"),
                timestamp: env.ledger().timestamp(),
                execution_time_ms: 100,
                gas_consumed: 50000,
                memory_peak_bytes: 1000000,
                cpu_instructions: 100000,
                io_operations: 5,
                is_bottleneck: false,
            });
        }

        // Create recent metrics with gas spike
        let mut recent = Vec::new(&env);
        for i in 0..5 {
            recent.push_back(PerformanceMetric {
                metric_id: Symbol::new(&env, &format!("recent_{}", i)),
                contract_id: contract_id.clone(),
                operation: Symbol::new(&env, "spike_op"),
                timestamp: env.ledger().timestamp(),
                execution_time_ms: 100,
                gas_consumed: 150000, // 3x baseline
                memory_peak_bytes: 1000000,
                cpu_instructions: 100000,
                io_operations: 5,
                is_bottleneck: true,
            });
        }

        let anomalies = AnomalyDetector::detect_anomalies(
            &env,
            &contract_id,
            &recent,
            &baseline,
        );

        assert!(!anomalies.is_empty());
        let first_anomaly = anomalies.get(0).unwrap();
        assert_eq!(first_anomaly.anomaly_type, AnomalyType::UnusualGasSpike);
    }

    #[test]
    fn test_calculate_severity_score() {
        let env = Env::default();
        let contract_id = Address::generate(&env);

        let critical_anomaly = AnomalyReport {
            anomaly_id: Symbol::new(&env, "test"),
            contract_id,
            detected_at: env.ledger().timestamp(),
            anomaly_type: AnomalyType::UnusualGasSpike,
            severity: AnomalySeverity::Critical,
            description: String::from_str(&env, "Test"),
            affected_operations: Vec::new(&env),
            root_cause_analysis: String::from_str(&env, "Test"),
            suggested_fixes: Vec::new(&env),
        };

        let score = AnomalyDetector::calculate_severity_score(&critical_anomaly);
        assert_eq!(score, 100);
    }
}