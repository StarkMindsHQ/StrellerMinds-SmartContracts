use soroban_sdk::{contracttype, Address, String};

// ───────────────────────────────────────────────
//  Achievement System
// ───────────────────────────────────────────────

/// Rarity or prestige tier of a gamification achievement.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AchievementTier {
    /// Common achievement earnable by most students.
    Bronze,
    /// Intermediate difficulty achievement.
    Silver,
    /// Achievement awarded for challenging milestones.
    Gold,
    /// Achievement reserved for expert-level accomplishments.
    Platinum,
    /// Legendary achievement attainable only by the top 1% of learners.
    Diamond,
}

/// Domain or activity type that an achievement belongs to.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AchievementCategory {
    /// Awarded for course and module completions.
    Learning,
    /// Awarded for peer interactions and endorsements.
    Social,
    /// Awarded for consistency and habit formation via streaks.
    Streak,
    /// Awarded for completing challenges and quests.
    Challenge,
    /// Awarded for team-based guild accomplishments.
    Guild,
    /// Awarded for seasonal accomplishments.
    Season,
    /// Awarded for community contributions that build reputation.
    Reputation,
}

/// A single achievement definition (admin-created or milestone-seeded).
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Achievement {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub tier: AchievementTier,
    pub category: AchievementCategory,
    pub xp_reward: u32,
    pub token_reward: i128,
    pub requirements: AchievementRequirements,
    pub created_at: u64,
    pub is_active: bool,
    pub is_cross_course: bool, // award once across all courses
}

/// Thresholds a user must meet to earn a specific achievement.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AchievementRequirements {
    /// Minimum number of courses the user must have completed.
    pub courses_completed: u32,
    /// Minimum number of modules the user must have completed.
    pub modules_completed: u32,
    /// Minimum consecutive-day login streak required.
    pub streak_days: u32,
    /// Minimum total XP the user must have accumulated.
    pub total_xp: u32,
    /// Minimum number of challenges the user must have completed.
    pub challenges_completed: u32,
    /// Minimum number of peer endorsements the user must have received.
    pub endorsements_received: u32,
    /// Minimum number of contributions made within a guild.
    pub guild_contributions: u32,
    /// Minimum number of seasons the user must have completed.
    pub seasons_completed: u32,
}

/// Record of a user earning a specific achievement.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UserAchievement {
    pub user: Address,
    pub achievement_id: u64,
    pub earned_at: u64,
    pub token_reward_claimed: bool,
    pub xp_reward: u32,
    pub token_reward: i128,
}

// ───────────────────────────────────────────────
//  User Profile
// ───────────────────────────────────────────────

/// Gamification state for an individual user, tracking XP, streaks, guilds, and rewards.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct GamificationProfile {
    /// Address of the user this profile belongs to.
    pub user: Address,
    /// Total XP accumulated by the user across all activities.
    pub total_xp: u32,
    /// Current level derived from total XP.
    pub level: u32,
    /// Number of consecutive active days in the current streak.
    pub current_streak: u32,
    /// All-time longest streak the user has achieved.
    pub max_streak: u32,
    /// Unix timestamp (seconds) of the user's most recent activity.
    pub last_activity: u64,
    /// Total number of courses the user has completed.
    pub courses_completed: u32,
    /// Total number of modules the user has completed.
    pub modules_completed: u32,
    /// Total number of achievements the user has earned.
    pub achievements_count: u32,
    /// Total number of challenges the user has completed.
    pub challenges_completed: u32,
    /// Identifier of the guild the user belongs to; 0 means no guild.
    pub guild_id: u64,
    /// Community reputation score.
    pub reputation_score: u32,
    /// XP earned in the currently active season; reset at the start of each new season.
    pub season_xp: u32,
    /// Total peer endorsements received.
    pub endorsements_received: u32,
    /// Total peer endorsements given.
    pub endorsements_given: u32,
    /// Cumulative tokens earned through all gamification rewards.
    pub total_tokens_earned: i128,
    /// Unix timestamp (seconds) when the user first engaged with the gamification system.
    pub joined_at: u64,
}

