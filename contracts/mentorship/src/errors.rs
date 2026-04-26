use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum MentorshipError {
    Unauthorized = 1,
    AlreadyRegistered = 2,
    NotRegistered = 3,
    InvalidRating = 4,
    SessionNotFound = 5,
    InvalidStatusTransition = 6,
    SelfMentorshipNotAllowed = 7,
}
