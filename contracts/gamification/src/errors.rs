use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    InvalidInput = 5,
    NotFound = 6,
    AlreadyExists = 7,
    ChallengeFull = 8,
    ChallengeInactive = 9,
    ChallengeExpired = 10,
    ChallengeNotStarted = 11,
    GuildFull = 12,
    AlreadyInGuild = 13,
    NotInGuild = 14,
    SeasonAlreadyActive = 15,
    SeasonInactive = 16,
    SelfEndorsement = 17,
    EndorsementLimitReached = 18,
    AchievementAlreadyClaimed = 19,
    PrerequisiteNotMet = 20,
    AlreadyJoinedChallenge = 21,
    NotJoinedChallenge = 22,
    GuildNameTooLong = 23,
    SeasonNotEnded = 24,
    InsufficientXP = 25,
}
