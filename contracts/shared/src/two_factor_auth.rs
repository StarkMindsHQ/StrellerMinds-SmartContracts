//! Two-Factor Authentication (2FA) Module
//! 
//! Provides comprehensive 2FA functionality including TOTP support,
//! SMS backup codes, and recovery mechanisms for enhanced security.

use soroban_sdk::{Address, BytesN, Env, String, Vec, Map, Symbol, U256};

// ─────────────────────────────────────────────────────────────
// 2FA Configuration Constants
// ─────────────────────────────────────────────────────────────

/// TOTP time window in seconds (30 seconds)
pub const TOTP_TIME_WINDOW: u64 = 30;
/// Maximum allowed time drift for TOTP validation (1 window)
pub const TOTP_MAX_DRIFT: u64 = 1;
/// Number of recovery codes generated per user
pub const RECOVERY_CODES_COUNT: u32 = 10;
/// Maximum failed attempts before account lockout
pub const MAX_FAILED_ATTEMPTS: u32 = 5;
/// Account lockout duration in seconds (15 minutes)
pub const LOCKOUT_DURATION: u64 = 900;
/// SMS code length
pub const SMS_CODE_LENGTH: u32 = 6;
/// SMS code validity duration (5 minutes)
pub const SMS_CODE_VALIDITY: u64 = 300;

// ─────────────────────────────────────────────────────────────
// 2FA Data Structures
// ─────────────────────────────────────────────────────────────

/// 2FA configuration for a user account
#[derive(Clone, Debug)]
pub struct TwoFactorConfig {
    /// Whether 2FA is enabled for this account
    pub enabled: bool,
    /// Whether 2FA is mandatory (for admin accounts)
    pub mandatory: bool,
    /// TOTP secret (encrypted)
    pub totp_secret: Option<BytesN<32>>,
    /// Phone number for SMS backup (encrypted)
    pub phone_number: Option<String>,
    /// Recovery codes (encrypted)
    pub recovery_codes: Vec<BytesN<32>>,
    /// Used recovery codes
    pub used_recovery_codes: Vec<BytesN<32>>,
    /// Last successful authentication timestamp
    pub last_auth_timestamp: u64,
    /// Failed attempt count
    pub failed_attempts: u32,
    /// Account lockout timestamp (0 if not locked)
    pub lockout_timestamp: u64,
}

/// 2FA verification request
#[derive(Clone, Debug)]
pub struct TwoFactorRequest {
    /// Unique request identifier
    pub request_id: BytesN<32>,
    /// User address being authenticated
    pub user: Address,
    /// Type of 2FA method being used
    pub method: TwoFactorMethod,
    /// Request timestamp
    pub timestamp: u64,
    /// Expiration timestamp
    pub expires_at: u64,
    /// Whether this request has been used
    pub used: bool,
}

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
    /// Invalid authentication method
    InvalidMethod,
}

/// 2FA audit log entry
#[derive(Clone, Debug)]
pub struct TwoFactorAuditEntry {
    /// User address
    pub user: Address,
    /// Authentication method used
    pub method: TwoFactorMethod,
    /// Result of the authentication attempt
    pub result: TwoFactorResult,
    /// Timestamp of the attempt
    pub timestamp: u64,
    /// IP address or identifier (if available)
    pub source: Option<String>,
}

// ─────────────────────────────────────────────────────────────
// 2FA Storage Keys
// ─────────────────────────────────────────────────────────────

#[derive(Clone)]
pub enum TwoFactorDataKey {
    /// User's 2FA configuration
    Config(Address),
    /// Active 2FA verification request
    Request(BytesN<32>),
    /// User's pending SMS codes
    SMSCode(Address),
    /// 2FA audit log entries
    AuditLog(Address),
    /// Global 2FA settings
    Settings,
}

// ─────────────────────────────────────────────────────────────
// 2FA Implementation Functions
// ─────────────────────────────────────────────────────────────

