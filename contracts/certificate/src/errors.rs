use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificateError {
    // Initialization
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,

    // Multi-sig
    MultiSigRequestNotFound = 10,
    MultiSigRequestExpired = 11,
    ApproverNotAuthorized = 12,
    InsufficientApprovals = 13,
    InvalidApprovalThreshold = 14,
    AlreadyApproved = 15,
    RequestNotPending = 16,
    RequestAlreadyExecuted = 17,

    // Certificate lifecycle
    CertificateNotFound = 20,
    CertificateAlreadyExists = 21,
    CertificateRevoked = 22,
    CertificateExpired = 23,
    CertificateNotEligibleForReissue = 24,

    // Template
    TemplateNotFound = 30,
    TemplateAlreadyExists = 31,
    TemplateInactive = 32,
    MissingRequiredField = 33,

    // Configuration
    InvalidConfig = 40,
    ConfigNotFound = 41,
    TooManyApprovers = 42,
    TimeoutTooShort = 43,
    TimeoutTooLong = 44,

    // Batch operations
    BatchTooLarge = 50,
    BatchEmpty = 51,

    // Compliance
    ComplianceCheckFailed = 60,
    UnsupportedStandard = 61,

    // Sharing
    ShareLimitReached = 70,

    // General
    InvalidInput = 80,
    InternalError = 99,
}
