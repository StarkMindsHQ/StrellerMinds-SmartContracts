pub mod achievements;
pub mod challenges;
pub mod errors;
pub mod events;
pub mod guilds;
pub mod leaderboard;
pub mod reputation;
pub mod seasons;
pub mod social;
pub mod storage;
pub mod types;

#[cfg(test)]
mod tests;

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

pub use errors::Error;
pub use types::*;

use achievements::AchievementManager;
use challenges::ChallengeManager;
use guilds::GuildManager;
use leaderboard::LeaderboardManager;
use reputation::ReputationManager;
use seasons::SeasonManager;
use social::SocialManager;
use storage::GamificationStorage;

#[contract]
pub struct Gamification;

#[contractimpl]
impl Gamification {
    // ══════════════════════════════════════════════════════════════════════
    //  Initialization
    // ══════════════════════════════════════════════════════════════════════

    /// One-time setup.  Seeds the 25 default milestone achievements.
    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();

        if GamificationStorage::is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage()
            .instance()
            .set(&GamificationKey::Admin, &admin);

        // Default config
        let config = GamificationConfig {
            base_module_xp: 50,
            base_course_xp: 500,
            streak_weekly_bonus: 25,
            max_streak_bonus_xp: 500,
            endorsement_xp: 25,
            help_xp: 30,
            max_endorsements_per_day: 5,
            guild_max_members: 50,
            leaderboard_size: 50,
        };
        env.storage()
            .instance()
            .set(&GamificationKey::Config, &config);

        // Initialise all counters
        for key in [
            GamificationKey::AchievementCounter,
            GamificationKey::ChallengeCounter,
            GamificationKey::GuildCounter,
            GamificationKey::SeasonCounter,
            GamificationKey::EndorsementCounter,
            GamificationKey::RecognitionCounter,
        ] {
            env.storage().persistent().set(&key, &0u64);
        }
        env.storage()
            .persistent()
            .set(&GamificationKey::ActiveSeasonId, &0u64);

        // Seed milestone achievements
        AchievementManager::seed_default_achievements(&env);

        events::GamificationEvents::emit_initialized(&env, &admin);
        Ok(())
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Core Activity Recording
    // ══════════════════════════════════════════════════════════════════════

