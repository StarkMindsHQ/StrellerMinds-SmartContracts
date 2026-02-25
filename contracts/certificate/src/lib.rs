#![no_std]

pub mod errors;
pub mod events;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

use errors::CertificateError;
use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, String, Vec};
use types::{
    AuditAction, BatchResult, Certificate, CertificateAnalytics, CertificateStatus,
    CertificateTemplate, ComplianceRecord, ComplianceStandard, MintCertificateParams,
    MultiSigAuditEntry, MultiSigCertificateRequest, MultiSigConfig, MultiSigRequestStatus,
    RevocationRecord, ShareRecord, TemplateField,
};

/// Maximum number of approvers per config (gas guard).
const MAX_APPROVERS: u32 = 10;
/// Minimum timeout: 1 hour.
const MIN_TIMEOUT: u64 = 3_600;
/// Maximum timeout: 30 days.
const MAX_TIMEOUT: u64 = 2_592_000;
/// Maximum batch size (gas guard).
const MAX_BATCH_SIZE: u32 = 25;
/// Maximum share records per certificate.
const MAX_SHARES_PER_CERT: u32 = 100;

#[contract]
pub struct CertificateContract;

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

fn record_audit(
    env: &Env,
    request_id: &BytesN<32>,
    action: AuditAction,
    actor: &Address,
    details: &str,
) {
    let entry = MultiSigAuditEntry {
        request_id: request_id.clone(),
        action,
        actor: actor.clone(),
        timestamp: env.ledger().timestamp(),
        details: String::from_str(env, details),
    };
    storage::add_audit_entry(env, request_id, &entry);
}

#[contractimpl]
impl CertificateContract {
    // ─────────────────────────────────────────────────────────
    // Initialisation
    // ─────────────────────────────────────────────────────────
    pub fn initialize(env: Env, admin: Address) -> Result<(), CertificateError> {
        if storage::is_initialized(&env) {
            return Err(CertificateError::AlreadyInitialized);
        }
        admin.require_auth();
        storage::set_admin(&env, &admin);
        storage::set_initialized(&env);
        Ok(())
    }

    // ─────────────────────────────────────────────────────────
    // 1. Hierarchical Multi-Sig Configuration
    // ─────────────────────────────────────────────────────────
    pub fn configure_multisig(
        env: Env,
        admin: Address,
        config: MultiSigConfig,
    ) -> Result<(), CertificateError> {
        require_initialized(&env)?;
        require_admin(&env, &admin)?;

        // Validate configuration bounds
        if config.required_approvals == 0 {
            return Err(CertificateError::InvalidApprovalThreshold);
        }
        if config.authorized_approvers.len() < config.required_approvals {
            return Err(CertificateError::InvalidApprovalThreshold);
        }
        if config.authorized_approvers.len() > MAX_APPROVERS {
            return Err(CertificateError::TooManyApprovers);
        }
        if config.timeout_duration < MIN_TIMEOUT {
            return Err(CertificateError::TimeoutTooShort);
        }
        if config.timeout_duration > MAX_TIMEOUT {
            return Err(CertificateError::TimeoutTooLong);
        }

        events::emit_multisig_config_updated(&env, &config.course_id, &admin);
        storage::set_multisig_config(&env, &config.course_id, &config);
        Ok(())
    }

    pub fn get_multisig_config(env: Env, course_id: String) -> Option<MultiSigConfig> {
        storage::get_multisig_config(&env, &course_id)
    }

    // ─────────────────────────────────────────────────────────
    // 2. Request Creation
    // ─────────────────────────────────────────────────────────
    pub fn create_multisig_request(
        env: Env,
        requester: Address,
        params: MintCertificateParams,
        reason: String,
    ) -> Result<BytesN<32>, CertificateError> {
        require_initialized(&env)?;
        requester.require_auth();

        let config = storage::get_multisig_config(&env, &params.course_id)
            .ok_or(CertificateError::ConfigNotFound)?;

        let request_id = generate_request_id(&env);
        let now = env.ledger().timestamp();

        let request = MultiSigCertificateRequest {
            request_id: request_id.clone(),
            certificate_params: params.clone(),
            requester: requester.clone(),
            required_approvals: config.required_approvals,
            current_approvals: 0,
            approvers: Vec::new(&env),
            approval_records: Vec::new(&env),
            status: MultiSigRequestStatus::Pending,
            created_at: now,
            expires_at: now + config.timeout_duration,
            reason,
            priority: config.priority.clone(),
        };

        storage::set_multisig_request(&env, &request_id, &request);
        storage::add_pending_request(&env, &request_id);

        // Track for each approver
        for approver in config.authorized_approvers.iter() {
            storage::add_approver_pending(&env, &approver, &request_id);
        }

        record_audit(
            &env,
            &request_id,
            AuditAction::Created,
            &requester,
            "Multi-sig request created",
        );

        events::emit_multisig_request_created(&env, &request_id, &requester, &params.course_id);

        update_analytics_field(&env, |a| a.pending_requests += 1);

        Ok(request_id)
    }

