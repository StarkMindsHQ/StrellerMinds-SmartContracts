use soroban_sdk::{contracttype, Address, String, Vec};

// ───────────────────────────────────────────────
//  Forum & Discussion System
// ───────────────────────────────────────────────

/// Top-level category that a forum post or knowledge contribution belongs to.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ForumCategory {
    /// General discussion not tied to a specific topic.
    General,
    /// Discussion related to a specific course.
    CourseSpecific,
    /// Questions and answers about technical problems.
    TechnicalHelp,
    /// Guidance and discussion around career development.
    CareerAdvice,
    /// Members showcasing their projects and work.
    ProjectShowcase,
    /// Official platform announcements.
    Announcements,
    /// Feedback and suggestions for improvement.
    Feedback,
}

/// Lifecycle status of a forum post.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum PostStatus {
    /// Post is open and accepting replies.
    Active,
    /// The original question or issue has been resolved.
    Resolved,
    /// Post is closed and no longer accepting replies.
    Closed,
    /// Post has been pinned to the top of its category by a moderator.
    Pinned,
    /// Post has been archived for historical reference.
    Archived,
}

/// A forum post created by a community member.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ForumPost {
    /// Unique numeric identifier for the post.
    pub id: u64,
    /// Address of the member who created the post.
    pub author: Address,
    /// Category this post belongs to.
    pub category: ForumCategory,
    /// Short title summarising the post.
    pub title: String,
    /// Full body content of the post.
    pub content: String,
    /// Current lifecycle status.
    pub status: PostStatus,
    /// Unix timestamp (seconds) when the post was created.
    pub created_at: u64,
    /// Unix timestamp (seconds) when the post was last edited.
    pub updated_at: u64,
    /// Number of times this post has been viewed.
    pub views: u32,
    /// Total number of replies on this post.
    pub replies_count: u32,
    /// Number of upvotes received.
    pub upvotes: u32,
    /// Number of downvotes received.
    pub downvotes: u32,
    /// Whether the post is pinned to the top of its category.
    pub is_pinned: bool,
    /// User-supplied tags for discoverability.
    pub tags: Vec<String>,
    /// Course identifier; empty if not course-specific.
    pub course_id: String,
}

/// A reply to a forum post or another reply.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ForumReply {
    /// Unique numeric identifier for the reply.
    pub id: u64,
    /// Identifier of the post this reply belongs to.
    pub post_id: u64,
    /// Address of the member who wrote the reply.
    pub author: Address,
    /// Full body content of the reply.
    pub content: String,
    /// Unix timestamp (seconds) when the reply was created.
    pub created_at: u64,
    /// Unix timestamp (seconds) when the reply was last edited.
    pub updated_at: u64,
    /// Number of upvotes received.
    pub upvotes: u32,
    /// Number of downvotes received.
    pub downvotes: u32,
    /// Whether this reply was marked as the accepted solution by the post author.
    pub is_solution: bool,
    /// Identifier of the parent reply; 0 means this is a top-level reply.
    pub parent_reply_id: u64,
}

// ───────────────────────────────────────────────
//  Mentorship System
// ───────────────────────────────────────────────

/// Lifecycle status of a mentorship engagement.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum MentorshipStatus {
    /// The mentee's request has been submitted but not yet accepted.
    Pending,
    /// The mentorship is ongoing.
    Active,
    /// The mentorship has been successfully concluded.
    Completed,
    /// The mentorship was cancelled before completion.
    Cancelled,
}

/// Self-reported expertise level of a mentor.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum MentorExpertise {
    /// Suitable for introductory topics.
    Beginner,
    /// Comfortable with mid-level concepts.
    Intermediate,
    /// Proficient in complex subject matter.
    Advanced,
    /// Deep domain knowledge and practical mastery.
    Expert,
}

/// Public profile of a registered mentor.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct MentorProfile {
    /// Address of the mentor.
    pub mentor: Address,
    /// List of topics the mentor is willing to cover.
    pub expertise_areas: Vec<String>,
    /// Overall expertise level of the mentor.
    pub expertise_level: MentorExpertise,
    /// Maximum number of mentees the mentor will accept concurrently.
    pub max_mentees: u32,
    /// Number of active mentees the mentor currently has.
    pub current_mentees: u32,
    /// Total number of completed mentorship sessions.
    pub total_sessions: u32,
    /// Community rating on a 0–100 scale.
    pub rating: u32,
    /// Whether the mentor is currently accepting new mentees.
    pub is_available: bool,
    /// Short biography provided by the mentor.
    pub bio: String,
    /// Unix timestamp (seconds) when the mentor registered.
    pub joined_at: u64,
}

