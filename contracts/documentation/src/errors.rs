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
