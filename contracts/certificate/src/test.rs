use crate::types::{CertificateStatus, MintCertificateParams, Permission, Role};
use crate::{errors::CertificateError, Certificate, CertificateClient};
use soroban_sdk::testutils::Ledger;
use soroban_sdk::IntoVal;
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    vec, Address, BytesN, Env, String,
};

// Test helper function to create a test environment with CertificateContract
fn setup_test() -> (
    Env,
    CertificateClient<'static>,
    Address, // admin
    Address, // user/student
) {
    let env = Env::default();
    let contract_id = env.register(Certificate, ());
    let admin = Address::generate(&env);
    let user = Address::generate(&env);

    // Initialize contract
    env.mock_all_auths();
    let client = CertificateClient::new(&env, &contract_id);
    client.initialize(&admin);

    (env, client, admin, user)
}

fn create_test_string(env: &Env, text: &str) -> String {
    String::from_str(env, text)
}

fn create_test_certificate_id(env: &Env, id: u32) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    bytes[0..4].copy_from_slice(&id.to_be_bytes());
    BytesN::from_array(env, &bytes)
}

// Create role objects for testing
fn create_issuer_role() -> Role {
    Role {
        can_issue: true,
        can_revoke: false,
    }
}

fn create_revoker_role() -> Role {
    Role {
        can_issue: false,
        can_revoke: true,
    }
}

fn create_full_role() -> Role {
    Role {
        can_issue: true,
        can_revoke: true,
    }
}

// Helper function to create MintCertificateParams for testing
fn create_mint_params(params: MintCertificateParams) -> MintCertificateParams {
    params
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register(Certificate, ());
    let admin = Address::generate(&env);

    env.mock_all_auths();

    // Initialize the contract
    let client = CertificateClient::new(&env, &contract_id);
    let result = client.try_initialize(&admin);

    // Verify initialization succeeded
    assert!(result.is_ok());

    // Test re-initialization (should fail)
    let result = client.try_initialize(&admin);
    assert_eq!(result, Err(Ok(CertificateError::AlreadyInitialized)));

    // Verify admin is set correctly
    let admin_res = client.get_admin();
    assert_eq!(admin_res, admin);
}

#[test]
fn test_role_management() {
    let (env, client, admin, user) = setup_test();
    let user2 = Address::generate(&env);

    let full_role = create_full_role();
    let issue_role = create_issuer_role();

    // --- Grant Role ---
    // Only admin can grant
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, user.to_val(), full_role.into_val(&env)],
            sub_invokes: &[],
        },
    }]);

    let grant_result = client.try_grant_role(&user, &full_role);
    assert!(grant_result.is_ok());

    // Verify role stored
    let stored_role = client.get_role(&user).unwrap();
    assert_eq!(stored_role, full_role);

    // Non-admin cannot grant
    env.mock_auths(&[MockAuth {
        // Mock auth for user2 trying to grant
        address: &user2,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, user2.to_val(), issue_role.into_val(&env)],
            sub_invokes: &[],
        },
    }]);

    let grant_fail = client.try_grant_role(&user2, &issue_role);
    // The error should be related to authorization failing within check_admin
    assert!(grant_fail.is_err());

    // --- Get Role ---
    let retrieved_role = client.get_role(&user).unwrap();
    assert_eq!(retrieved_role, full_role);
    let non_existent_role = client.get_role(&user2);
    assert_eq!(non_existent_role, None);

    // --- Update Role ---
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_role",
            args: vec![&env, user.to_val(), issue_role.into_val(&env)],
            sub_invokes: &[],
        },
    }]);

    let update_result = client.try_update_role(&user, &issue_role);
    assert!(update_result.is_ok());

    let updated_role = client.get_role(&user).unwrap();
    assert_eq!(updated_role, issue_role);

    // Update non-existent role should fail
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_role",
            args: vec![&env, user2.to_val(), issue_role.into_val(&env)],
            sub_invokes: &[],
        },
    }]);

    let update_fail = client.try_update_role(&user2, &issue_role);
    assert_eq!(update_fail, Err(Ok(CertificateError::RoleNotFound)));

    // --- Revoke Role ---
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "revoke_role",
            args: vec![&env, user.to_val()],
            sub_invokes: &[],
        },
    }]);

    let revoke_result = client.try_revoke_role(&user);
    assert!(revoke_result.is_ok());

    let revoked_role = client.get_role(&user);
    assert!(revoked_role.is_none());

    // Revoke non-existent role should fail
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "revoke_role",
            args: vec![&env, user2.to_val()],
            sub_invokes: &[],
        },
    }]);

    let revoke_fail = client.try_revoke_role(&user2);
    assert_eq!(revoke_fail, Err(Ok(CertificateError::RoleNotFound)));
}

