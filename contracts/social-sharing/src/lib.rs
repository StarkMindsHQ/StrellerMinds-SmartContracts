#![no_std]

pub mod errors;
pub mod events;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

use errors::SocialSharingError;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env, String, Vec};
use types::{SharePlatform, ShareRecord, SocialSharingAnalytics};

#[contract]
pub struct SocialSharingContract;

// ─────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────
fn require_auth(env: &Env, caller: &Address) {
    caller.require_auth();
}

fn validate_certificate_id(_env: &Env, cert_id: &BytesN<32>) -> Result<(), SocialSharingError> {
    // Validate that certificate ID is not zero
    let zero_bytes = BytesN::from_array(_env, &[0u8; 32]);
    if cert_id == &zero_bytes {
        return Err(SocialSharingError::InvalidCertificateId);
    }
    Ok(())
}

fn validate_share_message(message: &String) -> Result<(), SocialSharingError> {
    let message_len = message.len();
    if message_len == 0 || message_len > 500 {
        return Err(SocialSharingError::InvalidShareMessage);
    }
    Ok(())
}

fn validate_platform(_platform: &SharePlatform) -> Result<(), SocialSharingError> {
    // All platforms are valid if they're created through the enum
    Ok(())
}

// ─────────────────────────────────────────────────────────────
// Main Contract Implementation
// ─────────────────────────────────────────────────────────────

#[contractimpl]
impl SocialSharingContract {
    /// Record a share of an achievement/credential to a social platform.
    /// 
    /// # Arguments
    /// * `env` - The Soroban environment
    /// * `caller` - Address of the user sharing
    /// * `certificate_id` - ID of the certificate/achievement being shared
    /// * `platform` - Target social media platform (Twitter, LinkedIn, Facebook)
    /// * `custom_message` - Optional custom message (max 500 chars)
    /// 
    /// # Returns
    /// A `ShareRecord` confirming the share was recorded
    pub fn share_achievement(
        env: Env,
        caller: Address,
        certificate_id: BytesN<32>,
        platform: SharePlatform,
        custom_message: String,
    ) -> Result<ShareRecord, SocialSharingError> {
        require_auth(&env, &caller);
        
        // Validate inputs
        validate_certificate_id(&env, &certificate_id)?;
        validate_share_message(&custom_message)?;
        validate_platform(&platform)?;

        let timestamp = env.ledger().timestamp();

        // Create share record
        let share_record = ShareRecord {
            certificate_id: certificate_id.clone(),
            user: caller.clone(),
            platform: platform.clone(),
            custom_message: custom_message.clone(),
            share_url: generate_share_url(&env, &certificate_id, &platform),
            timestamp,
            engagement_count: 0,
            verified: false,
        };

        // Store the share record
        storage::store_share_record(&env, &share_record)?;

        // Emit share event
        events::emit_share_event(&env, &caller, &certificate_id, &platform, timestamp);

        // Update analytics
        storage::increment_share_count(&env, &certificate_id)?;

        Ok(share_record)
    }

    /// Get a specific share record.
    pub fn get_share_record(
        env: Env,
        certificate_id: BytesN<32>,
        user: Address,
        platform: SharePlatform,
    ) -> Result<Option<ShareRecord>, SocialSharingError> {
        storage::get_share_record(&env, &certificate_id, &user, &platform)
    }

    /// Get all share records for a certificate.
    pub fn get_certificate_shares(
        env: Env,
        certificate_id: BytesN<32>,
    ) -> Result<Vec<ShareRecord>, SocialSharingError> {
        storage::get_certificate_shares(&env, &certificate_id)
    }

    /// Get all shares by a user.
    pub fn get_user_shares(
        env: Env,
        user: Address,
    ) -> Result<Vec<ShareRecord>, SocialSharingError> {
        storage::get_user_shares(&env, &user)
    }

    /// Update engagement metrics for a share.
    /// Called by analytics service to track real-world engagement.
    pub fn update_engagement(
        env: Env,
        admin: Address,
        certificate_id: BytesN<32>,
        user: Address,
        platform: SharePlatform,
        engagement_count: u32,
    ) -> Result<(), SocialSharingError> {
        require_auth(&env, &admin);

        // Verify admin authorization
        let contract_admin = storage::get_admin(&env)?;
        if admin != contract_admin {
            return Err(SocialSharingError::Unauthorized);
        }

        storage::update_engagement(&env, &certificate_id, &user, &platform, engagement_count)?;

        Ok(())
    }

    /// Get analytics for shares platform-wide.
    pub fn get_analytics(env: Env) -> Result<SocialSharingAnalytics, SocialSharingError> {
        storage::get_analytics(&env)
    }

    /// Get analytics for a specific certificate.
    pub fn get_certificate_analytics(
        env: Env,
        certificate_id: BytesN<32>,
    ) -> Result<SocialSharingAnalytics, SocialSharingError> {
        storage::get_certificate_analytics(&env, &certificate_id)
    }

    /// Initialize the contract with an admin address.
    pub fn init_contract(env: Env, admin: Address) -> Result<(), SocialSharingError> {
        // Check if already initialized
        if storage::is_initialized(&env) {
            return Err(SocialSharingError::AlreadyInitialized);
        }

        storage::set_admin(&env, &admin)?;
        storage::mark_initialized(&env)?;

        Ok(())
    }

    /// Set a new admin (requires current admin auth).
    pub fn set_admin(env: Env, new_admin: Address) -> Result<(), SocialSharingError> {
        let current_admin = storage::get_admin(&env)?;
        require_auth(&env, &current_admin);

        storage::set_admin(&env, &new_admin)?;

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────
// Utilities
// ─────────────────────────────────────────────────────────────

fn generate_share_url(env: &Env, cert_id: &BytesN<32>, platform: &SharePlatform) -> String {
    let cert_hex = bytes_to_hex(cert_id);
    match platform {
        SharePlatform::Twitter => {
            String::from_slice(env, &format!("twitter://share/{}", cert_hex))
        }
        SharePlatform::LinkedIn => {
            String::from_slice(env, &format!("linkedin://share/{}", cert_hex))
        }
        SharePlatform::Facebook => {
            String::from_slice(env, &format!("facebook://share/{}", cert_hex))
        }
    }
}

fn bytes_to_hex(bytes: &BytesN<32>) -> String {
    let mut result = String::new();
    for i in 0..32 {
        let byte = bytes.get_unchecked(i);
        let hex = format!("{:02x}", byte);
        result = result.append(&String::from_slice(&Env::current(), &hex));
    }
    result
}
