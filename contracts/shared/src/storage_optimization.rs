use soroban_sdk::{contracttype, Address, BytesN, Symbol, Vec, Map};

/// Compact storage optimization utilities
pub struct CompactStorage;

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::Env;

    #[test]
    fn test_packed_student_data() {
        let env = Env::default();
        
        // Test packing and unpacking
        let packed = PackedStudentData::pack_fields(
            85, // completion_pct
            120, // total_time_hours
            5, // interaction_level
            3, // performance_tier
            0, // flags
        );

        assert_eq!(packed >> 96 & 0xFFFFFFFF, 85);
        assert_eq!(packed >> 64 & 0xFFFFFFFF, 120);
        assert_eq!(packed >> 56 & 0xFF, 5);
        assert_eq!(packed >> 48 & 0xFF, 3);
    }

    #[test]
    fn test_compressed_session_collection() {
        let env = Env::default();
        
        let sessions = Vec::from_array(&env, [
            (1000, 1800, 3),
            (2000, 3600, 4),
            (3000, 2700, 2),
        ]);

        let compressed = CompressedSessionCollection::compress_sessions(&env, sessions.clone());
        assert_eq!(compressed.session_count, 3);
        assert_eq!(compressed.base_timestamp, 1000);

        let decompressed = compressed.decompress_sessions(&env);
        assert_eq!(decompressed.len(), 3);
    }

    #[test]
    fn test_bloom_filter() {
        let env = Env::default();
        let mut filter = CompactBloomFilter::new(&env, 100);
        
        let item1 = BytesN::from_array(&env, &[1; 32]);
        let item2 = BytesN::from_array(&env, &[2; 32]);
        
        filter.add(&env, &item1);
        assert!(filter.might_contain(&item1));
        
        filter.add(&env, &item2);
        assert!(filter.might_contain(&item2));
        assert!(filter.might_contain(&item1)); // Should still contain item1
    }

    #[test]
    fn test_compact_storage() {
        let env = Env::default();
        
        let packed_data = CompactStorage::optimize_student_data(
            75, 100, 4, 2, 1000, 2000, 10, 5, 80, 7
        );
        
        assert_eq!(packed_data.first_activity, 1000);
        assert_eq!(packed_data.last_activity, 2000);
        assert_eq!(packed_data.total_sessions, 10);
        assert_eq!(packed_data.completed_modules, 5);
        assert_eq!(packed_data.average_score, 80);
        assert_eq!(packed_data.streak_days, 7);
    }
}

/// Optimized storage key patterns
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum CompactKey {
    // Packed student data: (student_address, course_id) -> packed_data
    StudentCourseData(Address, Symbol),
    
    // Time-based buckets for efficient querying
    DailyBucket(Symbol, u64), // (course_id, date)
    WeeklyBucket(Symbol, u64), // (course_id, week)
    
    // Compact counters using bit packing
    PackedCounters(Symbol),
    
    // Bloom filter for existence checks
    ExistenceFilter(Symbol),
    
    // Compressed collections
    CompressedSessions(Address, Symbol),
    CompressedCertificates(Address),
    
    // Index mappings for efficient lookups
    SessionIndex(BytesN<32>), // session_id -> (student, course, timestamp)
    CertificateIndex(BytesN<32>), // cert_id -> (student, course, timestamp)
}

/// Packed student course data (32 bytes)
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PackedStudentData {
    // Bit-packed fields (16 bytes)
    pub packed_fields: u128,
    
    // Timestamps (8 bytes each)
    pub first_activity: u64,
    pub last_activity: u64,
    
    // Counters (4 bytes each)
    pub total_sessions: u32,
    pub completed_modules: u32,
    
    // Performance metrics (4 bytes each)
    pub average_score: u32,
    pub streak_days: u32,
}

impl PackedStudentData {
    pub fn pack_fields(
        completion_pct: u32,
        total_time_hours: u32,
        interaction_level: u8,
        performance_tier: u8,
        flags: u8,
    ) -> u128 {
        // Pack multiple fields into 128 bits
        ((completion_pct as u128) << 96) |
        ((total_time_hours as u128) << 64) |
        ((interaction_level as u128) << 56) |
        ((performance_tier as u128) << 48) |
        ((flags as u128) << 40)
    }
    
    pub fn unpack_completion_percentage(&self) -> u32 {
        ((self.packed_fields >> 96) & 0xFFFFFFFF) as u32
    }
    
    pub fn unpack_total_time_hours(&self) -> u32 {
        ((self.packed_fields >> 64) & 0xFFFFFFFF) as u32
    }
    
