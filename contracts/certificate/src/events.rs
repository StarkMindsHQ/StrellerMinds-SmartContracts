use crate::types::{CertificateMetadata, Role, NotificationType};
use soroban_sdk::{Address, BytesN, Env, String, Symbol};

/// Contract event emissions
pub struct CertificateEvents;

impl CertificateEvents {
    /// Emits event when contract is initialized
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `admin` - Address of the initial admin
    pub fn emit_contract_initialized(env: &Env, admin: &Address) {
        let topics = (Symbol::new(env, "contract_initialized"),);
        env.events().publish(topics, admin);
    }

    /// Emits event when a role is added
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `user` - Address of the user
    /// * `role` - The role that was added
    pub fn emit_role_added(env: &Env, user: &Address, role: &Role) {
        let topics = (Symbol::new(env, "role_added"), user);
        let data = (role.can_issue, role.can_revoke);
        env.events().publish(topics, data);
    }

    /// Emits event when a role is removed
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `user` - Address of the user
    pub fn emit_role_removed(env: &Env, user: &Address) {
        let topics = (Symbol::new(env, "role_removed"), user);
        env.events().publish(topics, ());
    }

    /// Emits event when a role is updated
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `user` - Address of the user
    /// * `new_role` - The updated role
    pub fn emit_role_updated(env: &Env, user: &Address, new_role: &Role) {
        let topics = (Symbol::new(env, "role_updated"), user);
        let data = (new_role.can_issue, new_role.can_revoke);
        env.events().publish(topics, data);
    }

    /// Emits event when a certificate is minted
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `certificate_id` - Identifier of the certificate
    /// * `metadata` - Certificate metadata
    /// * `student` - Address of the student
    /// * `issuer` - Address of the issuer
    /// * `token_id` - Unique token ID for the certificate
    pub fn emit_certificate_minted(
        env: &Env,
        certificate_id: &BytesN<32>,
        metadata: &CertificateMetadata,
        student: &Address,
        issuer: &Address,
        token_id: &BytesN<32>,
    ) {
        let topics = (Symbol::new(env, "nft_certificate_minted"), certificate_id);
        let data = (
            metadata.clone(),
            student.clone(),
            issuer.clone(),
            token_id.clone(),
        );
        env.events().publish(topics, data);
    }

    /// Emits event when a certificate is revoked
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `certificate_id` - Identifier of the certificate
    /// * `metadata` - Certificate metadata
    /// * `revoker` - Address of the revoker
    /// * `timestamp` - Time of revocation
    pub fn emit_certificate_revoked(
        env: &Env,
        certificate_id: &BytesN<32>,
        metadata: &CertificateMetadata,
        revoker: &Address,
        timestamp: u64,
    ) {
        let topics = (Symbol::new(env, "nft_certificate_revoked"), certificate_id);
        let data = (metadata.clone(), revoker.clone(), timestamp);
        env.events().publish(topics, data);
    }

    /// Emits event when a certificate is transferred
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `certificate_id` - Identifier of the certificate
    /// * `from` - Address of the previous owner
    /// * `to` - Address of the new owner
    #[allow(dead_code)]
    pub fn emit_certificate_transferred(
        env: &Env,
        certificate_id: &BytesN<32>,
        from: &Address,
        to: &Address,
    ) {
        let topics = (Symbol::new(env, "certificate_transferred"), certificate_id);
        let data = (from, to);
        env.events().publish(topics, data);
    }

    /// Emits event when certificate metadata is updated
    ///
    /// # Arguments
    /// * `env` - Reference to the contract environment
    /// * `certificate_id` - Identifier of the certificate
    /// * `updater` - Address of the user who updated metadata
    /// * `old_uri` - Previous metadata URI
    /// * `new_uri` - New metadata URI
    pub fn emit_metadata_updated(
        env: &Env,
        certificate_id: &BytesN<32>,
        updater: &Address,
        old_uri: &String,
        new_uri: &String,
    ) {
        let topics = (Symbol::new(env, "metadata_updated"), certificate_id);
        let data = (
            updater.clone(),
            old_uri.clone(),
            new_uri.clone(),
            env.ledger().timestamp(),
        );
        env.events().publish(topics, data);
    }

