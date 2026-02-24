use soroban_sdk::{contracttype, Address, String};

// ───────────────────────────────────────────────
//  Achievement System
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AchievementTier {
    Bronze,   // Common – earnable by most students
    Silver,   // Intermediate difficulty
    Gold,     // Challenging milestones
    Platinum, // Expert-level accomplishments
    Diamond,  // Legendary, top 1% territory
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AchievementCategory {
    Learning,   // Course / module completions
    Social,     // Peer interactions & endorsements
    Streak,     // Consistency & habit formation
    Challenge,  // Challenge / quest completions
    Guild,      // Team achievements
    Season,     // Seasonal accomplishments
    Reputation, // Community contributions
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

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AchievementRequirements {
    pub courses_completed: u32,
    pub modules_completed: u32,
    pub streak_days: u32,
    pub total_xp: u32,
    pub challenges_completed: u32,
    pub endorsements_received: u32,
    pub guild_contributions: u32,
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

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct GamificationProfile {
    pub user: Address,
    pub total_xp: u32,
    pub level: u32,
    pub current_streak: u32,
    pub max_streak: u32,
    pub last_activity: u64,
    pub courses_completed: u32,
    pub modules_completed: u32,
    pub achievements_count: u32,
    pub challenges_completed: u32,
    /// 0 = no guild
    pub guild_id: u64,
    pub reputation_score: u32,
    /// XP earned in the currently active season (reset each season)
    pub season_xp: u32,
    pub endorsements_received: u32,
    pub endorsements_given: u32,
    pub total_tokens_earned: i128,
    pub joined_at: u64,
}

// ───────────────────────────────────────────────
//  Activity
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ActivityType {
    ModuleCompleted,
    CourseCompleted,
    AssessmentPassed,
    StudySession,
    PeerHelped,
    ChallengeProgress,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ActivityRecord {
    pub activity_type: ActivityType,
    pub course_id: String, // empty string = N/A
    pub module_id: String, // empty string = N/A
    /// 0-100, relevant for AssessmentPassed
    pub score: u32,
    /// time spent in seconds
    pub time_spent: u64,
    pub timestamp: u64,
}

// ───────────────────────────────────────────────
//  Leaderboard
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum LeaderboardCategory {
    TotalXP,
    CurrentStreak,
    CoursesCompleted,
    Reputation,
    SeasonXP,
    GuildContributions,
    ChallengesCompleted,
    Endorsements,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct LeaderboardEntry {
    pub user: Address,
    pub score: u32,
    pub rank: u32,
    pub category: LeaderboardCategory,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct GuildLeaderboardEntry {
    pub guild_id: u64,
    pub guild_name: String,
    pub total_xp: u32,
    pub member_count: u32,
    pub rank: u32,
}

// ───────────────────────────────────────────────
//  Challenge / Quest System
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ChallengeType {
    Individual,  // solo challenge
    Cooperative, // guild-wide joint challenge
    Competitive, // race; first to finish ranks highest
    Community,   // platform-wide participation
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ChallengeDifficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
    Legendary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Challenge {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub challenge_type: ChallengeType,
    pub difficulty: ChallengeDifficulty,
    pub xp_reward: u32,
    pub token_reward: i128,
    pub start_time: u64,
    pub end_time: u64,
    /// units of work to complete (e.g. complete N modules)
    pub target_progress: u32,
    /// 0 = unlimited
    pub max_participants: u32,
    pub current_participants: u32,
    pub is_active: bool,
    /// 0 = no prerequisite (quest chain support)
    pub prerequisite_challenge_id: u64,
    pub created_by: Address,
    pub created_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UserChallenge {
    pub user: Address,
    pub challenge_id: u64,
    pub joined_at: u64,
    pub current_progress: u32,
    pub completed: bool,
    /// 0 = not yet completed
    pub completed_at: u64,
    pub reward_claimed: bool,
    /// rank for competitive challenges; 0 = unranked
    pub rank: u32,
}

// ───────────────────────────────────────────────
//  Guild / Team System
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum GuildRole {
    Member,
    Officer,
    Leader,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Guild {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub leader: Address,
    pub total_xp: u32,
    pub member_count: u32,
    pub max_members: u32,
    pub is_public: bool,
    pub created_at: u64,
    pub challenge_wins: u32,
    pub season_xp: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct GuildMember {
    pub user: Address,
    pub guild_id: u64,
    pub role: GuildRole,
    pub joined_at: u64,
    pub contribution_xp: u32,
    pub challenges_participated: u32,
}

// ───────────────────────────────────────────────
//  Season System
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Season {
    pub id: u64,
    pub name: String,
    pub description: String,
    pub start_time: u64,
    pub end_time: u64,
    /// 100 = 1.0×, 150 = 1.5×
    pub xp_multiplier: u32,
    pub is_active: bool,
    pub total_participants: u32,
    pub reward_pool: i128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum SeasonRewardTier {
    None,
    Bronze,  // top 50 %
    Silver,  // top 25 %
    Gold,    // top 10 %
    Diamond, // top  1 %
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SeasonLeaderboardEntry {
    pub user: Address,
    pub season_xp: u32,
    pub rank: u32,
    pub reward_tier: SeasonRewardTier,
}

// ───────────────────────────────────────────────
//  Reputation System
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ReputationTier {
    Novice,
    Apprentice,
    Practitioner,
    Expert,
    Master,
    Grandmaster,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ReputationScore {
    pub user: Address,
    pub total_score: u32,
    pub teaching_points: u32,      // helping peers
    pub quality_points: u32,       // high-quality completions / high scores
    pub consistency_points: u32,   // regular activity
    pub collaboration_points: u32, // guild / team contributions
    pub innovation_points: u32,    // challenges & quests
    pub tier: ReputationTier,
    pub last_updated: u64,
}

// ───────────────────────────────────────────────
//  Social Features
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PeerEndorsement {
    pub endorser: Address,
    pub endorsee: Address,
    pub skill: String,
    pub created_at: u64,
    pub xp_value: u32,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum RecognitionType {
    HelpfulAnswer,
    GreatProgress,
    Inspiration,
    Collaboration,
    Innovation,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PeerRecognition {
    pub from: Address,
    pub to: Address,
    pub message: String,
    pub recognition_type: RecognitionType,
    pub created_at: u64,
}

// ───────────────────────────────────────────────
//  Adaptive Difficulty
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AdaptiveDifficulty {
    pub user: Address,
    pub recommended_difficulty: ChallengeDifficulty,
    /// 0-100 composite performance metric
    pub performance_score: u32,
    /// challenge completion rate 0-100
    pub completion_rate: u32,
    /// average assessment score 0-100
    pub avg_score: u32,
    pub last_calculated: u64,
}

// ───────────────────────────────────────────────
//  Gamification Config
// ───────────────────────────────────────────────

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct GamificationConfig {
    pub base_module_xp: u32,
    pub base_course_xp: u32,
    /// bonus XP per completed streak week
    pub streak_weekly_bonus: u32,
    /// maximum additional XP from streak (basis points over base)
    pub max_streak_bonus_xp: u32,
    /// XP awarded to the endorsee
    pub endorsement_xp: u32,
    /// XP awarded for helping a peer
    pub help_xp: u32,
    pub max_endorsements_per_day: u32,
    pub guild_max_members: u32,
    /// maximum entries kept in each leaderboard
    pub leaderboard_size: u32,
}

// ───────────────────────────────────────────────
//  Storage Keys
// ───────────────────────────────────────────────

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
}