/// Initialize 2FA for a user account
pub fn initialize_2fa(
    env: &Env,
    user: &Address,
    mandatory: bool,
    totp_secret: Option<BytesN<32>>,
    phone_number: Option<String>,
) -> Result<(), TwoFactorError> {
    // Check if user already has 2FA configured
    if has_2fa_config(env, user) {
        return Err(TwoFactorError::AlreadyConfigured);
    }
    
    // Generate recovery codes
    let recovery_codes = generate_recovery_codes(env);
    
    let config = TwoFactorConfig {
        enabled: totp_secret.is_some() || phone_number.is_some(),
        mandatory,
        totp_secret,
        phone_number,
        recovery_codes: recovery_codes.clone(),
        used_recovery_codes: Vec::new(env),
        last_auth_timestamp: 0,
        failed_attempts: 0,
        lockout_timestamp: 0,
    };
    
    // Store configuration
    env.storage().persistent().set(&TwoFactorDataKey::Config(user.clone()), &config);
    
    // Log initialization
    log_2fa_event(env, user, &TwoFactorMethod::TOTP, &TwoFactorResult::Success, None);
    
    Ok(())
}

/// Enable 2FA for an existing account
pub fn enable_2fa(
    env: &Env,
    user: &Address,
    totp_secret: BytesN<32>,
    phone_number: Option<String>,
) -> Result<(), TwoFactorError> {
    let mut config = get_2fa_config(env, user)?;
    
    if config.enabled {
        return Err(TwoFactorError::AlreadyEnabled);
    }
    
    config.enabled = true;
    config.totp_secret = Some(totp_secret);
    config.phone_number = phone_number;
    config.recovery_codes = generate_recovery_codes(env);
    config.used_recovery_codes = Vec::new(env);
    
    env.storage().persistent().set(&TwoFactorDataKey::Config(user.clone()), &config);
    
    log_2fa_event(env, user, &TwoFactorMethod::TOTP, &TwoFactorResult::Success, None);
    
    Ok(())
}

/// Disable 2FA for an account (requires 2FA verification)
pub fn disable_2fa(
    env: &Env,
    user: &Address,
    verification_code: &String,
    method: TwoFactorMethod,
) -> Result<(), TwoFactorError> {
    // Verify 2FA before disabling
    verify_2fa(env, user, verification_code, method)?;
    
    // Remove 2FA configuration
    env.storage().persistent().remove(&TwoFactorDataKey::Config(user.clone()));
    
    log_2fa_event(env, user, &method, &TwoFactorResult::Success, None);
    
    Ok(())
}

/// Verify 2FA authentication
pub fn verify_2fa(
    env: &Env,
    user: &Address,
    code: &String,
    method: TwoFactorMethod,
) -> Result<TwoFactorResult, TwoFactorError> {
    let config = get_2fa_config(env, user)?;
    
    // Check if 2FA is enabled
    if !config.enabled {
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
            update_last_auth_timestamp(env, user);
        }
        _ => {
            increment_failed_attempts(env, user);
            check_and_apply_lockout(env, user);
        }
    }
    
    // Log the attempt
    log_2fa_event(env, user, &method, &result, None);
    
    Ok(result)
}

/// Generate TOTP code for testing purposes
pub fn generate_totp_code(env: &Env, user: &Address, timestamp: u64) -> Result<String, TwoFactorError> {
    let config = get_2fa_config(env, user)?;
    
    if let Some(totp_secret) = config.totp_secret {
        // In a real implementation, this would use cryptographic TOTP algorithm
        // For demonstration, we'll generate a simple 6-digit code
        let time_counter = timestamp / TOTP_TIME_WINDOW;
        let hash = simple_hash(&totp_secret.to_bytes(), &time_counter.to_le_bytes());
        let code = (hash % 1_000_000).to_string();
        
        Ok(format!("{:06}", code.parse::<u32>().unwrap_or(0)))
    } else {
        Err(TwoFactorError::TOTPNotConfigured)
    }
}

/// Send SMS verification code
pub fn send_sms_code(env: &Env, user: &Address) -> Result<(), TwoFactorError> {
    let config = get_2fa_config(env, user)?;
    
    if config.phone_number.is_none() {
        return Err(TwoFactorError::SMSNotConfigured);
    }
    
    // Generate 6-digit SMS code
    let sms_code = generate_sms_code(env);
    let expires_at = env.ledger().timestamp() + SMS_CODE_VALIDITY;
    
    // Store SMS code (in production, this would be sent via SMS service)
    env.storage().temporary().set(&TwoFactorDataKey::SMSCode(user.clone()), &(sms_code, expires_at));
    
    Ok(())
}

