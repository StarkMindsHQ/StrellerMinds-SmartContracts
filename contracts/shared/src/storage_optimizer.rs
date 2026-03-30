//! Storage optimization utilities for StrellerMinds contracts
//! 
//! This module provides optimized storage patterns and data structures
//! to minimize gas costs associated with storage operations.

#![no_std]

use soroban_sdk::{
    contracttype, Address, BytesN, Env, Map, Symbol, Vec, 
    storage::{Storage, InstanceStorage, PersistentStorage}
};

/// Optimized storage key patterns to reduce gas costs
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum OptimizedKey {
    /// Single byte prefix for common operations
    Admin,
    Config,
    Counter(u8), // 0-255 for different counters
    /// Compact indexing with u16 instead of u64 where possible
    Index(u16),
    /// Batch operation keys
    Batch(u32),
    /// User-specific data with compact addressing
    User(Address, u8), // u8 for data type
    /// Time-based data with day precision (sufficient for most analytics)
    TimeBucket(u32), // days since epoch
}

/// Packed storage for multiple small values
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PackedData {
    /// Bit-packed boolean flags
    pub flags: u32, // 32 boolean flags in one u32
    /// Compact counters (each 8 bits)
    pub counters: [u8; 4], // 4 small counters
    /// Timestamp (last modified)
    pub timestamp: u32,
}

/// Optimized batch operation result
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BatchResult {
    pub successful: u32,
    pub failed: u32,
    pub gas_saved: u64,
}

/// Storage optimization utilities
pub struct StorageOptimizer;

impl StorageOptimizer {
    /// Pack multiple boolean values into a single u32 storage slot
    pub fn pack_flags(flags: &[bool]) -> u32 {
        let mut packed = 0u32;
        for (i, flag) in flags.iter().enumerate().take(32) {
            if *flag {
                packed |= 1 << i;
            }
        }
        packed
    }

    /// Unpack boolean flags from a u32 storage slot
    pub fn unpack_flags(packed: u32, count: usize) -> Vec<bool> {
        let mut flags = Vec::new(&Env::default()); // Note: This would need proper env in real usage
        for i in 0..count.min(32) {
            flags.push_back((packed & (1 << i)) != 0);
        }
        flags
    }

    /// Store multiple small counters in a single storage operation
    pub fn store_packed_counters(env: &Env, key: OptimizedKey, counters: &[u32]) {
        let packed: [u8; 4] = [
            counters.get(0).unwrap_or(&0).min(255) as u8,
            counters.get(1).unwrap_or(&0).min(255) as u8,
            counters.get(2).unwrap_or(&0).min(255) as u8,
            counters.get(3).unwrap_or(&0).min(255) as u8,
        ];
        
        env.storage().persistent().set(&key, &packed);
    }

    /// Retrieve packed counters from storage
    pub fn get_packed_counters(env: &Env, key: OptimizedKey) -> [u8; 4] {
        env.storage().persistent()
            .get(&key)
            .unwrap_or([0, 0, 0, 0])
    }

    /// Batch write multiple values to storage efficiently
    pub fn batch_write(env: &Env, writes: &Map<OptimizedKey, soroban_sdk::Bytes>) -> BatchResult {
        let mut successful = 0;
        let mut failed = 0;
        
        // Group writes by storage type for optimization
        let mut persistent_writes = Map::new(env);
        let mut instance_writes = Map::new(env);
        
        for (key, value) in writes.iter() {
            match key {
                OptimizedKey::Admin | OptimizedKey::Config | OptimizedKey::Counter(_) => {
                    instance_writes.set(key.clone(), value.clone());
                }
                _ => {
                    persistent_writes.set(key.clone(), value.clone());
                }
            }
        }
        
        // Execute batched writes
        for (key, value) in instance_writes.iter() {
            env.storage().instance().set(&key, &value);
            successful += 1;
        }
        
        for (key, value) in persistent_writes.iter() {
            env.storage().persistent().set(&key, &value);
            successful += 1;
        }
        
        BatchResult {
            successful,
            failed,
            gas_saved: (writes.len() as u32 - successful) as u64 * 1000, // Estimate
        }
    }

    /// Optimized counter increment with reduced storage operations
    pub fn increment_counter(env: &Env, counter_id: u8) -> u64 {
        let key = OptimizedKey::Counter(counter_id);
        let current: u64 = env.storage().instance()
            .get(&key)
            .unwrap_or(0);
        let next = current + 1;
        env.storage().instance().set(&key, &next);
        next
    }

    /// Time-based storage bucketing for analytics (daily buckets)
    pub fn get_time_bucket(timestamp: u64) -> u32 {
        (timestamp / (24 * 60 * 60)) as u32 // Days since epoch
    }

