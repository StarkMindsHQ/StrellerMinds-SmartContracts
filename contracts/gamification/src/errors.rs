use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum GamificationError {
    /// Contract has already been initialized and cannot be re-initialized.
    AlreadyInitialized = 1,
    /// Contract has not been initialized yet.
    NotInitialized = 2,
    /// Caller does not have the required admin or ownership privileges.
    Unauthorized = 3,
    /// A token or XP amount is zero or otherwise invalid.
    InvalidAmount = 4,
    /// One or more input fields fail validation.
    InvalidInput = 5,
    /// The requested resource (achievement, guild, season, etc.) was not found.
    NotFound = 6,
    /// A resource with the same identifier already exists.
    AlreadyExists = 7,
    /// The challenge has reached its maximum participant count.
    ChallengeFull = 8,
    /// The challenge is not currently active.
    ChallengeInactive = 9,
    /// The challenge's deadline has passed.
    ChallengeExpired = 10,
    /// The challenge's start time has not been reached yet.
    ChallengeNotStarted = 11,
    /// The guild has reached its maximum member count.
    GuildFull = 12,
    /// The user is already a member of a guild.
    AlreadyInGuild = 13,
    /// The user is not a member of any guild.
    NotInGuild = 14,
    /// A season is already running and a new one cannot be started.
    SeasonAlreadyActive = 15,
    /// There is no active season to interact with.
    SeasonInactive = 16,
    /// A user attempted to endorse themselves.
    SelfEndorsement = 17,
    /// The daily endorsement limit for the caller has been reached.
    EndorsementLimitReached = 18,
    /// The reward for this achievement has already been claimed.
    AchievementAlreadyClaimed = 19,
    /// A prerequisite achievement has not been earned yet.
    PrerequisiteNotMet = 20,
    /// The user has already joined this challenge.
    AlreadyJoinedChallenge = 21,
    /// The user has not joined this challenge.
    NotJoinedChallenge = 22,
    /// The provided guild name exceeds the maximum allowed length.
    GuildNameTooLong = 23,
    /// The season's end time has not been reached, so it cannot be closed.
    SeasonNotEnded = 24,
    /// The user does not have enough XP to perform the requested action.
    InsufficientXP = 25,
}

/// Backward-compatible alias used by internal submodules.
pub type Error = GamificationError;
