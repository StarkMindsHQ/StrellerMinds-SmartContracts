use crate::roles::{Role, RoleLevel, Permission};
use soroban_sdk::{vec, Vec};

/// Predefined role permissions for different user types
pub struct RolePermissions;

impl RolePermissions {
    /// Get default permissions for a Student role
    pub fn student_permissions() -> Vec<Permission> {
        vec![
            Permission::ViewProgress,
            Permission::MarkCompletion,
        ]
    }

    /// Get default permissions for a Moderator role
    pub fn moderator_permissions() -> Vec<Permission> {
        vec![
            Permission::ViewProgress,
            Permission::UpdateProgress,
            Permission::MarkCompletion,
            Permission::ViewAllCertificates,
            Permission::ViewAllCourses,
        ]
    }

    /// Get default permissions for an Instructor role
    pub fn instructor_permissions() -> Vec<Permission> {
        vec![
            Permission::IssueCertificate,
            Permission::UpdateCertificateMetadata,
            Permission::CreateCourse,
            Permission::UpdateCourse,
            Permission::EnrollStudent,
            Permission::UnenrollStudent,
            Permission::UpdateProgress,
            Permission::ViewProgress,
            Permission::MarkCompletion,
            Permission::ViewAllCertificates,
            Permission::ViewAllCourses,
        ]
    }

    /// Get default permissions for an Admin role
    pub fn admin_permissions() -> Vec<Permission> {
        vec![
            Permission::IssueCertificate,
            Permission::RevokeCertificate,
            Permission::TransferCertificate,
            Permission::UpdateCertificateMetadata,
            Permission::CreateCourse,
            Permission::UpdateCourse,
            Permission::DeleteCourse,
            Permission::EnrollStudent,
            Permission::UnenrollStudent,
            Permission::UpdateProgress,
            Permission::ViewProgress,
            Permission::MarkCompletion,
            Permission::GrantRole,
            Permission::RevokeRole,
            Permission::TransferRole,
            Permission::BatchMintCertificates,
            Permission::BatchRevokeCertificates,
            Permission::ViewAllCertificates,
            Permission::ViewAllCourses,
            Permission::ViewAllUsers,
            Permission::ViewSystemStats,
        ]
    }

    /// Get default permissions for a SuperAdmin role
    pub fn super_admin_permissions() -> Vec<Permission> {
        vec![
            Permission::IssueCertificate,
            Permission::RevokeCertificate,
            Permission::TransferCertificate,
            Permission::UpdateCertificateMetadata,
            Permission::CreateCourse,
            Permission::UpdateCourse,
            Permission::DeleteCourse,
            Permission::EnrollStudent,
            Permission::UnenrollStudent,
            Permission::UpdateProgress,
            Permission::ViewProgress,
            Permission::MarkCompletion,
            Permission::GrantRole,
            Permission::RevokeRole,
            Permission::TransferRole,
            Permission::InitializeContract,
            Permission::UpgradeContract,
            Permission::EmergencyPause,
            Permission::EmergencyResume,
            Permission::MintTokens,
            Permission::BurnTokens,
            Permission::TransferTokens,
            Permission::BatchMintCertificates,
            Permission::BatchRevokeCertificates,
            Permission::ViewAllCertificates,
            Permission::ViewAllCourses,
            Permission::ViewAllUsers,
            Permission::ViewSystemStats,
        ]
    }

    /// Get permissions for a specific role level
    pub fn get_permissions_for_level(level: &RoleLevel) -> Vec<Permission> {
        match level {
            RoleLevel::Student => Self::student_permissions(),
            RoleLevel::Moderator => Self::moderator_permissions(),
            RoleLevel::Instructor => Self::instructor_permissions(),
            RoleLevel::Admin => Self::admin_permissions(),
            RoleLevel::SuperAdmin => Self::super_admin_permissions(),
        }
    }

    /// Create a role with default permissions for a given level
    pub fn create_role_with_default_permissions(
        level: RoleLevel,
        granted_by: soroban_sdk::Address,
        granted_at: u64,
    ) -> Role {
        let permissions = Self::get_permissions_for_level(&level);
        Role::new(level, permissions, granted_by, granted_at)
    }

    /// Check if a role has a specific permission
    pub fn has_permission(role: &Role, permission: &Permission) -> bool {
        role.has_permission(permission)
    }

    /// Check if a role has any of the specified permissions
    pub fn has_any_permission(role: &Role, permissions: &[Permission]) -> bool {
        role.has_any_permission(permissions)
    }

    /// Check if a role has all of the specified permissions
    pub fn has_all_permissions(role: &Role, permissions: &[Permission]) -> bool {
        role.has_all_permissions(permissions)
    }

    /// Get all permissions that a role has
    pub fn get_role_permissions(role: &Role) -> &[Permission] {
        &role.permissions
    }

    /// Add a permission to a role
    pub fn add_permission(role: &mut Role, permission: Permission) {
        if !role.permissions.contains(&permission) {
            role.permissions.push(permission);
        }
    }

    /// Remove a permission from a role
    pub fn remove_permission(role: &mut Role, permission: &Permission) {
        role.permissions.retain(|p| p != permission);
    }

    /// Check if a role can grant another role (hierarchy check)
    pub fn can_grant_role(granter_role: &Role, target_role_level: &RoleLevel) -> bool {
        granter_role.level.can_grant(target_role_level)
    }

    /// Check if a role can revoke another role (hierarchy check)
    pub fn can_revoke_role(revoker_role: &Role, target_role_level: &RoleLevel) -> bool {
        revoker_role.level.can_revoke(target_role_level)
    }
} 