use soroban_sdk::{symbol_short, Address, Env};

pub struct GamificationEvents;

impl GamificationEvents {
    pub fn emit_initialized(env: &Env, admin: &Address) {
        env.events()
            .publish((symbol_short!("gam_init"),), (admin.clone(),));
    }

    pub fn emit_xp_earned(env: &Env, user: &Address, xp: u32) {
        env.events()
            .publish((symbol_short!("xp_earn"),), (user.clone(), xp));
    }

    pub fn emit_level_up(env: &Env, user: &Address, new_level: u32) {
        env.events()
            .publish((symbol_short!("lvl_up"),), (user.clone(), new_level));
    }

    pub fn emit_streak_milestone(env: &Env, user: &Address, streak_days: u32) {
        env.events()
            .publish((symbol_short!("streak"),), (user.clone(), streak_days));
    }

    pub fn emit_achievement_earned(env: &Env, user: &Address, achievement_id: u64, xp: u32) {
        env.events().publish(
            (symbol_short!("ach_earn"),),
            (user.clone(), achievement_id, xp),
        );
    }

    pub fn emit_achievement_claimed(env: &Env, user: &Address, achievement_id: u64, tokens: i128) {
        env.events().publish(
            (symbol_short!("ach_clm"),),
            (user.clone(), achievement_id, tokens),
        );
    }

    pub fn emit_challenge_created(env: &Env, challenge_id: u64, creator: &Address) {
        env.events().publish(
            (symbol_short!("chl_crt"),),
            (challenge_id, creator.clone()),
        );
    }

    pub fn emit_challenge_joined(env: &Env, user: &Address, challenge_id: u64) {
        env.events()
            .publish((symbol_short!("chl_join"),), (user.clone(), challenge_id));
    }

    pub fn emit_challenge_completed(env: &Env, user: &Address, challenge_id: u64, rank: u32) {
        env.events().publish(
            (symbol_short!("chl_done"),),
            (user.clone(), challenge_id, rank),
        );
    }

    pub fn emit_guild_created(env: &Env, guild_id: u64, creator: &Address) {
        env.events().publish(
            (symbol_short!("gld_crt"),),
            (guild_id, creator.clone()),
        );
    }

    pub fn emit_guild_joined(env: &Env, user: &Address, guild_id: u64) {
        env.events()
            .publish((symbol_short!("gld_join"),), (user.clone(), guild_id));
    }

    pub fn emit_guild_left(env: &Env, user: &Address, guild_id: u64) {
        env.events()
            .publish((symbol_short!("gld_left"),), (user.clone(), guild_id));
    }

    pub fn emit_season_started(env: &Env, season_id: u64) {
        env.events()
            .publish((symbol_short!("sea_strt"),), (season_id,));
    }

    pub fn emit_season_ended(env: &Env, season_id: u64) {
        env.events()
            .publish((symbol_short!("sea_end"),), (season_id,));
    }

    pub fn emit_endorsed(env: &Env, endorser: &Address, endorsee: &Address) {
        env.events().publish(
            (symbol_short!("endorsed"),),
            (endorser.clone(), endorsee.clone()),
        );
    }

    pub fn emit_recognized(env: &Env, from: &Address, to: &Address) {
        env.events()
            .publish((symbol_short!("recog"),), (from.clone(), to.clone()));
    }

    pub fn emit_reputation_updated(env: &Env, user: &Address, new_score: u32) {
        env.events().publish(
            (symbol_short!("rep_upd"),),
            (user.clone(), new_score),
        );
    }
}
