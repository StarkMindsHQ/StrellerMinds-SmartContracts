use soroban_sdk::{contracttype, Address, BytesN, Symbol, Vec, Map, Env};

/// Compact progress tracking using packed data structures
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CompactProgress {
    /// Packed module progress data
    /// Format: [module_id_16bits][progress_16bits][module_id_16bits][progress_16bits]...
    pub packed_data: Vec<u64>,
    /// Number of modules tracked
    pub module_count: u32,
}

impl CompactProgress {
    pub fn new(env: &Env) -> Self {
        Self {
            packed_data: Vec::new(env),
            module_count: 0,
        }
    }

    pub fn set_progress(&mut self, module_id: u16, progress: u32) {
        if progress > 100 {
            panic!("Progress cannot exceed 100");
        }

        // Check if module already exists
        for i in 0..self.packed_data.len() {
            let packed = self.packed_data.get(i).unwrap();
            let existing_module_id = (packed >> 48) as u16;
            
            if existing_module_id == module_id {
                // Update existing entry
                let new_packed = ((module_id as u64) << 48) | ((progress as u64) << 32) | (packed & 0xFFFFFFFF);
                self.packed_data.set(i, new_packed);
                return;
            }
        }

        // Add new entry
        let packed = ((module_id as u64) << 48) | ((progress as u64) << 32);
        self.packed_data.push_back(packed);
        self.module_count += 1;
    }

    pub fn get_progress(&self, module_id: u16) -> Option<u32> {
        for packed in self.packed_data.iter() {
            let existing_module_id = (packed >> 48) as u16;
            if existing_module_id == module_id {
                return Some(((packed >> 32) & 0xFFFF) as u32);
            }
        }
        None
    }

    pub fn get_all_progress(&self, env: &Env) -> Map<Symbol, u32> {
        let mut result = Map::new(env);
        for packed in self.packed_data.iter() {
            let module_id = (packed >> 48) as u16;
            let progress = ((packed >> 32) & 0xFFFF) as u32;
            // Convert module_id to string and then to symbol
            let module_str = format!("{}", module_id);
            let symbol = Symbol::new(env, &module_str);
            result.set(symbol, progress);
        }
        result
    }
}

/// Compact session data using bit packing for efficiency
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CompactSession {
    /// Packed session data: 
    /// - First 8 bytes: start_time (u64)
    /// - Next 4 bytes: duration_minutes (u32) 
    /// - Next 2 bytes: completion_percentage (u16)
    /// - Next 2 bytes: interaction_count (u16)
    /// - Final 1 byte: session_type + flags
    pub packed_data: BytesN<17>,
    pub student: Address,
    pub course_id: Symbol,
    pub module_id: Symbol,
}

impl CompactSession {
    pub fn pack(
        start_time: u64,
        duration_minutes: u32,
        completion_percentage: u32,
        interaction_count: u32,
        session_type: u8,
        flags: u8,
    ) -> BytesN<17> {
        // This is a simplified packing - in production you'd use proper bit manipulation
        let mut data = [0u8; 17];
        
        // Pack start_time (8 bytes)
        data[0..8].copy_from_slice(&start_time.to_be_bytes());
        
        // Pack duration_minutes (4 bytes)
        data[8..12].copy_from_slice(&duration_minutes.to_be_bytes());
        
        // Pack completion_percentage as u16 (2 bytes)
        data[12..14].copy_from_slice(&(completion_percentage as u16).to_be_bytes());
        
        // Pack interaction_count as u16 (2 bytes)
        data[14..16].copy_from_slice(&(interaction_count as u16).to_be_bytes());
        
        // Pack session_type + flags (1 byte)
        data[16] = (session_type & 0x0F) | ((flags & 0x0F) << 4);
        
        BytesN::from_array(&data)
    }
    
    pub fn unpack_start_time(packed: &BytesN<17>) -> u64 {
        let mut bytes = [0u8; 8];
        bytes.copy_from_slice(&packed.as_array()[0..8]);
        u64::from_be_bytes(bytes)
    }
    
    pub fn unpack_duration(packed: &BytesN<17>) -> u32 {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&packed.as_array()[8..12]);
        u32::from_be_bytes(bytes)
    }
    
    pub fn unpack_completion(packed: &BytesN<17>) -> u32 {
        let mut bytes = [0u8; 2];
        bytes.copy_from_slice(&packed.as_array()[12..14]);
        u16::from_be_bytes(bytes) as u32
    }
    
    pub fn unpack_interactions(packed: &BytesN<17>) -> u32 {
        let mut bytes = [0u8; 2];
        bytes.copy_from_slice(&packed.as_array()[14..16]);
        u16::from_be_bytes(bytes) as u32
    }
    
    pub fn unpack_session_type(packed: &BytesN<17>) -> u8 {
        packed.as_array()[16] & 0x0F
    }
    
    pub fn unpack_flags(packed: &BytesN<17>) -> u8 {
        (packed.as_array()[16] >> 4) & 0x0F
    }
}

/// Compact analytics data using bit packing
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CompactAnalytics {
    /// Packed analytics metrics:
    /// - First 4 bytes: total_sessions (u32)
    /// - Next 4 bytes: completed_modules (u32)
    /// - Next 4 bytes: total_time_hours (u32)
    /// - Next 2 bytes: streak_days (u16)
    /// - Next 2 bytes: average_score (u16, scaled by 100)
    /// - Final 1 byte: flags + trend
    pub packed_metrics: BytesN<17>,
    pub student: Address,
    pub course_id: Symbol,
    pub last_activity: u64,
}

