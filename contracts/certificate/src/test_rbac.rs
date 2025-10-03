#![cfg(test)]

use crate::{
    Certificate, CertificateClient, CertificateError,
    types::{CertificateStatus, MintCertificateParams},
};
use shared::roles::{RoleLevel, Permission};
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke},
    vec, Address, BytesN, Env, String,
};

// Test helper function to create a test environment with CertificateContract
fn setup_test() -> (
    Env,
    CertificateClient<'static>,
    Address, // admin
    Address, // instructor
    Address, // student
) {
    let env = Env::default();
    let contract_id = env.register(Certificate, ());
    let admin = Address::generate(&env);
    let instructor = Address::generate(&env);
    let student = Address::generate(&env);

    // Initialize contract
    env.mock_all_auths();
    let client = CertificateClient::new(&env, &contract_id);
    client.initialize(&admin);

    // Grant Instructor role to instructor
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &contract_id,
            fn_name: "grant_role",
            args: vec![&env, instructor.to_val(), RoleLevel::Instructor.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.grant_role(&instructor, &RoleLevel::Instructor.to_u32());

    (env, client, admin, instructor, student)
}

fn create_test_string(env: &Env, text: &str) -> String {
    String::from_str(env, text)
}

fn create_test_certificate_id(env: &Env, id: u32) -> BytesN<32> {
    let mut bytes = [0u8; 32];
    bytes[0..4].copy_from_slice(&id.to_be_bytes());
    BytesN::from_array(env, &bytes)
}

fn create_mint_params(
    env: &Env,
    certificate_id: BytesN<32>,
    student: Address,
    course_id: &str,
    title: &str,
    description: &str,
    metadata_uri: &str,
    expiry_date: u64,
) -> MintCertificateParams {
    MintCertificateParams {
        certificate_id,
        course_id: create_test_string(env, course_id),
        student,
        title: create_test_string(env, title),
        description: create_test_string(env, description),
        metadata_uri: create_test_string(env, metadata_uri),
        expiry_date,
    }
}

#[test]
fn test_rbac_initialization() {
    let env = Env::default();
    let contract_id = env.register(Certificate, ());
    let admin = Address::generate(&env);

    // Test successful initialization
    env.mock_all_auths();
    let client = CertificateClient::new(&env, &contract_id);
    let result = client.initialize(&admin);
    assert!(result.is_ok());

    // Verify admin is set correctly
    let admin_result = client.get_admin();
    assert_eq!(admin_result, admin);

    // Test re-initialization (should fail)
    let result = client.initialize(&admin);
    assert_eq!(result, Err(Ok(CertificateError::AlreadyInitialized)));
}

#[test]
fn test_role_granting() {
    let (env, client, admin, instructor, student) = setup_test();

    // Grant Student role to student
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, student.to_val(), RoleLevel::Student.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);

    let result = client.grant_role(&student, &RoleLevel::Student.to_u32());
    assert!(result.is_ok());

    // Verify role was granted
    let role = client.get_role(&student);
    assert!(role.is_some());
    let role = role.unwrap();
    assert_eq!(role.level, RoleLevel::Student);
    assert!(role.has_permission(&Permission::ViewProgress));
    assert!(!role.has_permission(&Permission::IssueCertificate));
}

#[test]
fn test_role_revoking() {
    let (env, client, admin, instructor, _) = setup_test();

    // Revoke Instructor role from instructor
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "revoke_role",
            args: vec![&env, instructor.to_val()],
            sub_invokes: &[],
        },
    }]);

    let result = client.revoke_role(&instructor);
    assert!(result.is_ok());

    // Verify role was revoked
    let role = client.get_role(&instructor);
    assert!(role.is_none());
}

#[test]
fn test_permission_checks() {
    let (env, client, admin, instructor, student) = setup_test();

    // Test instructor permissions
    assert!(client.has_permission(&instructor, &0)); // IssueCertificate
    assert!(!client.has_permission(&instructor, &1)); // RevokeCertificate

    // Test student permissions (no role granted yet)
    assert!(!client.has_permission(&student, &0)); // IssueCertificate
    assert!(!client.has_permission(&student, &1)); // RevokeCertificate

    // Grant Student role to student
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, student.to_val(), RoleLevel::Student.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.grant_role(&student, &RoleLevel::Student.to_u32());

    // Test student permissions after role grant
    assert!(!client.has_permission(&student, &0)); // IssueCertificate
    assert!(client.has_permission(&student, &2)); // ViewProgress (Student permission)
}

