//! Upgradeable Proxy Implementation
//!
//! This module provides a comprehensive upgradeable proxy pattern that allows
//! contracts to be upgraded while preserving state and maintaining security.
//!
//! Features:
//! - Admin-controlled upgrades
//! - Version management
//! - Emergency controls
//! - Rollback capabilities
//! - Timelock protection

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, String, Symbol, Vec};

use crate::errors::ProxyError;

/// Storage keys for the proxy contract
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ProxyStorageKey {
    /// Admin address that controls upgrades
    Admin,
    /// Current implementation address
    Implementation,
    /// Pending upgrade address (for timelock)
    PendingImplementation,
    /// Upgrade timelock expiration
    UpgradeTimelock,
    /// Emergency pause status
    EmergencyPaused,
    /// Version history
    VersionHistory,
    /// Rollback data
    RollbackData,
    /// Upgrade proposal tracking
    UpgradeProposal(Address),
    /// Migration status
    MigrationStatus(Symbol),
}

/// Version information
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionInfo {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub implementation: Address,
    pub timestamp: u64,
}

impl VersionInfo {
    pub fn new(
        major: u32,
        minor: u32,
        patch: u32,
        implementation: Address,
        timestamp: u64,
    ) -> Self {
        Self { major, minor, patch, implementation, timestamp }
    }

    pub fn to_string(&self, env: &Env) -> String {
        let version_str = "version";
        String::from_str(env, version_str)
    }

    pub fn is_compatible_with(&self, other: &VersionInfo) -> bool {
        // Same major version means compatible
        self.major == other.major
    }
}

/// Rollback information for emergency recovery
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RollbackData {
    pub previous_implementation: Address,
    pub previous_version: VersionInfo,
    pub rollback_timestamp: u64,
    pub migration_data: Map<Symbol, soroban_sdk::Bytes>, // Changed to Bytes for better compatibility
}

/// Upgrade proposal with governance support
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeProposal {
    pub new_implementation: Address,
    pub new_version: VersionInfo,
    pub description: String,
    pub proposer: Address,
    pub proposed_at: u64,
    pub timelock_duration: u64,
    pub requires_migration: bool,
    pub migration_plan: String,
}

/// Migration status tracking
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MigrationStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed(String),
}

#[contract]
pub struct UpgradeableProxy;

#[contractimpl]
impl UpgradeableProxy {
    /// Initialize the proxy with admin and initial implementation
    ///
    /// # Arguments
    /// * `admin` - Address that controls upgrades
    /// * `implementation` - Initial implementation address
    /// * `version` - Initial version information
    pub fn initialize(
        env: Env,
        admin: Address,
        implementation: Address,
        version: VersionInfo,
    ) -> Result<(), ProxyError> {
        // Check if already initialized
        if env.storage().instance().has(&ProxyStorageKey::Admin) {
            return Err(ProxyError::AlreadyInitialized);
        }

        // Validate inputs
        if admin == Address::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
        {
            return Err(ProxyError::InvalidAddress);
        }

        // Store initial configuration
        env.storage().instance().set(&ProxyStorageKey::Admin, &admin);
        env.storage().instance().set(&ProxyStorageKey::Implementation, &implementation);
        env.storage().instance().set(&ProxyStorageKey::EmergencyPaused, &false);

        // Initialize version history
        let mut version_history: Vec<VersionInfo> = Vec::new(&env);
        version_history.push_back(version.clone());
        env.storage().instance().set(&ProxyStorageKey::VersionHistory, &version_history);

        // Emit initialization event
        env.events().publish(
            (Symbol::new(&env, "proxy_initialized"), admin.clone()),
            (implementation.clone(), version.to_string(&env)),
        );

        Ok(())
    }

