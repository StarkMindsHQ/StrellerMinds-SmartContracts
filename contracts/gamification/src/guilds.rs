use soroban_sdk::{Address, Env, String, Vec};

use crate::errors::Error;
use crate::events::GamificationEvents;
use crate::storage::GamificationStorage;
use crate::types::{Guild, GuildMember, GuildRole, GamificationKey};

pub struct GuildManager;

impl GuildManager {
    // ── Create ─────────────────────────────────────────────────────────────

    pub fn create(
        env: &Env,
        creator: &Address,
        name: String,
        description: String,
        max_members: u32,
        is_public: bool,
    ) -> Result<u64, Error> {
        // Must not already be in a guild
        let profile = GamificationStorage::get_profile(env, creator);
        if profile.guild_id != 0 {
            return Err(Error::AlreadyInGuild);
        }

        let config = GamificationStorage::get_config(env);
        let effective_max = if max_members == 0 || max_members > config.guild_max_members {
            config.guild_max_members
        } else {
            max_members
        };

        let id = GamificationStorage::next_id(env, &GamificationKey::GuildCounter);
        let now = env.ledger().timestamp();

        let guild = Guild {
            id,
            name,
            description,
            leader: creator.clone(),
            total_xp: 0,
            member_count: 1,
            max_members: effective_max,
            is_public,
            created_at: now,
            challenge_wins: 0,
            season_xp: 0,
        };

        env.storage()
            .persistent()
            .set(&GamificationKey::Guild(id), &guild);

        // Register creator as leader
        let member = GuildMember {
            user: creator.clone(),
            guild_id: id,
            role: GuildRole::Leader,
            joined_at: now,
            contribution_xp: 0,
            challenges_participated: 0,
        };
        env.storage()
            .persistent()
            .set(&GamificationKey::GuildMember(creator.clone()), &member);

        let mut members = Vec::new(env);
        members.push_back(creator.clone());
        env.storage()
            .persistent()
            .set(&GamificationKey::GuildMembers(id), &members);

        // Update creator's profile
        let mut p = GamificationStorage::get_profile(env, creator);
        p.guild_id = id;
        GamificationStorage::set_profile(env, creator, &p);

        // Update guild leaderboard
        crate::leaderboard::LeaderboardManager::update_guild_score(env, &guild);

        GamificationEvents::emit_guild_created(env, id, creator);
        Ok(id)
    }

    // ── Join ───────────────────────────────────────────────────────────────

    pub fn join(env: &Env, user: &Address, guild_id: u64) -> Result<(), Error> {
        let profile = GamificationStorage::get_profile(env, user);
        if profile.guild_id != 0 {
            return Err(Error::AlreadyInGuild);
        }

        let mut guild: Guild = env
            .storage()
            .persistent()
            .get(&GamificationKey::Guild(guild_id))
            .ok_or(Error::NotFound)?;

        if !guild.is_public {
            return Err(Error::Unauthorized); // invite only
        }
        if guild.member_count >= guild.max_members {
            return Err(Error::GuildFull);
        }

        let now = env.ledger().timestamp();
        let member = GuildMember {
            user: user.clone(),
            guild_id,
            role: GuildRole::Member,
            joined_at: now,
            contribution_xp: 0,
            challenges_participated: 0,
        };
        env.storage()
            .persistent()
            .set(&GamificationKey::GuildMember(user.clone()), &member);

        // Append to member list
        let ml_key = GamificationKey::GuildMembers(guild_id);
        let mut ml: Vec<Address> = env
            .storage()
            .persistent()
            .get(&ml_key)
            .unwrap_or_else(|| Vec::new(env));
        ml.push_back(user.clone());
        env.storage().persistent().set(&ml_key, &ml);

        guild.member_count += 1;
        env.storage()
            .persistent()
            .set(&GamificationKey::Guild(guild_id), &guild);

        // Update user profile
        let mut p = GamificationStorage::get_profile(env, user);
        p.guild_id = guild_id;
        GamificationStorage::set_profile(env, user, &p);

        // Reputation: collaboration for joining
        crate::reputation::ReputationManager::add_collaboration_points(env, user, 5);

        GamificationEvents::emit_guild_joined(env, user, guild_id);
        Ok(())
    }

    // ── Leave ──────────────────────────────────────────────────────────────

