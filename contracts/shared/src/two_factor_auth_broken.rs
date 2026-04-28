//! Simplified Two-Factor Authentication (2FA) Module
//! 
//! Provides basic 2FA functionality that works with Soroban storage constraints.

use soroban_sdk::{Address, BytesN, Env, String, Vec, Symbol};

// ─────────────────────────────────────────────────────────────
// 2FA Configuration Constants
// ─────────────────────────────────────────────────────────────

/// TOTP time window in seconds (30 seconds)
pub const TOTP_TIME_WINDOW: u64 = 30;
/// Maximum failed attempts before account lockout
pub const MAX_FAILED_ATTEMPTS: u32 = 5;
/// Account lockout duration in seconds (15 minutes)
pub const LOCKOUT_DURATION: u64 = 900;

// ─────────────────────────────────────────────────────────────
// 2FA Data Structures (Simplified for Soroban)
// ─────────────────────────────────────────────────────────────

/// 2FA authentication methods
#[derive(Clone, Debug, PartialEq, contracttype)]
pub enum TwoFactorMethod {
    /// Time-based One-Time Password (TOTP)
    TOTP,
    /// SMS verification code
    SMS,
    /// Recovery code
    Recovery,
}

/// 2FA verification result
#[derive(Clone, Debug, PartialEq, contracttype)]
pub enum TwoFactorResult {
    /// Authentication successful
    Success,
    /// Invalid code provided
    InvalidCode,
    /// Code has expired
    Expired,
    /// Account is locked
    AccountLocked,
    /// 2FA not enabled for account
    NotEnabled,
}

/// Simple 2FA storage key
#[derive(Clone, contracttype)]
pub enum TwoFactorStorageKey {
    /// User's 2FA enabled status
    Enabled(Address),
    /// User's TOTP secret
    TOTPSecret(Address),
    /// User's failed attempts
    FailedAttempts(Address),
    /// User's lockout timestamp
    LockoutTimestamp(Address),
}

// ─────────────────────────────────────────────────────────────
// 2FA Implementation Functions
// ─────────────────────────────────────────────────────────────

/// Initialize 2FA for a user account
pub fn initialize_2fa(
    env: &Env,
    user: &Address,
    totp_secret: BytesN<32>,
) -> Result<(), TwoFactorError> {
    // Check if 2FA is already enabled
    if is_2fa_enabled(env, user) {
        return Err(TwoFactorError::AlreadyEnabled);
    }
    
    // Store 2FA configuration
    env.storage().persistent().set(&TwoFactorStorageKey::Enabled(user.clone()), &true);
    env.storage().persistent().set(&TwoFactorStorageKey::TOTPSecret(user.clone()), &totp_secret);
    env.storage().persistent().set(&TwoFactorStorageKey::FailedAttempts(user.clone()), &0u32);
    env.storage().persistent().set(&TwoFactorStorageKey::LockoutTimestamp(user.clone()), &0u64);
    
    // Log initialization event
    log_2fa_event(env, user, &TwoFactorMethod::TOTP, &TwoFactorResult::Success);
    
    Ok(())
}

/// Disable 2FA for an account
pub fn disable_2fa(env: &Env, user: &Address) -> Result<(), TwoFactorError> {
    if !is_2fa_enabled(env, user) {
        return Err(TwoFactorError::NotEnabled);
    }
    
    // Remove 2FA configuration
    env.storage().persistent().remove(&TwoFactorStorageKey::Enabled(user.clone()));
    env.storage().persistent().remove(&TwoFactorStorageKey::TOTPSecret(user.clone()));
    env.storage().persistent().remove(&TwoFactorStorageKey::FailedAttempts(user.clone()));
    env.storage().persistent().remove(&TwoFactorStorageKey::LockoutTimestamp(user.clone()));
    
    Ok(())
}

