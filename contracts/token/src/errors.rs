use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TokenError {
    // Initialization (1-9)
    AlreadyInitialized = 1,
    NotInitialized = 2,
    // Authorization (10-19)
    Unauthorized = 10,
    // Validation (20-49)
    InvalidAmount = 20,
    InvalidAddress = 21,
    // Business logic (80-199)
    InsufficientBalance = 80,
    TransferFailed = 81,
}
