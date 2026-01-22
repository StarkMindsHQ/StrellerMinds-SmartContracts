use crate::roles::{Permission, Role, RoleLevel};
use soroban_sdk::{Env, Vec};

/// Predefined role permissions for different user types
pub struct RolePermissions;

impl RolePermissions {
    /// Get default permissions for a Student role
    pub fn student_permissions(env: &Env) -> Vec<Permission> {
        let mut permissions = Vec::new(env);
        permissions.push_back(Permission::ViewProgress);
        permissions.push_back(Permission::MarkCompletion);
        permissions
    }

    /// Get default permissions for a Moderator role
    pub fn moderator_permissions(env: &Env) -> Vec<Permission> {
        let mut permissions = Vec::new(env);
        permissions.push_back(Permission::ViewProgress);
        permissions.push_back(Permission::UpdateProgress);
        permissions.push_back(Permission::MarkCompletion);
        permissions.push_back(Permission::ViewAllCertificates);
        permissions.push_back(Permission::ViewAllCourses);
        permissions
    }

    /// Get default permissions for an Instructor role
    pub fn instructor_permissions(env: &Env) -> Vec<Permission> {
        let mut permissions = Vec::new(env);
        permissions.push_back(Permission::IssueCertificate);
        permissions.push_back(Permission::UpdateCertificateMetadata);
        permissions.push_back(Permission::CreateCourse);
        permissions.push_back(Permission::UpdateCourse);
        permissions.push_back(Permission::EnrollStudent);
        permissions.push_back(Permission::UnenrollStudent);
        permissions.push_back(Permission::UpdateProgress);
        permissions.push_back(Permission::ViewProgress);
        permissions.push_back(Permission::MarkCompletion);
        permissions.push_back(Permission::ViewAllCertificates);
        permissions.push_back(Permission::ViewAllCourses);
        permissions
    }

    /// Get default permissions for an Admin role
    pub fn admin_permissions(env: &Env) -> Vec<Permission> {
        let mut permissions = Vec::new(env);
        permissions.push_back(Permission::IssueCertificate);
        permissions.push_back(Permission::RevokeCertificate);
        permissions.push_back(Permission::TransferCertificate);
        permissions.push_back(Permission::UpdateCertificateMetadata);
        permissions.push_back(Permission::CreateCourse);
        permissions.push_back(Permission::UpdateCourse);
        permissions.push_back(Permission::DeleteCourse);
        permissions.push_back(Permission::EnrollStudent);
        permissions.push_back(Permission::UnenrollStudent);
        permissions.push_back(Permission::UpdateProgress);
        permissions.push_back(Permission::ViewProgress);
        permissions.push_back(Permission::MarkCompletion);
        permissions.push_back(Permission::GrantRole);
        permissions.push_back(Permission::RevokeRole);
        permissions.push_back(Permission::TransferRole);
        permissions.push_back(Permission::BatchMintCertificates);
        permissions.push_back(Permission::BatchRevokeCertificates);
        permissions.push_back(Permission::ViewAllCertificates);
        permissions.push_back(Permission::ViewAllCourses);
        permissions.push_back(Permission::ViewAllUsers);
        permissions.push_back(Permission::ViewSystemStats);
        permissions
    }

    /// Get default permissions for a SuperAdmin role
    pub fn super_admin_permissions(env: &Env) -> Vec<Permission> {
        let mut permissions = Vec::new(env);
        permissions.push_back(Permission::IssueCertificate);
        permissions.push_back(Permission::RevokeCertificate);
        permissions.push_back(Permission::TransferCertificate);
        permissions.push_back(Permission::UpdateCertificateMetadata);
        permissions.push_back(Permission::CreateCourse);
        permissions.push_back(Permission::UpdateCourse);
        permissions.push_back(Permission::DeleteCourse);
        permissions.push_back(Permission::EnrollStudent);
        permissions.push_back(Permission::UnenrollStudent);
        permissions.push_back(Permission::UpdateProgress);
        permissions.push_back(Permission::ViewProgress);
        permissions.push_back(Permission::MarkCompletion);
        permissions.push_back(Permission::GrantRole);
        permissions.push_back(Permission::RevokeRole);
        permissions.push_back(Permission::TransferRole);
        permissions.push_back(Permission::InitializeContract);
        permissions.push_back(Permission::UpgradeContract);
        permissions.push_back(Permission::EmergencyPause);
        permissions.push_back(Permission::EmergencyResume);
        permissions.push_back(Permission::MintTokens);
        permissions.push_back(Permission::BurnTokens);
        permissions.push_back(Permission::TransferTokens);
        permissions.push_back(Permission::BatchMintCertificates);
        permissions.push_back(Permission::BatchRevokeCertificates);
        permissions.push_back(Permission::ViewAllCertificates);
        permissions.push_back(Permission::ViewAllCourses);
        permissions.push_back(Permission::ViewAllUsers);
        permissions.push_back(Permission::ViewSystemStats);
        permissions
    }

    /// Get permissions for a specific role level
    pub fn get_permissions_for_level(env: &Env, level: &RoleLevel) -> Vec<Permission> {
        match level {
            RoleLevel::Student => Self::student_permissions(env),
            RoleLevel::Moderator => Self::moderator_permissions(env),
            RoleLevel::Instructor => Self::instructor_permissions(env),
            RoleLevel::Admin => Self::admin_permissions(env),
            RoleLevel::SuperAdmin => Self::super_admin_permissions(env),
        }
    }

    /// Resolve all permissions for a role, including inherited ones
    pub fn resolve_all_permissions(env: &Env, role: &Role) -> Vec<Permission> {
        let mut all_permissions = role.permissions.clone();

        for inherited_level in role.inherited_roles.iter() {
            let inherited_permissions = Self::get_permissions_for_level(env, &inherited_level);
            for p in inherited_permissions.iter() {
                if !all_permissions.contains(&p) {
                    all_permissions.push_back(p);
                }
            }
        }
        all_permissions
    }

    /// Create a role with default permissions for a given level
    pub fn create_role_with_default_permissions(
        env: &Env,
        level: RoleLevel,
        granted_by: soroban_sdk::Address,
        granted_at: u64,
    ) -> Role {
        let permissions = Self::get_permissions_for_level(env, &level);
        Role::new(level, permissions, granted_by, granted_at)
    }

    /// Check if a role has a specific permission
    pub fn has_permission(role: &Role, permission: &Permission) -> bool {
        role.has_permission(permission)
    }

    /// Check if a role has any of the specified permissions
    pub fn has_any_permission(role: &Role, permissions: &Vec<Permission>) -> bool {
        role.has_any_permission(permissions)
    }

    /// Check if a role has all of the specified permissions
    pub fn has_all_permissions(role: &Role, permissions: &Vec<Permission>) -> bool {
        role.has_all_permissions(permissions)
    }

    /// Get all permissions that a role has
    pub fn get_role_permissions(role: &Role) -> &Vec<Permission> {
        &role.permissions
    }

    /// Add a permission to a role
    pub fn add_permission(role: &mut Role, permission: Permission) {
        if !role.permissions.contains(&permission) {
            role.permissions.push_back(permission);
        }
    }

    /// Remove a permission from a role
    pub fn remove_permission(role: &mut Role, permission: &Permission) {
        let mut new_permissions = Vec::new(&role.granted_by.env());
        for p in role.permissions.iter() {
            if p != *permission {
                new_permissions.push_back(p);
            }
        }
        role.permissions = new_permissions;
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
