use soroban_sdk::{Address, Env, Vec};

use crate::errors::Error;
use crate::events::GamificationEvents;
use crate::storage::GamificationStorage;
use crate::types::{Challenge, GamificationKey, UserChallenge};

pub struct ChallengeManager;

impl ChallengeManager {
    // ── Create ─────────────────────────────────────────────────────────────

    pub fn create(env: &Env, creator: &Address, mut challenge: Challenge) -> Result<u64, Error> {
        if challenge.target_progress == 0 {
            return Err(Error::InvalidInput);
        }
        if challenge.end_time <= challenge.start_time {
            return Err(Error::InvalidInput);
        }

        let id = GamificationStorage::next_id(env, &GamificationKey::ChallengeCounter);
        challenge.id = id;
        challenge.created_by = creator.clone();
        challenge.created_at = env.ledger().timestamp();
        challenge.current_participants = 0;
        challenge.is_active = true;

        env.storage()
            .persistent()
            .set(&GamificationKey::Challenge(id), &challenge);

        // Add to active-challenges list
        let active_key = GamificationKey::ActiveChallenges;
        let mut active: Vec<u64> = env
            .storage()
            .persistent()
            .get(&active_key)
            .unwrap_or_else(|| Vec::new(env));
        active.push_back(id);
        env.storage().persistent().set(&active_key, &active);

        env.storage()
            .persistent()
            .set(&GamificationKey::ChallengeCompletionCount(id), &0u32);

        GamificationEvents::emit_challenge_created(env, id, creator);
        Ok(id)
    }

    // ── Join ───────────────────────────────────────────────────────────────

    pub fn join(env: &Env, user: &Address, challenge_id: u64) -> Result<(), Error> {
        let mut challenge: Challenge = env
            .storage()
            .persistent()
            .get(&GamificationKey::Challenge(challenge_id))
            .ok_or(Error::NotFound)?;

        if !challenge.is_active {
            return Err(Error::ChallengeInactive);
        }

        let now = env.ledger().timestamp();
        if now < challenge.start_time {
            return Err(Error::ChallengeNotStarted);
        }
        if now > challenge.end_time {
            return Err(Error::ChallengeExpired);
        }
        if challenge.max_participants > 0
            && challenge.current_participants >= challenge.max_participants
        {
            return Err(Error::ChallengeFull);
        }

        // Prerequisite quest chain check
        if challenge.prerequisite_challenge_id > 0 {
            let pre_key = GamificationKey::UserChallenge(
                user.clone(),
                challenge.prerequisite_challenge_id,
            );
            let pre: Option<UserChallenge> = env.storage().persistent().get(&pre_key);
            match pre {
                Some(uc) if uc.completed => {}
                _ => return Err(Error::PrerequisiteNotMet),
            }
        }

        // Already joined?
        let uc_key = GamificationKey::UserChallenge(user.clone(), challenge_id);
        if env.storage().persistent().has(&uc_key) {
            return Err(Error::AlreadyJoinedChallenge);
        }

        let uc = UserChallenge {
            user: user.clone(),
            challenge_id,
            joined_at: now,
            current_progress: 0,
            completed: false,
            completed_at: 0,
            reward_claimed: false,
            rank: 0,
        };
        env.storage().persistent().set(&uc_key, &uc);

        // Update user's active challenge list
        let ua_key = GamificationKey::UserActiveChallenges(user.clone());
        let mut ua: Vec<u64> = env
            .storage()
            .persistent()
            .get(&ua_key)
            .unwrap_or_else(|| Vec::new(env));
        ua.push_back(challenge_id);
        env.storage().persistent().set(&ua_key, &ua);

        // Bump participant count
        challenge.current_participants += 1;
        env.storage()
            .persistent()
            .set(&GamificationKey::Challenge(challenge_id), &challenge);

        GamificationEvents::emit_challenge_joined(env, user, challenge_id);
        Ok(())
    }

    // ── Update progress ────────────────────────────────────────────────────

