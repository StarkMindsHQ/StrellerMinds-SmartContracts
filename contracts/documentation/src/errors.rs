use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum DocumentationError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    Unauthorized = 3,
    DocumentNotFound = 4,
    InvalidDocument = 5,
    VersionNotFound = 6,
    ContributionNotFound = 7,
    InvalidContribution = 8,
    TranslationNotFound = 9,
    InvalidLanguage = 10,
    DocumentTooLarge = 11,
    InvalidStatus = 12,
    AlreadyExists = 13,
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
        }
    }
}
