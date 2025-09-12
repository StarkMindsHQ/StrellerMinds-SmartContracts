use soroban_sdk::{Address, BytesN, Env, String, Vec};
use crate::errors::CertificateError;
use crate::types::{
    MultiSigCertificateRequest, MultiSigConfig, ApprovalRecord, MultiSigRequestStatus,
    CertificatePriority, MultiSigAuditEntry, AuditAction, DataKey, MintCertificateParams,
    CertificateMetadata, CertificateStatus
};
use crate::storage::CertificateStorage;
use crate::events::CertificateEvents;
use shared::access_control::AccessControl;
use shared::roles::Permission;

/// Multi-signature certificate management
pub struct MultiSigManager;

impl MultiSigManager {
    /// Configure multi-signature requirements for a course
    pub fn configure_multisig(
        env: &Env,
        admin: &Address,
        config: MultiSigConfig,
    ) -> Result<(), CertificateError> {
        // Validate admin permissions
        AccessControl::require_permission(env, admin, &Permission::UpdateCertificateMetadata)
            .map_err(|_| CertificateError::Unauthorized)?;

        // Validate configuration
        Self::validate_config(&config)?;

        // Store configuration
        env.storage()
            .persistent()
            .set(&DataKey::MultiSigConfig(config.course_id.clone()), &config);

        // Create audit entry
        let audit_entry = MultiSigAuditEntry {
            request_id: BytesN::from_array(env, &[0u8; 32]), // Config updates use zero ID
            action: AuditAction::ConfigUpdated,
            actor: admin.clone(),
            timestamp: env.ledger().timestamp(),
            details: format!("Multi-sig config updated for course: {}", config.course_id),
            previous_status: None,
            new_status: None,
        };
        Self::add_audit_entry(env, &audit_entry);

        Ok(())
    }

    /// Create a multi-signature certificate request
    pub fn create_multisig_request(
        env: &Env,
        requester: &Address,
        params: MintCertificateParams,
        reason: String,
    ) -> Result<BytesN<32>, CertificateError> {
        // Check if requester has permission to issue certificates
        AccessControl::require_permission(env, requester, &Permission::IssueCertificate)
            .map_err(|_| CertificateError::Unauthorized)?;

        // Get multi-sig configuration for the course
        let config = Self::get_config(env, &params.course_id)?;

        // Generate unique request ID
        let request_id = Self::generate_request_id(env, &params);

        // Check if request already exists
        if Self::has_request(env, &request_id) {
            return Err(CertificateError::MultiSigRequestAlreadyExists);
        }

        // Calculate expiry time
        let expires_at = env.ledger().timestamp() + config.timeout_duration;

        // Create the request
        let request = MultiSigCertificateRequest {
            request_id: request_id.clone(),
            certificate_params: params,
            requester: requester.clone(),
            required_approvals: config.required_approvals,
            current_approvals: 0,
            approvers: config.authorized_approvers.clone(),
            approval_records: Vec::new(env),
            status: MultiSigRequestStatus::Pending,
            created_at: env.ledger().timestamp(),
            expires_at,
            reason: reason.clone(),
            priority: config.priority,
        };

        // Store the request
        env.storage()
            .persistent()
            .set(&DataKey::MultiSigRequest(request_id.clone()), &request);

        // Add to pending approvals for each approver
        for approver in config.authorized_approvers.iter() {
            Self::add_pending_approval(env, &approver, &request_id);
        }

        // Create audit entry
        let audit_entry = MultiSigAuditEntry {
            request_id: request_id.clone(),
            action: AuditAction::RequestCreated,
            actor: requester.clone(),
            timestamp: env.ledger().timestamp(),
            details: reason,
            previous_status: None,
            new_status: Some(MultiSigRequestStatus::Pending),
        };
        Self::add_audit_entry(env, &audit_entry);

        // Emit event
        CertificateEvents::emit_multisig_request_created(env, &request_id, requester, &request.certificate_params.course_id);

        Ok(request_id)
    }

