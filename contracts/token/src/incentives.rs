use soroban_sdk::{Address, Env, String, Vec, Map};
use crate::types::{
    TokenReward, RewardType, Achievement, AchievementRequirements, AchievementRarity,
    UserAchievement, StakingPool, UserStake, BurnTransaction, BurnType, RewardMultiplier,
    MultiplierReason, TokenomicsConfig, UserStats, LeaderboardEntry, LeaderboardCategory,
    IncentiveEvent, IncentiveDataKey, GlobalStats, RewardCalculation, PremiumAccess,
    PremiumFeature, AccessSource, ReferralData, StreakData
};
use crate::Error;
use shared::access_control::AccessControl;
use shared::roles::Permission;

/// Token incentive management system
pub struct IncentiveManager;

impl IncentiveManager {
    /// Initialize the incentive system with default configuration
    pub fn initialize(env: &Env, admin: &Address) -> Result<(), Error> {
        // Validate admin permissions
        AccessControl::require_permission(env, admin, &Permission::UpdateCertificateMetadata)
            .map_err(|_| Error::NotInitialized)?;

        let config = TokenomicsConfig {
            base_course_reward: 100_000, // 100 tokens
            base_module_reward: 10_000,  // 10 tokens
            streak_bonus_rate: 500,      // 5% per day
            max_streak_multiplier: 300,  // 3x max
            referral_reward: 50_000,     // 50 tokens
            achievement_bonus_rate: 1000, // 10%
            burn_discount_rate: 2000,    // 20% discount
            inflation_rate: 500,         // 5% annual
            max_supply: 1_000_000_000_000, // 1B tokens
            treasury_address: admin.clone(),
        };

        env.storage().persistent().set(&IncentiveDataKey::TokenomicsConfig, &config);

        // Initialize global stats
        let global_stats = GlobalStats {
            total_tokens_minted: 0,
            total_tokens_burned: 0,
            total_rewards_distributed: 0,
            total_staked: 0,
            active_users: 0,
            total_achievements_earned: 0,
            last_updated: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&IncentiveDataKey::GlobalStats, &global_stats);

        // Initialize counters
        env.storage().persistent().set(&IncentiveDataKey::RewardCounter, &0u64);
        env.storage().persistent().set(&IncentiveDataKey::AchievementCounter, &0u64);
        env.storage().persistent().set(&IncentiveDataKey::BurnCounter, &0u64);
        env.storage().persistent().set(&IncentiveDataKey::EventCounter, &0u64);

        Ok(())
    }

    /// Reward user for course completion
    pub fn reward_course_completion(
        env: &Env,
        user: &Address,
        course_id: &String,
        completion_percentage: u32,
    ) -> Result<i128, Error> {
        let config = Self::get_config(env)?;
        let mut reward_amount = config.base_course_reward;

        // Apply completion percentage bonus
        if completion_percentage >= 90 {
            reward_amount = reward_amount * 150 / 100; // 1.5x for 90%+
        } else if completion_percentage >= 80 {
            reward_amount = reward_amount * 125 / 100; // 1.25x for 80%+
        }

        // Apply streak multiplier
        let streak_multiplier = Self::get_streak_multiplier(env, user);
        reward_amount = reward_amount * streak_multiplier as i128 / 100;

        // Apply event multipliers
        let event_multiplier = Self::get_active_event_multiplier(env, Some(course_id));
        reward_amount = reward_amount * event_multiplier as i128 / 100;

        // Create reward record
        let reward_id = Self::generate_reward_id(env);
        let reward = TokenReward {
            reward_type: RewardType::CourseCompletion,
            amount: reward_amount,
            recipient: user.clone(),
            course_id: Some(course_id.clone()),
            achievement_id: None,
            timestamp: env.ledger().timestamp(),
            multiplier: streak_multiplier * event_multiplier / 100,
        };

        Self::process_reward(env, &reward_id, &reward)?;
        Self::update_user_streak(env, user)?;
        Self::check_achievements(env, user)?;

        Ok(reward_amount)
    }

    /// Reward user for module completion
    pub fn reward_module_completion(
        env: &Env,
        user: &Address,
        course_id: &String,
        module_id: &String,
    ) -> Result<i128, Error> {
        let config = Self::get_config(env)?;
        let mut reward_amount = config.base_module_reward;

        // Apply multipliers
        let streak_multiplier = Self::get_streak_multiplier(env, user);
        reward_amount = reward_amount * streak_multiplier as i128 / 100;

        let reward_id = Self::generate_reward_id(env);
        let reward = TokenReward {
            reward_type: RewardType::ModuleCompletion,
            amount: reward_amount,
            recipient: user.clone(),
            course_id: Some(course_id.clone()),
            achievement_id: None,
            timestamp: env.ledger().timestamp(),
            multiplier: streak_multiplier,
        };

        Self::process_reward(env, &reward_id, &reward)?;
        Ok(reward_amount)
    }

