use soroban_sdk::contracterror;

/// Re-export standardized errors for backward compatibility
pub use crate::standardized_errors::StandardError;

/// Community-specific errors that extend the standard error set
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    // Forum specific errors (4000-4099)
    PostNotFound = 4000,
    ReplyNotFound = 4001,
    AlreadyVoted = 4002,
    CannotEditPost = 4003,
    PostClosed = 4004,

    // Mentorship specific errors (4100-4199)
    MentorNotAvailable = 4100,
    MentorshipNotFound = 4101,
    AlreadyMentor = 4102,
    MaxMenteesReached = 4103,
    InvalidMentorshipStatus = 4104,

    // Contribution specific errors (4200-4299)
    ContributionNotFound = 4200,
    InvalidContributionStatus = 4201,
    InsufficientReputation = 4202,

    // Event specific errors (4300-4399)
    EventNotFound = 4300,
    EventFull = 4301,
    AlreadyRegistered = 4302,
    EventNotActive = 4303,

    // Moderation specific errors (4400-4499)
    NotModerator = 4400,
    ReportNotFound = 4401,
    ReportLimitReached = 4402,
    AlreadyReported = 4403,

    // Governance specific errors (4500-4599)
    ProposalNotFound = 4500,
    VotingClosed = 4501,
    AlreadyVotedOnProposal = 4502,
    InsufficientVotingPower = 4503,
}

/// Error context for community operations
pub type CommunityErrorContext = crate::standardized_errors::ErrorContext;

/// Helper macro for community errors with context
#[macro_export]
macro_rules! community_error {
    ($error:expr, $operation:expr, $info:expr) => {
        $crate::standardized_errors::ErrorContext::new(
            $crate::standardized_errors::StandardError::from($error),
            $operation,
            "CommunityContract",
            $info,
        )
    };
}