    // ─────────────────────────────────────────────────────────
    // 3. Approval Processing
    // ─────────────────────────────────────────────────────────
    pub fn process_multisig_approval(
        env: Env,
        approver: Address,
        request_id: BytesN<32>,
        approved: bool,
        comments: String,
        signature_hash: Option<soroban_sdk::Bytes>,
    ) -> Result<(), CertificateError> {
        require_initialized(&env)?;
        approver.require_auth();

        let mut request = storage::get_multisig_request(&env, &request_id)
            .ok_or(CertificateError::MultiSigRequestNotFound)?;

        // Validate status
        if request.status != MultiSigRequestStatus::Pending {
            return Err(CertificateError::RequestNotPending);
        }

        // Check expiry
        let now = env.ledger().timestamp();
        if now > request.expires_at {
            request.status = MultiSigRequestStatus::Expired;
            storage::set_multisig_request(&env, &request_id, &request);
            record_audit(
                &env,
                &request_id,
                AuditAction::Expired,
                &approver,
                "Request expired",
            );
            return Err(CertificateError::MultiSigRequestExpired);
        }

        // Check authorization
        let config = storage::get_multisig_config(&env, &request.certificate_params.course_id)
            .ok_or(CertificateError::ConfigNotFound)?;

        let is_authorized = config.authorized_approvers.iter().any(|a| a == approver);
        if !is_authorized {
            return Err(CertificateError::ApproverNotAuthorized);
        }

        // Prevent duplicate approvals
        for record in request.approval_records.iter() {
            if record.approver == approver {
                return Err(CertificateError::AlreadyApproved);
            }
        }

        // Record the approval/rejection
        let record = types::ApprovalRecord {
            approver: approver.clone(),
            approved,
            timestamp: now,
            signature_hash,
            comments,
        };
        request.approval_records.push_back(record);

        if approved {
            request.current_approvals += 1;
            request.approvers.push_back(approver.clone());

            events::emit_multisig_approval_granted(
                &env,
                &request_id,
                &approver,
                request.current_approvals,
                request.required_approvals,
            );

            record_audit(
                &env,
                &request_id,
                AuditAction::ApprovalGranted,
                &approver,
                "Approval granted",
            );

            // Check if threshold reached
            if request.current_approvals >= request.required_approvals {
                request.status = MultiSigRequestStatus::Approved;
                events::emit_multisig_request_approved(&env, &request_id);

                // Auto-execute if configured
                if config.auto_execute {
                    Self::internal_execute(&env, &mut request)?;
                }
            }
        } else {
            request.status = MultiSigRequestStatus::Rejected;
            events::emit_multisig_request_rejected(&env, &request_id, &approver);
            record_audit(
                &env,
                &request_id,
                AuditAction::ApprovalRejected,
                &approver,
                "Request rejected",
            );
            update_analytics_field(&env, |a| {
                if a.pending_requests > 0 {
                    a.pending_requests -= 1;
                }
            });
        }

        // Remove from approver's pending list
        storage::remove_approver_pending(&env, &approver, &request_id);
        storage::set_multisig_request(&env, &request_id, &request);

        Ok(())
    }

    // ─────────────────────────────────────────────────────────
    // 4. Manual Execution
    // ─────────────────────────────────────────────────────────
    pub fn execute_multisig_request(
        env: Env,
        executor: Address,
        request_id: BytesN<32>,
    ) -> Result<(), CertificateError> {
        require_initialized(&env)?;
        executor.require_auth();

        let mut request = storage::get_multisig_request(&env, &request_id)
            .ok_or(CertificateError::MultiSigRequestNotFound)?;

        if request.status == MultiSigRequestStatus::Executed {
            return Err(CertificateError::RequestAlreadyExecuted);
        }
        if request.status != MultiSigRequestStatus::Approved {
            return Err(CertificateError::InsufficientApprovals);
        }

        Self::internal_execute(&env, &mut request)?;

        record_audit(
            &env,
            &request_id,
            AuditAction::Executed,
            &executor,
            "Request manually executed",
        );
        storage::set_multisig_request(&env, &request_id, &request);
        Ok(())
    }