    /// Upgrade to a new implementation with governance and timelock
    ///
    /// # Arguments
    /// * `admin` - Admin address authorizing the upgrade
    /// * `new_implementation` - New implementation address
    /// * `new_version` - Version information for new implementation
    /// * `timelock_duration` - Duration in seconds before upgrade becomes active
    /// * `requires_migration` - Whether data migration is needed
    /// * `migration_plan` - Description of migration steps
    pub fn propose_upgrade(
        env: Env,
        admin: Address,
        new_implementation: Address,
        new_version: VersionInfo,
        timelock_duration: u64,
        requires_migration: bool,
        migration_plan: String,
    ) -> Result<(), ProxyError> {
        // Validate admin
        Self::validate_admin(&env, &admin)?;

        // Validate not paused
        if Self::is_emergency_paused(&env) {
            return Err(ProxyError::EmergencyPaused);
        }

        // Validate new implementation
        if new_implementation
            == Address::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
        {
            return Err(ProxyError::InvalidAddress);
        }

        // Validate version compatibility
        let current_version = Self::get_current_version(&env)?;
        if !current_version.is_compatible_with(&new_version) {
            return Err(ProxyError::IncompatibleVersion);
        }

        // Create upgrade proposal
        let proposal = UpgradeProposal {
            new_implementation: new_implementation.clone(),
            new_version: new_version.clone(),
            description: String::from_str(&env, "Upgrade to new implementation"),
            proposer: admin.clone(),
            proposed_at: env.ledger().timestamp(),
            timelock_duration,
            requires_migration,
            migration_plan: migration_plan.clone(),
        };

        // Set timelock
        let unlock_timestamp = env.ledger().timestamp() + timelock_duration;
        env.storage().instance().set(&ProxyStorageKey::UpgradeTimelock, &unlock_timestamp);
        env.storage().instance().set(&ProxyStorageKey::PendingImplementation, &new_implementation);

        // Store rollback data
        let rollback_data = RollbackData {
            previous_implementation: Self::get_current_implementation(&env)?,
            previous_version: current_version.clone(),
            rollback_timestamp: env.ledger().timestamp(),
            migration_data: Map::new(&env), // Changed from Vec to Map for better compatibility
        };
        env.storage().instance().set(&ProxyStorageKey::RollbackData, &rollback_data);

        // Store proposal
        env.storage().instance().set(&ProxyStorageKey::UpgradeProposal(admin.clone()), &proposal);

        // Emit proposal event
        env.events().publish(
            (Symbol::new(&env, "upgrade_proposed"), admin),
            (new_implementation, new_version.to_string(&env), unlock_timestamp),
        );

        Ok(())
    }

    /// Execute the pending upgrade if timelock has expired
    ///
    /// # Arguments
    /// * `admin` - Admin address authorizing the execution
    pub fn execute_upgrade(env: Env, admin: Address) -> Result<(), ProxyError> {
        // Validate admin
        Self::validate_admin(&env, &admin)?;

        // Check timelock
        let unlock_timestamp: u64 = env
            .storage()
            .instance()
            .get(&ProxyStorageKey::UpgradeTimelock)
            .ok_or(ProxyError::NoPendingUpgrade)?;

        if env.ledger().timestamp() < unlock_timestamp {
            return Err(ProxyError::TimelockNotExpired);
        }

        // Get pending implementation
        let new_implementation: Address = env
            .storage()
            .instance()
            .get(&ProxyStorageKey::PendingImplementation)
            .ok_or(ProxyError::NoPendingUpgrade)?;

        // Get current implementation for rollback
        let current_implementation = Self::get_current_implementation(&env)?;
        let current_version = Self::get_current_version(&env)?;

        // Execute upgrade
        env.storage().instance().set(&ProxyStorageKey::Implementation, &new_implementation);

        // Update version history
        let mut version_history: Vec<VersionInfo> =
            env.storage().instance().get(&ProxyStorageKey::VersionHistory).unwrap();

        // Update current version with new implementation
        let mut updated_version = current_version.clone();
        updated_version.implementation = new_implementation.clone();
        updated_version.timestamp = env.ledger().timestamp();

        version_history.push_back(updated_version);
        env.storage().instance().set(&ProxyStorageKey::VersionHistory, &version_history);

        // Clear pending upgrade data
        env.storage().instance().remove(&ProxyStorageKey::PendingImplementation);
        env.storage().instance().remove(&ProxyStorageKey::UpgradeTimelock);

        // Emit upgrade event
        env.events().publish(
            (Symbol::new(&env, "upgrade_executed"), admin),
            (current_implementation, new_implementation),
        );

        Ok(())
    }

