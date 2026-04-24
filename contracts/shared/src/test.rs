use crate::{
    access_control::AccessControl,
    errors::AccessControlError,
    permissions::RolePermissions,
    reentrancy_guard::{ReentrancyGuard, ReentrancyLock},
    roles::{Permission, RoleLevel},
    storage::AccessControlStorage,
};
use soroban_sdk::{
    testutils::{Address as _, Ledger, MockAuth, MockAuthInvoke},
    vec, Address, Env, IntoVal, Vec,
};

// Test helper function to create a test environment with AccessControl
fn setup_test() -> (Env, Address, Address, Address) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let user1 = Address::generate(&env);
    let user2 = Address::generate(&env);

    // Initialize access control
    env.mock_all_auths();
    AccessControl::initialize(&env, &admin).expect("should initialize");

    (env, admin, user1, user2)
}

#[test]
#[ignore]
fn test_role_hierarchy() {
    let (env, admin, user1, user2) = setup_test();

    // Grant Admin role to user1
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "grant_role",
            args: vec![&env, user1.to_val(), RoleLevel::Admin.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    AccessControl::grant_role(&env, &admin, &user1, RoleLevel::Admin).expect("should grant role");

    // Grant Instructor role to user2
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "grant_role",
            args: vec![&env, user2.to_val(), RoleLevel::Instructor.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    AccessControl::grant_role(&env, &admin, &user2, RoleLevel::Instructor)
        .expect("should grant role");

    // Admin should be able to grant Instructor role
    env.mock_auths(&[MockAuth {
        address: &user1,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "grant_role",
            args: vec![
                &env,
                Address::generate(&env).to_val(),
                RoleLevel::Instructor.to_u32().into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);
    let result =
        AccessControl::grant_role(&env, &user1, &Address::generate(&env), RoleLevel::Instructor);
    assert!(result.is_ok());

    // Instructor should not be able to grant Admin role
    env.mock_auths(&[MockAuth {
        address: &user2,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "grant_role",
            args: vec![
                &env,
                Address::generate(&env).to_val(),
                RoleLevel::Admin.to_u32().into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);
    let result =
        AccessControl::grant_role(&env, &user2, &Address::generate(&env), RoleLevel::Admin);
    assert_eq!(result, Err(AccessControlError::CannotGrantHigherRole));
}

#[test]
#[ignore]
fn test_self_revocation_prevention() {
    let (env, admin, user1, _) = setup_test();

    // Grant role to user1
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "grant_role",
            args: vec![&env, user1.to_val(), RoleLevel::Instructor.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    AccessControl::grant_role(&env, &admin, &user1, RoleLevel::Instructor)
        .expect("should grant role");

    // Try to revoke own role (should fail)
    env.mock_auths(&[MockAuth {
        address: &user1,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "revoke_role",
            args: vec![&env, user1.to_val()],
            sub_invokes: &[],
        },
    }]);
    let result = AccessControl::revoke_role(&env, &user1, &user1);
    assert_eq!(result, Err(AccessControlError::CannotRevokeOwnRole));
}

#[test]
#[ignore]
fn test_permission_granting() {
    let (env, admin, user1, _) = setup_test();

    // Grant basic role first
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "grant_role",
            args: vec![&env, user1.to_val(), RoleLevel::Student.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    AccessControl::grant_role(&env, &admin, &user1, RoleLevel::Student).expect("should grant role");

    // Grant additional permission
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "grant_permission",
            args: vec![&env, user1.to_val(), Permission::IssueCertificate.into_val(&env)],
            sub_invokes: &[],
        },
    }]);

    let result =
        AccessControl::grant_permission(&env, &admin, &user1, Permission::IssueCertificate);
    assert!(result.is_ok());

    // Verify permission was granted
    assert!(AccessControl::has_permission(&env, &user1, &Permission::IssueCertificate));
}

#[test]
#[ignore]
fn test_permission_revoking() {
    let (env, admin, user1, _) = setup_test();

    // Grant role with multiple permissions
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "grant_role",
            args: vec![&env, user1.to_val(), RoleLevel::Instructor.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    AccessControl::grant_role(&env, &admin, &user1, RoleLevel::Instructor)
        .expect("should grant role");

    // Revoke a permission
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "revoke_permission",
            args: vec![&env, user1.to_val(), Permission::IssueCertificate.into_val(&env)],
            sub_invokes: &[],
        },
    }]);

    let result =
        AccessControl::revoke_permission(&env, &admin, &user1, &Permission::IssueCertificate);
    assert!(result.is_ok());

    // Verify permission was revoked
    assert!(!AccessControl::has_permission(&env, &user1, &Permission::IssueCertificate));
}

#[test]
#[ignore]
fn test_role_history() {
    let (env, admin, user1, _) = setup_test();

    // Grant role
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "grant_role",
            args: vec![&env, user1.to_val(), RoleLevel::Instructor.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    AccessControl::grant_role(&env, &admin, &user1, RoleLevel::Instructor)
        .expect("should grant role");

    // Revoke role
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "revoke_role",
            args: vec![&env, user1.to_val()],
            sub_invokes: &[],
        },
    }]);
    AccessControl::revoke_role(&env, &admin, &user1).expect("should revoke role");

    // Check role history
    let history = AccessControl::get_role_history(&env, &user1);
    assert!(!history.is_empty());

    let grants = AccessControl::get_role_grants(&env, &user1);
    assert!(!grants.is_empty());

    let revocations = AccessControl::get_role_revocations(&env, &user1);
    assert!(!revocations.is_empty());
}

