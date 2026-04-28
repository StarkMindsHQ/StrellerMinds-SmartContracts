use soroban_sdk::{contracttype, Address, Bytes, String, Vec};

// ─────────────────────────────────────────────────────────────
// API Key Status
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApiKeyStatus {
    Active,
    Pending,      // New key awaiting activation
    Deprecated,   // Being phased out
    Revoked,      // No longer valid
}

// ─────────────────────────────────────────────────────────────
// API Key Record
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiKeyRecord {
    pub key_id: String,
    pub key_hash: Bytes,
    pub owner: Address,
    pub created_at: u64,
    pub expires_at: u64,
    pub status: ApiKeyStatus,
    pub rotation_count: u32,
    pub last_used_at: u64,
    pub permissions: Vec<String>,
}

// ─────────────────────────────────────────────────────────────
// API Key Rotation Config
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApiKeyRotationConfig {
    pub rotation_interval: u64,    // Seconds between rotations
    pub grace_period: u64,         // Seconds old key remains valid
    pub max_keys_per_user: u32,    // Maximum concurrent keys
    pub auto_rotate: bool,         // Enable automatic rotation
    pub alert_before_expiry: u64,  // Seconds before expiry to alert
}

// ─────────────────────────────────────────────────────────────
// Rotation Event
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct KeyRotationEvent {
    pub key_id: String,
    pub new_key_id: String,
    pub rotated_by: Address,
    pub timestamp: u64,
    pub reason: String,
}

// ─────────────────────────────────────────────────────────────
// Storage Keys
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApiKeyDataKey {
    RotationConfig,
    ApiKey(String),               // key_id -> ApiKeyRecord
    UserKeys(Address),            // owner -> Vec<key_id>
    ActiveKeys,                   // Vec<key_id>
    RotationHistory(String),      // key_id -> Vec<KeyRotationEvent>
    KeyCounter,                   // Global counter for key IDs
}