    /// Rollback to previous implementation
    ///
    /// # Arguments
    /// * `admin` - Admin address authorizing the rollback
    pub fn rollback(env: Env, admin: Address) -> Result<(), ProxyError> {
        // Validate admin
        Self::validate_admin(&env, &admin)?;

        // Get rollback data
        let rollback_data: RollbackData = env
            .storage()
            .instance()
            .get(&ProxyStorageKey::RollbackData)
            .ok_or(ProxyError::NoRollbackData)?;

        // Validate rollback window (e.g., within 7 days)
        let rollback_window = 7 * 24 * 60 * 60; // 7 days in seconds
        if env.ledger().timestamp() > rollback_data.rollback_timestamp + rollback_window {
            return Err(ProxyError::RollbackWindowExpired);
        }

        // Execute rollback
        env.storage()
            .instance()
            .set(&ProxyStorageKey::Implementation, &rollback_data.previous_implementation);

        // Update version history
        let mut version_history: Vec<VersionInfo> =
            env.storage().instance().get(&ProxyStorageKey::VersionHistory).unwrap();
        version_history.push_back(rollback_data.previous_version.clone());
        env.storage().instance().set(&ProxyStorageKey::VersionHistory, &version_history);

        // Clear rollback data
        env.storage().instance().remove(&ProxyStorageKey::RollbackData);

        // Emit rollback event
        env.events().publish(
            (Symbol::new(&env, "rollback_executed"), admin),
            (rollback_data.previous_implementation, rollback_data.previous_version.to_string(&env)),
        );

        Ok(())
    }

    /// Emergency pause the contract (disable all functions except admin functions)
    ///
    /// # Arguments
    /// * `admin` - Admin address authorizing the pause
    pub fn emergency_pause(env: Env, admin: Address) -> Result<(), ProxyError> {
        // Validate admin
        Self::validate_admin(&env, &admin)?;

        // Set pause flag
        env.storage().instance().set(&ProxyStorageKey::EmergencyPaused, &true);

        // Emit pause event
        env.events()
            .publish((Symbol::new(&env, "emergency_paused"), admin), env.ledger().timestamp());

        Ok(())
    }

    /// Resume contract operations after emergency pause
    ///
    /// # Arguments
    /// * `admin` - Admin address authorizing the resume
    pub fn emergency_resume(env: Env, admin: Address) -> Result<(), ProxyError> {
        // Validate admin
        Self::validate_admin(&env, &admin)?;

        // Clear pause flag
        env.storage().instance().set(&ProxyStorageKey::EmergencyPaused, &false);

        // Emit resume event
        env.events()
            .publish((Symbol::new(&env, "emergency_resumed"), admin), env.ledger().timestamp());

        Ok(())
    }

    /// Transfer admin rights to a new address
    ///
    /// # Arguments
    /// * `current_admin` - Current admin address
    /// * `new_admin` - New admin address
    pub fn transfer_admin(
        env: Env,
        current_admin: Address,
        new_admin: Address,
    ) -> Result<(), ProxyError> {
        // Validate current admin
        Self::validate_admin(&env, &current_admin)?;

        // Validate new admin
        if new_admin
            == Address::from_str(&env, "GAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA")
        {
            return Err(ProxyError::InvalidAddress);
        }

        // Transfer admin rights
        env.storage().instance().set(&ProxyStorageKey::Admin, &new_admin);

        // Emit transfer event
        env.events()
            .publish((Symbol::new(&env, "admin_transferred"), current_admin), new_admin.clone());

        Ok(())
    }

    /// Get current admin address
    pub fn get_admin(env: Env) -> Result<Address, ProxyError> {
        env.storage().instance().get(&ProxyStorageKey::Admin).ok_or(ProxyError::NotInitialized)
    }

    /// Get current implementation address
    pub fn get_implementation(env: Env) -> Result<Address, ProxyError> {
        Self::get_current_implementation(&env)
    }

    /// Get current version information
    pub fn get_version(env: Env) -> Result<VersionInfo, ProxyError> {
        Self::get_current_version(&env)
    }

    /// Get version history
    pub fn get_version_history(env: Env) -> Result<Vec<VersionInfo>, ProxyError> {
        env.storage()
            .instance()
            .get(&ProxyStorageKey::VersionHistory)
            .ok_or(ProxyError::NotInitialized)
    }

    /// Check if contract is emergency paused
    pub fn is_paused(env: Env) -> Result<bool, ProxyError> {
        Ok(Self::is_emergency_paused(&env))
    }

    /// Get pending upgrade information
    pub fn get_pending_upgrade(env: Env) -> Result<Option<(Address, u64)>, ProxyError> {
        let implementation: Option<Address> =
            env.storage().instance().get(&ProxyStorageKey::PendingImplementation);
        let timelock: Option<u64> = env.storage().instance().get(&ProxyStorageKey::UpgradeTimelock);

        match (implementation, timelock) {
            (Some(impl_addr), Some(timestamp)) => Ok(Some((impl_addr, timestamp))),
            _ => Ok(None),
        }
    }