    /// Returns `true` if the challenge was completed in this call.
    pub fn update_progress(
        env: &Env,
        user: &Address,
        challenge_id: u64,
        progress: u32,
    ) -> Result<bool, Error> {
        let challenge: Challenge = env
            .storage()
            .persistent()
            .get(&GamificationKey::Challenge(challenge_id))
            .ok_or(Error::NotFound)?;

        if !challenge.is_active {
            return Err(Error::ChallengeInactive);
        }

        let now = env.ledger().timestamp();
        if now > challenge.end_time {
            return Err(Error::ChallengeExpired);
        }

        let uc_key = GamificationKey::UserChallenge(user.clone(), challenge_id);
        let mut uc: UserChallenge = env
            .storage()
            .persistent()
            .get(&uc_key)
            .ok_or(Error::NotJoinedChallenge)?;

        if uc.completed {
            return Ok(true); // already done
        }

        uc.current_progress = progress.min(challenge.target_progress);
        let completed = uc.current_progress >= challenge.target_progress;

        if completed {
            uc.completed = true;
            uc.completed_at = now;

            // Rank = position among completions + 1
            let count_key = GamificationKey::ChallengeCompletionCount(challenge_id);
            let count: u32 = env
                .storage()
                .persistent()
                .get(&count_key)
                .unwrap_or(0u32);
            let new_count = count + 1;
            uc.rank = new_count;
            env.storage().persistent().set(&count_key, &new_count);

            // Award XP to user profile
            let mut profile = GamificationStorage::get_profile(env, user);
            profile.total_xp += challenge.xp_reward;
            profile.challenges_completed += 1;
            let new_level = crate::achievements::AchievementManager::calculate_level(profile.total_xp);
            let leveled_up = new_level > profile.level;
            profile.level = new_level;
            GamificationStorage::set_profile(env, user, &profile);

            if leveled_up {
                GamificationEvents::emit_level_up(env, user, profile.level);
            }
            GamificationEvents::emit_xp_earned(env, user, challenge.xp_reward);

            // Guild contribution
            crate::guilds::GuildManager::add_contribution(env, user, challenge.xp_reward);

            // Reputation: innovation points
            crate::reputation::ReputationManager::add_innovation_points(
                env,
                user,
                challenge.xp_reward / 10,
            );

            // Update leaderboards
            let updated = GamificationStorage::get_profile(env, user);
            crate::leaderboard::LeaderboardManager::update_user_score(env, &updated);

            // Achievement check
            crate::achievements::AchievementManager::check_and_award_achievements(env, user, &updated);

            // Adaptive difficulty update (success)
            crate::achievements::AchievementManager::update_adaptive_difficulty(env, user, true, 100);

            GamificationEvents::emit_challenge_completed(env, user, challenge_id, uc.rank);
        }

        env.storage().persistent().set(&uc_key, &uc);
        Ok(completed)
    }

    // ── Queries ────────────────────────────────────────────────────────────

    pub fn get_active_challenges(env: &Env) -> Vec<Challenge> {
        let ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&GamificationKey::ActiveChallenges)
            .unwrap_or_else(|| Vec::new(env));

        let now = env.ledger().timestamp();
        let mut out = Vec::new(env);
        for id in ids.iter() {
            if let Some(c) = env
                .storage()
                .persistent()
                .get::<GamificationKey, Challenge>(&GamificationKey::Challenge(id))
            {
                if c.is_active && now >= c.start_time && now <= c.end_time {
                    out.push_back(c);
                }
            }
        }
        out
    }

    pub fn get_user_challenge(
        env: &Env,
        user: &Address,
        challenge_id: u64,
    ) -> Option<UserChallenge> {
        env.storage()
            .persistent()
            .get(&GamificationKey::UserChallenge(user.clone(), challenge_id))
    }

    pub fn get_user_active_challenges(env: &Env, user: &Address) -> Vec<u64> {
        env.storage()
            .persistent()
            .get(&GamificationKey::UserActiveChallenges(user.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }
}
