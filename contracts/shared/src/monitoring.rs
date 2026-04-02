use soroban_sdk::{contracttype, Env, Symbol, Vec};

use crate::event_schema::{
    AlertResolvedEventData, AlertTriggeredEventData, EventData, HealthCheckEventData,
    MetricRecordedEventData, MonitoringEventData, StandardEvent,
};

/// Maximum number of custom metrics per health report (gas guard)
const MAX_CUSTOM_METRICS: u32 = 10;

/// Contract health status levels
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ContractHealthStatus {
    Healthy = 0,
    Degraded = 1,
    Unhealthy = 2,
    Unknown = 3,
}

/// A snapshot of a single metric value
#[contracttype]
#[derive(Clone, Debug)]
pub struct MetricSnapshot {
    pub name: Symbol,
    pub value: i128,
    pub timestamp: u64,
}

/// Health report returned by each contract's health_check() endpoint
#[contracttype]
#[derive(Clone, Debug)]
pub struct ContractHealthReport {
    pub contract_id: Symbol,
    pub status: ContractHealthStatus,
    pub timestamp: u64,
    pub initialized: bool,
    pub error_count: u32,
    pub custom_metrics: Vec<MetricSnapshot>,
}

/// Comparison direction for alert threshold evaluation
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum ThresholdComparison {
    GreaterThan = 0,
    LessThan = 1,
}

/// A configurable alert rule for per-metric threshold monitoring
#[contracttype]
#[derive(Clone, Debug)]
pub struct AlertRule {
    pub metric_name: Symbol,
    pub warning_threshold: i128,
    pub critical_threshold: i128,
    pub comparison: ThresholdComparison,
}

/// Result of evaluating an alert rule against a metric
#[contracttype]
#[derive(Clone, Debug)]
pub struct AlertInfo {
    pub metric_name: Symbol,
    pub level: u32,
    pub current_value: i128,
    pub threshold_value: i128,
}

/// Centralized monitoring utilities for standardized health checks, metrics, and alerts.
///
/// All methods emit events via the `StandardEvent` schema so off-chain indexers
/// can aggregate monitoring data across contracts.
pub struct Monitor;

impl Monitor {
    /// Build a basic health report. Contracts augment this with custom metrics.
    pub fn build_health_report(
        env: &Env,
        contract_id: Symbol,
        initialized: bool,
    ) -> ContractHealthReport {
        let status =
            if initialized { ContractHealthStatus::Healthy } else { ContractHealthStatus::Unknown };

        ContractHealthReport {
            contract_id,
            status,
            timestamp: env.ledger().timestamp(),
            initialized,
            error_count: 0,
            custom_metrics: Vec::new(env),
        }
    }

    /// Emit a health check event via the StandardEvent schema.
    pub fn emit_health_check(env: &Env, report: &ContractHealthReport) {
        let details = match report.status {
            ContractHealthStatus::Healthy => Symbol::new(env, "healthy"),
            ContractHealthStatus::Degraded => Symbol::new(env, "degraded"),
            ContractHealthStatus::Unhealthy => Symbol::new(env, "unhealthy"),
            ContractHealthStatus::Unknown => Symbol::new(env, "unknown"),
        };

        let event_data =
            EventData::Monitoring(MonitoringEventData::HealthCheck(HealthCheckEventData {
                contract_id: report.contract_id.clone(),
                status: report.status as u32,
                timestamp: report.timestamp,
                details,
            }));

        StandardEvent::new(
            env,
            report.contract_id.clone(),
            env.current_contract_address(),
            event_data,
        )
        .emit(env);
    }

    /// Emit a metric event for off-chain indexing.
    pub fn emit_metric(env: &Env, contract_id: Symbol, name: Symbol, value: i128) {
        let event_data =
            EventData::Monitoring(MonitoringEventData::MetricRecorded(MetricRecordedEventData {
                contract_id: contract_id.clone(),
                metric_name: name,
                value,
                timestamp: env.ledger().timestamp(),
            }));

        StandardEvent::new(env, contract_id, env.current_contract_address(), event_data).emit(env);
    }

    /// Emit an alert event when a threshold is breached.
    pub fn emit_alert(
        env: &Env,
        contract_id: Symbol,
        level: u32,
        metric_name: Symbol,
        current_value: i128,
        threshold_value: i128,
    ) {
        let event_data =
            EventData::Monitoring(MonitoringEventData::AlertTriggered(AlertTriggeredEventData {
                contract_id: contract_id.clone(),
                alert_level: level,
                metric_name,
                current_value,
                threshold_value,
            }));

        StandardEvent::new(env, contract_id, env.current_contract_address(), event_data).emit(env);
    }

