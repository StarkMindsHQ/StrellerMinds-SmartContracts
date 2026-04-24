//! Minimal Two-Factor Authentication (2FA) Module
//! 
//! Provides basic 2FA functionality using simple storage patterns.

use soroban_sdk::{Address, BytesN, Env, String, Symbol};

// ─────────────────────────────────────────────────────────────
// 2FA Configuration Constants
// ─────────────────────────────────────────────────────────────

/// Maximum failed attempts before account lockout
pub const MAX_FAILED_ATTEMPTS: u32 = 5;
/// Account lockout duration in seconds (15 minutes)
pub const LOCKOUT_DURATION: u64 = 900;

// ─────────────────────────────────────────────────────────────
// 2FA Data Structures
// ─────────────────────────────────────────────────────────────

/// 2FA authentication methods
#[derive(Clone, Debug, PartialEq)]
pub enum TwoFactorMethod {
    /// Time-based One-Time Password (TOTP)
    TOTP,
    /// SMS verification code
    SMS,
    /// Recovery code
    Recovery,
}

/// 2FA verification result
#[derive(Clone, Debug, PartialEq)]
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

// ─────────────────────────────────────────────────────────────
// 2FA Implementation Functions
// ─────────────────────────────────────────────────────────────

/// Initialize 2FA for a user account
pub fn initialize_2fa(
    env: &Env,
    user: &Address,
    totp_secret: &BytesN<32>,
) -> Result<(), TwoFactorError> {
    // Use simple Symbol-based storage keys
    let enabled_key = Symbol::new(env, "2fa_enabled");
    let secret_key = Symbol::new(env, "2fa_secret");
    
    // Store 2FA configuration using instance storage
    env.storage().instance().set(&(enabled_key, user.clone()), &true);
    env.storage().instance().set(&(secret_key, user.clone()), totp_secret);
    
    // Log initialization event
    let event_type = Symbol::new(env, "2fa_initialized");
    env.events().publish((event_type, user.clone()), ());
    
    Ok(())
}

/// Disable 2FA for an account
pub fn disable_2fa(env: &Env, user: &Address) -> Result<(), TwoFactorError> {
    if !is_2fa_enabled(env, user) {
        return Err(TwoFactorError::NotEnabled);
    }
    
    // Remove 2FA configuration
    let enabled_key = Symbol::new(env, "2fa_enabled");
    let secret_key = Symbol::new(env, "2fa_secret");
    
    env.storage().instance().remove(&(enabled_key, user.clone()));
    env.storage().instance().remove(&(secret_key, user.clone()));
    
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
    
    // Log the attempt
    let event_type = match result {
        TwoFactorResult::Success => Symbol::new(env, "2fa_success"),
        _ => Symbol::new(env, "2fa_failed"),
    };
    
    env.events().publish((event_type, user.clone()), ());
    
    Ok(result)
}

/// Check if 2FA is enabled for a user
pub fn is_2fa_enabled(env: &Env, user: &Address) -> bool {
    let enabled_key = Symbol::new(env, "2fa_enabled");
    env.storage()
        .instance()
        .get(&(enabled_key, user.clone()))
        .unwrap_or(false)
}

/// Generate TOTP code for testing
pub fn generate_totp_code(env: &Env, user: &Address, timestamp: u64) -> Result<String, TwoFactorError> {
    let secret_key = Symbol::new(env, "2fa_secret");
    
    if let Some(totp_secret) = env.storage()
        .instance()
        .get::<_, BytesN<32>>(&(secret_key, user.clone())) {
        
        // Simple TOTP simulation
        let time_window = timestamp / 30; // 30-second windows
        let hash = simple_hash(&totp_secret.to_bytes(), &time_window.to_le_bytes());
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
        let time_window = current_time / 30;
        let check_time = if drift == 0 {
            time_window
        } else if current_time % 30 < 15 {
            time_window - drift
        } else {
            time_window + drift
        };
        
        if let Ok(expected_code) = generate_totp_code(env, user, check_time * 30) {
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
    let lockout_key = Symbol::new(env, "2fa_lockout");
    
    if let Some(lockout_timestamp) = env.storage()
        .instance()
        .get::<_, u64>(&(lockout_key, user.clone())) {
        
        if lockout_timestamp > 0 {
            let current_time = env.ledger().timestamp();
            return current_time < lockout_timestamp + LOCKOUT_DURATION;
        }
    }
    false
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