// ───────────────────────────────────────────────
//  Activity
// ───────────────────────────────────────────────

/// Classification of a user activity that may trigger XP or achievement updates.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ActivityType {
    /// User completed a learning module.
    ModuleCompleted,
    /// User completed an entire course.
    CourseCompleted,
    /// User passed an assessment.
    AssessmentPassed,
    /// User logged a self-directed study session.
    StudySession,
    /// User helped a peer (e.g., answered a forum question).
    PeerHelped,
    /// User made progress towards completing a challenge.
    ChallengeProgress,
}

/// A record of a single user activity used for XP calculation and analytics.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ActivityRecord {
    /// Type of activity that was performed.
    pub activity_type: ActivityType,
    /// Identifier of the course involved; empty string if not applicable.
    pub course_id: String,
    /// Identifier of the module involved; empty string if not applicable.
    pub module_id: String,
    /// Score achieved (0–100); relevant for `AssessmentPassed` activities.
    pub score: u32,
    /// Duration of the activity in seconds.
    pub time_spent: u64,
    /// Unix timestamp (seconds) when the activity occurred.
    pub timestamp: u64,
}

// ───────────────────────────────────────────────
//  Leaderboard
// ───────────────────────────────────────────────

/// Metric used to rank users on a leaderboard.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum LeaderboardCategory {
    /// Ranked by cumulative XP across all time.
    TotalXP,
    /// Ranked by the length of the user's current activity streak.
    CurrentStreak,
    /// Ranked by total number of courses completed.
    CoursesCompleted,
    /// Ranked by community reputation score.
    Reputation,
    /// Ranked by XP earned in the current season.
    SeasonXP,
    /// Ranked by XP contributed to guild activities.
    GuildContributions,
    /// Ranked by total number of challenges completed.
    ChallengesCompleted,
    /// Ranked by total peer endorsements received.
    Endorsements,
}

/// A single user's entry in a gamification leaderboard.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct LeaderboardEntry {
    /// Address of the ranked user.
    pub user: Address,
    /// Score used to determine the user's rank in this category.
    pub score: u32,
    /// Ordinal position of the user in the leaderboard (1 = first place).
    pub rank: u32,
    /// Leaderboard category this entry belongs to.
    pub category: LeaderboardCategory,
}

/// A single guild's entry in the guild leaderboard.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct GuildLeaderboardEntry {
    /// Identifier of the ranked guild.
    pub guild_id: u64,
    /// Display name of the guild.
    pub guild_name: String,
    /// Aggregate XP of all guild members.
    pub total_xp: u32,
    /// Number of members in the guild.
    pub member_count: u32,
    /// Ordinal position of the guild in the leaderboard (1 = first place).
    pub rank: u32,
}

// ───────────────────────────────────────────────
//  Challenge / Quest System
// ───────────────────────────────────────────────

/// Participation model for a gamification challenge.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ChallengeType {
    /// A solo challenge completed independently.
    Individual,
    /// A guild-wide challenge where all members contribute toward a shared goal.
    Cooperative,
    /// A race-style challenge where the first participant to finish ranks highest.
    Competitive,
    /// A platform-wide participation challenge open to all users.
    Community,
}

/// Difficulty rating of a challenge.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ChallengeDifficulty {
    /// Suitable for new learners.
    Beginner,
    /// Requires moderate experience.
    Intermediate,
    /// Demands significant skill and effort.
    Advanced,
    /// Reserved for highly proficient users.
    Expert,
    /// Exceptionally rare and difficult; the hardest tier.
    Legendary,
}

