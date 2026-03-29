use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
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
            _ => "Review the certificate workflow state and retry the next valid operation",
        }
    }
}
