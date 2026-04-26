use soroban_sdk::{contracttype, Address, String, Vec};

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MentorshipStatus {
    Pending,
    Active,
    Completed,
    Cancelled,
    Rejected,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MentorProfile {
    pub address: Address,
    pub name: String,
    pub expertise: Vec<String>,
    pub bio: String,
    pub rating_sum: u32,
    pub rating_count: u32,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MentorshipSession {
    pub id: u64,
    pub mentor: Address,
    pub mentee: Address,
    pub status: MentorshipStatus,
    pub start_time: u64,
    pub end_time: u64,
    pub notes: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Review {
    pub reviewer: Address,
    pub target: Address,
    pub rating: u32, // 1-5
    pub comment: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MentorshipDataKey {
    Admin,
    Mentor(Address),
    MenteeRequests(Address),
    MentorRequests(Address),
    Session(u64),
    SessionCounter,
    Review(Address, Address), // (reviewer, target)
}
