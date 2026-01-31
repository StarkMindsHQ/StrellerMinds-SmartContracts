#![no_std]
use shared::access_control::AccessControl;
use shared::roles::Permission;
use shared::upgrade::{GovernanceUpgrade, UpgradeUtils, VersionInfo};
use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, String, Symbol, Vec};

pub struct ProxyEvents;

impl ProxyEvents {
    pub fn emit_initialized(env: &Env, admin: &Address, implementation: &Address) {
        let topics = (Symbol::new(env, "proxy_initialized"), admin, implementation);
        env.events().publish(topics, ());
    }
    pub fn emit_upgraded(env: &Env, admin: &Address, new_impl: &Address) {
        let topics = (Symbol::new(env, "proxy_upgraded"), admin, new_impl);
        env.events().publish(topics, ());
    }
    pub fn emit_rollback(env: &Env, admin: &Address, prev_impl: &Address) {
        let topics = (Symbol::new(env, "proxy_rollback"), admin, prev_impl);
        env.events().publish(topics, ());
    }
    pub fn emit_upgrade_proposed(env: &Env, proposer: &Address, proposal_id: &Symbol) {
        let topics = (Symbol::new(env, "upgrade_proposed"), proposer, proposal_id);
        env.events().publish(topics, ());
    }
    pub fn emit_upgrade_executed(env: &Env, executor: &Address, new_impl: &Address) {
        let topics = (Symbol::new(env, "upgrade_executed"), executor, new_impl);
        env.events().publish(topics, ());
    }
    pub fn emit_emergency_pause(env: &Env, admin: &Address, paused: bool) {
        let topics = (Symbol::new(env, "emergency_pause"), admin, paused);
        env.events().publish(topics, ());
    }
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Implementation,
    Admin,
    RollbackStack,
    // Enhanced upgrade system
    CurrentVersion,
    UpgradeTimelock,
    PendingUpgrade,
    UpgradeProposer,
    EmergencyPaused,
}

#[contract]
pub struct Proxy;

#[contractimpl]
impl Proxy {
    /// Initialize proxy with admin and implementation address
    pub fn initialize(env: Env, admin: Address, implementation: Address) {
        // Prevent re-initialization
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Contract already initialized");
        }

        admin.require_auth();
        // Initialize centralized RBAC (grants SuperAdmin to admin)
        let _ = AccessControl::initialize(&env, &admin);
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage()
            .instance()
            .set(&DataKey::Implementation, &implementation);
        env.storage()
            .instance()
            .set(&DataKey::RollbackStack, &Vec::<Address>::new(&env));

        // Initialize upgrade system
        let initial_version = VersionInfo::new(1, 0, 0, env.ledger().timestamp());
        UpgradeUtils::initialize(&env, &initial_version);
        env.storage()
            .instance()
            .set(&DataKey::CurrentVersion, &initial_version);
        env.storage()
            .instance()
            .set(&DataKey::EmergencyPaused, &false);

