use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CrossChainError {
    /// Contract has already been initialized; `initialize` may only be called once.
    AlreadyInitialized = 1,
    /// Contract has not been initialized; call `initialize` first.
    NotInitialized = 2,
    /// Caller does not have the required admin privileges.
    Unauthorized = 10,
    /// No credential was found with the given ID.
    CredentialNotFound = 50,
    /// No cross-chain proof has been generated for the given credential.
    ProofNotFound = 51,
    /// No verification request was found with the given ID.
    VerificationRequestNotFound = 52,
    /// Credential exists but is not in `Active` status and cannot be used for verification.
    CredentialNotActive = 80,
    /// Credential has been permanently revoked and cannot be reactivated.
    CredentialRevoked = 81,
    /// Credential has been temporarily suspended.
    CredentialSuspended = 82,
}
