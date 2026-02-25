

use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, String, Vec};

use crate::{
    types::{
        CertificatePriority, CertificateStatus, ComplianceStandard, FieldType,
        MintCertificateParams, MultiSigConfig, MultiSigRequestStatus, TemplateField,
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
    let approvers: [Address; 3] = [
        Address::generate(&env),
        Address::generate(&env),
        Address::generate(&env),
    ];
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

    let config = make_multisig_config(
        &env,
        "REJECTION_COURSE",
        &[approver1.clone(), approver2.clone()],
        2,
    );
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

    let config = make_multisig_config(
        &env,
        "DUP_COURSE",
        &[approver1.clone(), approver2.clone()],
        2,
    );
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

    let mut config = make_multisig_config(&env, "MANUAL_COURSE", core::slice::from_ref(&approver1), 1);
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

    // Verify analytics
    let analytics = client.get_analytics();
    assert_eq!(analytics.total_issued, 3);
    assert_eq!(analytics.active_certificates, 3);
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

    let mut field_values: Vec<String> = Vec::new(&env);
    field_values.push_back(String::from_str(&env, "John Doe"));
    field_values.push_back(String::from_str(&env, "2026-02-25"));

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
    let mut field_values: Vec<String> = Vec::new(&env);
    field_values.push_back(String::from_str(&env, "Only Name"));

    let result = client.try_issue_with_template(&admin, &template_id, &params, &field_values);
    assert!(result.is_err());
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

    let record = client
        .get_compliance_record(&params.certificate_id)
        .unwrap();
    assert!(record.is_compliant);
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
// 12. Audit Trail
// ─────────────────────────────────────────────────────────────
#[test]
fn test_audit_trail() {
    let (env, client, admin) = setup_env();
    let student = Address::generate(&env);
    let approver1 = Address::generate(&env);
    let approver2 = Address::generate(&env);

    let config = make_multisig_config(
        &env,
        "AUDIT_COURSE",
        &[approver1.clone(), approver2.clone()],
        2,
    );
    client.configure_multisig(&admin, &config);

    let mut params = make_cert_params(&env, "AUDIT_COURSE", &student);
    params.certificate_id = BytesN::from_array(&env, &[90u8; 32]);

    let requester = Address::generate(&env);
    let request_id =
        client.create_multisig_request(&requester, &params, &String::from_str(&env, "Audit test"));

    client.process_multisig_approval(
        &approver1,
        &request_id,
        &true,
        &String::from_str(&env, "LGTM"),
        &None,
    );
    client.process_multisig_approval(
        &approver2,
        &request_id,
        &true,
        &String::from_str(&env, "Confirmed"),
        &None,
    );

    let trail = client.get_multisig_audit_trail(&request_id);
    // Should have: Created + ApprovalGranted + ApprovalGranted
    assert!(trail.len() >= 3);
}

// ─────────────────────────────────────────────────────────────
// 13. Pending Approvals Query
// ─────────────────────────────────────────────────────────────
#[test]
fn test_get_pending_approvals() {
    let (env, client, admin) = setup_env();
    let student = Address::generate(&env);
    let approver1 = Address::generate(&env);
    let approver2 = Address::generate(&env);

    let config = make_multisig_config(
        &env,
        "PENDING_COURSE",
        &[approver1.clone(), approver2.clone()],
        2,
    );
    client.configure_multisig(&admin, &config);

    let mut params = make_cert_params(&env, "PENDING_COURSE", &student);
    params.certificate_id = BytesN::from_array(&env, &[95u8; 32]);

    let requester = Address::generate(&env);
    let _request_id = client.create_multisig_request(
        &requester,
        &params,
        &String::from_str(&env, "Pending test"),
    );

    let pending = client.get_pending_approvals(&approver1);
    assert_eq!(pending.len(), 1);
}

// ─────────────────────────────────────────────────────────────
// 14. Analytics Tracking
// ─────────────────────────────────────────────────────────────
#[test]
fn test_analytics_tracking() {
    let (env, client, admin) = setup_env();

    // Issue a batch
    let mut params_list: Vec<MintCertificateParams> = Vec::new(&env);
    for i in 0u8..5 {
        let student = Address::generate(&env);
        let mut cert_id_bytes = [0u8; 32];
        cert_id_bytes[0] = 200 + i;
        let params = MintCertificateParams {
            certificate_id: BytesN::from_array(&env, &cert_id_bytes),
            course_id: String::from_str(&env, "ANALYTICS_COURSE"),
            student,
            title: String::from_str(&env, "Analytics Cert"),
            description: String::from_str(&env, "For analytics testing"),
            metadata_uri: String::from_str(&env, "https://example.com/analytics"),
            expiry_date: env.ledger().timestamp() + 31_536_000,
        };
        params_list.push_back(params);
    }

    client.batch_issue_certificates(&admin, &params_list);

    let analytics = client.get_analytics();
    assert_eq!(analytics.total_issued, 5);
    assert_eq!(analytics.active_certificates, 5);
    assert_eq!(analytics.total_revoked, 0);
    assert_eq!(analytics.total_shared, 0);
}

// ─────────────────────────────────────────────────────────────
// 15. Student Certificates Query
// ─────────────────────────────────────────────────────────────
#[test]
fn test_student_certificates() {
    let (env, client, admin) = setup_env();

    let student = Address::generate(&env);

    let mut params_list: Vec<MintCertificateParams> = Vec::new(&env);
    for i in 0u8..3 {
        let mut cert_id_bytes = [0u8; 32];
        cert_id_bytes[0] = 150 + i;
        let params = MintCertificateParams {
            certificate_id: BytesN::from_array(&env, &cert_id_bytes),
            course_id: String::from_str(&env, "STUDENT_COURSE"),
            student: student.clone(),
            title: String::from_str(&env, "Student Cert"),
            description: String::from_str(&env, "For student query testing"),
            metadata_uri: String::from_str(&env, "https://example.com/student"),
            expiry_date: env.ledger().timestamp() + 31_536_000,
        };
        params_list.push_back(params);
    }

    client.batch_issue_certificates(&admin, &params_list);

    let certs = client.get_student_certificates(&student);
    assert_eq!(certs.len(), 3);
}

// ─────────────────────────────────────────────────────────────
// 16. Priority-based Configuration
// ─────────────────────────────────────────────────────────────
#[test]
fn test_certificate_priority_levels() {
    assert_eq!(CertificatePriority::Standard.required_approvals(), 1);
    assert_eq!(CertificatePriority::Premium.required_approvals(), 2);
    assert_eq!(CertificatePriority::Enterprise.required_approvals(), 3);
    assert_eq!(CertificatePriority::Institutional.required_approvals(), 5);
}
