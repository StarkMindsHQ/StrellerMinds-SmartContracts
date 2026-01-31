use crate::types::*;
use soroban_sdk::{Address, Env, Map, String, Symbol, Vec};

/// Performance profiler for detecting bottlenecks
pub struct PerformanceProfiler;

impl PerformanceProfiler {
    /// Record a performance metric
    pub fn record_metric(
        env: &Env,
        contract_id: &Address,
        operation: Symbol,
        execution_time_ms: u32,
        gas_consumed: u64,
        memory_peak_bytes: u64,
        cpu_instructions: u64,
        io_operations: u32,
    ) -> PerformanceMetric {
        let metric_id = Self::generate_metric_id(env, &operation);
        let timestamp = env.ledger().timestamp();

        // Determine if this is a bottleneck
        let is_bottleneck = Self::is_performance_bottleneck(
            execution_time_ms,
            gas_consumed,
            memory_peak_bytes,
        );

        let metric = PerformanceMetric {
            metric_id: metric_id.clone(),
            contract_id: contract_id.clone(),
            operation,
            timestamp,
            execution_time_ms,
            gas_consumed,
            memory_peak_bytes,
            cpu_instructions,
            io_operations,
            is_bottleneck,
        };

        // Store metric for analysis
        Self::store_metric(env, &metric);

        metric
    }

    /// Analyze metrics to identify bottlenecks
    pub fn identify_bottlenecks(
        env: &Env,
        metrics: &Vec<PerformanceMetric>,
        operation_filter: Option<Symbol>,
    ) -> Vec<BottleneckReport> {
        let mut bottleneck_reports = Vec::new(env);

        // Group metrics by operation
        let grouped = Self::group_by_operation(env, metrics, operation_filter);

        for (operation, operation_metrics) in grouped.iter() {
            if operation_metrics.is_empty() {
                continue;
            }

            let report = Self::analyze_operation_performance(
                env,
                &operation,
                &operation_metrics,
            );

            // Only include if severity is Medium or higher
            if matches!(
                report.severity,
                BottleneckSeverity::Medium | BottleneckSeverity::High | BottleneckSeverity::Critical
            ) {
                bottleneck_reports.push_back(report);
            }
        }

        bottleneck_reports
    }

    /// Generate optimization recommendations
    pub fn generate_recommendations(
        env: &Env,
        bottleneck: &BottleneckReport,
    ) -> Vec<String> {
        let mut recommendations = Vec::new(env);

        // High execution time recommendations
        if bottleneck.avg_execution_time > 500 {
            recommendations.push_back(String::from_str(
                env,
                "Consider batching operations to reduce overhead",
            ));
            recommendations.push_back(String::from_str(
                env,
                "Review algorithm complexity - optimize loops and recursion",
            ));
        }

        // High gas usage recommendations
        if bottleneck.avg_gas_usage > 200000 {
            recommendations.push_back(String::from_str(
                env,
                "Optimize storage access - minimize reads and writes",
            ));
            recommendations.push_back(String::from_str(
                env,
                "Consider caching frequently accessed data",
            ));
        }

        // High occurrence count
        if bottleneck.occurrence_count > 100 {
            recommendations.push_back(String::from_str(
                env,
                "This operation is called frequently - optimize for performance",
            ));
            recommendations.push_back(String::from_str(
                env,
                "Consider implementing rate limiting if appropriate",
            ));
        }

        // Severity-specific recommendations
        match bottleneck.severity {
            BottleneckSeverity::Critical => {
                recommendations.push_back(String::from_str(
                    env,
                    "CRITICAL: Immediate optimization required to prevent system degradation",
                ));
            }
            BottleneckSeverity::High => {
                recommendations.push_back(String::from_str(
                    env,
                    "HIGH PRIORITY: Schedule optimization in next development cycle",
                ));
            }
            _ => {}
        }

        recommendations
    }

    /// Get performance comparison between time periods
    pub fn compare_performance(
        env: &Env,
        period1_metrics: &Vec<PerformanceMetric>,
        period2_metrics: &Vec<PerformanceMetric>,
    ) -> Map<Symbol, i64> {
        let mut comparison = Map::new(env);

        let stats1 = Self::calculate_aggregate_stats(env, period1_metrics);
        let stats2 = Self::calculate_aggregate_stats(env, period2_metrics);

        // Calculate percentage changes
        let time_change = Self::calculate_percentage_change(
            stats1.get(Symbol::new(env, "avg_time")).unwrap_or(0),
            stats2.get(Symbol::new(env, "avg_time")).unwrap_or(0),
        );

        let gas_change = Self::calculate_percentage_change(
            stats1.get(Symbol::new(env, "avg_gas")).unwrap_or(0),
            stats2.get(Symbol::new(env, "avg_gas")).unwrap_or(0),
        );

        comparison.set(Symbol::new(env, "time_change_pct"), time_change);
        comparison.set(Symbol::new(env, "gas_change_pct"), gas_change);

        comparison
    }

