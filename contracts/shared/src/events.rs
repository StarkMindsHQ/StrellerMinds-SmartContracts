use soroban_sdk::{Address, Env, Symbol};
use crate::roles::{Role, RoleLevel, Permission};

/// RBAC event emissions
pub struct AccessControlEvents;

impl AccessControlEvents {
    /// Emits event when contract is initialized
    pub fn emit_contract_initialized(env: &Env, admin: &Address) {
        let topics = (Symbol::new(env, "contract_initialized"),);
        env.events().publish(topics, admin);
    }

    /// Emits event when a role is granted
    pub fn emit_role_granted(env: &Env, granter: &Address, user: &Address, role: &Role) {
        let topics = (Symbol::new(env, "role_granted"), granter, user);
        let data = (
            role.level.to_u32(),
            role.granted_at,
            role.expires_at,
        );
        env.events().publish(topics, data);
    }

    /// Emits event when a role is revoked
    pub fn emit_role_revoked(env: &Env, revoker: &Address, user: &Address, role: &Role) {
        let topics = (Symbol::new(env, "role_revoked"), revoker, user);
        let data = (
            role.level.to_u32(),
            role.granted_at,
            role.expires_at,
        );
        env.events().publish(topics, data);
    }

    /// Emits event when a role is transferred
    pub fn emit_role_transferred(env: &Env, from: &Address, to: &Address, role: &Role) {
        let topics = (Symbol::new(env, "role_transferred"), from, to);
        let data = (
            role.level.to_u32(),
            role.granted_at,
            role.expires_at,
        );
        env.events().publish(topics, data);
    }

    /// Emits event when a role is updated
    pub fn emit_role_updated(env: &Env, updater: &Address, user: &Address, role: &Role) {
        let topics = (Symbol::new(env, "role_updated"), updater, user);
        let data = (
            role.level.to_u32(),
            role.granted_at,
            role.expires_at,
        );
        env.events().publish(topics, data);
    }

    /// Emits event when a permission is granted
    pub fn emit_permission_granted(env: &Env, granter: &Address, user: &Address, permission: &Permission) {
        let topics = (Symbol::new(env, "permission_granted"), granter, user);
        let data = (permission.to_string(),);
        env.events().publish(topics, data);
    }

    /// Emits event when a permission is revoked
    pub fn emit_permission_revoked(env: &Env, revoker: &Address, user: &Address, permission: &Permission) {
        let topics = (Symbol::new(env, "permission_revoked"), revoker, user);
        let data = (permission.to_string(),);
        env.events().publish(topics, data);
    }

    /// Emits event when admin is changed
    pub fn emit_admin_changed(env: &Env, old_admin: &Address, new_admin: &Address) {
        let topics = (Symbol::new(env, "admin_changed"),);
        let data = (old_admin, new_admin);
        env.events().publish(topics, data);
    }

    /// Emits event when a role expires
    pub fn emit_role_expired(env: &Env, user: &Address, role: &Role) {
        let topics = (Symbol::new(env, "role_expired"), user);
        let data = (
            role.level.to_u32(),
            role.granted_at,
            role.expires_at,
        );
        env.events().publish(topics, data);
    }

    /// Emits event when access is denied
    pub fn emit_access_denied(env: &Env, user: &Address, permission: &Permission) {
        let topics = (Symbol::new(env, "access_denied"), user);
        let data = (permission.to_string(),);
        env.events().publish(topics, data);
    }

    /// Emits event when role hierarchy is violated
    pub fn emit_hierarchy_violation(env: &Env, granter: &Address, target: &Address, target_level: &RoleLevel) {
        let topics = (Symbol::new(env, "hierarchy_violation"), granter, target);
        let data = (target_level.to_u32(),);
        env.events().publish(topics, data);
    }
} 