/// A mentee's request for mentorship from a specific mentor.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct MentorshipRequest {
    /// Unique numeric identifier for the request.
    pub id: u64,
    /// Address of the student seeking mentorship.
    pub mentee: Address,
    /// Address of the mentor being requested.
    pub mentor: Address,
    /// Topic or subject the mentee wants to be guided on.
    pub topic: String,
    /// Introductory message from the mentee to the mentor.
    pub message: String,
    /// Current lifecycle status of the request.
    pub status: MentorshipStatus,
    /// Unix timestamp (seconds) when the request was submitted.
    pub created_at: u64,
    /// Unix timestamp (seconds) when the mentorship became active; 0 if not yet started.
    pub started_at: u64,
    /// Unix timestamp (seconds) when the mentorship was completed; 0 if ongoing.
    pub completed_at: u64,
}

/// Record of a completed mentorship session between a mentor and mentee.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct MentorshipSession {
    /// Unique numeric identifier for the session.
    pub id: u64,
    /// Identifier of the mentorship request this session belongs to.
    pub request_id: u64,
    /// Address of the mentor.
    pub mentor: Address,
    /// Address of the mentee.
    pub mentee: Address,
    /// Topic covered during the session.
    pub topic: String,
    /// Duration of the session in seconds.
    pub duration: u64,
    /// Notes recorded during or after the session.
    pub notes: String,
    /// Mentee's rating of the session on a 0–100 scale.
    pub rating: u32,
    /// Unix timestamp (seconds) when the session ended.
    pub completed_at: u64,
}

// ───────────────────────────────────────────────
//  Knowledge Base & Contributions
// ───────────────────────────────────────────────

/// Format or type of a knowledge base contribution.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ContributionType {
    /// A long-form written article.
    Article,
    /// A step-by-step instructional tutorial.
    Tutorial,
    /// A reusable snippet of code.
    CodeSnippet,
    /// An external link or reference material.
    Resource,
    /// A frequently asked question with its answer.
    FAQ,
}

/// Lifecycle status of a knowledge base contribution.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ContributionStatus {
    /// Work in progress; not yet submitted for review.
    Draft,
    /// Submitted and awaiting review assignment.
    Submitted,
    /// Actively being reviewed by a moderator.
    UnderReview,
    /// Approved by a reviewer but not yet visible to all users.
    Approved,
    /// Rejected during review.
    Rejected,
    /// Published and visible to the community.
    Published,
}

/// A community-submitted knowledge base entry.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct KnowledgeContribution {
    /// Unique numeric identifier for the contribution.
    pub id: u64,
    /// Address of the member who submitted the contribution.
    pub contributor: Address,
    /// Format or category of the contribution.
    pub contribution_type: ContributionType,
    /// Title of the contribution.
    pub title: String,
    /// Full body content of the contribution.
    pub content: String,
    /// Current review and publication status.
    pub status: ContributionStatus,
    /// Forum category the contribution is classified under.
    pub category: ForumCategory,
    /// Tags for discoverability.
    pub tags: Vec<String>,
    /// Number of upvotes received from community members.
    pub upvotes: u32,
    /// Number of times the contribution has been viewed.
    pub views: u32,
    /// Unix timestamp (seconds) when the contribution was created.
    pub created_at: u64,
    /// Unix timestamp (seconds) when the contribution was published; 0 if not published.
    pub published_at: u64,
    /// Amount of XP awarded to the contributor upon publication.
    pub xp_reward: u32,
    /// Amount of tokens awarded to the contributor upon publication.
    pub token_reward: i128,
}

// ───────────────────────────────────────────────
//  Community Events
// ───────────────────────────────────────────────