    /// Internal certificate issuance on approval completion.
    fn internal_execute(
        env: &Env,
        request: &mut MultiSigCertificateRequest,
    ) -> Result<(), CertificateError> {
        let params = &request.certificate_params;
        let anchor = generate_blockchain_anchor(env, &params.certificate_id);

        let certificate = Certificate {
            certificate_id: params.certificate_id.clone(),
            course_id: params.course_id.clone(),
            student: params.student.clone(),
            title: params.title.clone(),
            description: params.description.clone(),
            metadata_uri: params.metadata_uri.clone(),
            issued_at: env.ledger().timestamp(),
            expiry_date: params.expiry_date,
            status: CertificateStatus::Active,
            issuer: request.requester.clone(),
            version: 1,
            blockchain_anchor: Some(anchor),
            template_id: None,
            share_count: 0,
        };

        storage::set_certificate(env, &params.certificate_id, &certificate);
        storage::add_student_certificate(env, &params.student, &params.certificate_id);

        request.status = MultiSigRequestStatus::Executed;
        storage::set_multisig_request(env, &request.request_id, request);

        events::emit_certificate_issued(
            env,
            &params.certificate_id,
            &params.student,
            &params.course_id,
        );

        update_analytics_field(env, |a| {
            a.total_issued += 1;
            a.active_certificates += 1;
            if a.pending_requests > 0 {
                a.pending_requests -= 1;
            }
        });

        Ok(())
    }

    // ─────────────────────────────────────────────────────────
    // 5. Batch Certificate Issuance & Verification
    // ─────────────────────────────────────────────────────────
    pub fn batch_issue_certificates(
        env: Env,
        admin: Address,
        params_list: Vec<MintCertificateParams>,
    ) -> Result<BatchResult, CertificateError> {
        require_initialized(&env)?;
        require_admin(&env, &admin)?;

        let count = params_list.len();
        if count == 0 {
            return Err(CertificateError::BatchEmpty);
        }
        if count > MAX_BATCH_SIZE {
            return Err(CertificateError::BatchTooLarge);
        }

        let mut succeeded: u32 = 0;
        let mut failed: u32 = 0;
        let mut cert_ids: Vec<BytesN<32>> = Vec::new(&env);

        for params in params_list.iter() {
            // Skip duplicates
            if storage::get_certificate(&env, &params.certificate_id).is_some() {
                failed += 1;
                continue;
            }

            let anchor = generate_blockchain_anchor(&env, &params.certificate_id);
            let certificate = Certificate {
                certificate_id: params.certificate_id.clone(),
                course_id: params.course_id.clone(),
                student: params.student.clone(),
                title: params.title.clone(),
                description: params.description.clone(),
                metadata_uri: params.metadata_uri.clone(),
                issued_at: env.ledger().timestamp(),
                expiry_date: params.expiry_date,
                status: CertificateStatus::Active,
                issuer: admin.clone(),
                version: 1,
                blockchain_anchor: Some(anchor),
                template_id: None,
                share_count: 0,
            };

            storage::set_certificate(&env, &params.certificate_id, &certificate);
            storage::add_student_certificate(&env, &params.student, &params.certificate_id);
            cert_ids.push_back(params.certificate_id.clone());
            succeeded += 1;
        }

        update_analytics_field(&env, |a| {
            a.total_issued += succeeded;
            a.active_certificates += succeeded;
        });

        let result = BatchResult {
            total: count,
            succeeded,
            failed,
            certificate_ids: cert_ids,
        };

        events::emit_batch_completed(&env, count, succeeded, failed);
        Ok(result)
    }

    pub fn verify_certificate(
        env: Env,
        certificate_id: BytesN<32>,
    ) -> Result<bool, CertificateError> {
        require_initialized(&env)?;

        let cert = storage::get_certificate(&env, &certificate_id)
            .ok_or(CertificateError::CertificateNotFound)?;

        let is_valid = cert.status == CertificateStatus::Active
            && (cert.expiry_date == 0 || env.ledger().timestamp() <= cert.expiry_date)
            && cert.blockchain_anchor.is_some();

        events::emit_certificate_verified(&env, &certificate_id, is_valid);
        update_analytics_field(&env, |a| a.total_verified += 1);

        Ok(is_valid)
    }

