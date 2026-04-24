use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StudentProgressError {
    // Initialization (1-9)
    /// The student progress tracker has already been initialized.
    AlreadyInitialized = 1,
    /// The student progress tracker has not been initialized yet.
    NotInitialized = 2,
    // Authorization (10-19)
    /// Caller does not have the required permissions to perform this action.
    Unauthorized = 10,
    /// The admin address has not been set in contract storage.
    AdminNotSet = 11,
    // Validation (20-49)
    /// The provided completion percentage is outside the valid 0–100 range.
    InvalidPercent = 20,
}
