#![cfg(test)]

use crate::{
    access_control::AccessControl,
    errors::AccessControlError,
    roles::{Role, RoleLevel, Permission},
    permissions::RolePermissions,
    storage::AccessControlStorage,
    events::AccessControlEvents,
    reentrancy_guard::{ReentrancyGuard, ReentrancyLock},
};
use soroban_sdk::{
    testutils::{Address as _, MockAuth, MockAuthInvoke, Ledger},
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
    AccessControl::initialize(&env, &admin).unwrap();

    (env, admin, user1, user2)
}

#[test]
fn test_initialize() {
    let env = Env::default();
    let admin = Address::generate(&env);

    // Test successful initialization
    env.mock_all_auths();
    let result = AccessControl::initialize(&env, &admin);
    assert!(result.is_ok());

    // Verify admin is set correctly
    let admin_result = AccessControl::get_admin(&env);
    assert_eq!(admin_result, Ok(admin.clone()));

    // Test re-initialization (should fail)
    let result = AccessControl::initialize(&env, &admin);
    assert_eq!(result, Err(AccessControlError::AlreadyInitialized));
}

#[test]
fn test_grant_role() {
    let (env, admin, user1, _) = setup_test();

    // Grant Instructor role to user1
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env), // Mock contract address
            fn_name: "grant_role",
            args: vec![&env, user1.to_val(), RoleLevel::Instructor.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);

    let result = AccessControl::grant_role(&env, &admin, &user1, RoleLevel::Instructor);
    assert!(result.is_ok());

    // Verify role was granted
    let role = AccessControl::get_role(&env, &user1);
    assert!(role.is_some());
    let role = role.unwrap();
    assert_eq!(role.level, RoleLevel::Instructor);
    assert!(role.has_permission(&Permission::IssueCertificate));
}

#[test]
fn test_grant_custom_role() {
    let (env, admin, user1, _) = setup_test();

      let mut custom_permissions = Vec::new(&env);
      custom_permissions.push_back(Permission::IssueCertificate);
      custom_permissions.push_back(Permission::ViewProgress);

    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "grant_custom_role",
            args: vec![
                &env,
                user1.to_val(),
                RoleLevel::Instructor.to_u32().into_val(&env),
                custom_permissions.clone().into_val(&env),
            ],
            sub_invokes: &[],
        },
    }]);

    let result = AccessControl::grant_custom_role(
        &env,
        &admin,
        &user1,
        RoleLevel::Instructor,
        custom_permissions.clone(),
    );
    assert!(result.is_ok());

    // Verify custom role was granted
    let role = AccessControl::get_role(&env, &user1);
    assert!(role.is_some());
    let role = role.unwrap();
    assert_eq!(role.level, RoleLevel::Instructor);
    assert!(role.has_permission(&Permission::IssueCertificate));
    assert!(role.has_permission(&Permission::ViewProgress));
    assert!(!role.has_permission(&Permission::RevokeCertificate));
}

#[test]
fn test_revoke_role() {
    let (env, admin, user1, _) = setup_test();

    // Grant role first
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "grant_role",
            args: vec![&env, user1.to_val(), RoleLevel::Instructor.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    AccessControl::grant_role(&env, &admin, &user1, RoleLevel::Instructor).unwrap();

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

    let result = AccessControl::revoke_role(&env, &admin, &user1);
    assert!(result.is_ok());

    // Verify role was revoked
    let role = AccessControl::get_role(&env, &user1);
    assert!(role.is_none());
}

#[test]
fn test_transfer_role() {
    let (env, admin, user1, user2) = setup_test();

    // Grant role to user1 first
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "grant_role",
            args: vec![&env, user1.to_val(), RoleLevel::Instructor.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    AccessControl::grant_role(&env, &admin, &user1, RoleLevel::Instructor).unwrap();

    // Transfer role from user1 to user2
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "transfer_role",
            args: vec![&env, user1.to_val(), user2.to_val()],
            sub_invokes: &[],
        },
    }]);

    let result = AccessControl::transfer_role(&env, &admin, &user1, &user2);
    assert!(result.is_ok());

    // Verify role was transferred
    let role1 = AccessControl::get_role(&env, &user1);
    assert!(role1.is_none());

    let role2 = AccessControl::get_role(&env, &user2);
    assert!(role2.is_some());
    let role2 = role2.unwrap();
    assert_eq!(role2.level, RoleLevel::Instructor);
}

