use soroban_sdk::{Env, Vec};

use crate::types::{
    GamificationKey, GamificationProfile, Guild, GuildLeaderboardEntry, LeaderboardCategory,
    LeaderboardEntry, SeasonLeaderboardEntry,
};

/// Maximum number of entries kept per leaderboard.
const MAX_ENTRIES: u32 = 50;

pub struct LeaderboardManager;

impl LeaderboardManager {
    // ── Global leaderboards ────────────────────────────────────────────────

    /// Called whenever a user's profile changes to reflect the new score
    /// across all relevant leaderboard categories.
    pub fn update_user_score(env: &Env, profile: &GamificationProfile) {
        Self::upsert(env, &profile.user, profile.total_xp, &LeaderboardCategory::TotalXP);
        Self::upsert(
            env,
            &profile.user,
            profile.current_streak,
            &LeaderboardCategory::CurrentStreak,
        );
        Self::upsert(
            env,
            &profile.user,
            profile.courses_completed,
            &LeaderboardCategory::CoursesCompleted,
        );
        Self::upsert(
            env,
            &profile.user,
            profile.reputation_score,
            &LeaderboardCategory::Reputation,
        );
        Self::upsert(
            env,
            &profile.user,
            profile.season_xp,
            &LeaderboardCategory::SeasonXP,
        );
        Self::upsert(
            env,
            &profile.user,
            profile.challenges_completed,
            &LeaderboardCategory::ChallengesCompleted,
        );
        Self::upsert(
            env,
            &profile.user,
            profile.endorsements_received,
            &LeaderboardCategory::Endorsements,
        );
    }

    pub fn get_leaderboard(
        env: &Env,
        category: &LeaderboardCategory,
        limit: u32,
    ) -> Vec<LeaderboardEntry> {
        let all: Vec<LeaderboardEntry> = env
            .storage()
            .persistent()
            .get(&GamificationKey::Leaderboard(category.clone()))
            .unwrap_or_else(|| Vec::new(env));

        let cap = limit.min(MAX_ENTRIES).min(all.len()) as u32;
        let mut out = Vec::new(env);
        for i in 0..cap {
            if let Some(e) = all.get(i) {
                out.push_back(e);
            }
        }
        out
    }

    // ── Guild leaderboard ──────────────────────────────────────────────────

