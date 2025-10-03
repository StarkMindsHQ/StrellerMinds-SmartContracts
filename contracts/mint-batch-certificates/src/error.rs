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

impl Error {
    pub fn message(&self) -> &'static str {
        match self {
            Error::Unauthorized => "Unauthorized: The caller does not have permission to perform this action.",
            Error::AlreadyInitialized => "AlreadyInitialized: The contract has already been initialized.",
            Error::NotInitialized => "NotInitialized: The contract has not been initialized.",
            Error::InvalidInput => "InvalidInput: The input provided is invalid.",
            Error::DuplicateCertificate => "DuplicateCertificate: A certificate with this ID already exists.",
            Error::StorageError => "StorageError: An error occurred while accessing storage.",
            Error::BatchSizeTooLarge => "BatchSizeTooLarge: The batch size exceeds the allowed maximum.",
            Error::InvalidTimeRange => "InvalidTimeRange: The certificate's time range is invalid.",
            Error::BatchSizeExceeded => "BatchSizeExceeded: The batch size was exceeded during processing.",
            Error::CertificateNotFound => "CertificateNotFound: The specified certificate was not found.",
            Error::CertificateAlreadyRevoked => "CertificateAlreadyRevoked: The certificate has already been revoked.",
            Error::CertificateNotRevocable => "CertificateNotRevocable: The certificate cannot be revoked.",
        }
    }
}

// Result type for batch minting operations
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MintResult {
    Success(u64), // Certificate ID that was successfully minted
    Failure(u64, Error), // Certificate ID and the error that occurred
}