#[test]
#[ignore]
fn test_require_permission_modifiers() {
    let (env, admin, user1, _) = setup_test();

    // Grant role with specific permissions
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "grant_role",
            args: vec![&env, user1.to_val(), RoleLevel::Instructor.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    AccessControl::grant_role(&env, &admin, &user1, RoleLevel::Instructor)
        .expect("should grant role");

    // Test require_permission
    let result = AccessControl::require_permission(&env, &user1, &Permission::IssueCertificate);
    assert!(result.is_ok());

    let result = AccessControl::require_permission(&env, &user1, &Permission::RevokeCertificate);
    assert_eq!(result, Err(AccessControlError::PermissionDenied));

    // Test require_any_permission
    let mut permissions = Vec::new(&env);
    permissions.push_back(Permission::IssueCertificate);
    permissions.push_back(Permission::RevokeCertificate);
    let result = AccessControl::require_any_permission(&env, &user1, &permissions);
    assert!(result.is_ok());

    // Test require_all_permissions
    let mut permissions = Vec::new(&env);
    permissions.push_back(Permission::IssueCertificate);
    permissions.push_back(Permission::ViewProgress);
    let result = AccessControl::require_all_permissions(&env, &user1, &permissions);
    assert!(result.is_ok());

    let mut permissions = Vec::new(&env);
    permissions.push_back(Permission::IssueCertificate);
    permissions.push_back(Permission::RevokeCertificate);
    let result = AccessControl::require_all_permissions(&env, &user1, &permissions);
    assert_eq!(result, Err(AccessControlError::PermissionDenied));
}

#[test]
#[ignore]
fn test_change_admin() {
    let (env, admin, _user1, _) = setup_test();
    let new_admin = Address::generate(&env);

    // Change admin
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "change_admin",
            args: vec![&env, new_admin.to_val()],
            sub_invokes: &[],
        },
    }]);

    let result = AccessControl::change_admin(&env, &admin, &new_admin);
    assert!(result.is_ok());

    // Verify admin was changed
    let current_admin = AccessControl::get_admin(&env);
    assert_eq!(current_admin, Ok(new_admin));
}