/// Format or type of a community event.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum EventType {
    /// Hands-on workshop session.
    Workshop,
    /// Online seminar with a presenter.
    Webinar,
    /// Collaborative group study session.
    StudyGroup,
    /// Competitive hackathon event.
    Hackathon,
    /// Formal competition among participants.
    Competition,
    /// Informal in-person or virtual meetup.
    Meetup,
    /// Open Q&A session with a guest or expert.
    AMA,
}

/// Lifecycle status of a community event.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum EventStatus {
    /// Event is upcoming and open for registration.
    Scheduled,
    /// Event is currently taking place.
    InProgress,
    /// Event has ended.
    Completed,
    /// Event was cancelled before it started.
    Cancelled,
}

/// A community-organised event that members can register for.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CommunityEvent {
    /// Unique numeric identifier for the event.
    pub id: u64,
    /// Address of the member who organised the event.
    pub organizer: Address,
    /// Format or type of the event.
    pub event_type: EventType,
    /// Short title of the event.
    pub title: String,
    /// Detailed description of the event's content and goals.
    pub description: String,
    /// Unix timestamp (seconds) when the event begins.
    pub start_time: u64,
    /// Unix timestamp (seconds) when the event ends.
    pub end_time: u64,
    /// Maximum number of participants allowed; 0 means unlimited.
    pub max_participants: u32,
    /// Number of members currently registered.
    pub current_participants: u32,
    /// Current lifecycle status of the event.
    pub status: EventStatus,
    /// Whether the event is open to all community members.
    pub is_public: bool,
    /// XP awarded to members who attend the event.
    pub xp_reward: u32,
    /// Unix timestamp (seconds) when the event was created.
    pub created_at: u64,
}

/// Registration and attendance record for a single participant at an event.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct EventParticipant {
    /// Address of the participant.
    pub user: Address,
    /// Identifier of the event the participant registered for.
    pub event_id: u64,
    /// Unix timestamp (seconds) when the participant registered.
    pub registered_at: u64,
    /// Whether the participant attended the event.
    pub attended: bool,
    /// Participant's feedback rating for the event on a 0–100 scale.
    pub feedback_rating: u32,
}

// ───────────────────────────────────────────────
//  Moderation System
// ───────────────────────────────────────────────

/// Role assigned to a community moderator.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ModeratorRole {
    /// Standard moderator with basic moderation capabilities.
    Moderator,
    /// Elevated moderator with additional permissions.
    SeniorModerator,
    /// Full administrative access over the community contract.
    Admin,
}

/// Reason provided when a community member reports content.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ReportReason {
    /// Content is unsolicited advertising or repetitive noise.
    Spam,
    /// Content targets or demeans another member.
    Harassment,
    /// Content violates community standards.
    Inappropriate,
    /// Content is not relevant to the forum category or discussion.
    OffTopic,
    /// Content contains factually incorrect or misleading information.
    Misinformation,
    /// A reason not covered by the above categories.
    Other,
}

/// Lifecycle status of a content moderation report.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ReportStatus {
    /// Report has been filed but not yet assigned to a moderator.
    Pending,
    /// A moderator is actively reviewing the report.
    UnderReview,
    /// The report has been actioned and closed.
    Resolved,
    /// The report was reviewed and found to not warrant action.
    Dismissed,
}

/// A community member's report of problematic content.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ContentReport {
    /// Unique numeric identifier for the report.
    pub id: u64,
    /// Address of the member who filed the report.
    pub reporter: Address,
    /// Type of content being reported (e.g., "post", "reply", "contribution").
    pub content_type: String,
    /// Identifier of the reported content item.
    pub content_id: u64,
    /// Reason given for the report.
    pub reason: ReportReason,
    /// Additional context provided by the reporter.
    pub description: String,
    /// Current status of the report.
    pub status: ReportStatus,
    /// Unix timestamp (seconds) when the report was filed.
    pub created_at: u64,
    /// Unix timestamp (seconds) when the report was resolved; 0 if unresolved.
    pub resolved_at: u64,
    /// Address of the moderator who resolved the report.
    pub resolved_by: Address,
}