    /// Emits event when a renewal request is submitted
    pub fn emit_renewal_requested(
        env: &Env,
        certificate_id: &BytesN<32>,
        requester: &Address,
        requested_extension: u64,
    ) {
        let topics = (Symbol::new(env, "renewal_requested"), certificate_id);
        let data = (requester.clone(), requested_extension, env.ledger().timestamp());
        env.events().publish(topics, data);
    }

    /// Emits event when a renewal request is approved
    pub fn emit_renewal_approved(
        env: &Env,
        certificate_id: &BytesN<32>,
        approver: &Address,
        requester: &Address,
        extension_period: u64,
    ) {
        let topics = (Symbol::new(env, "renewal_approved"), certificate_id);
        let data = (approver.clone(), requester.clone(), extension_period, env.ledger().timestamp());
        env.events().publish(topics, data);
    }

    /// Emits event when a renewal request is rejected
    pub fn emit_renewal_rejected(
        env: &Env,
        certificate_id: &BytesN<32>,
        approver: &Address,
        requester: &Address,
        reason: String,
    ) {
        let topics = (Symbol::new(env, "renewal_rejected"), certificate_id);
        let data = (approver.clone(), requester.clone(), reason, env.ledger().timestamp());
        env.events().publish(topics, data);
    }

    /// Emits event when certificate expiry is extended by admin
    pub fn emit_certificate_extended(
        env: &Env,
        certificate_id: &BytesN<32>,
        admin: &Address,
        owner: &Address,
        extension_period: u64,
        reason: String,
    ) {
        let topics = (Symbol::new(env, "certificate_extended"), certificate_id);
        let data = (admin.clone(), owner.clone(), extension_period, reason, env.ledger().timestamp());
        env.events().publish(topics, data);
    }

    /// Emits event when bulk extension operation is completed
    pub fn emit_bulk_extension_completed(
        env: &Env,
        admin: &Address,
        certificates_updated: u32,
        reason: String,
    ) {
        let topics = (Symbol::new(env, "bulk_extension_completed"),);
        let data = (admin.clone(), certificates_updated, reason, env.ledger().timestamp());
        env.events().publish(topics, data);
    }

    /// Emits event when an expiry notification is created
    pub fn emit_expiry_notification(
        env: &Env,
        certificate_id: &BytesN<32>,
        owner: &Address,
        notification_type: &NotificationType,
        expiry_date: u64,
    ) {
        let topics = (Symbol::new(env, "expiry_notification"), certificate_id);
        let notification_type_str = match notification_type {
            NotificationType::Warning30Days => "warning_30_days",
            NotificationType::Warning7Days => "warning_7_days",
            NotificationType::Warning1Day => "warning_1_day",
            NotificationType::Expired => "expired",
        };
        let data = (owner.clone(), notification_type_str, expiry_date, env.ledger().timestamp());
        env.events().publish(topics, data);
    }

    /// Emits event when certificate status changes to expired
    pub fn emit_certificate_expired(
        env: &Env,
        certificate_id: &BytesN<32>,
        owner: &Address,
        expiry_date: u64,
    ) {
        let topics = (Symbol::new(env, "certificate_expired"), certificate_id);
        let data = (owner.clone(), expiry_date, env.ledger().timestamp());
        env.events().publish(topics, data);
    }

    /// Emits event when certificate is automatically renewed
    pub fn emit_certificate_auto_renewed(
        env: &Env,
        certificate_id: &BytesN<32>,
        owner: &Address,
        new_expiry_date: u64,
        renewal_count: u32,
    ) {
        let topics = (Symbol::new(env, "certificate_auto_renewed"), certificate_id);
        let data = (owner.clone(), new_expiry_date, renewal_count, env.ledger().timestamp());
        env.events().publish(topics, data);
    }