#[test]
fn test_permission_checks() {
    let (env, client, admin, user) = setup_test();
    let user2 = Address::generate(&env);

    let issuer_role = create_issuer_role();
    let revoker_role = create_revoker_role();

    // Grant issuer role to user
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, user.to_val(), issuer_role.into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.grant_role(&user, &issuer_role);

    // Grant revoker role to user2
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, user2.to_val(), revoker_role.into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.grant_role(&user2, &revoker_role);

    // Check permissions
    assert!(client.has_permission(&user, &Permission::Issue));
    assert!(!client.has_permission(&user, &Permission::Revoke));
    assert!(client.has_permission(&user2, &Permission::Revoke));
    assert!(!client.has_permission(&user2, &Permission::Issue));
    assert!(!client.has_permission(&Address::generate(&env), &Permission::Issue));
}

#[test]
fn test_certificate_mint_and_verify() {
    let (env, client, admin, student) = setup_test();
    let issuer = Address::generate(&env);

    // Grant issuer role
    let issuer_role = create_issuer_role();
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, issuer.to_val(), issuer_role.into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.grant_role(&issuer, &issuer_role);

    // Certificate data
    let cert_id = create_test_certificate_id(&env, 1);
    let course_id = create_test_string(&env, "CS101");
    let title = create_test_string(&env, "Intro to Computer Science");
    let desc = create_test_string(&env, "Fundamentals of Computer Science");
    let uri = create_test_string(&env, "ipfs://certificate-metadata-uri");
    let expiry_date = env.ledger().timestamp() + 1000000000;

    // Create params
    let params = create_mint_params(MintCertificateParams {
        certificate_id: cert_id.clone(),
        course_id: course_id.clone(),
        student: student.clone(),
        title: title.clone(),
        description: desc.clone(),
        metadata_uri: uri.clone(),
        expiry_date,
    });

    // Mint certificate
    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![
                &env,
                issuer.to_val(),
                params.clone().into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    let mint_result = client.try_mint_certificate(
        &issuer,
        &params,
    );
    assert!(mint_result.is_ok());
    // Verify certificate data
    let metadata = client.try_verify_certificate(&cert_id).unwrap();
    let metadata = metadata.unwrap();
    assert_eq!(metadata.title, title);
    assert_eq!(metadata.course_id, course_id);
    assert_eq!(metadata.description, desc);
    assert_eq!(metadata.metadata_uri, uri);
    assert_eq!(metadata.student_id, student);
    assert_eq!(metadata.instructor_id, issuer);
    assert_eq!(metadata.status, CertificateStatus::Active);

    // Verify user certificates
    let user_certs = client.track_certificates(&student);
    assert_eq!(user_certs.len(), 1);
    assert!(user_certs.contains(&cert_id));

    // Try minting same certificate again (should fail)
    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![
                &env,
                issuer.to_val(),
                params.clone().into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    let mint_fail = client.try_mint_certificate(
        &issuer,
        &params,
    );
    assert_eq!(
        mint_fail,
        Err(Ok(CertificateError::CertificateAlreadyExists))
    );

    // Try minting with invalid metadata
    let empty_title = create_test_string(&env, "");
    let test_cert_id2 = create_test_certificate_id(&env, 2);
    let invalid_params = create_mint_params(MintCertificateParams {
        certificate_id: test_cert_id2.clone(),
        course_id: course_id.clone(),
        student: student.clone(),
        title: empty_title.clone(),
        description: desc.clone(),
        metadata_uri: uri.clone(),
        expiry_date,
    });

    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![
                &env,
                issuer.to_val(),
                invalid_params.clone().into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    let mint_invalid = client.try_mint_certificate(
        &issuer,
        &invalid_params,
    );
    assert_eq!(mint_invalid, Err(Ok(CertificateError::InvalidMetadata)));
}

#[test]
fn test_certificate_revocation() {
    let (env, client, admin, student) = setup_test();
    let issuer = Address::generate(&env);
    let revoker = Address::generate(&env);

    // Grant roles
    let issuer_role = create_issuer_role();
    let revoker_role = create_revoker_role();

    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, issuer.to_val(), issuer_role.into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.grant_role(&issuer, &issuer_role);

    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, revoker.to_val(), revoker_role.into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.grant_role(&revoker, &revoker_role);

    // Certificate data
    let cert_id = create_test_certificate_id(&env, 1);
    let course_id = create_test_string(&env, "CS101");
    let title = create_test_string(&env, "Intro Certificate");
    let desc = create_test_string(&env, "Description");
    let uri = create_test_string(&env, "ipfs://...");
    let expiry_date = env.ledger().timestamp() + 1000000000;

    // Create params
    let params = create_mint_params(MintCertificateParams {
        certificate_id: cert_id.clone(),
        course_id: course_id.clone(),
        student: student.clone(),
        title: title.clone(),
        description: desc.clone(),
        metadata_uri: uri.clone(),
        expiry_date,
    });

    // Mint certificate
    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![
                &env,
                issuer.to_val(),
                params.clone().into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    client.mint_certificate(
        &issuer,
        &params,
    );

    // Try to revoke with issuer (should fail)
    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "revoke_certificate",
            args: vec![&env, cert_id.to_val()],
            sub_invokes: &[],
        },
    }]);

    let revoke_fail_issuer = client.try_revoke_certificate(&issuer, &cert_id);
    assert_eq!(revoke_fail_issuer, Err(Ok(CertificateError::Unauthorized)));

    // Revoke with proper authority
    env.mock_auths(&[MockAuth {
        address: &revoker,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "revoke_certificate",
            args: vec![&env, cert_id.to_val()],
            sub_invokes: &[],
        },
    }]);

    let revoke_result = client.try_revoke_certificate(&revoker, &cert_id);
    assert!(revoke_result.is_ok());

    // Verify certificate is revoked
    let revoked_metadata = client.try_verify_certificate(&cert_id);
    assert_eq!(
        revoked_metadata,
        Err(Ok(CertificateError::CertificateRevoked))
    );

    // Revoke non-existent certificate
    let non_existent_cert = create_test_certificate_id(&env, 99);
    env.mock_auths(&[MockAuth {
        address: &revoker,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "revoke_certificate",
            args: vec![&env, non_existent_cert.to_val()],
            sub_invokes: &[],
        },
    }]);

    let revoke_non_existent = client.try_revoke_certificate(&revoker, &non_existent_cert);
    assert_eq!(
        revoke_non_existent,
        Err(Ok(CertificateError::CertificateNotFound))
    );
}

