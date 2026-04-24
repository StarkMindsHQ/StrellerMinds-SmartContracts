//! End-to-End Test Suite for Certificate Workflow
//!
//! Comprehensive E2E tests covering the complete certificate lifecycle:
//! - Certificate creation (batch & template-based)
//! - Multi-sig approval flow
//! - Issuance and storage
//! - Verification (validity & authenticity)
//! - Revocation & reissuance
//! - Compliance checks
//! - Sharing & social verification
//! - Expired request cleanup
//! - Analytics & audit trail
//! - Performance baselines

#![cfg(test)]

use certificate::{
    types::{
        AuditAction, BatchResult, CertificateAnalytics, CertificatePriority, CertificateStatus,
        CertificateTemplate, ComplianceStandard, FieldType, MintCertificateParams, MultiSigConfig,
        MultiSigRequestStatus, RevocationRecord, ShareRecord, TemplateField,
    },
    CertificateContract, CertificateContractClient,
};
use soroban_sdk::{
    testutils::{Address as _, BytesN as _, Ledger},
    Address, BytesN, Env, String, Vec,
};
use std::time::Instant;

// ─────────────────────────────────────────────────────────────
// Test Environment Setup
// ─────────────────────────────────────────────────────────────

struct CertificateTestEnv {
    env: Env,
    client: CertificateContractClient<'static>,
    admin: Address,
}

impl CertificateTestEnv {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        env.ledger().set_timestamp(1_700_000_000);

        let contract_id = env.register(CertificateContract, ());
        let client = CertificateContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);

        client.initialize(&admin);

        Self { env, client, admin }
    }

    fn generate_address(&self) -> Address {
        Address::generate(&self.env)
    }

    fn make_course_id(&self, id: &str) -> String {
        String::from_str(&self.env, id)
    }

    fn make_cert_params(
        &self,
        cert_id_bytes: &[u8; 32],
        course_id: &str,
        student: &Address,
    ) -> MintCertificateParams {
        MintCertificateParams {
            certificate_id: BytesN::from_array(&self.env, cert_id_bytes),
            course_id: self.make_course_id(course_id),
            student: student.clone(),
            title: String::from_str(&self.env, "E2E Test Certificate"),
            description: String::from_str(&self.env, "Comprehensive workflow test certificate"),
            metadata_uri: String::from_str(&self.env, "https://strellerminds.io/cert/metadata"),
            expiry_date: self.env.ledger().timestamp() + 31_536_000, // 1 year
        }
    }

    fn make_multisig_config(
        &self,
        course_id: &str,
        approvers: &[Address],
        required: u32,
        auto_execute: bool,
    ) -> MultiSigConfig {
        let mut appr_vec: Vec<Address> = Vec::new(&self.env);
        for a in approvers {
            appr_vec.push_back(a.clone());
        }
        MultiSigConfig {
            course_id: self.make_course_id(course_id),
            required_approvals: required,
            authorized_approvers: appr_vec,
            timeout_duration: 604_800, // 7 days
            priority: CertificatePriority::Enterprise,
            auto_execute,
        }
    }

    fn advance_time(&self, seconds: u64) {
        self.env.ledger().set_timestamp(self.env.ledger().timestamp() + seconds);
    }
}

// ─────────────────────────────────────────────────────────────
// 1. Initialization & Configuration
// ─────────────────────────────────────────────────────────────

#[test]
fn test_certificate_contract_initialization() {
    let test_env = CertificateTestEnv::new();

    // Verify contract is initialized by checking analytics exist
    let analytics = test_env.client.get_analytics();
    assert_eq!(analytics.total_issued, 0);
    assert_eq!(analytics.active_certificates, 0);
}

