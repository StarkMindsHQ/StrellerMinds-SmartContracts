use shared::gas_optimizer::{
    extend_instance_if_needed, pack_u32, unpack_u32, BatchResult, TTL_BUMP_THRESHOLD,
};
use soroban_sdk::{contracttype, symbol_short, Address, Env, Symbol, Vec};

const KEY_METRICS: Symbol = symbol_short!("METRICS");
const KEY_EV_COUNT: Symbol = symbol_short!("EVCOUNT");

#[contracttype]
#[derive(Clone, Default, PartialEq)]
pub struct PackedMetrics {
    pub total_views: u64,
    pub total_completions: u64,
    pub total_time_secs: u64,
    pub learners_and_score: u64,
}

impl PackedMetrics {
    pub fn active_learners(&self) -> u32 {
        unpack_u32(self.learners_and_score).0
    }
    pub fn avg_score_pct(&self) -> u32 {
        unpack_u32(self.learners_and_score).1
    }
    pub fn set_learners_and_score(&mut self, learners: u32, score_x100: u32) {
        self.learners_and_score = pack_u32(learners, score_x100);
    }
}

#[contracttype]
#[derive(Clone)]
pub struct CompactEvent {
    pub learner: Address,
    pub event_type: u32,
    pub value: u64,
    pub ledger_seq: u32,
}

pub fn record_event_optimized(env: &Env, learner: &Address, event_type: u32, value: u64) {
    let mut metrics: PackedMetrics = env
        .storage()
        .instance()
        .get(&KEY_METRICS)
        .unwrap_or_default();

    match event_type {
        0 => metrics.total_views += 1,
        1 => {
            metrics.total_completions += 1;
            metrics.total_time_secs = metrics.total_time_secs.saturating_add(value);
        }
        2 => {
            let (learners, prev_avg) = unpack_u32(metrics.learners_and_score);
            let new_avg = if learners == 0 {
                value as u32
            } else {
                (((prev_avg as u64) * (learners as u64) + value) / (learners as u64 + 1)) as u32
            };
            metrics.set_learners_and_score(learners.saturating_add(1), new_avg);
        }
        _ => {}
    }

    env.storage().instance().set(&KEY_METRICS, &metrics);

    let ev_count: u32 = env.storage().instance().get(&KEY_EV_COUNT).unwrap_or(0);
    let ev_key = (KEY_METRICS, ev_count);
    env.storage().temporary().set(
        &ev_key,
        &CompactEvent {
            learner: learner.clone(),
            event_type,
            value,
            ledger_seq: env.ledger().sequence(),
        },
    );
    env.storage().temporary().extend_ttl(&ev_key, 1, 100_000);
    env.storage()
        .instance()
        .set(&KEY_EV_COUNT, &ev_count.saturating_add(1));
    extend_instance_if_needed(env);
}

pub fn batch_record_events(env: &Env, _learner: &Address, events: &Vec<(u32, u64)>) -> BatchResult {
    let mut result = BatchResult::new();
    let mut metrics: PackedMetrics = env
        .storage()
        .instance()
        .get(&KEY_METRICS)
        .unwrap_or_default();

    for i in 0..events.len() {
        if let Some((event_type, value)) = events.get(i) {
            match event_type {
                0 => {
                    metrics.total_views += 1;
                    result.processed += 1;
                }
                1 => {
                    metrics.total_completions += 1;
                    metrics.total_time_secs = metrics.total_time_secs.saturating_add(value);
                    result.processed += 1;
                }
                2 => {
                    let (learners, prev_avg) = unpack_u32(metrics.learners_and_score);
                    let new_avg = if learners == 0 {
                        value as u32
                    } else {
                        (((prev_avg as u64) * (learners as u64) + value) / (learners as u64 + 1))
                            as u32
                    };
                    metrics.set_learners_and_score(learners.saturating_add(1), new_avg);
                    result.processed += 1;
                }
                _ => {
                    result.skipped += 1;
                }
            }
        }
    }

    env.storage().instance().set(&KEY_METRICS, &metrics);
    extend_instance_if_needed(env);
    result
}

pub fn get_metrics(env: &Env) -> PackedMetrics {
    env.storage()
        .instance()
        .get(&KEY_METRICS)
        .unwrap_or_default()
}

pub fn refresh_storage_ttls(env: &Env) {
    env.storage()
        .instance()
        .extend_ttl(TTL_BUMP_THRESHOLD, 535_680);
}
