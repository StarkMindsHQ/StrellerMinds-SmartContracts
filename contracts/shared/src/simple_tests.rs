#![cfg(test)]

use crate::{
    roles::{Permission, RoleLevel},
    permissions::RolePermissions,
};
use soroban_sdk::{Env, Vec};

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

#[test]
fn test_role_level_conversions() {
    // Test conversion from u32
    assert_eq!(RoleLevel::from_u32(1), Some(RoleLevel::Student));
    assert_eq!(RoleLevel::from_u32(2), Some(RoleLevel::Moderator));
    assert_eq!(RoleLevel::from_u32(3), Some(RoleLevel::Instructor));
    assert_eq!(RoleLevel::from_u32(4), Some(RoleLevel::Admin));
    assert_eq!(RoleLevel::from_u32(5), Some(RoleLevel::SuperAdmin));
    assert_eq!(RoleLevel::from_u32(99), None);

    // Test conversion to u32
    assert_eq!(RoleLevel::Student.to_u32(), 1);
    assert_eq!(RoleLevel::Moderator.to_u32(), 2);
    assert_eq!(RoleLevel::Instructor.to_u32(), 3);
    assert_eq!(RoleLevel::Admin.to_u32(), 4);
    assert_eq!(RoleLevel::SuperAdmin.to_u32(), 5);
}