    /// Create new achievement
    pub fn create_achievement(
        env: &Env,
        admin: &Address,
        achievement: Achievement,
    ) -> Result<String, Error> {
        AccessControl::require_permission(env, admin, &Permission::UpdateCertificateMetadata)
            .map_err(|_| Error::NotInitialized)?;

        let achievement_id = Self::generate_achievement_id(env);
        let mut new_achievement = achievement;
        new_achievement.id = achievement_id.clone();
        new_achievement.created_at = env.ledger().timestamp();

        env.storage().persistent().set(
            &IncentiveDataKey::Achievement(achievement_id.clone()),
            &new_achievement,
        );

        Ok(achievement_id)
    }

    /// Check and award achievements for user
    pub fn check_achievements(env: &Env, user: &Address) -> Result<Vec<String>, Error> {
        let user_stats = Self::get_user_stats(env, user);
        let mut awarded_achievements = Vec::new(env);

        // Get all achievements (simplified - in production would paginate)
        // Check course completion achievements
        if user_stats.courses_completed >= 1 {
            Self::try_award_achievement(env, user, "first_course", &mut awarded_achievements)?;
        }
        if user_stats.courses_completed >= 5 {
            Self::try_award_achievement(env, user, "course_explorer", &mut awarded_achievements)?;
        }
        if user_stats.courses_completed >= 10 {
            Self::try_award_achievement(env, user, "dedicated_learner", &mut awarded_achievements)?;
        }

        // Check streak achievements
        if user_stats.current_streak >= 7 {
            Self::try_award_achievement(env, user, "week_warrior", &mut awarded_achievements)?;
        }
        if user_stats.current_streak >= 30 {
            Self::try_award_achievement(env, user, "month_master", &mut awarded_achievements)?;
        }

        Ok(awarded_achievements)
    }

    /// Create staking pool
    pub fn create_staking_pool(
        env: &Env,
        admin: &Address,
        pool: StakingPool,
    ) -> Result<String, Error> {
        AccessControl::require_permission(env, admin, &Permission::UpdateCertificateMetadata)
            .map_err(|_| Error::NotInitialized)?;

        let pool_id = format!("pool_{}", env.ledger().timestamp());
        let mut new_pool = pool;
        new_pool.id = pool_id.clone();
        new_pool.created_at = env.ledger().timestamp();

        env.storage().persistent().set(
            &IncentiveDataKey::StakingPool(pool_id.clone()),
            &new_pool,
        );

        Ok(pool_id)
    }

    /// Stake tokens in pool
    pub fn stake_tokens(
        env: &Env,
        user: &Address,
        pool_id: &String,
        amount: i128,
    ) -> Result<(), Error> {
        user.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let pool = Self::get_staking_pool(env, pool_id)?;
        if !pool.is_active || amount < pool.minimum_stake {
            return Err(Error::InvalidAmount);
        }

        // Check user balance (would integrate with token contract)
        let user_balance = Self::get_token_balance(env, user);
        if user_balance < amount {
            return Err(Error::InsufficientBalance);
        }

        let stake = UserStake {
            user: user.clone(),
            pool_id: pool_id.clone(),
            amount,
            staked_at: env.ledger().timestamp(),
            unlock_at: env.ledger().timestamp() + pool.lock_duration,
            rewards_earned: 0,
            last_reward_claim: env.ledger().timestamp(),
        };

        env.storage().persistent().set(
            &IncentiveDataKey::UserStake(user.clone(), pool_id.clone()),
            &stake,
        );

        // Update pool total
        let mut updated_pool = pool;
        updated_pool.total_staked += amount;
        env.storage().persistent().set(
            &IncentiveDataKey::StakingPool(pool_id.clone()),
            &updated_pool,
        );

        // Grant premium access
        Self::grant_premium_access(env, user, &updated_pool.premium_features)?;

        Ok(())
    }

