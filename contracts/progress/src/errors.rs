use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ProgressError {
    /// Contract has already been initialized; `initialize` may only be called once.
    AlreadyInitialized = 1,
    /// Contract has not been initialized; call `initialize` first.
    NotInitialized = 2,
    /// Caller does not have the required admin privileges.
    Unauthorized = 10,
    /// Progress value is out of the valid range (0–100).
    InvalidProgress = 20,
    /// Provided course ID is empty or otherwise invalid.
    InvalidCourseId = 21,
    /// No progress record was found for the given student and course combination.
    ProgressNotFound = 50,
}