    /// Emit an alert-resolved event.
    pub fn emit_alert_resolved(env: &Env, contract_id: Symbol, metric_name: Symbol) {
        let event_data =
            EventData::Monitoring(MonitoringEventData::AlertResolved(AlertResolvedEventData {
                contract_id: contract_id.clone(),
                metric_name,
                resolved_at: env.ledger().timestamp(),
            }));

        StandardEvent::new(env, contract_id, env.current_contract_address(), event_data).emit(env);
    }

    /// Evaluate alert rules against a set of metrics.
    /// Returns a list of breached alerts (warning level = 2, critical level = 4).
    pub fn check_thresholds(
        env: &Env,
        rules: &Vec<AlertRule>,
        metrics: &Vec<MetricSnapshot>,
    ) -> Vec<AlertInfo> {
        let mut alerts = Vec::new(env);

        for rule in rules.iter() {
            for metric in metrics.iter() {
                if metric.name != rule.metric_name {
                    continue;
                }

                let breached_critical = match rule.comparison {
                    ThresholdComparison::GreaterThan => metric.value >= rule.critical_threshold,
                    ThresholdComparison::LessThan => metric.value <= rule.critical_threshold,
                };

                if breached_critical {
                    alerts.push_back(AlertInfo {
                        metric_name: rule.metric_name.clone(),
                        level: 4, // Critical
                        current_value: metric.value,
                        threshold_value: rule.critical_threshold,
                    });
                    continue;
                }

                let breached_warning = match rule.comparison {
                    ThresholdComparison::GreaterThan => metric.value >= rule.warning_threshold,
                    ThresholdComparison::LessThan => metric.value <= rule.warning_threshold,
                };

                if breached_warning {
                    alerts.push_back(AlertInfo {
                        metric_name: rule.metric_name.clone(),
                        level: 2, // Warning
                        current_value: metric.value,
                        threshold_value: rule.warning_threshold,
                    });
                }
            }
        }

        alerts
    }

    /// Add a custom metric to a health report, respecting the max cap.
    pub fn add_metric(
        report: &mut ContractHealthReport,
        name: Symbol,
        value: i128,
        timestamp: u64,
    ) {
        if report.custom_metrics.len() < MAX_CUSTOM_METRICS {
            report.custom_metrics.push_back(MetricSnapshot { name, value, timestamp });
        }
    }
}

// ---------------------------------------------------------------------------
// Storage-backed counter / gauge metrics
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
enum MonitoringKey {
    Counter(Symbol),
    Gauge(Symbol),
    ContractCalls(Symbol),
    ContractSuccesses(Symbol),
    ContractFailures(Symbol),
    ContractLatency(Symbol),
    InfraBacklog(Symbol),
}

/// Storage-backed counter and gauge utilities for per-metric tracking.
pub struct Monitoring;

impl Monitoring {
    pub fn increment_counter(env: &Env, name: &Symbol, delta: i128) -> MetricSnapshot {
        let next_value = Self::read_metric(env, &MonitoringKey::Counter(name.clone())) + delta;
        let snapshot = MetricSnapshot {
            name: name.clone(),
            value: next_value,
            timestamp: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&MonitoringKey::Counter(name.clone()), &snapshot);
        Self::emit_metric_event(env, name, "counter", next_value);
        snapshot
    }

    pub fn set_gauge(env: &Env, name: &Symbol, value: i128) -> MetricSnapshot {
        let snapshot =
            MetricSnapshot { name: name.clone(), value, timestamp: env.ledger().timestamp() };

        env.storage().persistent().set(&MonitoringKey::Gauge(name.clone()), &snapshot);
        Self::emit_metric_event(env, name, "gauge", value);
        snapshot
    }

    pub fn get_counter(env: &Env, name: &Symbol) -> Option<MetricSnapshot> {
        env.storage().persistent().get(&MonitoringKey::Counter(name.clone()))
    }

    pub fn get_gauge(env: &Env, name: &Symbol) -> Option<MetricSnapshot> {
        env.storage().persistent().get(&MonitoringKey::Gauge(name.clone()))
    }