    /// Calculate CPU efficiency score (0-100)
    pub fn calculate_efficiency_score(
        env: &Env,
        metrics: &Vec<PerformanceMetric>,
    ) -> u32 {
        if metrics.is_empty() {
            return 0;
        }

        let mut total_score = 0u64;

        for metric in metrics {
            let mut score = 100u64;

            // Penalize high execution time (>500ms)
            if metric.execution_time_ms > 500 {
                score = score.saturating_sub(20);
            }

            // Penalize high gas usage (>200000)
            if metric.gas_consumed > 200000 {
                score = score.saturating_sub(20);
            }

            // Penalize high memory usage (>10MB)
            if metric.memory_peak_bytes > 10_000_000 {
                score = score.saturating_sub(15);
            }

            // Penalize excessive IO operations (>50)
            if metric.io_operations > 50 {
                score = score.saturating_sub(10);
            }

            total_score += score;
        }

        (total_score / metrics.len() as u64) as u32
    }

    /// Profile specific operation in real-time
    pub fn profile_operation<F, R>(
        env: &Env,
        contract_id: &Address,
        operation_name: Symbol,
        operation: F,
    ) -> (R, PerformanceMetric)
    where
        F: FnOnce() -> R,
    {
        let start_time = env.ledger().timestamp();
        
        // Execute operation
        let result = operation();
        
        let end_time = env.ledger().timestamp();
        let execution_time_ms = ((end_time - start_time) * 1000) as u32;

        // Record metric with estimates
        let metric = Self::record_metric(
            env,
            contract_id,
            operation_name,
            execution_time_ms,
            50000, // Estimated gas
            1000000, // Estimated memory
            execution_time_ms as u64 * 1000, // Estimated CPU instructions
            5, // Estimated IO ops
        );

        (result, metric)
    }

    // Helper functions

    fn generate_metric_id(env: &Env, operation: &Symbol) -> Symbol {
        Symbol::new(
            env,
            &format!("metric_{}_{}", operation.to_string(), env.ledger().timestamp()),
        )
    }

    fn is_performance_bottleneck(
        execution_time_ms: u32,
        gas_consumed: u64,
        memory_peak_bytes: u64,
    ) -> bool {
        execution_time_ms > 500 || gas_consumed > 200000 || memory_peak_bytes > 10_000_000
    }

    fn store_metric(env: &Env, metric: &PerformanceMetric) {
        env.storage().persistent().set(
            &(Symbol::new(env, "perf_metric"), metric.metric_id.clone()),
            metric,
        );
    }

    fn group_by_operation(
        env: &Env,
        metrics: &Vec<PerformanceMetric>,
        filter: Option<Symbol>,
    ) -> Map<Symbol, Vec<PerformanceMetric>> {
        let mut grouped = Map::new(env);

        for metric in metrics {
            // Apply filter if provided
            if let Some(ref filter_op) = filter {
                if &metric.operation != filter_op {
                    continue;
                }
            }

            let operation = metric.operation.clone();
            let mut operation_metrics = grouped.get(operation.clone()).unwrap_or(Vec::new(env));
            operation_metrics.push_back(metric.clone());
            grouped.set(operation, operation_metrics);
        }

        grouped
    }

