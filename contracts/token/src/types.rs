use soroban_sdk::{contracttype, Address, String, Vec};

/// A record of a token reward issued to a user for a specific platform activity.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct TokenReward {
    /// Category of activity that triggered this reward.
    pub reward_type: RewardType,
    /// Number of tokens awarded.
    pub amount: i128,
    /// Address of the user who received the reward.
    pub recipient: Address,
    /// Optional identifier of the course associated with the reward.
    pub course_id: Option<String>,
    /// Optional identifier of the achievement associated with the reward.
    pub achievement_id: Option<String>,
    /// Unix timestamp (seconds) when the reward was issued.
    pub timestamp: u64,
    /// Reward multiplier applied; 100 = 1.0×, 150 = 1.5×.
    pub multiplier: u32,
}

/// Category of user activity that qualifies for a token reward.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum RewardType {
    /// Reward for completing a full course.
    CourseCompletion,
    /// Reward for completing a single learning module.
    ModuleCompletion,
    /// Reward for earning a defined achievement.
    Achievement,
    /// Reward for maintaining a login or activity streak.
    Streak,
    /// Reward for successfully referring a new user.
    Referral,
    /// Reward for participating in a community event or activity.
    Participation,
    /// Reward for achieving a high score or exceptional performance.
    Excellence,
    /// Reward granted the first time a user completes a specific action.
    FirstTime,
}

/// Definition of a token-backed achievement that users can earn.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Achievement {
    /// Unique string identifier for the achievement.
    pub id: String,
    /// Display name of the achievement.
    pub name: String,
    /// Description of what the achievement represents and how to earn it.
    pub description: String,
    /// Number of tokens awarded when the achievement is earned.
    pub reward_amount: i128,
    /// Conditions that must be met to earn the achievement.
    pub requirements: AchievementRequirements,
    /// Rarity tier that determines the reward range.
    pub rarity: AchievementRarity,
    /// Unix timestamp (seconds) when the achievement was created.
    pub created_at: u64,
    /// Whether the achievement is currently earnable.
    pub is_active: bool,
}

/// Optional conditions a user must satisfy to earn a specific achievement.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AchievementRequirements {
    /// Minimum number of courses the user must have completed.
    pub courses_completed: Option<u32>,
    /// Minimum course completion percentage required.
    pub completion_percentage: Option<u32>,
    /// Maximum time in seconds the user may take to satisfy the conditions.
    pub time_limit: Option<u64>,
    /// Specific course identifiers the user must have completed.
    pub specific_courses: Option<Vec<String>>,
    /// Minimum consecutive activity streak in days.
    pub streak_days: Option<u32>,
    /// Minimum number of successful referrals required.
    pub referrals_count: Option<u32>,
    /// Free-form string expressing a custom eligibility condition.
    pub custom_criteria: Option<String>,
}

/// Rarity bracket for an achievement, determining its token reward range.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AchievementRarity {
    /// Common achievement; typical reward 100–500 tokens.
    Common,
    /// Uncommon achievement; typical reward 500–1 000 tokens.
    Uncommon,
    /// Rare achievement; typical reward 1 000–2 500 tokens.
    Rare,
    /// Epic achievement; typical reward 2 500–5 000 tokens.
    Epic,
    /// Legendary achievement; typical reward 5 000+ tokens.
    Legendary,
}

/// Record of a user earning a specific achievement.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UserAchievement {
    /// Address of the user who earned the achievement.
    pub user: Address,
    /// Identifier of the achievement that was earned.
    pub achievement_id: String,
    /// Unix timestamp (seconds) when the achievement was earned.
    pub earned_at: u64,
    /// Whether the user has already claimed the token reward for this achievement.
    pub reward_claimed: bool,
    /// Token amount that was (or will be) awarded for this achievement.
    pub reward_amount: i128,
}

/// A staking pool that users can deposit tokens into to earn rewards and unlock premium features.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct StakingPool {
    /// Unique string identifier for the pool.
    pub id: String,
    /// Display name of the staking pool.
    pub name: String,
    /// Description of the pool's purpose and terms.
    pub description: String,
    /// Minimum token amount required to stake in this pool.
    pub minimum_stake: i128,
    /// Annual reward rate expressed in basis points (100 = 1%).
    pub reward_rate: u32,
    /// Minimum lock-up duration in seconds before a stake can be withdrawn.
    pub lock_duration: u64,
    /// Total tokens currently staked across all participants.
    pub total_staked: i128,
    /// Cumulative tokens distributed as staking rewards since pool creation.
    pub total_rewards_distributed: i128,
    /// Whether the pool is currently accepting new stakes.
    pub is_active: bool,
    /// Unix timestamp (seconds) when the pool was created.
    pub created_at: u64,
    /// Premium features unlocked for users who stake in this pool.
    pub premium_features: Vec<PremiumFeature>,
}