/// A time-limited challenge or quest that users can join to earn rewards.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Challenge {
    /// Unique numeric identifier for the challenge.
    pub id: u64,
    /// Display name of the challenge.
    pub name: String,
    /// Description of the challenge objective and rules.
    pub description: String,
    /// Participation model for the challenge.
    pub challenge_type: ChallengeType,
    /// Difficulty rating of the challenge.
    pub difficulty: ChallengeDifficulty,
    /// XP awarded to users who complete the challenge.
    pub xp_reward: u32,
    /// Token amount awarded to users who complete the challenge.
    pub token_reward: i128,
    /// Unix timestamp (seconds) when the challenge opens.
    pub start_time: u64,
    /// Unix timestamp (seconds) when the challenge closes.
    pub end_time: u64,
    /// Units of work required to complete the challenge (e.g., complete N modules).
    pub target_progress: u32,
    /// Maximum number of participants allowed; 0 means unlimited.
    pub max_participants: u32,
    /// Number of users currently enrolled in the challenge.
    pub current_participants: u32,
    /// Whether the challenge is currently accepting new participants.
    pub is_active: bool,
    /// ID of a challenge that must be completed before joining this one; 0 means no prerequisite.
    pub prerequisite_challenge_id: u64,
    /// Address of the admin or system account that created the challenge.
    pub created_by: Address,
    /// Unix timestamp (seconds) when the challenge was created.
    pub created_at: u64,
}

/// A user's enrollment and progress record for a specific challenge.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UserChallenge {
    /// Address of the enrolled user.
    pub user: Address,
    /// Identifier of the challenge the user enrolled in.
    pub challenge_id: u64,
    /// Unix timestamp (seconds) when the user joined the challenge.
    pub joined_at: u64,
    /// Amount of progress units the user has accumulated toward the target.
    pub current_progress: u32,
    /// Whether the user has reached the target progress.
    pub completed: bool,
    /// Unix timestamp (seconds) when the user completed the challenge; 0 if not yet completed.
    pub completed_at: u64,
    /// Whether the user has claimed their reward for completing the challenge.
    pub reward_claimed: bool,
    /// User's rank in competitive challenges; 0 means unranked.
    pub rank: u32,
}

// ───────────────────────────────────────────────
//  Guild / Team System
// ───────────────────────────────────────────────

/// Role held by a member within a guild.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum GuildRole {
    /// Standard guild member with basic participation rights.
    Member,
    /// Elevated member with additional administrative privileges.
    Officer,
    /// The founding or primary leader of the guild.
    Leader,
}

/// A guild (team) that users can create and join to collaborate on challenges.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Guild {
    /// Unique numeric identifier for the guild.
    pub id: u64,
    /// Display name of the guild.
    pub name: String,
    /// Description of the guild's purpose and focus.
    pub description: String,
    /// Address of the guild leader.
    pub leader: Address,
    /// Aggregate XP contributed by all guild members.
    pub total_xp: u32,
    /// Current number of members in the guild.
    pub member_count: u32,
    /// Maximum number of members the guild can hold.
    pub max_members: u32,
    /// Whether the guild is open for anyone to join without an invitation.
    pub is_public: bool,
    /// Unix timestamp (seconds) when the guild was created.
    pub created_at: u64,
    /// Total number of competitive challenges the guild has won.
    pub challenge_wins: u32,
    /// XP accumulated by guild members in the current season.
    pub season_xp: u32,
}

/// Membership record linking a user to a guild and their role within it.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct GuildMember {
    /// Address of the guild member.
    pub user: Address,
    /// Identifier of the guild the user belongs to.
    pub guild_id: u64,
    /// Role the user holds within the guild.
    pub role: GuildRole,
    /// Unix timestamp (seconds) when the user joined the guild.
    pub joined_at: u64,
    /// XP this member has contributed to the guild.
    pub contribution_xp: u32,
    /// Number of guild challenges this member has participated in.
    pub challenges_participated: u32,
}

// ───────────────────────────────────────────────
//  Season System
// ───────────────────────────────────────────────

/// A time-boxed competitive season during which users earn season-specific XP and rewards.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Season {
    /// Unique numeric identifier for the season.
    pub id: u64,
    /// Display name of the season.
    pub name: String,
    /// Description of the season's theme and goals.
    pub description: String,
    /// Unix timestamp (seconds) when the season begins.
    pub start_time: u64,
    /// Unix timestamp (seconds) when the season ends.
    pub end_time: u64,
    /// XP multiplier applied during the season; 100 = 1.0×, 150 = 1.5×.
    pub xp_multiplier: u32,
    /// Whether this season is currently active.
    pub is_active: bool,
    /// Total number of users who participated in this season.
    pub total_participants: u32,
    /// Total token pool distributed as season-end rewards.
    pub reward_pool: i128,
}

