use crate::errors::AccessControlError;
use crate::events::AccessControlEvents;
use crate::permissions::RolePermissions;
use crate::roles::{Permission, Role, RoleLevel};
use crate::storage::AccessControlStorage;
use soroban_sdk::{Address, Env, Vec};

/// OpenZeppelin-style AccessControl implementation
pub struct AccessControl;

impl AccessControl {
    /// Initialize the access control system
    pub fn initialize(env: &Env, admin: &Address) -> Result<(), AccessControlError> {
        if AccessControlStorage::is_initialized(env) {
            return Err(AccessControlError::AlreadyInitialized);
        }

        // Require authorization from the admin
        admin.require_auth();

        // Store admin address and mark as initialized
        AccessControlStorage::set_admin(env, admin);
        AccessControlStorage::set_initialized(env);

        // Grant SuperAdmin role to the initial admin
        let super_admin_role = RolePermissions::create_role_with_default_permissions(
            &env,
            RoleLevel::SuperAdmin,
            admin.clone(),
            env.ledger().timestamp(),
        );
        AccessControlStorage::set_role(env, admin, &super_admin_role);

        // Emit initialization event
        AccessControlEvents::emit_contract_initialized(env, admin);

        Ok(())
    }

    /// Get the current admin address
    pub fn get_admin(env: &Env) -> Result<Address, AccessControlError> {
        if !AccessControlStorage::is_initialized(env) {
            return Err(AccessControlError::NotInitialized);
        }
        Ok(AccessControlStorage::get_admin(env))
    }

    /// Grant a role to a user
    pub fn grant_role(
        env: &Env,
        granter: &Address,
        user: &Address,
        role_level: RoleLevel,
    ) -> Result<(), AccessControlError> {
        // Validate granter has permission
        let granter_role = AccessControlStorage::validate_user_role(env, granter)?;

        if !granter_role.has_permission(&Permission::GrantRole) {
            AccessControlEvents::emit_access_denied(env, granter, &Permission::GrantRole);
            return Err(AccessControlError::PermissionDenied);
        }

        // Check role hierarchy
        if !granter_role.level.can_grant(&role_level) {
            AccessControlEvents::emit_hierarchy_violation(env, granter, user, &role_level);
            return Err(AccessControlError::CannotGrantHigherRole);
        }

        // Create role with default permissions
        let role = RolePermissions::create_role_with_default_permissions(
            &env,
            role_level,
            granter.clone(),
            env.ledger().timestamp(),
        );

        // Store role
        AccessControlStorage::set_role(env, user, &role);
        AccessControlStorage::add_role_grant(env, user, &role);

        // Emit event
        AccessControlEvents::emit_role_granted(env, granter, user, &role);

        Ok(())
    }

    /// Grant a custom role with specific permissions
    pub fn grant_custom_role(
        env: &Env,
        granter: &Address,
        user: &Address,
        role_level: RoleLevel,
        permissions: Vec<Permission>,
    ) -> Result<(), AccessControlError> {
        // Validate granter has permission
        let granter_role = AccessControlStorage::validate_user_role(env, granter)?;

        if !granter_role.has_permission(&Permission::GrantRole) {
            AccessControlEvents::emit_access_denied(env, granter, &Permission::GrantRole);
            return Err(AccessControlError::PermissionDenied);
        }

        // Check role hierarchy
        if !granter_role.level.can_grant(&role_level) {
            AccessControlEvents::emit_hierarchy_violation(env, granter, user, &role_level);
            return Err(AccessControlError::CannotGrantHigherRole);
        }

        // Create custom role
        let role = Role::new(
            role_level,
            permissions,
            granter.clone(),
            env.ledger().timestamp(),
        );

        // Store role
        AccessControlStorage::set_role(env, user, &role);
        AccessControlStorage::add_role_grant(env, user, &role);

        // Emit event
        AccessControlEvents::emit_role_granted(env, granter, user, &role);

        Ok(())
    }

    /// Revoke a role from a user
    pub fn revoke_role(
        env: &Env,
        revoker: &Address,
        user: &Address,
    ) -> Result<(), AccessControlError> {
        // Validate revoker has permission
        let revoker_role = AccessControlStorage::validate_user_role(env, revoker)?;

        if !revoker_role.has_permission(&Permission::RevokeRole) {
            AccessControlEvents::emit_access_denied(env, revoker, &Permission::RevokeRole);
            return Err(AccessControlError::PermissionDenied);
        }

        // Get user's current role
        let user_role =
            AccessControlStorage::get_role(env, user).ok_or(AccessControlError::RoleNotFound)?;

        // Check role hierarchy
        if !revoker_role.level.can_revoke(&user_role.level) {
            AccessControlEvents::emit_hierarchy_violation(env, revoker, user, &user_role.level);
            return Err(AccessControlError::CannotGrantHigherRole);
        }

        // Prevent self-revocation
        if revoker == user {
            return Err(AccessControlError::CannotRevokeOwnRole);
        }

        // Store role in history before removing
        AccessControlStorage::add_role_history(env, user, &user_role);
        AccessControlStorage::add_role_revocation(env, user, &user_role);

        // Remove role
        AccessControlStorage::remove_role(env, user);

        // Emit event
        AccessControlEvents::emit_role_revoked(env, revoker, user, &user_role);

        Ok(())
    }