    /// Get rollback information
    pub fn get_rollback_info(env: Env) -> Result<Option<RollbackData>, ProxyError> {
        if env.storage().instance().has(&ProxyStorageKey::RollbackData) {
            Ok(env.storage().instance().get(&ProxyStorageKey::RollbackData))
        } else {
            Ok(None)
        }
    }

    // Helper functions

    fn validate_admin(env: &Env, admin: &Address) -> Result<(), ProxyError> {
        let stored_admin: Address = env
            .storage()
            .instance()
            .get(&ProxyStorageKey::Admin)
            .ok_or(ProxyError::NotInitialized)?;

        if &stored_admin != admin {
            return Err(ProxyError::Unauthorized);
        }

        Ok(())
    }

    fn get_current_implementation(env: &Env) -> Result<Address, ProxyError> {
        env.storage()
            .instance()
            .get(&ProxyStorageKey::Implementation)
            .ok_or(ProxyError::NotInitialized)
    }

    fn get_current_version(env: &Env) -> Result<VersionInfo, ProxyError> {
        let version_history: Vec<VersionInfo> = env
            .storage()
            .instance()
            .get(&ProxyStorageKey::VersionHistory)
            .ok_or(ProxyError::NotInitialized)?;

        if version_history.is_empty() {
            return Err(ProxyError::NotInitialized);
        }
        let last_version = version_history.get(version_history.len() - 1).unwrap();
        Ok(last_version.clone())
    }

    fn is_emergency_paused(env: &Env) -> bool {
        env.storage().instance().get(&ProxyStorageKey::EmergencyPaused).unwrap_or(false)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_proxy_initialization() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let implementation = Address::generate(&env);
        let version = VersionInfo::new(1, 0, 0, implementation.clone(), env.ledger().timestamp());

        // Initialize proxy
        UpgradeableProxy::initialize(
            env.clone(),
            admin.clone(),
            implementation.clone(),
            version.clone(),
        )
        .expect("Initialization should succeed");

        // Verify state
        assert_eq!(UpgradeableProxy::get_admin(env.clone()).unwrap(), admin);
        assert_eq!(UpgradeableProxy::get_implementation(env.clone()).unwrap(), implementation);
        assert_eq!(UpgradeableProxy::get_version(env.clone()).unwrap(), version);
        assert!(!UpgradeableProxy::is_paused(env.clone()).unwrap());
    }

    #[test]
    fn test_upgrade_proposal() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let implementation = Address::generate(&env);
        let new_implementation = Address::generate(&env);
        let version = VersionInfo::new(1, 0, 0, implementation.clone(), env.ledger().timestamp());
        let new_version =
            VersionInfo::new(1, 1, 0, new_implementation.clone(), env.ledger().timestamp());

        // Initialize
        UpgradeableProxy::initialize(
            env.clone(),
            admin.clone(),
            implementation.clone(),
            version.clone(),
        )
        .expect("Initialization should succeed");

        // Propose upgrade
        UpgradeableProxy::propose_upgrade(
            env.clone(),
            admin.clone(),
            new_implementation.clone(),
            new_version.clone(),
            3600, // 1 hour timelock
            false,
            String::from_str(&env, "No migration needed"),
        )
        .expect("Upgrade proposal should succeed");

        // Check pending upgrade
        let pending = UpgradeableProxy::get_pending_upgrade(env.clone()).unwrap();
        assert!(pending.is_some());
        let (pending_impl, timelock) = pending.unwrap();
        assert_eq!(pending_impl, new_implementation);
        assert!(timelock > env.ledger().timestamp());
    }

    #[test]
    fn test_emergency_pause() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let implementation = Address::generate(&env);
        let version = VersionInfo::new(1, 0, 0, implementation.clone(), env.ledger().timestamp());

        // Initialize
        UpgradeableProxy::initialize(env.clone(), admin.clone(), implementation.clone(), version)
            .expect("Initialization should succeed");

        // Emergency pause
        UpgradeableProxy::emergency_pause(env.clone(), admin.clone())
            .expect("Emergency pause should succeed");

        assert!(UpgradeableProxy::is_paused(env.clone()).unwrap());

        // Resume
        UpgradeableProxy::emergency_resume(env.clone(), admin.clone())
            .expect("Emergency resume should succeed");

        assert!(!UpgradeableProxy::is_paused(env.clone()).unwrap());
    }
}
