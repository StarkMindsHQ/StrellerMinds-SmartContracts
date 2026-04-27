use shared::emit_certification_event;
use shared::event_schema::{
    BatchCompletedEvent, CertificateReissuedEvent, CertificateSharedEvent,
    CertificateVerifiedEvent, CertificationEventData, CertificationIssuedEvent,
    CertificationRevokedEvent, ComplianceCheckedEvent, MultisigApprovalGrantedEvent,
    MultisigConfigUpdatedEvent, MultisigRequestApprovedEvent, MultisigRequestCreatedEvent,
    MultisigRequestRejectedEvent, TemplateCreatedEvent,
};
use soroban_sdk::{symbol_short, Address, BytesN, Env, String};

/// Emit when a multi-sig certificate request is created.
pub fn emit_multisig_request_created(
    env: &Env,
    request_id: &BytesN<32>,
    requester: &Address,
    course_id: &String,
) {
    emit_certification_event!(
        env,
        symbol_short!("cert"),
        requester.clone(),
        CertificationEventData::MultisigRequestCreated(MultisigRequestCreatedEvent {
            request_id: request_id.clone(),
            requester: requester.clone(),
            course_id: course_id.clone(),
        })
    );
}

/// Emit when an approval is granted.
pub fn emit_multisig_approval_granted(
    env: &Env,
    request_id: &BytesN<32>,
    approver: &Address,
    current: u32,
    required: u32,
) {
    emit_certification_event!(
        env,
        symbol_short!("cert"),
        approver.clone(),
        CertificationEventData::MultisigApprovalGranted(MultisigApprovalGrantedEvent {
            request_id: request_id.clone(),
            approver: approver.clone(),
            current,
            required,
        })
    );
}

/// Emit when a request is rejected by an approver.
pub fn emit_multisig_request_rejected(env: &Env, request_id: &BytesN<32>, approver: &Address) {
    emit_certification_event!(
        env,
        symbol_short!("cert"),
        approver.clone(),
        CertificationEventData::MultisigRequestRejected(MultisigRequestRejectedEvent {
            request_id: request_id.clone(),
            approver: approver.clone(),
        })
    );
}

/// Emit when a request reaches full approval.
pub fn emit_multisig_request_approved(env: &Env, request_id: &BytesN<32>, approver: &Address) {
    emit_certification_event!(
        env,
        symbol_short!("cert"),
        approver.clone(),
        CertificationEventData::MultisigRequestApproved(MultisigRequestApprovedEvent {
            request_id: request_id.clone(),
        })
    );
}

/// Emit when a certificate is issued via multi-sig.
pub fn emit_certificate_issued(
    env: &Env,
    certificate_id: &BytesN<32>,
    student: &Address,
    course_id: &String,
) {
    emit_certification_event!(
        env,
        symbol_short!("cert"),
        student.clone(),
        CertificationEventData::CertificateIssued(CertificationIssuedEvent {
            certificate_id: certificate_id.clone(),
            student: student.clone(),
            course_id: course_id.clone(),
        })
    );
}

/// Emit when a certificate is revoked.
pub fn emit_certificate_revoked(env: &Env, certificate_id: &BytesN<32>, revoked_by: &Address) {
    emit_certification_event!(
        env,
        symbol_short!("cert"),
        revoked_by.clone(),
        CertificationEventData::CertificateRevoked(CertificationRevokedEvent {
            certificate_id: certificate_id.clone(),
            admin: revoked_by.clone(),
        })
    );
}

/// Emit when a certificate is reissued.
pub fn emit_certificate_reissued(
    env: &Env,
    old_id: &BytesN<32>,
    new_id: &BytesN<32>,
    student: &Address,
) {
    emit_certification_event!(
        env,
        symbol_short!("cert"),
        student.clone(),
        CertificationEventData::CertificateReissued(CertificateReissuedEvent {
            old_id: old_id.clone(),
            new_id: new_id.clone(),
            student: student.clone(),
        })
    );
}

/// Emit when a multi-sig config is updated.
pub fn emit_multisig_config_updated(env: &Env, course_id: &String, admin: &Address) {
    emit_certification_event!(
        env,
        symbol_short!("cert"),
        admin.clone(),
        CertificationEventData::MultisigConfigUpdated(MultisigConfigUpdatedEvent {
            course_id: course_id.clone(),
            admin: admin.clone(),
        })
    );
}

/// Emit when a batch operation completes.
pub fn emit_batch_completed(env: &Env, admin: &Address, total: u32, succeeded: u32, failed: u32) {
    emit_certification_event!(
        env,
        symbol_short!("cert"),
        admin.clone(),
        CertificationEventData::BatchCompleted(BatchCompletedEvent { total, succeeded, failed })
    );
}

/// Emit when a certificate is shared.
pub fn emit_certificate_shared(
    env: &Env,
    certificate_id: &BytesN<32>,
    shared_by: &Address,
    platform: &String,
) {
    emit_certification_event!(
        env,
        symbol_short!("cert"),
        shared_by.clone(),
        CertificationEventData::CertificateShared(CertificateSharedEvent {
            certificate_id: certificate_id.clone(),
            platform: platform.clone(),
        })
    );
}

pub fn emit_compliance_checked(
    env: &Env,
    verifier: &Address,
    certificate_id: &BytesN<32>,
    is_compliant: bool,
) {
    emit_certification_event!(
        env,
        symbol_short!("cert"),
        verifier.clone(),
        CertificationEventData::ComplianceChecked(ComplianceCheckedEvent {
            certificate_id: certificate_id.clone(),
            is_compliant,
        })
    );
}

/// Emit when a compliance violation is detected.
pub fn emit_compliance_violation(
    env: &Env,
    certificate_id: &BytesN<32>,
    standard: &str,
    details: &str,
) {
    emit_certification_event!(
        env,
        symbol_short!("cert"),
        env.current_contract_address(),
        CertificationEventData::ComplianceViolation(ComplianceViolationEvent {
            certificate_id: certificate_id.clone(),
            standard: String::from_str(env, standard),
            violation_details: String::from_str(env, details),
        })
    );
}

/// Emit when a certificate template is created.
pub fn emit_template_created(env: &Env, template_id: &String, created_by: &Address) {
    emit_certification_event!(
        env,
        symbol_short!("cert"),
        created_by.clone(),
        CertificationEventData::TemplateCreated(TemplateCreatedEvent {
            template_id: template_id.clone(),
        })
    );
}

/// Emit when a certificate is verified.
pub fn emit_certificate_verified(
    env: &Env,
    verifier: &Address,
    certificate_id: &BytesN<32>,
    is_authentic: bool,
) {
    emit_certification_event!(
        env,
        symbol_short!("cert"),
        verifier.clone(),
        CertificationEventData::CertificateVerified(CertificateVerifiedEvent {
            certificate_id: certificate_id.clone(),
            is_valid: is_authentic,
        })
    );
}
