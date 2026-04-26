use shared::event_schema::{
    AccessControlEventData, AchievementClaimedEvent, AchievementEarnedEvent,
    ChallengeCompletedEvent, ChallengeCreatedEvent, ChallengeJoinedEvent, ContractInitializedEvent,
    EndorsedEvent, GamificationEventData, GuildCreatedEvent, GuildJoinedEvent, GuildLeftEvent,
    LevelUpEvent, RecognizedEvent, ReputationUpdatedEvent, SeasonEndedEvent, SeasonStartedEvent,
    StreakMilestoneEvent, XPAddedEvent,
};
use shared::{emit_access_control_event, emit_gamification_event};
use soroban_sdk::{symbol_short, Address, Env};

pub struct GamificationEvents;

impl GamificationEvents {
    pub fn emit_initialized(env: &Env, admin: &Address) {
        emit_access_control_event!(
            env,
            symbol_short!("gam"),
            admin.clone(),
            AccessControlEventData::ContractInitialized(ContractInitializedEvent {
                admin: admin.clone()
            })
        );
    }

    pub fn emit_xp_earned(env: &Env, user: &Address, xp: u32) {
        emit_gamification_event!(
            env,
            symbol_short!("gam"),
            user.clone(),
            GamificationEventData::XPAdded(XPAddedEvent { user: user.clone(), amount: xp })
        );
    }

    pub fn emit_level_up(env: &Env, user: &Address, new_level: u32) {
        emit_gamification_event!(
            env,
            symbol_short!("gam"),
            user.clone(),
            GamificationEventData::LevelUp(LevelUpEvent { user: user.clone(), new_level })
        );
    }

    pub fn emit_streak_milestone(env: &Env, user: &Address, streak_days: u32) {
        emit_gamification_event!(
            env,
            symbol_short!("gam"),
            user.clone(),
            GamificationEventData::StreakMilestone(StreakMilestoneEvent {
                user: user.clone(),
                streak_days
            })
        );
    }

    pub fn emit_achievement_earned(env: &Env, user: &Address, achievement_id: u64, _xp: u32) {
        emit_gamification_event!(
            env,
            symbol_short!("gam"),
            user.clone(),
            GamificationEventData::AchievementEarned(AchievementEarnedEvent {
                user: user.clone(),
                achievement_id
            })
        );
    }

    pub fn emit_achievement_claimed(env: &Env, user: &Address, achievement_id: u64, tokens: i128) {
        emit_gamification_event!(
            env,
            symbol_short!("gam"),
            user.clone(),
            GamificationEventData::AchievementClaimed(AchievementClaimedEvent {
                user: user.clone(),
                achievement_id,
                tokens
            })
        );
    }

    pub fn emit_challenge_created(env: &Env, challenge_id: u64, creator: &Address) {
        emit_gamification_event!(
            env,
            symbol_short!("gam"),
            creator.clone(),
            GamificationEventData::ChallengeCreated(ChallengeCreatedEvent {
                challenge_id,
                creator: creator.clone()
            })
        );
    }

    pub fn emit_challenge_joined(env: &Env, user: &Address, challenge_id: u64) {
        emit_gamification_event!(
            env,
            symbol_short!("gam"),
            user.clone(),
            GamificationEventData::ChallengeJoined(ChallengeJoinedEvent {
                user: user.clone(),
                challenge_id
            })
        );
    }

    pub fn emit_challenge_completed(env: &Env, user: &Address, challenge_id: u64, rank: u32) {
        emit_gamification_event!(
            env,
            symbol_short!("gam"),
            user.clone(),
            GamificationEventData::ChallengeCompleted(ChallengeCompletedEvent {
                user: user.clone(),
                challenge_id,
                rank
            })
        );
    }

    pub fn emit_guild_created(env: &Env, guild_id: u64, creator: &Address) {
        emit_gamification_event!(
            env,
            symbol_short!("gam"),
            creator.clone(),
            GamificationEventData::GuildCreated(GuildCreatedEvent {
                guild_id,
                creator: creator.clone()
            })
        );
    }

    pub fn emit_guild_joined(env: &Env, user: &Address, guild_id: u64) {
        emit_gamification_event!(
            env,
            symbol_short!("gam"),
            user.clone(),
            GamificationEventData::GuildJoined(GuildJoinedEvent { user: user.clone(), guild_id })
        );
    }

    pub fn emit_guild_left(env: &Env, user: &Address, guild_id: u64) {
        emit_gamification_event!(
            env,
            symbol_short!("gam"),
            user.clone(),
            GamificationEventData::GuildLeft(GuildLeftEvent { user: user.clone(), guild_id })
        );
    }

    pub fn emit_season_started(env: &Env, admin: &Address, season_id: u64) {
        emit_gamification_event!(
            env,
            symbol_short!("gam"),
            admin.clone(),
            GamificationEventData::SeasonStarted(SeasonStartedEvent { season_id })
        );
    }

    pub fn emit_season_ended(env: &Env, admin: &Address, season_id: u64) {
        emit_gamification_event!(
            env,
            symbol_short!("gam"),
            admin.clone(),
            GamificationEventData::SeasonEnded(SeasonEndedEvent { season_id })
        );
    }

    pub fn emit_endorsed(env: &Env, endorser: &Address, endorsee: &Address) {
        emit_gamification_event!(
            env,
            symbol_short!("gam"),
            endorser.clone(),
            GamificationEventData::Endorsed(EndorsedEvent {
                endorser: endorser.clone(),
                endorsee: endorsee.clone()
            })
        );
    }

    pub fn emit_recognized(env: &Env, from: &Address, to: &Address) {
        emit_gamification_event!(
            env,
            symbol_short!("gam"),
            from.clone(),
            GamificationEventData::Recognized(RecognizedEvent {
                from: from.clone(),
                to: to.clone()
            })
        );
    }

    pub fn emit_reputation_updated(env: &Env, user: &Address, new_score: u32) {
        emit_gamification_event!(
            env,
            symbol_short!("gam"),
            user.clone(),
            GamificationEventData::ReputationUpdated(ReputationUpdatedEvent {
                user: user.clone(),
                new_score
            })
        );
    }
}
