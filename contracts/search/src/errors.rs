use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum SearchError {
    /// Contract has already been initialized and cannot be re-initialized.
    AlreadyInitialized = 1,
    /// Contract has not been initialized yet.
    NotInitialized = 2,
    /// Caller does not have the required permissions to perform this action.
    Unauthorized = 3,
    /// The provided search query is malformed or empty.
    InvalidQuery = 4,
    /// The requested content item was not found in storage.
    ContentNotFound = 5,
    /// The provided semantic or visual metadata is invalid or malformed.
    InvalidMetadata = 6,
    /// The provided similarity or quality score is out of the valid range.
    InvalidScore = 7,
    /// The conversation session has expired or does not exist.
    SessionExpired = 8,
    /// The specified language code is not recognized or supported.
    InvalidLanguage = 9,
    /// The oracle address is not in the authorized oracle list.
    OracleNotAuthorized = 10,
}

/// Backward-compatible alias used by internal submodules.
pub type Error = SearchError;

impl SearchError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::AlreadyInitialized => "SRCH-001",
            Self::NotInitialized => "SRCH-002",
            Self::Unauthorized => "SRCH-003",
            Self::InvalidQuery => "SRCH-004",
            Self::ContentNotFound => "SRCH-005",
            Self::InvalidMetadata => "SRCH-006",
            Self::InvalidScore => "SRCH-007",
            Self::SessionExpired => "SRCH-008",
            Self::InvalidLanguage => "SRCH-009",
            Self::OracleNotAuthorized => "SRCH-010",
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            Self::AlreadyInitialized => "Search contract is already initialized",
            Self::NotInitialized => "Search contract is not initialized",
            Self::Unauthorized => "Caller is not authorized for this search operation",
            Self::InvalidQuery => "Search query is invalid",
            Self::ContentNotFound => "Requested search content was not found",
            Self::InvalidMetadata => "Search metadata is invalid",
            Self::InvalidScore => "Search score is outside the supported range",
            Self::SessionExpired => "Search session has expired",
            Self::InvalidLanguage => "Requested language is not supported",
            Self::OracleNotAuthorized => "Oracle is not authorized for this search callback",
        }
    }

    pub fn action(&self) -> &'static str {
        match self {
            Self::AlreadyInitialized => {
                "Reuse the existing search configuration instead of reinitializing it"
            }
            Self::NotInitialized => "Initialize the search contract before calling this function",
            Self::Unauthorized | Self::OracleNotAuthorized => {
                "Retry with an authorized caller or update the oracle permissions"
            }
            Self::InvalidQuery
            | Self::InvalidMetadata
            | Self::InvalidScore
            | Self::InvalidLanguage => "Correct the request payload and retry the search operation",
            Self::ContentNotFound => "Confirm the content identifier exists before retrying",
            Self::SessionExpired => "Create a new search session and retry the request",
        }
    }
}
