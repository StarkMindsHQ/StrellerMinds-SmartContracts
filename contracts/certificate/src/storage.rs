use soroban_sdk::{Address, BytesN, Env, Map, String, Vec};

use crate::types::{
    CertDataKey, Certificate, CertificateAnalytics, CertificateBackup, CertificateTemplate,
    ComplianceRecord, MultiSigAuditEntry, MultiSigCertificateRequest, MultiSigConfig,
    RecoveryRequest, RevocationRecord, ShareRecord, TemplateVersion,
};

// ─────────────────────────────────────────────────────────────
// Admin / Initialisation
// ─────────────────────────────────────────────────────────────
pub fn set_admin(env: &Env, admin: &Address) {
    env.storage().instance().set(&CertDataKey::Admin, admin);
}

pub fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&CertDataKey::Admin).unwrap()
}

pub fn is_initialized(env: &Env) -> bool {
    env.storage().instance().has(&CertDataKey::Initialized)
}

pub fn set_initialized(env: &Env) {
    env.storage().instance().set(&CertDataKey::Initialized, &true);
}

// ─────────────────────────────────────────────────────────────
// Multi-Sig Configs
// ─────────────────────────────────────────────────────────────
pub fn set_multisig_config(env: &Env, course_id: &String, config: &MultiSigConfig) {
    env.storage().persistent().set(&CertDataKey::MultiSigConfig(course_id.clone()), config);
}

pub fn get_multisig_config(env: &Env, course_id: &String) -> Option<MultiSigConfig> {
    env.storage().persistent().get(&CertDataKey::MultiSigConfig(course_id.clone()))
}

// ─────────────────────────────────────────────────────────────
// Multi-Sig Requests
// ─────────────────────────────────────────────────────────────
pub fn set_multisig_request(env: &Env, request_id: &BytesN<32>, req: &MultiSigCertificateRequest) {
    env.storage().persistent().set(&CertDataKey::MultiSigRequest(request_id.clone()), req);
}

pub fn get_multisig_request(
    env: &Env,
    request_id: &BytesN<32>,
) -> Option<MultiSigCertificateRequest> {
    env.storage().persistent().get(&CertDataKey::MultiSigRequest(request_id.clone()))
}

pub fn add_pending_request(env: &Env, request_id: &BytesN<32>) {
    let mut pending: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&CertDataKey::PendingRequests)
        .unwrap_or_else(|| Vec::new(env));
    pending.push_back(request_id.clone());
    env.storage().persistent().set(&CertDataKey::PendingRequests, &pending);
}

pub fn get_pending_requests(env: &Env) -> Vec<BytesN<32>> {
    env.storage().persistent().get(&CertDataKey::PendingRequests).unwrap_or_else(|| Vec::new(env))
}

pub fn set_pending_requests(env: &Env, pending: &Vec<BytesN<32>>) {
    env.storage().persistent().set(&CertDataKey::PendingRequests, pending);
}

// ─────────────────────────────────────────────────────────────
// Approver Pending Tracking
// ─────────────────────────────────────────────────────────────
pub fn add_approver_pending(env: &Env, approver: &Address, request_id: &BytesN<32>) {
    let mut ids: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&CertDataKey::ApproverPending(approver.clone()))
        .unwrap_or_else(|| Vec::new(env));
    ids.push_back(request_id.clone());
    env.storage().persistent().set(&CertDataKey::ApproverPending(approver.clone()), &ids);
}