    // ─────────────────────────────────────────────────────────
    // 6. Certificate Revocation & Reissuance
    // ─────────────────────────────────────────────────────────
    pub fn revoke_certificate(
        env: Env,
        admin: Address,
        certificate_id: BytesN<32>,
        reason: String,
        reissuance_eligible: bool,
    ) -> Result<(), CertificateError> {
        require_initialized(&env)?;
        require_admin(&env, &admin)?;

        let mut cert = storage::get_certificate(&env, &certificate_id)
            .ok_or(CertificateError::CertificateNotFound)?;

        if cert.status == CertificateStatus::Revoked {
            return Err(CertificateError::CertificateRevoked);
        }

        cert.status = CertificateStatus::Revoked;
        storage::set_certificate(&env, &certificate_id, &cert);

        let revocation = RevocationRecord {
            certificate_id: certificate_id.clone(),
            revoked_by: admin.clone(),
            revoked_at: env.ledger().timestamp(),
            reason,
            reissuance_eligible,
        };
        storage::set_revocation(&env, &certificate_id, &revocation);

        events::emit_certificate_revoked(&env, &certificate_id, &admin);

        update_analytics_field(&env, |a| {
            a.total_revoked += 1;
            if a.active_certificates > 0 {
                a.active_certificates -= 1;
            }
        });

        Ok(())
    }

    pub fn reissue_certificate(
        env: Env,
        admin: Address,
        old_certificate_id: BytesN<32>,
        new_params: MintCertificateParams,
    ) -> Result<BytesN<32>, CertificateError> {
        require_initialized(&env)?;
        require_admin(&env, &admin)?;

        // Verify old certificate was revoked and is eligible
        let old_cert = storage::get_certificate(&env, &old_certificate_id)
            .ok_or(CertificateError::CertificateNotFound)?;
        if old_cert.status != CertificateStatus::Revoked {
            return Err(CertificateError::CertificateNotEligibleForReissue);
        }

        let revocation = storage::get_revocation(&env, &old_certificate_id)
            .ok_or(CertificateError::CertificateNotEligibleForReissue)?;
        if !revocation.reissuance_eligible {
            return Err(CertificateError::CertificateNotEligibleForReissue);
        }

        // Mark old certificate as reissued
        let mut old_mut = old_cert;
        old_mut.status = CertificateStatus::Reissued;
        storage::set_certificate(&env, &old_certificate_id, &old_mut);

        // Issue the new certificate
        let anchor = generate_blockchain_anchor(&env, &new_params.certificate_id);
        let new_cert = Certificate {
            certificate_id: new_params.certificate_id.clone(),
            course_id: new_params.course_id.clone(),
            student: new_params.student.clone(),
            title: new_params.title.clone(),
            description: new_params.description.clone(),
            metadata_uri: new_params.metadata_uri.clone(),
            issued_at: env.ledger().timestamp(),
            expiry_date: new_params.expiry_date,
            status: CertificateStatus::Active,
            issuer: admin.clone(),
            version: old_mut.version + 1,
            blockchain_anchor: Some(anchor),
            template_id: None,
            share_count: 0,
        };

        storage::set_certificate(&env, &new_params.certificate_id, &new_cert);
        storage::add_student_certificate(&env, &new_params.student, &new_params.certificate_id);

        events::emit_certificate_reissued(
            &env,
            &old_certificate_id,
            &new_params.certificate_id,
            &new_params.student,
        );

        update_analytics_field(&env, |a| {
            a.total_reissued += 1;
            a.active_certificates += 1;
        });

        Ok(new_params.certificate_id)
    }

    // ─────────────────────────────────────────────────────────
    // 7. Certificate Template System
    // ─────────────────────────────────────────────────────────
    pub fn create_template(
        env: Env,
        admin: Address,
        template_id: String,
        name: String,
        description: String,
        fields: Vec<TemplateField>,
    ) -> Result<(), CertificateError> {
        require_initialized(&env)?;
        require_admin(&env, &admin)?;

        if storage::get_template(&env, &template_id).is_some() {
            return Err(CertificateError::TemplateAlreadyExists);
        }

        let template = CertificateTemplate {
            template_id: template_id.clone(),
            name,
            description,
            fields,
            created_by: admin.clone(),
            created_at: env.ledger().timestamp(),
            is_active: true,
        };

        storage::set_template(&env, &template_id, &template);
        events::emit_template_created(&env, &template_id, &admin);

        record_audit(
            &env,
            // Use a zero-filled BytesN for template audit entries
            &BytesN::from_array(&env, &[0u8; 32]),
            AuditAction::TemplateCreated,
            &admin,
            "Template created",
        );

        Ok(())
    }

