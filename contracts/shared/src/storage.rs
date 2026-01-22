use crate::errors::AccessControlError;
use crate::permissions::RolePermissions;
use crate::roles::Role;
use soroban_sdk::{contracttype, Address, Env, Vec};

/// Storage keys for the RBAC system
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Key for the admin address
    Admin,
    /// Flag indicating if contract is initialized
    Initialized,
    /// Key for storing user roles
    Role(Address),
    /// Key for storing role history
    RoleHistory(Address),
    /// Key for storing role grants
    RoleGrants(Address),
    /// Key for storing role revocations
    RoleRevocations(Address),
    /// Key for storing system configuration
    Config,
    /// Key for permission templates
    PermissionTemplate(soroban_sdk::Symbol),
    /// Key for role inheritance
    RoleInheritance(crate::roles::RoleLevel),
}

/// RBAC storage operations
pub struct AccessControlStorage;

impl AccessControlStorage {
    /// Sets the contract admin
    pub fn set_admin(env: &Env, admin: &Address) {
        env.storage().instance().set(&DataKey::Admin, admin);
    }

    /// Retrieves the current admin
    pub fn get_admin(env: &Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    /// Marks the contract as initialized
    pub fn set_initialized(env: &Env) {
        env.storage().instance().set(&DataKey::Initialized, &true);
    }

    /// Checks if the contract is initialized
    pub fn is_initialized(env: &Env) -> bool {
        env.storage().instance().has(&DataKey::Initialized)
    }

    /// Sets a role for a user
    pub fn set_role(env: &Env, user: &Address, role: &Role) {
        let key = DataKey::Role(user.clone());
        env.storage().instance().set(&key, role);
    }

    /// Gets a role for a user
    pub fn get_role(env: &Env, user: &Address) -> Option<Role> {
        let key = DataKey::Role(user.clone());
        if env.storage().instance().has(&key) {
            env.storage().instance().get(&key)
        } else {
            None
        }
    }

    /// Removes a role for a user
    pub fn remove_role(env: &Env, user: &Address) {
        let key = DataKey::Role(user.clone());
        env.storage().instance().remove(&key);
    }

    /// Checks if a user has a role
    pub fn has_role(env: &Env, user: &Address) -> bool {
        Self::get_role(env, user).is_some()
    }

    /// Stores role history for a user
    pub fn add_role_history(env: &Env, user: &Address, role: &Role) {
        let key = DataKey::RoleHistory(user.clone());
        let mut history: Vec<Role> = if env.storage().instance().has(&key) {
            env.storage().instance().get(&key).unwrap()
        } else {
            Vec::new(env)
        };
        history.push_back(role.clone());
        env.storage().instance().set(&key, &history);
    }

    /// Gets role history for a user
    pub fn get_role_history(env: &Env, user: &Address) -> Vec<Role> {
        let key = DataKey::RoleHistory(user.clone());
        if env.storage().instance().has(&key) {
            env.storage().instance().get(&key).unwrap()
        } else {
            Vec::new(env)
        }
    }

    /// Stores role grants for a user
    pub fn add_role_grant(env: &Env, user: &Address, role: &Role) {
        let key = DataKey::RoleGrants(user.clone());
        let mut grants: Vec<Role> = if env.storage().instance().has(&key) {
            env.storage().instance().get(&key).unwrap()
        } else {
            Vec::new(env)
        };
        grants.push_back(role.clone());
        env.storage().instance().set(&key, &grants);
    }

    /// Gets role grants for a user
    pub fn get_role_grants(env: &Env, user: &Address) -> Vec<Role> {
        let key = DataKey::RoleGrants(user.clone());
        if env.storage().instance().has(&key) {
            env.storage().instance().get(&key).unwrap()
        } else {
            Vec::new(env)
        }
    }

    /// Stores role revocations for a user
    pub fn add_role_revocation(env: &Env, user: &Address, role: &Role) {
        let key = DataKey::RoleRevocations(user.clone());
        let mut revocations: Vec<Role> = if env.storage().instance().has(&key) {
            env.storage().instance().get(&key).unwrap()
        } else {
            Vec::new(env)
        };
        revocations.push_back(role.clone());
        env.storage().instance().set(&key, &revocations);
    }

    /// Gets role revocations for a user
    pub fn get_role_revocations(env: &Env, user: &Address) -> Vec<Role> {
        let key = DataKey::RoleRevocations(user.clone());
        if env.storage().instance().has(&key) {
            env.storage().instance().get(&key).unwrap()
        } else {
            Vec::new(env)
        }
    }

    /// Validates that a user has a valid role
    pub fn validate_user_role(env: &Env, user: &Address) -> Result<Role, AccessControlError> {
        if let Some(role) = Self::get_role(env, user) {
            let current_time = env.ledger().timestamp();
            if role.is_valid(current_time) {
                Ok(role)
            } else {
                Err(AccessControlError::RoleNotFound) // Role expired
            }
        } else {
            Err(AccessControlError::RoleNotFound)
        }
    }

    /// Checks if a user has a specific permission
    pub fn has_permission(
        env: &Env,
        user: &Address,
        permission: &crate::roles::Permission,
    ) -> bool {
        if let Ok(role) = Self::validate_user_role(env, user) {
            let resolved = RolePermissions::resolve_all_permissions(env, &role);
            resolved.contains(permission)
        } else {
            false
        }
    }

    /// Checks if a user has any of the specified permissions
    pub fn has_any_permission(
        env: &Env,
        user: &Address,
        permissions: &Vec<crate::roles::Permission>,
    ) -> bool {
        if let Ok(role) = Self::validate_user_role(env, user) {
            let resolved = RolePermissions::resolve_all_permissions(env, &role);
            permissions.iter().any(|p| resolved.contains(&p))
        } else {
            false
        }
    }

    /// Checks if a user has all of the specified permissions
    pub fn has_all_permissions(
        env: &Env,
        user: &Address,
        permissions: &Vec<crate::roles::Permission>,
    ) -> bool {
        if let Ok(role) = Self::validate_user_role(env, user) {
            let resolved = RolePermissions::resolve_all_permissions(env, &role);
            permissions.iter().all(|p| resolved.contains(&p))
        } else {
            false
        }
    }

    /// Gets all users with a specific role level
    pub fn get_users_with_role_level(env: &Env, level: &crate::roles::RoleLevel) -> Vec<Address> {
        // This would require iterating through all users, which is not efficient
        // In a real implementation, you might want to maintain a separate index
        Vec::new(env) // Placeholder implementation
    }

    /// Gets all users with a specific permission
    pub fn get_users_with_permission(
        env: &Env,
        _permission: &crate::roles::Permission,
    ) -> Vec<Address> {
        // This would require iterating through all users, which is not efficient
        // In a real implementation, you might want to maintain a separate index
        Vec::new(env) // Placeholder implementation
    }

    /// Sets a permission template
    pub fn set_permission_template(
        env: &Env,
        template_id: &soroban_sdk::Symbol,
        permissions: &Vec<crate::roles::Permission>,
    ) {
        let key = DataKey::PermissionTemplate(template_id.clone());
        env.storage().instance().set(&key, permissions);
    }

    /// Gets a permission template
    pub fn get_permission_template(
        env: &Env,
        template_id: &soroban_sdk::Symbol,
    ) -> Option<Vec<crate::roles::Permission>> {
        let key = DataKey::PermissionTemplate(template_id.clone());
        if env.storage().instance().has(&key) {
            env.storage().instance().get(&key)
        } else {
            None
        }
    }

    /// Sets role inheritance
    pub fn set_role_inheritance(
        env: &Env,
        level: crate::roles::RoleLevel,
        inherited: &Vec<crate::roles::RoleLevel>,
    ) {
        let key = DataKey::RoleInheritance(level);
        env.storage().instance().set(&key, inherited);
    }

    /// Gets role inheritance
    pub fn get_role_inheritance(
        env: &Env,
        level: &crate::roles::RoleLevel,
    ) -> Vec<crate::roles::RoleLevel> {
        let key = DataKey::RoleInheritance(level.clone());
        if env.storage().instance().has(&key) {
            env.storage().instance().get(&key).unwrap()
        } else {
            Vec::new(env)
        }
    }
}