    /// Transfer a role from one user to another
    pub fn transfer_role(
        env: &Env,
        transferrer: &Address,
        from: &Address,
        to: &Address,
    ) -> Result<(), AccessControlError> {
        // Validate transferrer has permission
        let transferrer_role = AccessControlStorage::validate_user_role(env, transferrer)?;

        if !transferrer_role.has_permission(&Permission::TransferRole) {
            AccessControlEvents::emit_access_denied(env, transferrer, &Permission::TransferRole);
            return Err(AccessControlError::PermissionDenied);
        }

        // Get source user's role
        let source_role =
            AccessControlStorage::get_role(env, from).ok_or(AccessControlError::RoleNotFound)?;

        // Check role hierarchy
        if !transferrer_role.level.can_revoke(&source_role.level) {
            AccessControlEvents::emit_hierarchy_violation(
                env,
                transferrer,
                from,
                &source_role.level,
            );
            return Err(AccessControlError::CannotGrantHigherRole);
        }

        // Create new role for target user
        let new_role = Role::new(
            source_role.level.clone(),
            source_role.permissions.clone(),
            transferrer.clone(),
            env.ledger().timestamp(),
        );

        // Store role in history for source user
        AccessControlStorage::add_role_history(env, from, &source_role);
        AccessControlStorage::add_role_revocation(env, from, &source_role);

        // Remove role from source user
        AccessControlStorage::remove_role(env, from);

        // Grant role to target user
        AccessControlStorage::set_role(env, to, &new_role);
        AccessControlStorage::add_role_grant(env, to, &new_role);

        // Emit event
        AccessControlEvents::emit_role_transferred(env, from, to, &new_role);

        Ok(())
    }

    /// Update a user's role
    pub fn update_role(
        env: &Env,
        updater: &Address,
        user: &Address,
        new_role_level: RoleLevel,
        new_permissions: Vec<Permission>,
    ) -> Result<(), AccessControlError> {
        // Validate updater has permission
        let updater_role = AccessControlStorage::validate_user_role(env, updater)?;

        if !updater_role.has_permission(&Permission::GrantRole) {
            AccessControlEvents::emit_access_denied(env, updater, &Permission::GrantRole);
            return Err(AccessControlError::PermissionDenied);
        }

        // Get current role
        let current_role =
            AccessControlStorage::get_role(env, user).ok_or(AccessControlError::RoleNotFound)?;

        // Check role hierarchy
        if !updater_role.level.can_revoke(&current_role.level) {
            AccessControlEvents::emit_hierarchy_violation(env, updater, user, &current_role.level);
            return Err(AccessControlError::CannotGrantHigherRole);
        }

        // Create new role
        let new_role = Role::new(
            new_role_level,
            new_permissions,
            updater.clone(),
            env.ledger().timestamp(),
        );

        // Store old role in history
        AccessControlStorage::add_role_history(env, user, &current_role);

        // Update role
        AccessControlStorage::set_role(env, user, &new_role);

        // Emit event
        AccessControlEvents::emit_role_updated(env, updater, user, &new_role);

        Ok(())
    }

    /// Grant a specific permission to a user
    pub fn grant_permission(
        env: &Env,
        granter: &Address,
        user: &Address,
        permission: Permission,
    ) -> Result<(), AccessControlError> {
        // Validate granter has permission
        let granter_role = AccessControlStorage::validate_user_role(env, granter)?;

        if !granter_role.has_permission(&Permission::GrantRole) {
            AccessControlEvents::emit_access_denied(env, granter, &Permission::GrantRole);
            return Err(AccessControlError::PermissionDenied);
        }

        // Get user's current role
        let mut user_role =
            AccessControlStorage::get_role(env, user).ok_or(AccessControlError::RoleNotFound)?;

        // Add permission
        RolePermissions::add_permission(&mut user_role, permission.clone());

        // Update role
        AccessControlStorage::set_role(env, user, &user_role);

        // Emit event
        AccessControlEvents::emit_permission_granted(env, granter, user, &permission);

        Ok(())
    }

