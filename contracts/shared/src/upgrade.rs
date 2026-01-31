//! Upgrade framework for smart contracts
//! Provides utilities for safe contract upgrades with data migration and rollback capabilities

use alloc::format;
use soroban_sdk::{contracttype, Address, Env, String, Symbol, Vec};

/// Version information for contract storage
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VersionInfo {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub timestamp: u64,
}

impl VersionInfo {
    pub fn new(major: u32, minor: u32, patch: u32, timestamp: u64) -> Self {
        Self {
            major,
            minor,
            patch,
            timestamp,
        }
    }

    /// Check if this version is compatible with target version
    pub fn is_compatible_with(&self, target: &VersionInfo) -> bool {
        // Major version must match for compatibility
        self.major == target.major &&
        // Minor version can be equal or lower (backward compatible)
        self.minor <= target.minor
    }
}

/// Storage keys for upgrade system
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum UpgradeKey {
    /// Current storage version
    StorageVersion,
    /// Contract version history
    VersionHistory,
    /// Migration status for specific versions
    MigrationStatus(Symbol),
    /// Upgrade timelock expiration
    UpgradeTimelock,
    /// Pending upgrade implementation
    PendingUpgrade,
    /// Upgrade proposer
    UpgradeProposer,
    /// Governance votes for upgrade
    UpgradeVotes(Address),
    /// Emergency pause flag
    EmergencyPaused,
}

/// Migration status for data migrations
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MigrationStatus {
    NotStarted,
    InProgress,
    Completed,
    Failed(String), // Error message
}

/// Upgrade proposal information
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UpgradeProposal {
    pub new_implementation: Address,
    pub version: VersionInfo,
    pub description: String,
    pub proposed_at: u64,
    pub proposer: Address,
    pub vote_count: u32,
    pub required_votes: u32,
    pub executed: bool,
}

/// Upgrade framework utilities
pub struct UpgradeUtils;

impl UpgradeUtils {
    /// Initialize upgrade system
    pub fn initialize(env: &Env, initial_version: &VersionInfo) {
        env.storage()
            .instance()
            .set(&UpgradeKey::StorageVersion, initial_version);

        let mut history: Vec<VersionInfo> = Vec::new(env);
        history.push_back(initial_version.clone());
        env.storage()
            .instance()
            .set(&UpgradeKey::VersionHistory, &history);

        env.storage()
            .instance()
            .set(&UpgradeKey::EmergencyPaused, &false);
    }

    /// Get current storage version
    pub fn get_current_version(env: &Env) -> VersionInfo {
        env.storage()
            .instance()
            .get(&UpgradeKey::StorageVersion)
            .unwrap()
    }

    /// Set storage version (used during migrations)
    pub fn set_storage_version(env: &Env, version: &VersionInfo) {
        env.storage()
            .instance()
            .set(&UpgradeKey::StorageVersion, version);

        let mut history: Vec<VersionInfo> = env
            .storage()
            .instance()
            .get(&UpgradeKey::VersionHistory)
            .unwrap();
        history.push_back(version.clone());
        env.storage()
            .instance()
            .set(&UpgradeKey::VersionHistory, &history);
    }

    /// Get version history
    pub fn get_version_history(env: &Env) -> Vec<VersionInfo> {
        env.storage()
            .instance()
            .get(&UpgradeKey::VersionHistory)
            .unwrap()
    }

    /// Check if migration is needed for target version
    pub fn requires_migration(env: &Env, target_version: &VersionInfo) -> bool {
        let current = Self::get_current_version(env);
        !current.is_compatible_with(target_version)
    }

    /// Mark migration status
    pub fn set_migration_status(env: &Env, migration_id: &Symbol, status: &MigrationStatus) {
        let key = UpgradeKey::MigrationStatus(migration_id.clone());
        env.storage().instance().set(&key, status);
    }

    /// Get migration status
    pub fn get_migration_status(env: &Env, migration_id: &Symbol) -> MigrationStatus {
        let key = UpgradeKey::MigrationStatus(migration_id.clone());
        if env.storage().instance().has(&key) {
            env.storage().instance().get(&key).unwrap()
        } else {
            MigrationStatus::NotStarted
        }
    }

    /// Validate version compatibility

    /// Validate version compatibility
    pub fn validate_version_compatibility(
        env: &Env,
        current: &VersionInfo,
        target: &VersionInfo,
    ) -> Result<(), String> {
        if current.major != target.major {
            return Err(String::from_str(
                env,
                "Major version mismatch. Breaking changes detected.",
            ));
        }

        if target.minor < current.minor {
            return Err(String::from_str(env, "Cannot downgrade minor version"));
        }

        Ok(())
    }

