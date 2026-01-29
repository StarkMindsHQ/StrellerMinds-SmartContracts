use crate::types::{
    Achievement, BurnTransaction, GlobalStats, IncentiveEvent, LeaderboardCategory,
    LeaderboardEntry, PremiumAccess, PremiumFeature, ReferralData, StakingPool, TokenomicsConfig,
    UserAchievement, UserStake, UserStats,
};
use crate::Error;
use soroban_sdk::{Address, Env, String, Vec};

/// Token contract interface with incentive system
#[allow(dead_code)]
pub trait TokenTrait {
    // === Basic Token Operations ===

    /// Initialize the token contract
    fn initialize(env: Env, admin: Address) -> Result<(), Error>;

    /// Mint tokens to an address
    fn mint(env: Env, to: Address, amount: i128) -> Result<(), Error>;

    /// Get token balance
    fn balance(env: Env, id: Address) -> i128;

    /// Transfer tokens between addresses
    fn transfer(env: Env, from: Address, to: Address, amount: i128) -> Result<(), Error>;

    /// Burn tokens from an address
    fn burn(env: Env, from: Address, amount: i128) -> Result<(), Error>;

    // === Incentive System ===

    /// Initialize the incentive system
    fn initialize_incentives(env: Env, admin: Address) -> Result<(), Error>;

    /// Reward user for course completion
    fn reward_course_completion(
        env: Env,
        user: Address,
        course_id: String,
        completion_percentage: u32,
    ) -> Result<i128, Error>;

    /// Reward user for module completion
    fn reward_module_completion(
        env: Env,
        user: Address,
        course_id: String,
        module_id: String,
    ) -> Result<i128, Error>;

    /// Reward user for achievement
    fn reward_achievement(env: Env, user: Address, achievement_id: String) -> Result<i128, Error>;

    /// Reward user for referral
    fn reward_referral(env: Env, referrer: Address, referee: Address) -> Result<i128, Error>;

    // === Achievement System ===

    /// Create new achievement
    fn create_achievement(
        env: Env,
        admin: Address,
        achievement: Achievement,
    ) -> Result<String, Error>;

    /// Get achievement details
    fn get_achievement(env: Env, achievement_id: String) -> Option<Achievement>;

    /// Get user achievements
    fn get_user_achievements(env: Env, user: Address) -> Vec<UserAchievement>;

    /// Claim achievement reward
    fn claim_achievement_reward(
        env: Env,
        user: Address,
        achievement_id: String,
    ) -> Result<i128, Error>;

    /// Check and award achievements for user
    fn check_achievements(env: Env, user: Address) -> Result<Vec<String>, Error>;

    // === Staking System ===

    /// Create staking pool
    fn create_staking_pool(env: Env, admin: Address, pool: StakingPool) -> Result<String, Error>;

    /// Get staking pool details
    fn get_staking_pool(env: Env, pool_id: String) -> Option<StakingPool>;

    /// Stake tokens in pool
    fn stake_tokens(env: Env, user: Address, pool_id: String, amount: i128) -> Result<(), Error>;

    /// Unstake tokens from pool
    fn unstake_tokens(env: Env, user: Address, pool_id: String, amount: i128) -> Result<(), Error>;

    /// Claim staking rewards
    fn claim_staking_rewards(env: Env, user: Address, pool_id: String) -> Result<i128, Error>;

    /// Get user stake in pool
    fn get_user_stake(env: Env, user: Address, pool_id: String) -> Option<UserStake>;

    /// Get all user stakes
    fn get_user_stakes(env: Env, user: Address) -> Vec<UserStake>;

    // === Token Burning ===

    /// Burn tokens for certificate upgrade
    fn burn_for_upgrade(
        env: Env,
        user: Address,
        amount: i128,
        certificate_id: String,
        upgrade_type: String,
    ) -> Result<String, Error>;

    /// Burn tokens for premium feature
    fn burn_for_premium(
        env: Env,
        user: Address,
        amount: i128,
        feature: PremiumFeature,
        duration: u64,
    ) -> Result<String, Error>;

    /// Get burn transaction details
    fn get_burn_transaction(env: Env, burn_id: String) -> Option<BurnTransaction>;

    /// Get user burn history
    fn get_user_burns(env: Env, user: Address) -> Vec<BurnTransaction>;

