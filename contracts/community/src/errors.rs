use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum CommunityError {
    /// Contract has already been initialized and cannot be re-initialized.
    AlreadyInitialized = 1,
    /// Contract has not been initialized yet.
    NotInitialized = 2,
    /// Caller does not have the required privileges to perform this action.
    Unauthorized = 3,
    /// The requested generic resource was not found.
    NotFound = 4,
    /// One or more input fields fail validation.
    InvalidInput = 5,

    // Forum errors
    /// The specified forum post does not exist.
    PostNotFound = 10,
    /// The specified forum reply does not exist.
    ReplyNotFound = 11,
    /// The user has already cast a vote on this content.
    AlreadyVoted = 12,
    /// The post cannot be edited in its current state.
    CannotEditPost = 13,
    /// The post is closed and no longer accepting replies or votes.
    PostClosed = 14,

    // Mentorship errors
    /// The mentor is at capacity or has not made themselves available.
    MentorNotAvailable = 20,
    /// The specified mentorship request does not exist.
    MentorshipNotFound = 21,
    /// The user is already registered as a mentor.
    AlreadyMentor = 22,
    /// The mentor has reached their maximum allowed number of concurrent mentees.
    MaxMenteesReached = 23,
    /// The mentorship request is in a state that does not allow this transition.
    InvalidMentorshipStatus = 24,

    // Contribution errors
    /// The specified knowledge contribution does not exist.
    ContributionNotFound = 30,
    /// The contribution is in a state that does not allow this operation.
    InvalidContributionStatus = 31,
    /// The user does not have enough reputation to perform this action.
    InsufficientReputation = 32,

    // Event errors
    /// The specified community event does not exist.
    EventNotFound = 40,
    /// The event has reached its maximum participant count.
    EventFull = 41,
    /// The user is already registered for this event.
    AlreadyRegistered = 42,
    /// The event is not in an active state that allows this operation.
    EventNotActive = 43,

    // Moderation errors
    /// The caller does not hold a moderator role.
    NotModerator = 50,
    /// The specified content report does not exist.
    ReportNotFound = 51,
    /// The reporter has reached their daily report submission limit.
    ReportLimitReached = 52,
    /// The reporter has already filed a report against this specific content.
    AlreadyReported = 53,

    // Governance errors
    /// The specified governance proposal does not exist.
    ProposalNotFound = 60,
    /// The voting window for this proposal has expired or not yet opened.
    VotingClosed = 61,
    /// The user has already voted on this proposal.
    AlreadyVotedOnProposal = 62,
    /// The user does not have enough voting power to participate.
    InsufficientVotingPower = 63,

    // Rate limiting errors
    RateLimitExceeded = 70,
}

/// Backward-compatible alias used by internal submodules.
pub type Error = CommunityError;
