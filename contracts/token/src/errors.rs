use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum TokenError {
    /// Contract has already been initialized; `initialize` may only be called once.
    AlreadyInitialized = 1,
    /// Contract has not been initialized; call `initialize` first.
    NotInitialized = 2,
    /// Caller does not have the required admin privileges.
    Unauthorized = 10,
    /// Provided token amount is zero or otherwise invalid.
    InvalidAmount = 20,
    /// Provided address is not a valid account address.
    InvalidAddress = 21,
    /// Sender does not hold enough tokens to complete the transfer.
    InsufficientBalance = 80,
    /// Token transfer could not be completed.
    TransferFailed = 81,
    RateLimitExceeded = 100,
}

impl TokenError {
    pub fn code(&self) -> &'static str {
        match self {
            Self::AlreadyInitialized => "TKN-001",
            Self::NotInitialized => "TKN-002",
            Self::Unauthorized => "TKN-010",
            Self::InvalidAmount => "TKN-020",
            Self::InvalidAddress => "TKN-021",
            Self::InsufficientBalance => "TKN-080",
            Self::TransferFailed => "TKN-081",
        }
    }

    pub fn message(&self) -> &'static str {
        match self {
            Self::AlreadyInitialized => "Token contract is already initialized",
            Self::NotInitialized => "Token contract is not initialized",
            Self::Unauthorized => "Caller is not authorized for this token operation",
            Self::InvalidAmount => "Provided token amount is invalid",
            Self::InvalidAddress => "Provided token address is invalid",
            Self::InsufficientBalance => "Account balance is too low for this transfer",
            Self::TransferFailed => "Token transfer could not be completed",
        }
    }

    pub fn action(&self) -> &'static str {
        match self {
            Self::AlreadyInitialized => {
                "Reuse the existing token state instead of reinitializing it"
            }
            Self::NotInitialized => "Initialize the token contract before calling this function",
            Self::Unauthorized => {
                "Retry with an authorized account or update the contract permissions"
            }
            Self::InvalidAmount => "Provide a positive amount that matches the token rules",
            Self::InvalidAddress => "Retry with a valid Stellar address",
            Self::InsufficientBalance => {
                "Reduce the amount or fund the source account before retrying"
            }
            Self::TransferFailed => {
                "Check balances, approvals, and contract state, then retry the transfer"
            }
        }
    }
}
