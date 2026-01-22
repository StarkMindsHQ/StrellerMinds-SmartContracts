use crate::event_schema::{AccessControlEventData, EventData, StandardEvent};
use crate::roles::{Permission, Role, RoleLevel};
use soroban_sdk::{Address, Env, String, Symbol};

/// RBAC event emissions
pub struct AccessControlEvents;

impl AccessControlEvents {
    /// Emits event when contract is initialized
    pub fn emit_contract_initialized(env: &Env, admin: &Address) {
        let event_data = AccessControlEventData::ContractInitialized {
            admin: admin.clone(),
        };
        StandardEvent::new(
            env,
            Symbol::new(env, "access_control"),
            admin.clone(),
            EventData::AccessControl(event_data),
        )
        .emit(env);
    }

    /// Emits event when a role is granted
    pub fn emit_role_granted(env: &Env, granter: &Address, user: &Address, role: &Role) {
        let event_data = AccessControlEventData::RoleGranted {
            granter: granter.clone(),
            user: user.clone(),
            role_level: role.level.to_u32(),
            granted_at: role.granted_at,
            expires_at: role.expires_at,
        };
        StandardEvent::new(
            env,
            Symbol::new(env, "access_control"),
            granter.clone(),
            EventData::AccessControl(event_data),
        )
        .emit(env);
    }

    /// Emits event when a role is revoked
    pub fn emit_role_revoked(env: &Env, revoker: &Address, user: &Address, role: &Role) {
        let event_data = AccessControlEventData::RoleRevoked {
            revoker: revoker.clone(),
            user: user.clone(),
            role_level: role.level.to_u32(),
        };
        StandardEvent::new(
            env,
            Symbol::new(env, "access_control"),
            revoker.clone(),
            EventData::AccessControl(event_data),
        )
        .emit(env);
    }

    /// Emits event when a role is transferred
    pub fn emit_role_transferred(env: &Env, from: &Address, to: &Address, role: &Role) {
        let event_data = AccessControlEventData::RoleTransferred {
            from: from.clone(),
            to: to.clone(),
            role_level: role.level.to_u32(),
        };
        StandardEvent::new(
            env,
            Symbol::new(env, "access_control"),
            from.clone(),
            EventData::AccessControl(event_data),
        )
        .emit(env);
    }

    /// Emits event when a role is updated
    pub fn emit_role_updated(env: &Env, updater: &Address, user: &Address, role: &Role) {
        let event_data = AccessControlEventData::RoleUpdated {
            updater: updater.clone(),
            user: user.clone(),
            role_level: role.level.to_u32(),
        };
        StandardEvent::new(
            env,
            Symbol::new(env, "access_control"),
            updater.clone(),
            EventData::AccessControl(event_data),
        )
        .emit(env);
    }

    /// Emits event when a permission is granted
    pub fn emit_permission_granted(
        env: &Env,
        granter: &Address,
        user: &Address,
        permission: &Permission,
    ) {
        let event_data = AccessControlEventData::PermissionGranted {
            granter: granter.clone(),
            user: user.clone(),
            permission: String::from_str(env, permission.to_string()),
        };
        StandardEvent::new(
            env,
            Symbol::new(env, "access_control"),
            granter.clone(),
            EventData::AccessControl(event_data),
        )
        .emit(env);
    }

    /// Emits event when a permission is revoked
    pub fn emit_permission_revoked(
        env: &Env,
        revoker: &Address,
        user: &Address,
        permission: &Permission,
    ) {
        let event_data = AccessControlEventData::PermissionRevoked {
            revoker: revoker.clone(),
            user: user.clone(),
            permission: String::from_str(env, permission.to_string()),
        };
        StandardEvent::new(
            env,
            Symbol::new(env, "access_control"),
            revoker.clone(),
            EventData::AccessControl(event_data),
        )
        .emit(env);
    }

    /// Emits event when admin is changed
    pub fn emit_admin_changed(env: &Env, old_admin: &Address, new_admin: &Address) {
        let event_data = AccessControlEventData::AdminChanged {
            old_admin: old_admin.clone(),
            new_admin: new_admin.clone(),
        };
        StandardEvent::new(
            env,
            Symbol::new(env, "access_control"),
            new_admin.clone(),
            EventData::AccessControl(event_data),
        )
        .emit(env);
    }

    /// Emits event when a role expires
    pub fn emit_role_expired(env: &Env, user: &Address, role: &Role) {
        let event_data = AccessControlEventData::RoleExpired {
            user: user.clone(),
            role_level: role.level.to_u32(),
        };
        StandardEvent::new(
            env,
            Symbol::new(env, "access_control"),
            user.clone(),
            EventData::AccessControl(event_data),
        )
        .emit(env);
    }

    /// Emits event when access is denied
    pub fn emit_access_denied(env: &Env, user: &Address, permission: &Permission) {
        let event_data = AccessControlEventData::AccessDenied {
            user: user.clone(),
            permission: String::from_str(env, permission.to_string()),
        };
        StandardEvent::new(
            env,
            Symbol::new(env, "access_control"),
            user.clone(),
            EventData::AccessControl(event_data),
        )
        .emit(env);
    }

    /// Emits event when role hierarchy is violated
    pub fn emit_hierarchy_violation(
        env: &Env,
        granter: &Address,
        target: &Address,
        target_level: &RoleLevel,
    ) {
        let event_data = AccessControlEventData::HierarchyViolation {
            granter: granter.clone(),
            target: target.clone(),
            target_level: target_level.to_u32(),
        };
        StandardEvent::new(
            env,
            Symbol::new(env, "access_control"),
            granter.clone(),
            EventData::AccessControl(event_data),
        )
        .emit(env);
    }
}
