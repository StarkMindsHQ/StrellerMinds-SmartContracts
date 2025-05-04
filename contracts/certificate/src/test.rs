#![cfg(test)]
use crate::types::{CertificateStatus, Permission, Role};
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
    let contract_id = env.register(Certificate, {});
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

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register(Certificate, {});
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

    // Mint certificate
    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![
                &env,
                cert_id.to_val(),
                course_id.to_val(),
                student.to_val(),
                title.to_val(),
                desc.to_val(),
                uri.to_val(),
                expiry_date.into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    let mint_result = client.try_mint_certificate(
        &issuer,
        &cert_id,
        &course_id,
        &student,
        &title,
        &desc,
        &uri,
        &expiry_date,
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
                cert_id.to_val(),
                course_id.to_val(),
                student.to_val(),
                title.to_val(),
                desc.to_val(),
                uri.to_val(),
                expiry_date.into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    let mint_fail = client.try_mint_certificate(
        &issuer,
        &cert_id,
        &course_id,
        &student,
        &title,
        &desc,
        &uri,
        &expiry_date,
    );
    assert_eq!(
        mint_fail,
        Err(Ok(CertificateError::CertificateAlreadyExists))
    );

    // Try minting with invalid metadata
    let empty_title = create_test_string(&env, "");
    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![
                &env,
                create_test_certificate_id(&env, 2).to_val(),
                course_id.to_val(),
                student.to_val(),
                empty_title.to_val(),
                desc.to_val(),
                uri.to_val(),
                expiry_date.into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    let mint_invalid = client.try_mint_certificate(
        &issuer,
        &create_test_certificate_id(&env, 2),
        &course_id,
        &student,
        &empty_title,
        &desc,
        &uri,
        &expiry_date,
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

    // Mint certificate
    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![
                &env,
                cert_id.to_val(),
                course_id.to_val(),
                student.to_val(),
                title.to_val(),
                desc.to_val(),
                uri.to_val(),
                expiry_date.into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    client.mint_certificate(
        &issuer,
        &cert_id,
        &course_id,
        &student,
        &title,
        &desc,
        &uri,
        &expiry_date,
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

    // Mint certificates for first student
    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![
                &env,
                cert_id1.to_val(),
                course_id.to_val(),
                student.to_val(),
                title.to_val(),
                desc.to_val(),
                uri.to_val(),
                expiry_date.into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    client.mint_certificate(
        &issuer,
        &cert_id1,
        &course_id,
        &student,
        &title,
        &desc,
        &uri,
        &expiry_date,
    );

    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![
                &env,
                cert_id2.to_val(),
                course_id.to_val(),
                student.to_val(),
                title.to_val(),
                desc.to_val(),
                uri.to_val(),
                expiry_date.into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);
    client.mint_certificate(
        &issuer,
        &cert_id2,
        &course_id,
        &student,
        &title,
        &desc,
        &uri,
        &expiry_date,
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

    // Mint certificate with future expiry date
    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![
                &env,
                cert_id.to_val(),
                course_id.to_val(),
                student.to_val(),
                title.to_val(),
                desc.to_val(),
                uri.to_val(),
                expiry_date.into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    client.mint_certificate(
        &issuer,
        &cert_id,
        &course_id,
        &student,
        &title,
        &desc,
        &uri,
        &expiry_date,
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

    env.mock_auths(&[MockAuth {
        address: &issuer,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![
                &env,
                permanent_cert_id.to_val(),
                course_id.to_val(),
                student.to_val(),
                title.to_val(),
                desc.to_val(),
                uri.to_val(),
                permanent_expiry.into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    client.mint_certificate(
        &issuer,
        &permanent_cert_id,
        &course_id,
        &student,
        &title,
        &desc,
        &uri,
        &permanent_expiry,
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
