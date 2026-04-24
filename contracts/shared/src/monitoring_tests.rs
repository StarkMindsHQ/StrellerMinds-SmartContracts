#[cfg(test)]
mod tests {
    use crate::monitoring::{
        AlertRule, ContractHealthStatus, MetricSnapshot, Monitor, ThresholdComparison,
    };
    use soroban_sdk::{symbol_short, Env, Vec};

    #[test]
    fn test_build_health_report_initialized() {
        let env = Env::default();
        let report = Monitor::build_health_report(&env, symbol_short!("token"), true);

        assert_eq!(report.status, ContractHealthStatus::Healthy);
        assert!(report.initialized);
        assert_eq!(report.error_count, 0);
        assert_eq!(report.custom_metrics.len(), 0);
        assert_eq!(report.contract_id, symbol_short!("token"));
    }

    #[test]
    fn test_build_health_report_not_initialized() {
        let env = Env::default();
        let report = Monitor::build_health_report(&env, symbol_short!("token"), false);

        assert_eq!(report.status, ContractHealthStatus::Unknown);
        assert!(!report.initialized);
    }

    #[test]
    fn test_add_metric_to_report() {
        let env = Env::default();
        let mut report = Monitor::build_health_report(&env, symbol_short!("token"), true);

        Monitor::add_metric(&mut report, symbol_short!("txcount"), 42, 1000);

        assert_eq!(report.custom_metrics.len(), 1);
        let metric = report.custom_metrics.get(0).unwrap();
        assert_eq!(metric.name, symbol_short!("txcount"));
        assert_eq!(metric.value, 42);
        assert_eq!(metric.timestamp, 1000);
    }

    #[test]
    fn test_add_metric_respects_max_cap() {
        let env = Env::default();
        let mut report = Monitor::build_health_report(&env, symbol_short!("token"), true);

        // Add 11 metrics — only 10 should be kept
        for i in 0..11i128 {
            Monitor::add_metric(&mut report, symbol_short!("m"), i, 1000);
        }

        assert_eq!(report.custom_metrics.len(), 10);
    }

    // NOTE: Event emission tests (emit_health_check, emit_metric, emit_alert)
    // require a contract context and are tested at the contract level
    // (see token/src/test.rs, certificate/src/test.rs, cross-chain-credentials/src/tests.rs).

    #[test]
    fn test_check_thresholds_no_breach() {
        let env = Env::default();

        let mut rules = Vec::new(&env);
        rules.push_back(AlertRule {
            metric_name: symbol_short!("errors"),
            warning_threshold: 10,
            critical_threshold: 50,
            comparison: ThresholdComparison::GreaterThan,
        });

        let mut metrics = Vec::new(&env);
        metrics.push_back(MetricSnapshot {
            name: symbol_short!("errors"),
            value: 5, // Below warning threshold
            timestamp: 1000,
        });

        let alerts = Monitor::check_thresholds(&env, &rules, &metrics);
        assert_eq!(alerts.len(), 0);
    }

    #[test]
    fn test_check_thresholds_warning_breach() {
        let env = Env::default();

        let mut rules = Vec::new(&env);
        rules.push_back(AlertRule {
            metric_name: symbol_short!("errors"),
            warning_threshold: 10,
            critical_threshold: 50,
            comparison: ThresholdComparison::GreaterThan,
        });

        let mut metrics = Vec::new(&env);
        metrics.push_back(MetricSnapshot {
            name: symbol_short!("errors"),
            value: 25, // Above warning, below critical
            timestamp: 1000,
        });

        let alerts = Monitor::check_thresholds(&env, &rules, &metrics);
        assert_eq!(alerts.len(), 1);
        let alert = alerts.get(0).unwrap();
        assert_eq!(alert.level, 2); // Warning
        assert_eq!(alert.current_value, 25);
        assert_eq!(alert.threshold_value, 10);
    }

    #[test]
    fn test_check_thresholds_critical_breach() {
        let env = Env::default();

        let mut rules = Vec::new(&env);
        rules.push_back(AlertRule {
            metric_name: symbol_short!("errors"),
            warning_threshold: 10,
            critical_threshold: 50,
            comparison: ThresholdComparison::GreaterThan,
        });

        let mut metrics = Vec::new(&env);
        metrics.push_back(MetricSnapshot {
            name: symbol_short!("errors"),
            value: 75, // Above critical
            timestamp: 1000,
        });

        let alerts = Monitor::check_thresholds(&env, &rules, &metrics);
        assert_eq!(alerts.len(), 1);
        let alert = alerts.get(0).unwrap();
        assert_eq!(alert.level, 4); // Critical
        assert_eq!(alert.current_value, 75);
        assert_eq!(alert.threshold_value, 50);
    }

    #[test]
    fn test_check_thresholds_less_than_comparison() {
        let env = Env::default();

        let mut rules = Vec::new(&env);
        rules.push_back(AlertRule {
            metric_name: symbol_short!("uptime"),
            warning_threshold: 90,
            critical_threshold: 50,
            comparison: ThresholdComparison::LessThan,
        });

        let mut metrics = Vec::new(&env);
        metrics.push_back(MetricSnapshot {
            name: symbol_short!("uptime"),
            value: 30, // Below critical (50)
            timestamp: 1000,
        });

        let alerts = Monitor::check_thresholds(&env, &rules, &metrics);
        assert_eq!(alerts.len(), 1);
        let alert = alerts.get(0).unwrap();
        assert_eq!(alert.level, 4); // Critical
        assert_eq!(alert.current_value, 30);
    }

    #[test]
    fn test_check_thresholds_less_than_warning_only() {
        let env = Env::default();

        let mut rules = Vec::new(&env);
        rules.push_back(AlertRule {
            metric_name: symbol_short!("uptime"),
            warning_threshold: 90,
            critical_threshold: 50,
            comparison: ThresholdComparison::LessThan,
        });

        let mut metrics = Vec::new(&env);
        metrics.push_back(MetricSnapshot {
            name: symbol_short!("uptime"),
            value: 70, // Below warning (90) but above critical (50)
            timestamp: 1000,
        });

        let alerts = Monitor::check_thresholds(&env, &rules, &metrics);
        assert_eq!(alerts.len(), 1);
        let alert = alerts.get(0).unwrap();
        assert_eq!(alert.level, 2); // Warning
    }

    #[test]
    fn test_check_thresholds_unmatched_metric_ignored() {
        let env = Env::default();

        let mut rules = Vec::new(&env);
        rules.push_back(AlertRule {
            metric_name: symbol_short!("errors"),
            warning_threshold: 10,
            critical_threshold: 50,
            comparison: ThresholdComparison::GreaterThan,
        });

        let mut metrics = Vec::new(&env);
        metrics.push_back(MetricSnapshot {
            name: symbol_short!("latency"), // Different metric name
            value: 999,
            timestamp: 1000,
        });

        let alerts = Monitor::check_thresholds(&env, &rules, &metrics);
        assert_eq!(alerts.len(), 0);
    }
}
