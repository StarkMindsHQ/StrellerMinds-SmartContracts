use soroban_sdk::{Address, BytesN, Env, Symbol, String, Vec};
use crate::event_schema::{StandardEvent, EventData};
use crate::event_publisher::EventPublisher;
use crate::event_filter::{EventFilter, EventRouter, EventFilterBuilder};
use crate::event_aggregator::EventAggregator;
use crate::event_replay::EventReplay;
use crate::event_utils::EventUtils;

/// Unified event manager that provides a single interface for all event operations
pub struct EventManager;

impl EventManager {
    /// Publish an event with full validation and ordering guarantees
    pub fn publish_event(
        env: &Env,
        event: StandardEvent,
    ) -> Result<u32, String> {
        // Validate event
        EventUtils::validate_event(env, &event)?;

        // Publish with sequence number for ordering
        EventPublisher::publish(env, event)
    }

    /// Subscribe to events with filtering
    pub fn subscribe(
        env: &Env,
        subscriber: &Address,
        categories: Vec<Symbol>,
        event_types: Vec<Symbol>,
        contracts: Vec<Symbol>,
    ) -> Result<u32, String> {
        EventPublisher::subscribe(env, subscriber, categories, event_types, contracts)
    }

    /// Unsubscribe from events
    pub fn unsubscribe(
        env: &Env,
        subscriber: &Address,
        subscription_id: u32,
    ) -> Result<(), String> {
        EventPublisher::unsubscribe(env, subscriber, subscription_id)
    }

    /// Filter events based on criteria
    pub fn filter_events(
        env: &Env,
        events: Vec<StandardEvent>,
        filter: &EventFilter,
    ) -> Vec<StandardEvent> {
        let mut filtered = Vec::new(env);
        for event in events.iter() {
            if EventRouter::matches_filter(env, event, filter) {
                filtered.push_back(event.clone());
            }
        }
        filtered
    }

    /// Aggregate events and calculate statistics
    pub fn aggregate_events(
        env: &Env,
        events: Vec<StandardEvent>,
    ) -> crate::event_aggregator::EventStats {
        EventAggregator::calculate_stats(env, events)
    }

    /// Replay events from a sequence range
    pub fn replay_events(
        env: &Env,
        from_sequence: u32,
        to_sequence: Option<u32>,
    ) -> Result<(Vec<StandardEvent>, EventReplay::ReplayState), String> {
        let state = EventReplay::start_replay(env, from_sequence, to_sequence)?;
        EventReplay::replay_next_batch(env, 100)
    }

    /// Replay events in a time range
    pub fn replay_time_range(
        env: &Env,
        start_timestamp: u64,
        end_timestamp: u64,
    ) -> Result<Vec<StandardEvent>, String> {
        EventReplay::replay_time_range(env, start_timestamp, end_timestamp)
    }

    /// Create a filter builder for fluent API
    pub fn filter_builder() -> EventFilterBuilder {
        EventFilterBuilder::new()
    }

    /// Emit a minimal event (gas optimized)
    pub fn emit_minimal(
        env: &Env,
        contract: Symbol,
        event_type: Symbol,
        actor: Address,
    ) {
        EventUtils::emit_minimal(env, contract, event_type, actor);
    }

    /// Batch emit events (gas optimized)
    pub fn emit_batched(
        env: &Env,
        contract: Symbol,
        event_type: Symbol,
        events: Vec<StandardEvent>,
    ) {
        EventUtils::emit_batched(env, contract, event_type, events);
    }

    /// Get event statistics for a time window
    pub fn get_window_stats(
        env: &Env,
        window_id: u64,
    ) -> Option<crate::event_aggregator::EventStats> {
        EventAggregator::get_stats(env, window_id)
    }

    /// Calculate time window ID from timestamp
    pub fn get_window_id(timestamp: u64) -> u64 {
        EventAggregator::get_window_id(timestamp)
    }

    /// Verify event integrity
    pub fn verify_event(
        env: &Env,
        event: &StandardEvent,
        expected_hash: &BytesN<32>,
    ) -> bool {
        EventUtils::verify_event_integrity(env, event, expected_hash)
    }

    /// Check rate limit for an actor
    pub fn check_rate_limit(
        env: &Env,
        actor: &Address,
        max_events_per_period: u32,
        period_seconds: u64,
    ) -> Result<(), String> {
        EventUtils::check_rate_limit(env, actor, max_events_per_period, period_seconds)
    }

    /// Get current event sequence
    pub fn get_current_sequence(env: &Env) -> u32 {
        EventReplay::get_current_sequence(env)
    }
}
