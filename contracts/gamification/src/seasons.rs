use soroban_sdk::{Address, Env, Vec};

use crate::errors::Error;
use crate::events::GamificationEvents;
use crate::storage::GamificationStorage;
use crate::types::{GamificationKey, Season, SeasonLeaderboardEntry, SeasonRewardTier};

pub struct SeasonManager;

impl SeasonManager {
    // ── Create ─────────────────────────────────────────────────────────────

    pub fn create(env: &Env, mut season: Season) -> Result<u64, Error> {
        if GamificationStorage::get_active_season_id(env) != 0 {
            return Err(Error::SeasonAlreadyActive);
        }
        if season.end_time <= season.start_time {
            return Err(Error::InvalidInput);
        }
        if season.xp_multiplier == 0 {
            season.xp_multiplier = 100; // default 1×
        }

        let id = GamificationStorage::next_id(env, &GamificationKey::SeasonCounter);
        season.id = id;
        season.is_active = true;
        season.total_participants = 0;

        env.storage()
            .persistent()
            .set(&GamificationKey::Season(id), &season);
        GamificationStorage::set_active_season_id(env, id);

        GamificationEvents::emit_season_started(env, id);
        Ok(id)
    }

    // ── End current season ─────────────────────────────────────────────────

    pub fn end_current_season(env: &Env) -> Result<(), Error> {
        let id = GamificationStorage::get_active_season_id(env);
        if id == 0 {
            return Err(Error::SeasonInactive);
        }

        let mut season: Season = env
            .storage()
            .persistent()
            .get(&GamificationKey::Season(id))
            .ok_or(Error::NotFound)?;

        let now = env.ledger().timestamp();
        if now < season.end_time {
            return Err(Error::SeasonNotEnded);
        }

        season.is_active = false;
        env.storage()
            .persistent()
            .set(&GamificationKey::Season(id), &season);
        GamificationStorage::set_active_season_id(env, 0);

        GamificationEvents::emit_season_ended(env, id);
        Ok(())
    }

    // ── XP contribution (called by AchievementManager) ────────────────────

    /// Adds `xp` to the user's season tally (if a season is active).
    /// Returns the new season XP for the user (0 if no active season).
    pub fn add_season_xp(env: &Env, user: &Address, xp: u32) -> u32 {
        let season_id = GamificationStorage::get_active_season_id(env);
        if season_id == 0 || xp == 0 {
            return 0;
        }

        // Check season is still within time window
        let season: Season = match env
            .storage()
            .persistent()
            .get(&GamificationKey::Season(season_id))
        {
            Some(s) => s,
            None => return 0,
        };
        let now = env.ledger().timestamp();
        if !season.is_active || now > season.end_time {
            return 0;
        }

        let key = GamificationKey::UserSeasonXP(user.clone(), season_id);
        let prev: u32 = env.storage().persistent().get(&key).unwrap_or(0u32);
        let is_new_participant = prev == 0;
        let new_xp = prev + xp;

        env.storage().persistent().set(&key, &new_xp);

        // Update season participant count for new entrants
        if is_new_participant {
            if let Some(mut s) = env
                .storage()
                .persistent()
                .get::<GamificationKey, Season>(&GamificationKey::Season(season_id))
            {
                s.total_participants += 1;
                env.storage()
                    .persistent()
                    .set(&GamificationKey::Season(season_id), &s);
            }
        }

        // Update season leaderboard
        let entry = SeasonLeaderboardEntry {
            user: user.clone(),
            season_xp: new_xp,
            rank: 0,
            reward_tier: SeasonRewardTier::None,
        };
        crate::leaderboard::LeaderboardManager::update_season_score(env, season_id, entry);

        // Guild season XP
        crate::guilds::GuildManager::add_season_xp(env, user, xp);

        new_xp
    }

    // ── Queries ────────────────────────────────────────────────────────────

    pub fn get_active_season(env: &Env) -> Option<Season> {
        let id = GamificationStorage::get_active_season_id(env);
        if id == 0 {
            return None;
        }
        env.storage()
            .persistent()
            .get(&GamificationKey::Season(id))
    }

    pub fn get_leaderboard(env: &Env, season_id: u64) -> Vec<SeasonLeaderboardEntry> {
        crate::leaderboard::LeaderboardManager::get_season_leaderboard(env, season_id)
    }

    /// Returns the XP multiplier (100 = 1×) for the currently active season.
    pub fn get_xp_multiplier(env: &Env) -> u32 {
        let id = GamificationStorage::get_active_season_id(env);
        if id == 0 {
            return 100;
        }
        let season: Option<Season> = env
            .storage()
            .persistent()
            .get(&GamificationKey::Season(id));
        season
            .filter(|s| s.is_active)
            .map(|s| s.xp_multiplier)
            .unwrap_or(100)
    }
}