    /// Approve or reject a multi-signature request
    pub fn process_approval(
        env: &Env,
        approver: &Address,
        request_id: &BytesN<32>,
        approved: bool,
        comments: String,
        signature_hash: Option<BytesN<32>>,
    ) -> Result<(), CertificateError> {
        // Get the request
        let mut request = Self::get_request(env, request_id)?;

        // Validate request status
        if request.status != MultiSigRequestStatus::Pending {
            return match request.status {
                MultiSigRequestStatus::Approved => Err(CertificateError::MultiSigRequestAlreadyApproved),
                MultiSigRequestStatus::Rejected => Err(CertificateError::MultiSigRequestAlreadyRejected),
                MultiSigRequestStatus::Executed => Err(CertificateError::MultiSigRequestAlreadyExecuted),
                MultiSigRequestStatus::Expired => Err(CertificateError::MultiSigRequestExpired),
                _ => Err(CertificateError::InvalidInput),
            };
        }

        // Check if request has expired
        if env.ledger().timestamp() > request.expires_at {
            request.status = MultiSigRequestStatus::Expired;
            Self::update_request(env, &request);
            return Err(CertificateError::MultiSigRequestExpired);
        }

        // Validate approver authorization
        if !request.approvers.contains(approver) {
            return Err(CertificateError::ApproverNotAuthorized);
        }

        // Check if approver has already provided approval/rejection
        for record in request.approval_records.iter() {
            if record.approver == *approver {
                return Err(CertificateError::ApprovalAlreadyExists);
            }
        }

        // Create approval record
        let approval_record = ApprovalRecord {
            approver: approver.clone(),
            approved,
            timestamp: env.ledger().timestamp(),
            signature_hash,
            comments: comments.clone(),
        };

        // Add approval record
        request.approval_records.push_back(approval_record);

        // Update approval count if approved
        if approved {
            request.current_approvals += 1;
        }

        // Check if we have enough approvals or rejections
        let rejections = request.approval_records.iter()
            .filter(|r| !r.approved)
            .count() as u32;

        let new_status = if !approved && rejections >= 1 {
            // Single rejection rejects the entire request
            MultiSigRequestStatus::Rejected
        } else if request.current_approvals >= request.required_approvals {
            MultiSigRequestStatus::Approved
        } else {
            MultiSigRequestStatus::Pending
        };

        let previous_status = request.status.clone();
        request.status = new_status.clone();

        // Update the request
        Self::update_request(env, &request);

        // Create audit entry
        let action = if approved {
            AuditAction::ApprovalGranted
        } else {
            AuditAction::RequestRejected
        };

        let audit_entry = MultiSigAuditEntry {
            request_id: request_id.clone(),
            action,
            actor: approver.clone(),
            timestamp: env.ledger().timestamp(),
            details: comments,
            previous_status: Some(previous_status),
            new_status: Some(new_status.clone()),
        };
        Self::add_audit_entry(env, &audit_entry);

        // If approved and auto-execute is enabled, execute the certificate issuance
        if new_status == MultiSigRequestStatus::Approved {
            let config = Self::get_config(env, &request.certificate_params.course_id)?;
            if config.auto_execute {
                Self::execute_certificate_issuance(env, &request)?;
            }
        }

        Ok(())
    }

    /// Execute certificate issuance after approval
    pub fn execute_certificate_issuance(
        env: &Env,
        request: &MultiSigCertificateRequest,
    ) -> Result<(), CertificateError> {
        if request.status != MultiSigRequestStatus::Approved {
            return Err(CertificateError::InsufficientApprovals);
        }

        // Check if certificate already exists
        if CertificateStorage::has_certificate(env, &request.certificate_params.certificate_id) {
            return Err(CertificateError::CertificateAlreadyExists);
        }

        // Create packed certificate data directly
        let metadata = CertificateMetadata {
            course_id: request.certificate_params.course_id.clone(),
            student_id: request.certificate_params.student.clone(),
            instructor_id: request.requester.clone(),
            issue_date: env.ledger().timestamp(),
            metadata_uri: request.certificate_params.metadata_uri.clone(),
            token_id: request.certificate_params.certificate_id.clone(),
            title: request.certificate_params.title.clone(),
            description: request.certificate_params.description.clone(),
            status: CertificateStatus::Active,
            expiry_date: request.certificate_params.expiry_date,
            original_expiry_date: request.certificate_params.expiry_date,
            renewal_count: 0,
            last_renewed_date: 0,
        };
        
        let packed = crate::types::PackedCertificateData {
            metadata: metadata.clone(),
            owner: request.certificate_params.student.clone(),
            history: Vec::new(env),
        };

        // Store packed certificate
        CertificateStorage::set_certificate(env, &request.certificate_params.certificate_id, &packed);

        // Track certificate ownership
        CertificateStorage::add_user_certificate(env, &request.certificate_params.student, &request.certificate_params.certificate_id);
        CertificateStorage::add_instructor_certificate(env, &request.requester, &request.certificate_params.certificate_id);

        // Emit certificate minted event
        CertificateEvents::emit_certificate_minted(env, &request.requester, &request.certificate_params.student, &metadata);

        // Update request status to executed
        let mut updated_request = request.clone();
        updated_request.status = MultiSigRequestStatus::Executed;
        Self::update_request(env, &updated_request);

        // Create audit entry
        let audit_entry = MultiSigAuditEntry {
            request_id: request.request_id.clone(),
            action: AuditAction::CertificateIssued,
            actor: request.requester.clone(),
            timestamp: env.ledger().timestamp(),
            details: String::from_str(env, "Certificate issued via multi-sig approval"),
            previous_status: Some(MultiSigRequestStatus::Approved),
            new_status: Some(MultiSigRequestStatus::Executed),
        };
        Self::add_audit_entry(env, &audit_entry);

        Ok(())
    }