#[test]
fn test_multisig_configuration_workflow() {
    let test_env = CertificateTestEnv::new();
    let approver1 = test_env.generate_address();
    let approver2 = test_env.generate_address();

    let config = test_env.make_multisig_config("CS101", &[approver1, approver2], 2, true);

    test_env.client.configure_multisig(&test_env.admin, &config);

    let retrieved = test_env.client.get_multisig_config(&test_env.make_course_id("CS101"));
    assert!(retrieved.is_some());

    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.required_approvals, 2);
    assert_eq!(retrieved.authorized_approvers.len(), 2);
    assert!(retrieved.auto_execute);
}

// ─────────────────────────────────────────────────────────────
// 2. Multi-Sig Approval Flow (Auto-Execute)
// ─────────────────────────────────────────────────────────────

#[test]
fn test_full_multisig_auto_execute_workflow() {
    let test_env = CertificateTestEnv::new();
    let approver1 = test_env.generate_address();
    let approver2 = test_env.generate_address();
    let student = test_env.generate_address();

    // Step 1: Configure multi-sig with auto-execute
    let config =
        test_env.make_multisig_config("CS101", &[approver1.clone(), approver2.clone()], 2, true);
    test_env.client.configure_multisig(&test_env.admin, &config);

    // Step 2: Create certificate request
    let cert_id_bytes = [1u8; 32];
    let params = test_env.make_cert_params(&cert_id_bytes, "CS101", &student);
    let reason = String::from_str(&test_env.env, "Requesting certificate for course completion");

    let request_id = test_env.client.create_multisig_request(&student, &params, &reason);

    // Verify request is pending
    let request = test_env.client.get_multisig_request(&request_id).unwrap();
    assert_eq!(request.status, MultiSigRequestStatus::Pending);
    assert_eq!(request.current_approvals, 0);

    // Step 3: First approval
    let comments1 = String::from_str(&test_env.env, "Looks good");
    test_env.client.process_multisig_approval(&approver1, &request_id, &true, &comments1, &None);

    let request = test_env.client.get_multisig_request(&request_id).unwrap();
    assert_eq!(request.current_approvals, 1);
    assert_eq!(request.status, MultiSigRequestStatus::Pending);

    // Step 4: Second approval triggers auto-execution
    let comments2 = String::from_str(&test_env.env, "Approved");
    test_env.client.process_multisig_approval(&approver2, &request_id, &true, &comments2, &None);

    let request = test_env.client.get_multisig_request(&request_id).unwrap();
    assert_eq!(request.status, MultiSigRequestStatus::Executed);

    // Step 5: Verify certificate was issued
    let cert = test_env.client.get_certificate(&params.certificate_id).unwrap();
    assert_eq!(cert.student, student);
    assert_eq!(cert.status, CertificateStatus::Active);
    assert!(cert.blockchain_anchor.is_some());

    // Step 6: Verify analytics updated
    let analytics = test_env.client.get_analytics();
    assert_eq!(analytics.total_issued, 1);
    assert_eq!(analytics.active_certificates, 1);
    assert_eq!(analytics.pending_requests, 0);
}

// ─────────────────────────────────────────────────────────────
// 3. Multi-Sig Manual Execution Flow
// ─────────────────────────────────────────────────────────────