    pub fn get_template(env: Env, template_id: String) -> Option<CertificateTemplate> {
        storage::get_template(&env, &template_id)
    }

    /// Issue a certificate using a template, validating required fields.
    pub fn issue_with_template(
        env: Env,
        admin: Address,
        template_id: String,
        params: MintCertificateParams,
        field_values: Vec<String>,
    ) -> Result<BytesN<32>, CertificateError> {
        require_initialized(&env)?;
        require_admin(&env, &admin)?;

        let template =
            storage::get_template(&env, &template_id).ok_or(CertificateError::TemplateNotFound)?;
        if !template.is_active {
            return Err(CertificateError::TemplateInactive);
        }

        // Validate required field count matches
        let required_count = template.fields.iter().filter(|f| f.is_required).count();
        if field_values.len() < required_count as u32 {
            return Err(CertificateError::MissingRequiredField);
        }

        let anchor = generate_blockchain_anchor(&env, &params.certificate_id);
        let certificate = Certificate {
            certificate_id: params.certificate_id.clone(),
            course_id: params.course_id.clone(),
            student: params.student.clone(),
            title: params.title.clone(),
            description: params.description.clone(),
            metadata_uri: params.metadata_uri.clone(),
            issued_at: env.ledger().timestamp(),
            expiry_date: params.expiry_date,
            status: CertificateStatus::Active,
            issuer: admin.clone(),
            version: 1,
            blockchain_anchor: Some(anchor),
            template_id: Some(template_id),
            share_count: 0,
        };

        storage::set_certificate(&env, &params.certificate_id, &certificate);
        storage::add_student_certificate(&env, &params.student, &params.certificate_id);

        events::emit_certificate_issued(
            &env,
            &params.certificate_id,
            &params.student,
            &params.course_id,
        );

        update_analytics_field(&env, |a| {
            a.total_issued += 1;
            a.active_certificates += 1;
        });

        Ok(params.certificate_id)
    }

    // ─────────────────────────────────────────────────────────
    // 8. Certificate Analytics & Usage Tracking
    // ─────────────────────────────────────────────────────────
    pub fn get_analytics(env: Env) -> CertificateAnalytics {
        storage::get_analytics(&env)
    }

    pub fn get_student_certificates(env: Env, student: Address) -> Vec<BytesN<32>> {
        storage::get_student_certificates(&env, &student)
    }

    pub fn get_certificate(env: Env, certificate_id: BytesN<32>) -> Option<Certificate> {
        storage::get_certificate(&env, &certificate_id)
    }

    // ─────────────────────────────────────────────────────────
    // 9. Compliance Verification
    // ─────────────────────────────────────────────────────────
    pub fn verify_compliance(
        env: Env,
        verifier: Address,
        certificate_id: BytesN<32>,
        standard: ComplianceStandard,
        notes: String,
    ) -> Result<bool, CertificateError> {
        require_initialized(&env)?;
        verifier.require_auth();

        let cert = storage::get_certificate(&env, &certificate_id)
            .ok_or(CertificateError::CertificateNotFound)?;

        // Determine compliance based on certificate state
        let is_compliant = cert.status == CertificateStatus::Active
            && cert.blockchain_anchor.is_some()
            && (cert.expiry_date == 0 || env.ledger().timestamp() <= cert.expiry_date);

        let record = ComplianceRecord {
            certificate_id: certificate_id.clone(),
            standard,
            verified_at: env.ledger().timestamp(),
            verified_by: verifier.clone(),
            is_compliant,
            notes,
        };

        storage::set_compliance(&env, &certificate_id, &record);
        events::emit_compliance_checked(&env, &certificate_id, is_compliant);

        record_audit(
            &env,
            &certificate_id,
            AuditAction::ComplianceChecked,
            &verifier,
            "Compliance verification performed",
        );

        Ok(is_compliant)
    }

    pub fn get_compliance_record(env: Env, certificate_id: BytesN<32>) -> Option<ComplianceRecord> {
        storage::get_compliance(&env, &certificate_id)
    }

