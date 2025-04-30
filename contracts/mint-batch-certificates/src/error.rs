use soroban_sdk::{contracterror};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // General errors
    Unauthorized = 1,
    AlreadyInitialized = 2,
    NotInitialized = 3,
    
    // Minting errors
    InvalidInput = 100,
    DuplicateCertificate = 101,
    StorageError = 102,
    BatchSizeTooLarge = 103,
    InvalidTimeRange = 104,
    BatchSizeExceeded = 105,
    
    // Certificate management errors
    CertificateNotFound = 200,
    CertificateAlreadyRevoked = 201,
    CertificateNotRevocable = 202,
}

// Result type for batch minting operations
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MintResult {
    Success(u64), // Certificate ID that was successfully minted
    Failure(u64, Error), // Certificate ID and the error that occurred
}