    /// Store time-series data in daily buckets
    pub fn store_time_bucketed(env: &Env, user: &Address, data_type: u8, value: &soroban_sdk::Bytes) {
        let bucket = Self::get_time_bucket(env.ledger().timestamp());
        let key = OptimizedKey::TimeBucket(bucket);
        let mut bucket_data: Map<(Address, u8), soroban_sdk::Bytes> = env.storage().persistent()
            .get(&key)
            .unwrap_or(Map::new(env));
        
        bucket_data.set((user.clone(), data_type), value.clone());
        env.storage().persistent().set(&key, &bucket_data);
    }

    /// Retrieve time-bucketed data for analytics
    pub fn get_time_bucketed(env: &Env, bucket: u32, user: &Address, data_type: u8) -> Option<soroban_sdk::Bytes> {
        let key = OptimizedKey::TimeBucket(bucket);
        let bucket_data: Map<(Address, u8), soroban_sdk::Bytes> = env.storage().persistent()
            .get(&key)?;
        
        bucket_data.get((user.clone(), data_type))
    }

    /// Compact address storage (useful for large user bases)
    pub fn store_compact_address_list(env: &Env, key: OptimizedKey, addresses: &[Address]) {
        // Store addresses in a compact format
        let mut compact_data = Vec::new(env);
        for addr in addresses {
            let addr_bytes = addr.to_xdr(env);
            compact_data.push_back(addr_bytes);
        }
        env.storage().persistent().set(&key, &compact_data);
    }

    /// Retrieve compact address list
    pub fn get_compact_address_list(env: &Env, key: OptimizedKey) -> Vec<Address> {
        let compact_data: Vec<soroban_sdk::Bytes> = env.storage().persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));
        
        let mut addresses = Vec::new(env);
        for addr_bytes in compact_data.iter() {
            if let Ok(addr) = Address::from_xdr(env, &addr_bytes) {
                addresses.push_back(addr);
            }
        }
        addresses
    }

    /// Lazy loading pattern for large data structures
    pub fn lazy_load<T: contracttype>(env: &Env, key: OptimizedKey, loader: impl FnOnce() -> T) -> T {
        if let Some(data) = env.storage().persistent().get::<_, T>(&key) {
            data
        } else {
            let data = loader();
            env.storage().persistent().set(&key, &data);
            data
        }
    }

    /// Cache frequently accessed data in instance storage
    pub fn cache_in_instance<T: contracttype>(env: &Env, persistent_key: OptimizedKey, cache_key: OptimizedKey, ttl: u32) {
        if let Some(data) = env.storage().persistent().get::<_, T>(&persistent_key) {
            env.storage().instance().set(&cache_key, &data);
            // Store expiry timestamp
            let expiry = env.ledger().timestamp() + ttl as u64;
            env.storage().instance().set(&OptimizedKey::TimeBucket(expiry as u32), &cache_key);
        }
    }

    /// Get cached data if still valid
    pub fn get_cached<T: contracttype>(env: &Env, cache_key: OptimizedKey) -> Option<T> {
        // Check if cache is still valid (simplified - in practice would check expiry)
        env.storage().instance().get(&cache_key)
    }

    /// Batch cleanup of expired cached data
    pub fn cleanup_expired_cache(env: &Env) -> u32 {
        let mut cleaned = 0;
        let current_time = env.ledger().timestamp();
        
        // This is a simplified version - in practice would need more sophisticated tracking
        // For now, we'll just clean up obvious expired entries
        if let Some(expired_keys) = env.storage().instance().get::<_, Vec<OptimizedKey>>(&Symbol::short("cache_expiry")) {
            for key in expired_keys.iter() {
                if let OptimizedKey::TimeBucket(expiry_time) = key {
                    if *expiry_time < current_time as u32 {
                        env.storage().instance().remove(key);
                        cleaned += 1;
                    }
                }
            }
        }
        
        cleaned
    }
}

/// Gas-optimized storage patterns for common use cases
pub struct OptimizedPatterns;

impl OptimizedPatterns {
    /// Optimized user profile storage
    pub fn store_user_profile(env: &Env, user: &Address, profile_data: &PackedData) {
        let key = OptimizedKey::User(user.clone(), 0); // 0 = profile data type
        env.storage().persistent().set(&key, profile_data);
    }

    /// Optimized activity tracking with daily buckets
    pub fn track_user_activity(env: &Env, user: &Address, activity_type: u8) {
        StorageOptimizer::store_time_bucketed(env, user, activity_type, &soroban_sdk::Bytes::from_array(env, &[1]));
    }

    /// Optimized counter for analytics
    pub fn increment_analytics_counter(env: &Env, metric: u8) {
        StorageOptimizer::increment_counter(env, metric);
    }

    /// Batch update multiple user metrics
    pub fn batch_update_metrics(env: &Env, updates: &Map<(Address, u8), u32>) -> BatchResult {
        let mut writes = Map::new(env);
        
        for ((user, metric_type), value) in updates.iter() {
            let key = OptimizedKey::User(user.clone(), *metric_type);
            let value_bytes = soroban_sdk::Bytes::from_array(env, &value.to_be_bytes());
            writes.set(key, value_bytes);
        }
        
        StorageOptimizer::batch_write(env, &writes)
    }
}
