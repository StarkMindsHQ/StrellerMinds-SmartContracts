use soroban_sdk::{Address, Bytes, Env, String, Vec};

use crate::api_key_types::{
    ApiKeyDataKey, ApiKeyRecord, ApiKeyRotationConfig, ApiKeyStatus, KeyRotationEvent,
};

// Default configuration values
const DEFAULT_ROTATION_INTERVAL: u64 = 7_776_000; // 90 days
const DEFAULT_GRACE_PERIOD: u64 = 604_800; // 7 days
const DEFAULT_MAX_KEYS_PER_USER: u32 = 3;
const DEFAULT_ALERT_BEFORE_EXPIRY: u64 = 2_592_000; // 30 days

// ─────────────────────────────────────────────────────────────
// Configuration Management
// ─────────────────────────────────────────────────────────────
pub fn set_rotation_config(env: &Env, config: &ApiKeyRotationConfig) {
    env.storage()
        .instance()
        .set(&ApiKeyDataKey::RotationConfig, config);
}

pub fn get_rotation_config(env: &Env) -> ApiKeyRotationConfig {
    env.storage()
        .instance()
        .get(&ApiKeyDataKey::RotationConfig)
        .unwrap_or(ApiKeyRotationConfig {
            rotation_interval: DEFAULT_ROTATION_INTERVAL,
            grace_period: DEFAULT_GRACE_PERIOD,
            max_keys_per_user: DEFAULT_MAX_KEYS_PER_USER,
            auto_rotate: false,
            alert_before_expiry: DEFAULT_ALERT_BEFORE_EXPIRY,
        })
}

// ─────────────────────────────────────────────────────────────
// API Key Management
// ─────────────────────────────────────────────────────────────
pub fn create_api_key(
    env: &Env,
    owner: &Address,
    key_hash: &Bytes,
    permissions: &Vec<String>,
) -> String {
    let config = get_rotation_config(env);
    let key_id = generate_key_id(env);

    let now = env.ledger().timestamp();
    let expires_at = now + config.rotation_interval;

    let key_record = ApiKeyRecord {
        key_id: key_id.clone(),
        key_hash: key_hash.clone(),
        owner: owner.clone(),
        created_at: now,
        expires_at,
        status: ApiKeyStatus::Active,
        rotation_count: 0,
        last_used_at: now,
        permissions: permissions.clone(),
    };

    // Store key
    env.storage()
        .persistent()
        .set(&ApiKeyDataKey::ApiKey(key_id.clone()), &key_record);

    // Add to user's keys
    add_user_key(env, owner, &key_id);

    // Add to active keys
    add_active_key(env, &key_id);

    key_id
}

pub fn get_api_key(env: &Env, key_id: &String) -> Option<ApiKeyRecord> {
    env.storage().persistent().get(&ApiKeyDataKey::ApiKey(key_id.clone()))
}

pub fn validate_api_key(env: &Env, key_id: &String, key_hash: &Bytes) -> bool {
    if let Some(key_record) = get_api_key(env, key_id) {
        // Check if key exists and hash matches
        if key_record.key_hash != *key_hash {
            return false;
        }

        // Check status
        if key_record.status != ApiKeyStatus::Active
            && key_record.status != ApiKeyStatus::Deprecated
        {
            return false;
        }

        // Check expiry (allow deprecated keys within grace period)
        let now = env.ledger().timestamp();
        let config = get_rotation_config(env);

        if key_record.status == ApiKeyStatus::Active && now > key_record.expires_at {
            return false;
        }

        if key_record.status == ApiKeyStatus::Deprecated
            && now > key_record.expires_at + config.grace_period
        {
            return false;
        }

        // Update last used
        let mut updated = key_record;
        updated.last_used_at = now;
        env.storage()
            .persistent()
            .set(&ApiKeyDataKey::ApiKey(key_id.clone()), &updated);

        true
    } else {
        false
    }
}

// ─────────────────────────────────────────────────────────────
// Key Rotation
// ─────────────────────────────────────────────────────────────
pub fn rotate_api_key(
    env: &Env,
    owner: &Address,
    old_key_id: &String,
    new_key_hash: &Bytes,
    reason: &String,
) -> Result<String, String> {
    let config = get_rotation_config(env);

    // Get old key
    let old_key = get_api_key(env, old_key_id)
        .ok_or("Old key not found".to_string())?;

    // Verify ownership
    if old_key.owner != *owner {
        return Err("Unauthorized: key owner mismatch".to_string());
    }

    // Check if old key is active
    if old_key.status != ApiKeyStatus::Active {
        return Err("Old key is not active".to_string());
    }

    // Check max keys limit
    let user_keys = get_user_keys(env, owner);
    if user_keys.len() >= config.max_keys_per_user {
        return Err("Maximum number of keys reached".to_string());
    }

    // Create new key
    let new_key_id = create_api_key(env, owner, new_key_hash, &old_key.permissions);

    // Deprecate old key (grace period)
    let mut deprecated_key = old_key;
    deprecated_key.status = ApiKeyStatus::Deprecated;
    env.storage()
        .persistent()
        .set(&ApiKeyDataKey::ApiKey(old_key_id.clone()), &deprecated_key);

    // Record rotation event
    record_rotation_event(
        env,
        old_key_id,
        &new_key_id,
        owner,
        reason,
    );

    Ok(new_key_id)
}