    pub fn update_guild_score(env: &Env, guild: &Guild) {
        let key = GamificationKey::GuildLeaderboard;
        let existing: Vec<GuildLeaderboardEntry> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));

        // Remove existing entry for this guild
        let filtered = Self::remove_guild(env, &existing, guild.id);

        // Build new entry
        let entry = GuildLeaderboardEntry {
            guild_id: guild.id,
            guild_name: guild.name.clone(),
            total_xp: guild.total_xp,
            member_count: guild.member_count,
            rank: 0, // will be assigned after sort
        };

        // Insert sorted (descending by total_xp)
        let mut sorted = Self::insert_guild_sorted(env, &filtered, entry);
        Self::assign_guild_ranks(env, &mut sorted);

        env.storage().persistent().set(&key, &sorted);
    }

    pub fn get_guild_leaderboard(env: &Env) -> Vec<GuildLeaderboardEntry> {
        env.storage()
            .persistent()
            .get(&GamificationKey::GuildLeaderboard)
            .unwrap_or_else(|| Vec::new(env))
    }

    // ── Season leaderboard ─────────────────────────────────────────────────

    pub fn update_season_score(env: &Env, season_id: u64, entry: SeasonLeaderboardEntry) {
        let key = GamificationKey::SeasonLeaderboard(season_id);
        let existing: Vec<SeasonLeaderboardEntry> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));

        // Remove old entry for this user
        let filtered = Self::remove_season_user(env, &existing, &entry.user);

        // Insert sorted descending by season_xp
        let mut sorted = Self::insert_season_sorted(env, &filtered, entry);
        Self::assign_season_ranks(env, &mut sorted);

        env.storage().persistent().set(&key, &sorted);
    }

    pub fn get_season_leaderboard(env: &Env, season_id: u64) -> Vec<SeasonLeaderboardEntry> {
        env.storage()
            .persistent()
            .get(&GamificationKey::SeasonLeaderboard(season_id))
            .unwrap_or_else(|| Vec::new(env))
    }

    // ── Internals – global ─────────────────────────────────────────────────

    fn upsert(
        env: &Env,
        user: &soroban_sdk::Address,
        score: u32,
        category: &LeaderboardCategory,
    ) {
        let key = GamificationKey::Leaderboard(category.clone());
        let existing: Vec<LeaderboardEntry> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));

        // Remove old entry for this user
        let filtered = Self::remove_user(env, &existing, user);

        // Build new entry
        let entry = LeaderboardEntry {
            user: user.clone(),
            score,
            rank: 0,
            category: category.clone(),
        };

        // Insert at correct sorted position
        let mut sorted = Self::insert_sorted(env, &filtered, entry);

        // Trim to MAX_ENTRIES
        Self::trim(&mut sorted, env);

        // Assign ranks
        Self::assign_ranks(env, &mut sorted, category);

        env.storage().persistent().set(&key, &sorted);
    }

    fn remove_user(
        env: &Env,
        board: &Vec<LeaderboardEntry>,
        user: &soroban_sdk::Address,
    ) -> Vec<LeaderboardEntry> {
        let mut out = Vec::new(env);
        for e in board.iter() {
            if &e.user != user {
                out.push_back(e);
            }
        }
        out
    }

    /// Insert `entry` into a descending-sorted `board`.
    fn insert_sorted(
        env: &Env,
        board: &Vec<LeaderboardEntry>,
        entry: LeaderboardEntry,
    ) -> Vec<LeaderboardEntry> {
        let mut out = Vec::new(env);
        let mut inserted = false;
        for e in board.iter() {
            if !inserted && entry.score >= e.score {
                out.push_back(entry.clone());
                inserted = true;
            }
            out.push_back(e);
        }
        if !inserted {
            out.push_back(entry);
        }
        out
    }

    fn trim(board: &mut Vec<LeaderboardEntry>, env: &Env) {
        if board.len() <= MAX_ENTRIES {
            return;
        }
        let mut trimmed = Vec::new(env);
        for i in 0..MAX_ENTRIES {
            if let Some(e) = board.get(i) {
                trimmed.push_back(e);
            }
        }
        *board = trimmed;
    }

    fn assign_ranks(
        env: &Env,
        board: &mut Vec<LeaderboardEntry>,
        category: &LeaderboardCategory,
    ) {
        let mut ranked = Vec::new(env);
        let mut rank: u32 = 1;
        for mut e in board.iter() {
            e.rank = rank;
            e.category = category.clone();
            ranked.push_back(e);
            rank += 1;
        }
        *board = ranked;
    }

    // ── Internals – guild ──────────────────────────────────────────────────

    fn remove_guild(
        env: &Env,
        board: &Vec<GuildLeaderboardEntry>,
        guild_id: u64,
    ) -> Vec<GuildLeaderboardEntry> {
        let mut out = Vec::new(env);
        for e in board.iter() {
            if e.guild_id != guild_id {
                out.push_back(e);
            }
        }
        out
    }

    fn insert_guild_sorted(
        env: &Env,
        board: &Vec<GuildLeaderboardEntry>,
        entry: GuildLeaderboardEntry,
    ) -> Vec<GuildLeaderboardEntry> {
        let mut out = Vec::new(env);
        let mut inserted = false;
        for e in board.iter() {
            if !inserted && entry.total_xp >= e.total_xp {
                out.push_back(entry.clone());
                inserted = true;
            }
            out.push_back(e);
        }
        if !inserted {
            out.push_back(entry);
        }
        out
    }

    fn assign_guild_ranks(env: &Env, board: &mut Vec<GuildLeaderboardEntry>) {
        let mut ranked = Vec::new(env);
        let mut rank: u32 = 1;
        for mut e in board.iter() {
            e.rank = rank;
            ranked.push_back(e);
            rank += 1;
        }
        *board = ranked;
    }

    // ── Internals – season ─────────────────────────────────────────────────

    fn remove_season_user(
        env: &Env,
        board: &Vec<SeasonLeaderboardEntry>,
        user: &soroban_sdk::Address,
    ) -> Vec<SeasonLeaderboardEntry> {
        let mut out = Vec::new(env);
        for e in board.iter() {
            if &e.user != user {
                out.push_back(e);
            }
        }
        out
    }

    fn insert_season_sorted(
        env: &Env,
        board: &Vec<SeasonLeaderboardEntry>,
        entry: SeasonLeaderboardEntry,
    ) -> Vec<SeasonLeaderboardEntry> {
        let mut out = Vec::new(env);
        let mut inserted = false;
        for e in board.iter() {
            if !inserted && entry.season_xp >= e.season_xp {
                out.push_back(entry.clone());
                inserted = true;
            }
            out.push_back(e);
        }
        if !inserted {
            out.push_back(entry);
        }
        out
    }

    fn assign_season_ranks(env: &Env, board: &mut Vec<SeasonLeaderboardEntry>) {
        let total = board.len();
        let mut ranked = Vec::new(env);
        let mut rank: u32 = 1;
        for mut e in board.iter() {
            e.rank = rank;
            e.reward_tier = Self::season_reward_tier(rank, total);
            ranked.push_back(e);
            rank += 1;
        }
        *board = ranked;
    }

    fn season_reward_tier(rank: u32, total: u32) -> crate::types::SeasonRewardTier {
        use crate::types::SeasonRewardTier;
        if total == 0 {
            return SeasonRewardTier::None;
        }
        let pct = rank * 100 / total; // rank 1 → pct ≈ 0
        if pct < 1 {
            SeasonRewardTier::Diamond
        } else if pct < 10 {
            SeasonRewardTier::Gold
        } else if pct < 25 {
            SeasonRewardTier::Silver
        } else if pct < 50 {
            SeasonRewardTier::Bronze
        } else {
            SeasonRewardTier::None
        }
    }
}
