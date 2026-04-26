use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ProxyError {
    // Initialization (1-9)
    /// The proxy contract has already been initialized.
    AlreadyInitialized = 1,
    /// The proxy contract has not been initialized yet.
    NotInitialized = 2,
    /// Invalid address provided (null address or zero address).
    InvalidAddress = 3,

    // Authorization (10-19)
    /// Caller does not have admin authority to perform this action.
    Unauthorized = 10,
    /// Caller is not authorized to perform the requested operation.
    AccessDenied = 11,

    // Upgrade Operations (20-29)
    /// The contract upgrade operation failed.
    UpgradeFailed = 20,
    /// The rollback to a previous contract version failed.
    RollbackFailed = 21,
    /// No pending upgrade found to execute.
    NoPendingUpgrade = 22,
    /// Upgrade timelock period has not expired yet.
    TimelockNotExpired = 23,
    /// No rollback data available for emergency recovery.
    NoRollbackData = 24,
    /// Rollback window has expired (typically 7 days after upgrade).
    RollbackWindowExpired = 25,

    // Version Management (30-39)
    /// Target version is incompatible with current version.
    IncompatibleVersion = 30,
    /// Version format is invalid or malformed.
    InvalidVersion = 31,
    /// Cannot downgrade to a previous version.
    DowngradeNotAllowed = 32,

    // Emergency Operations (40-49)
    /// Contract is currently paused due to emergency.
    EmergencyPaused = 40,
    /// Emergency operation failed due to invalid state.
    EmergencyFailed = 41,

    // Data Migration (50-59)
    /// Data migration operation failed.
    MigrationFailed = 50,
    /// Migration is in progress and cannot be interrupted.
    MigrationInProgress = 51,
    /// Migration data is corrupted or invalid.
    InvalidMigrationData = 52,

    // Governance (60-69)
    /// Upgrade proposal not found or expired.
    ProposalNotFound = 60,
    /// Voting period for proposal has not ended.
    VotingPeriodNotEnded = 61,
    /// Insufficient votes to approve the upgrade.
    InsufficientVotes = 62,
    /// Proposal has already been executed.
    ProposalAlreadyExecuted = 63,

    // Business Logic (80-199)
    /// Operation failed due to business logic constraints.
    BusinessLogicError = 80,
    /// Contract state is invalid for the requested operation.
    InvalidState = 81,
    /// Required resources or conditions are not available.
    InsufficientResources = 82,
}
