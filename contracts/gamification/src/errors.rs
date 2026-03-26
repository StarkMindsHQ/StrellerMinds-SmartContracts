use soroban_sdk::contracterror;

/// Re-export standardized errors for backward compatibility
pub use crate::standardized_errors::StandardError;

/// Gamification-specific errors that extend the standard error set
#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // Amount specific errors (8000-8099)
    InvalidAmount = 8000,

    // Challenge specific errors (8100-8199)
    ChallengeFull = 8100,
    ChallengeInactive = 8101,
    ChallengeExpired = 8102,
    ChallengeNotStarted = 8103,
    AlreadyJoinedChallenge = 8104,
    NotJoinedChallenge = 8105,

    // Guild specific errors (8200-8299)
    GuildFull = 8200,
    AlreadyInGuild = 8201,
    NotInGuild = 8202,
    GuildNameTooLong = 8203,

    // Season specific errors (8300-8399)
    SeasonAlreadyActive = 8300,
    SeasonInactive = 8301,
    SeasonNotEnded = 8302,

    // Endorsement specific errors (8400-8499)
    SelfEndorsement = 8400,
    EndorsementLimitReached = 8401,

    // Achievement specific errors (8500-8599)
    AchievementAlreadyClaimed = 8500,
    PrerequisiteNotMet = 8501,
    InsufficientXP = 8502,
}

/// Error context for gamification operations
pub type GamificationErrorContext = crate::standardized_errors::ErrorContext;

/// Helper macro for gamification errors with context
#[macro_export]
macro_rules! gamification_error {
    ($error:expr, $operation:expr, $info:expr) => {
        $crate::standardized_errors::ErrorContext::new(
            $crate::standardized_errors::StandardError::from($error),
            $operation,
            "GamificationContract",
            $info,
        )
    };
}