    /// Revoke a specific permission from a user
    pub fn revoke_permission(
        env: &Env,
        revoker: &Address,
        user: &Address,
        permission: &Permission,
    ) -> Result<(), AccessControlError> {
        // Validate revoker has permission
        let revoker_role = AccessControlStorage::validate_user_role(env, revoker)?;

        if !revoker_role.has_permission(&Permission::RevokeRole) {
            AccessControlEvents::emit_access_denied(env, revoker, &Permission::RevokeRole);
            return Err(AccessControlError::PermissionDenied);
        }

        // Get user's current role
        let mut user_role =
            AccessControlStorage::get_role(env, user).ok_or(AccessControlError::RoleNotFound)?;

        // Remove permission
        RolePermissions::remove_permission(&mut user_role, permission);

        // Update role
        AccessControlStorage::set_role(env, user, &user_role);

        // Emit event
        AccessControlEvents::emit_permission_revoked(env, revoker, user, permission);

        Ok(())
    }

    /// Check if a user has a specific permission
    pub fn has_permission(env: &Env, user: &Address, permission: &Permission) -> bool {
        AccessControlStorage::has_permission(env, user, permission)
    }

    /// Check if a user has any of the specified permissions
    pub fn has_any_permission(env: &Env, user: &Address, permissions: &Vec<Permission>) -> bool {
        AccessControlStorage::has_any_permission(env, user, permissions)
    }

    /// Check if a user has all of the specified permissions
    pub fn has_all_permissions(env: &Env, user: &Address, permissions: &Vec<Permission>) -> bool {
        AccessControlStorage::has_all_permissions(env, user, permissions)
    }

    /// Get a user's role
    pub fn get_role(env: &Env, user: &Address) -> Option<Role> {
        AccessControlStorage::get_role(env, user)
    }

    /// Get a user's role history
    pub fn get_role_history(env: &Env, user: &Address) -> Vec<Role> {
        AccessControlStorage::get_role_history(env, user)
    }

    /// Get a user's role grants
    pub fn get_role_grants(env: &Env, user: &Address) -> Vec<Role> {
        AccessControlStorage::get_role_grants(env, user)
    }

    /// Get a user's role revocations
    pub fn get_role_revocations(env: &Env, user: &Address) -> Vec<Role> {
        AccessControlStorage::get_role_revocations(env, user)
    }

    /// Change the admin
    pub fn change_admin(
        env: &Env,
        current_admin: &Address,
        new_admin: &Address,
    ) -> Result<(), AccessControlError> {
        // Validate current admin
        let admin_role = AccessControlStorage::validate_user_role(env, current_admin)?;

        if admin_role.level != RoleLevel::SuperAdmin {
            AccessControlEvents::emit_access_denied(
                env,
                current_admin,
                &Permission::InitializeContract,
            );
            return Err(AccessControlError::PermissionDenied);
        }

        let old_admin = AccessControlStorage::get_admin(env);

        // Set new admin
        AccessControlStorage::set_admin(env, new_admin);

        // Emit event
        AccessControlEvents::emit_admin_changed(env, &old_admin, new_admin);

        Ok(())
    }

    /// Require a specific permission (for use in function modifiers)
    pub fn require_permission(
        env: &Env,
        user: &Address,
        permission: &Permission,
    ) -> Result<(), AccessControlError> {
        if Self::has_permission(env, user, permission) {
            Ok(())
        } else {
            AccessControlEvents::emit_access_denied(env, user, permission);
            Err(AccessControlError::PermissionDenied)
        }
    }

    /// Require any of the specified permissions
    pub fn require_any_permission(
        env: &Env,
        user: &Address,
        permissions: &Vec<Permission>,
    ) -> Result<(), AccessControlError> {
        if Self::has_any_permission(env, user, permissions) {
            Ok(())
        } else {
            // Emit access denied for the first permission as representative
            if let Some(first_permission) = permissions.first() {
                AccessControlEvents::emit_access_denied(env, user, &first_permission);
            }
            Err(AccessControlError::PermissionDenied)
        }
    }

    /// Require all of the specified permissions
    pub fn require_all_permissions(
        env: &Env,
        user: &Address,
        permissions: &Vec<Permission>,
    ) -> Result<(), AccessControlError> {
        if Self::has_all_permissions(env, user, permissions) {
            Ok(())
        } else {
            // Emit access denied for the first permission as representative
            if let Some(first_permission) = permissions.first() {
                AccessControlEvents::emit_access_denied(env, user, &first_permission);
            }
            Err(AccessControlError::PermissionDenied)
        }
    }
}