    // ─────────────────────────────────────────────────────────
    // 10. Certificate Sharing & Social Verification
    // ─────────────────────────────────────────────────────────
    pub fn share_certificate(
        env: Env,
        owner: Address,
        certificate_id: BytesN<32>,
        platform: String,
        verification_url: String,
    ) -> Result<(), CertificateError> {
        require_initialized(&env)?;
        owner.require_auth();

        let mut cert = storage::get_certificate(&env, &certificate_id)
            .ok_or(CertificateError::CertificateNotFound)?;

        if cert.status != CertificateStatus::Active {
            return Err(CertificateError::CertificateRevoked);
        }
        if cert.student != owner {
            return Err(CertificateError::Unauthorized);
        }
        if cert.share_count >= MAX_SHARES_PER_CERT {
            return Err(CertificateError::ShareLimitReached);
        }

        let record = ShareRecord {
            certificate_id: certificate_id.clone(),
            shared_by: owner.clone(),
            shared_at: env.ledger().timestamp(),
            platform: platform.clone(),
            verification_url,
        };

        storage::add_share_record(&env, &certificate_id, &record);
        cert.share_count += 1;
        storage::set_certificate(&env, &certificate_id, &cert);

        events::emit_certificate_shared(&env, &certificate_id, &owner, &platform);

        update_analytics_field(&env, |a| a.total_shared += 1);

        record_audit(
            &env,
            &certificate_id,
            AuditAction::Shared,
            &owner,
            "Certificate shared",
        );

        Ok(())
    }

    pub fn get_share_records(env: Env, certificate_id: BytesN<32>) -> Vec<ShareRecord> {
        storage::get_share_records(&env, &certificate_id)
    }

    // ─────────────────────────────────────────────────────────
    // 11. Authenticity Verification with Blockchain Anchors
    // ─────────────────────────────────────────────────────────
    pub fn verify_authenticity(
        env: Env,
        certificate_id: BytesN<32>,
    ) -> Result<bool, CertificateError> {
        require_initialized(&env)?;

        let cert = storage::get_certificate(&env, &certificate_id)
            .ok_or(CertificateError::CertificateNotFound)?;

        let is_authentic =
            cert.blockchain_anchor.is_some() && cert.status != CertificateStatus::Revoked;

        events::emit_certificate_verified(&env, &certificate_id, is_authentic);
        update_analytics_field(&env, |a| a.total_verified += 1);

        Ok(is_authentic)
    }

    // ─────────────────────────────────────────────────────────
    // 12. Query & Audit Trail
    // ─────────────────────────────────────────────────────────
    pub fn get_multisig_request(
        env: Env,
        request_id: BytesN<32>,
    ) -> Option<MultiSigCertificateRequest> {
        storage::get_multisig_request(&env, &request_id)
    }

    pub fn get_pending_approvals(env: Env, approver: Address) -> Vec<BytesN<32>> {
        storage::get_approver_pending(&env, &approver)
    }

    pub fn get_multisig_audit_trail(env: Env, request_id: BytesN<32>) -> Vec<MultiSigAuditEntry> {
        storage::get_audit_trail(&env, &request_id)
    }

    // ─────────────────────────────────────────────────────────
    // 13. Automated Lifecycle – Cleanup Expired Requests
    // ─────────────────────────────────────────────────────────
    pub fn cleanup_expired_requests(env: Env) -> Result<u32, CertificateError> {
        require_initialized(&env)?;

        let pending = storage::get_pending_requests(&env);
        let now = env.ledger().timestamp();
        let mut cleaned: u32 = 0;
        let mut remaining: Vec<BytesN<32>> = Vec::new(&env);

        for request_id in pending.iter() {
            if let Some(mut req) = storage::get_multisig_request(&env, &request_id) {
                if req.status == MultiSigRequestStatus::Pending && now > req.expires_at {
                    req.status = MultiSigRequestStatus::Expired;
                    storage::set_multisig_request(&env, &request_id, &req);
                    cleaned += 1;
                } else if req.status == MultiSigRequestStatus::Pending {
                    remaining.push_back(request_id);
                }
            }
        }

        storage::set_pending_requests(&env, &remaining);

        update_analytics_field(&env, |a| {
            a.total_expired += cleaned;
            if a.pending_requests >= cleaned {
                a.pending_requests -= cleaned;
            } else {
                a.pending_requests = 0;
            }
        });

        Ok(cleaned)
    }
}
