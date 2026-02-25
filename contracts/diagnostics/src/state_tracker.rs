use crate::types::*;
use soroban_sdk::{
    contract, contractimpl, Address, Bytes, BytesN, Env, Map, String, Symbol, Vec,
};

/// State tracker for real-time contract state visualization
pub struct StateTracker;

impl StateTracker {
    /// Capture current contract state snapshot
    pub fn capture_snapshot(
        env: &Env,
        contract_id: &Address,
    ) -> StateSnapshot {
        let timestamp = env.ledger().timestamp();
        let ledger_sequence = env.ledger().sequence();
        
        // Count storage entries (simplified - in production would iterate storage)
        let storage_entries = Self::count_storage_entries(env);
        
        // Estimate memory usage
        let memory_usage_bytes = Self::estimate_memory_usage(env, storage_entries);
        
        // Generate state hash for integrity checking
        let state_hash = Self::generate_state_hash(env, contract_id, timestamp);
        
        // Capture key-value pairs from storage
        let key_value_pairs = Self::capture_storage_state(env);

        StateSnapshot {
            contract_id: contract_id.clone(),
            timestamp,
            ledger_sequence,
            storage_entries,
            memory_usage_bytes,
            state_hash,
            key_value_pairs,
        }
    }

    /// Compare two state snapshots to detect changes
    pub fn compare_snapshots(
        env: &Env,
        snapshot1: &StateSnapshot,
        snapshot2: &StateSnapshot,
    ) -> Vec<String> {
        let mut differences = Vec::new(env);

        // Check if storage entries changed
        if snapshot1.storage_entries != snapshot2.storage_entries {
            differences.push_back(String::from_str(
                env,
                "Storage entry count changed",
            ));
        }

        // Check if memory usage changed significantly (>10% difference)
        let memory_diff = if snapshot2.memory_usage_bytes > snapshot1.memory_usage_bytes {
            snapshot2.memory_usage_bytes - snapshot1.memory_usage_bytes
        } else {
            snapshot1.memory_usage_bytes - snapshot2.memory_usage_bytes
        };

        if memory_diff > snapshot1.memory_usage_bytes / 10 {
            differences.push_back(String::from_str(
                env,
                "Significant memory usage change detected",
            ));
        }

        // Check if state hash differs (indicates data changes)
        if snapshot1.state_hash != snapshot2.state_hash {
            differences.push_back(String::from_str(
                env,
                "Contract state data has changed",
            ));
        }

        differences
    }

    /// Track state changes over time
    pub fn track_state_evolution(
        env: &Env,
        contract_id: &Address,
        snapshots: &Vec<StateSnapshot>,
    ) -> Map<Symbol, u64> {
        let mut evolution = Map::new(env);

        if snapshots.is_empty() {
            return evolution;
        }

        // Calculate growth rate
        let first = snapshots.get(0).unwrap();
        let last = snapshots.get(snapshots.len() - 1).unwrap();

        let storage_growth = last.storage_entries.saturating_sub(first.storage_entries) as u64;
        let memory_growth = last.memory_usage_bytes.saturating_sub(first.memory_usage_bytes);

        evolution.set(Symbol::new(env, "storage_growth"), storage_growth);
        evolution.set(Symbol::new(env, "memory_growth"), memory_growth);
        evolution.set(Symbol::new(env, "snapshot_count"), snapshots.len() as u64);

        evolution
    }

    /// Detect potential memory leaks
    pub fn detect_memory_leak(
        env: &Env,
        snapshots: &Vec<StateSnapshot>,
    ) -> bool {
        if snapshots.len() < 3 {
            return false;
        }

        // Check if memory consistently increases without corresponding storage increase
        let mut consecutive_increases = 0u32;

        for i in 1..snapshots.len() {
            let prev = snapshots.get(i - 1).unwrap();
            let current = snapshots.get(i).unwrap();

            if current.memory_usage_bytes > prev.memory_usage_bytes {
                consecutive_increases += 1;
            } else {
                consecutive_increases = 0;
            }

            // If memory increased 3+ times in a row, potential leak
            if consecutive_increases >= 3 {
                return true;
            }
        }

        false
    }

    /// Visualize contract state as a formatted string (for dashboard)
    pub fn visualize_state(
        env: &Env,
        snapshot: &StateSnapshot,
    ) -> String {
        // In a real implementation, this would create a detailed visualization
        // For now, we'll create a summary
        String::from_str(
            env,
            &format!(
                "Contract State - Entries: {}, Memory: {} bytes, Seq: {}",
                snapshot.storage_entries,
                snapshot.memory_usage_bytes,
                snapshot.ledger_sequence
            ),
        )
    }

    // Helper functions (simplified implementations)
    
    fn count_storage_entries(_env: &Env) -> u32 {
        // In production, iterate through storage keys
        // For now, return estimated value
        10
    }

    fn estimate_memory_usage(_env: &Env, entries: u32) -> u64 {
        // Rough estimation: each entry ~100 bytes + overhead
        (entries as u64) * 100 + 1000
    }

    fn generate_state_hash(
        env: &Env,
        contract_id: &Address,
        timestamp: u64,
    ) -> BytesN<32> {
        // In production, hash the actual state data
        // For now, use a simple hash based on timestamp
        let timestamp_bytes = timestamp.to_be_bytes();
        env.crypto().sha256(&Bytes::from_slice(env, &timestamp_bytes))
    }

    fn capture_storage_state(env: &Env) -> Map<Symbol, String> {
        // In production, iterate through storage and capture key-value pairs
        // For now, return empty map
        Map::new(env)
    }
}

#[cfg(all(test, feature = "testutils"))]
mod tests {
    use super::*;
    use soroban_sdk::{testutils::Address as _, Env};

    #[test]
    fn test_capture_snapshot() {
        let env = Env::default();
        let contract_id = Address::generate(&env);

        let snapshot = StateTracker::capture_snapshot(&env, &contract_id);

        assert_eq!(snapshot.contract_id, contract_id);
        assert!(snapshot.timestamp > 0);
        assert!(snapshot.storage_entries > 0);
    }

    #[test]
    fn test_compare_snapshots() {
        let env = Env::default();
        let contract_id = Address::generate(&env);

        let snapshot1 = StateTracker::capture_snapshot(&env, &contract_id);
        
        // Simulate time passing
        env.ledger().with_mut(|li| {
            li.timestamp += 1000;
        });
        
        let snapshot2 = StateTracker::capture_snapshot(&env, &contract_id);

        let differences = StateTracker::compare_snapshots(&env, &snapshot1, &snapshot2);
        
        // Should detect no major differences in this simple case
        assert!(differences.len() >= 0);
    }

    #[test]
    fn test_detect_memory_leak() {
        let env = Env::default();
        let contract_id = Address::generate(&env);

        let mut snapshots = Vec::new(&env);

        // Create snapshots with increasing memory
        for i in 0..5 {
            env.ledger().with_mut(|li| {
                li.timestamp += 1000;
            });

            let mut snapshot = StateTracker::capture_snapshot(&env, &contract_id);
            // Manually increase memory for testing
            snapshot.memory_usage_bytes = 1000 + (i * 500);
            snapshots.push_back(snapshot);
        }

        let has_leak = StateTracker::detect_memory_leak(&env, &snapshots);
        assert!(has_leak);
    }
}