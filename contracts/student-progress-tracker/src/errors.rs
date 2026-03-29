use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StudentProgressError {
    // Initialization (1-9)
    AlreadyInitialized = 1,
    NotInitialized = 2,
    // Authorization (10-19)
    Unauthorized = 10,
    AdminNotSet = 11,
    // Validation (20-49)
    InvalidPercent = 20,
}