pub fn revoke_api_key(env: &Env, owner: &Address, key_id: &String) -> Result<(), String> {
    let key_record = get_api_key(env, key_id)
        .ok_or("Key not found".to_string())?;

    // Verify ownership or admin
    if key_record.owner != *owner {
        return Err("Unauthorized: key owner mismatch".to_string());
    }

    // Update status
    let mut revoked_key = key_record;
    revoked_key.status = ApiKeyStatus::Revoked;
    env.storage()
        .persistent()
        .set(&ApiKeyDataKey::ApiKey(key_id.clone()), &revoked_key);

    // Remove from active keys
    remove_active_key(env, key_id);

    Ok(())
}

// ─────────────────────────────────────────────────────────────
// Dual Key Support
// ─────────────────────────────────────────────────────────────
pub fn get_active_keys(env: &Env) -> Vec<String> {
    env.storage()
        .persistent()
        .get(&ApiKeyDataKey::ActiveKeys)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn get_user_keys(env: &Env, owner: &Address) -> Vec<String> {
    env.storage()
        .persistent()
        .get(&ApiKeyDataKey::UserKeys(owner.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn check_keys_need_rotation(env: &Env) -> Vec<String> {
    let config = get_rotation_config(env);
    let active_keys = get_active_keys(env);
    let now = env.ledger().timestamp();
    let mut keys_to_rotate: Vec<String> = Vec::new(env);

    for key_id in active_keys.iter() {
        if let Some(key_record) = get_api_key(env, &key_id) {
            // Check if approaching expiry
            if key_record.status == ApiKeyStatus::Active
                && now + config.alert_before_expiry > key_record.expires_at
            {
                keys_to_rotate.push_back(key_id);
            }
        }
    }

    keys_to_rotate
}

// ─────────────────────────────────────────────────────────────
// Alert System
// ─────────────────────────────────────────────────────────────
pub fn get_keys_expiring_soon(env: &Env) -> Vec<ApiKeyRecord> {
    let config = get_rotation_config(env);
    let active_keys = get_active_keys(env);
    let now = env.ledger().timestamp();
    let mut expiring_keys: Vec<ApiKeyRecord> = Vec::new(env);

    for key_id in active_keys.iter() {
        if let Some(key_record) = get_api_key(env, &key_id) {
            if key_record.status == ApiKeyStatus::Active
                && now + config.alert_before_expiry > key_record.expires_at
                && now < key_record.expires_at
            {
                expiring_keys.push_back(key_record);
            }
        }
    }

    expiring_keys
}

// ─────────────────────────────────────────────────────────────
// Rotation History
// ─────────────────────────────────────────────────────────────
pub fn get_rotation_history(env: &Env, key_id: &String) -> Vec<KeyRotationEvent> {
    env.storage()
        .persistent()
        .get(&ApiKeyDataKey::RotationHistory(key_id.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

// ─────────────────────────────────────────────────────────────
// Helper Functions
// ─────────────────────────────────────────────────────────────
fn generate_key_id(env: &Env) -> String {
    let counter: u64 = env
        .storage()
        .instance()
        .get(&ApiKeyDataKey::KeyCounter)
        .unwrap_or(0);
    let next = counter + 1;
    env.storage().instance().set(&ApiKeyDataKey::KeyCounter, &next);

    String::from_str(env, &format!("key_{}", next))
}

fn add_user_key(env: &Env, owner: &Address, key_id: &String) {
    let mut keys = get_user_keys(env, owner);
    keys.push_back(key_id.clone());
    env.storage()
        .persistent()
        .set(&ApiKeyDataKey::UserKeys(owner.clone()), &keys);
}

fn add_active_key(env: &Env, key_id: &String) {
    let mut active = get_active_keys(env);
    active.push_back(key_id.clone());
    env.storage()
        .persistent()
        .set(&ApiKeyDataKey::ActiveKeys, &active);
}

fn remove_active_key(env: &Env, key_id: &String) {
    let active = get_active_keys(env);
    let mut new_active: Vec<String> = Vec::new(env);

    for id in active.iter() {
        if id != *key_id {
            new_active.push_back(id);
        }
    }

    env.storage().persistent().set(&ApiKeyDataKey::ActiveKeys, &new_active);
}

fn record_rotation_event(
    env: &Env,
    old_key_id: &String,
    new_key_id: &String,
    rotated_by: &Address,
    reason: &String,
) {
    let event = KeyRotationEvent {
        key_id: old_key_id.clone(),
        new_key_id: new_key_id.clone(),
        rotated_by: rotated_by.clone(),
        timestamp: env.ledger().timestamp(),
        reason: reason.clone(),
    };

    let mut history = get_rotation_history(env, old_key_id);
    history.push_back(event);
    env.storage()
        .persistent()
        .set(&ApiKeyDataKey::RotationHistory(old_key_id.clone()), &history);
}
