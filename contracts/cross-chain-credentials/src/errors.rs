use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CrossChainError {
    // Initialization (1-9)
    AlreadyInitialized = 1,
    NotInitialized = 2,
    // Authorization (10-19)
    Unauthorized = 10,
    // Not Found (50-79)
    CredentialNotFound = 50,
    ProofNotFound = 51,
    VerificationRequestNotFound = 52,
    // Business logic (80-199)
    CredentialNotActive = 80,
    CredentialRevoked = 81,
    CredentialSuspended = 82,
}