    /// Burn tokens for certificate upgrade
    pub fn burn_for_upgrade(
        env: &Env,
        user: &Address,
        amount: i128,
        certificate_id: &String,
        upgrade_type: &String,
    ) -> Result<String, Error> {
        user.require_auth();

        if amount <= 0 {
            return Err(Error::InvalidAmount);
        }

        let user_balance = Self::get_token_balance(env, user);
        if user_balance < amount {
            return Err(Error::InsufficientBalance);
        }

        let burn_id = Self::generate_burn_id(env);
        let burn_tx = BurnTransaction {
            id: burn_id.clone(),
            user: user.clone(),
            amount,
            burn_type: BurnType::CertificateUpgrade,
            certificate_id: Some(certificate_id.clone()),
            upgrade_type: Some(upgrade_type.clone()),
            timestamp: env.ledger().timestamp(),
        };

        env.storage().persistent().set(
            &IncentiveDataKey::BurnTransaction(burn_id.clone()),
            &burn_tx,
        );

        // Update global stats
        Self::update_burn_stats(env, amount)?;

        Ok(burn_id)
    }

    // Helper methods
    fn get_config(env: &Env) -> Result<TokenomicsConfig, Error> {
        env.storage()
            .persistent()
            .get(&IncentiveDataKey::TokenomicsConfig)
            .ok_or(Error::NotInitialized)
    }

    fn get_streak_multiplier(env: &Env, user: &Address) -> u32 {
        let streak_data: Option<StreakData> = env.storage()
            .persistent()
            .get(&IncentiveDataKey::UserStats(user.clone()));
        
        if let Some(data) = streak_data {
            let config = Self::get_config(env).unwrap_or_default();
            let bonus = data.current_streak * config.streak_bonus_rate / 10000;
            (100 + bonus).min(config.max_streak_multiplier)
        } else {
            100 // 1.0x
        }
    }

    fn get_active_event_multiplier(env: &Env, course_id: Option<&String>) -> u32 {
        // Simplified - would check active events
        100 // 1.0x default
    }

    fn process_reward(env: &Env, reward_id: &String, reward: &TokenReward) -> Result<(), Error> {
        env.storage().persistent().set(
            &IncentiveDataKey::TokenReward(reward_id.clone()),
            reward,
        );

        // Add to user rewards
        let mut user_rewards: Vec<TokenReward> = env.storage()
            .persistent()
            .get(&IncentiveDataKey::UserRewards(reward.recipient.clone()))
            .unwrap_or_else(|| Vec::new(env));
        
        user_rewards.push_back(reward.clone());
        env.storage().persistent().set(
            &IncentiveDataKey::UserRewards(reward.recipient.clone()),
            &user_rewards,
        );

        // Update user stats
        Self::update_user_stats(env, &reward.recipient, reward.amount)?;

        Ok(())
    }

    fn update_user_streak(env: &Env, user: &Address) -> Result<(), Error> {
        let current_time = env.ledger().timestamp();
        let one_day = 86400u64;

        let mut streak_data: StreakData = env.storage()
            .persistent()
            .get(&IncentiveDataKey::UserStats(user.clone()))
            .unwrap_or_else(|| StreakData {
                user: user.clone(),
                current_streak: 0,
                max_streak: 0,
                last_activity_date: 0,
                streak_rewards_earned: 0,
            });

        // Check if activity is within streak window
        if current_time - streak_data.last_activity_date <= one_day * 2 {
            streak_data.current_streak += 1;
        } else {
            streak_data.current_streak = 1;
        }

        streak_data.max_streak = streak_data.max_streak.max(streak_data.current_streak);
        streak_data.last_activity_date = current_time;

        env.storage().persistent().set(
            &IncentiveDataKey::UserStats(user.clone()),
            &streak_data,
        );

        Ok(())
    }

    fn generate_reward_id(env: &Env) -> String {
        let counter: u64 = env.storage()
            .persistent()
            .get(&IncentiveDataKey::RewardCounter)
            .unwrap_or(0);
        
        let new_counter = counter + 1;
        env.storage().persistent().set(&IncentiveDataKey::RewardCounter, &new_counter);
        
        format!("reward_{}", new_counter)
    }

    fn generate_achievement_id(env: &Env) -> String {
        let counter: u64 = env.storage()
            .persistent()
            .get(&IncentiveDataKey::AchievementCounter)
            .unwrap_or(0);
        
        let new_counter = counter + 1;
        env.storage().persistent().set(&IncentiveDataKey::AchievementCounter, &new_counter);
        
        format!("achievement_{}", new_counter)
    }