#[test]
fn test_multisig_manual_execution_workflow() {
    let test_env = CertificateTestEnv::new();
    let approver1 = test_env.generate_address();
    let approver2 = test_env.generate_address();
    let student = test_env.generate_address();
    let executor = test_env.generate_address();

    // Configure with auto_execute = false
    let config =
        test_env.make_multisig_config("CS202", &[approver1.clone(), approver2.clone()], 2, false);
    test_env.client.configure_multisig(&test_env.admin, &config);

    let cert_id_bytes = [2u8; 32];
    let params = test_env.make_cert_params(&cert_id_bytes, "CS202", &student);
    let reason = String::from_str(&test_env.env, "Manual execution test");

    let request_id = test_env.client.create_multisig_request(&student, &params, &reason);

    // Both approvers approve
    test_env.client.process_multisig_approval(
        &approver1,
        &request_id,
        &true,
        &String::from_str(&test_env.env, "Approve"),
        &None,
    );
    test_env.client.process_multisig_approval(
        &approver2,
        &request_id,
        &true,
        &String::from_str(&test_env.env, "Approve"),
        &None,
    );

    // Request should be Approved but not Executed
    let request = test_env.client.get_multisig_request(&request_id).unwrap();
    assert_eq!(request.status, MultiSigRequestStatus::Approved);

    // Manually execute
    test_env.client.execute_multisig_request(&executor, &request_id);

    let request = test_env.client.get_multisig_request(&request_id).unwrap();
    assert_eq!(request.status, MultiSigRequestStatus::Executed);

    let cert = test_env.client.get_certificate(&params.certificate_id).unwrap();
    assert_eq!(cert.status, CertificateStatus::Active);
}

// ─────────────────────────────────────────────────────────────
// 4. Batch Certificate Issuance
// ─────────────────────────────────────────────────────────────

#[test]
fn test_batch_certificate_issuance() {
    let test_env = CertificateTestEnv::new();

    let mut params_list: Vec<MintCertificateParams> = Vec::new(&test_env.env);
    let mut expected_ids: Vec<BytesN<32>> = Vec::new(&test_env.env);

    for i in 0u8..5 {
        let student = test_env.generate_address();
        let cert_id_bytes = [i; 32];
        let params = test_env.make_cert_params(&cert_id_bytes, "BATCH101", &student);
        expected_ids.push_back(params.certificate_id.clone());
        params_list.push_back(params);
    }

    let result: BatchResult =
        test_env.client.batch_issue_certificates(&test_env.admin, &params_list);

    assert_eq!(result.total, 5);
    assert_eq!(result.succeeded, 5);
    assert_eq!(result.failed, 0);
    assert_eq!(result.certificate_ids.len(), 5);

    // Verify all certificates exist
    for i in 0..5 {
        let cert_id = result.certificate_ids.get(i).unwrap();
        let cert = test_env.client.get_certificate(&cert_id).unwrap();
        assert_eq!(cert.status, CertificateStatus::Active);
        assert!(cert.blockchain_anchor.is_some());
    }

    // Verify analytics
    let analytics = test_env.client.get_analytics();
    assert_eq!(analytics.total_issued, 5);
    assert_eq!(analytics.active_certificates, 5);
}

#[test]
fn test_batch_issuance_skips_duplicates() {
    let test_env = CertificateTestEnv::new();
    let student = test_env.generate_address();

    // Issue first certificate
    let cert_id_bytes = [10u8; 32];
    let params = test_env.make_cert_params(&cert_id_bytes, "DUP101", &student);
    let mut first_batch: Vec<MintCertificateParams> = Vec::new(&test_env.env);
    first_batch.push_back(params.clone());
    test_env.client.batch_issue_certificates(&test_env.admin, &first_batch);

    // Try to issue same certificate again in a batch with a new one
    let student2 = test_env.generate_address();
    let cert_id_bytes2 = [11u8; 32];
    let params2 = test_env.make_cert_params(&cert_id_bytes2, "DUP101", &student2);
    let mut second_batch: Vec<MintCertificateParams> = Vec::new(&test_env.env);
    second_batch.push_back(params); // duplicate
    second_batch.push_back(params2); // new

    let result = test_env.client.batch_issue_certificates(&test_env.admin, &second_batch);

    assert_eq!(result.total, 2);
    assert_eq!(result.succeeded, 1);
    assert_eq!(result.failed, 1);
}

// ─────────────────────────────────────────────────────────────
// 5. Certificate Verification
// ─────────────────────────────────────────────────────────────

