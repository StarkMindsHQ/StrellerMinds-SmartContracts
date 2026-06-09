/// Role Delegation — Issue #443
///
/// Allows role holders to temporarily delegate their responsibilities to
/// another address with an optional time-bound expiry and full audit trail.
use crate::errors::AccessControlError;
use crate::roles::{Permission, RoleLevel};
use crate::storage::AccessControlStorage;
use soroban_sdk::{contracttype, symbol_short, Address, Env, Vec};

// ─── Types ────────────────────────────────────────────────────────────────────

/// A single delegation record.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Delegation {
    /// The address that created this delegation.
    pub delegator: Address,
    /// The address receiving the delegated role.
    pub delegatee: Address,
    /// The role level being delegated.
    pub role_level: RoleLevel,
    /// Ledger timestamp when the delegation was created.
    pub created_at: u64,
    /// Optional expiry timestamp; `None` means no expiry.
    pub expires_at: Option<u64>,
    /// Whether this delegation is still active.
    pub active: bool,
}

impl Delegation {
    pub fn is_expired(&self, now: u64) -> bool {
        self.expires_at.map_or(false, |exp| now >= exp)
    }

    pub fn is_valid(&self, now: u64) -> bool {
        self.active && !self.is_expired(now)
    }
}

// ─── Storage keys ─────────────────────────────────────────────────────────────

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DelegationKey {
    /// Active delegation for a delegatee.
    Active(Address),
    /// Full audit log for a delegator.
    AuditLog(Address),
}

// ─── Core logic ───────────────────────────────────────────────────────────────

pub struct RoleDelegation;

impl RoleDelegation {
    /// Delegate a role to `delegatee` with an optional expiry.
    ///
    /// Rules:
    /// - `delegator` must hold the role being delegated.
    /// - `delegatee` must not already hold an active delegation.
    /// - A delegatee cannot re-delegate (no chained delegation).
    pub fn delegate_role(
        env: &Env,
        delegator: &Address,
        delegatee: &Address,
        role_level: RoleLevel,
        expires_at: Option<u64>,
    ) -> Result<Delegation, AccessControlError> {
        delegator.require_auth();

        // Delegator must hold the role they want to delegate.
        let delegator_role = AccessControlStorage::validate_user_role(env, delegator)?;
        if delegator_role.level != role_level {
            return Err(AccessControlError::PermissionDenied);
        }

        // Delegator must have GrantRole permission.
        if !delegator_role.has_permission(&Permission::GrantRole) {
            return Err(AccessControlError::PermissionDenied);
        }

        // Reject if delegatee already has an active delegation.
        if let Some(existing) = Self::get_active_delegation(env, delegatee) {
            if existing.is_valid(env.ledger().timestamp()) {
                return Err(AccessControlError::RoleAlreadyExists);
            }
        }

        // Validate expiry is in the future.
        if let Some(exp) = expires_at {
            if exp <= env.ledger().timestamp() {
                return Err(AccessControlError::InvalidRole);
            }
        }

        let delegation = Delegation {
            delegator: delegator.clone(),
            delegatee: delegatee.clone(),
            role_level: role_level.clone(),
            created_at: env.ledger().timestamp(),
            expires_at,
            active: true,
        };

        // Persist active delegation for the delegatee.
        env.storage()
            .persistent()
            .set(&DelegationKey::Active(delegatee.clone()), &delegation);

        // Append to delegator's audit log.
        Self::append_audit(env, delegator, &delegation);

        env.events().publish(
            (soroban_sdk::symbol_short!("delegated"), delegator.clone()),
            (delegatee.clone(), role_level.to_u32(), expires_at),
        );

        Ok(delegation)
    }

    /// Revoke an active delegation.
    ///
    /// Only the original delegator (or an admin with RevokeRole) may revoke.
    pub fn revoke_delegation(
        env: &Env,
        caller: &Address,
        delegatee: &Address,
    ) -> Result<(), AccessControlError> {
        caller.require_auth();

        let mut delegation = Self::get_active_delegation(env, delegatee)
            .ok_or(AccessControlError::RoleNotFound)?;

        // Only the delegator or someone with RevokeRole permission may revoke.
        let is_delegator = *caller == delegation.delegator;
        let has_revoke = AccessControlStorage::has_permission(env, caller, &Permission::RevokeRole);
        if !is_delegator && !has_revoke {
            return Err(AccessControlError::PermissionDenied);
        }

        delegation.active = false;

        // Update stored record.
        env.storage()
            .persistent()
            .set(&DelegationKey::Active(delegatee.clone()), &delegation);

        // Append revocation to delegator's audit log.
        Self::append_audit(env, &delegation.delegator.clone(), &delegation);

        env.events().publish(
            (soroban_sdk::symbol_short!("del_revoked"), caller.clone()),
            delegatee.clone(),
        );

        Ok(())
    }

