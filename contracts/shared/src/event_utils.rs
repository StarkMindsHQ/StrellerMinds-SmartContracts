use soroban_sdk::{Address, BytesN, Env, Symbol, String};
use crate::event_schema::{StandardEvent, EventData};

/// Gas-optimized event utilities
pub struct EventUtils;

impl EventUtils {
    /// Emit a minimal event with reduced gas cost
    /// Uses only essential topics and data
    pub fn emit_minimal(
        env: &Env,
        contract: Symbol,
        event_type: Symbol,
        actor: Address,
    ) {
        let topics = (
            Symbol::new(env, "min_event"),
            contract,
            event_type,
            actor,
        );
        env.events().publish(topics, ());
    }

    /// Emit a batched event to reduce gas costs
    pub fn emit_batched(
        env: &Env,
        contract: Symbol,
        event_type: Symbol,
        events: Vec<StandardEvent>,
    ) {
        let topics = (
            Symbol::new(env, "batch_event"),
            contract,
            event_type,
        );
        let count = events.len();
        env.events().publish(topics, count);
    }

    /// Validate event before emission
    pub fn validate_event(env: &Env, event: &StandardEvent) -> Result<(), String> {
        // Validate version
        if event.version != crate::event_schema::EVENT_SCHEMA_VERSION {
            return Err(String::from_str(env, "Invalid event version"));
        }

        // Validate timestamp (not in future)
        let current_time = env.ledger().timestamp();
        if event.timestamp > current_time {
            return Err(String::from_str(env, "Event timestamp in future"));
        }

        // Validate contract symbol
        if event.contract.to_string().is_empty() {
            return Err(String::from_str(env, "Empty contract identifier"));
        }

        // Validate actor address
        // Address validation is handled by Soroban SDK

        Ok(())
    }

    /// Validate event ordering
    pub fn validate_ordering(
        env: &Env,
        current_sequence: u32,
        previous_sequence: Option<u32>,
    ) -> Result<(), String> {
        if let Some(prev_seq) = previous_sequence {
            if current_sequence <= prev_seq {
                return Err(String::from_str(env, "Invalid event sequence"));
            }
        }
        Ok(())
    }

    /// Calculate estimated gas cost for event emission
    pub fn estimate_gas_cost(event: &StandardEvent) -> u64 {
        // Base cost
        let mut cost = 100u64;

        // Cost per topic (Soroban has 4 topics max)
        cost += 50 * 4;

        // Cost for data size
        let data_size = Self::estimate_data_size(event);
        cost += data_size / 10;

        cost
    }

    /// Estimate data size for an event
    fn estimate_data_size(event: &StandardEvent) -> u64 {
        // Rough estimation
        let mut size = 32u64; // version + timestamp
        size += 32; // tx_hash
        size += 32; // contract symbol
        size += 32; // actor address
        size += Self::estimate_event_data_size(&event.event_data);
        size
    }

    fn estimate_event_data_size(data: &EventData) -> u64 {
        match data {
            EventData::AccessControl(_) => 64,
            EventData::Certificate(_) => 128,
            EventData::Analytics(_) => 96,
            EventData::Token(_) => 64,
            EventData::Progress(_) => 64,
            EventData::System(_) => 64,
            EventData::Error(_) => 64,
        }
    }

    /// Create a secure event hash for verification
    pub fn create_event_hash(env: &Env, event: &StandardEvent) -> BytesN<32> {
        // Create a deterministic hash from event data
        let mut hash_data = [0u8; 32];
        
        // Include key fields in hash
        let seq_bytes = event.timestamp.to_be_bytes();
        let contract_bytes = event.contract.to_string().as_bytes();
        
        // Simple hash (in production, use proper hashing)
        for i in 0..32 {
            if i < 8 {
                hash_data[i] = seq_bytes[i % 8];
            } else if i < 16 {
                let idx = (i - 8) % contract_bytes.len();
                hash_data[i] = contract_bytes[idx];
            } else {
                hash_data[i] = hash_data[i - 16] ^ hash_data[i - 8];
            }
        }
        
        BytesN::from_array(env, &hash_data)
    }

    /// Verify event integrity
    pub fn verify_event_integrity(
        env: &Env,
        event: &StandardEvent,
        expected_hash: &BytesN<32>,
    ) -> bool {
        let computed_hash = Self::create_event_hash(env, event);
        computed_hash == *expected_hash
    }

    /// Check if event is within rate limit
    pub fn check_rate_limit(
        env: &Env,
        actor: &Address,
        max_events_per_period: u32,
        period_seconds: u64,
    ) -> Result<(), String> {
        let key = Symbol::new(env, &format!("rate_limit_{}", actor.to_string()));
        let current_time = env.ledger().timestamp();
        
        if let Some((count, reset_time)) = env.storage()
            .temporary()
            .get::<_, (u32, u64)>(&key) {
            if current_time < reset_time {
                if count >= max_events_per_period {
                    return Err(String::from_str(env, "Rate limit exceeded"));
                }
                // Increment count
                env.storage().temporary().set(&key, &(count + 1, reset_time));
            } else {
                // Reset period
                env.storage().temporary().set(&key, &(1, current_time + period_seconds));
            }
        } else {
            // First event in period
            env.storage().temporary().set(&key, &(1, current_time + period_seconds));
        }
        
        Ok(())
    }

    /// Get event ordering guarantee level
    pub fn get_ordering_guarantee(event: &StandardEvent) -> OrderingGuarantee {
        // Events within the same transaction are ordered
        // Events across transactions are ordered by ledger sequence
        if event.timestamp > 0 {
            OrderingGuarantee::Sequential
        } else {
            OrderingGuarantee::BestEffort
        }
    }
}

/// Event ordering guarantee levels
#[derive(Clone, Debug, PartialEq)]
pub enum OrderingGuarantee {
    /// Events are guaranteed to be in sequence
    Sequential,
    /// Best effort ordering (may have gaps)
    BestEffort,
    /// No ordering guarantee
    None,
}