#[test]
fn test_certificate_verification_workflow() {
    let test_env = CertificateTestEnv::new();
    let student = test_env.generate_address();

    let cert_id_bytes = [3u8; 32];
    let params = test_env.make_cert_params(&cert_id_bytes, "VERIFY101", &student);
    let mut batch: Vec<MintCertificateParams> = Vec::new(&test_env.env);
    batch.push_back(params.clone());
    test_env.client.batch_issue_certificates(&test_env.admin, &batch);

    // verify_certificate should return true for active, unexpired cert with anchor
    let is_valid = test_env.client.verify_certificate(&params.certificate_id);
    assert!(is_valid);

    // verify_authenticity should also return true
    let is_authentic = test_env.client.verify_authenticity(&params.certificate_id);
    assert!(is_authentic);

    // Analytics should track verification
    let analytics = test_env.client.get_analytics();
    assert!(analytics.total_verified >= 2);
}

#[test]
fn test_certificate_verification_expired() {
    let test_env = CertificateTestEnv::new();
    let student = test_env.generate_address();

    // Create certificate that expires immediately
    let mut params = test_env.make_cert_params(&[4u8; 32], "EXP101", &student);
    params.expiry_date = test_env.env.ledger().timestamp() + 100;

    let mut batch: Vec<MintCertificateParams> = Vec::new(&test_env.env);
    batch.push_back(params.clone());
    test_env.client.batch_issue_certificates(&test_env.admin, &batch);

    // Advance time past expiry
    test_env.advance_time(200);

    // Should return false because expired
    let is_valid = test_env.client.verify_certificate(&params.certificate_id);
    assert!(!is_valid);

    // Authenticity should still be true (not revoked, has anchor)
    let is_authentic = test_env.client.verify_authenticity(&params.certificate_id);
    assert!(is_authentic);
}

// ─────────────────────────────────────────────────────────────
// 6. Certificate Revocation & Reissuance
// ─────────────────────────────────────────────────────────────

#[test]
fn test_certificate_revocation_workflow() {
    let test_env = CertificateTestEnv::new();
    let student = test_env.generate_address();

    let cert_id_bytes = [5u8; 32];
    let params = test_env.make_cert_params(&cert_id_bytes, "REVOKE101", &student);
    let mut batch: Vec<MintCertificateParams> = Vec::new(&test_env.env);
    batch.push_back(params.clone());
    test_env.client.batch_issue_certificates(&test_env.admin, &batch);

    let reason = String::from_str(&test_env.env, "Academic integrity violation");
    test_env.client.revoke_certificate(
        &test_env.admin,
        &params.certificate_id,
        &reason,
        &true, // reissuance_eligible
    );

    // Certificate should be revoked
    let cert = test_env.client.get_certificate(&params.certificate_id).unwrap();
    assert_eq!(cert.status, CertificateStatus::Revoked);

    // Verification should fail
    let is_valid = test_env.client.verify_certificate(&params.certificate_id);
    assert!(!is_valid);

    // Authenticity should fail because revoked
    let is_authentic = test_env.client.verify_authenticity(&params.certificate_id);
    assert!(!is_authentic);

    // Analytics should reflect revocation
    let analytics = test_env.client.get_analytics();
    assert_eq!(analytics.total_revoked, 1);
    assert_eq!(analytics.active_certificates, 0);
}

