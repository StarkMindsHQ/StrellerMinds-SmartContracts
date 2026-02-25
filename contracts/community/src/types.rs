use soroban_sdk::{contracttype, Address, String, Vec};

// ───────────────────────────────────────────────
//  Forum & Discussion System
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ForumCategory {
    General,
    CourseSpecific,
    TechnicalHelp,
    CareerAdvice,
    ProjectShowcase,
    Announcements,
    Feedback,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum PostStatus {
    Active,
    Resolved,
    Closed,
    Pinned,
    Archived,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ForumPost {
    pub id: u64,
    pub author: Address,
    pub category: ForumCategory,
    pub title: String,
    pub content: String,
    pub status: PostStatus,
    pub created_at: u64,
    pub updated_at: u64,
    pub views: u32,
    pub replies_count: u32,
    pub upvotes: u32,
    pub downvotes: u32,
    pub is_pinned: bool,
    pub tags: Vec<String>,
    pub course_id: String, // empty if not course-specific
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ForumReply {
    pub id: u64,
    pub post_id: u64,
    pub author: Address,
    pub content: String,
    pub created_at: u64,
    pub updated_at: u64,
    pub upvotes: u32,
    pub downvotes: u32,
    pub is_solution: bool,
    pub parent_reply_id: u64, // 0 = top-level reply
}

// ───────────────────────────────────────────────
//  Mentorship System
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum MentorshipStatus {
    Pending,
    Active,
    Completed,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum MentorExpertise {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct MentorProfile {
    pub mentor: Address,
    pub expertise_areas: Vec<String>,
    pub expertise_level: MentorExpertise,
    pub max_mentees: u32,
    pub current_mentees: u32,
    pub total_sessions: u32,
    pub rating: u32, // 0-100
    pub is_available: bool,
    pub bio: String,
    pub joined_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct MentorshipRequest {
    pub id: u64,
    pub mentee: Address,
    pub mentor: Address,
    pub topic: String,
    pub message: String,
    pub status: MentorshipStatus,
    pub created_at: u64,
    pub started_at: u64,
    pub completed_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct MentorshipSession {
    pub id: u64,
    pub request_id: u64,
    pub mentor: Address,
    pub mentee: Address,
    pub topic: String,
    pub duration: u64, // seconds
    pub notes: String,
    pub rating: u32, // 0-100
    pub completed_at: u64,
}

// ───────────────────────────────────────────────
//  Knowledge Base & Contributions
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ContributionType {
    Article,
    Tutorial,
    CodeSnippet,
    Resource,
    FAQ,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ContributionStatus {
    Draft,
    Submitted,
    UnderReview,
    Approved,
    Rejected,
    Published,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct KnowledgeContribution {
    pub id: u64,
    pub contributor: Address,
    pub contribution_type: ContributionType,
    pub title: String,
    pub content: String,
    pub status: ContributionStatus,
    pub category: ForumCategory,
    pub tags: Vec<String>,
    pub upvotes: u32,
    pub views: u32,
    pub created_at: u64,
    pub published_at: u64,
    pub xp_reward: u32,
    pub token_reward: i128,
}

// ───────────────────────────────────────────────
//  Community Events
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum EventType {
    Workshop,
    Webinar,
    StudyGroup,
    Hackathon,
    Competition,
    Meetup,
    AMA, // Ask Me Anything
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum EventStatus {
    Scheduled,
    InProgress,
    Completed,
    Cancelled,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CommunityEvent {
    pub id: u64,
    pub organizer: Address,
    pub event_type: EventType,
    pub title: String,
    pub description: String,
    pub start_time: u64,
    pub end_time: u64,
    pub max_participants: u32,
    pub current_participants: u32,
    pub status: EventStatus,
    pub is_public: bool,
    pub xp_reward: u32,
    pub created_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct EventParticipant {
    pub user: Address,
    pub event_id: u64,
    pub registered_at: u64,
    pub attended: bool,
    pub feedback_rating: u32, // 0-100
}

// ───────────────────────────────────────────────
//  Moderation System
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ModeratorRole {
    Moderator,
    SeniorModerator,
    Admin,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ReportReason {
    Spam,
    Harassment,
    Inappropriate,
    OffTopic,
    Misinformation,
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ReportStatus {
    Pending,
    UnderReview,
    Resolved,
    Dismissed,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ContentReport {
    pub id: u64,
    pub reporter: Address,
    pub content_type: String, // "post", "reply", "contribution"
    pub content_id: u64,
    pub reason: ReportReason,
    pub description: String,
    pub status: ReportStatus,
    pub created_at: u64,
    pub resolved_at: u64,
    pub resolved_by: Address,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ModeratorAction {
    pub id: u64,
    pub moderator: Address,
    pub action_type: String, // "warn", "mute", "ban", "delete"
    pub target_user: Address,
    pub reason: String,
    pub duration: u64, // 0 = permanent
    pub created_at: u64,
}

// ───────────────────────────────────────────────
//  Community Analytics
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CommunityMetrics {
    pub total_posts: u32,
    pub total_replies: u32,
    pub total_contributions: u32,
    pub total_events: u32,
    pub active_mentorships: u32,
    pub total_members: u32,
    pub daily_active_users: u32,
    pub weekly_active_users: u32,
    pub avg_response_time: u64, // seconds
    pub resolution_rate: u32,   // 0-100
    pub last_updated: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UserCommunityStats {
    pub user: Address,
    pub posts_created: u32,
    pub replies_given: u32,
    pub solutions_provided: u32,
    pub contributions_made: u32,
    pub events_attended: u32,
    pub mentorship_sessions: u32,
    pub helpful_votes_received: u32,
    pub reputation_score: u32,
    pub joined_at: u64,
}

// ───────────────────────────────────────────────
//  Community Governance
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ProposalType {
    FeatureRequest,
    PolicyChange,
    CommunityRule,
    EventProposal,
    Other,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ProposalStatus {
    Draft,
    Active,
    Passed,
    Rejected,
    Implemented,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CommunityProposal {
    pub id: u64,
    pub proposer: Address,
    pub proposal_type: ProposalType,
    pub title: String,
    pub description: String,
    pub status: ProposalStatus,
    pub votes_for: u32,
    pub votes_against: u32,
    pub created_at: u64,
    pub voting_ends_at: u64,
    pub min_votes_required: u32,
}

// ───────────────────────────────────────────────
//  Configuration
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CommunityConfig {
    pub post_xp_reward: u32,
    pub reply_xp_reward: u32,
    pub solution_xp_reward: u32,
    pub contribution_base_xp: u32,
    pub contribution_base_tokens: i128,
    pub mentor_session_xp: u32,
    pub event_attendance_xp: u32,
    pub min_reputation_to_moderate: u32,
    pub max_reports_per_day: u32,
    pub vote_weight_threshold: u32,
}

// ───────────────────────────────────────────────
//  Storage Keys
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum CommunityKey {
    Admin,
    Config,

    // Counters
    PostCounter,
    ReplyCounter,
    ContributionCounter,
    EventCounter,
    ReportCounter,
    ProposalCounter,
    MentorshipCounter,
    SessionCounter,

    // Forum
    Post(u64),
    Reply(u64),
    PostReplies(u64),             // Vec<u64>
    CategoryPosts(ForumCategory), // Vec<u64>
    UserPosts(Address),           // Vec<u64>
    PostVote(Address, u64),       // user vote on post
    ReplyVote(Address, u64),      // user vote on reply

    // Mentorship
    MentorProfile(Address),
    MentorshipRequest(u64),
    UserMentorships(Address), // Vec<u64>
    MentorshipSession(u64),

    // Knowledge Base
    Contribution(u64),
    UserContributions(Address),           // Vec<u64>
    CategoryContributions(ForumCategory), // Vec<u64>

    // Events
    Event(u64),
    EventParticipants(u64), // Vec<Address>
    UserEvents(Address),    // Vec<u64>
    EventParticipant(Address, u64),

    // Moderation
    Moderator(Address),
    Report(u64),
    PendingReports, // Vec<u64>
    ModeratorAction(u64),
    UserActions(Address), // Vec<u64>

    // Analytics
    CommunityMetrics,
    UserStats(Address),

    // Governance
    Proposal(u64),
    ActiveProposals, // Vec<u64>
    ProposalVote(Address, u64),
}