    /// Record a learning activity for `user`.
    /// Returns the list of achievement IDs newly awarded as a result.
    pub fn record_activity(
        env: Env,
        user: Address,
        activity: ActivityRecord,
    ) -> Result<Vec<u64>, Error> {
        user.require_auth();
        AchievementManager::process_activity(&env, &user, &activity)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Profile Queries
    // ══════════════════════════════════════════════════════════════════════

    pub fn get_user_profile(env: Env, user: Address) -> GamificationProfile {
        GamificationStorage::get_profile(&env, &user)
    }

    pub fn get_adaptive_difficulty(env: Env, user: Address) -> AdaptiveDifficulty {
        AchievementManager::get_adaptive_difficulty(&env, &user)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Achievement Functions
    // ══════════════════════════════════════════════════════════════════════

    /// Admin: create a custom achievement beyond the 25 seeded milestones.
    pub fn create_achievement(
        env: Env,
        admin: Address,
        achievement: Achievement,
    ) -> Result<u64, Error> {
        admin.require_auth();
        GamificationStorage::require_admin(&env, &admin)?;
        AchievementManager::create(&env, achievement)
    }

    pub fn get_user_achievements(env: Env, user: Address) -> Vec<UserAchievement> {
        AchievementManager::get_user_achievements(&env, &user)
    }

    /// Claim the token reward attached to an earned achievement.
    pub fn claim_achievement_reward(
        env: Env,
        user: Address,
        achievement_id: u64,
    ) -> Result<i128, Error> {
        user.require_auth();
        AchievementManager::claim_reward(&env, &user, achievement_id)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Leaderboard Functions
    // ══════════════════════════════════════════════════════════════════════

    /// Returns up to `limit` entries for `category` (max 50).
    pub fn get_leaderboard(
        env: Env,
        category: LeaderboardCategory,
        limit: u32,
    ) -> Vec<LeaderboardEntry> {
        LeaderboardManager::get_leaderboard(&env, &category, limit)
    }

    pub fn get_guild_leaderboard(env: Env) -> Vec<GuildLeaderboardEntry> {
        LeaderboardManager::get_guild_leaderboard(&env)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Challenge / Quest Functions
    // ══════════════════════════════════════════════════════════════════════

    pub fn create_challenge(env: Env, admin: Address, challenge: Challenge) -> Result<u64, Error> {
        admin.require_auth();
        GamificationStorage::require_admin(&env, &admin)?;
        ChallengeManager::create(&env, &admin, challenge)
    }

    pub fn join_challenge(env: Env, user: Address, challenge_id: u64) -> Result<(), Error> {
        user.require_auth();
        ChallengeManager::join(&env, &user, challenge_id)
    }

    /// Update progress on an active challenge.  Returns `true` when completed.
    pub fn update_challenge_progress(
        env: Env,
        user: Address,
        challenge_id: u64,
        progress: u32,
    ) -> Result<bool, Error> {
        user.require_auth();
        ChallengeManager::update_progress(&env, &user, challenge_id, progress)
    }

    pub fn get_active_challenges(env: Env) -> Vec<Challenge> {
        ChallengeManager::get_active_challenges(&env)
    }

    pub fn get_user_challenge_status(
        env: Env,
        user: Address,
        challenge_id: u64,
    ) -> Option<UserChallenge> {
        ChallengeManager::get_user_challenge(&env, &user, challenge_id)
    }

    pub fn get_user_active_challenges(env: Env, user: Address) -> Vec<u64> {
        ChallengeManager::get_user_active_challenges(&env, &user)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Guild Functions
    // ══════════════════════════════════════════════════════════════════════

    pub fn create_guild(
        env: Env,
        creator: Address,
        name: String,
        description: String,
        max_members: u32,
        is_public: bool,
    ) -> Result<u64, Error> {
        creator.require_auth();
        GuildManager::create(&env, &creator, name, description, max_members, is_public)
    }

    pub fn join_guild(env: Env, user: Address, guild_id: u64) -> Result<(), Error> {
        user.require_auth();
        GuildManager::join(&env, &user, guild_id)
    }

    pub fn leave_guild(env: Env, user: Address) -> Result<(), Error> {
        user.require_auth();
        GuildManager::leave(&env, &user)
    }

    pub fn get_guild(env: Env, guild_id: u64) -> Option<Guild> {
        GuildManager::get_guild(&env, guild_id)
    }

    pub fn get_guild_members(env: Env, guild_id: u64) -> Vec<GuildMember> {
        GuildManager::get_members(&env, guild_id)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Season Functions
    // ══════════════════════════════════════════════════════════════════════

    pub fn create_season(env: Env, admin: Address, season: Season) -> Result<u64, Error> {
        admin.require_auth();
        GamificationStorage::require_admin(&env, &admin)?;
        SeasonManager::create(&env, season)
    }

    pub fn get_active_season(env: Env) -> Option<Season> {
        SeasonManager::get_active_season(&env)
    }

    /// End the current season (only callable after `end_time` has passed).
    pub fn end_season(env: Env, admin: Address) -> Result<(), Error> {
        admin.require_auth();
        GamificationStorage::require_admin(&env, &admin)?;
        SeasonManager::end_current_season(&env)
    }

    pub fn get_season_leaderboard(env: Env, season_id: u64) -> Vec<SeasonLeaderboardEntry> {
        SeasonManager::get_leaderboard(&env, season_id)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Social Functions
    // ══════════════════════════════════════════════════════════════════════

    pub fn endorse_peer(
        env: Env,
        endorser: Address,
        endorsee: Address,
        skill: String,
    ) -> Result<(), Error> {
        endorser.require_auth();
        SocialManager::endorse(&env, &endorser, &endorsee, skill)
    }

    pub fn recognize_peer(
        env: Env,
        from: Address,
        to: Address,
        recognition_type: RecognitionType,
        message: String,
    ) -> Result<(), Error> {
        from.require_auth();
        SocialManager::recognize(&env, &from, &to, recognition_type, message)
    }

    pub fn get_user_endorsements(env: Env, user: Address) -> Vec<PeerEndorsement> {
        SocialManager::get_endorsements(&env, &user)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Reputation Functions
    // ══════════════════════════════════════════════════════════════════════

    pub fn get_reputation(env: Env, user: Address) -> ReputationScore {
        ReputationManager::get_reputation(&env, &user)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Admin Functions
    // ══════════════════════════════════════════════════════════════════════

    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&GamificationKey::Admin)
    }
}