#[test]
fn test_certificate_minting_with_rbac() {
    let (env, client, admin, instructor, student) = setup_test();

    // Create certificate parameters
    let cert_id = create_test_certificate_id(&env, 1);
    let params = create_mint_params(
        &env,
        cert_id.clone(),
        student.clone(),
        "CS101",
        "Introduction to Computer Science",
        "Basic computer science concepts",
        "ipfs://metadata/1",
        env.ledger().timestamp() + 1000000000,
    );

    // Mint certificate as instructor
    env.mock_auths(&[MockAuth {
        address: &instructor,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![&env, instructor.to_val(), params.clone().into_val(&env)],
            sub_invokes: &[],
        },
    }]);

    let result = client.mint_certificate(&instructor, &params);
    assert!(result.is_ok());

    // Verify certificate was minted
    let certificate = client.get_certificate(&cert_id);
    assert!(certificate.is_some());
    let certificate = certificate.unwrap();
    assert_eq!(certificate.student_id, student);
    assert_eq!(certificate.instructor_id, instructor);
    assert_eq!(certificate.status, CertificateStatus::Active);
}

#[test]
fn test_certificate_minting_unauthorized() {
    let (env, client, admin, instructor, student) = setup_test();

    // Grant Student role to student (should not have IssueCertificate permission)
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, student.to_val(), RoleLevel::Student.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.grant_role(&student, &RoleLevel::Student.to_u32());

    // Try to mint certificate as student (should fail)
    let cert_id = create_test_certificate_id(&env, 1);
    let params = create_mint_params(
        &env,
        cert_id.clone(),
        Address::generate(&env),
        "CS101",
        "Introduction to Computer Science",
        "Basic computer science concepts",
        "ipfs://metadata/1",
        env.ledger().timestamp() + 1000000000,
    );

    env.mock_auths(&[MockAuth {
        address: &student,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![&env, student.to_val(), params.clone().into_val(&env)],
            sub_invokes: &[],
        },
    }]);

    let result = client.mint_certificate(&student, &params);
    assert_eq!(result, Err(Ok(CertificateError::Unauthorized)));
}

#[test]
fn test_certificate_revocation_with_rbac() {
    let (env, client, admin, instructor, student) = setup_test();

    // Grant Admin role to admin (has RevokeCertificate permission)
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, admin.to_val(), RoleLevel::Admin.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.grant_role(&admin, &RoleLevel::Admin.to_u32());

    // First mint a certificate
    let cert_id = create_test_certificate_id(&env, 1);
    let params = create_mint_params(
        &env,
        cert_id.clone(),
        student.clone(),
        "CS101",
        "Introduction to Computer Science",
        "Basic computer science concepts",
        "ipfs://metadata/1",
        env.ledger().timestamp() + 1000000000,
    );

    env.mock_auths(&[MockAuth {
        address: &instructor,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![&env, instructor.to_val(), params.clone().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.mint_certificate(&instructor, &params).unwrap();

    // Revoke certificate as admin
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "revoke_certificate",
            args: vec![&env, admin.to_val(), cert_id.to_val()],
            sub_invokes: &[],
        },
    }]);

    let result = client.revoke_certificate(&admin, &cert_id);
    assert!(result.is_ok());

    // Verify certificate was revoked
    let certificate = client.get_certificate(&cert_id);
    assert!(certificate.is_some());
    let certificate = certificate.unwrap();
    assert_eq!(certificate.status, CertificateStatus::Revoked);
}

#[test]
fn test_certificate_revocation_unauthorized() {
    let (env, client, admin, instructor, student) = setup_test();

    // Grant Student role to student (should not have RevokeCertificate permission)
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, student.to_val(), RoleLevel::Student.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.grant_role(&student, &RoleLevel::Student.to_u32());

    // Try to revoke certificate as student (should fail)
    let cert_id = create_test_certificate_id(&env, 1);

    env.mock_auths(&[MockAuth {
        address: &student,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "revoke_certificate",
            args: vec![&env, student.to_val(), cert_id.to_val()],
            sub_invokes: &[],
        },
    }]);

    let result = client.revoke_certificate(&student, &cert_id);
    assert_eq!(result, Err(Ok(CertificateError::Unauthorized)));
}