    fn generate_burn_id(env: &Env) -> String {
        let counter: u64 = env.storage()
            .persistent()
            .get(&IncentiveDataKey::BurnCounter)
            .unwrap_or(0);
        
        let new_counter = counter + 1;
        env.storage().persistent().set(&IncentiveDataKey::BurnCounter, &new_counter);
        
        format!("burn_{}", new_counter)
    }

    fn get_user_stats(env: &Env, user: &Address) -> UserStats {
        env.storage()
            .persistent()
            .get(&IncentiveDataKey::UserStats(user.clone()))
            .unwrap_or_else(|| UserStats {
                user: user.clone(),
                total_earned: 0,
                total_spent: 0,
                total_staked: 0,
                current_streak: 0,
                max_streak: 0,
                achievements_count: 0,
                courses_completed: 0,
                referrals_made: 0,
                last_activity: 0,
            })
    }

    fn update_user_stats(env: &Env, user: &Address, reward_amount: i128) -> Result<(), Error> {
        let mut stats = Self::get_user_stats(env, user);
        stats.total_earned += reward_amount;
        stats.last_activity = env.ledger().timestamp();

        env.storage().persistent().set(
            &IncentiveDataKey::UserStats(user.clone()),
            &stats,
        );

        Ok(())
    }

    fn try_award_achievement(
        env: &Env,
        user: &Address,
        achievement_id: &str,
        awarded: &mut Vec<String>,
    ) -> Result<(), Error> {
        // Check if user already has this achievement
        let existing = env.storage()
            .persistent()
            .get(&IncentiveDataKey::UserAchievement(
                user.clone(),
                String::from_str(env, achievement_id),
            ));

        if existing.is_none() {
            // Award achievement
            let user_achievement = UserAchievement {
                user: user.clone(),
                achievement_id: String::from_str(env, achievement_id),
                earned_at: env.ledger().timestamp(),
                reward_claimed: false,
                reward_amount: 1000, // Default achievement reward
            };

            env.storage().persistent().set(
                &IncentiveDataKey::UserAchievement(
                    user.clone(),
                    String::from_str(env, achievement_id),
                ),
                &user_achievement,
            );

            awarded.push_back(String::from_str(env, achievement_id));
        }

        Ok(())
    }

    fn get_staking_pool(env: &Env, pool_id: &String) -> Result<StakingPool, Error> {
        env.storage()
            .persistent()
            .get(&IncentiveDataKey::StakingPool(pool_id.clone()))
            .ok_or(Error::NotInitialized)
    }

    fn get_token_balance(env: &Env, user: &Address) -> i128 {
        // Would integrate with main token contract
        0 // Placeholder
    }

    fn grant_premium_access(
        env: &Env,
        user: &Address,
        features: &Vec<PremiumFeature>,
    ) -> Result<(), Error> {
        for feature in features.iter() {
            let access = PremiumAccess {
                user: user.clone(),
                feature: feature.clone(),
                granted_at: env.ledger().timestamp(),
                expires_at: None, // Permanent while staking
                source: AccessSource::Staking,
            };

            // Store premium access (simplified key structure)
            env.storage().persistent().set(
                &format!("premium_{}_{}", user.to_string(), feature.clone() as u32),
                &access,
            );
        }

        Ok(())
    }

    fn update_burn_stats(env: &Env, amount: i128) -> Result<(), Error> {
        let mut global_stats: GlobalStats = env.storage()
            .persistent()
            .get(&IncentiveDataKey::GlobalStats)
            .unwrap_or_default();

        global_stats.total_tokens_burned += amount;
        global_stats.last_updated = env.ledger().timestamp();

        env.storage().persistent().set(&IncentiveDataKey::GlobalStats, &global_stats);
        Ok(())
    }
}

// Default implementations
impl Default for TokenomicsConfig {
    fn default() -> Self {
        Self {
            base_course_reward: 100_000,
            base_module_reward: 10_000,
            streak_bonus_rate: 500,
            max_streak_multiplier: 300,
            referral_reward: 50_000,
            achievement_bonus_rate: 1000,
            burn_discount_rate: 2000,
            inflation_rate: 500,
            max_supply: 1_000_000_000_000,
            treasury_address: Address::from_string(&String::from_str(&Env::default(), "treasury")),
        }
    }
}

impl Default for GlobalStats {
    fn default() -> Self {
        Self {
            total_tokens_minted: 0,
            total_tokens_burned: 0,
            total_rewards_distributed: 0,
            total_staked: 0,
            active_users: 0,
            total_achievements_earned: 0,
            last_updated: 0,
        }
    }
}