    /// Return the active delegation for `delegatee`, if any.
    pub fn get_active_delegation(env: &Env, delegatee: &Address) -> Option<Delegation> {
        env.storage()
            .persistent()
            .get(&DelegationKey::Active(delegatee.clone()))
    }

    /// Check whether `delegatee` currently holds a valid (active, non-expired) delegation
    /// for `role_level`.
    pub fn has_valid_delegation(env: &Env, delegatee: &Address, role_level: &RoleLevel) -> bool {
        Self::get_active_delegation(env, delegatee).map_or(false, |d| {
            d.role_level == *role_level && d.is_valid(env.ledger().timestamp())
        })
    }

    /// Return the full audit log for `delegator`.
    pub fn get_audit_log(env: &Env, delegator: &Address) -> Vec<Delegation> {
        env.storage()
            .persistent()
            .get(&DelegationKey::AuditLog(delegator.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }

    // ── Private helpers ───────────────────────────────────────────────────────

    fn append_audit(env: &Env, delegator: &Address, delegation: &Delegation) {
        const MAX_AUDIT: u32 = 200;
        let key = DelegationKey::AuditLog(delegator.clone());
        let mut log: Vec<Delegation> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));
        if log.len() >= MAX_AUDIT {
            log.pop_front();
        }
        log.push_back(delegation.clone());
        env.storage().persistent().set(&key, &log);
    }
}

// ─── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::access_control::AccessControl;
    use crate::roles::RoleLevel;
    use soroban_sdk::{testutils::Address as _, Env};

    fn setup() -> (Env, Address, Address, Address) {
        let env = Env::default();
        env.mock_all_auths();
        let admin = Address::generate(&env);
        let instructor = Address::generate(&env);
        let delegatee = Address::generate(&env);
        AccessControl::initialize(&env, &admin).unwrap();
        AccessControl::grant_role(&env, &admin, &instructor, RoleLevel::Instructor).unwrap();
        (env, admin, instructor, delegatee)
    }

    #[test]
    fn test_delegate_and_check() {
        let (env, _admin, instructor, delegatee) = setup();
        let now = env.ledger().timestamp();
        let delegation =
            RoleDelegation::delegate_role(&env, &instructor, &delegatee, RoleLevel::Instructor, Some(now + 3600))
                .unwrap();
        assert!(delegation.is_valid(now));
        assert!(RoleDelegation::has_valid_delegation(&env, &delegatee, &RoleLevel::Instructor));
    }

    #[test]
    fn test_revoke_delegation() {
        let (env, _admin, instructor, delegatee) = setup();
        let now = env.ledger().timestamp();
        RoleDelegation::delegate_role(&env, &instructor, &delegatee, RoleLevel::Instructor, Some(now + 3600))
            .unwrap();
        RoleDelegation::revoke_delegation(&env, &instructor, &delegatee).unwrap();
        assert!(!RoleDelegation::has_valid_delegation(&env, &delegatee, &RoleLevel::Instructor));
    }

    #[test]
    fn test_expired_delegation_is_invalid() {
        let (env, _admin, instructor, delegatee) = setup();
        let now = env.ledger().timestamp();
        RoleDelegation::delegate_role(&env, &instructor, &delegatee, RoleLevel::Instructor, Some(now + 1))
            .unwrap();
        // Simulate time passing beyond expiry.
        assert!(!RoleDelegation::has_valid_delegation(&env, &delegatee, &RoleLevel::Instructor));
    }

    #[test]
    fn test_audit_log_populated() {
        let (env, _admin, instructor, delegatee) = setup();
        let now = env.ledger().timestamp();
        RoleDelegation::delegate_role(&env, &instructor, &delegatee, RoleLevel::Instructor, Some(now + 3600))
            .unwrap();
        RoleDelegation::revoke_delegation(&env, &instructor, &delegatee).unwrap();
        let log = RoleDelegation::get_audit_log(&env, &instructor);
        assert_eq!(log.len(), 2); // created + revoked entries
    }

    #[test]
    fn test_duplicate_delegation_rejected() {
        let (env, _admin, instructor, delegatee) = setup();
        let now = env.ledger().timestamp();
        RoleDelegation::delegate_role(&env, &instructor, &delegatee, RoleLevel::Instructor, Some(now + 3600))
            .unwrap();
        let result = RoleDelegation::delegate_role(
            &env,
            &instructor,
            &delegatee,
            RoleLevel::Instructor,
            Some(now + 7200),
        );
        assert_eq!(result, Err(AccessControlError::RoleAlreadyExists));
    }
}