#[test]
fn test_certificate_reissuance_workflow() {
    let test_env = CertificateTestEnv::new();
    let student = test_env.generate_address();

    // Issue original certificate
    let old_cert_id_bytes = [6u8; 32];
    let old_params = test_env.make_cert_params(&old_cert_id_bytes, "REISSUE101", &student);
    let mut batch: Vec<MintCertificateParams> = Vec::new(&test_env.env);
    batch.push_back(old_params.clone());
    test_env.client.batch_issue_certificates(&test_env.admin, &batch);

    // Revoke with reissuance eligibility
    test_env.client.revoke_certificate(
        &test_env.admin,
        &old_params.certificate_id,
        &String::from_str(&test_env.env, "Correction needed"),
        &true,
    );

    // Reissue with new certificate ID
    let new_cert_id_bytes = [7u8; 32];
    let new_params = test_env.make_cert_params(&new_cert_id_bytes, "REISSUE101", &student);

    let new_cert_id = test_env.client.reissue_certificate(
        &test_env.admin,
        &old_params.certificate_id,
        &new_params,
    );

    assert_eq!(new_cert_id, new_params.certificate_id);

    // Old certificate should be marked Reissued
    let old_cert = test_env.client.get_certificate(&old_params.certificate_id).unwrap();
    assert_eq!(old_cert.status, CertificateStatus::Reissued);

    // New certificate should be Active with incremented version
    let new_cert = test_env.client.get_certificate(&new_cert_id).unwrap();
    assert_eq!(new_cert.status, CertificateStatus::Active);
    assert_eq!(new_cert.version, old_cert.version + 1);

    let analytics = test_env.client.get_analytics();
    assert_eq!(analytics.total_reissued, 1);
    assert_eq!(analytics.active_certificates, 1);
}

// ─────────────────────────────────────────────────────────────
// 7. Certificate Template System
// ─────────────────────────────────────────────────────────────

#[test]
fn test_template_creation_and_issuance() {
    let test_env = CertificateTestEnv::new();
    let student = test_env.generate_address();

    // Create template
    let template_id = String::from_str(&test_env.env, "template-001");
    let template_name = String::from_str(&test_env.env, "Course Completion Template");
    let template_desc = String::from_str(&test_env.env, "Standard course completion certificate");

    let mut fields: Vec<TemplateField> = Vec::new(&test_env.env);
    fields.push_back(TemplateField {
        field_name: String::from_str(&test_env.env, "grade"),
        field_type: FieldType::Text,
        is_required: true,
        default_value: None,
    });
    fields.push_back(TemplateField {
        field_name: String::from_str(&test_env.env, "completion_date"),
        field_type: FieldType::Date,
        is_required: true,
        default_value: None,
    });

    test_env.client.create_template(
        &test_env.admin,
        &template_id,
        &template_name,
        &template_desc,
        &fields,
    );

    // Retrieve template
    let template = test_env.client.get_template(&template_id).unwrap();
    assert_eq!(template.name, template_name);
    assert!(template.is_active);

    // Issue certificate using template
    let cert_id_bytes = [8u8; 32];
    let params = test_env.make_cert_params(&cert_id_bytes, "TMPL101", &student);
    let mut field_values: Vec<String> = Vec::new(&test_env.env);
    field_values.push_back(String::from_str(&test_env.env, "A+"));
    field_values.push_back(String::from_str(&test_env.env, "2024-01-15"));

    let issued_id =
        test_env.client.issue_with_template(&test_env.admin, &template_id, &params, &field_values);

    let cert = test_env.client.get_certificate(&issued_id).unwrap();
    assert_eq!(cert.status, CertificateStatus::Active);
    assert_eq!(cert.template_id, Some(template_id));
}

// ─────────────────────────────────────────────────────────────
// 8. Certificate Sharing
// ─────────────────────────────────────────────────────────────

#[test]
fn test_certificate_sharing_workflow() {
    let test_env = CertificateTestEnv::new();
    let student = test_env.generate_address();

    let cert_id_bytes = [9u8; 32];
    let params = test_env.make_cert_params(&cert_id_bytes, "SHARE101", &student);
    let mut batch: Vec<MintCertificateParams> = Vec::new(&test_env.env);
    batch.push_back(params.clone());
    test_env.client.batch_issue_certificates(&test_env.admin, &batch);

    test_env.client.share_certificate(
        &student,
        &params.certificate_id,
        &String::from_str(&test_env.env, "LinkedIn"),
        &String::from_str(&test_env.env, "https://verify.strellerminds.io/cert/123"),
    );

    let share_records = test_env.client.get_share_records(&params.certificate_id);
    assert_eq!(share_records.len(), 1);

    let record = share_records.get(0).unwrap();
    assert_eq!(record.platform, String::from_str(&test_env.env, "LinkedIn"));
    assert_eq!(record.shared_by, student);

    let analytics = test_env.client.get_analytics();
    assert_eq!(analytics.total_shared, 1);
}