/// Record of an action taken by a moderator against a community member.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ModeratorAction {
    /// Unique numeric identifier for the action.
    pub id: u64,
    /// Address of the moderator who performed the action.
    pub moderator: Address,
    /// Type of action taken (e.g., "warn", "mute", "ban", "delete").
    pub action_type: String,
    /// Address of the member the action was applied to.
    pub target_user: Address,
    /// Reason given for the moderation action.
    pub reason: String,
    /// Duration of the action in seconds; 0 means permanent.
    pub duration: u64,
    /// Unix timestamp (seconds) when the action was applied.
    pub created_at: u64,
}

// ───────────────────────────────────────────────
//  Community Analytics
// ───────────────────────────────────────────────

/// Aggregate metrics for the community contract.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CommunityMetrics {
    /// Total number of forum posts ever created.
    pub total_posts: u32,
    /// Total number of forum replies ever created.
    pub total_replies: u32,
    /// Total number of knowledge contributions ever submitted.
    pub total_contributions: u32,
    /// Total number of community events ever created.
    pub total_events: u32,
    /// Number of mentorships currently in the Active status.
    pub active_mentorships: u32,
    /// Total number of registered community members.
    pub total_members: u32,
    /// Number of unique users active within the last 24 hours.
    pub daily_active_users: u32,
    /// Number of unique users active within the last 7 days.
    pub weekly_active_users: u32,
    /// Average time in seconds between a post being created and its first reply.
    pub avg_response_time: u64,
    /// Percentage of reported content that has been resolved, on a 0–100 scale.
    pub resolution_rate: u32,
    /// Unix timestamp (seconds) when these metrics were last updated.
    pub last_updated: u64,
}

/// Community activity statistics for an individual user.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UserCommunityStats {
    /// Address of the user these statistics belong to.
    pub user: Address,
    /// Number of forum posts the user has created.
    pub posts_created: u32,
    /// Number of forum replies the user has written.
    pub replies_given: u32,
    /// Number of times one of the user's replies was marked as the solution.
    pub solutions_provided: u32,
    /// Number of knowledge base contributions the user has made.
    pub contributions_made: u32,
    /// Number of community events the user has attended.
    pub events_attended: u32,
    /// Total number of mentorship sessions the user has participated in.
    pub mentorship_sessions: u32,
    /// Total upvotes the user has received across posts, replies, and contributions.
    pub helpful_votes_received: u32,
    /// Composite community reputation score.
    pub reputation_score: u32,
    /// Unix timestamp (seconds) when the user joined the community.
    pub joined_at: u64,
}

// ───────────────────────────────────────────────
//  Community Governance
// ───────────────────────────────────────────────

/// Category of a community governance proposal.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ProposalType {
    /// Proposal to add or change a platform feature.
    FeatureRequest,
    /// Proposal to change an existing community policy.
    PolicyChange,
    /// Proposal to introduce or amend a community rule.
    CommunityRule,
    /// Proposal to organise a community event.
    EventProposal,
    /// A proposal that does not fit the above categories.
    Other,
}

/// Lifecycle status of a governance proposal.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ProposalStatus {
    /// Proposal is being drafted and not yet open for voting.
    Draft,
    /// Proposal is open for community voting.
    Active,
    /// Proposal received enough votes and was accepted.
    Passed,
    /// Proposal did not receive enough votes or was voted down.
    Rejected,
    /// Accepted proposal has been put into effect.
    Implemented,
}

/// A community governance proposal that members can vote on.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CommunityProposal {
    /// Unique numeric identifier for the proposal.
    pub id: u64,
    /// Address of the member who submitted the proposal.
    pub proposer: Address,
    /// Category of the proposal.
    pub proposal_type: ProposalType,
    /// Short title of the proposal.
    pub title: String,
    /// Full description of the proposed change or initiative.
    pub description: String,
    /// Current lifecycle status.
    pub status: ProposalStatus,
    /// Number of votes cast in favour.
    pub votes_for: u32,
    /// Number of votes cast against.
    pub votes_against: u32,
    /// Unix timestamp (seconds) when the proposal was created.
    pub created_at: u64,
    /// Unix timestamp (seconds) when the voting period closes.
    pub voting_ends_at: u64,
    /// Minimum number of votes required for the result to be valid.
    pub min_votes_required: u32,
}

// ───────────────────────────────────────────────
//  Configuration
// ───────────────────────────────────────────────

