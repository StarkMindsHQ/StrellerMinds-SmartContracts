use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    NotFound = 4,
    InvalidInput = 5,

    // Forum errors
    PostNotFound = 10,
    ReplyNotFound = 11,
    AlreadyVoted = 12,
    CannotEditPost = 13,
    PostClosed = 14,

    // Mentorship errors
    MentorNotAvailable = 20,
    MentorshipNotFound = 21,
    AlreadyMentor = 22,
    MaxMenteesReached = 23,
    InvalidMentorshipStatus = 24,

    // Contribution errors
    ContributionNotFound = 30,
    InvalidContributionStatus = 31,
    InsufficientReputation = 32,

    // Event errors
    EventNotFound = 40,
    EventFull = 41,
    AlreadyRegistered = 42,
    EventNotActive = 43,

    // Moderation errors
    NotModerator = 50,
    ReportNotFound = 51,
    ReportLimitReached = 52,
    AlreadyReported = 53,

    // Governance errors
    ProposalNotFound = 60,
    VotingClosed = 61,
    AlreadyVotedOnProposal = 62,
    InsufficientVotingPower = 63,
}