// ─────────────────────────────────────────────────────────────
// 9. Compliance Verification
// ─────────────────────────────────────────────────────────────

#[test]
fn test_compliance_verification_workflow() {
    let test_env = CertificateTestEnv::new();
    let student = test_env.generate_address();
    let verifier = test_env.generate_address();

    let cert_id_bytes = [12u8; 32];
    let params = test_env.make_cert_params(&cert_id_bytes, "COMP101", &student);
    let mut batch: Vec<MintCertificateParams> = Vec::new(&test_env.env);
    batch.push_back(params.clone());
    test_env.client.batch_issue_certificates(&test_env.admin, &batch);

    let is_compliant = test_env.client.verify_compliance(
        &verifier,
        &params.certificate_id,
        &ComplianceStandard::Iso17024,
        &String::from_str(&test_env.env, "Verified against ISO 17024 standard"),
    );

    assert!(is_compliant);

    let record = test_env.client.get_compliance_record(&params.certificate_id).unwrap();
    assert_eq!(record.standard, ComplianceStandard::Iso17024);
    assert!(record.is_compliant);
    assert_eq!(record.verified_by, verifier);
}

// ─────────────────────────────────────────────────────────────
// 10. Expired Request Cleanup
// ─────────────────────────────────────────────────────────────

#[test]
fn test_cleanup_expired_multisig_requests() {
    let test_env = CertificateTestEnv::new();
    let approver1 = test_env.generate_address();
    let student = test_env.generate_address();

    // Configure multi-sig with short timeout
    let mut config = test_env.make_multisig_config("EXP_REQ", &[approver1.clone()], 1, false);
    config.timeout_duration = 3_600; // 1 hour
    test_env.client.configure_multisig(&test_env.admin, &config);

    let params = test_env.make_cert_params(&[13u8; 32], "EXP_REQ", &student);
    let reason = String::from_str(&test_env.env, "Will expire");
    let request_id = test_env.client.create_multisig_request(&student, &params, &reason);

    // Advance time past expiry
    test_env.advance_time(3_601);

    // Cleanup should mark the request as expired
    let cleaned = test_env.client.cleanup_expired_requests();
    assert_eq!(cleaned, 1);

    let request = test_env.client.get_multisig_request(&request_id).unwrap();
    assert_eq!(request.status, MultiSigRequestStatus::Expired);
}

// ─────────────────────────────────────────────────────────────
// 11. Audit Trail
// ─────────────────────────────────────────────────────────────

#[test]
fn test_multisig_audit_trail() {
    let test_env = CertificateTestEnv::new();
    let approver1 = test_env.generate_address();
    let approver2 = test_env.generate_address();
    let student = test_env.generate_address();

    let config =
        test_env.make_multisig_config("AUDIT101", &[approver1.clone(), approver2.clone()], 2, true);
    test_env.client.configure_multisig(&test_env.admin, &config);

    let params = test_env.make_cert_params(&[14u8; 32], "AUDIT101", &student);
    let request_id = test_env.client.create_multisig_request(
        &student,
        &params,
        &String::from_str(&test_env.env, "Audit test"),
    );

    test_env.client.process_multisig_approval(
        &approver1,
        &request_id,
        &true,
        &String::from_str(&test_env.env, "First approval"),
        &None,
    );
    test_env.client.process_multisig_approval(
        &approver2,
        &request_id,
        &true,
        &String::from_str(&test_env.env, "Second approval"),
        &None,
    );

    let audit_trail = test_env.client.get_multisig_audit_trail(&request_id);
    assert!(audit_trail.len() >= 3); // Created + 2 approvals

    // Verify at least one approval entry exists
    let has_approval = audit_trail.iter().any(|entry| entry.action == AuditAction::ApprovalGranted);
    assert!(has_approval);
}