#[test]
fn test_track_certificates() {
    let (env, client, admin, student) = setup_test();
    let issuer = Address::generate(&env);
    let other_student = Address::generate(&env);

    // Grant issuer role
    let issuer_role = create_issuer_role();
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, issuer.to_val(), issuer_role.into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.grant_role(&issuer, &issuer_role);

    // Create certificates
    let cert_id1 = create_test_certificate_id(&env, 1);
    let cert_id2 = create_test_certificate_id(&env, 2);
    let course_id = create_test_string(&env, "COURSE");
    let title = create_test_string(&env, "TITLE");
    let desc = create_test_string(&env, "DESC");
    let uri = create_test_string(&env, "URI");
    let expiry_date = env.ledger().timestamp() + 1000000000;

    // Create params for first certificate
    let params1 = create_mint_params(MintCertificateParams {
        certificate_id: cert_id1.clone(),
        course_id: course_id.clone(),
        student: student.clone(),
        title: title.clone(),
        description: desc.clone(),
        metadata_uri: uri.clone(),
        expiry_date,
    });

    // Mint certificates for first student
    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![
                &env,
                issuer.to_val(),
                params1.clone().into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    client.mint_certificate(
        &issuer,
        &params1,
    );

    // Create params for second certificate
    let params2 = create_mint_params(MintCertificateParams {
        certificate_id: cert_id2.clone(),
        course_id: course_id.clone(),
        student: student.clone(),
        title: title.clone(),
        description: desc.clone(),
        metadata_uri: uri.clone(),
        expiry_date,
    });

    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![
                &env,
                issuer.to_val(),
                params2.clone().into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);
    client.mint_certificate(
        &issuer,
        &params2,
    );

    // Track certificates for student
    let student_certs = client.track_certificates(&student);

    // Verify student has both certificates
    assert_eq!(student_certs.len(), 2);
    assert!(student_certs.contains(&cert_id1));
    assert!(student_certs.contains(&cert_id2));

    // Track certificates for other student (should be empty)
    let other_certs = client.track_certificates(&other_student);
    assert_eq!(other_certs.len(), 0);
}

#[test]
fn test_certificate_expiry() {
    let (env, client, admin, student) = setup_test();
    let issuer = Address::generate(&env);

    // Grant issuer role
    let issuer_role = create_issuer_role();
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, issuer.to_val(), issuer_role.into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.grant_role(&issuer, &issuer_role);

    // Certificate data
    let cert_id = create_test_certificate_id(&env, 1);
    let course_id = create_test_string(&env, "CS101");
    let title = create_test_string(&env, "Intro to Computer Science");
    let desc = create_test_string(&env, "Fundamentals of Computer Science");
    let uri = create_test_string(&env, "ipfs://certificate-metadata-uri");

    // Set expiry date to current time + 1000 seconds
    let current_time = env.ledger().timestamp();
    let expiry_date = current_time + 1000;

    // Create params
    let params = create_mint_params(MintCertificateParams {
        certificate_id: cert_id.clone(),
        course_id: course_id.clone(),
        student: student.clone(),
        title: title.clone(),
        description: desc.clone(),
        metadata_uri: uri.clone(),
        expiry_date,
    });

    // Mint certificate with future expiry date
    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![
                &env,
                issuer.to_val(),
                params.clone().into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    client.mint_certificate(
        &issuer,
        &params,
    );

    // Check certificate is not expired
    assert!(!client.is_certificate_expired(&cert_id));

    // Verify certificate should work
    let result = client.try_verify_certificate(&cert_id);
    assert!(result.is_ok());

    // Fast forward time past expiry date
    env.ledger().set_timestamp(expiry_date + 10);

    // Check certificate is now expired
    assert!(client.is_certificate_expired(&cert_id));

    // Verify certificate should now fail due to expiration
    let result = client.try_verify_certificate(&cert_id);
    assert_eq!(result, Err(Ok(CertificateError::CertificateExpired)));

    // Test permanent certificate (expiry_date = 0)
    let permanent_cert_id = create_test_certificate_id(&env, 2);
    let permanent_expiry = 0u64;

    // Create params for permanent certificate
    let permanent_params = create_mint_params(MintCertificateParams {
        certificate_id: permanent_cert_id.clone(),
        course_id: course_id.clone(),
        student: student.clone(),
        title: title.clone(),
        description: desc.clone(),
        metadata_uri: uri.clone(),
        expiry_date: permanent_expiry,
    });

    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![
                &env,
                issuer.to_val(),
                permanent_params.clone().into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    client.mint_certificate(
        &issuer,
        &permanent_params,
    );

    // Check permanent certificate is never expired, even when time advances
    assert!(!client.is_certificate_expired(&permanent_cert_id));

    // Fast forward time significantly (1 year in seconds)
    env.ledger().set_timestamp(current_time + 31536000);

    // Check permanent certificate is still not expired
    assert!(!client.is_certificate_expired(&permanent_cert_id));

    // Verify permanent certificate should still be valid
    let result = client.try_verify_certificate(&permanent_cert_id);
    assert!(result.is_ok());
}

#[test]
fn test_is_valid_certificate() {
    let (env, client, admin, student) = setup_test();
    let issuer = Address::generate(&env);

    // Grant issuer role
    let issuer_role = create_issuer_role();
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, issuer.to_val(), issuer_role.into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.grant_role(&issuer, &issuer_role);

    // Prepare certificate data
    let cert_id = create_test_certificate_id(&env, 1);
    let course_id = create_test_string(&env, "CS101");
    let title = create_test_string(&env, "Computer Science 101");
    let desc = create_test_string(&env, "Intro to CS certificate");
    let uri = create_test_string(&env, "ipfs://valid-metadata");

    // Test certificate that will not expire
    let non_expiring_date = 0u64;

    // Create params for non-expiring certificate
    let params = create_mint_params(MintCertificateParams {
        certificate_id: cert_id.clone(),
        course_id: course_id.clone(),
        student: student.clone(),
        title: title.clone(),
        description: desc.clone(),
        metadata_uri: uri.clone(),
        expiry_date: non_expiring_date,
    });

    // Mint certificate that won't expire
    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![
                &env,
                issuer.to_val(),
                params.clone().into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    client.mint_certificate(
        &issuer,
        &params,
    );

    // Check validity - should be valid
    let (is_valid, metadata) = client.try_is_valid_certificate(&cert_id).unwrap().unwrap();
    assert!(is_valid);
    assert_eq!(metadata.status, CertificateStatus::Active);

    // Now test with an expired certificate
    let expired_cert_id = create_test_certificate_id(&env, 2);
    // Set a specific past timestamp to avoid overflow
    let expired_date = 1000u64; // Use a small positive number for past date

    // Set the ledger timestamp to a value higher than expired_date
    env.ledger().set_timestamp(2000);

    // Create params for expired certificate
    let expired_params = create_mint_params(MintCertificateParams {
        certificate_id: expired_cert_id.clone(),
        course_id: course_id.clone(),
        student: student.clone(),
        title: title.clone(),
        description: desc.clone(),
        metadata_uri: uri.clone(),
        expiry_date: expired_date,
    });

    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![
                &env,
                issuer.to_val(),
                expired_params.clone().into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    client.mint_certificate(
        &issuer,
        &expired_params,
    );

    // Check validity - should be invalid due to expiry
    let (is_valid, metadata) = client
        .try_is_valid_certificate(&expired_cert_id)
        .unwrap()
        .unwrap();
    assert!(!is_valid);
    assert_eq!(metadata.status, CertificateStatus::Active); // Status is active but expired

    // Test non-existent certificate
    let non_existent_id = create_test_certificate_id(&env, 999);
    let result = client.try_is_valid_certificate(&non_existent_id);
    assert_eq!(result, Err(Ok(CertificateError::CertificateNotFound)));
}

#[test]
fn test_update_certificate_uri() {
    let (env, client, admin, student) = setup_test();
    let issuer = Address::generate(&env);
    let other_user = Address::generate(&env);

    // Grant issuer role
    let issuer_role = create_issuer_role();
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, issuer.to_val(), issuer_role.into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.grant_role(&issuer, &issuer_role);

    // Mint a certificate
    let cert_id = create_test_certificate_id(&env, 1);
    let course_id = create_test_string(&env, "CS101");
    let title = create_test_string(&env, "Computer Science 101");
    let desc = create_test_string(&env, "Intro to CS certificate");
    let original_uri = create_test_string(&env, "ipfs://original-metadata");
    let non_expiring_date = 0u64;

    // Create params
    let params = create_mint_params(MintCertificateParams {
        certificate_id: cert_id.clone(),
        course_id: course_id.clone(),
        student: student.clone(),
        title: title.clone(),
        description: desc.clone(),
        metadata_uri: original_uri.clone(),
        expiry_date: non_expiring_date,
    });

    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![
                &env,
                issuer.to_val(),
                params.clone().into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    client.mint_certificate(
        &issuer,
        &params,
    );

    // Test 1: Update URI as original issuer (should succeed)
    let new_uri_1 = create_test_string(&env, "ipfs://updated-metadata-v1");
    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_certificate_uri",
            args: vec![&env, issuer.to_val(), cert_id.to_val(), new_uri_1.to_val()],
            sub_invokes: &[],
        },
    }]);

    let update_result = client.try_update_certificate_uri(&issuer, &cert_id, &new_uri_1);
    assert!(update_result.is_ok());

    // Verify the URI was updated
    let metadata = client.verify_certificate(&cert_id);
    assert_eq!(metadata.metadata_uri, new_uri_1);

    // Test 2: Update URI as admin (should succeed)
    let new_uri_2 = create_test_string(&env, "ipfs://updated-metadata-v2");
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_certificate_uri",
            args: vec![&env, admin.to_val(), cert_id.to_val(), new_uri_2.to_val()],
            sub_invokes: &[],
        },
    }]);

    let update_result = client.try_update_certificate_uri(&admin, &cert_id, &new_uri_2);
    assert!(update_result.is_ok());

    // Verify the URI was updated again
    let metadata = client.verify_certificate(&cert_id);
    assert_eq!(metadata.metadata_uri, new_uri_2);

    // Test 3: Update URI as unauthorized user (should fail)
    let new_uri_3 = create_test_string(&env, "ipfs://unauthorized-update");
    env.mock_auths(&[MockAuth {
        address: &other_user,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_certificate_uri",
            args: vec![
                &env,
                other_user.to_val(),
                cert_id.to_val(),
                new_uri_3.to_val(),
            ],
            sub_invokes: &[],
        },
    }]);

    let update_result = client.try_update_certificate_uri(&other_user, &cert_id, &new_uri_3);
    assert_eq!(update_result, Err(Ok(CertificateError::Unauthorized)));

    // Test 4: Update with empty URI (should fail)
    let empty_uri = create_test_string(&env, "");
    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_certificate_uri",
            args: vec![&env, issuer.to_val(), cert_id.to_val(), empty_uri.to_val()],
            sub_invokes: &[],
        },
    }]);

    let update_result = client.try_update_certificate_uri(&issuer, &cert_id, &empty_uri);
    assert_eq!(update_result, Err(Ok(CertificateError::InvalidUri)));

    // Test 5: Update non-existent certificate (should fail)
    let non_existent_id = create_test_certificate_id(&env, 999);
    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_certificate_uri",
            args: vec![
                &env,
                issuer.to_val(),
                non_existent_id.to_val(),
                new_uri_1.to_val(),
            ],
            sub_invokes: &[],
        },
    }]);

    let update_result = client.try_update_certificate_uri(&issuer, &non_existent_id, &new_uri_1);
    assert_eq!(
        update_result,
        Err(Ok(CertificateError::CertificateNotFound))
    );

    // Test 6: Verify metadata history
    let history = client.get_metadata_history(&cert_id);
    assert_eq!(history.len(), 2); // We made 2 successful updates

    // Verify first update
    let first_update = &history.get(0).unwrap();
    assert_eq!(first_update.updater, issuer);
    assert_eq!(first_update.old_uri, original_uri);
    assert_eq!(first_update.new_uri, new_uri_1);

    // Verify second update
    let second_update = &history.get(1).unwrap();
    assert_eq!(second_update.updater, admin);
    assert_eq!(second_update.old_uri, new_uri_1);
    assert_eq!(second_update.new_uri, new_uri_2);
}

#[test]
fn test_reentrancy_guard_mint_certificate() {
    use std::panic;
    let (env, client, admin, student) = setup_test();
    let issuer = Address::generate(&env);
    let issuer_role = create_issuer_role();
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, issuer.to_val(), issuer_role.into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.grant_role(&issuer, &issuer_role);
    let cert_id = create_test_certificate_id(&env, 99);
    let params = create_mint_params(MintCertificateParams {
        certificate_id: cert_id.clone(),
        course_id: create_test_string(&env, "CS999"),
        student: student.clone(),
        title: create_test_string(&env, "Reentrancy Test"),
        description: create_test_string(&env, "Test"),
        metadata_uri: create_test_string(&env, "uri"),
        expiry_date: env.ledger().timestamp() + 1000000,
    });
    let result = panic::catch_unwind(|| {
        let _ = client.mint_certificate(&issuer, &params);
        // Attempt reentrant call
        let _ = client.mint_certificate(&issuer, &params);
    });
    assert!(result.is_err(), "Reentrancy was not prevented");
}
