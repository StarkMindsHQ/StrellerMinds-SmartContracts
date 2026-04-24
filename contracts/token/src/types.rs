use soroban_sdk::{contracttype, Address, String, Vec};

/// Token incentive system data types
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct TokenReward {
    pub reward_type: RewardType,
    pub amount: i128,
    pub recipient: Address,
    pub course_id: Option<String>,
    pub achievement_id: Option<String>,
    pub timestamp: u64,
    pub multiplier: u32, // 100 = 1.0x, 150 = 1.5x
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum RewardType {
    CourseCompletion,
    ModuleCompletion,
    Achievement,
    Streak,
    Referral,
    Participation,
    Excellence,
    FirstTime,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Achievement {
    pub id: String,
    pub name: String,
    pub description: String,
    pub reward_amount: i128,
    pub requirements: AchievementRequirements,
    pub rarity: AchievementRarity,
    pub created_at: u64,
    pub is_active: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AchievementRequirements {
    pub courses_completed: Option<u32>,
    pub completion_percentage: Option<u32>,
    pub time_limit: Option<u64>, // seconds
    pub specific_courses: Option<Vec<String>>,
    pub streak_days: Option<u32>,
    pub referrals_count: Option<u32>,
    pub custom_criteria: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AchievementRarity {
    Common,    // 100-500 tokens
    Uncommon,  // 500-1000 tokens
    Rare,      // 1000-2500 tokens
    Epic,      // 2500-5000 tokens
    Legendary, // 5000+ tokens
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UserAchievement {
    pub user: Address,
    pub achievement_id: String,
    pub earned_at: u64,
    pub reward_claimed: bool,
    pub reward_amount: i128,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct StakingPool {
    pub id: String,
    pub name: String,
    pub description: String,
    pub minimum_stake: i128,
    pub reward_rate: u32,   // basis points (100 = 1%)
    pub lock_duration: u64, // seconds
    pub total_staked: i128,
    pub total_rewards_distributed: i128,
    pub is_active: bool,
    pub created_at: u64,
    pub premium_features: Vec<PremiumFeature>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum PremiumFeature {
    AdvancedAnalytics,
    PrioritySupport,
    ExclusiveCourses,
    CertificateCustomization,
    MentorAccess,
    EarlyAccess,
    ReducedFees,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UserStake {
    pub user: Address,
    pub pool_id: String,
    pub amount: i128,
    pub staked_at: u64,
    pub unlock_at: u64,
    pub rewards_earned: i128,
    pub last_reward_claim: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BurnTransaction {
    pub id: String,
    pub user: Address,
    pub amount: i128,
    pub burn_type: BurnType,
    pub certificate_id: Option<String>,
    pub upgrade_type: Option<String>,
    pub timestamp: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum BurnType {
    CertificateUpgrade,
    PremiumFeature,
    CustomDesign,
    FastTrack,
    SkipPrerequisite,
    ExtraAttempts,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RewardMultiplier {
    pub user: Address,
    pub multiplier: u32, // 100 = 1.0x
    pub reason: MultiplierReason,
    pub expires_at: Option<u64>,
    pub applied_at: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum MultiplierReason {
    Streak,
    VipStatus,
    Referral,
    Event,
    Achievement,
    Staking,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct TokenomicsConfig {
    pub base_course_reward: i128,
    pub base_module_reward: i128,
    pub streak_bonus_rate: u32,     // basis points per day
    pub max_streak_multiplier: u32, // maximum multiplier (200 = 2.0x)
    pub referral_reward: i128,
    pub achievement_bonus_rate: u32,
    pub burn_discount_rate: u32, // discount for token burning
    pub inflation_rate: u32,     // annual inflation in basis points
    pub max_supply: i128,
    pub treasury_address: Address,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UserStats {
    pub user: Address,
    pub total_earned: i128,
    pub total_spent: i128,
    pub total_staked: i128,
    pub current_streak: u32,
    pub max_streak: u32,
    pub achievements_count: u32,
    pub courses_completed: u32,
    pub referrals_made: u32,
    pub last_activity: u64,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct LeaderboardEntry {
    pub user: Address,
    pub score: i128,
    pub rank: u32,
    pub category: LeaderboardCategory,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum LeaderboardCategory {
    TotalEarned,
    CoursesCompleted,
    Achievements,
    CurrentStreak,
    Referrals,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct IncentiveEvent {
    pub id: String,
    pub name: String,
    pub description: String,
    pub start_time: u64,
    pub end_time: u64,
    pub reward_multiplier: u32,
    pub eligible_courses: Option<Vec<String>>,
    pub max_participants: Option<u32>,
    pub total_reward_pool: i128,
    pub is_active: bool,
}

/// Storage keys for the incentive system
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum IncentiveDataKey {
    // Token rewards
    TokenReward(String),  // reward_id
    UserRewards(Address), // user -> Vec<TokenReward>

    // Achievements
    Achievement(String),              // achievement_id
    UserAchievement(Address, String), // user, achievement_id
    UserAchievements(Address),        // user -> Vec<UserAchievement>

    // Staking
    StakingPool(String),        // pool_id
    UserStake(Address, String), // user, pool_id
    UserStakes(Address),        // user -> Vec<UserStake>

    // Burning
    BurnTransaction(String), // transaction_id
    UserBurns(Address),      // user -> Vec<BurnTransaction>

    // Multipliers
    UserMultiplier(Address), // user -> RewardMultiplier

    // Configuration
    TokenomicsConfig,

    // Statistics
    UserStats(Address),
    GlobalStats,

    // Leaderboards
    Leaderboard(LeaderboardCategory),

    // Events
    IncentiveEvent(String), // event_id
    ActiveEvents,

    // Counters
    RewardCounter,
    AchievementCounter,
    BurnCounter,
    EventCounter,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct GlobalStats {
    pub total_tokens_minted: i128,
    pub total_tokens_burned: i128,
    pub total_rewards_distributed: i128,
    pub total_staked: i128,
    pub active_users: u32,
    pub total_achievements_earned: u32,
    pub last_updated: u64,
}

/// Reward calculation parameters
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RewardCalculation {
    pub base_amount: i128,
    pub streak_multiplier: u32,
    pub achievement_bonus: u32,
    pub event_multiplier: u32,
    pub staking_bonus: u32,
    pub final_amount: i128,
}

/// Premium feature access
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PremiumAccess {
    pub user: Address,
    pub feature: PremiumFeature,
    pub granted_at: u64,
    pub expires_at: Option<u64>,
    pub source: AccessSource,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AccessSource {
    Staking,
    Purchase,
    Achievement,
    Event,
    Admin,
}

/// Referral system
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ReferralData {
    pub referrer: Address,
    pub referee: Address,
    pub reward_amount: i128,
    pub created_at: u64,
    pub reward_claimed: bool,
}

/// Streak tracking
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct StreakData {
    pub user: Address,
    pub current_streak: u32,
    pub max_streak: u32,
    pub last_activity_date: u64,
    pub streak_rewards_earned: i128,
}