    fn analyze_operation_performance(
        env: &Env,
        operation: &Symbol,
        metrics: &Vec<PerformanceMetric>,
    ) -> BottleneckReport {
        let contract_id = metrics.get(0).unwrap().contract_id.clone();
        
        let mut total_time = 0u64;
        let mut max_time = 0u32;
        let mut total_gas = 0u64;
        let mut max_gas = 0u64;

        for metric in metrics {
            total_time += metric.execution_time_ms as u64;
            max_time = max_time.max(metric.execution_time_ms);
            total_gas += metric.gas_consumed;
            max_gas = max_gas.max(metric.gas_consumed);
        }

        let count = metrics.len() as u64;
        let avg_time = (total_time / count) as u32;
        let avg_gas = total_gas / count;

        // Determine severity
        let severity = if avg_time > 1000 || avg_gas > 500000 {
            BottleneckSeverity::Critical
        } else if avg_time > 500 || avg_gas > 200000 {
            BottleneckSeverity::High
        } else if avg_time > 200 || avg_gas > 100000 {
            BottleneckSeverity::Medium
        } else {
            BottleneckSeverity::Low
        };

        let recommendations = Self::generate_recommendations(
            env,
            &BottleneckReport {
                contract_id: contract_id.clone(),
                operation: operation.clone(),
                severity,
                avg_execution_time: avg_time,
                max_execution_time: max_time,
                avg_gas_usage: avg_gas,
                max_gas_usage: max_gas,
                occurrence_count: count as u32,
                recommendations: Vec::new(env),
            },
        );

        BottleneckReport {
            contract_id,
            operation: operation.clone(),
            severity,
            avg_execution_time: avg_time,
            max_execution_time: max_time,
            avg_gas_usage: avg_gas,
            max_gas_usage: max_gas,
            occurrence_count: count as u32,
            recommendations,
        }
    }

    fn calculate_aggregate_stats(
        env: &Env,
        metrics: &Vec<PerformanceMetric>,
    ) -> Map<Symbol, u64> {
        let mut stats = Map::new(env);

        if metrics.is_empty() {
            return stats;
        }

        let mut total_time = 0u64;
        let mut total_gas = 0u64;

        for metric in metrics {
            total_time += metric.execution_time_ms as u64;
            total_gas += metric.gas_consumed;
        }

        let count = metrics.len() as u64;
        stats.set(Symbol::new(env, "avg_time"), total_time / count);
        stats.set(Symbol::new(env, "avg_gas"), total_gas / count);

        stats
    }

    fn calculate_percentage_change(old_value: u64, new_value: u64) -> i64 {
        if old_value == 0 {
            return 0;
        }

        let change = new_value as i64 - old_value as i64;
        (change * 100) / old_value as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_record_metric() {
        let env = Env::default();
        let contract_id = Address::generate(&env);
        let operation = Symbol::new(&env, "test_operation");

        let metric = PerformanceProfiler::record_metric(
            &env,
            &contract_id,
            operation,
            100,
            50000,
            1000000,
            100000,
            5,
        );

        assert_eq!(metric.contract_id, contract_id);
        assert_eq!(metric.execution_time_ms, 100);
        assert_eq!(metric.gas_consumed, 50000);
    }

    #[test]
    fn test_identify_bottlenecks() {
        let env = Env::default();
        let contract_id = Address::generate(&env);

        let mut metrics = Vec::new(&env);

        // Add some slow metrics
        for i in 0..5 {
            let metric = PerformanceMetric {
                metric_id: Symbol::new(&env, &format!("metric_{}", i)),
                contract_id: contract_id.clone(),
                operation: Symbol::new(&env, "slow_op"),
                timestamp: env.ledger().timestamp(),
                execution_time_ms: 600, // Above threshold
                gas_consumed: 250000, // Above threshold
                memory_peak_bytes: 5000000,
                cpu_instructions: 600000,
                io_operations: 10,
                is_bottleneck: true,
            };
            metrics.push_back(metric);
        }

        let bottlenecks = PerformanceProfiler::identify_bottlenecks(&env, &metrics, None);

        assert!(!bottlenecks.is_empty());
        assert!(matches!(
            bottlenecks.get(0).unwrap().severity,
            BottleneckSeverity::High | BottleneckSeverity::Critical
        ));
    }

    #[test]
    fn test_calculate_efficiency_score() {
        let env = Env::default();
        let contract_id = Address::generate(&env);

        let mut metrics = Vec::new(&env);

        // Add efficient metrics
        for i in 0..5 {
            let metric = PerformanceMetric {
                metric_id: Symbol::new(&env, &format!("metric_{}", i)),
                contract_id: contract_id.clone(),
                operation: Symbol::new(&env, "efficient_op"),
                timestamp: env.ledger().timestamp(),
                execution_time_ms: 50,
                gas_consumed: 30000,
                memory_peak_bytes: 500000,
                cpu_instructions: 50000,
                io_operations: 3,
                is_bottleneck: false,
            };
            metrics.push_back(metric);
        }

        let score = PerformanceProfiler::calculate_efficiency_score(&env, &metrics);

        assert!(score >= 85); // Should have high efficiency
    }
}