    pub fn unpack_interaction_level(&self) -> u8 {
        ((self.packed_fields >> 56) & 0xFF) as u8
    }
    
    pub fn unpack_performance_tier(&self) -> u8 {
        ((self.packed_fields >> 48) & 0xFF) as u8
    }
}

/// Compressed session collection using delta encoding
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CompressedSessionCollection {
    pub base_timestamp: u64,
    pub delta_encoded_durations: Vec<u32>,
    pub packed_metadata: Vec<u64>, // Each u64 packs multiple session metadata
    pub session_count: u32,
}

impl CompressedSessionCollection {
    pub fn compress_sessions(env: &soroban_sdk::Env, sessions: Vec<(u64, u32, u8)>) -> Self {
        if sessions.is_empty() {
            return CompressedSessionCollection {
                base_timestamp: 0,
                delta_encoded_durations: Vec::new(env),
                packed_metadata: Vec::new(env),
                session_count: 0,
            };
        }
        
        let base_timestamp = sessions.first().unwrap().0;
        let mut deltas = Vec::new(env);
        let mut packed = Vec::new(env);
        
        for (timestamp, duration, score_tier) in sessions.iter() {
            let delta = timestamp.saturating_sub(base_timestamp) as u32;
            deltas.push_back(delta);
            
            // Pack duration and score tier into u64
            let packed_value = ((duration as u64) << 8) | (score_tier as u64);
            packed.push_back(packed_value);
        }
        
        CompressedSessionCollection {
            base_timestamp,
            delta_encoded_durations: deltas,
            packed_metadata: packed,
            session_count: sessions.len() as u32,
        }
    }
    
    pub fn decompress_sessions(&self, env: &soroban_sdk::Env) -> Vec<(u64, u32, u8)> {
        let mut sessions = Vec::new(env);
        
        for i in 0..self.session_count {
            let delta = self.delta_encoded_durations.get(i as u32).unwrap_or(0);
            let timestamp = self.base_timestamp.saturating_add(delta as u64);
            
            let packed = self.packed_metadata.get(i as u32).unwrap_or(0);
            let duration = (packed >> 8) as u32;
            let score_tier = (packed & 0xFF) as u8;
            
            sessions.push_back((timestamp, duration, score_tier));
        }
        
        sessions
    }
}

/// Bloom filter for efficient existence checks
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CompactBloomFilter {
    pub bit_vector: Vec<u64>,
    pub hash_count: u8,
    pub item_count: u32,
}

impl CompactBloomFilter {
    pub fn new(env: &soroban_sdk::Env, expected_items: u32) -> Self {
        let bit_count = Self::optimal_bit_count(expected_items);
        let hash_count = Self::optimal_hash_count(expected_items, bit_count);
        
        let bit_vector = Vec::new(env);
        
        CompactBloomFilter {
            bit_vector,
            hash_count,
            item_count: 0,
        }
    }
    
    fn optimal_bit_count(expected_items: u32) -> u32 {
        // m = -n * ln(p) / ln(2)^2, where p = 0.01 (1% false positive rate)
        let result = (expected_items as f64 * 9.585) as u32;
        if result == 0 {
            1
        } else if result.is_power_of_two() {
            result
        } else {
            result.next_power_of_two()
        }
    }
    
    fn optimal_hash_count(n: u32, m: u32) -> u8 {
        // k = (m/n) * ln(2)
        ((m as f64 / n as f64) * 0.693) as u8
    }
    
    pub fn add(&mut self, env: &soroban_sdk::Env, item: &BytesN<32>) {
        // Ensure bit vector has enough capacity
        let bit_count = Self::optimal_bit_count(1000); // Default capacity
        let required_words = (bit_count + 63) / 64;
        
        while self.bit_vector.len() < required_words {
            self.bit_vector.push_back(0);
        }
        
        // Simple hash implementation for demonstration
        let hash1 = self.hash(item, 0);
        let hash2 = self.hash(item, 1);
        
        for i in 0..self.hash_count {
            let hash = hash1.wrapping_add((i as u64).wrapping_mul(hash2));
            let bit_index = (hash % (self.bit_vector.len() as u64 * 64)) as u32;
            let word_index = bit_index / 64;
            let bit_offset = bit_index % 64;
            
            if word_index < self.bit_vector.len() as u32 {
                let mut word = self.bit_vector.get(word_index).unwrap_or(0);
                word |= 1u64 << bit_offset;
                self.bit_vector.set(word_index, word);
            }
        }
        
        self.item_count += 1;
    }
    