pub fn get_approver_pending(env: &Env, approver: &Address) -> Vec<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&CertDataKey::ApproverPending(approver.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn remove_approver_pending(env: &Env, approver: &Address, request_id: &BytesN<32>) {
    let ids = get_approver_pending(env, approver);
    let mut new_ids: Vec<BytesN<32>> = Vec::new(env);
    for id in ids.iter() {
        if id != *request_id {
            new_ids.push_back(id);
        }
    }
    env.storage().persistent().set(&CertDataKey::ApproverPending(approver.clone()), &new_ids);
}

// ─────────────────────────────────────────────────────────────
// Certificates
// ─────────────────────────────────────────────────────────────
pub fn set_certificate(env: &Env, cert_id: &BytesN<32>, cert: &Certificate) {
    env.storage().persistent().set(&CertDataKey::Certificate(cert_id.clone()), cert);
}

pub fn get_certificate(env: &Env, cert_id: &BytesN<32>) -> Option<Certificate> {
    env.storage().persistent().get(&CertDataKey::Certificate(cert_id.clone()))
}

pub fn add_student_certificate(env: &Env, student: &Address, cert_id: &BytesN<32>) {
    let mut certs: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&CertDataKey::StudentCertificates(student.clone()))
        .unwrap_or_else(|| Vec::new(env));
    certs.push_back(cert_id.clone());
    env.storage().persistent().set(&CertDataKey::StudentCertificates(student.clone()), &certs);
}

pub fn get_student_certificates(env: &Env, student: &Address) -> Vec<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&CertDataKey::StudentCertificates(student.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn has_course_student_certificate(env: &Env, course_id: &String, student: &Address) -> bool {
    env.storage()
        .persistent()
        .has(&CertDataKey::CourseStudentCertificate(course_id.clone(), student.clone()))
}

pub fn set_course_student_certificate(
    env: &Env,
    course_id: &String,
    student: &Address,
    cert_id: &BytesN<32>,
) {
    env.storage()
        .persistent()
        .set(&CertDataKey::CourseStudentCertificate(course_id.clone(), student.clone()), cert_id);
}

/// Batch-appends multiple certificate IDs for multiple students in a single pass.
/// Groups cert IDs by student, then does one read + one write per unique student,
/// reducing storage ops from O(2N) to O(2 * unique_students).
pub fn add_student_certificates_batch(env: &Env, entries: &Vec<(Address, BytesN<32>)>) {
    // Group cert_ids by student using a Map for O(1) lookup
    let mut student_map: Map<Address, Vec<BytesN<32>>> = Map::new(env);

    for (student, cert_id) in entries.iter() {
        let mut ids = student_map.get(student.clone()).unwrap_or_else(|| Vec::new(env));
        ids.push_back(cert_id.clone());
        student_map.set(student.clone(), ids);
    }

    // One read + one write per unique student
    for (student, new_ids) in student_map.iter() {
        let mut existing: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&CertDataKey::StudentCertificates(student.clone()))
            .unwrap_or_else(|| Vec::new(env));
        for id in new_ids.iter() {
            existing.push_back(id);
        }
        env.storage()
            .persistent()
            .set(&CertDataKey::StudentCertificates(student.clone()), &existing);
    }
}

// ─────────────────────────────────────────────────────────────
// Templates
// ─────────────────────────────────────────────────────────────
pub fn set_template(env: &Env, template_id: &String, template: &CertificateTemplate) {
    env.storage().persistent().set(&CertDataKey::Template(template_id.clone()), template);

    // Add to template list
    let mut list: Vec<String> =
        env.storage().persistent().get(&CertDataKey::TemplateList).unwrap_or_else(|| Vec::new(env));
    list.push_back(template_id.clone());
    env.storage().persistent().set(&CertDataKey::TemplateList, &list);
}

pub fn get_template(env: &Env, template_id: &String) -> Option<CertificateTemplate> {
    env.storage().persistent().get(&CertDataKey::Template(template_id.clone()))
}

// ─────────────────────────────────────────────────────────────
// Template Versioning
// ─────────────────────────────────────────────────────────────
pub fn add_template_version(env: &Env, template_id: &String, version: &TemplateVersion) {
    let mut versions: Vec<TemplateVersion> = env
        .storage()
        .persistent()
        .get(&CertDataKey::TemplateVersionHistory(template_id.clone()))
        .unwrap_or_else(|| Vec::new(env));
    versions.push_back(version.clone());
    env.storage()
        .persistent()
        .set(&CertDataKey::TemplateVersionHistory(template_id.clone()), &versions);
}