    /// Clean up expired requests
    pub fn cleanup_expired_requests(env: &Env) -> Result<u32, CertificateError> {
        let current_time = env.ledger().timestamp();
        let mut cleaned_count = 0u32;

        // This is a simplified cleanup - in a real implementation, you'd want to
        // iterate through a time-indexed structure for efficiency
        // For now, we'll mark this as a placeholder for the cleanup logic
        
        Ok(cleaned_count)
    }

    /// Get multi-signature configuration for a course
    pub fn get_config(env: &Env, course_id: &String) -> Result<MultiSigConfig, CertificateError> {
        env.storage()
            .persistent()
            .get(&DataKey::MultiSigConfig(course_id.clone()))
            .ok_or(CertificateError::MultiSigConfigNotFound)
    }

    /// Get multi-signature request
    pub fn get_request(env: &Env, request_id: &BytesN<32>) -> Result<MultiSigCertificateRequest, CertificateError> {
        env.storage()
            .persistent()
            .get(&DataKey::MultiSigRequest(request_id.clone()))
            .ok_or(CertificateError::MultiSigRequestNotFound)
    }

    /// Check if request exists
    pub fn has_request(env: &Env, request_id: &BytesN<32>) -> bool {
        env.storage()
            .persistent()
            .has(&DataKey::MultiSigRequest(request_id.clone()))
    }

    /// Get pending approvals for an approver
    pub fn get_pending_approvals(env: &Env, approver: &Address) -> Vec<BytesN<32>> {
        env.storage()
            .persistent()
            .get(&DataKey::PendingApprovals(approver.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }

    /// Get audit trail for a request
    pub fn get_audit_trail(env: &Env, request_id: &BytesN<32>) -> Vec<MultiSigAuditEntry> {
        env.storage()
            .persistent()
            .get(&DataKey::MultiSigAudit(request_id.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }

    // Private helper methods

    fn validate_config(config: &MultiSigConfig) -> Result<(), CertificateError> {
        if config.required_approvals == 0 {
            return Err(CertificateError::InvalidApprovalThreshold);
        }

        if config.required_approvals > config.authorized_approvers.len() as u32 {
            return Err(CertificateError::InvalidApprovalThreshold);
        }

        if config.timeout_duration < 3600 { // Minimum 1 hour
            return Err(CertificateError::TimeoutTooShort);
        }

        if config.timeout_duration > 2592000 { // Maximum 30 days
            return Err(CertificateError::TimeoutTooLong);
        }

        Ok(())
    }

    fn generate_request_id(env: &Env, params: &MintCertificateParams) -> BytesN<32> {
        // Generate a unique request ID based on certificate params and timestamp
        let mut data = Vec::new(env);
        data.extend_from_slice(&params.certificate_id.to_array());
        data.extend_from_slice(&env.ledger().timestamp().to_be_bytes());
        
        // Use the first 32 bytes of the combined data as the request ID
        let mut id_array = [0u8; 32];
        let data_slice = data.as_slice();
        let copy_len = core::cmp::min(32, data_slice.len());
        id_array[..copy_len].copy_from_slice(&data_slice[..copy_len]);
        
        BytesN::from_array(env, &id_array)
    }

    fn update_request(env: &Env, request: &MultiSigCertificateRequest) {
        env.storage()
            .persistent()
            .set(&DataKey::MultiSigRequest(request.request_id.clone()), request);
    }

    fn add_pending_approval(env: &Env, approver: &Address, request_id: &BytesN<32>) {
        let mut pending = Self::get_pending_approvals(env, approver);
        pending.push_back(request_id.clone());
        env.storage()
            .persistent()
            .set(&DataKey::PendingApprovals(approver.clone()), &pending);
    }

    fn add_audit_entry(env: &Env, entry: &MultiSigAuditEntry) {
        let mut audit_trail = Self::get_audit_trail(env, &entry.request_id);
        audit_trail.push_back(entry.clone());
        env.storage()
            .persistent()
            .set(&DataKey::MultiSigAudit(entry.request_id.clone()), &audit_trail);
    }
}