impl CompactAnalytics {
    pub fn pack(
        total_sessions: u32,
        completed_modules: u32,
        total_time_hours: u32,
        streak_days: u32,
        average_score: Option<u32>,
        trend: u8,
        flags: u8,
    ) -> BytesN<17> {
        let mut data = [0u8; 17];
        
        // Pack metrics
        data[0..4].copy_from_slice(&total_sessions.to_be_bytes());
        data[4..8].copy_from_slice(&completed_modules.to_be_bytes());
        data[8..12].copy_from_slice(&total_time_hours.to_be_bytes());
        data[12..14].copy_from_slice(&(streak_days as u16).to_be_bytes());
        
        // Pack average score (scaled by 100 to fit in u16)
        let score_scaled = (average_score.unwrap_or(0) / 100) as u16;
        data[14..16].copy_from_slice(&score_scaled.to_be_bytes());
        
        // Pack trend + flags
        data[16] = (trend & 0x0F) | ((flags & 0x0F) << 4);
        
        BytesN::from_array(&data)
    }
    
    pub fn unpack_total_sessions(packed: &BytesN<17>) -> u32 {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&packed.as_array()[0..4]);
        u32::from_be_bytes(bytes)
    }
    
    pub fn unpack_completed_modules(packed: &BytesN<17>) -> u32 {
        let mut bytes = [0u8; 4];
        bytes.copy_from_slice(&packed.as_array()[4..8]);
        u32::from_be_bytes(bytes)
    }
    
    pub fn unpack_average_score(packed: &BytesN<17>) -> Option<u32> {
        let mut bytes = [0u8; 2];
        bytes.copy_from_slice(&packed.as_array()[14..16]);
        let score_scaled = u16::from_be_bytes(bytes);
        if score_scaled == 0 {
            None
        } else {
            Some(score_scaled as u32 * 100)
        }
    }
}

/// Compact achievement storage using bit flags
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CompactAchievement {
    /// Packed achievement data:
    /// - First 4 bytes: achievement_id (as u32 hash)
    /// - Next 4 bytes: earned_date (timestamp / 86400 to fit in u32)
    /// - Next 2 bytes: achievement_type + rarity
    /// - Final 2 bytes: flags + points
    pub packed_data: BytesN<12>,
    pub student: Address,
}

impl CompactAchievement {
    pub fn pack(
        achievement_id: u32,
        earned_date: u64,
        achievement_type: u8,
        rarity: u8,
        flags: u8,
        points: u16,
    ) -> BytesN<12> {
        let mut data = [0u8; 12];
        
        // Pack achievement_id
        data[0..4].copy_from_slice(&achievement_id.to_be_bytes());
        
        // Pack earned_date as days since epoch (fits in u32)
        data[4..8].copy_from_slice(&((earned_date / 86400) as u32).to_be_bytes());
        
        // Pack type + rarity
        data[8] = (achievement_type & 0x0F) | ((rarity & 0x0F) << 4);
        
        // Pack flags + points
        data[9] = flags;
        data[10..12].copy_from_slice(&points.to_be_bytes());
        
        BytesN::from_array(&data)
    }
}

/// Compact certificate data
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CompactCertificate {
    /// Packed certificate data:
    /// - First 4 bytes: certificate_id (as u32 hash)
    /// - Next 4 bytes: issued_date (timestamp / 86400)
    /// - Next 4 bytes: expiry_date (timestamp / 86400)
    /// - Final 1 byte: status + priority
    pub packed_data: BytesN<13>,
    pub student: Address,
    pub course_id: Symbol,
}

impl CompactCertificate {
    pub fn pack(
        certificate_id: u32,
        issued_date: u64,
        expiry_date: u64,
        status: u8,
        priority: u8,
    ) -> BytesN<13> {
        let mut data = [0u8; 13];
        
        data[0..4].copy_from_slice(&certificate_id.to_be_bytes());
        data[4..8].copy_from_slice(&((issued_date / 86400) as u32).to_be_bytes());
        data[8..12].copy_from_slice(&((expiry_date / 86400) as u32).to_be_bytes());
        data[12] = (status & 0x0F) | ((priority & 0x0F) << 4);
        
        BytesN::from_array(&data)
    }
}

/// Compact index structure for efficient lookups
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CompactIndex {
    /// Bitmap for quick existence checks (256 bits = 32 bytes)
    pub bitmap: BytesN<32>,
    /// Count of items in the index
    pub count: u32,
    /// Last updated timestamp
    pub last_updated: u64,
}

impl CompactIndex {
    pub fn new() -> Self {
        Self {
            bitmap: BytesN::from_array(&[0u8; 32]),
            count: 0,
            last_updated: 0,
        }
    }
    
    pub fn set_bit(&mut self, index: u32) {
        if index < 256 {
            let byte_index = (index / 8) as usize;
            let bit_index = index % 8;
            self.bitmap.as_array_mut()[byte_index] |= 1 << bit_index;
            self.count += 1;
        }
    }
    
    pub fn has_bit(&self, index: u32) -> bool {
        if index < 256 {
            let byte_index = (index / 8) as usize;
            let bit_index = index % 8;
            (self.bitmap.as_array()[byte_index] & (1 << bit_index)) != 0
        } else {
            false
        }
    }
    
    pub fn clear_bit(&mut self, index: u32) {
        if index < 256 && self.has_bit(index) {
            let byte_index = (index / 8) as usize;
            let bit_index = index % 8;
            self.bitmap.as_array_mut()[byte_index] &= !(1 << bit_index);
            self.count -= 1;
        }
    }
}

/// Compact storage key optimization
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CompactStorageKey {
    /// Hash of the full key for compact storage
    pub key_hash: BytesN<32>,
    /// Type identifier for the stored data
    pub data_type: u8,
    /// Creation timestamp for cleanup purposes
    pub created_at: u64,
}