// ─────────────────────────────────────────────────────────────
// 12. Student Certificate Queries
// ─────────────────────────────────────────────────────────────

#[test]
fn test_get_student_certificates() {
    let test_env = CertificateTestEnv::new();
    let student = test_env.generate_address();

    for i in 0u8..3 {
        let cert_id_bytes = [20 + i; 32];
        let params = test_env.make_cert_params(&cert_id_bytes, &format!("QUERY{i}"), &student);
        let mut batch: Vec<MintCertificateParams> = Vec::new(&test_env.env);
        batch.push_back(params);
        test_env.client.batch_issue_certificates(&test_env.admin, &batch);
    }

    let student_certs = test_env.client.get_student_certificates(&student);
    assert_eq!(student_certs.len(), 3);
}

// ─────────────────────────────────────────────────────────────
// 13. Error Scenarios
// ─────────────────────────────────────────────────────────────

#[test]
fn test_unauthorized_revocation_fails() {
    let test_env = CertificateTestEnv::new();
    let student = test_env.generate_address();
    let unauthorized = test_env.generate_address();

    let params = test_env.make_cert_params(&[15u8; 32], "UNAUTH101", &student);
    let mut batch: Vec<MintCertificateParams> = Vec::new(&test_env.env);
    batch.push_back(params.clone());
    test_env.client.batch_issue_certificates(&test_env.admin, &batch);

    let result = test_env.client.try_revoke_certificate(
        &unauthorized,
        &params.certificate_id,
        &String::from_str(&test_env.env, "Unauthorized"),
        &false,
    );
    assert!(result.is_err());
}

#[test]
fn test_double_revocation_fails() {
    let test_env = CertificateTestEnv::new();
    let student = test_env.generate_address();

    let params = test_env.make_cert_params(&[16u8; 32], "DUPREV101", &student);
    let mut batch: Vec<MintCertificateParams> = Vec::new(&test_env.env);
    batch.push_back(params.clone());
    test_env.client.batch_issue_certificates(&test_env.admin, &batch);

    test_env.client.revoke_certificate(
        &test_env.admin,
        &params.certificate_id,
        &String::from_str(&test_env.env, "First revoke"),
        &false,
    );

    let result = test_env.client.try_revoke_certificate(
        &test_env.admin,
        &params.certificate_id,
        &String::from_str(&test_env.env, "Second revoke"),
        &false,
    );
    assert!(result.is_err());
}

#[test]
fn test_reissue_non_eligible_certificate_fails() {
    let test_env = CertificateTestEnv::new();
    let student = test_env.generate_address();

    let old_params = test_env.make_cert_params(&[17u8; 32], "NO_REISSUE", &student);
    let mut batch: Vec<MintCertificateParams> = Vec::new(&test_env.env);
    batch.push_back(old_params.clone());
    test_env.client.batch_issue_certificates(&test_env.admin, &batch);

    // Revoke WITHOUT reissuance eligibility
    test_env.client.revoke_certificate(
        &test_env.admin,
        &old_params.certificate_id,
        &String::from_str(&test_env.env, "Not eligible"),
        &false,
    );

    let new_params = test_env.make_cert_params(&[18u8; 32], "NO_REISSUE", &student);
    let result = test_env.client.try_reissue_certificate(
        &test_env.admin,
        &old_params.certificate_id,
        &new_params,
    );
    assert!(result.is_err());
}