        ProxyEvents::emit_initialized(&env, &admin, &implementation);
    }

    /// Standard upgrade (admin only) - immediate execution
    pub fn upgrade(env: Env, new_implementation: Address) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        // RBAC: require upgrade permission
        if AccessControl::require_permission(&env, &admin, &Permission::UpgradeContract).is_err() {
            panic!("Unauthorized");
        }

        // Check emergency pause
        if env
            .storage()
            .instance()
            .get(&DataKey::EmergencyPaused)
            .unwrap_or(false)
        {
            panic!("Contract is emergency paused");
        }

        let current: Address = env
            .storage()
            .instance()
            .get(&DataKey::Implementation)
            .unwrap();
        let mut stack: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::RollbackStack)
            .unwrap();
        stack.push_back(current.clone());
        env.storage()
            .instance()
            .set(&DataKey::RollbackStack, &stack);
        env.storage()
            .instance()
            .set(&DataKey::Implementation, &new_implementation);
        ProxyEvents::emit_upgraded(&env, &admin, &new_implementation);
    }

    /// Governance-based upgrade proposal
    #[allow(clippy::too_many_arguments)]
    pub fn propose_upgrade(
        env: Env,
        proposer: Address,
        new_implementation: Address,
        version_major: u32,
        version_minor: u32,
        version_patch: u32,
        description: String,
        required_votes: u32,
    ) -> Symbol {
        proposer.require_auth();

        // Validate proposer has upgrade permission
        if AccessControl::require_permission(&env, &proposer, &Permission::UpgradeContract).is_err()
        {
            panic!("Unauthorized");
        }

        let version = VersionInfo::new(
            version_major,
            version_minor,
            version_patch,
            env.ledger().timestamp(),
        );

        let proposal_id = GovernanceUpgrade::propose_upgrade(
            &env,
            &proposer,
            &new_implementation,
            &version,
            &description,
            required_votes,
        )
        .expect("Failed to propose upgrade");

        ProxyEvents::emit_upgrade_proposed(&env, &proposer, &proposal_id);
        proposal_id
    }

    /// Vote on upgrade proposal
    pub fn vote_on_upgrade(env: Env, voter: Address, proposal_id: Symbol) -> u32 {
        voter.require_auth();

        // Validate voter has upgrade permission
        if AccessControl::require_permission(&env, &voter, &Permission::UpgradeContract).is_err() {
            panic!("Unauthorized");
        }

        GovernanceUpgrade::vote_on_upgrade(&env, &voter, &proposal_id)
            .expect("Failed to vote on upgrade")
    }

    /// Execute approved upgrade
    pub fn execute_upgrade(env: Env) {
        // Check emergency pause
        if env
            .storage()
            .instance()
            .get(&DataKey::EmergencyPaused)
            .unwrap_or(false)
        {
            panic!("Contract is emergency paused");
        }

        // Check timelock
        let current_time = env.ledger().timestamp();
        let unlock_time: u32 = env
            .storage()
            .instance()
            .get(&DataKey::UpgradeTimelock)
            .unwrap_or(0);
        if current_time < unlock_time as u64 {
            panic!("Upgrade timelock not expired");
        }

        let pending_impl: Address = env
            .storage()
            .instance()
            .get(&DataKey::PendingUpgrade)
            .expect("No pending upgrade");
        let proposer: Address = env
            .storage()
            .instance()
            .get(&DataKey::UpgradeProposer)
            .expect("No upgrade proposer");

        // Perform upgrade
        let current: Address = env
            .storage()
            .instance()
            .get(&DataKey::Implementation)
            .unwrap();
        let mut stack: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::RollbackStack)
            .unwrap();
        stack.push_back(current);
        env.storage()
            .instance()
            .set(&DataKey::RollbackStack, &stack);
        env.storage()
            .instance()
            .set(&DataKey::Implementation, &pending_impl);

        // Clear pending upgrade
        env.storage().instance().remove(&DataKey::PendingUpgrade);
        env.storage().instance().remove(&DataKey::UpgradeProposer);

        ProxyEvents::emit_upgrade_executed(&env, &proposer, &pending_impl);
    }

    /// Set upgrade timelock duration (in seconds)
    pub fn set_upgrade_timelock(env: Env, duration_seconds: u32) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if AccessControl::require_permission(&env, &admin, &Permission::UpgradeContract).is_err() {
            panic!("Unauthorized");
        }

        let unlock_time = env.ledger().timestamp() + duration_seconds as u64;
        env.storage()
            .instance()
            .set(&DataKey::UpgradeTimelock, &unlock_time);
    }

    /// Set emergency pause
    pub fn set_emergency_pause(env: Env, paused: bool) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();

        if AccessControl::require_permission(&env, &admin, &Permission::UpgradeContract).is_err() {
            panic!("Unauthorized");
        }

        env.storage()
            .instance()
            .set(&DataKey::EmergencyPaused, &paused);
        ProxyEvents::emit_emergency_pause(&env, &admin, paused);
    }

    /// Rollback to previous implementation (admin only)
    pub fn rollback(env: Env) {
        let admin: Address = env.storage().instance().get(&DataKey::Admin).unwrap();
        admin.require_auth();
        // RBAC: require upgrade/rollback permission
        if AccessControl::require_permission(&env, &admin, &Permission::UpgradeContract).is_err() {
            panic!("Unauthorized");
        }
        let mut stack: Vec<Address> = env
            .storage()
            .instance()
            .get(&DataKey::RollbackStack)
            .unwrap();
        let prev = stack.pop_back().expect("No previous implementation");
        env.storage()
            .instance()
            .set(&DataKey::RollbackStack, &stack);
        env.storage()
            .instance()
            .set(&DataKey::Implementation, &prev);
        ProxyEvents::emit_rollback(&env, &admin, &prev);
    }

    /// Get current implementation address
    pub fn get_implementation(env: Env) -> Address {
        env.storage()
            .instance()
            .get(&DataKey::Implementation)
            .unwrap()
    }

    /// Get admin address
    pub fn get_admin(env: Env) -> Address {
        env.storage().instance().get(&DataKey::Admin).unwrap()
    }

    /// Get current storage version
    pub fn get_current_version(env: Env) -> VersionInfo {
        env.storage()
            .instance()
            .get(&DataKey::CurrentVersion)
            .unwrap()
    }

    /// Check if emergency pause is active
    pub fn is_emergency_paused(env: Env) -> bool {
        env.storage()
            .instance()
            .get(&DataKey::EmergencyPaused)
            .unwrap_or(false)
    }

    /// Get upgrade timelock expiration
    pub fn get_upgrade_timelock(env: Env) -> u64 {
        env.storage()
            .instance()
            .get(&DataKey::UpgradeTimelock)
            .unwrap_or(0)
    }

    /// Get pending upgrade implementation (if any)
    pub fn get_pending_upgrade(env: Env) -> Option<Address> {
        env.storage().instance().get(&DataKey::PendingUpgrade)
    }
}

// Note: Actual call delegation is handled by Soroban host, not in userland Rust.
// For a real proxy, you would use Soroban's host functions to forward calls.

#[cfg(test)]
mod tests;
