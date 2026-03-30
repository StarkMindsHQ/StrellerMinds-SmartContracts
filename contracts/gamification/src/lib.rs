#![no_std]

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
pub use errors::GamificationError;
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
    pub fn initialize(env: Env, admin: Address) -> Result<(), GamificationError> {
        admin.require_auth();

        if GamificationStorage::is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&GamificationKey::Admin, &admin);

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
        env.storage().instance().set(&GamificationKey::Config, &config);

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
        env.storage().persistent().set(&GamificationKey::ActiveSeasonId, &0u64);

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
    ) -> Result<Vec<u64>, GamificationError> {
        user.require_auth();
        AchievementManager::process_activity(&env, &user, &activity)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Profile Queries
    // ══════════════════════════════════════════════════════════════════════

    /// Retrieve the full gamification profile for `user`, creating a default one if absent.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `user` - Address of the user whose profile to fetch.
    ///
    /// # Example
    /// ```ignore
    /// client.get_user_profile(&user);
    /// ```
    pub fn get_user_profile(env: Env, user: Address) -> GamificationProfile {
        GamificationStorage::get_profile(&env, &user)
    }

    /// Return the current adaptive difficulty settings calculated for `user`.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `user` - Address of the user to evaluate.
    ///
    /// # Example
    /// ```ignore
    /// client.get_adaptive_difficulty(&user);
    /// ```
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
    ) -> Result<u64, GamificationError> {
        admin.require_auth();
        GamificationStorage::require_admin(&env, &admin)?;
        AchievementManager::create(&env, achievement)
    }

    /// Return all achievements earned by `user`.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `user` - Address of the user whose achievements to list.
    ///
    /// # Example
    /// ```ignore
    /// client.get_user_achievements(&user);
    /// ```
    pub fn get_user_achievements(env: Env, user: Address) -> Vec<UserAchievement> {
        AchievementManager::get_user_achievements(&env, &user)
    }

    /// Claim the token reward attached to an earned achievement.
    pub fn claim_achievement_reward(
        env: Env,
        user: Address,
        achievement_id: u64,
    ) -> Result<i128, GamificationError> {
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

    /// Return the global guild leaderboard sorted by total XP.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    ///
    /// # Example
    /// ```ignore
    /// client.get_guild_leaderboard();
    /// ```
    pub fn get_guild_leaderboard(env: Env) -> Vec<GuildLeaderboardEntry> {
        LeaderboardManager::get_guild_leaderboard(&env)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Challenge / Quest Functions
    // ══════════════════════════════════════════════════════════════════════

    /// Admin: create a new time-bound challenge that users can join.
    ///
    /// Caller must be the contract admin.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `admin` - Admin address (must match stored admin).
    /// * `challenge` - Challenge definition to persist.
    ///
    /// # Errors
    /// Returns [`GamificationError::Unauthorized`] if `admin` is not the contract admin.
    ///
    /// # Example
    /// ```ignore
    /// client.create_challenge(&admin, &challenge);
    /// ```
    pub fn create_challenge(
        env: Env,
        admin: Address,
        challenge: Challenge,
    ) -> Result<u64, GamificationError> {
        admin.require_auth();
        GamificationStorage::require_admin(&env, &admin)?;
        ChallengeManager::create(&env, &admin, challenge)
    }

    /// Enroll `user` into an active challenge.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `user` - Address of the user joining the challenge.
    /// * `challenge_id` - ID of the challenge to join.
    ///
    /// # Errors
    /// Returns [`GamificationError::ChallengeInactive`] if the challenge is not active.
    /// Returns [`GamificationError::AlreadyJoinedChallenge`] if the user already joined.
    ///
    /// # Example
    /// ```ignore
    /// client.join_challenge(&user, &challenge_id);
    /// ```
    pub fn join_challenge(
        env: Env,
        user: Address,
        challenge_id: u64,
    ) -> Result<(), GamificationError> {
        user.require_auth();
        ChallengeManager::join(&env, &user, challenge_id)
    }

    /// Update progress on an active challenge.  Returns `true` when completed.
    pub fn update_challenge_progress(
        env: Env,
        user: Address,
        challenge_id: u64,
        progress: u32,
    ) -> Result<bool, GamificationError> {
        user.require_auth();
        ChallengeManager::update_progress(&env, &user, challenge_id, progress)
    }

    /// Return all currently active (non-expired) challenges.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    ///
    /// # Example
    /// ```ignore
    /// client.get_active_challenges();
    /// ```
    pub fn get_active_challenges(env: Env) -> Vec<Challenge> {
        ChallengeManager::get_active_challenges(&env)
    }

    /// Return the enrollment status of `user` for a specific challenge, or `None` if not joined.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `user` - Address of the user to query.
    /// * `challenge_id` - ID of the challenge to look up.
    ///
    /// # Example
    /// ```ignore
    /// client.get_user_challenge_status(&user, &challenge_id);
    /// ```
    pub fn get_user_challenge_status(
        env: Env,
        user: Address,
        challenge_id: u64,
    ) -> Option<UserChallenge> {
        ChallengeManager::get_user_challenge(&env, &user, challenge_id)
    }

    /// Return the IDs of all challenges `user` is currently enrolled in.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `user` - Address of the user to query.
    ///
    /// # Example
    /// ```ignore
    /// client.get_user_active_challenges(&user);
    /// ```
    pub fn get_user_active_challenges(env: Env, user: Address) -> Vec<u64> {
        ChallengeManager::get_user_active_challenges(&env, &user)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Guild Functions
    // ══════════════════════════════════════════════════════════════════════

    /// Create a new guild with `creator` as its founding member.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `creator` - Address of the user creating the guild.
    /// * `name` - Display name of the guild (must fit within the max length).
    /// * `description` - Short description of the guild's purpose.
    /// * `max_members` - Maximum number of members allowed (capped by global config).
    /// * `is_public` - Whether the guild is open for anyone to join.
    ///
    /// # Errors
    /// Returns [`GamificationError::GuildNameTooLong`] if `name` exceeds the allowed length.
    ///
    /// # Example
    /// ```ignore
    /// client.create_guild(&creator, &name, &description, &max_members, &is_public);
    /// ```
    pub fn create_guild(
        env: Env,
        creator: Address,
        name: String,
        description: String,
        max_members: u32,
        is_public: bool,
    ) -> Result<u64, GamificationError> {
        creator.require_auth();
        GuildManager::create(&env, &creator, name, description, max_members, is_public)
    }

    /// Join an existing public guild.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `user` - Address of the user joining the guild.
    /// * `guild_id` - ID of the target guild.
    ///
    /// # Errors
    /// Returns [`GamificationError::GuildFull`] if the guild has reached its member limit.
    /// Returns [`GamificationError::AlreadyInGuild`] if the user already belongs to a guild.
    ///
    /// # Example
    /// ```ignore
    /// client.join_guild(&user, &guild_id);
    /// ```
    pub fn join_guild(env: Env, user: Address, guild_id: u64) -> Result<(), GamificationError> {
        user.require_auth();
        GuildManager::join(&env, &user, guild_id)
    }

    /// Remove `user` from their current guild.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `user` - Address of the user leaving the guild.
    ///
    /// # Errors
    /// Returns [`GamificationError::NotInGuild`] if the user is not a member of any guild.
    ///
    /// # Example
    /// ```ignore
    /// client.leave_guild(&user);
    /// ```
    pub fn leave_guild(env: Env, user: Address) -> Result<(), GamificationError> {
        user.require_auth();
        GuildManager::leave(&env, &user)
    }

    /// Fetch a guild by ID, returning `None` if it does not exist.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `guild_id` - ID of the guild to retrieve.
    ///
    /// # Example
    /// ```ignore
    /// client.get_guild(&guild_id);
    /// ```
    pub fn get_guild(env: Env, guild_id: u64) -> Option<Guild> {
        GuildManager::get_guild(&env, guild_id)
    }

    /// Return all current members of a guild.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `guild_id` - ID of the guild to query.
    ///
    /// # Example
    /// ```ignore
    /// client.get_guild_members(&guild_id);
    /// ```
    pub fn get_guild_members(env: Env, guild_id: u64) -> Vec<GuildMember> {
        GuildManager::get_members(&env, guild_id)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Season Functions
    // ══════════════════════════════════════════════════════════════════════

    /// Admin: create a new competitive season.
    ///
    /// Only one season may be active at a time.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `admin` - Admin address (must match stored admin).
    /// * `season` - Season definition including start/end times and rewards.
    ///
    /// # Errors
    /// Returns [`GamificationError::Unauthorized`] if `admin` is not the contract admin.
    /// Returns [`GamificationError::SeasonAlreadyActive`] if a season is currently running.
    ///
    /// # Example
    /// ```ignore
    /// client.create_season(&admin, &season);
    /// ```
    pub fn create_season(
        env: Env,
        admin: Address,
        season: Season,
    ) -> Result<u64, GamificationError> {
        admin.require_auth();
        GamificationStorage::require_admin(&env, &admin)?;
        SeasonManager::create(&env, &admin, season)
    }

    /// Return the currently active season, or `None` if no season is running.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    ///
    /// # Example
    /// ```ignore
    /// client.get_active_season();
    /// ```
    pub fn get_active_season(env: Env) -> Option<Season> {
        SeasonManager::get_active_season(&env)
    }

    /// End the current season (only callable after `end_time` has passed).
    pub fn end_season(env: Env, admin: Address) -> Result<(), GamificationError> {
        admin.require_auth();
        GamificationStorage::require_admin(&env, &admin)?;
        SeasonManager::end_current_season(&env, &admin)
    }

    /// Return the final leaderboard snapshot for a completed season.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `season_id` - ID of the season to retrieve rankings for.
    ///
    /// # Example
    /// ```ignore
    /// client.get_season_leaderboard(&season_id);
    /// ```
    pub fn get_season_leaderboard(env: Env, season_id: u64) -> Vec<SeasonLeaderboardEntry> {
        SeasonManager::get_leaderboard(&env, season_id)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Social Functions
    // ══════════════════════════════════════════════════════════════════════

    /// Endorse `endorsee` for a specific `skill`, awarding XP to both parties.
    ///
    /// A user cannot endorse themselves, and daily endorsement limits apply.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `endorser` - Address of the user giving the endorsement.
    /// * `endorsee` - Address of the user receiving the endorsement.
    /// * `skill` - Name of the skill being endorsed.
    ///
    /// # Errors
    /// Returns [`GamificationError::SelfEndorsement`] if `endorser` and `endorsee` are the same.
    /// Returns [`GamificationError::EndorsementLimitReached`] if the daily cap is exceeded.
    ///
    /// # Example
    /// ```ignore
    /// client.endorse_peer(&endorser, &endorsee, &skill);
    /// ```
    pub fn endorse_peer(
        env: Env,
        endorser: Address,
        endorsee: Address,
        skill: String,
    ) -> Result<(), GamificationError> {
        endorser.require_auth();
        SocialManager::endorse(&env, &endorser, &endorsee, skill)
    }

    /// Send a public recognition badge from `from` to `to`.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `from` - Address of the user sending the recognition.
    /// * `to` - Address of the user being recognized.
    /// * `recognition_type` - Category of the recognition award.
    /// * `message` - Optional text message accompanying the recognition.
    ///
    /// # Example
    /// ```ignore
    /// client.recognize_peer(&from, &to, &recognition_type, &message);
    /// ```
    pub fn recognize_peer(
        env: Env,
        from: Address,
        to: Address,
        recognition_type: RecognitionType,
        message: String,
    ) -> Result<(), GamificationError> {
        from.require_auth();
        SocialManager::recognize(&env, &from, &to, recognition_type, message)
    }

    /// Return all peer endorsements received by `user`.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `user` - Address of the user whose endorsements to list.
    ///
    /// # Example
    /// ```ignore
    /// client.get_user_endorsements(&user);
    /// ```
    pub fn get_user_endorsements(env: Env, user: Address) -> Vec<PeerEndorsement> {
        SocialManager::get_endorsements(&env, &user)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Reputation Functions
    // ══════════════════════════════════════════════════════════════════════

    /// Return the computed reputation score for `user`.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `user` - Address of the user to evaluate.
    ///
    /// # Example
    /// ```ignore
    /// client.get_reputation(&user);
    /// ```
    pub fn get_reputation(env: Env, user: Address) -> ReputationScore {
        ReputationManager::get_reputation(&env, &user)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Admin Functions
    // ══════════════════════════════════════════════════════════════════════

    /// Return the admin address stored in the contract, or `None` if not yet initialized.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    ///
    /// # Example
    /// ```ignore
    /// client.get_admin();
    /// ```
    pub fn get_admin(env: Env) -> Option<Address> {
        env.storage().instance().get(&GamificationKey::Admin)
    }
}