    pub fn record_contract_call(
        env: &Env,
        contract: &Symbol,
        success: bool,
        latency_ms: u64,
    ) -> MetricSnapshot {
        let call_metric = Symbol::new(env, "contract_calls");
        let latency_metric = Symbol::new(env, "contract_latency");
        let status_metric = if success {
            Symbol::new(env, "contract_successes")
        } else {
            Symbol::new(env, "contract_failures")
        };

        Self::increment_keyed_counter(
            env,
            &call_metric,
            &MonitoringKey::ContractCalls(contract.clone()),
            1,
        );
        if success {
            Self::increment_keyed_counter(
                env,
                &status_metric,
                &MonitoringKey::ContractSuccesses(contract.clone()),
                1,
            );
        } else {
            Self::increment_keyed_counter(
                env,
                &status_metric,
                &MonitoringKey::ContractFailures(contract.clone()),
                1,
            );
        }
        Self::set_keyed_gauge(
            env,
            &latency_metric,
            &MonitoringKey::ContractLatency(contract.clone()),
            latency_ms as i128,
        )
    }

    pub fn record_infra_backlog(env: &Env, pipeline: &Symbol, pending_jobs: u32) -> MetricSnapshot {
        Self::set_keyed_gauge(
            env,
            &Symbol::new(env, "infra_backlog"),
            &MonitoringKey::InfraBacklog(pipeline.clone()),
            pending_jobs as i128,
        )
    }

    pub fn metric_catalog(env: &Env) -> Vec<Symbol> {
        let mut metrics = Vec::new(env);
        for name in [
            "rpc_calls",
            "rpc_failures",
            "rpc_latency_ms",
            "contract_calls",
            "contract_failures",
            "queue_backlog",
            "ledger_lag",
        ] {
            metrics.push_back(Symbol::new(env, name));
        }
        metrics
    }

    fn read_metric(env: &Env, key: &MonitoringKey) -> i128 {
        env.storage()
            .persistent()
            .get::<MonitoringKey, MetricSnapshot>(key)
            .map(|snapshot| snapshot.value)
            .unwrap_or(0)
    }

    fn increment_keyed_counter(
        env: &Env,
        name: &Symbol,
        key: &MonitoringKey,
        delta: i128,
    ) -> MetricSnapshot {
        let next_value = Self::read_metric(env, key) + delta;
        let snapshot = MetricSnapshot {
            name: name.clone(),
            value: next_value,
            timestamp: env.ledger().timestamp(),
        };

        env.storage().persistent().set(key, &snapshot);
        Self::emit_metric_event(env, name, "counter", next_value);
        snapshot
    }

    fn set_keyed_gauge(
        env: &Env,
        name: &Symbol,
        key: &MonitoringKey,
        value: i128,
    ) -> MetricSnapshot {
        let snapshot =
            MetricSnapshot { name: name.clone(), value, timestamp: env.ledger().timestamp() };

        env.storage().persistent().set(key, &snapshot);
        Self::emit_metric_event(env, name, "gauge", value);
        snapshot
    }

    fn emit_metric_event(env: &Env, name: &Symbol, kind: &str, value: i128) {
        let kind_sym = Symbol::new(env, kind);
        env.events().publish((Symbol::new(env, "monitoring"), name.clone()), (kind_sym, value));
    }
}

#[cfg(test)]
mod tests {
    use super::Monitoring;
    use soroban_sdk::{contract, contractimpl, testutils::Ledger, Env, Symbol};

    #[contract]
    struct MonitoringTestContract;

    #[contractimpl]
    impl MonitoringTestContract {}

    #[test]
    fn increment_counter_accumulates_values() {
        let env = Env::default();
        let contract_id = env.register(MonitoringTestContract, ());
        env.ledger().set_timestamp(100);
        let metric = Symbol::new(&env, "rpc_calls");

        let first =
            env.as_contract(&contract_id, || Monitoring::increment_counter(&env, &metric, 1));
        let second =
            env.as_contract(&contract_id, || Monitoring::increment_counter(&env, &metric, 2));
        let stored = env.as_contract(&contract_id, || Monitoring::get_counter(&env, &metric));

        assert_eq!(first.value, 1);
        assert_eq!(second.value, 3);
        assert_eq!(stored.unwrap().value, 3);
    }

    #[test]
    fn record_contract_call_tracks_status_and_latency() {
        let env = Env::default();
        let contract_id = env.register(MonitoringTestContract, ());
        env.ledger().set_timestamp(250);
        let contract = Symbol::new(&env, "token");

        let latency = env.as_contract(&contract_id, || {
            Monitoring::record_contract_call(&env, &contract, false, 45)
        });

        assert_eq!(latency.value, 45);
        assert_eq!(latency.name, Symbol::new(&env, "contract_latency"));
        assert_eq!(latency.timestamp, 250);
    }
}