#[test]
fn test_certificate_transfer_with_rbac() {
    let (env, client, admin, instructor, student) = setup_test();

    // Grant TransferCertificate permission to student
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, student.to_val(), RoleLevel::Instructor.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.grant_role(&student, &RoleLevel::Instructor.to_u32());

    // First mint a certificate to student
    let cert_id = create_test_certificate_id(&env, 1);
    let params = create_mint_params(
        &env,
        cert_id.clone(),
        student.clone(),
        "CS101",
        "Introduction to Computer Science",
        "Basic computer science concepts",
        "ipfs://metadata/1",
        env.ledger().timestamp() + 1000000000,
    );

    env.mock_auths(&[MockAuth {
        address: &instructor,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![&env, instructor.to_val(), params.clone().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.mint_certificate(&instructor, &params).unwrap();

    // Transfer certificate from student to another address
    let new_owner = Address::generate(&env);

    env.mock_auths(&[MockAuth {
        address: &student,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "transfer_certificate",
            args: vec![&env, student.to_val(), new_owner.to_val(), cert_id.to_val()],
            sub_invokes: &[],
        },
    }]);

    let result = client.transfer_certificate(&student, &new_owner, &cert_id);
    assert!(result.is_ok());

    // Verify certificate was transferred
    let certificate = client.get_certificate(&cert_id);
    assert!(certificate.is_some());
    let certificate = certificate.unwrap();
    assert_eq!(certificate.student_id, new_owner);
}

#[test]
fn test_metadata_update_with_rbac() {
    let (env, client, admin, instructor, student) = setup_test();

    // First mint a certificate
    let cert_id = create_test_certificate_id(&env, 1);
    let params = create_mint_params(
        &env,
        cert_id.clone(),
        student.clone(),
        "CS101",
        "Introduction to Computer Science",
        "Basic computer science concepts",
        "ipfs://metadata/1",
        env.ledger().timestamp() + 1000000000,
    );

    env.mock_auths(&[MockAuth {
        address: &instructor,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![&env, instructor.to_val(), params.clone().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.mint_certificate(&instructor, &params).unwrap();

    // Update metadata URI as instructor
    let new_uri = create_test_string(&env, "ipfs://metadata/updated");

    env.mock_auths(&[MockAuth {
        address: &instructor,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "update_certificate_uri",
            args: vec![&env, instructor.to_val(), cert_id.to_val(), new_uri.to_val()],
            sub_invokes: &[],
        },
    }]);

    let result = client.update_certificate_uri(&instructor, &cert_id, &new_uri);
    assert!(result.is_ok());

    // Verify metadata was updated
    let certificate = client.get_certificate(&cert_id);
    assert!(certificate.is_some());
    let certificate = certificate.unwrap();
    assert_eq!(certificate.metadata_uri, new_uri);
}

#[test]
fn test_role_hierarchy_enforcement() {
    let (env, client, admin, instructor, student) = setup_test();

    // Try to grant Admin role as Instructor (should fail due to hierarchy)
    env.mock_auths(&[MockAuth {
        address: &instructor,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, student.to_val(), RoleLevel::Admin.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);

    let result = client.grant_role(&student, &RoleLevel::Admin.to_u32());
    assert_eq!(result, Err(Ok(CertificateError::Unauthorized)));
}

#[test]
fn test_comprehensive_rbac_workflow() {
    let (env, client, admin, instructor, student) = setup_test();

    // 1. Grant different roles
    let moderator = Address::generate(&env);
    
    // Grant Moderator role
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "grant_role",
            args: vec![&env, moderator.to_val(), RoleLevel::Moderator.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    client.grant_role(&moderator, &RoleLevel::Moderator.to_u32());

    // 2. Verify role permissions
    let moderator_role = client.get_role(&moderator);
    assert!(moderator_role.is_some());
    let moderator_role = moderator_role.unwrap();
    assert_eq!(moderator_role.level, RoleLevel::Moderator);
    assert!(moderator_role.has_permission(&Permission::ViewProgress));
    assert!(moderator_role.has_permission(&Permission::UpdateProgress));
    assert!(!moderator_role.has_permission(&Permission::IssueCertificate));

    // 3. Test certificate operations with different roles
    let cert_id = create_test_certificate_id(&env, 1);
    let params = create_mint_params(
        &env,
        cert_id.clone(),
        student.clone(),
        "CS101",
        "Introduction to Computer Science",
        "Basic computer science concepts",
        "ipfs://metadata/1",
        env.ledger().timestamp() + 1000000000,
    );

    // Instructor can mint certificate
    env.mock_auths(&[MockAuth {
        address: &instructor,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![&env, instructor.to_val(), params.clone().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    let result = client.mint_certificate(&instructor, &params);
    assert!(result.is_ok());

    // Moderator cannot mint certificate
    env.mock_auths(&[MockAuth {
        address: &moderator,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "mint_certificate",
            args: vec![&env, moderator.to_val(), params.clone().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    let result = client.mint_certificate(&moderator, &params);
    assert_eq!(result, Err(Ok(CertificateError::Unauthorized)));

    // 4. Test role revocation
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &client.address,
            fn_name: "revoke_role",
            args: vec![&env, moderator.to_val()],
            sub_invokes: &[],
        },
    }]);
    let result = client.revoke_role(&moderator);
    assert!(result.is_ok());

    // Verify role was revoked
    let role = client.get_role(&moderator);
    assert!(role.is_none());
} 