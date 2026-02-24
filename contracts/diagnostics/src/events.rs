use crate::types::*;
use soroban_sdk::{Address, BytesN, Env, String, Symbol};

/// Events emitted by the diagnostics platform
pub struct DiagnosticsEvents;

impl DiagnosticsEvents {
    /// Emit initialization event
    pub fn emit_initialized(env: &Env, admin: &Address) {
        env.events()
            .publish(("DIAGNOSTICS", "INITIALIZED"), (admin,));
    }

    /// Emit performance monitoring started event
    pub fn emit_monitoring_started(
        env: &Env,
        contract_address: &Address,
        monitoring_id: &BytesN<32>,
    ) {
        env.events().publish(
            ("DIAGNOSTICS", "MONITORING_STARTED"),
            (contract_address, monitoring_id),
        );
    }

    /// Emit performance monitoring stopped event
    pub fn emit_monitoring_stopped(env: &Env, contract_address: &Address) {
        env.events()
            .publish(("DIAGNOSTICS", "MONITORING_STOPPED"), (contract_address,));
    }

    /// Emit performance metrics recorded event
    pub fn emit_metrics_recorded(
        env: &Env,
        contract_address: &Address,
        timestamp: u64,
        execution_time: u64,
        gas_used: u64,
    ) {
        env.events().publish(
            ("DIAGNOSTICS", "METRICS_RECORDED"),
            (contract_address, timestamp, execution_time, gas_used),
        );
    }

    /// Emit capacity prediction generated event
    pub fn emit_prediction_generated(
        env: &Env,
        contract_address: &Address,
        prediction_id: &BytesN<32>,
        confidence_score: u32,
    ) {
        env.events().publish(
            ("DIAGNOSTICS", "PREDICTION_GENERATED"),
            (contract_address, prediction_id, confidence_score),
        );
    }

    /// Emit behavior analysis completed event
    pub fn emit_behavior_analysis_completed(
        env: &Env,
        user: &Address,
        analysis_id: &BytesN<32>,
        effectiveness_trend: &EffectivenessTrend,
    ) {
        let trend_str = match effectiveness_trend {
            EffectivenessTrend::Improving => "IMPROVING",
            EffectivenessTrend::Stable => "STABLE",
            EffectivenessTrend::Declining => "DECLINING",
            EffectivenessTrend::Inconsistent => "INCONSISTENT",
        };

        env.events().publish(
            ("DIAGNOSTICS", "BEHAVIOR_ANALYSIS_COMPLETED"),
            (user, analysis_id, Symbol::new(env, trend_str)),
        );
    }

    /// Emit optimization recommendations generated event
    pub fn emit_optimization_recommendations_generated(
        env: &Env,
        contract_address: &Address,
        recommendation_count: u32,
        total_potential_savings: u64,
    ) {
        env.events().publish(
            ("DIAGNOSTICS", "OPTIMIZATION_RECOMMENDATIONS"),
            (
                contract_address,
                recommendation_count,
                total_potential_savings,
            ),
        );
    }

    /// Emit distributed trace started event
    pub fn emit_trace_started(
        env: &Env,
        trace_id: &BytesN<32>,
        trace_name: &String,
        contract_address: &Address,
    ) {
        env.events().publish(
            ("DIAGNOSTICS", "TRACE_STARTED"),
            (trace_id, trace_name.clone(), contract_address.clone()),
        );
    }

    /// Emit trace span added event
    pub fn emit_trace_span_added(
        env: &Env,
        trace_id: &BytesN<32>,
        span_id: &BytesN<32>,
        operation_name: &String,
        duration: u64,
    ) {
        env.events().publish(
            ("DIAGNOSTICS", "TRACE_SPAN_ADDED"),
            (
                trace_id.clone(),
                span_id.clone(),
                operation_name.clone(),
                duration,
            ),
        );
    }

    /// Emit trace completed event
    pub fn emit_trace_completed(
        env: &Env,
        trace_id: &BytesN<32>,
        total_duration: u64,
        span_count: u32,
        bottleneck_count: u32,
    ) {
        env.events().publish(
            ("DIAGNOSTICS", "TRACE_COMPLETED"),
            (trace_id, total_duration, span_count, bottleneck_count),
        );
    }

