use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum DocumentationError {
    /// Contract has not been initialized yet.
    NotInitialized = 1,
    /// Contract has already been initialized and cannot be re-initialized.
    AlreadyInitialized = 2,
    /// Caller does not have the required permissions to perform this action.
    Unauthorized = 3,
    /// The requested document was not found in storage.
    DocumentNotFound = 4,
    /// The document data is malformed or fails validation.
    InvalidDocument = 5,
    /// The requested document version was not found.
    VersionNotFound = 6,
    /// The requested community contribution was not found.
    ContributionNotFound = 7,
    /// The contribution data is malformed or fails validation.
    InvalidContribution = 8,
    /// The requested translation was not found.
    TranslationNotFound = 9,
    /// The specified language code is not recognized or supported.
    InvalidLanguage = 10,
    /// The document content exceeds the configured maximum size limit.
    DocumentTooLarge = 11,
    /// The provided status transition is not valid for the current state.
    InvalidStatus = 12,
    /// A document, translation, or contribution with this ID already exists.
    AlreadyExists = 13,
    RateLimitExceeded = 14,
}

/// Backward-compatible alias used by internal modules.
pub type Error = DocumentationError;

impl DocumentationError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::NotInitialized => "DOC-001",
            Self::AlreadyInitialized => "DOC-002",
            Self::Unauthorized => "DOC-003",
            Self::DocumentNotFound => "DOC-004",
            Self::InvalidDocument => "DOC-005",
            Self::VersionNotFound => "DOC-006",
            Self::ContributionNotFound => "DOC-007",
            Self::InvalidContribution => "DOC-008",
            Self::TranslationNotFound => "DOC-009",
            Self::InvalidLanguage => "DOC-010",
            Self::DocumentTooLarge => "DOC-011",
            Self::InvalidStatus => "DOC-012",
            Self::AlreadyExists => "DOC-013",
            Self::RateLimitExceeded => "DOC-014",
        }
    }

    pub fn action(&self) -> &'static str {
        match self {
            Self::Unauthorized => {
                "Retry with an authorized documentation administrator or contributor"
            }
            Self::DocumentNotFound
            | Self::VersionNotFound
            | Self::ContributionNotFound
            | Self::TranslationNotFound => {
                "Verify the referenced documentation resource exists before retrying"
            }
            Self::InvalidDocument
            | Self::InvalidContribution
            | Self::InvalidLanguage
            | Self::InvalidStatus => {
                "Correct the request data and retry the documentation operation"
            }
            Self::DocumentTooLarge => "Reduce the document size to the configured limit and retry",
            Self::NotInitialized => {
                "Initialize the documentation contract before calling this function"
            }
            Self::AlreadyInitialized | Self::AlreadyExists => {
                "Reuse the existing resource instead of creating a duplicate"
            }
            Self::RateLimitExceeded => {
                "Wait for the rate limit window to reset before retrying"
            }
        }
    }
}
