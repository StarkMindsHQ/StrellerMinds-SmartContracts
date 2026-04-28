use shared::monitoring::ContractHealthStatus;
use soroban_sdk::{
    testutils::{Address as _, Ledger as _},
    Address, BytesN, Env, Map, String, Vec,
};

use crate::{
    types::{
        CertDataKey, CertRateLimitConfig, CertificatePriority, CertificateStatus,
        ComplianceStandard, FieldType, MintCertificateParams, MultiSigConfig,
        MultiSigRequestStatus, TemplateField,
    },
    CertificateContract, CertificateContractClient,
};

// ─────────────────────────────────────────────────────────────
// Helper utilities
// ─────────────────────────────────────────────────────────────
fn setup_env() -> (Env, CertificateContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(CertificateContract, ());
    let client = CertificateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);
    (env, client, admin)
}

fn make_cert_params(env: &Env, course_id: &str, student: &Address) -> MintCertificateParams {
    MintCertificateParams {
        certificate_id: BytesN::from_array(env, &[1u8; 32]),
        course_id: String::from_str(env, course_id),
        student: student.clone(),
        title: String::from_str(env, "Test Certificate"),
        description: String::from_str(env, "Certificate for testing"),
        metadata_uri: String::from_str(env, "https://example.com/cert/metadata"),
        expiry_date: env.ledger().timestamp() + 31_536_000, // 1 year
    }
}

fn make_multisig_config(
    env: &Env,
    course_id: &str,
    approvers: &[Address],
    required: u32,
) -> MultiSigConfig {
    let mut appr_vec: Vec<Address> = Vec::new(env);
    for a in approvers {
        appr_vec.push_back(a.clone());
    }
    MultiSigConfig {
        course_id: String::from_str(env, course_id),
        required_approvals: required,
        authorized_approvers: appr_vec,
        timeout_duration: 604_800, // 7 days
        priority: CertificatePriority::Enterprise,
        auto_execute: true,
    }
}

// ─────────────────────────────────────────────────────────────
// 1. Initialisation tests
// ─────────────────────────────────────────────────────────────
#[test]
fn test_initialize() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CertificateContract, ());
    let client = CertificateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);
}

#[test]
fn test_double_initialize_fails() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(CertificateContract, ());
    let client = CertificateContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);
    let result = client.try_initialize(&admin);
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────
// 2. Multi-Sig Configuration tests
// ─────────────────────────────────────────────────────────────
#[test]
fn test_configure_multisig() {
    let (env, client, admin) = setup_env();
    let approvers: [Address; 3] =
        [Address::generate(&env), Address::generate(&env), Address::generate(&env)];
    let config = make_multisig_config(&env, "COURSE_001", &approvers, 2);

    client.configure_multisig(&admin, &config);

    let retrieved = client.get_multisig_config(&String::from_str(&env, "COURSE_001"));
    assert!(retrieved.is_some());
    let retrieved = retrieved.unwrap();
    assert_eq!(retrieved.required_approvals, 2);
}

#[test]
fn test_configure_multisig_invalid_threshold() {
    let (env, client, admin) = setup_env();
    let approvers: [Address; 1] = [Address::generate(&env)];
    // required > available approvers
    let config = make_multisig_config(&env, "COURSE_002", &approvers, 5);

    let result = client.try_configure_multisig(&admin, &config);
    assert!(result.is_err());
}