#[test]
fn test_approver_not_authorized_fails() {
    let test_env = CertificateTestEnv::new();
    let approver1 = test_env.generate_address();
    let unauthorized_approver = test_env.generate_address();
    let student = test_env.generate_address();

    let config = test_env.make_multisig_config("UNAUTH_APP", &[approver1.clone()], 1, true);
    test_env.client.configure_multisig(&test_env.admin, &config);

    let params = test_env.make_cert_params(&[19u8; 32], "UNAUTH_APP", &student);
    let request_id = test_env.client.create_multisig_request(
        &student,
        &params,
        &String::from_str(&test_env.env, "Test"),
    );

    let result = test_env.client.try_process_multisig_approval(
        &unauthorized_approver,
        &request_id,
        &true,
        &String::from_str(&test_env.env, "Not authorized"),
        &None,
    );
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────
// 14. Performance Baselines
// ─────────────────────────────────────────────────────────────

#[test]
fn test_performance_batch_issuance_baseline() {
    let test_env = CertificateTestEnv::new();
    let start = Instant::now();

    let mut params_list: Vec<MintCertificateParams> = Vec::new(&test_env.env);
    for i in 0u8..25 {
        let student = test_env.generate_address();
        let cert_id_bytes = [i; 32];
        let params = test_env.make_cert_params(&cert_id_bytes, "PERF101", &student);
        params_list.push_back(params);
    }

    let result = test_env.client.batch_issue_certificates(&test_env.admin, &params_list);

    let duration = start.elapsed();

    assert_eq!(result.succeeded, 25);
    assert_eq!(result.failed, 0);
    assert!(
        duration.as_millis() < 10_000,
        "Batch of 25 certificates should complete within 10 seconds"
    );
}

#[test]
fn test_performance_multisig_workflow_baseline() {
    let test_env = CertificateTestEnv::new();
    let start = Instant::now();

    let approver1 = test_env.generate_address();
    let approver2 = test_env.generate_address();
    let student = test_env.generate_address();

    let config =
        test_env.make_multisig_config("PERF_MS", &[approver1.clone(), approver2.clone()], 2, true);
    test_env.client.configure_multisig(&test_env.admin, &config);

    for i in 0u8..10 {
        let cert_id_bytes = [30 + i; 32];
        let params = test_env.make_cert_params(&cert_id_bytes, "PERF_MS", &student);
        let request_id = test_env.client.create_multisig_request(
            &student,
            &params,
            &String::from_str(&test_env.env, "Performance test"),
        );

        test_env.client.process_multisig_approval(
            &approver1,
            &request_id,
            &true,
            &String::from_str(&test_env.env, "Approve"),
            &None,
        );
        test_env.client.process_multisig_approval(
            &approver2,
            &request_id,
            &true,
            &String::from_str(&test_env.env, "Approve"),
            &None,
        );
    }

    let duration = start.elapsed();
    let avg_time = duration.as_millis() / 10;

    assert!(
        avg_time < 500,
        "Average multisig workflow should complete within 500ms, took {}ms",
        avg_time
    );
}

// ─────────────────────────────────────────────────────────────
// 15. Health Check
// ─────────────────────────────────────────────────────────────

#[test]
fn test_certificate_contract_health_check() {
    let test_env = CertificateTestEnv::new();
    let report = test_env.client.health_check();
    assert_eq!(report.status, shared::monitoring::ContractHealthStatus::Healthy);
}

// ─────────────────────────────────────────────────────────────
// 16. Rate Limits
// ─────────────────────────────────────────────────────────────

#[test]
fn test_rate_limit_configuration() {
    let test_env = CertificateTestEnv::new();

    let new_limits = certificate::types::CertRateLimitConfig {
        max_requests_per_day: 50,
        window_seconds: 86_400,
    };

    test_env.client.update_rate_limits(&test_env.admin, &new_limits);

    // Rate limit update should succeed without error
    // Subsequent requests should use the new limits
    let student = test_env.generate_address();
    let config = test_env.make_multisig_config("RL101", &[student.clone()], 1, true);
    test_env.client.configure_multisig(&test_env.admin, &config);
}
