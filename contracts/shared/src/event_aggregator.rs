use crate::event_schema::StandardEvent;
use soroban_sdk::{Address, Env, Map, Symbol, Vec};

/// Aggregated event statistics
#[soroban_sdk::contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct EventStats {
    /// Total event count
    pub total_count: u32,
    /// Count by category
    pub category_counts: Map<Symbol, u32>,
    /// Count by contract
    pub contract_counts: Map<Symbol, u32>,
    /// Count by event type
    pub event_type_counts: Map<Symbol, u32>,
    /// First event timestamp
    pub first_timestamp: u64,
    /// Last event timestamp
    pub last_timestamp: u64,
}

/// Event aggregation utilities
pub struct EventAggregator;

impl EventAggregator {
    const STATS_KEY: &'static str = "event_stats";
    const AGGREGATION_WINDOW: u64 = 86400; // 24 hours in seconds

    /// Aggregate events by category
    pub fn aggregate_by_category(env: &Env, events: Vec<StandardEvent>) -> Map<Symbol, u32> {
        let mut result = Map::new(env);

        for event in events.iter() {
            let category = Symbol::new(env, event.get_category());
            let count = result.get(category.clone()).unwrap_or(0);
            result.set(category, count + 1);
        }

        result
    }

    /// Aggregate events by contract
    pub fn aggregate_by_contract(env: &Env, events: Vec<StandardEvent>) -> Map<Symbol, u32> {
        let mut result = Map::new(env);

        for event in events.iter() {
            let contract = event.contract.clone();
            let count = result.get(contract.clone()).unwrap_or(0);
            result.set(contract, count + 1);
        }

        result
    }

    /// Aggregate events by actor
    pub fn aggregate_by_actor(env: &Env, events: Vec<StandardEvent>) -> Map<Address, u32> {
        let mut result = Map::new(env);

        for event in events.iter() {
            let actor = event.actor.clone();
            let count = result.get(actor.clone()).unwrap_or(0);
            result.set(actor, count + 1);
        }

        result
    }

    /// Calculate event statistics
    pub fn calculate_stats(env: &Env, events: Vec<StandardEvent>) -> EventStats {
        let mut category_counts = Map::new(env);
        let mut contract_counts = Map::new(env);
        let mut event_type_counts = Map::new(env);
        let mut first_timestamp = u64::MAX;
        let mut last_timestamp = 0u64;
        let mut total_count = 0u32;

        for event in events.iter() {
            total_count += 1;

            // Category count
            let category = Symbol::new(env, event.get_category());
            let cat_count = category_counts.get(category.clone()).unwrap_or(0);
            category_counts.set(category, cat_count + 1);

            // Contract count
            let contract = event.contract.clone();
            let contract_count = contract_counts.get(contract.clone()).unwrap_or(0);
            contract_counts.set(contract, contract_count + 1);

            // Event type count
            let event_type = Symbol::new(env, event.get_event_type());
            let type_count = event_type_counts.get(event_type.clone()).unwrap_or(0);
            event_type_counts.set(event_type, type_count + 1);

            // Timestamp tracking
            if event.timestamp < first_timestamp {
                first_timestamp = event.timestamp;
            }
            if event.timestamp > last_timestamp {
                last_timestamp = event.timestamp;
            }
        }

        EventStats {
            total_count,
            category_counts,
            contract_counts,
            event_type_counts,
            first_timestamp: if first_timestamp == u64::MAX {
                0
            } else {
                first_timestamp
            },
            last_timestamp,
        }
    }

    /// Batch process events for aggregation
    pub fn batch_aggregate(
        env: &Env,
        events: Vec<StandardEvent>,
        batch_size: u32,
    ) -> Vec<EventStats> {
        let mut results = Vec::new(env);
        let mut batch = Vec::new(env);
        let mut count = 0u32;

        for event in events.iter() {
            batch.push_back(event.clone());
            count += 1;

            if count >= batch_size {
                let stats = Self::calculate_stats(env, batch.clone());
                results.push_back(stats);
                batch = Vec::new(env);
                count = 0;
            }
        }

        // Process remaining events
        if !batch.is_empty() {
            let stats = Self::calculate_stats(env, batch);
            results.push_back(stats);
        }

        results
    }

    /// Get top N contracts by event count
    pub fn top_contracts(env: &Env, events: Vec<StandardEvent>, top_n: u32) -> Vec<(Symbol, u32)> {
        let contract_counts = Self::aggregate_by_contract(env, events);
        let mut pairs = Vec::new(env);

        // Note: In a real implementation, we'd sort this properly
        // For now, we return the first N entries
        let mut count = 0u32;
        for (contract, count_val) in contract_counts.iter() {
            if count >= top_n {
                break;
            }
            pairs.push_back((contract, count_val));
            count += 1;
        }

        pairs
    }

    /// Get top N actors by event count
    pub fn top_actors(env: &Env, events: Vec<StandardEvent>, top_n: u32) -> Vec<(Address, u32)> {
        let actor_counts = Self::aggregate_by_actor(env, events);
        let mut pairs = Vec::new(env);

        let mut count = 0u32;
        for (actor, count_val) in actor_counts.iter() {
            if count >= top_n {
                break;
            }
            pairs.push_back((actor, count_val));
            count += 1;
        }

        pairs
    }

    /// Store aggregated statistics
    pub fn store_stats(env: &Env, stats: &EventStats, window_id: u64) {
        let key = (Symbol::new(env, Self::STATS_KEY), window_id);
        env.storage().persistent().set(&key, stats);
    }

    /// Get stored statistics for a time window
    pub fn get_stats(env: &Env, window_id: u64) -> Option<EventStats> {
        let key = (Symbol::new(env, Self::STATS_KEY), window_id);
        env.storage().persistent().get(&key)
    }

    /// Calculate time window ID from timestamp
    pub fn get_window_id(timestamp: u64) -> u64 {
        timestamp / Self::AGGREGATION_WINDOW
    }
}