/// Verify 2FA authentication
pub fn verify_2fa(
    env: &Env,
    user: &Address,
    code: &String,
    method: TwoFactorMethod,
) -> Result<TwoFactorResult, TwoFactorError> {
    // Check if 2FA is enabled
    if !is_2fa_enabled(env, user) {
        return Ok(TwoFactorResult::NotEnabled);
    }
    
    // Check account lockout
    if is_account_locked(env, user) {
        return Ok(TwoFactorResult::AccountLocked);
    }
    
    let result = match method {
        TwoFactorMethod::TOTP => verify_totp(env, user, code),
        TwoFactorMethod::SMS => verify_sms_code(env, user, code),
        TwoFactorMethod::Recovery => verify_recovery_code(env, user, code),
    };
    
    // Update failed attempts and lockout if needed
    match result {
        TwoFactorResult::Success => {
            reset_failed_attempts(env, user);
        }
        _ => {
            increment_failed_attempts(env, user);
            check_and_apply_lockout(env, user);
        }
    }
    
    // Log the attempt
    log_2fa_event(env, user, &method, &result);
    
    Ok(result)
}

/// Check if 2FA is enabled for a user
pub fn is_2fa_enabled(env: &Env, user: &Address) -> bool {
    env.storage()
        .persistent()
        .get(&TwoFactorStorageKey::Enabled(user.clone()))
        .unwrap_or(false)
}

/// Generate TOTP code for testing
pub fn generate_totp_code(env: &Env, user: &Address, timestamp: u64) -> Result<String, TwoFactorError> {
    if let Some(totp_secret) = env.storage()
        .persistent()
        .get::<_, BytesN<32>>(&TwoFactorStorageKey::TOTPSecret(user.clone())) {
        
        // Simple TOTP simulation
        let time_counter = timestamp / TOTP_TIME_WINDOW;
        let hash = simple_hash(&totp_secret.to_bytes(), &time_counter.to_le_bytes());
        let code = (hash % 1_000_000).to_string();
        
        Ok(format!("{:06}", code.parse::<u32>().unwrap_or(0)))
    } else {
        Err(TwoFactorError::TOTPNotConfigured)
    }
}

// ─────────────────────────────────────────────────────────────
// Helper Functions
// ─────────────────────────────────────────────────────────────

/// Verify TOTP code
fn verify_totp(env: &Env, user: &Address, code: &String) -> TwoFactorResult {
    let current_time = env.ledger().timestamp();
    
    // Check current time window and adjacent windows
    for drift in 0..=1 {
        let time_window = current_time / TOTP_TIME_WINDOW;
        let check_time = if drift == 0 {
            time_window
        } else if current_time % TOTP_TIME_WINDOW < TOTP_TIME_WINDOW / 2 {
            time_window - drift
        } else {
            time_window + drift
        };
        
        if let Ok(expected_code) = generate_totp_code(env, user, check_time * TOTP_TIME_WINDOW) {
            if expected_code == *code {
                return TwoFactorResult::Success;
            }
        }
    }
    
    TwoFactorResult::InvalidCode
}

/// Verify SMS code (simplified)
fn verify_sms_code(_env: &Env, _user: &Address, code: &String) -> TwoFactorResult {
    // In production, this would verify against stored SMS codes
    // For now, just validate format
    if code.len() == 6 && code.chars().all(|c| c.is_ascii_digit()) {
        TwoFactorResult::Success
    } else {
        TwoFactorResult::InvalidCode
    }
}

/// Verify recovery code (simplified)
fn verify_recovery_code(_env: &Env, _user: &Address, code: &String) -> TwoFactorResult {
    // In production, this would verify against stored recovery codes
    // For now, just validate format
    if code.len() == 8 && code.chars().all(|c| c.is_ascii_alphanumeric()) {
        TwoFactorResult::Success
    } else {
        TwoFactorResult::InvalidCode
    }
}

/// Check if account is locked
fn is_account_locked(env: &Env, user: &Address) -> bool {
    if let Some(lockout_timestamp) = env.storage()
        .persistent()
        .get::<_, u64>(&TwoFactorStorageKey::LockoutTimestamp(user.clone())) {
        
        if lockout_timestamp > 0 {
            let current_time = env.ledger().timestamp();
            return current_time < lockout_timestamp + LOCKOUT_DURATION;
        }
    }
    false
}

