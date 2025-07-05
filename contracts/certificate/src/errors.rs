use soroban_sdk::{contracterror};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CertificateError {
    // Initialization errors
    AlreadyInitialized = 1,
    NotInitialized = 2,
    InitializationFailed = 3,
    
    // Authorization errors
    Unauthorized = 4,
    RoleNotFound = 5,
    InvalidRole = 6,
    
    // Certificate errors
    CertificateNotFound = 7,
    CertificateAlreadyExists = 8,
    CertificateAlreadyRevoked = 9,
    CertificateRevoked = 10,
    CertificateExpired = 11,
    
    // Metadata errors
    InvalidMetadata = 12,
    InvalidUri = 13,
    
    // Input validation errors
    InvalidAddress = 14,
    InvalidInput = 15,
    
    // Storage errors
    StorageError = 16,
    
    // Event errors
    EventError = 17,
}
