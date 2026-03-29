use soroban_sdk::{contracttype, Env, Symbol, Vec};

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum MetricKind {
    Counter,
    Gauge,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct MetricSnapshot {
    pub name: Symbol,
    pub kind: MetricKind,
    pub value: i128,
    pub updated_at: u64,
}

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

pub struct Monitoring;

impl Monitoring {
    pub fn increment_counter(env: &Env, name: &Symbol, delta: i128) -> MetricSnapshot {
        let next_value = Self::read_metric(env, &MonitoringKey::Counter(name.clone())) + delta;
        let snapshot = MetricSnapshot {
            name: name.clone(),
            kind: MetricKind::Counter,
            value: next_value,
            updated_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&MonitoringKey::Counter(name.clone()), &snapshot);
        Self::emit_metric_event(env, name, &MetricKind::Counter, next_value);
        snapshot
    }

    pub fn set_gauge(env: &Env, name: &Symbol, value: i128) -> MetricSnapshot {
        let snapshot = MetricSnapshot {
            name: name.clone(),
            kind: MetricKind::Gauge,
            value,
            updated_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&MonitoringKey::Gauge(name.clone()), &snapshot);
        Self::emit_metric_event(env, name, &MetricKind::Gauge, value);
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
            kind: MetricKind::Counter,
            value: next_value,
            updated_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(key, &snapshot);
        Self::emit_metric_event(env, name, &MetricKind::Counter, next_value);
        snapshot
    }

    fn set_keyed_gauge(
        env: &Env,
        name: &Symbol,
        key: &MonitoringKey,
        value: i128,
    ) -> MetricSnapshot {
        let snapshot = MetricSnapshot {
            name: name.clone(),
            kind: MetricKind::Gauge,
            value,
            updated_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(key, &snapshot);
        Self::emit_metric_event(env, name, &MetricKind::Gauge, value);
        snapshot
    }

    fn emit_metric_event(env: &Env, name: &Symbol, kind: &MetricKind, value: i128) {
        let kind_name = match kind {
            MetricKind::Counter => Symbol::new(env, "counter"),
            MetricKind::Gauge => Symbol::new(env, "gauge"),
        };

        env.events().publish((Symbol::new(env, "monitoring"), name.clone()), (kind_name, value));
    }
}

#[cfg(test)]
mod tests {
    use super::{MetricKind, Monitoring};
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

        assert_eq!(first.kind, MetricKind::Counter);
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

        assert_eq!(latency.kind, MetricKind::Gauge);
        assert_eq!(latency.value, 45);
        assert_eq!(latency.name, Symbol::new(&env, "contract_latency"));
        assert_eq!(latency.updated_at, 250);
    }
}
