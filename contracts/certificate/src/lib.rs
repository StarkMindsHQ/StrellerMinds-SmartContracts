#![no_std]

pub mod errors;
pub mod events;
pub mod storage;
pub mod storage_optimizer;
pub mod two_factor_integration;
pub mod types;

#[cfg(test)]
mod test;

use errors::CertificateError;
use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env, Vec};
use types::CertificateStatus;

#[contract]
pub struct CertificateContract;

#[contractimpl]
impl CertificateContract {
    /// Initialize the certificate contract with an admin address.
    pub fn initialize(env: Env, admin: Address) -> Result<(), CertificateError> {
        if storage::is_initialized(&env) {
            return Err(CertificateError::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        storage::set_initialized(&env);
        Ok(())
    }

    /// Scan all issued certificates and remove storage entries for those that have
    /// passed their `expiry_date`, freeing ledger memory (fixes #439).
    ///
    /// Only the contract admin may call this function.
    /// Returns the number of expired certificate entries that were cleaned up.
    pub fn cleanup_expired_certificates(
        env: Env,
        caller: Address,
    ) -> Result<u32, CertificateError> {
        require_initialized(&env)?;
        require_admin(&env, &caller)?;

        let now = env.ledger().timestamp();
        let all_ids = storage::get_all_certificates(&env);
        let mut remaining: Vec<BytesN<32>> = Vec::new(&env);
        let mut cleaned: u32 = 0;

        for cert_id in all_ids.iter() {
            match storage::get_certificate(&env, &cert_id) {
                Some(cert) => {
                    if cert.expiry_date > 0 && now >= cert.expiry_date {
                        // Remove the storage entry to release ledger memory
                        storage::remove_certificate(&env, &cert_id);
                        cleaned += 1;
                    } else {
                        remaining.push_back(cert_id);
                    }
                }
                None => {
                    // Already removed; drop from index silently
                }
            }
        }

        storage::set_all_certificates(&env, &remaining);
        Ok(cleaned)
    }

    /// Return the number of certificates currently tracked in the global index.
    pub fn get_certificate_count(env: Env) -> u32 {
        storage::get_all_certificates(&env).len()
    }

    pub fn get_certificate(env: Env, certificate_id: BytesN<32>) -> Option<types::Certificate> {
        storage::get_certificate(&env, &certificate_id)
    }

    pub fn verify_certificate(env: Env, certificate_id: BytesN<32>) -> bool {
        if let Some(cert) = storage::get_certificate(&env, &certificate_id) {
            if cert.status != types::CertificateStatus::Active {
                return false;
            }
            if cert.expiry_date > 0 && cert.expiry_date < env.ledger().timestamp() {
                return false;
            }
            true
        } else {
            false
        }
    }

    pub fn batch_issue_certificates(
        env: Env,
        admin: Address,
        params_list: Vec<types::MintCertificateParams>,
    ) -> types::BatchResult {
        require_admin(&env, &admin);
        admin.require_auth();

        let mut succeeded = 0;
        let mut failed = 0;
        let mut certificate_ids = Vec::new(&env);

        for params in params_list.iter() {
            // Check if multi-sig is required for this course
            if let Some(config) = storage::get_multisig_config(&env, &params.course_id) {
                // Create multi-sig request ID by hashing the certificate ID + student address
                let mut hasher = env.crypto().sha256(&params.certificate_id.clone().into());
                // For simplicity, we just use the cert ID hash as the request ID
                let request_id: BytesN<32> = hasher.into();

                let request = types::MultiSigRequest {
                    request_id: request_id.clone(),
                    params: params.clone(),
                    approvals: Vec::new(&env),
                    status: types::MultiSigRequestStatus::Pending,
                    created_at: env.ledger().timestamp(),
                };
                storage::set_multisig_request(&env, &request_id, &request);
                storage::add_pending_request(&env, &request_id);
                
                events::emit_multisig_request_created(&env, &request_id, &params.course_id);
                succeeded += 1; // Counted as "submitted"
            } else {
                // Issue immediately
                let cert = types::Certificate {
                    certificate_id: params.certificate_id.clone(),
                    course_id: params.course_id.clone(),
                    student: params.student.clone(),
                    title: params.title.clone(),
                    description: params.description.clone(),
                    metadata_uri: params.metadata_uri.clone(),
                    issued_at: env.ledger().timestamp(),
                    expiry_date: params.expiry_date,
                    status: types::CertificateStatus::Active,
                    issuer: admin.clone(),
                    version: 1,
                    blockchain_anchor: None,
                    template_id: None,
                    share_count: 0,
                };
                storage::set_certificate(&env, &params.certificate_id, &cert);
                storage::add_student_certificate(&env, &params.student, &params.certificate_id);
                storage::add_to_all_certificates(&env, &params.certificate_id);
                
                // Update analytics
                let mut analytics = storage::get_analytics(&env);
                analytics.total_issued += 1;
                analytics.active_certificates += 1;
                storage::set_analytics(&env, &analytics);

                events::emit_certificate_issued(&env, &params.certificate_id, &params.student);
                certificate_ids.push_back(params.certificate_id.clone());
                succeeded += 1;
            }
        }

        types::BatchResult {
            total: params_list.len(),
            succeeded,
            failed,
            certificate_ids,
        }
    }

    pub fn configure_multisig(env: Env, admin: Address, config: types::MultiSigConfig) {
        require_admin(&env, &admin);
        admin.require_auth();
        storage::set_multisig_config(&env, &config.course_id, &config);
    }

    pub fn process_multisig_approval(
        env: Env,
        approver: Address,
        request_id: BytesN<32>,
        approved: bool,
    ) -> Result<(), CertificateError> {
        approver.require_auth();

        let mut request = storage::get_multisig_request(&env, &request_id)
            .ok_or(CertificateError::RequestNotFound)?;

        if request.status != types::MultiSigRequestStatus::Pending {
            return Err(CertificateError::InvalidStatus);
        }

        let config = storage::get_multisig_config(&env, &request.params.course_id)
            .ok_or(CertificateError::ConfigNotFound)?;

        // Verify approver is authorized
        if !config.authorized_approvers.contains(&approver) {
            return Err(CertificateError::NotAuthorized);
        }

        // Add approval
        if approved {
            if !request.approvals.contains(&approver) {
                request.approvals.push_back(approver.clone());
            }
        } else {
            request.status = types::MultiSigRequestStatus::Rejected;
            storage::set_multisig_request(&env, &request_id, &request);
            storage::remove_pending_request(&env, &request_id);
            return Ok(());
        }

        // Check threshold
        if request.approvals.len() >= config.required_approvals {
            request.status = types::MultiSigRequestStatus::Executed;
            
            // Execute: Issue the certificate
            let params = &request.params;
            let cert = types::Certificate {
                certificate_id: params.certificate_id.clone(),
                course_id: params.course_id.clone(),
                student: params.student.clone(),
                title: params.title.clone(),
                description: params.description.clone(),
                metadata_uri: params.metadata_uri.clone(),
                issued_at: env.ledger().timestamp(),
                expiry_date: params.expiry_date,
                status: types::CertificateStatus::Active,
                issuer: approver.clone(), // Final approver as "issuer" context
                version: 1,
                blockchain_anchor: None,
                template_id: None,
                share_count: 0,
            };
            storage::set_certificate(&env, &params.certificate_id, &cert);
            storage::add_student_certificate(&env, &params.student, &params.certificate_id);
            storage::add_to_all_certificates(&env, &params.certificate_id);
            
            // Update analytics
            let mut analytics = storage::get_analytics(&env);
            analytics.total_issued += 1;
            analytics.active_certificates += 1;
            storage::set_analytics(&env, &analytics);

            storage::remove_pending_request(&env, &request_id);
            events::emit_certificate_issued(&env, &params.certificate_id, &params.student);
        }

        storage::set_multisig_request(&env, &request_id, &request);
        Ok(())
    }

    pub fn get_analytics(env: Env) -> types::CertificateAnalytics {
        storage::get_analytics(&env)
    }

    pub fn get_student_certificates(env: Env, student: Address) -> Vec<BytesN<32>> {
        storage::get_student_certificates(&env, &student)
    }

    pub fn revoke_certificate(
        env: Env,
        admin: Address,
        certificate_id: BytesN<32>,
        reason: String,
    ) -> Result<(), CertificateError> {
        require_admin(&env, &admin);
        admin.require_auth();

        let mut cert = storage::get_certificate(&env, &certificate_id)
            .ok_or(CertificateError::CertificateNotFound)?;

        if cert.status == types::CertificateStatus::Revoked {
            return Err(CertificateError::InvalidStatus);
        }

        cert.status = types::CertificateStatus::Revoked;
        storage::set_certificate(&env, &certificate_id, &cert);

        let revocation = types::RevocationRecord {
            certificate_id: certificate_id.clone(),
            revoked_by: admin.clone(),
            revoked_at: env.ledger().timestamp(),
            reason,
            reissuance_eligible: true,
        };
        storage::set_revocation(&env, &certificate_id, &revocation);

        // Update analytics
        let mut analytics = storage::get_analytics(&env);
        analytics.total_revoked += 1;
        if analytics.active_certificates > 0 {
            analytics.active_certificates -= 1;
        }
        storage::set_analytics(&env, &analytics);

        events::emit_certificate_revoked(&env, &certificate_id);
        Ok(())
    }

    pub fn get_revocation_record(env: Env, certificate_id: BytesN<32>) -> Option<types::RevocationRecord> {
        storage::get_revocation(&env, &certificate_id)
    }

    pub fn share_achievement(
        env: Env,
        user: Address,
        certificate_id: BytesN<32>,
        platform: u32,
        custom_message: String,
    ) -> types::ShareRecord {
        user.require_auth();

        let mut cert = storage::get_certificate(&env, &certificate_id)
            .ok_or(CertificateError::CertificateNotFound)
            .unwrap();

        let share = types::ShareRecord {
            certificate_id: certificate_id.clone(),
            user: user.clone(),
            platform,
            custom_message,
            share_url: String::from_str(&env, "https://strellerminds.com/verify/"), // Mock URL prefix
            timestamp: env.ledger().timestamp(),
            engagement_count: 0,
            verified: true,
        };

        storage::set_share_record(&env, &certificate_id, &user, &share);
        
        cert.share_count += 1;
        storage::set_certificate(&env, &certificate_id, &cert);

        // Update analytics
        let mut analytics = storage::get_analytics(&env);
        analytics.total_shared += 1;
        storage::set_analytics(&env, &analytics);

        events::emit_certificate_shared(&env, &certificate_id, &user, platform);
        share
    }

    pub fn get_certificate_shares(env: Env, certificate_id: BytesN<32>) -> Vec<types::ShareRecord> {
        storage::get_certificate_shares(&env, &certificate_id)
    }

    pub fn get_user_shares(env: Env, user: Address) -> Vec<types::ShareRecord> {
        storage::get_user_shares(&env, &user)
    }

    pub fn update_engagement(
        env: Env,
        admin: Address,
        certificate_id: BytesN<32>,
        user: Address,
        platform: u32,
        engagement_count: u32,
    ) -> Result<(), CertificateError> {
        require_admin(&env, &admin);
        admin.require_auth();

        let mut share = storage::get_share_record(&env, &certificate_id, &user)
            .ok_or(CertificateError::ShareNotFound)?;

        share.engagement_count = engagement_count;
        storage::set_share_record(&env, &certificate_id, &user, &share);

        Ok(())
    }
}

// ─────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────
fn require_admin(env: &Env, caller: &Address) -> Result<(), CertificateError> {
    caller.require_auth();
    let admin = storage::get_admin(env);
    if *caller != admin {
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