    // === Premium Features ===

    /// Check if user has premium access
    fn has_premium_access(env: Env, user: Address, feature: PremiumFeature) -> bool;

    /// Get user premium features
    fn get_user_premium_features(env: Env, user: Address) -> Vec<PremiumAccess>;

    /// Grant premium access (admin only)
    fn grant_premium_access(
        env: Env,
        admin: Address,
        user: Address,
        feature: PremiumFeature,
        duration: Option<u64>,
    ) -> Result<(), Error>;

    /// Revoke premium access (admin only)
    fn revoke_premium_access(
        env: Env,
        admin: Address,
        user: Address,
        feature: PremiumFeature,
    ) -> Result<(), Error>;

    // === Statistics and Analytics ===

    /// Get user statistics
    fn get_user_stats(env: Env, user: Address) -> UserStats;

    /// Get global statistics
    fn get_global_stats(env: Env) -> GlobalStats;

    /// Get leaderboard
    fn get_leaderboard(
        env: Env,
        category: LeaderboardCategory,
        limit: u32,
    ) -> Vec<LeaderboardEntry>;

    /// Update user streak
    fn update_user_streak(env: Env, user: Address) -> Result<u32, Error>;

    /// Get user current streak
    fn get_user_streak(env: Env, user: Address) -> u32;

    // === Events and Campaigns ===

    /// Create incentive event
    fn create_incentive_event(
        env: Env,
        admin: Address,
        event: IncentiveEvent,
    ) -> Result<String, Error>;

    /// Get active events
    fn get_active_events(env: Env) -> Vec<IncentiveEvent>;

    /// Get event details
    fn get_incentive_event(env: Env, event_id: String) -> Option<IncentiveEvent>;

    /// Activate/deactivate event
    fn toggle_event_status(
        env: Env,
        admin: Address,
        event_id: String,
        active: bool,
    ) -> Result<(), Error>;

    // === Referral System ===

    /// Create referral
    fn create_referral(env: Env, referrer: Address, referee: Address) -> Result<String, Error>;

    /// Get user referrals
    fn get_user_referrals(env: Env, user: Address) -> Vec<ReferralData>;

    /// Claim referral reward
    fn claim_referral_reward(
        env: Env,
        referrer: Address,
        referral_id: String,
    ) -> Result<i128, Error>;

    // === Configuration ===

    /// Update tokenomics configuration (admin only)
    fn update_tokenomics_config(
        env: Env,
        admin: Address,
        config: TokenomicsConfig,
    ) -> Result<(), Error>;

    /// Get tokenomics configuration
    fn get_tokenomics_config(env: Env) -> Option<TokenomicsConfig>;

    /// Set reward multiplier for user
    fn set_reward_multiplier(
        env: Env,
        admin: Address,
        user: Address,
        multiplier: u32,
        duration: Option<u64>,
        reason: String,
    ) -> Result<(), Error>;

    /// Get user reward multiplier
    fn get_reward_multiplier(env: Env, user: Address) -> u32;

    // === Governance ===

    /// Propose tokenomics change
    fn propose_tokenomics_change(
        env: Env,
        proposer: Address,
        new_config: TokenomicsConfig,
        description: String,
    ) -> Result<String, Error>;

    /// Vote on proposal
    fn vote_on_proposal(
        env: Env,
        voter: Address,
        proposal_id: String,
        support: bool,
        voting_power: i128,
    ) -> Result<(), Error>;

    /// Execute approved proposal
    fn execute_proposal(env: Env, executor: Address, proposal_id: String) -> Result<(), Error>;

    // === Utility Functions ===

    /// Calculate reward for completion
    fn calculate_reward(env: Env, user: Address, base_amount: i128, reward_type: String) -> i128;

    /// Get total supply
    fn total_supply(env: Env) -> i128;

    /// Get circulating supply
    fn circulating_supply(env: Env) -> i128;

    /// Emergency pause (admin only)
    fn emergency_pause(env: Env, admin: Address) -> Result<(), Error>;

    /// Resume operations (admin only)
    fn resume_operations(env: Env, admin: Address) -> Result<(), Error>;

    /// Is contract paused
    fn is_paused(env: Env) -> bool;
}