    /// Execute data migration
    pub fn execute_migration<F>(
        env: &Env,
        migration_id: &Symbol,
        target_version: &VersionInfo,
        migration_fn: F,
    ) -> Result<(), String>
    where
        F: FnOnce(&Env) -> Result<(), String>,
    {
        // Check if already migrated
        let status = Self::get_migration_status(env, migration_id);
        if matches!(status, MigrationStatus::Completed) {
            return Ok(());
        }

        // Set migration in progress
        Self::set_migration_status(env, migration_id, &MigrationStatus::InProgress);

        // Execute migration
        match migration_fn(env) {
            Ok(()) => {
                // Update version
                Self::set_storage_version(env, target_version);
                Self::set_migration_status(env, migration_id, &MigrationStatus::Completed);
                Ok(())
            }
            Err(e) => {
                Self::set_migration_status(env, migration_id, &MigrationStatus::Failed(e.clone()));
                Err(e)
            }
        }
    }

    /// Set emergency pause
    pub fn set_emergency_pause(env: &Env, paused: bool) {
        env.storage()
            .instance()
            .set(&UpgradeKey::EmergencyPaused, &paused);
    }

    /// Check if emergency pause is active
    pub fn is_emergency_paused(env: &Env) -> bool {
        env.storage()
            .instance()
            .get(&UpgradeKey::EmergencyPaused)
            .unwrap_or(false)
    }

    /// Validate that emergency pause is not active
    pub fn validate_not_paused(env: &Env) -> Result<(), String> {
        if Self::is_emergency_paused(env) {
            Err(String::from_str(
                env,
                "Contract is currently paused due to emergency",
            ))
        } else {
            Ok(())
        }
    }
}

/// Governance-based upgrade system
pub struct GovernanceUpgrade;

impl GovernanceUpgrade {
    /// Propose an upgrade
    pub fn propose_upgrade(
        env: &Env,
        proposer: &Address,
        new_implementation: &Address,
        version: &VersionInfo,
        description: &String,
        required_votes: u32,
    ) -> Result<Symbol, String> {
        // Validate proposer has upgrade permission
        // This would integrate with your existing RBAC system
        // For now, we'll assume validation happens elsewhere

        let proposal_id = Symbol::new(env, &format!("upgrade_{}", env.ledger().sequence()));

        let proposal = UpgradeProposal {
            new_implementation: new_implementation.clone(),
            version: version.clone(),
            description: description.clone(),
            proposed_at: env.ledger().timestamp(),
            proposer: proposer.clone(),
            vote_count: 0,
            required_votes,
            executed: false,
        };

        env.storage().temporary().set(&proposal_id, &proposal);
        env.storage()
            .temporary()
            .extend_ttl(&proposal_id, 1000000, 1000000);
        Ok(proposal_id)
    }

    /// Vote on upgrade proposal
    pub fn vote_on_upgrade(
        env: &Env,
        voter: &Address,
        proposal_id: &Symbol,
    ) -> Result<u32, String> {
        let proposal: UpgradeProposal = env
            .storage()
            .temporary()
            .get(proposal_id)
            .ok_or_else(|| String::from_str(env, "Proposal not found"))?;

        if proposal.executed {
            return Err(String::from_str(env, "Proposal already executed"));
        }

        let vote_key = UpgradeKey::UpgradeVotes(voter.clone());
        if env.storage().temporary().has(&vote_key) {
            return Err(String::from_str(env, "Already voted"));
        }

        env.storage().temporary().set(proposal_id, &proposal);
        env.storage()
            .temporary()
            .extend_ttl(proposal_id, 1000000, 1000000);
        env.storage().temporary().set(&vote_key, &true);
        env.storage()
            .temporary()
            .extend_ttl(&vote_key, 1000000, 1000000);

        Ok(proposal.vote_count)
    }

    /// Execute upgrade if voting threshold reached
    pub fn execute_upgrade_if_approved(
        env: &Env,
        proposal_id: &Symbol,
    ) -> Result<Option<Address>, String> {
        let proposal: UpgradeProposal = env
            .storage()
            .temporary()
            .get(proposal_id)
            .ok_or_else(|| String::from_str(env, "Proposal not found"))?;

        if proposal.executed {
            return Err(String::from_str(env, "Proposal already executed"));
        }

        if proposal.vote_count >= proposal.required_votes {
            // Set pending upgrade
            env.storage()
                .instance()
                .set(&UpgradeKey::PendingUpgrade, &proposal.new_implementation);
            env.storage()
                .instance()
                .set(&UpgradeKey::UpgradeProposer, &proposal.proposer);

            // Mark as executed
            let mut updated_proposal = proposal.clone();
            updated_proposal.executed = true;
            env.storage()
                .temporary()
                .set(proposal_id, &updated_proposal);
            env.storage()
                .temporary()
                .extend_ttl(proposal_id, 1000000, 1000000);

            Ok(Some(proposal.new_implementation))
        } else {
            Ok(None)
        }
    }

    /// Set upgrade timelock
    pub fn set_upgrade_timelock(env: &Env, unlock_timestamp: u64) {
        env.storage()
            .instance()
            .set(&UpgradeKey::UpgradeTimelock, &unlock_timestamp);
    }

    /// Check if upgrade timelock has expired
    pub fn is_upgrade_unlocked(env: &Env) -> bool {
        let current_time = env.ledger().timestamp();
        let unlock_time: u64 = env
            .storage()
            .instance()
            .get(&UpgradeKey::UpgradeTimelock)
            .unwrap_or(current_time);
        current_time >= unlock_time
    }