pub fn get_template_versions(env: &Env, template_id: &String) -> Vec<TemplateVersion> {
    env.storage()
        .persistent()
        .get(&CertDataKey::TemplateVersionHistory(template_id.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn get_template_version(
    env: &Env,
    template_id: &String,
    version: u32,
) -> Option<TemplateVersion> {
    let versions = get_template_versions(env, template_id);
    for v in versions.iter() {
        if v.version == version {
            return Some(v);
        }
    }
    None
}

pub fn set_latest_template_version(env: &Env, template_id: &String, version: u32) {
    env.storage()
        .persistent()
        .set(&CertDataKey::LatestTemplateVersion(template_id.clone()), &version);
}

pub fn get_latest_template_version(env: &Env, template_id: &String) -> u32 {
    env.storage()
        .persistent()
        .get(&CertDataKey::LatestTemplateVersion(template_id.clone()))
        .unwrap_or(0)
}

// ─────────────────────────────────────────────────────────────
// Revocations
// ─────────────────────────────────────────────────────────────
pub fn set_revocation(env: &Env, cert_id: &BytesN<32>, record: &RevocationRecord) {
    env.storage().persistent().set(&CertDataKey::RevocationRecord(cert_id.clone()), record);
}

pub fn get_revocation(env: &Env, cert_id: &BytesN<32>) -> Option<RevocationRecord> {
    env.storage().persistent().get(&CertDataKey::RevocationRecord(cert_id.clone()))
}

// ─────────────────────────────────────────────────────────────
// Analytics
// ─────────────────────────────────────────────────────────────
pub fn get_analytics(env: &Env) -> CertificateAnalytics {
    env.storage().instance().get(&CertDataKey::Analytics).unwrap_or(CertificateAnalytics {
        total_issued: 0,
        total_revoked: 0,
        total_expired: 0,
        total_reissued: 0,
        total_shared: 0,
        total_verified: 0,
        active_certificates: 0,
        pending_requests: 0,
        avg_approval_time: 0,
        last_updated: 0,
    })
}

pub fn set_analytics(env: &Env, analytics: &CertificateAnalytics) {
    env.storage().instance().set(&CertDataKey::Analytics, analytics);
}

// ─────────────────────────────────────────────────────────────
// Compliance
// ─────────────────────────────────────────────────────────────
pub fn set_compliance(env: &Env, cert_id: &BytesN<32>, record: &ComplianceRecord) {
    env.storage().persistent().set(&CertDataKey::ComplianceRecord(cert_id.clone()), record);
}

pub fn get_compliance(env: &Env, cert_id: &BytesN<32>) -> Option<ComplianceRecord> {
    env.storage().persistent().get(&CertDataKey::ComplianceRecord(cert_id.clone()))
}

// ─────────────────────────────────────────────────────────────
// Share Records
// ─────────────────────────────────────────────────────────────
pub fn add_share_record(env: &Env, cert_id: &BytesN<32>, record: &ShareRecord) {
    let mut records: Vec<ShareRecord> = env
        .storage()
        .persistent()
        .get(&CertDataKey::ShareRecords(cert_id.clone()))
        .unwrap_or_else(|| Vec::new(env));
    records.push_back(record.clone());
    env.storage().persistent().set(&CertDataKey::ShareRecords(cert_id.clone()), &records);
}

pub fn get_share_records(env: &Env, cert_id: &BytesN<32>) -> Vec<ShareRecord> {
    env.storage()
        .persistent()
        .get(&CertDataKey::ShareRecords(cert_id.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

// ─────────────────────────────────────────────────────────────
// Audit Trail
// ─────────────────────────────────────────────────────────────
pub fn add_audit_entry(env: &Env, request_id: &BytesN<32>, entry: &MultiSigAuditEntry) {
    let mut entries: Vec<MultiSigAuditEntry> = env
        .storage()
        .persistent()
        .get(&CertDataKey::AuditTrail(request_id.clone()))
        .unwrap_or_else(|| Vec::new(env));
    entries.push_back(entry.clone());
    env.storage().persistent().set(&CertDataKey::AuditTrail(request_id.clone()), &entries);
}

pub fn get_audit_trail(env: &Env, request_id: &BytesN<32>) -> Vec<MultiSigAuditEntry> {
    env.storage()
        .persistent()
        .get(&CertDataKey::AuditTrail(request_id.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

// ─────────────────────────────────────────────────────────────
// Counters (gas-efficient via instance storage)
// ─────────────────────────────────────────────────────────────
pub fn next_request_counter(env: &Env) -> u64 {
    let c: u64 = env.storage().instance().get(&CertDataKey::RequestCounter).unwrap_or(0);
    let next = c + 1;
    env.storage().instance().set(&CertDataKey::RequestCounter, &next);
    next
}

pub fn next_certificate_counter(env: &Env) -> u64 {
    let c: u64 = env.storage().instance().get(&CertDataKey::CertificateCounter).unwrap_or(0);
    let next = c + 1;
    env.storage().instance().set(&CertDataKey::CertificateCounter, &next);
    next
}

// ─────────────────────────────────────────────────────────────
// Certificate Recovery
// ─────────────────────────────────────────────────────────────
pub fn set_certificate_backup(env: &Env, backup_id: &BytesN<32>, backup: &CertificateBackup) {
    env.storage().persistent().set(&CertDataKey::CertificateBackup(backup_id.clone()), backup);
}

pub fn get_certificate_backup(env: &Env, backup_id: &BytesN<32>) -> Option<CertificateBackup> {
    env.storage().persistent().get(&CertDataKey::CertificateBackup(backup_id.clone()))
}

pub fn add_student_backup(env: &Env, student: &Address, backup_id: &BytesN<32>) {
    let mut backups: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&CertDataKey::StudentBackups(student.clone()))
        .unwrap_or_else(|| Vec::new(env));
    backups.push_back(backup_id.clone());
    env.storage().persistent().set(&CertDataKey::StudentBackups(student.clone()), &backups);
}

pub fn get_student_backups(env: &Env, student: &Address) -> Vec<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&CertDataKey::StudentBackups(student.clone()))
        .unwrap_or_else(|| Vec::new(env))
}

pub fn set_recovery_request(env: &Env, request_id: &BytesN<32>, req: &RecoveryRequest) {
    env.storage().persistent().set(&CertDataKey::RecoveryRequest(request_id.clone()), req);
}

pub fn get_recovery_request(env: &Env, request_id: &BytesN<32>) -> Option<RecoveryRequest> {
    env.storage().persistent().get(&CertDataKey::RecoveryRequest(request_id.clone()))
}

pub fn add_pending_recovery_request(env: &Env, request_id: &BytesN<32>) {
    let mut pending: Vec<BytesN<32>> = env
        .storage()
        .persistent()
        .get(&CertDataKey::PendingRecoveryRequests)
        .unwrap_or_else(|| Vec::new(env));
    pending.push_back(request_id.clone());
    env.storage().persistent().set(&CertDataKey::PendingRecoveryRequests, &pending);
}

pub fn get_pending_recovery_requests(env: &Env) -> Vec<BytesN<32>> {
    env.storage()
        .persistent()
        .get(&CertDataKey::PendingRecoveryRequests)
        .unwrap_or_else(|| Vec::new(env))
}

pub fn set_pending_recovery_requests(env: &Env, pending: &Vec<BytesN<32>>) {
    env.storage().persistent().set(&CertDataKey::PendingRecoveryRequests, pending);
}
