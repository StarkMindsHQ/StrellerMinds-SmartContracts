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