#[test]
fn test_permission_checks() {
    let (env, admin, user1, _) = setup_test();

    // Grant Instructor role to user1
    env.mock_auths(&[MockAuth {
        address: &admin,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "grant_role",
            args: vec![&env, user1.to_val(), RoleLevel::Instructor.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    AccessControl::grant_role(&env, &admin, &user1, RoleLevel::Instructor).unwrap();

    // Test permission checks
    assert!(AccessControl::has_permission(&env, &user1, &Permission::IssueCertificate));
    assert!(!AccessControl::has_permission(&env, &user1, &Permission::RevokeCertificate));
    assert!(AccessControl::has_permission(&env, &admin, &Permission::GrantRole));

    // Test multiple permission checks
    let mut permissions = Vec::new(&env);
    permissions.push_back(Permission::IssueCertificate);
    permissions.push_back(Permission::ViewProgress);
    assert!(AccessControl::has_any_permission(&env, &user1, &permissions));
    assert!(AccessControl::has_all_permissions(&env, &user1, &permissions));
}

#[test]
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
    AccessControl::grant_role(&env, &admin, &user1, RoleLevel::Admin).unwrap();

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
    AccessControl::grant_role(&env, &admin, &user2, RoleLevel::Instructor).unwrap();

    // Admin should be able to grant Instructor role
    env.mock_auths(&[MockAuth {
        address: &user1,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "grant_role",
            args: vec![&env, Address::generate(&env).to_val(), RoleLevel::Instructor.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    let result = AccessControl::grant_role(&env, &user1, &Address::generate(&env), RoleLevel::Instructor);
    assert!(result.is_ok());

    // Instructor should not be able to grant Admin role
    env.mock_auths(&[MockAuth {
        address: &user2,
        invoke: &MockAuthInvoke {
            contract: &Address::generate(&env),
            fn_name: "grant_role",
            args: vec![&env, Address::generate(&env).to_val(), RoleLevel::Admin.to_u32().into_val(&env)],
            sub_invokes: &[],
        },
    }]);
    let result = AccessControl::grant_role(&env, &user2, &Address::generate(&env), RoleLevel::Admin);
    assert_eq!(result, Err(AccessControlError::CannotGrantHigherRole));
}

#[test]
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
    AccessControl::grant_role(&env, &admin, &user1, RoleLevel::Instructor).unwrap();

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
    AccessControl::grant_role(&env, &admin, &user1, RoleLevel::Student).unwrap();

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

    let result = AccessControl::grant_permission(&env, &admin, &user1, Permission::IssueCertificate);
    assert!(result.is_ok());

    // Verify permission was granted
    assert!(AccessControl::has_permission(&env, &user1, &Permission::IssueCertificate));
}

#[test]
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
    AccessControl::grant_role(&env, &admin, &user1, RoleLevel::Instructor).unwrap();

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

    let result = AccessControl::revoke_permission(&env, &admin, &user1, &Permission::IssueCertificate);
    assert!(result.is_ok());

    // Verify permission was revoked
    assert!(!AccessControl::has_permission(&env, &user1, &Permission::IssueCertificate));
}

#[test]
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
    AccessControl::grant_role(&env, &admin, &user1, RoleLevel::Instructor).unwrap();

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
    AccessControl::revoke_role(&env, &admin, &user1).unwrap();

    // Check role history
    let history = AccessControl::get_role_history(&env, &user1);
    assert!(!history.is_empty());

    let grants = AccessControl::get_role_grants(&env, &user1);
    assert!(!grants.is_empty());

    let revocations = AccessControl::get_role_revocations(&env, &user1);
    assert!(!revocations.is_empty());
}

#[test]
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
    AccessControl::grant_role(&env, &admin, &user1, RoleLevel::Instructor).unwrap();

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
fn test_change_admin() {
    let (env, admin, user1, _) = setup_test();
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
#[should_panic(expected = "ReentrancyGuard: reentrant call")]
fn test_reentrancy_guard_prevents_reentrancy() {
    let env = Env::default();
    
    // First call should succeed
    ReentrancyGuard::enter(&env);
    
    // Second call should panic
    ReentrancyGuard::enter(&env);
}

#[test]
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
#[should_panic(expected = "ReentrancyGuard: reentrant call")]
fn test_reentrancy_lock_prevents_reentrancy() {
    let env = Env::default();
    
    // First lock should succeed
    let _lock1 = ReentrancyLock::new(&env);
    
    // Second lock should panic
    let _lock2 = ReentrancyLock::new(&env);
}

#[test]
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
fn test_reentrancy_guard_exit_without_enter() {
    let env = Env::default();
    
    // Exit without enter should not panic (just remove non-existent key)
    ReentrancyGuard::exit(&env);
    
    // Should still be able to enter after
    ReentrancyGuard::enter(&env);
    ReentrancyGuard::exit(&env);
} 