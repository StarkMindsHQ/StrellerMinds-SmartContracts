use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ProgressError {
    // Initialization (1-9)
    AlreadyInitialized = 1,
    NotInitialized = 2,
    // Authorization (10-19)
    Unauthorized = 10,
    // Validation (20-49)
    InvalidProgress = 20,
    InvalidCourseId = 21,
    // Not Found (50-79)
    ProgressNotFound = 50,
}