/// Increment failed attempts
fn increment_failed_attempts(env: &Env, user: &Address) {
    let current_attempts = env.storage()
        .persistent()
        .get::<_, u32>(&TwoFactorStorageKey::FailedAttempts(user.clone()))
        .unwrap_or(0);
    
    let new_attempts = current_attempts + 1;
    env.storage().persistent().set(&TwoFactorStorageKey::FailedAttempts(user.clone()), &new_attempts);
}

/// Reset failed attempts
fn reset_failed_attempts(env: &Env, user: &Address) {
    env.storage().persistent().set(&TwoFactorStorageKey::FailedAttempts(user.clone()), &0u32);
    env.storage().persistent().set(&TwoFactorStorageKey::LockoutTimestamp(user.clone()), &0u64);
}

/// Check and apply account lockout
fn check_and_apply_lockout(env: &Env, user: &Address) {
    let current_attempts = env.storage()
        .persistent()
        .get::<_, u32>(&TwoFactorStorageKey::FailedAttempts(user.clone()))
        .unwrap_or(0);
    
    if current_attempts >= MAX_FAILED_ATTEMPTS {
        let lockout_timestamp = env.ledger().timestamp();
        env.storage().persistent().set(&TwoFactorStorageKey::LockoutTimestamp(user.clone()), &lockout_timestamp);
    }
}

/// Log 2FA event
fn log_2fa_event(
    env: &Env,
    user: &Address,
    method: &TwoFactorMethod,
    result: &TwoFactorResult,
) {
    let event_type = match result {
        TwoFactorResult::Success => Symbol::new(env, "2fa_success"),
        _ => Symbol::new(env, "2fa_failed"),
    };
    
    let mut topics = Vec::new(env);
    topics.push_back(user.clone());
    
    let method_str = match method {
        TwoFactorMethod::TOTP => "totp",
        TwoFactorMethod::SMS => "sms", 
        TwoFactorMethod::Recovery => "recovery",
    };
    topics.push_back(Symbol::new(env, method_str));
    
    env.events().publish(event_type, topics);
}

/// Simple hash function for demonstration
fn simple_hash(secret: &[u8], counter: &[u8]) -> u64 {
    let mut hash: u64 = 0;
    
    for (i, &byte) in secret.iter().enumerate() {
        hash = hash.wrapping_add((byte as u64).wrapping_mul(i as u64 + 1));
    }
    
    for (i, &byte) in counter.iter().enumerate() {
        hash = hash.wrapping_add((byte as u64).wrapping_mul((i + 32) as u64));
    }
    
    hash
}

// ─────────────────────────────────────────────────────────────
// 2FA Error Types
// ─────────────────────────────────────────────────────────────

#[derive(Clone, Debug, PartialEq)]
pub enum TwoFactorError {
    /// 2FA is already enabled for this account
    AlreadyEnabled,
    /// 2FA is not configured for this account
    NotEnabled,
    /// TOTP is not configured for this account
    TOTPNotConfigured,
    /// Account is temporarily locked
    AccountLocked,
    /// Internal error occurred
    InternalError,
}

// ─────────────────────────────────────────────────────────────
// 2FA Utility Functions
// ─────────────────────────────────────────────────────────────

/// Check if 2FA is required for a user
pub fn is_2fa_required(env: &Env, user: &Address) -> bool {
    is_2fa_enabled(env, user)
}

/// Get account lockout status
pub fn get_lockout_status(env: &Env, user: &Address) -> (bool, u64) {
    if let Some(lockout_timestamp) = env.storage()
        .persistent()
        .get::<_, u64>(&TwoFactorStorageKey::LockoutTimestamp(user.clone())) {
        
        if lockout_timestamp > 0 {
            let current_time = env.ledger().timestamp();
            let lockout_end = lockout_timestamp + LOCKOUT_DURATION;
            let is_locked = current_time < lockout_end;
            let remaining_time = if is_locked { lockout_end - current_time } else { 0 };
            return (is_locked, remaining_time);
        }
    }
    
    (false, 0)
}