/// End-of-season reward bracket based on a user's final ranking.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum SeasonRewardTier {
    /// User did not qualify for a reward.
    None,
    /// Top 50% of participants.
    Bronze,
    /// Top 25% of participants.
    Silver,
    /// Top 10% of participants.
    Gold,
    /// Top 1% of participants.
    Diamond,
}

/// A user's end-of-season standing including their rank and reward tier.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SeasonLeaderboardEntry {
    /// Address of the ranked user.
    pub user: Address,
    /// XP accumulated by the user during the season.
    pub season_xp: u32,
    /// Ordinal position of the user in the season leaderboard (1 = first place).
    pub rank: u32,
    /// Reward bracket the user qualified for based on their final rank.
    pub reward_tier: SeasonRewardTier,
}

// ───────────────────────────────────────────────
//  Reputation System
// ───────────────────────────────────────────────

/// Standing tier assigned to a user based on their reputation score.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ReputationTier {
    /// Starting tier for new community members.
    Novice,
    /// Early-progress tier for developing learners.
    Apprentice,
    /// Mid-level tier for consistently contributing members.
    Practitioner,
    /// Advanced tier recognising deep knowledge and contributions.
    Expert,
    /// High-prestige tier for outstanding community contributors.
    Master,
    /// The highest achievable reputation tier.
    Grandmaster,
}

/// Detailed reputation score breakdown for an individual user.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ReputationScore {
    /// Address of the user this score belongs to.
    pub user: Address,
    /// Sum of all reputation sub-scores.
    pub total_score: u32,
    /// Points earned by helping peers (e.g., answering questions, mentoring).
    pub teaching_points: u32,
    /// Points earned through high-quality course completions and high assessment scores.
    pub quality_points: u32,
    /// Points earned through regular, consistent platform activity.
    pub consistency_points: u32,
    /// Points earned through guild and team contributions.
    pub collaboration_points: u32,
    /// Points earned through challenge and quest completions.
    pub innovation_points: u32,
    /// Current standing tier based on the total score.
    pub tier: ReputationTier,
    /// Unix timestamp (seconds) when this record was last recalculated.
    pub last_updated: u64,
}

// ───────────────────────────────────────────────
//  Social Features
// ───────────────────────────────────────────────

/// A skill endorsement given by one user to another.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PeerEndorsement {
    /// Address of the user giving the endorsement.
    pub endorser: Address,
    /// Address of the user receiving the endorsement.
    pub endorsee: Address,
    /// Name of the skill being endorsed.
    pub skill: String,
    /// Unix timestamp (seconds) when the endorsement was given.
    pub created_at: u64,
    /// XP awarded to the endorsee for receiving this endorsement.
    pub xp_value: u32,
}

/// Category of a peer recognition award.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum RecognitionType {
    /// Recognises a user for providing a helpful answer.
    HelpfulAnswer,
    /// Recognises a user for making exceptional learning progress.
    GreatProgress,
    /// Recognises a user whose work or attitude inspired others.
    Inspiration,
    /// Recognises a user for outstanding collaborative behaviour.
    Collaboration,
    /// Recognises a user for creative or novel contributions.
    Innovation,
}

/// A recognition message sent from one user to another.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PeerRecognition {
    /// Address of the user giving the recognition.
    pub from: Address,
    /// Address of the user being recognised.
    pub to: Address,
    /// Personalised message accompanying the recognition.
    pub message: String,
    /// Category of the recognition.
    pub recognition_type: RecognitionType,
    /// Unix timestamp (seconds) when the recognition was created.
    pub created_at: u64,
}

// ───────────────────────────────────────────────
//  Adaptive Difficulty
// ───────────────────────────────────────────────

