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
    MetadataTooLarge = 14,
    MetadataTooSmall = 15,
    InvalidCharacters = 16,
    InvalidFormat = 17,
    
    // Input validation errors
    InvalidAddress = 18,
    InvalidInput = 19,
    
    // Storage errors
    StorageError = 20,
    
    // Event errors
    EventError = 21,
    
    // Multi-signature errors
    MultiSigRequestNotFound = 22,
    MultiSigRequestAlreadyExists = 23,
    MultiSigRequestExpired = 24,
    MultiSigRequestAlreadyApproved = 25,
    MultiSigRequestAlreadyRejected = 26,
    MultiSigRequestAlreadyExecuted = 27,
    InsufficientApprovals = 28,
    ApproverNotAuthorized = 29,
    ApprovalAlreadyExists = 30,
    InvalidMultiSigConfig = 31,
    MultiSigConfigNotFound = 32,
    InvalidApprovalThreshold = 33,
    TimeoutTooShort = 34,
    TimeoutTooLong = 35,
    
    // Prerequisite errors
    PrerequisiteNotMet = 36,
    PrerequisiteNotFound = 37,
    PrerequisiteAlreadyExists = 38,
    CircularDependency = 39,
    InvalidPrerequisiteConfig = 40,
    PrerequisiteOverrideNotFound = 41,
    PrerequisiteOverrideExpired = 42,
    InsufficientProgress = 43,
    CertificateRequired = 44,
    InvalidLearningPath = 45,
}
