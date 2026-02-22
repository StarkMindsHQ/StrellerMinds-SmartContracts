use soroban_sdk::{Address, Env};

use crate::errors::Error;
use crate::types::{GamificationConfig, GamificationKey, GamificationProfile};

pub struct GamificationStorage;

impl GamificationStorage {
    // ── Admin ──────────────────────────────────────────────────────────────

    pub fn require_admin(env: &Env, caller: &Address) -> Result<(), Error> {
        let admin: Address = env
            .storage()
            .instance()
            .get(&GamificationKey::Admin)
            .ok_or(Error::NotInitialized)?;
        if &admin != caller {
            return Err(Error::Unauthorized);
        }
        Ok(())
    }

    pub fn is_initialized(env: &Env) -> bool {
        env.storage().instance().has(&GamificationKey::Admin)
    }

    // ── Config ─────────────────────────────────────────────────────────────

    pub fn get_config(env: &Env) -> GamificationConfig {
        env.storage()
            .instance()
            .get(&GamificationKey::Config)
            .unwrap_or_else(|| GamificationConfig {
                base_module_xp: 50,
                base_course_xp: 500,
                streak_weekly_bonus: 25,
                max_streak_bonus_xp: 500,
                endorsement_xp: 25,
                help_xp: 30,
                max_endorsements_per_day: 5,
                guild_max_members: 50,
                leaderboard_size: 50,
            })
    }

    // ── User Profile ───────────────────────────────────────────────────────

    pub fn get_profile(env: &Env, user: &Address) -> GamificationProfile {
        env.storage()
            .persistent()
            .get(&GamificationKey::UserProfile(user.clone()))
            .unwrap_or_else(|| GamificationProfile {
                user: user.clone(),
                total_xp: 0,
                level: 1,
                current_streak: 0,
                max_streak: 0,
                last_activity: 0,
                courses_completed: 0,
                modules_completed: 0,
                achievements_count: 0,
                challenges_completed: 0,
                guild_id: 0,
                reputation_score: 0,
                season_xp: 0,
                endorsements_received: 0,
                endorsements_given: 0,
                total_tokens_earned: 0,
                joined_at: env.ledger().timestamp(),
            })
    }

    pub fn set_profile(env: &Env, user: &Address, profile: &GamificationProfile) {
        env.storage()
            .persistent()
            .set(&GamificationKey::UserProfile(user.clone()), profile);
    }

    // ── Counters ───────────────────────────────────────────────────────────

    /// Increment the counter stored at `key` and return the new (post-increment) value.
    pub fn next_id(env: &Env, key: &GamificationKey) -> u64 {
        let current: u64 = env.storage().persistent().get(key).unwrap_or(0u64);
        let next = current + 1;
        env.storage().persistent().set(key, &next);
        next
    }

    // ── Active Season (0 = none) ───────────────────────────────────────────

    pub fn get_active_season_id(env: &Env) -> u64 {
        env.storage()
            .persistent()
            .get(&GamificationKey::ActiveSeasonId)
            .unwrap_or(0u64)
    }

    pub fn set_active_season_id(env: &Env, id: u64) {
        env.storage()
            .persistent()
            .set(&GamificationKey::ActiveSeasonId, &id);
    }
}
