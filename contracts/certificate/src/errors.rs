use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum CertificateError {
    // Initialization
    /// Contract has already been initialized.
    AlreadyInitialized = 1,
    /// Contract has not been initialized yet.
    NotInitialized = 2,
    /// Caller is not authorized to perform this operation.
    Unauthorized = 3,

    // Multi-sig
    /// The specified multi-sig request does not exist.
    MultiSigRequestNotFound = 10,
    /// The multi-sig request has passed its deadline and can no longer be processed.
    MultiSigRequestExpired = 11,
    /// The caller is not in the list of authorized approvers for this request.
    ApproverNotAuthorized = 12,
    /// The request does not have enough approvals to be executed.
    InsufficientApprovals = 13,
    /// The approval threshold is zero or exceeds the number of available approvers.
    InvalidApprovalThreshold = 14,
    /// This approver has already submitted a decision for the request.
    AlreadyApproved = 15,
    /// The request is not in the pending state required for this operation.
    RequestNotPending = 16,
    /// The request has already been executed and cannot be executed again.
    RequestAlreadyExecuted = 17,

    // Certificate lifecycle
    /// No certificate was found with the given ID.
    CertificateNotFound = 20,
    /// A certificate with this ID has already been issued.
    CertificateAlreadyExists = 21,
    /// The certificate has been revoked and cannot be used for this operation.
    CertificateRevoked = 22,
    /// The certificate has passed its expiry date.
    CertificateExpired = 23,
    /// The certificate is not revoked or is not marked as eligible for reissuance.
    CertificateNotEligibleForReissue = 24,

    // Template
    /// No template was found with the given ID.
    TemplateNotFound = 30,
    /// A template with this ID already exists.
    TemplateAlreadyExists = 31,
    /// The template is inactive and cannot be used to issue certificates.
    TemplateInactive = 32,
    /// Not all required template fields were provided.
    MissingRequiredField = 33,

    // Configuration
    /// The provided configuration contains invalid values.
    InvalidConfig = 40,
    /// No multi-sig configuration was found for the specified course.
    ConfigNotFound = 41,
    /// The approver list exceeds the maximum allowed number of approvers.
    TooManyApprovers = 42,
    /// The timeout duration is below the minimum allowed value.
    TimeoutTooShort = 43,
    /// The timeout duration exceeds the maximum allowed value.
    TimeoutTooLong = 44,

    // Batch operations
    /// The batch size exceeds the maximum number of certificates allowed per call.
    BatchTooLarge = 50,
    /// The batch list is empty and contains no certificates to issue.
    BatchEmpty = 51,

    // Compliance
    /// The compliance check could not be completed successfully.
    ComplianceCheckFailed = 60,
    /// The specified compliance standard is not supported by this contract.
    UnsupportedStandard = 61,

    // Sharing
    /// The certificate has reached the maximum number of allowed share records.
    ShareLimitReached = 70,

    // General
    /// One or more input values are invalid.
    InvalidInput = 80,
    /// An unexpected internal error occurred.
    InternalError = 99,

    // Rate limiting
    RateLimitExceeded = 90,

    // ZKP
    /// The provided zero-knowledge proof is invalid or malformed.
    InvalidProof = 100,
    /// The zero-knowledge verification process failed.
    VerificationFailed = 101,
}

impl CertificateError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::AlreadyInitialized => "CERT-001",
            Self::NotInitialized => "CERT-002",
            Self::Unauthorized => "CERT-003",
            Self::MultiSigRequestNotFound => "CERT-010",
            Self::MultiSigRequestExpired => "CERT-011",
            Self::ApproverNotAuthorized => "CERT-012",
            Self::InsufficientApprovals => "CERT-013",
            Self::InvalidApprovalThreshold => "CERT-014",
            Self::AlreadyApproved => "CERT-015",
            Self::RequestNotPending => "CERT-016",
            Self::RequestAlreadyExecuted => "CERT-017",
            Self::CertificateNotFound => "CERT-020",
            Self::CertificateAlreadyExists => "CERT-021",
            Self::CertificateRevoked => "CERT-022",
            Self::CertificateExpired => "CERT-023",
            Self::CertificateNotEligibleForReissue => "CERT-024",
            Self::TemplateNotFound => "CERT-030",
            Self::TemplateAlreadyExists => "CERT-031",
            Self::TemplateInactive => "CERT-032",
            Self::MissingRequiredField => "CERT-033",
            Self::InvalidConfig => "CERT-040",
            Self::ConfigNotFound => "CERT-041",
            Self::TooManyApprovers => "CERT-042",
            Self::TimeoutTooShort => "CERT-043",
            Self::TimeoutTooLong => "CERT-044",
            Self::BatchTooLarge => "CERT-050",
            Self::BatchEmpty => "CERT-051",
            Self::ComplianceCheckFailed => "CERT-060",
            Self::UnsupportedStandard => "CERT-061",
            Self::ShareLimitReached => "CERT-070",
            Self::InvalidInput => "CERT-080",
            Self::InternalError => "CERT-099",
            Self::RateLimitExceeded => "CERT-090",
            Self::InvalidProof => "CERT-100",
            Self::VerificationFailed => "CERT-101",
        }
    }

    pub fn action(&self) -> &'static str {
        match self {
            Self::Unauthorized | Self::ApproverNotAuthorized => {
                "Retry with an authorized certificate administrator or approver"
            }
            Self::CertificateNotFound | Self::TemplateNotFound | Self::ConfigNotFound => {
                "Verify the referenced certificate, template, or configuration exists before retrying"
            }
            Self::InvalidApprovalThreshold
            | Self::InvalidConfig
            | Self::InvalidInput
            | Self::MissingRequiredField
            | Self::TimeoutTooShort
            | Self::TimeoutTooLong => {
                "Correct the request payload or configuration and submit the request again"
            }
            Self::BatchTooLarge | Self::BatchEmpty => {
                "Resize the batch to a supported non-empty range and retry"
            }
            Self::ComplianceCheckFailed | Self::UnsupportedStandard => {
                "Review compliance requirements and supported standards before retrying"
            }
            Self::InvalidProof | Self::VerificationFailed => {
                "Verify the zero-knowledge proof data and re-generate the proof if necessary"
            }
            _ => "Review the certificate workflow state and retry the next valid operation",
        }
    }
}