/// A premium platform feature that can be unlocked through staking or other means.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum PremiumFeature {
    /// Access to detailed learning analytics dashboards.
    AdvancedAnalytics,
    /// Elevated customer support response priority.
    PrioritySupport,
    /// Access to courses not available to standard users.
    ExclusiveCourses,
    /// Ability to customise the appearance of issued certificates.
    CertificateCustomization,
    /// Direct access to expert mentors.
    MentorAccess,
    /// Early access to new platform features before general release.
    EarlyAccess,
    /// Discounted platform fees.
    ReducedFees,
}

/// Record of a user's active stake in a specific staking pool.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UserStake {
    /// Address of the staking user.
    pub user: Address,
    /// Identifier of the pool the tokens are staked in.
    pub pool_id: String,
    /// Amount of tokens staked.
    pub amount: i128,
    /// Unix timestamp (seconds) when the stake was created.
    pub staked_at: u64,
    /// Unix timestamp (seconds) when the stake lock-up expires and withdrawal is allowed.
    pub unlock_at: u64,
    /// Total rewards earned by this stake so far.
    pub rewards_earned: i128,
    /// Unix timestamp (seconds) of the user's most recent reward claim.
    pub last_reward_claim: u64,
}

/// Record of tokens burned by a user in exchange for a platform benefit.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BurnTransaction {
    /// Unique string identifier for this burn transaction.
    pub id: String,
    /// Address of the user who burned tokens.
    pub user: Address,
    /// Amount of tokens burned.
    pub amount: i128,
    /// Benefit category the tokens were burned for.
    pub burn_type: BurnType,
    /// Optional identifier of the certificate involved in the burn.
    pub certificate_id: Option<String>,
    /// Optional description of the upgrade obtained through the burn.
    pub upgrade_type: Option<String>,
    /// Unix timestamp (seconds) when the tokens were burned.
    pub timestamp: u64,
}

/// Category of benefit a user can obtain by burning tokens.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum BurnType {
    /// Upgrade an existing certificate to a higher-tier version.
    CertificateUpgrade,
    /// Unlock a premium platform feature.
    PremiumFeature,
    /// Apply a custom visual design to a certificate.
    CustomDesign,
    /// Fast-track access to a course or module.
    FastTrack,
    /// Skip a course prerequisite requirement.
    SkipPrerequisite,
    /// Purchase additional assessment attempts.
    ExtraAttempts,
}

/// A time-limited token reward multiplier applied to a user's future rewards.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RewardMultiplier {
    /// Address of the user this multiplier applies to.
    pub user: Address,
    /// Multiplier value; 100 = 1.0×, 150 = 1.5×.
    pub multiplier: u32,
    /// Reason the multiplier was granted.
    pub reason: MultiplierReason,
    /// Optional Unix timestamp (seconds) when the multiplier expires; `None` means permanent.
    pub expires_at: Option<u64>,
    /// Unix timestamp (seconds) when the multiplier was applied.
    pub applied_at: u64,
}

/// Reason a reward multiplier was granted to a user.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum MultiplierReason {
    /// Granted for maintaining a consecutive activity streak.
    Streak,
    /// Granted to VIP or premium tier users.
    VipStatus,
    /// Granted as part of a referral promotion.
    Referral,
    /// Granted during a special platform event.
    Event,
    /// Granted upon earning a specific achievement.
    Achievement,
    /// Granted for holding tokens in a staking pool.
    Staking,
}

/// Contract-wide tokenomics parameters governing reward amounts, multipliers, and supply.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct TokenomicsConfig {
    /// Base token amount rewarded for completing a course.
    pub base_course_reward: i128,
    /// Base token amount rewarded for completing a module.
    pub base_module_reward: i128,
    /// Streak bonus rate in basis points added per active day.
    pub streak_bonus_rate: u32,
    /// Maximum streak multiplier cap; 200 = 2.0×.
    pub max_streak_multiplier: u32,
    /// Token amount rewarded for each successful referral.
    pub referral_reward: i128,
    /// Bonus rate (basis points) applied on top of base rewards for achievement-related actions.
    pub achievement_bonus_rate: u32,
    /// Discount rate (basis points) applied to token burning costs.
    pub burn_discount_rate: u32,
    /// Annual token inflation rate in basis points.
    pub inflation_rate: u32,
    /// Maximum total supply of tokens that may ever exist.
    pub max_supply: i128,
    /// Address of the treasury account that receives platform fees and unallocated rewards.
    pub treasury_address: Address,
}

/// Token-related activity statistics for an individual user.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UserStats {
    /// Address of the user these statistics belong to.
    pub user: Address,
    /// Total tokens earned through all reward mechanisms.
    pub total_earned: i128,
    /// Total tokens spent on burns and premium features.
    pub total_spent: i128,
    /// Total tokens currently locked in staking pools.
    pub total_staked: i128,
    /// Length of the user's current consecutive activity streak in days.
    pub current_streak: u32,
    /// Longest streak the user has ever maintained.
    pub max_streak: u32,
    /// Total number of achievements the user has earned.
    pub achievements_count: u32,
    /// Total number of courses the user has completed.
    pub courses_completed: u32,
    /// Total number of successful referrals the user has made.
    pub referrals_made: u32,
    /// Unix timestamp (seconds) of the user's most recent activity.
    pub last_activity: u64,
}