/// Generate recovery codes for a user
pub fn generate_recovery_codes(env: &Env) -> Vec<BytesN<32>> {
    let mut codes = Vec::new(env);
    
    for i in 0..RECOVERY_CODES_COUNT {
        let mut code_bytes = [0u8; 32];
        code_bytes[0..4].copy_from_slice(&i.to_le_bytes());
        
        // Generate pseudo-random recovery code
        for j in 4..32 {
            code_bytes[j] = ((i as u32 * 7 + j as u32) % 256) as u8;
        }
        
        codes.push_back(BytesN::from_array(env, &code_bytes));
    }
    
    codes
}

// ─────────────────────────────────────────────────────────────
// Helper Functions
// ─────────────────────────────────────────────────────────────

/// Get user's 2FA configuration
pub fn get_2fa_config(env: &Env, user: &Address) -> Result<TwoFactorConfig, TwoFactorError> {
    env.storage()
        .persistent()
        .get(&TwoFactorDataKey::Config(user.clone()))
        .ok_or(TwoFactorError::NotConfigured)
}

/// Check if user has 2FA configured
pub fn has_2fa_config(env: &Env, user: &Address) -> bool {
    env.storage().persistent().has(&TwoFactorDataKey::Config(user.clone()))
}

/// Verify TOTP code
fn verify_totp(env: &Env, user: &Address, code: &String) -> TwoFactorResult {
    let current_time = env.ledger().timestamp();
    
    // Check current time window and adjacent windows (for clock drift)
    for drift in 0..=TOTP_MAX_DRIFT {
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

/// Verify SMS code
fn verify_sms_code(env: &Env, user: &Address, code: &String) -> TwoFactorResult {
    if let Some((stored_code, expires_at)) = env.storage()
        .temporary()
        .get::<_, (String, u64)>(&TwoFactorDataKey::SMSCode(user.clone())) {
        
        let current_time = env.ledger().timestamp();
        
        if current_time > expires_at {
            return TwoFactorResult::Expired;
        }
        
        if stored_code == *code {
            // Remove used SMS code
            env.storage().temporary().remove(&TwoFactorDataKey::SMSCode(user.clone()));
            return TwoFactorResult::Success;
        }
    }
    
    TwoFactorResult::InvalidCode
}

/// Verify recovery code
fn verify_recovery_code(env: &Env, user: &Address, code: &String) -> TwoFactorResult {
    let mut config = get_2fa_config(env, user).unwrap_or_else(|_| TwoFactorConfig {
        enabled: false,
        mandatory: false,
        totp_secret: None,
        phone_number: None,
        recovery_codes: Vec::new(env),
        used_recovery_codes: Vec::new(env),
        last_auth_timestamp: 0,
        failed_attempts: 0,
        lockout_timestamp: 0,
    });
    
    // Parse recovery code
    let code_bytes = if let Ok(parsed) = hex::decode(code) {
        if parsed.len() == 32 {
            BytesN::from_array(env, &parsed.try_into().unwrap_or([0u8; 32]))
        } else {
            return TwoFactorResult::InvalidCode;
        }
    } else {
        return TwoFactorResult::InvalidCode;
    };
    
    // Check if code is valid and not used
    if config.recovery_codes.contains(&code_bytes) && 
       !config.used_recovery_codes.contains(&code_bytes) {
        
        // Mark code as used
        config.used_recovery_codes.push_back(code_bytes);
        env.storage().persistent().set(&TwoFactorDataKey::Config(user.clone()), &config);
        
        return TwoFactorResult::Success;
    }
    
    TwoFactorResult::InvalidCode
}

/// Generate SMS verification code
fn generate_sms_code(env: &Env) -> String {
    let timestamp = env.ledger().timestamp();
    let code = (timestamp % 1_000_000).to_string();
    format!("{:06}", code.parse::<u32>().unwrap_or(123456))
}

/// Check if account is locked
fn is_account_locked(env: &Env, user: &Address) -> bool {
    if let Ok(config) = get_2fa_config(env, user) {
        if config.lockout_timestamp > 0 {
            let current_time = env.ledger().timestamp();
            return current_time < config.lockout_timestamp + LOCKOUT_DURATION;
        }
    }
    false
}

/// Increment failed attempts
fn increment_failed_attempts(env: &Env, user: &Address) {
    if let Ok(mut config) = get_2fa_config(env, user) {
        config.failed_attempts += 1;
        env.storage().persistent().set(&TwoFactorDataKey::Config(user.clone()), &config);
    }
}

/// Reset failed attempts
fn reset_failed_attempts(env: &Env, user: &Address) {
    if let Ok(mut config) = get_2fa_config(env, user) {
        config.failed_attempts = 0;
        config.lockout_timestamp = 0;
        env.storage().persistent().set(&TwoFactorDataKey::Config(user.clone()), &config);
    }
}

/// Check and apply account lockout
fn check_and_apply_lockout(env: &Env, user: &Address) {
    if let Ok(mut config) = get_2fa_config(env, user) {
        if config.failed_attempts >= MAX_FAILED_ATTEMPTS && config.lockout_timestamp == 0 {
            config.lockout_timestamp = env.ledger().timestamp();
            env.storage().persistent().set(&TwoFactorDataKey::Config(user.clone()), &config);
        }
    }
}

/// Update last authentication timestamp
fn update_last_auth_timestamp(env: &Env, user: &Address) {
    if let Ok(mut config) = get_2fa_config(env, user) {
        config.last_auth_timestamp = env.ledger().timestamp();
        env.storage().persistent().set(&TwoFactorDataKey::Config(user.clone()), &config);
    }
}

/// Log 2FA event
fn log_2fa_event(
    env: &Env,
    user: &Address,
    method: &TwoFactorMethod,
    result: &TwoFactorResult,
    source: Option<String>,
) {
    let entry = TwoFactorAuditEntry {
        user: user.clone(),
        method: method.clone(),
        result: result.clone(),
        timestamp: env.ledger().timestamp(),
        source,
    };
    
    // Store audit entry (in production, this might go to a separate audit log)
    let audit_key = TwoFactorDataKey::AuditLog(user.clone());
    let mut audit_log: Vec<TwoFactorAuditEntry> = env.storage()
        .persistent()
        .get(&audit_key)
        .unwrap_or_else(|| Vec::new(env));
    
    audit_log.push_back(entry);
    
    // Keep only last 100 entries per user
    if audit_log.len() > 100 {
        audit_log.remove(audit_log.len() - 1);
    }
    
    env.storage().persistent().set(&audit_key, &audit_log);
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
    /// 2FA is already configured for this account
    AlreadyConfigured,
    /// 2FA is already enabled for this account
    AlreadyEnabled,
    /// 2FA is not configured for this account
    NotConfigured,
    /// TOTP is not configured for this account
    TOTPNotConfigured,
    /// SMS is not configured for this account
    SMSNotConfigured,
    /// Invalid recovery code provided
    InvalidRecoveryCode,
    /// Account is temporarily locked
    AccountLocked,
    /// Invalid authentication method
    InvalidMethod,
    /// Internal error occurred
    InternalError,
}

// ─────────────────────────────────────────────────────────────
// 2FA Utility Functions
// ─────────────────────────────────────────────────────────────

/// Check if 2FA is required for a user
pub fn is_2fa_required(env: &Env, user: &Address) -> Result<bool, TwoFactorError> {
    let config = get_2fa_config(env, user)?;
    Ok(config.enabled && (config.mandatory || config.totp_secret.is_some()))
}

/// Get remaining recovery codes count
pub fn get_remaining_recovery_codes(env: &Env, user: &Address) -> Result<u32, TwoFactorError> {
    let config = get_2fa_config(env, user)?;
    Ok(config.recovery_codes.len() as u32 - config.used_recovery_codes.len() as u32)
}

/// Check if user can use SMS authentication
pub fn can_use_sms(env: &Env, user: &Address) -> Result<bool, TwoFactorError> {
    let config = get_2fa_config(env, user)?;
    Ok(config.phone_number.is_some())
}

/// Get account lockout status
pub fn get_lockout_status(env: &Env, user: &Address) -> Result<(bool, u64), TwoFactorError> {
    let config = get_2fa_config(env, user)?;
    
    if config.lockout_timestamp > 0 {
        let current_time = env.ledger().timestamp();
        let lockout_end = config.lockout_timestamp + LOCKOUT_DURATION;
        let is_locked = current_time < lockout_end;
        let remaining_time = if is_locked { lockout_end - current_time } else { 0 };
        Ok((is_locked, remaining_time))
    } else {
        Ok((false, 0))
    }
}
