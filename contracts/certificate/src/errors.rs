use soroban_sdk::contracterror;

/// Re-export standardized errors for backward compatibility
pub use crate::standardized_errors::StandardError;

/// Certificate-specific errors that extend the standard error set
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum CertificateError {
    // Multi-sig specific errors (3000-3099)
    MultiSigRequestNotFound = 3000,
    MultiSigRequestExpired = 3001,
    ApproverNotAuthorized = 3002,
    InsufficientApprovals = 3003,
    InvalidApprovalThreshold = 3004,
    AlreadyApproved = 3005,
    RequestNotPending = 3006,
    RequestAlreadyExecuted = 3007,

    // Certificate lifecycle specific errors (3100-3199)
    CertificateRevoked = 3100,
    CertificateExpired = 3101,
    CertificateNotEligibleForReissue = 3102,

    // Template specific errors (3200-3299)
    TemplateInactive = 3200,

    // Configuration specific errors (3300-3399)
    TooManyApprovers = 3300,
    TimeoutTooShort = 3301,
    TimeoutTooLong = 3302,

    // Batch operations specific errors (3400-3499)
    BatchTooLarge = 3400,
    BatchEmpty = 3401,

    // Compliance specific errors (3500-3599)
    ComplianceCheckFailed = 3500,
    UnsupportedStandard = 3501,

    // Sharing specific errors (3600-3699)
    ShareLimitReached = 3600,
}

/// Error context for certificate operations
pub type CertificateErrorContext = crate::standardized_errors::ErrorContext;

/// Helper macro for certificate errors with context
#[macro_export]
macro_rules! certificate_error {
    ($error:expr, $operation:expr, $info:expr) => {
        $crate::standardized_errors::ErrorContext::new(
            $crate::standardized_errors::StandardError::from($error),
            $operation,
            "CertificateContract",
            $info,
        )
    };
}
