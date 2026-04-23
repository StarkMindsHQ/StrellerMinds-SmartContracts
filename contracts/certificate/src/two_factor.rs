//! Two-Factor Authentication module for the Certificate contract.
//!
//! Provides TOTP-style verification, backup codes, and recovery codes
//! to protect sensitive operations. Admin 2FA can be enforced globally.

use soroban_sdk::{Address, Bytes, BytesN, Env, String, Vec};

use crate::errors::CertificateError;
use crate::storage;
use crate::types::{RecoveryCode, TwoFactorConfig, TwoFactorMethod};

/// Standard TOTP time window in seconds.
const TOTP_WINDOW: u64 = 30;
/// Session duration after successful 2FA verification (5 minutes).
const SESSION_DURATION: u64 = 300;
/// Default number of recovery codes generated.
const DEFAULT_RECOVERY_CODES: u32 = 8;

/// Compute a TOTP-style code hash for the current time bucket.
/// Uses SHA256(secret_hash || time_bucket) and returns the first 32 bytes.
fn compute_totp_hash(env: &Env, secret_hash: &BytesN<32>, time_bucket: u64) -> BytesN<32> {
    let mut input = Bytes::new(env);
    input.extend_from_slice(&secret_hash.to_array());
    input.extend_from_slice(&time_bucket.to_be_bytes());
    let hash: BytesN<32> = env.crypto().sha256(&input).into();
    hash
}

/// Check if the user has an active 2FA verification session.
fn has_active_session(env: &Env, user: &Address) -> bool {
    if let Some(expires_at) = storage::get_two_factor_session(env, user) {
        env.ledger().timestamp() < expires_at
    } else {
        false
    }
}

/// Enable two-factor authentication for a user.
///
/// # Arguments
/// * `user` - The address enabling 2FA.
/// * `secret_hash` - Hash of the shared TOTP secret.
/// * `recovery_code_hashes` - Hashes of recovery codes (typically 8).
pub fn enable(
    env: &Env,
    user: &Address,
    secret_hash: &BytesN<32>,
    recovery_code_hashes: &Vec<BytesN<32>>,
) -> Result<(), CertificateError> {
    user.require_auth();

    if storage::get_two_factor_config(env, user).is_some() {
        return Err(CertificateError::TwoFactorAlreadyEnabled);
    }

    let config = TwoFactorConfig {
        enabled: true,
        secret_hash: secret_hash.clone(),
        recovery_codes_remaining: recovery_code_hashes.len(),
        enabled_at: env.ledger().timestamp(),
        method: TwoFactorMethod::Totp,
    };

    storage::set_two_factor_config(env, user, &config);

    // Store recovery codes
    for i in 0..recovery_code_hashes.len() {
        let code = RecoveryCode {
            code_hash: recovery_code_hashes.get(i).unwrap(),
            used: false,
        };
        storage::set_recovery_code(env, user, i, &code);
    }

    crate::events::emit_two_factor_enabled(env, user, &TwoFactorMethod::Totp);
    Ok(())
}

/// Disable two-factor authentication for a user. Admin only.
pub fn disable(
    env: &Env,
    admin: &Address,
    user: &Address,
) -> Result<(), CertificateError> {
    admin.require_auth();
    let stored_admin = storage::get_admin(env);
    if *admin != stored_admin {
        return Err(CertificateError::Unauthorized);
    }

    if storage::get_two_factor_config(env, user).is_none() {
        return Err(CertificateError::TwoFactorNotEnabled);
    }

    // Remove config and all recovery codes
    env.storage().persistent().remove(&crate::types::CertDataKey::TwoFactorConfig(user.clone()));

    // Best-effort removal of recovery codes (up to default count)
    for i in 0..DEFAULT_RECOVERY_CODES {
        env.storage().persistent().remove(&crate::types::CertDataKey::RecoveryCode(user.clone(), i));
    }

    storage::clear_two_factor_session(env, user);
    crate::events::emit_two_factor_disabled(env, user);
    Ok(())
}