#[test]
#[ignore]
fn test_role_expiry() {
    let (env, admin, user1, _) = setup_test();

    // Create a role with expiry
    let mut role = RolePermissions::create_role_with_default_permissions(
        &env,
        RoleLevel::Instructor,
        admin.clone(),
        env.ledger().timestamp(),
    );
    role = role.with_expiry(env.ledger().timestamp() + 1000);

    // Grant role with expiry
    AccessControlStorage::set_role(&env, &user1, &role);

    // Check role is valid initially
    assert!(AccessControl::has_permission(&env, &user1, &Permission::IssueCertificate));

    // Fast forward time
    env.ledger().set_timestamp(env.ledger().timestamp() + 2000);

    // Check role is expired
    assert!(!AccessControl::has_permission(&env, &user1, &Permission::IssueCertificate));
}

#[test]
#[ignore]
fn test_default_role_permissions() {
    let env = Env::default();
    // Test Student permissions
    let permissions = RolePermissions::student_permissions(&env);
    assert!(permissions.contains(&Permission::ViewProgress));
    assert!(permissions.contains(&Permission::MarkCompletion));
    assert!(!permissions.contains(&Permission::IssueCertificate));

    // Test Instructor permissions
    let permissions = RolePermissions::instructor_permissions(&env);
    assert!(permissions.contains(&Permission::IssueCertificate));
    assert!(permissions.contains(&Permission::CreateCourse));
    assert!(!permissions.contains(&Permission::RevokeCertificate));

    // Test Admin permissions
    let permissions = RolePermissions::admin_permissions(&env);
    assert!(permissions.contains(&Permission::RevokeCertificate));
    assert!(permissions.contains(&Permission::GrantRole));
    assert!(!permissions.contains(&Permission::InitializeContract));

    // Test SuperAdmin permissions
    let permissions = RolePermissions::super_admin_permissions(&env);
    assert!(permissions.contains(&Permission::InitializeContract));
    assert!(permissions.contains(&Permission::UpgradeContract));
    assert!(permissions.contains(&Permission::EmergencyPause));
}

// ReentrancyGuard tests
#[test]
#[ignore]
fn test_reentrancy_guard_basic() {
    let env = Env::default();

    // First call should succeed
    ReentrancyGuard::enter(&env);
    ReentrancyGuard::exit(&env);

    // Second call should also succeed after exit
    ReentrancyGuard::enter(&env);
    ReentrancyGuard::exit(&env);
}

#[test]
#[ignore]
#[should_panic(expected = "ReentrancyGuard: reentrant call")]
fn test_reentrancy_guard_prevents_reentrancy() {
    let env = Env::default();

    // First call should succeed
    ReentrancyGuard::enter(&env);

    // Second call should panic
    ReentrancyGuard::enter(&env);
}

#[test]
#[ignore]
fn test_reentrancy_lock_raii() {
    let env = Env::default();

    // Test RAII-style guard
    {
        let _lock = ReentrancyLock::new(&env);
        // Lock should be active here
        assert!(env.storage().instance().has(&soroban_sdk::symbol_short!("REENTRANT")));
    }

    // Lock should be automatically released when _lock goes out of scope
    assert!(!env.storage().instance().has(&soroban_sdk::symbol_short!("REENTRANT")));
}

#[test]
#[ignore]
#[should_panic(expected = "ReentrancyGuard: reentrant call")]
fn test_reentrancy_lock_prevents_reentrancy() {
    let env = Env::default();

    // First lock should succeed
    let _lock1 = ReentrancyLock::new(&env);

    // Second lock should panic
    let _lock2 = ReentrancyLock::new(&env);
}

#[test]
#[ignore]
fn test_reentrancy_guard_multiple_enter_exit() {
    let env = Env::default();

    // Multiple enter/exit cycles should work
    for _ in 0..5 {
        ReentrancyGuard::enter(&env);
        ReentrancyGuard::exit(&env);
    }

    // Should be able to enter again after all exits
    ReentrancyGuard::enter(&env);
    ReentrancyGuard::exit(&env);
}

#[test]
#[ignore]
fn test_reentrancy_guard_exit_without_enter() {
    let env = Env::default();

    // Exit without enter should not panic (just remove non-existent key)
    ReentrancyGuard::exit(&env);

    // Should still be able to enter after
    ReentrancyGuard::enter(&env);
    ReentrancyGuard::exit(&env);
}