#[test]
fn test_configure_multisig_timeout_too_short() {
    let (env, client, admin) = setup_env();
    let approvers: [Address; 2] = [Address::generate(&env), Address::generate(&env)];
    let mut config = make_multisig_config(&env, "COURSE_003", &approvers, 1);
    config.timeout_duration = 60; // Too short (minimum 1 hour)

    let result = client.try_configure_multisig(&admin, &config);
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────
// 3. Multi-Sig Request Creation & Approval Flow
// ─────────────────────────────────────────────────────────────
#[test]
fn test_full_multisig_workflow() {
    let (env, client, admin) = setup_env();
    let student = Address::generate(&env);
    let approver1 = Address::generate(&env);
    let approver2 = Address::generate(&env);
    let approver3 = Address::generate(&env);

    let config = make_multisig_config(
        &env,
        "BLOCKCHAIN_101",
        &[approver1.clone(), approver2.clone(), approver3.clone()],
        2,
    );
    client.configure_multisig(&admin, &config);

    let params = make_cert_params(&env, "BLOCKCHAIN_101", &student);
    let requester = Address::generate(&env);
    let request_id = client.create_multisig_request(
        &requester,
        &params,
        &String::from_str(&env, "Student completed all requirements"),
    );

    // First approval
    client.process_multisig_approval(
        &approver1,
        &request_id,
        &true,
        &String::from_str(&env, "Approved - great work"),
        &None,
    );

    // Check request state after 1 approval
    let req = client.get_multisig_request(&request_id).unwrap();
    assert_eq!(req.current_approvals, 1);
    assert_eq!(req.status, MultiSigRequestStatus::Pending);

    // Second approval → threshold reached, auto-execute enabled
    client.process_multisig_approval(
        &approver2,
        &request_id,
        &true,
        &String::from_str(&env, "Confirmed completion"),
        &None,
    );

    // Verify certificate was auto-issued
    let req = client.get_multisig_request(&request_id).unwrap();
    assert_eq!(req.status, MultiSigRequestStatus::Executed);

    let cert = client
        .get_certificate(&params.certificate_id)
        .expect("Certificate should exist after auto-execute");
    assert_eq!(cert.status, CertificateStatus::Active);
    assert!(cert.blockchain_anchor.is_some());
}

#[test]
fn test_multisig_rejection() {
    let (env, client, admin) = setup_env();
    let student = Address::generate(&env);
    let approver1 = Address::generate(&env);
    let approver2 = Address::generate(&env);

    let config =
        make_multisig_config(&env, "REJECTION_COURSE", &[approver1.clone(), approver2.clone()], 2);
    client.configure_multisig(&admin, &config);

    let mut params = make_cert_params(&env, "REJECTION_COURSE", &student);
    params.certificate_id = BytesN::from_array(&env, &[2u8; 32]);

    let requester = Address::generate(&env);
    let request_id = client.create_multisig_request(
        &requester,
        &params,
        &String::from_str(&env, "Request for testing rejection"),
    );

    // Reject
    client.process_multisig_approval(
        &approver1,
        &request_id,
        &false,
        &String::from_str(&env, "Requirements not met"),
        &None,
    );

    let req = client.get_multisig_request(&request_id).unwrap();
    assert_eq!(req.status, MultiSigRequestStatus::Rejected);
}

#[test]
fn test_duplicate_approval_fails() {
    let (env, client, admin) = setup_env();
    let student = Address::generate(&env);
    let approver1 = Address::generate(&env);
    let approver2 = Address::generate(&env);

    let config =
        make_multisig_config(&env, "DUP_COURSE", &[approver1.clone(), approver2.clone()], 2);
    client.configure_multisig(&admin, &config);

    let mut params = make_cert_params(&env, "DUP_COURSE", &student);
    params.certificate_id = BytesN::from_array(&env, &[3u8; 32]);

    let requester = Address::generate(&env);
    let request_id = client.create_multisig_request(
        &requester,
        &params,
        &String::from_str(&env, "Duplicate test"),
    );

    client.process_multisig_approval(
        &approver1,
        &request_id,
        &true,
        &String::from_str(&env, "OK"),
        &None,
    );

    // Same approver again
    let result = client.try_process_multisig_approval(
        &approver1,
        &request_id,
        &true,
        &String::from_str(&env, "Duplicate"),
        &None,
    );
    assert!(result.is_err());
}

#[test]
fn test_unauthorized_approver_fails() {
    let (env, client, admin) = setup_env();
    let student = Address::generate(&env);
    let approver1 = Address::generate(&env);
    let unauthorized = Address::generate(&env);

    let config = make_multisig_config(&env, "AUTH_COURSE", core::slice::from_ref(&approver1), 1);
    client.configure_multisig(&admin, &config);

    let mut params = make_cert_params(&env, "AUTH_COURSE", &student);
    params.certificate_id = BytesN::from_array(&env, &[4u8; 32]);

    let requester = Address::generate(&env);
    let request_id =
        client.create_multisig_request(&requester, &params, &String::from_str(&env, "Auth test"));

    let result = client.try_process_multisig_approval(
        &unauthorized,
        &request_id,
        &true,
        &String::from_str(&env, "Trying"),
        &None,
    );
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────
// 4. Manual Execution
// ─────────────────────────────────────────────────────────────
#[test]
fn test_manual_execution() {
    let (env, client, admin) = setup_env();
    let student = Address::generate(&env);
    let approver1 = Address::generate(&env);

    let mut config =
        make_multisig_config(&env, "MANUAL_COURSE", core::slice::from_ref(&approver1), 1);
    config.auto_execute = false; // Disable auto-execute
    client.configure_multisig(&admin, &config);

    let mut params = make_cert_params(&env, "MANUAL_COURSE", &student);
    params.certificate_id = BytesN::from_array(&env, &[5u8; 32]);

    let requester = Address::generate(&env);
    let request_id = client.create_multisig_request(
        &requester,
        &params,
        &String::from_str(&env, "Manual exec test"),
    );

    client.process_multisig_approval(
        &approver1,
        &request_id,
        &true,
        &String::from_str(&env, "Approved"),
        &None,
    );

    // Request should be approved but not executed
    let req = client.get_multisig_request(&request_id).unwrap();
    assert_eq!(req.status, MultiSigRequestStatus::Approved);

    // Manually execute
    let executor = Address::generate(&env);
    client.execute_multisig_request(&executor, &request_id);

    let req = client.get_multisig_request(&request_id).unwrap();
    assert_eq!(req.status, MultiSigRequestStatus::Executed);

    let cert = client.get_certificate(&params.certificate_id).unwrap();
    assert_eq!(cert.status, CertificateStatus::Active);
}

// ─────────────────────────────────────────────────────────────
// 5. Batch Certificate Issuance
// ─────────────────────────────────────────────────────────────
#[test]
fn test_batch_issue_certificates() {
    let (env, client, admin) = setup_env();

    let mut params_list: Vec<MintCertificateParams> = Vec::new(&env);
    for i in 0u8..3 {
        let student = Address::generate(&env);
        let mut cert_id_bytes = [0u8; 32];
        cert_id_bytes[0] = 10 + i;
        let params = MintCertificateParams {
            certificate_id: BytesN::from_array(&env, &cert_id_bytes),
            course_id: String::from_str(&env, "BATCH_COURSE"),
            student,
            title: String::from_str(&env, "Batch Cert"),
            description: String::from_str(&env, "Batch issued"),
            metadata_uri: String::from_str(&env, "https://example.com/batch"),
            expiry_date: env.ledger().timestamp() + 31_536_000,
        };
        params_list.push_back(params);
    }

    let result = client.batch_issue_certificates(&admin, &params_list);
    assert_eq!(result.total, 3);
    assert_eq!(result.succeeded, 3);
    assert_eq!(result.failed, 0);
    assert_eq!(result.certificate_ids.len(), 3);

    // Verify student certificates
    let _student_certs = client.get_student_certificates(&params_list.first().unwrap().student);
    // Note: since students were randomly generated each iteration, we need to check the last one
    let last_student = params_list.last().unwrap().student.clone();
    let last_certs = client.get_student_certificates(&last_student);
    assert_eq!(last_certs.len(), 1, "Student should have 1 certificate");

    // Verify analytics
    let analytics = client.get_analytics();
    assert_eq!(analytics.total_issued, 3);
    assert_eq!(analytics.active_certificates, 3);
}

#[test]
fn test_batch_issue_certificates_duplicate() {
    let (env, client, admin) = setup_env();

    let mut params_list: Vec<MintCertificateParams> = Vec::new(&env);
    let student = Address::generate(&env);

    // Add same certificate twice
    let mut cert_id_bytes = [0u8; 32];
    cert_id_bytes[0] = 10;
    let params = MintCertificateParams {
        certificate_id: BytesN::from_array(&env, &cert_id_bytes),
        course_id: String::from_str(&env, "BATCH_COURSE"),
        student: student.clone(),
        title: String::from_str(&env, "Batch Cert"),
        description: String::from_str(&env, "Batch issued"),
        metadata_uri: String::from_str(&env, "https://example.com/batch"),
        expiry_date: env.ledger().timestamp() + 31_536_000,
    };

    params_list.push_back(params.clone());
    params_list.push_back(params.clone());

    let result = client.batch_issue_certificates(&admin, &params_list);
    assert_eq!(result.total, 2);
    assert_eq!(result.succeeded, 1, "Duplicate should be ignored");
    assert_eq!(result.failed, 1, "Duplicate should fail");
}

#[test]
fn test_batch_empty_fails() {
    let (env, client, admin) = setup_env();
    let empty: Vec<MintCertificateParams> = Vec::new(&env);

    let result = client.try_batch_issue_certificates(&admin, &empty);
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────
// 6. Certificate Verification
// ─────────────────────────────────────────────────────────────
#[test]
fn test_verify_certificate() {
    let (env, client, admin) = setup_env();

    let mut params_list: Vec<MintCertificateParams> = Vec::new(&env);
    let student = Address::generate(&env);
    let params = make_cert_params(&env, "VERIFY_COURSE", &student);
    params_list.push_back(params.clone());

    client.batch_issue_certificates(&admin, &params_list);

    let is_valid = client.verify_certificate(&params.certificate_id);
    assert!(is_valid);
}

// ─────────────────────────────────────────────────────────────
// 7. Revocation & Reissuance
// ─────────────────────────────────────────────────────────────
#[test]
fn test_revoke_certificate() {
    let (env, client, admin) = setup_env();

    let student = Address::generate(&env);
    let params = make_cert_params(&env, "REVOKE_COURSE", &student);
    let mut list: Vec<MintCertificateParams> = Vec::new(&env);
    list.push_back(params.clone());
    client.batch_issue_certificates(&admin, &list);

    client.revoke_certificate(
        &admin,
        &params.certificate_id,
        &String::from_str(&env, "Academic dishonesty"),
        &true,
    );

    let cert = client.get_certificate(&params.certificate_id).unwrap();
    assert_eq!(cert.status, CertificateStatus::Revoked);

    let analytics = client.get_analytics();
    assert_eq!(analytics.total_revoked, 1);
}

#[test]
fn test_reissue_certificate() {
    let (env, client, admin) = setup_env();

    let student = Address::generate(&env);
    let params = make_cert_params(&env, "REISSUE_COURSE", &student);
    let mut list: Vec<MintCertificateParams> = Vec::new(&env);
    list.push_back(params.clone());
    client.batch_issue_certificates(&admin, &list);

    // Revoke with reissuance eligible
    client.revoke_certificate(
        &admin,
        &params.certificate_id,
        &String::from_str(&env, "Error in certificate"),
        &true,
    );

    // Create new params for reissue
    let new_params = MintCertificateParams {
        certificate_id: BytesN::from_array(&env, &[99u8; 32]),
        course_id: String::from_str(&env, "REISSUE_COURSE"),
        student: student.clone(),
        title: String::from_str(&env, "Corrected Certificate"),
        description: String::from_str(&env, "Reissued with corrections"),
        metadata_uri: String::from_str(&env, "https://example.com/reissued"),
        expiry_date: env.ledger().timestamp() + 31_536_000,
    };

    let new_id = client.reissue_certificate(&admin, &params.certificate_id, &new_params);
    assert_eq!(new_id, new_params.certificate_id);

    // Old cert should be marked reissued
    let old_cert = client.get_certificate(&params.certificate_id).unwrap();
    assert_eq!(old_cert.status, CertificateStatus::Reissued);

    // New cert should be active with incremented version
    let new_cert = client.get_certificate(&new_params.certificate_id).unwrap();
    assert_eq!(new_cert.status, CertificateStatus::Active);
    assert_eq!(new_cert.version, 2);
}

#[test]
fn test_reissue_not_eligible_fails() {
    let (env, client, admin) = setup_env();

    let student = Address::generate(&env);
    let mut params = make_cert_params(&env, "NOELIGIBLE_COURSE", &student);
    params.certificate_id = BytesN::from_array(&env, &[77u8; 32]);
    let mut list: Vec<MintCertificateParams> = Vec::new(&env);
    list.push_back(params.clone());
    client.batch_issue_certificates(&admin, &list);

    // Revoke WITHOUT reissuance eligibility
    client.revoke_certificate(
        &admin,
        &params.certificate_id,
        &String::from_str(&env, "Permanent revocation"),
        &false,
    );

    let new_params = MintCertificateParams {
        certificate_id: BytesN::from_array(&env, &[88u8; 32]),
        course_id: String::from_str(&env, "NOELIGIBLE_COURSE"),
        student: student.clone(),
        title: String::from_str(&env, "Attempt Reissue"),
        description: String::from_str(&env, "Should fail"),
        metadata_uri: String::from_str(&env, "https://example.com/fail"),
        expiry_date: env.ledger().timestamp() + 31_536_000,
    };

    let result = client.try_reissue_certificate(&admin, &params.certificate_id, &new_params);
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────
// 8. Certificate Templates
// ─────────────────────────────────────────────────────────────
#[test]
fn test_create_and_use_template() {
    let (env, client, admin) = setup_env();

    let template_id = String::from_str(&env, "PROFESSIONAL_CERT_V1");
    let mut fields: Vec<TemplateField> = Vec::new(&env);
    fields.push_back(TemplateField {
        field_name: String::from_str(&env, "student_name"),
        field_type: FieldType::Text,
        is_required: true,
        default_value: None,
    });
    fields.push_back(TemplateField {
        field_name: String::from_str(&env, "completion_date"),
        field_type: FieldType::Date,
        is_required: true,
        default_value: None,
    });
    fields.push_back(TemplateField {
        field_name: String::from_str(&env, "grade"),
        field_type: FieldType::Text,
        is_required: false,
        default_value: Some(String::from_str(&env, "Pass")),
    });

    client.create_template(
        &admin,
        &template_id,
        &String::from_str(&env, "Professional Certificate"),
        &String::from_str(&env, "Template for professional certifications"),
        &fields,
    );

    let template = client.get_template(&template_id).unwrap();
    assert!(template.is_active);
    assert_eq!(template.fields.len(), 3);

    // Issue certificate with template
    let student = Address::generate(&env);
    let mut params = make_cert_params(&env, "TEMPLATE_COURSE", &student);
    params.certificate_id = BytesN::from_array(&env, &[50u8; 32]);

    let mut field_values: Map<String, String> = Map::new(&env);
    field_values.set(String::from_str(&env, "student_name"), String::from_str(&env, "John Doe"));
    field_values
        .set(String::from_str(&env, "completion_date"), String::from_str(&env, "2026-02-25"));

    let cert_id = client.issue_with_template(&admin, &template_id, &params, &field_values);
    assert_eq!(cert_id, params.certificate_id);

    let cert = client.get_certificate(&cert_id).unwrap();
    assert_eq!(cert.template_id, Some(template_id));
}

#[test]
fn test_template_missing_required_fields_fails() {
    let (env, client, admin) = setup_env();

    let template_id = String::from_str(&env, "STRICT_TEMPLATE");
    let mut fields: Vec<TemplateField> = Vec::new(&env);
    fields.push_back(TemplateField {
        field_name: String::from_str(&env, "name"),
        field_type: FieldType::Text,
        is_required: true,
        default_value: None,
    });
    fields.push_back(TemplateField {
        field_name: String::from_str(&env, "date"),
        field_type: FieldType::Date,
        is_required: true,
        default_value: None,
    });

    client.create_template(
        &admin,
        &template_id,
        &String::from_str(&env, "Strict Template"),
        &String::from_str(&env, "All fields required"),
        &fields,
    );

    let student = Address::generate(&env);
    let mut params = make_cert_params(&env, "STRICT_COURSE", &student);
    params.certificate_id = BytesN::from_array(&env, &[60u8; 32]);

    // Only provide 1 value when 2 are required
    let mut field_values: Map<String, String> = Map::new(&env);
    field_values.set(String::from_str(&env, "name"), String::from_str(&env, "Only Name"));

    let result = client.try_issue_with_template(&admin, &template_id, &params, &field_values);
    assert!(result.is_err());
}

#[test]
fn test_automated_compliance_audit() {
    let (env, client, admin) = setup_env();
    let student = Address::generate(&env);
    let params = make_cert_params(&env, "AUDIT_TEST_COURSE", &student);
    let mut list: Vec<MintCertificateParams> = Vec::new(&env);
    list.push_back(params.clone());
    client.batch_issue_certificates(&admin, &list);

    // Initial audit should pass
    let is_compliant = client.automated_compliance_audit(&params.certificate_id);
    assert!(is_compliant);

    // Revoke and check again
    client.revoke_certificate(
        &admin,
        &params.certificate_id,
        &String::from_str(&env, "Revoked"),
        &false,
    );
    let is_compliant_after = client.automated_compliance_audit(&params.certificate_id);
    assert!(!is_compliant_after);
}

// ─────────────────────────────────────────────────────────────
// 9. Compliance Verification
// ─────────────────────────────────────────────────────────────
#[test]
fn test_verify_compliance() {
    let (env, client, admin) = setup_env();

    let student = Address::generate(&env);
    let params = make_cert_params(&env, "COMPLY_COURSE", &student);
    let mut list: Vec<MintCertificateParams> = Vec::new(&env);
    list.push_back(params.clone());
    client.batch_issue_certificates(&admin, &list);

    let verifier = Address::generate(&env);
    let is_compliant = client.verify_compliance(
        &verifier,
        &params.certificate_id,
        &ComplianceStandard::Iso17024,
        &String::from_str(&env, "Meets ISO 17024 requirements"),
    );
    assert!(is_compliant);

    let record = client.get_compliance_record(&params.certificate_id).unwrap();
    assert!(record.is_compliant);
}

#[test]
fn test_verify_compliance_iso9001() {
    let (env, client, admin) = setup_env();

    let student = Address::generate(&env);
    let mut params = make_cert_params(&env, "ISO9001_COURSE", &student);
    params.certificate_id = BytesN::from_array(&env, &[91u8; 32]);
    let mut list: Vec<MintCertificateParams> = Vec::new(&env);
    list.push_back(params.clone());
    client.batch_issue_certificates(&admin, &list);

    let verifier = Address::generate(&env);
    let is_compliant = client.verify_compliance(
        &verifier,
        &params.certificate_id,
        &ComplianceStandard::Iso9001,
        &String::from_str(&env, "Meets ISO 9001 quality management requirements"),
    );
    assert!(is_compliant, "ISO 9001 compliance check must return true for active certificate");

    let record = client.get_compliance_record(&params.certificate_id).unwrap();
    assert!(record.is_compliant);
    assert_eq!(record.standard, ComplianceStandard::Iso9001);
}

// ─────────────────────────────────────────────────────────────
// 10. Certificate Sharing & Social Verification
// ─────────────────────────────────────────────────────────────
#[test]
fn test_share_certificate() {
    let (env, client, admin) = setup_env();

    let student = Address::generate(&env);
    let params = make_cert_params(&env, "SHARE_COURSE", &student);
    let mut list: Vec<MintCertificateParams> = Vec::new(&env);
    list.push_back(params.clone());
    client.batch_issue_certificates(&admin, &list);

    client.share_certificate(
        &student,
        &params.certificate_id,
        &String::from_str(&env, "LinkedIn"),
        &String::from_str(&env, "https://verify.example.com/cert/abc123"),
    );

    let records = client.get_share_records(&params.certificate_id);
    assert_eq!(records.len(), 1);

    let cert = client.get_certificate(&params.certificate_id).unwrap();
    assert_eq!(cert.share_count, 1);

    let analytics = client.get_analytics();
    assert_eq!(analytics.total_shared, 1);
}

#[test]
fn test_share_revoked_cert_fails() {
    let (env, client, admin) = setup_env();

    let student = Address::generate(&env);
    let mut params = make_cert_params(&env, "SHARE_REVOKED", &student);
    params.certificate_id = BytesN::from_array(&env, &[70u8; 32]);
    let mut list: Vec<MintCertificateParams> = Vec::new(&env);
    list.push_back(params.clone());
    client.batch_issue_certificates(&admin, &list);

    client.revoke_certificate(
        &admin,
        &params.certificate_id,
        &String::from_str(&env, "Revoked"),
        &false,
    );

    let result = client.try_share_certificate(
        &student,
        &params.certificate_id,
        &String::from_str(&env, "LinkedIn"),
        &String::from_str(&env, "https://verify.example.com/revoked"),
    );
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────
// 11. Authenticity Verification
// ─────────────────────────────────────────────────────────────
#[test]
fn test_verify_authenticity() {
    let (env, client, admin) = setup_env();

    let student = Address::generate(&env);
    let params = make_cert_params(&env, "AUTH_VERIFY_COURSE", &student);
    let mut list: Vec<MintCertificateParams> = Vec::new(&env);
    list.push_back(params.clone());
    client.batch_issue_certificates(&admin, &list);

    let is_authentic = client.verify_authenticity(&params.certificate_id);
    assert!(is_authentic);
}

#[test]
fn test_verify_revoked_cert_not_authentic() {
    let (env, client, admin) = setup_env();

    let student = Address::generate(&env);
    let mut params = make_cert_params(&env, "AUTH_REVOKED_COURSE", &student);
    params.certificate_id = BytesN::from_array(&env, &[80u8; 32]);
    let mut list: Vec<MintCertificateParams> = Vec::new(&env);
    list.push_back(params.clone());
    client.batch_issue_certificates(&admin, &list);

    client.revoke_certificate(
        &admin,
        &params.certificate_id,
        &String::from_str(&env, "Revoked for testing"),
        &false,
    );

    let is_authentic = client.verify_authenticity(&params.certificate_id);
    assert!(!is_authentic);
}

// ─────────────────────────────────────────────────────────────
// Reproduce Issue #378: Revoked certificate still passing verification
// ─────────────────────────────────────────────────────────────
#[test]
fn test_reproduce_issue_378() {
    let (env, client, admin) = setup_env();

    let student = Address::generate(&env);
    let params = make_cert_params(&env, "REPRO_378", &student);
    let mut list: Vec<MintCertificateParams> = Vec::new(&env);
    list.push_back(params.clone());
    client.batch_issue_certificates(&admin, &list);

    // Verify it is active first
    assert!(client.verify_certificate(&params.certificate_id));

    // Revoke the certificate
    client.revoke_certificate(
        &admin,
        &params.certificate_id,
        &String::from_str(&env, "Test Revocation"),
        &false,
    );

    // Verify certificate SHOULD BE false
    let is_valid = client.verify_certificate(&params.certificate_id);
    assert!(!is_valid, "Revoked certificate should not be valid");

    // Verify authenticity SHOULD BE false
    let is_authentic = client.verify_authenticity(&params.certificate_id);
    assert!(!is_authentic, "Revoked certificate should not be authentic");
}

    let contract_id = env.register(DashboardPreferencesContract, ());
    let client = DashboardPreferencesContractClient::new(&env, &contract_id);

    let user = Address::generate(&env);
    let layout_json = String::from_str(&env, r#"{"widgets":["progress","certificates"],"theme":"dark"}"#);

    // 1. Test Saving Layout
    client.save_layout(&user, &layout_json);

    // 2. Test Retrieving Layout
    let retrieved_layout = client.get_layout(&user).unwrap();
    assert_eq!(retrieved_layout, layout_json);

    // 3. Test Updating Layout
    let updated_layout = String::from_str(&env, r#"{"widgets":["analytics"],"theme":"light"}"#);
    client.save_layout(&user, &updated_layout);
    let new_retrieved = client.get_layout(&user).unwrap();
    assert_eq!(new_retrieved, updated_layout);

    let report = client.health_check();
    assert_eq!(report.status, ContractHealthStatus::Unknown);
    assert!(!report.initialized);
}

// ─────────────────────────────────────────────────────────────
// 18. Advanced Compliance & Governance
// ─────────────────────────────────────────────────────────────
#[test]
fn test_compliance_officer_and_manual_override() {
    let (env, client, admin) = setup_env();
    let officer = Address::generate(&env);

    // Set officer
    client.set_compliance_officer(&admin, &officer);

    // Issue certificate
    let student = Address::generate(&env);
    let params = make_cert_params(&env, "OVERRIDE_COURSE", &student);
    let mut list = Vec::new(&env);
    list.push_back(params.clone());
    client.batch_issue_certificates(&admin, &list);

    // Manual override
    client.manual_compliance_override(
        &officer,
        &params.certificate_id,
        &String::from_str(&env, "Flagged for manual review"),
    );

    let cert = client.get_certificate(&params.certificate_id).unwrap();
    assert_eq!(cert.status, CertificateStatus::NonCompliant);

    let analytics = client.get_analytics();
    assert_eq!(analytics.compliance_violations_count, 1);
}

#[test]
fn test_automated_compliance_audit_rate_limit() {
    let (env, client, admin) = setup_env();

    // Issue certificate
    let student = Address::generate(&env);
    let params = make_cert_params(&env, "RL_COURSE", &student);
    let mut list = Vec::new(&env);
    list.push_back(params.clone());
    client.batch_issue_certificates(&admin, &list);

    // Configure strict rate limit (3 calls per day)
    env.as_contract(&client.address, || {
        env.storage().instance().set(
            &CertDataKey::RateLimitCfg,
            &CertRateLimitConfig { max_requests_per_day: 3, window_seconds: 86_400 },
        );
    });

    // Call 1, 2, 3: Succeed
    assert!(client.try_automated_compliance_audit(&params.certificate_id).is_ok());
    assert!(client.try_automated_compliance_audit(&params.certificate_id).is_ok());
    assert!(client.try_automated_compliance_audit(&params.certificate_id).is_ok());

    // Call 4: Fail
    let res = client.try_automated_compliance_audit(&params.certificate_id);
    assert!(res.is_err());
}