/// Verify a TOTP or backup code and create a session.
///
/// Returns Ok(true) on success, Ok(false) if code was invalid.
pub fn verify(
    env: &Env,
    user: &Address,
    code_hash: &BytesN<32>,
) -> Result<bool, CertificateError> {
    user.require_auth();

    let config = storage::get_two_factor_config(env, user)
        .ok_or(CertificateError::TwoFactorNotEnabled)?;

    if !config.enabled {
        return Err(CertificateError::TwoFactorNotEnabled);
    }

    // Try TOTP verification first
    let now = env.ledger().timestamp();
    let time_bucket = now / TOTP_WINDOW;

    // Check current and previous window for clock skew
    let windows = [time_bucket, time_bucket.saturating_sub(1)];
    for window in windows {
        let expected = compute_totp_hash(env, &config.secret_hash, window);
        if expected == *code_hash {
            storage::set_two_factor_session(env, user, now + SESSION_DURATION);
            crate::events::emit_two_factor_verified(env, user, &TwoFactorMethod::Totp);
            return Ok(true);
        }
    }

    // Try recovery/backup codes
    for i in 0..DEFAULT_RECOVERY_CODES {
        if let Some(recovery) = storage::get_recovery_code(env, user, i) {
            if !recovery.used && recovery.code_hash == *code_hash {
                // Mark as used
                let updated = RecoveryCode {
                    code_hash: recovery.code_hash,
                    used: true,
                };
                storage::set_recovery_code(env, user, i, &updated);

                // Update remaining count
                let mut new_config = config.clone();
                if new_config.recovery_codes_remaining > 0 {
                    new_config.recovery_codes_remaining -= 1;
                }
                storage::set_two_factor_config(env, user, &new_config);

                storage::set_two_factor_session(env, user, now + SESSION_DURATION);
                crate::events::emit_recovery_code_used(env, user, i);
                crate::events::emit_two_factor_verified(env, user, &TwoFactorMethod::Recovery);
                return Ok(true);
            }
        }
    }

    Err(CertificateError::InvalidTwoFactorCode)
}

/// Check whether 2FA verification is required and satisfied for the given user.
///
/// If admin 2FA is enforced globally, returns an error when the user is admin
/// and does not have an active session.
pub fn require_if_needed(
    env: &Env,
    user: &Address,
) -> Result<(), CertificateError> {
    if !storage::is_admin_two_factor_required(env) {
        return Ok(());
    }

    // Only enforce for admin
    let admin = storage::get_admin(env);
    if *user != admin {
        return Ok(());
    }

    let config = storage::get_two_factor_config(env, user);
    if config.is_none() {
        return Err(CertificateError::TwoFactorRequired);
    }

    if !has_active_session(env, user) {
        return Err(CertificateError::TwoFactorRequired);
    }

    Ok(())
}

/// Generate a new set of recovery codes for a user.
pub fn regenerate_recovery_codes(
    env: &Env,
    user: &Address,
    new_code_hashes: &Vec<BytesN<32>>,
) -> Result<(), CertificateError> {
    user.require_auth();

    let mut config = storage::get_two_factor_config(env, user)
        .ok_or(CertificateError::TwoFactorNotEnabled)?;

    // Clear old codes
    for i in 0..DEFAULT_RECOVERY_CODES {
        env.storage().persistent().remove(&crate::types::CertDataKey::RecoveryCode(user.clone(), i));
    }

    // Store new codes
    for i in 0..new_code_hashes.len() {
        let code = RecoveryCode {
            code_hash: new_code_hashes.get(i).unwrap(),
            used: false,
        };
        storage::set_recovery_code(env, user, i, &code);
    }

    config.recovery_codes_remaining = new_code_hashes.len();
    storage::set_two_factor_config(env, user, &config);

    Ok(())
}