    pub fn leave(env: &Env, user: &Address) -> Result<(), Error> {
        let profile = GamificationStorage::get_profile(env, user);
        if profile.guild_id == 0 {
            return Err(Error::NotInGuild);
        }

        let guild_id = profile.guild_id;

        let mut guild: Guild = env
            .storage()
            .persistent()
            .get(&GamificationKey::Guild(guild_id))
            .ok_or(Error::NotFound)?;

        // Remove member record
        env.storage()
            .persistent()
            .remove(&GamificationKey::GuildMember(user.clone()));

        // Remove from member list
        let ml_key = GamificationKey::GuildMembers(guild_id);
        let old_ml: Vec<Address> = env
            .storage()
            .persistent()
            .get(&ml_key)
            .unwrap_or_else(|| Vec::new(env));
        let mut new_ml = Vec::new(env);
        for addr in old_ml.iter() {
            if &addr != user {
                new_ml.push_back(addr);
            }
        }
        env.storage().persistent().set(&ml_key, &new_ml);

        if guild.member_count > 0 {
            guild.member_count -= 1;
        }
        env.storage()
            .persistent()
            .set(&GamificationKey::Guild(guild_id), &guild);

        // Clear guild from profile
        let mut p = GamificationStorage::get_profile(env, user);
        p.guild_id = 0;
        GamificationStorage::set_profile(env, user, &p);

        crate::leaderboard::LeaderboardManager::update_guild_score(env, &guild);

        GamificationEvents::emit_guild_left(env, user, guild_id);
        Ok(())
    }

    // ── Contribution ───────────────────────────────────────────────────────

    /// Called by AchievementManager / ChallengeManager to add XP to the member's
    /// guild.  Does nothing if the user is not in a guild.
    pub fn add_contribution(env: &Env, user: &Address, xp: u32) {
        if xp == 0 {
            return;
        }

        let member_key = GamificationKey::GuildMember(user.clone());
        let member_opt: Option<GuildMember> = env.storage().persistent().get(&member_key);

        if let Some(mut member) = member_opt {
            member.contribution_xp += xp;
            env.storage().persistent().set(&member_key, &member);

            let guild_id = member.guild_id;
            if let Some(mut guild) = env
                .storage()
                .persistent()
                .get::<GamificationKey, Guild>(&GamificationKey::Guild(guild_id))
            {
                guild.total_xp += xp;
                env.storage()
                    .persistent()
                    .set(&GamificationKey::Guild(guild_id), &guild);

                crate::leaderboard::LeaderboardManager::update_guild_score(env, &guild);
                crate::reputation::ReputationManager::add_collaboration_points(env, user, xp / 20);
            }
        }
    }

    // ── Add season XP to guild ─────────────────────────────────────────────

    pub fn add_season_xp(env: &Env, user: &Address, xp: u32) {
        if xp == 0 {
            return;
        }
        let member_key = GamificationKey::GuildMember(user.clone());
        let member_opt: Option<GuildMember> = env.storage().persistent().get(&member_key);

        if let Some(member) = member_opt {
            let guild_id = member.guild_id;
            if let Some(mut guild) = env
                .storage()
                .persistent()
                .get::<GamificationKey, Guild>(&GamificationKey::Guild(guild_id))
            {
                guild.season_xp += xp;
                env.storage()
                    .persistent()
                    .set(&GamificationKey::Guild(guild_id), &guild);
            }
        }
    }

    // ── Queries ────────────────────────────────────────────────────────────

    pub fn get_guild(env: &Env, guild_id: u64) -> Option<Guild> {
        env.storage()
            .persistent()
            .get(&GamificationKey::Guild(guild_id))
    }

    pub fn get_members(env: &Env, guild_id: u64) -> Vec<GuildMember> {
        let addrs: Vec<Address> = env
            .storage()
            .persistent()
            .get(&GamificationKey::GuildMembers(guild_id))
            .unwrap_or_else(|| Vec::new(env));

        let mut out = Vec::new(env);
        for addr in addrs.iter() {
            if let Some(m) = env
                .storage()
                .persistent()
                .get::<GamificationKey, GuildMember>(&GamificationKey::GuildMember(addr.clone()))
            {
                out.push_back(m);
            }
        }
        out
    }
}