    /// Emits event when notification is acknowledged
    pub fn emit_notification_acknowledged(
        env: &Env,
        certificate_id: &BytesN<32>,
        user: &Address,
        notification_type: &NotificationType,
    ) {
        let topics = (Symbol::new(env, "notification_acknowledged"), certificate_id);
        let notification_type_str = match notification_type {
            NotificationType::Warning30Days => "warning_30_days",
            NotificationType::Warning7Days => "warning_7_days",
            NotificationType::Warning1Day => "warning_1_day",
            NotificationType::Expired => "expired",
        };
        let data = (user.clone(), notification_type_str, env.ledger().timestamp());
        env.events().publish(topics, data);
    }

    /// Emits event when a multi-signature certificate request is created
    pub fn emit_multisig_request_created(
        env: &Env,
        request_id: &BytesN<32>,
        requester: &Address,
        course_id: &String,
    ) {
        let topics = (Symbol::new(env, "multisig_request_created"), request_id);
        let data = (requester.clone(), course_id.clone(), env.ledger().timestamp());
        env.events().publish(topics, data);
    }

    /// Emits event when a multi-signature request receives approval
    pub fn emit_multisig_approval_granted(
        env: &Env,
        request_id: &BytesN<32>,
        approver: &Address,
        current_approvals: u32,
        required_approvals: u32,
    ) {
        let topics = (Symbol::new(env, "multisig_approval_granted"), request_id);
        let data = (approver.clone(), current_approvals, required_approvals, env.ledger().timestamp());
        env.events().publish(topics, data);
    }

    /// Emits event when a multi-signature request is rejected
    pub fn emit_multisig_request_rejected(
        env: &Env,
        request_id: &BytesN<32>,
        rejector: &Address,
        reason: &String,
    ) {
        let topics = (Symbol::new(env, "multisig_request_rejected"), request_id);
        let data = (rejector.clone(), reason.clone(), env.ledger().timestamp());
        env.events().publish(topics, data);
    }

    /// Emits event when a multi-signature request is fully approved
    pub fn emit_multisig_request_approved(
        env: &Env,
        request_id: &BytesN<32>,
        certificate_id: &BytesN<32>,
        final_approvals: u32,
    ) {
        let topics = (Symbol::new(env, "multisig_request_approved"), request_id);
        let data = (certificate_id.clone(), final_approvals, env.ledger().timestamp());
        env.events().publish(topics, data);
    }

    /// Emits event when a multi-signature request expires
    pub fn emit_multisig_request_expired(
        env: &Env,
        request_id: &BytesN<32>,
        certificate_id: &BytesN<32>,
    ) {
        let topics = (Symbol::new(env, "multisig_request_expired"), request_id);
        let data = (certificate_id.clone(), env.ledger().timestamp());
        env.events().publish(topics, data);
    }

    /// Emits event when a certificate is issued via multi-signature approval
    pub fn emit_multisig_certificate_issued(
        env: &Env,
        request_id: &BytesN<32>,
        certificate_id: &BytesN<32>,
        student: &Address,
        approvers: &Vec<Address>,
    ) {
        let topics = (Symbol::new(env, "multisig_certificate_issued"), request_id);
        let data = (certificate_id.clone(), student.clone(), approvers.len() as u32, env.ledger().timestamp());
        env.events().publish(topics, data);
    }

    /// Emits event when multi-signature configuration is updated
    pub fn emit_multisig_config_updated(
        env: &Env,
        course_id: &String,
        admin: &Address,
        required_approvals: u32,
        approvers_count: u32,
    ) {
        let topics = (Symbol::new(env, "multisig_config_updated"), course_id);
        let data = (admin.clone(), required_approvals, approvers_count, env.ledger().timestamp());
        env.events().publish(topics, data);
    }
}