/// A single user's entry in a token incentive leaderboard.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct LeaderboardEntry {
    /// Address of the ranked user.
    pub user: Address,
    /// Score used to rank the user in this category.
    pub score: i128,
    /// Ordinal position of the user in the leaderboard (1 = first place).
    pub rank: u32,
    /// Leaderboard category this entry belongs to.
    pub category: LeaderboardCategory,
}

/// Metric used to rank users on a token incentive leaderboard.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum LeaderboardCategory {
    /// Ranked by total tokens earned.
    TotalEarned,
    /// Ranked by number of courses completed.
    CoursesCompleted,
    /// Ranked by number of achievements earned.
    Achievements,
    /// Ranked by length of current activity streak.
    CurrentStreak,
    /// Ranked by number of successful referrals.
    Referrals,
}

/// A time-limited incentive event that boosts token rewards for participants.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct IncentiveEvent {
    /// Unique string identifier for the event.
    pub id: String,
    /// Display name of the event.
    pub name: String,
    /// Description of the event's goals and reward structure.
    pub description: String,
    /// Unix timestamp (seconds) when the event starts.
    pub start_time: u64,
    /// Unix timestamp (seconds) when the event ends.
    pub end_time: u64,
    /// Reward multiplier applied during the event; 100 = 1.0×.
    pub reward_multiplier: u32,
    /// Optional list of course identifiers eligible for the boosted rewards.
    pub eligible_courses: Option<Vec<String>>,
    /// Optional maximum number of participants; `None` means unlimited.
    pub max_participants: Option<u32>,
    /// Total token pool available as event rewards.
    pub total_reward_pool: i128,
    /// Whether the event is currently active.
    pub is_active: bool,
}

/// Storage key enum used to namespace all token incentive contract state in the ledger.
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

/// Platform-wide aggregate statistics for the token incentive system.
#[derive(Clone, Debug, Eq, PartialEq, Default)]
#[contracttype]
pub struct GlobalStats {
    /// Total tokens ever minted across the platform.
    pub total_tokens_minted: i128,
    /// Total tokens permanently removed from circulation through burns.
    pub total_tokens_burned: i128,
    /// Total tokens distributed as rewards to users.
    pub total_rewards_distributed: i128,
    /// Total tokens currently locked across all staking pools.
    pub total_staked: i128,
    /// Number of users who have interacted with the incentive system.
    pub active_users: u32,
    /// Total number of achievements earned across all users.
    pub total_achievements_earned: u32,
    /// Unix timestamp (seconds) when these statistics were last updated.
    pub last_updated: u64,
}

/// Intermediate breakdown of how a final token reward amount was calculated.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RewardCalculation {
    /// Starting reward amount before any multipliers or bonuses.
    pub base_amount: i128,
    /// Streak-based multiplier applied to the base amount; 100 = 1.0×.
    pub streak_multiplier: u32,
    /// Flat bonus added for achievement-related rewards (basis points).
    pub achievement_bonus: u32,
    /// Event-based multiplier applied during an active incentive event; 100 = 1.0×.
    pub event_multiplier: u32,
    /// Bonus added for active staking participation (basis points).
    pub staking_bonus: u32,
    /// Final token amount after all multipliers and bonuses have been applied.
    pub final_amount: i128,
}

/// Record of a user's access to a specific premium feature.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PremiumAccess {
    /// Address of the user who has access.
    pub user: Address,
    /// The premium feature the user can access.
    pub feature: PremiumFeature,
    /// Unix timestamp (seconds) when access was granted.
    pub granted_at: u64,
    /// Optional Unix timestamp (seconds) when access expires; `None` means permanent.
    pub expires_at: Option<u64>,
    /// Mechanism through which access was obtained.
    pub source: AccessSource,
}

/// Mechanism through which a user obtained access to a premium feature.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AccessSource {
    /// Access was earned by staking tokens.
    Staking,
    /// Access was purchased directly.
    Purchase,
    /// Access was granted as part of an achievement reward.
    Achievement,
    /// Access was granted during a promotional event.
    Event,
    /// Access was granted manually by an administrator.
    Admin,
}

/// Record of a referral relationship between two users.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ReferralData {
    /// Address of the user who made the referral.
    pub referrer: Address,
    /// Address of the user who was referred.
    pub referee: Address,
    /// Token amount to be awarded to the referrer.
    pub reward_amount: i128,
    /// Unix timestamp (seconds) when the referral was registered.
    pub created_at: u64,
    /// Whether the referral reward has been claimed by the referrer.
    pub reward_claimed: bool,
}

/// Streak activity record for a user, used to calculate streak bonuses.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct StreakData {
    /// Address of the user this streak record belongs to.
    pub user: Address,
    /// Number of consecutive active days in the current streak.
    pub current_streak: u32,
    /// Longest streak the user has ever achieved.
    pub max_streak: u32,
    /// Unix timestamp (seconds) of the user's last recorded activity date.
    pub last_activity_date: u64,
    /// Total tokens earned through streak bonuses.
    pub streak_rewards_earned: i128,
}
