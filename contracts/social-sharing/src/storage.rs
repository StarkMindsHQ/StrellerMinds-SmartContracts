use soroban_sdk::{symbol_short, Address, BytesN, Env, Map, String, Vec};
use crate::errors::SocialSharingError;
use crate::types::{SharePlatform, ShareRecord, SocialSharingAnalytics};

// Storage keys
const ADMIN: &str = "admin";
const INITIALIZED: &str = "initialized";
const SHARES_BY_CERT: &str = "shares_cert";
const SHARES_BY_USER: &str = "shares_user";
const ANALYTICS: &str = "analytics";
const CERT_ANALYTICS: &str = "cert_analytics";
const SHARE_COUNT: &str = "share_count";

/// Get the admin address.
pub fn get_admin(env: &Env) -> Result<Address, SocialSharingError> {
    env.storage()
        .persistent()
        .get(&symbol_short!("admin"))
        .ok_or(SocialSharingError::NotInitialized)
        .and_then(|v| Ok(v))
}

/// Set the admin address.
pub fn set_admin(env: &Env, admin: &Address) -> Result<(), SocialSharingError> {
    env.storage()
        .persistent()
        .set(&symbol_short!("admin"), admin);
    Ok(())
}

/// Check if contract is initialized.
pub fn is_initialized(env: &Env) -> bool {
    env.storage()
        .persistent()
        .has(&symbol_short!("initialized"))
}

/// Mark contract as initialized.
pub fn mark_initialized(env: &Env) -> Result<(), SocialSharingError> {
    env.storage()
        .persistent()
        .set(&symbol_short!("initialized"), &true);
    Ok(())
}

/// Store a share record.
pub fn store_share_record(env: &Env, record: &ShareRecord) -> Result<(), SocialSharingError> {
    let cert_key = format_cert_key(&record.certificate_id, &record.user, &record.platform);
    
    env.storage()
        .persistent()
        .set(&cert_key, record);

    Ok(())
}

/// Get a specific share record.
pub fn get_share_record(
    env: &Env,
    certificate_id: &BytesN<32>,
    user: &Address,
    platform: &SharePlatform,
) -> Result<Option<ShareRecord>, SocialSharingError> {
    let key = format_cert_key(certificate_id, user, platform);
    
    Ok(env.storage()
        .persistent()
        .get(&key)
        .map(|v| v))
}

/// Get all shares for a certificate.
pub fn get_certificate_shares(
    env: &Env,
    certificate_id: &BytesN<32>,
) -> Result<Vec<ShareRecord>, SocialSharingError> {
    let key = format_certificate_shares_key(certificate_id);
    
    Ok(env.storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env)))
}

/// Get all shares by a user.
pub fn get_user_shares(
    env: &Env,
    user: &Address,
) -> Result<Vec<ShareRecord>, SocialSharingError> {
    let key = format_user_shares_key(user);
    
    Ok(env.storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| Vec::new(env)))
}

/// Update engagement metrics for a share.
pub fn update_engagement(
    env: &Env,
    certificate_id: &BytesN<32>,
    user: &Address,
    platform: &SharePlatform,
    engagement_count: u32,
) -> Result<(), SocialSharingError> {
    let key = format_cert_key(certificate_id, user, platform);
    
    if let Some(mut record) = env.storage().persistent().get::<_, ShareRecord>(&key) {
        record.engagement_count = engagement_count;
        record.verified = true;
        env.storage().persistent().set(&key, &record);
        Ok(())
    } else {
        Err(SocialSharingError::ShareRecordNotFound)
    }
}

/// Get global analytics.
pub fn get_analytics(env: &Env) -> Result<SocialSharingAnalytics, SocialSharingError> {
    let key = symbol_short!("analytics");
    
    Ok(env.storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| SocialSharingAnalytics::new()))
}

/// Get analytics for a specific certificate.
pub fn get_certificate_analytics(
    env: &Env,
    certificate_id: &BytesN<32>,
) -> Result<SocialSharingAnalytics, SocialSharingError> {
    let key = format_cert_analytics_key(certificate_id);
    
    Ok(env.storage()
        .persistent()
        .get(&key)
        .unwrap_or_else(|| SocialSharingAnalytics::new()))
}

/// Increment share count for a certificate.
pub fn increment_share_count(env: &Env, certificate_id: &BytesN<32>) -> Result<(), SocialSharingError> {
    let key = format_certificate_shares_key(certificate_id);
    let mut analytics = get_certificate_analytics(env, certificate_id)?;
    
    analytics.total_shares += 1;
    
    env.storage()
        .persistent()
        .set(&key, &analytics);

    Ok(())
}

// ─────────────────────────────────────────────────────────────
// Key Formatting Helpers
// ─────────────────────────────────────────────────────────────

fn format_cert_key(
    certificate_id: &BytesN<32>,
    user: &Address,
    platform: &SharePlatform,
) -> String {
    // Use a composite key format: cert_id:user:platform
    let platform_str = match platform {
        SharePlatform::Twitter => "twitter",
        SharePlatform::LinkedIn => "linkedin",
        SharePlatform::Facebook => "facebook",
    };

    let cert_hex = format_bytes_to_hex(certificate_id);
    String::from_slice(
        &Env::current(),
        &format!("share:{}:{}:{}", cert_hex, user, platform_str),
    )
}

fn format_certificate_shares_key(certificate_id: &BytesN<32>) -> String {
    let cert_hex = format_bytes_to_hex(certificate_id);
    String::from_slice(&Env::current(), &format!("cert_shares:{}", cert_hex))
}

fn format_user_shares_key(user: &Address) -> String {
    String::from_slice(&Env::current(), &format!("user_shares:{}", user))
}

fn format_cert_analytics_key(certificate_id: &BytesN<32>) -> String {
    let cert_hex = format_bytes_to_hex(certificate_id);
    String::from_slice(&Env::current(), &format!("cert_analytics:{}", cert_hex))
}

fn format_bytes_to_hex(bytes: &BytesN<32>) -> String {
    let mut result = String::new();
    let env = Env::current();
    
    for i in 0..32 {
        let byte = bytes.get_unchecked(i);
        let hex = format!("{:02x}", byte);
        result = result.append(&String::from_slice(&env, &hex));
    }
    result
}