    pub fn might_contain(&self, item: &BytesN<32>) -> bool {
        let hash1 = self.hash(item, 0);
        let hash2 = self.hash(item, 1);
        
        for i in 0..self.hash_count {
            let hash = hash1.wrapping_add((i as u64).wrapping_mul(hash2));
            let bit_index = (hash % (self.bit_vector.len() as u64 * 64)) as u32;
            let word_index = bit_index / 64;
            let bit_offset = bit_index % 64;
            
            if word_index >= self.bit_vector.len() as u32 {
                return false;
            }
            
            let word = self.bit_vector.get(word_index).unwrap_or(0);
            if (word & (1u64 << bit_offset)) == 0 {
                return false;
            }
        }
        
        true
    }
    
    fn hash(&self, item: &BytesN<32>, seed: u8) -> u64 {
        // Simple hash function - in production, use a proper hash like xxHash
        let mut hash = seed as u64;
        for (i, &byte) in item.as_bytes().iter().enumerate() {
            hash = hash.wrapping_mul(31).wrapping_add(byte as u64).wrapping_add(i as u64);
        }
        hash
    }
}

/// Time-based bucket for efficient range queries
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct TimeBucket {
    pub bucket_start: u64,
    pub bucket_end: u64,
    pub compressed_data: Vec<u64>, // Compressed session/certificate data
    pub item_count: u32,
    pub checksum: u32,
}

impl TimeBucket {
    pub fn new(env: &soroban_sdk::Env, start_time: u64, duration_hours: u32) -> Self {
        TimeBucket {
            bucket_start: start_time,
            bucket_end: start_time.saturating_add(duration_hours as u64 * 3600),
            compressed_data: Vec::new(env),
            item_count: 0,
            checksum: 0,
        }
    }
    
    pub fn add_item(&mut self, timestamp: u64, value: u32) {
        // Delta compression for timestamps
        let delta = timestamp.saturating_sub(self.bucket_start) as u32;
        
        // Pack delta and value into u64
        let packed = ((delta as u64) << 32) | (value as u64);
        self.compressed_data.push_back(packed);
        self.item_count += 1;
        
        // Update simple checksum
        self.checksum = self.checksum.wrapping_add(value).wrapping_add(delta);
    }
    
    pub fn verify_integrity(&self) -> bool {
        if self.item_count == 0 {
            return self.checksum == 0;
        }
        
        let mut calculated_checksum = 0u32;
        for packed in self.compressed_data.iter() {
            let delta = (packed >> 32) as u32;
            let value = (packed & 0xFFFFFFFF) as u32;
            calculated_checksum = calculated_checksum.wrapping_add(value).wrapping_add(delta);
        }
        
        calculated_checksum == self.checksum
    }
}

/// Storage optimization utilities
impl CompactStorage {
    /// Optimize student course data storage
    pub fn optimize_student_data(
        completion_pct: u32,
        total_time_hours: u32,
        interaction_level: u8,
        performance_tier: u8,
        first_activity: u64,
        last_activity: u64,
        total_sessions: u32,
        completed_modules: u32,
        average_score: u32,
        streak_days: u32,
    ) -> PackedStudentData {
        let flags = 0u8; // Reserved for future use
        
        PackedStudentData {
            packed_fields: PackedStudentData::pack_fields(
                completion_pct,
                total_time_hours,
                interaction_level,
                performance_tier,
                flags,
            ),
            first_activity,
            last_activity,
            total_sessions,
            completed_modules,
            average_score,
            streak_days,
        }
    }
    
    /// Create time bucket for efficient range queries
    pub fn create_time_bucket(
        env: &soroban_sdk::Env,
        start_time: u64,
        duration_hours: u32,
    ) -> TimeBucket {
        TimeBucket::new(env, start_time, duration_hours)
    }
    
    /// Compress session collection
    pub fn compress_sessions(
        env: &soroban_sdk::Env,
        sessions: Vec<(u64, u32, u8)>,
    ) -> CompressedSessionCollection {
        CompressedSessionCollection::compress_sessions(env, sessions)
    }
    
    /// Create bloom filter for existence checks
    pub fn create_bloom_filter(
        env: &soroban_sdk::Env,
        expected_items: u32,
    ) -> CompactBloomFilter {
        CompactBloomFilter::new(env, expected_items)
    }
    
    /// Calculate storage savings estimate
    pub fn estimate_storage_savings(
        original_items: u32,
        item_size_bytes: u32,
        compression_ratio: f32,
    ) -> (u32, u32) {
        let original_size = original_items * item_size_bytes;
        let compressed_size = (original_size as f32 * compression_ratio) as u32;
        let savings = original_size.saturating_sub(compressed_size);
        
        (original_size, savings)
    }
}
