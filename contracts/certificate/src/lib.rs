#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String};

#[cfg(test)]
mod test;

use errors::CertificateError;
use shared::logger::{LogLevel, Logger};
use shared::monitoring::{ContractHealthReport, Monitor};
use shared::rate_limiter::{enforce_rate_limit, RateLimitConfig};
use shared::{log_error, log_info, log_warn};
use soroban_sdk::{contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map, String, Symbol, Vec};
use types::{
    AuditAction, BatchResult, CertDataKey, CertRateLimitConfig, Certificate, CertificateAnalytics,
    CertificateBackup, CertificateStatus, CertificateTemplate, ComplianceRecord,
    ComplianceStandard, MintCertificateParams, MultiSigAuditEntry, MultiSigCertificateRequest,
    MultiSigConfig, MultiSigRequestStatus, RecoveryRequest, RecoveryStatus, RevocationRecord,
    ShareRecord, TemplateField,
};

use shared::gdpr_types::GdprCertificateExport;

/// Maximum number of approvers per config (gas guard).
const MAX_APPROVERS: u32 = 10;
/// Minimum timeout: 1 hour.
const MIN_TIMEOUT: u64 = 3_600;
/// Maximum timeout: 30 days.
const MAX_TIMEOUT: u64 = 2_592_000;
/// Maximum batch size (gas guard).
const MAX_BATCH_SIZE: u32 = 100;
/// Maximum share records per certificate.
const MAX_SHARES_PER_CERT: u32 = 100;
/// Rate limit operation ID for multisig requests.
const RL_OP_MULTISIG_REQUEST: u64 = 1;

#[contract]
pub struct CertificateContract;

// ─────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────
fn require_admin(env: &Env, caller: &Address) -> Result<(), CertificateError> {
    caller.require_auth();
    let admin = storage::get_admin(env);
    if *caller != admin {
        log_error!(env, symbol_short!("cert"), symbol_short!("unauth"));
        return Err(CertificateError::Unauthorized);
    }
    Ok(())
}

fn require_initialized(env: &Env) -> Result<(), CertificateError> {
    if !storage::is_initialized(env) {
        return Err(CertificateError::NotInitialized);
    }
    Ok(())
}

/// Deterministic request ID from counter.
fn generate_request_id(env: &Env) -> BytesN<32> {
    let counter = storage::next_request_counter(env);
    let mut bytes = [0u8; 32];
    let counter_bytes = counter.to_be_bytes();
    bytes[24..32].copy_from_slice(&counter_bytes);
    // Mix in ledger timestamp for uniqueness
    let ts = env.ledger().timestamp().to_be_bytes();
    bytes[16..24].copy_from_slice(&ts);
    BytesN::from_array(env, &bytes)
}

/// Deterministic certificate anchor hash.
fn generate_blockchain_anchor(env: &Env, cert_id: &BytesN<32>) -> soroban_sdk::Bytes {
    let counter = storage::next_certificate_counter(env);
    let mut bytes = [0u8; 32];
    // Embed certificate id prefix
    let cert_bytes = cert_id.to_array();
    bytes[0..16].copy_from_slice(&cert_bytes[0..16]);
    // Embed counter
    let counter_bytes = counter.to_be_bytes();
    bytes[24..32].copy_from_slice(&counter_bytes);
    soroban_sdk::Bytes::from_array(env, &bytes)
}

fn update_analytics_field(env: &Env, updater: impl FnOnce(&mut CertificateAnalytics)) {
    let mut analytics = storage::get_analytics(env);
    updater(&mut analytics);
    analytics.last_updated = env.ledger().timestamp();
    storage::set_analytics(env, &analytics);
}

#[contract]
pub struct DashboardPreferencesContract;

#[contractimpl]
impl DashboardPreferencesContract {
    /// Saves the user's customized dashboard layout and widget preferences.
    /// The layout is expected to be a serialized string (e.g. JSON)
    /// that the frontend can parse to restore the drag-and-drop state.
    pub fn save_layout(
        env: Env,
        user: Address,
        layout_data: String,
    ) {
        user.require_auth();
        
        env.storage()
            .persistent()
            .set(&DataKey::UserLayout(user), &layout_data);
    }

    /// Retrieves the user's dashboard layout.
    pub fn get_layout(env: Env, user: Address) -> Option<String> {
        env.storage()
            .persistent()
            .get(&DataKey::UserLayout(user))
    }

    /// Deletes the user's dashboard layout, reverting to the frontend's default.
    pub fn clear_layout(env: Env, user: Address) {
        user.require_auth();
        
        env.storage()
            .persistent()
            .remove(&DataKey::UserLayout(user));
    }
}

#[cfg(test)]
mod test;