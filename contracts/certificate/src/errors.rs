use soroban_sdk::{contracterror, ConversionError};

/// Errors that can occur during certificate contract execution
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CertificateError {
    /// Contract has already been initialized
    AlreadyInitialized = 1,
    /// Certificate with the specified ID already exists
    CertificateAlreadyExists = 2,
    /// Certificate with the specified ID was not found
    CertificateNotFound = 3,
    /// Contract has not been initialized
    NotInitialized = 4,
    /// User is not authorized to perform this action
    Unauthorized = 5,
    /// User is not the instructor for this certificate
    NotInstructor = 6,
    /// Invalid token ID provided
    InvalidTokenId = 7,
    /// Invalid certificate metadata provided
    InvalidMetadata = 8,
    /// Certificate has been revoked
    CertificateRevoked = 9,
    /// Transfer of this certificate is not allowed
    TransferNotAllowed = 10,
    /// User's role was not found
    RoleNotFound = 11,
    /// Certificate has expired
    CertificateExpired = 12,
}

/// Implementation to convert ConversionError to CertificateError
impl From<ConversionError> for CertificateError {
    fn from(_: ConversionError) -> Self {
        CertificateError::InvalidMetadata
    }
}