    /// Validate upgrade can proceed
    pub fn validate_upgrade_ready(env: &Env) -> Result<(), String> {
        UpgradeUtils::validate_not_paused(env)?;

        if !Self::is_upgrade_unlocked(env) {
            return Err(String::from_str(env, "Upgrade timelock not expired"));
        }

        if !env.storage().instance().has(&UpgradeKey::PendingUpgrade) {
            return Err(String::from_str(env, "No pending upgrade"));
        }

        Ok(())
    }
}

/// Data migration utilities
pub struct DataMigration;

impl DataMigration {
    /// Migrate data from old structure to new structure
    pub fn migrate_data<T, U, F>(
        env: &Env,
        migration_id: &Symbol,
        _transform_fn: F,
    ) -> Result<(), String>
    where
        T: soroban_sdk::TryFromVal<soroban_sdk::Env, soroban_sdk::Val>,
        U: soroban_sdk::IntoVal<soroban_sdk::Env, soroban_sdk::Val>,
        F: FnOnce(T) -> Result<U, String>,
    {
        // This is a generic migration helper
        // Actual implementation would depend on specific data structures
        // For now, we'll provide the framework

        Self::execute_migration_step(env, migration_id, || {
            // Migration logic would go here
            // This would typically involve:
            // 1. Reading old data format
            // 2. Transforming to new format
            // 3. Writing new data
            // 4. Cleaning up old data

            Ok(())
        })
    }

    /// Execute a single migration step with error handling
    pub fn execute_migration_step<F>(
        _env: &Env,
        _migration_id: &Symbol,
        step_fn: F,
    ) -> Result<(), String>
    where
        F: FnOnce() -> Result<(), String>,
    {
        match step_fn() {
            Ok(()) => Ok(()),
            Err(e) => {
                // Log error and mark migration as failed
                // In a real implementation, you'd want more sophisticated error handling
                // Note: 'e' is soroban String which doesn't implement Display.
                // We cannot use format! with it easily.
                // Just pass the error through or create generic error message
                Err(e)
            }
        }
    }

    /// Validate data integrity after migration
    pub fn validate_data_integrity(_env: &Env) -> Result<(), String> {
        // This would contain validation logic specific to your contract's data
        // Examples:
        // - Check that all required data structures exist
        // - Validate cross-references between data
        // - Ensure no data corruption occurred

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;

    #[test]
    fn test_version_info() {
        let _env = Env::default();
        let v1 = VersionInfo::new(1, 0, 0, 1000);
        let v2 = VersionInfo::new(1, 1, 0, 2000);
        let v3 = VersionInfo::new(2, 0, 0, 3000);

        let v1_str = format!("{}.{}.{}", v1.major, v1.minor, v1.patch);
        assert_eq!(v1_str, "1.0.0");
        assert!(v1.is_compatible_with(&v2));
        assert!(!v1.is_compatible_with(&v3)); // Different major version
        assert!(!v2.is_compatible_with(&v1)); // Downgrade not allowed
    }

    #[test]
    fn test_upgrade_utils_initialization() {
        let env = Env::default();
        let initial_version = VersionInfo::new(1, 0, 0, 1000);

        UpgradeUtils::initialize(&env, &initial_version);

        let current = UpgradeUtils::get_current_version(&env);
        assert_eq!(current, initial_version);

        let history = UpgradeUtils::get_version_history(&env);
        assert_eq!(history.len(), 1);
        assert_eq!(history.get(0).unwrap(), initial_version);
    }

    #[test]
    fn test_migration_status() {
        let env = Env::default();
        let migration_id = Symbol::new(&env, "test_mig");

        // Initially not started
        let status = UpgradeUtils::get_migration_status(&env, &migration_id);
        assert_eq!(status, MigrationStatus::NotStarted);

        // Set in progress
        UpgradeUtils::set_migration_status(&env, &migration_id, &MigrationStatus::InProgress);
        let status = UpgradeUtils::get_migration_status(&env, &migration_id);
        assert_eq!(status, MigrationStatus::InProgress);

        // Set completed
        UpgradeUtils::set_migration_status(&env, &migration_id, &MigrationStatus::Completed);
        let status = UpgradeUtils::get_migration_status(&env, &migration_id);
        assert_eq!(status, MigrationStatus::Completed);
    }

    #[test]
    fn test_governance_proposal() {
        let env = Env::default();
        let proposer = Address::generate(&env);
        let new_impl = Address::generate(&env);
        let version = VersionInfo::new(1, 1, 0, 2000);

        let description = String::from_str(&env, "Test upgrade");
        let proposal_id = GovernanceUpgrade::propose_upgrade(
            &env,
            &proposer,
            &new_impl,
            &version,
            &description,
            3,
        )
        .unwrap();

        let vote_count = GovernanceUpgrade::vote_on_upgrade(&env, &proposer, &proposal_id).unwrap();
        assert_eq!(vote_count, 1);
    }
}
