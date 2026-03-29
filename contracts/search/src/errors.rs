use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SearchError {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidQuery = 4,
    ContentNotFound = 5,
    InvalidMetadata = 6,
    InvalidScore = 7,
    SessionExpired = 8,
    InvalidLanguage = 9,
    OracleNotAuthorized = 10,
}

/// Backward-compatible alias used by internal submodules.
pub type Error = SearchError;