/// Runtime configuration for XP rewards, moderation thresholds, and governance rules.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CommunityConfig {
    /// XP awarded to a user for creating a forum post.
    pub post_xp_reward: u32,
    /// XP awarded to a user for writing a forum reply.
    pub reply_xp_reward: u32,
    /// XP awarded to a user whose reply is marked as the accepted solution.
    pub solution_xp_reward: u32,
    /// Base XP awarded for a published knowledge contribution.
    pub contribution_base_xp: u32,
    /// Base token amount awarded for a published knowledge contribution.
    pub contribution_base_tokens: i128,
    /// XP awarded to a mentor for completing a mentorship session.
    pub mentor_session_xp: u32,
    /// XP awarded to a user for attending a community event.
    pub event_attendance_xp: u32,
    /// Minimum reputation score required before a user can be assigned a moderator role.
    pub min_reputation_to_moderate: u32,
    /// Maximum number of content reports a user may file per day.
    pub max_reports_per_day: u32,
    /// Minimum reputation score required for a user's governance vote to carry weight.
    pub vote_weight_threshold: u32,
}

// ───────────────────────────────────────────────
//  Storage Keys
// ───────────────────────────────────────────────

/// Storage key enum used to namespace all community contract state in the ledger.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum CommunityKey {
    /// Address of the contract administrator.
    Admin,
    /// Runtime configuration for the community contract.
    Config,

    // Counters
    /// Monotonically increasing counter used to assign forum post IDs.
    PostCounter,
    /// Monotonically increasing counter used to assign forum reply IDs.
    ReplyCounter,
    /// Monotonically increasing counter used to assign contribution IDs.
    ContributionCounter,
    /// Monotonically increasing counter used to assign event IDs.
    EventCounter,
    /// Monotonically increasing counter used to assign report IDs.
    ReportCounter,
    /// Monotonically increasing counter used to assign proposal IDs.
    ProposalCounter,
    /// Monotonically increasing counter used to assign mentorship IDs.
    MentorshipCounter,
    /// Monotonically increasing counter used to assign session IDs.
    SessionCounter,

    // Forum
    /// A specific forum post keyed by its ID.
    Post(u64),
    /// A specific forum reply keyed by its ID.
    Reply(u64),
    /// List of reply IDs belonging to a forum post.
    PostReplies(u64),
    /// List of post IDs belonging to a forum category.
    CategoryPosts(ForumCategory),
    /// List of post IDs created by a specific user.
    UserPosts(Address),
    /// Vote record for a specific user on a specific post.
    PostVote(Address, u64),
    /// Vote record for a specific user on a specific reply.
    ReplyVote(Address, u64),

    // Mentorship
    /// Mentor profile for a specific address.
    MentorProfile(Address),
    /// A specific mentorship request keyed by its ID.
    MentorshipRequest(u64),
    /// List of mentorship request IDs associated with a user.
    UserMentorships(Address),
    /// A specific mentorship session keyed by its ID.
    MentorshipSession(u64),

    // Knowledge Base
    /// A specific knowledge contribution keyed by its ID.
    Contribution(u64),
    /// List of contribution IDs submitted by a specific user.
    UserContributions(Address),
    /// List of contribution IDs belonging to a forum category.
    CategoryContributions(ForumCategory),

    // Events
    /// A specific community event keyed by its ID.
    Event(u64),
    /// List of participant addresses registered for an event.
    EventParticipants(u64),
    /// List of event IDs a specific user has registered for.
    UserEvents(Address),
    /// Registration record for a specific user at a specific event.
    EventParticipant(Address, u64),

    // Moderation
    /// Moderator role record for a specific address.
    Moderator(Address),
    /// A specific content report keyed by its ID.
    Report(u64),
    /// List of report IDs that are awaiting review.
    PendingReports,
    /// A specific moderator action record keyed by its ID.
    ModeratorAction(u64),
    /// List of moderator action IDs applied to a specific user.
    UserActions(Address),

    // Analytics
    /// Aggregate community metrics.
    CommunityMetrics,
    /// Community activity statistics for a specific user.
    UserStats(Address),

    // Governance
    /// A specific governance proposal keyed by its ID.
    Proposal(u64),
    /// List of proposal IDs currently open for voting.
    ActiveProposals,
    /// Vote record for a specific user on a specific proposal.
    ProposalVote(Address, u64),
}