/// Personalised difficulty recommendation derived from a user's historical performance.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AdaptiveDifficulty {
    /// Address of the user this recommendation belongs to.
    pub user: Address,
    /// Suggested challenge difficulty level based on the user's recent performance.
    pub recommended_difficulty: ChallengeDifficulty,
    /// Composite performance metric on a 0–100 scale.
    pub performance_score: u32,
    /// Challenge completion rate on a 0–100 scale.
    pub completion_rate: u32,
    /// Average assessment score on a 0–100 scale.
    pub avg_score: u32,
    /// Unix timestamp (seconds) when this recommendation was last recalculated.
    pub last_calculated: u64,
}

// ───────────────────────────────────────────────
//  Gamification Config
// ───────────────────────────────────────────────

/// Runtime configuration for XP values, endorsement limits, guild caps, and leaderboard sizes.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct GamificationConfig {
    /// Base XP awarded for completing a single module.
    pub base_module_xp: u32,
    /// Base XP awarded for completing an entire course.
    pub base_course_xp: u32,
    /// Bonus XP awarded for each completed streak week.
    pub streak_weekly_bonus: u32,
    /// Maximum additional XP earnable through streak bonuses (basis points over base).
    pub max_streak_bonus_xp: u32,
    /// XP awarded to the endorsee when they receive a peer endorsement.
    pub endorsement_xp: u32,
    /// XP awarded to a user for helping a peer.
    pub help_xp: u32,
    /// Maximum number of endorsements a user may give per day.
    pub max_endorsements_per_day: u32,
    /// Maximum number of members allowed in a single guild.
    pub guild_max_members: u32,
    /// Maximum number of entries retained in each leaderboard.
    pub leaderboard_size: u32,
    // Rate limits (max calls per rate_limit_window)
    pub rate_limit_activity: u32,
    pub rate_limit_recognition: u32,
    pub rate_limit_window: u64,
}

// ───────────────────────────────────────────────
//  Storage Keys
// ───────────────────────────────────────────────

/// Storage key enum used to namespace all gamification contract state in the ledger.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum GamificationKey {
    // ── Admin / Config ──────────────────────────
    Admin,
    Config,

    // ── Counters ────────────────────────────────
    AchievementCounter,
    ChallengeCounter,
    GuildCounter,
    SeasonCounter,
    EndorsementCounter,
    RecognitionCounter,

    // ── Achievements ────────────────────────────
    Achievement(u64),
    UserAchievement(Address, u64),
    UserAchievements(Address), // Vec<u64>

    // ── User Profile ────────────────────────────
    UserProfile(Address),

    // ── Leaderboards ────────────────────────────
    Leaderboard(LeaderboardCategory), // Vec<LeaderboardEntry>
    GuildLeaderboard,                 // Vec<GuildLeaderboardEntry>

    // ── Challenges ──────────────────────────────
    Challenge(u64),
    ActiveChallenges, // Vec<u64>
    UserChallenge(Address, u64),
    UserActiveChallenges(Address), // Vec<u64>
    ChallengeCompletionCount(u64), // u32 – how many finished this challenge

    // ── Guilds ──────────────────────────────────
    Guild(u64),
    GuildMember(Address), // Address → GuildMember
    GuildMembers(u64),    // guild_id → Vec<Address>

    // ── Seasons ─────────────────────────────────
    Season(u64),
    /// 0 = no active season
    ActiveSeasonId,
    SeasonLeaderboard(u64),     // season_id → Vec<SeasonLeaderboardEntry>
    UserSeasonXP(Address, u64), // (user, season_id) → u32

    // ── Reputation ──────────────────────────────
    UserReputation(Address),

    // ── Social ──────────────────────────────────
    UserEndorsements(Address), // endorsee → Vec<PeerEndorsement>
    /// endorser → day-bucket → count (for rate limiting)
    EndorserDailyCount(Address, u64),

    // ── Adaptive Difficulty ─────────────────────
    UserDifficulty(Address),

    // ── Rate Limiting ──────────────────────────
    RateLimit(Address, u64), // (user, operation_id) -> RateLimitState
}
