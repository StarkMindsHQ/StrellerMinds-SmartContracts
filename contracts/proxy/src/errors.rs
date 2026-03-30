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
    // Authorization (10-19)
    /// Caller does not have admin authority to perform this action.
    Unauthorized = 10,
    // Business logic (80-199)
    /// The contract upgrade operation failed.
    UpgradeFailed = 80,
    /// The rollback to a previous contract version failed.
    RollbackFailed = 81,
}