    /// Emit benchmark started event
    pub fn emit_benchmark_started(
        env: &Env,
        benchmark_id: &BytesN<32>,
        benchmark_name: &String,
        target_contracts_count: u32,
    ) {
        env.events().publish(
            ("DIAGNOSTICS", "BENCHMARK_STARTED"),
            (
                benchmark_id.clone(),
                benchmark_name.clone(),
                target_contracts_count,
            ),
        );
    }

    /// Emit benchmark completed event
    pub fn emit_benchmark_completed(
        env: &Env,
        benchmark_id: &BytesN<32>,
        execution_time: u64,
        overall_score: u32,
        regression_detected: bool,
    ) {
        env.events().publish(
            ("DIAGNOSTICS", "BENCHMARK_COMPLETED"),
            (
                benchmark_id,
                execution_time,
                overall_score,
                regression_detected,
            ),
        );
    }

    /// Emit anomaly detected event
    pub fn emit_anomaly_detected(
        env: &Env,
        anomaly_id: &BytesN<32>,
        contract_address: &Address,
        anomaly_type: &AnomalyType,
        severity: &RiskLevel,
    ) {
        let type_str = match anomaly_type {
            AnomalyType::PerformanceDegradation => "PERFORMANCE_DEGRADATION",
            AnomalyType::MemoryLeak => "MEMORY_LEAK",
            AnomalyType::GasSpike => "GAS_SPIKE",
            AnomalyType::ErrorRateSpike => "ERROR_RATE_SPIKE",
            AnomalyType::ThroughputDrop => "THROUGHPUT_DROP",
            AnomalyType::LatencyIncrease => "LATENCY_INCREASE",
            AnomalyType::ResourceExhaustion => "RESOURCE_EXHAUSTION",
            AnomalyType::UnusualPatterns => "UNUSUAL_PATTERNS",
            AnomalyType::StateInconsistency => "STATE_INCONSISTENCY",
        };

        let severity_str = match severity {
            RiskLevel::Low => "LOW",
            RiskLevel::Medium => "MEDIUM",
            RiskLevel::High => "HIGH",
            RiskLevel::Critical => "CRITICAL",
        };

        env.events().publish(
            ("DIAGNOSTICS", "ANOMALY_DETECTED"),
            (
                anomaly_id,
                contract_address,
                Symbol::new(env, type_str),
                Symbol::new(env, severity_str),
            ),
        );
    }

    /// Emit resource utilization analyzed event
    pub fn emit_resource_utilization_analyzed(
        env: &Env,
        analysis_id: &BytesN<32>,
        contract_address: &Address,
        total_cost: u64,
        optimization_opportunities_count: u32,
    ) {
        env.events().publish(
            ("DIAGNOSTICS", "RESOURCE_UTILIZATION_ANALYZED"),
            (
                analysis_id,
                contract_address,
                total_cost,
                optimization_opportunities_count,
            ),
        );
    }

    /// Emit regression test completed event
    pub fn emit_regression_test_completed(
        env: &Env,
        report_id: &BytesN<32>,
        test_name: &String,
        regression_detected: bool,
        verdict: &TestVerdict,
    ) {
        let verdict_str = match verdict {
            TestVerdict::Pass => "PASS",
            TestVerdict::Warning => "WARNING",
            TestVerdict::Fail => "FAIL",
            TestVerdict::Inconclusive => "INCONCLUSIVE",
        };

        env.events().publish(
            ("DIAGNOSTICS", "REGRESSION_TEST_COMPLETED"),
            (
                report_id.clone(),
                test_name.clone(),
                regression_detected,
                Symbol::new(env, verdict_str),
            ),
        );
    }

    /// Emit system health report generated event
    pub fn emit_system_health_report_generated(
        env: &Env,
        timestamp: u64,
        overall_health: &HealthStatus,
        total_contracts: u32,
        active_contracts: u32,
        anomaly_count: u32,
    ) {
        let health_str = match overall_health {
            HealthStatus::Healthy => "HEALTHY",
            HealthStatus::Warning => "WARNING",
            HealthStatus::Critical => "CRITICAL",
            HealthStatus::Unknown => "UNKNOWN",
        };

        env.events().publish(
            ("DIAGNOSTICS", "SYSTEM_HEALTH_REPORT"),
            (
                timestamp,
                Symbol::new(env, health_str),
                total_contracts,
                active_contracts,
                anomaly_count,
            ),
        );
    }

