use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum SocialSharingError {
    InvalidCertificateId = 1,
    InvalidShareMessage = 2,
    InvalidPlatform = 3,
    ShareRecordNotFound = 4,
    Unauthorized = 5,
    StorageError = 6,
    AlreadyInitialized = 7,
    NotInitialized = 8,
}