    /// Emit performance alert event
    pub fn emit_performance_alert(
        env: &Env,
        contract_address: &Address,
        alert_type: &String,
        threshold_exceeded: u64,
        current_value: u64,
    ) {
        env.events().publish(
            ("DIAGNOSTICS", "PERFORMANCE_ALERT"),
            (
                contract_address.clone(),
                alert_type.clone(),
                threshold_exceeded,
                current_value,
            ),
        );
    }

    /// Emit cost optimization opportunity event
    pub fn emit_cost_optimization_opportunity(
        env: &Env,
        contract_address: &Address,
        potential_savings: u64,
        optimization_type: &OptimizationCategory,
    ) {
        let type_str = match optimization_type {
            OptimizationCategory::GasOptimization => "GAS",
            OptimizationCategory::StorageOptimization => "STORAGE",
            OptimizationCategory::MemoryOptimization => "MEMORY",
            OptimizationCategory::NetworkOptimization => "NETWORK",
            OptimizationCategory::AlgorithmOptimization => "ALGORITHM",
            OptimizationCategory::ArchitectureOptimization => "ARCHITECTURE",
        };

        env.events().publish(
            ("DIAGNOSTICS", "COST_OPTIMIZATION_OPPORTUNITY"),
            (
                contract_address,
                potential_savings,
                Symbol::new(env, type_str),
            ),
        );
    }

    /// Emit bottleneck identified event
    pub fn emit_bottleneck_identified(
        env: &Env,
        span_id: &BytesN<32>,
        bottleneck_type: &BottleneckType,
        severity: &RiskLevel,
        impact_percentage: u32,
    ) {
        let type_str = match bottleneck_type {
            BottleneckType::CPU => "CPU",
            BottleneckType::Memory => "MEMORY",
            BottleneckType::Storage => "STORAGE",
            BottleneckType::Network => "NETWORK",
            BottleneckType::Gas => "GAS",
            BottleneckType::Throughput => "THROUGHPUT",
        };

        let severity_str = match severity {
            RiskLevel::Low => "LOW",
            RiskLevel::Medium => "MEDIUM",
            RiskLevel::High => "HIGH",
            RiskLevel::Critical => "CRITICAL",
        };

        env.events().publish(
            ("DIAGNOSTICS", "BOTTLENECK_IDENTIFIED"),
            (
                span_id,
                Symbol::new(env, type_str),
                Symbol::new(env, severity_str),
                impact_percentage,
            ),
        );
    }

    // Additional event functions for new diagnostic features

    /// Emit regression test completion event
    pub fn emit_regression_test_complete(
        env: &Env,
        contract_address: &Address,
        regression_count: u32,
    ) {
        env.events().publish(
            ("DIAGNOSTICS", "REGRESSION_TEST_COMPLETE"),
            (contract_address, regression_count),
        );
    }

    /// Emit resource analysis completion event
    pub fn emit_resource_analysis_complete(env: &Env, contract_address: &Address) {
        env.events().publish(
            ("DIAGNOSTICS", "RESOURCE_ANALYSIS_COMPLETE"),
            contract_address,
        );
    }

    /// Emit anomalies detected event
    pub fn emit_anomalies_detected(env: &Env, contract_address: &Address, anomaly_count: u32) {
        env.events().publish(
            ("DIAGNOSTICS", "ANOMALIES_DETECTED"),
            (contract_address, anomaly_count),
        );
    }

    /// Emit critical regression alert
    pub fn emit_critical_regression_alert(
        env: &Env,
        contract_address: &Address,
        regression_id: &BytesN<32>,
    ) {
        env.events().publish(
            ("DIAGNOSTICS", "CRITICAL_REGRESSION_ALERT"),
            (contract_address, regression_id),
        );
    }

    /// Emit high severity regression alert
    pub fn emit_high_severity_regression_alert(
        env: &Env,
        contract_address: &Address,
        regression_id: &BytesN<32>,
    ) {
        env.events().publish(
            ("DIAGNOSTICS", "HIGH_SEVERITY_REGRESSION_ALERT"),
            (contract_address, regression_id),
        );
    }

    /// Emit general regression alert
    pub fn emit_regression_alert(
        env: &Env,
        contract_address: &Address,
        regression_id: &BytesN<32>,
    ) {
        env.events().publish(
            ("DIAGNOSTICS", "REGRESSION_ALERT"),
            (contract_address, regression_id),
        );
    }

    /// Emit monitoring scheduled event
    pub fn emit_monitoring_scheduled(env: &Env) {
        env.events()
            .publish(("DIAGNOSTICS", "MONITORING_SCHEDULED"), ());
    }
}
