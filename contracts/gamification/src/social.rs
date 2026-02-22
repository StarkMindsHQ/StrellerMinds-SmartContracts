use soroban_sdk::{Address, Env, String, Vec};

use crate::errors::Error;
use crate::events::GamificationEvents;
use crate::storage::GamificationStorage;
use crate::types::{GamificationKey, PeerEndorsement, PeerRecognition, RecognitionType};

pub struct SocialManager;

impl SocialManager {
    // ── Peer Endorsement ───────────────────────────────────────────────────

    pub fn endorse(
        env: &Env,
        endorser: &Address,
        endorsee: &Address,
        skill: String,
    ) -> Result<(), Error> {
        if endorser == endorsee {
            return Err(Error::SelfEndorsement);
        }

        let config = GamificationStorage::get_config(env);

        // Rate-limit: max N endorsements given per day (day = 86400s bucket)
        let now = env.ledger().timestamp();
        let day_bucket = now / 86_400;
        let rate_key = GamificationKey::EndorserDailyCount(endorser.clone(), day_bucket);
        let given_today: u32 = env
            .storage()
            .persistent()
            .get(&rate_key)
            .unwrap_or(0u32);
        if given_today >= config.max_endorsements_per_day {
            return Err(Error::EndorsementLimitReached);
        }
        env.storage()
            .persistent()
            .set(&rate_key, &(given_today + 1));

        // Record endorsement
        let endorsement = PeerEndorsement {
            endorser: endorser.clone(),
            endorsee: endorsee.clone(),
            skill,
            created_at: now,
            xp_value: config.endorsement_xp,
        };

        let key = GamificationKey::UserEndorsements(endorsee.clone());
        let mut list: Vec<PeerEndorsement> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or_else(|| Vec::new(env));
        list.push_back(endorsement);
        env.storage().persistent().set(&key, &list);

        // Update endorsee profile counters
        let mut endorsee_profile = GamificationStorage::get_profile(env, endorsee);
        endorsee_profile.endorsements_received += 1;
        endorsee_profile.total_xp += config.endorsement_xp;
        endorsee_profile.level =
            crate::achievements::AchievementManager::calculate_level(endorsee_profile.total_xp);
        GamificationStorage::set_profile(env, endorsee, &endorsee_profile);

        // Update endorser profile counter
        let mut endorser_profile = GamificationStorage::get_profile(env, endorser);
        endorser_profile.endorsements_given += 1;
        GamificationStorage::set_profile(env, endorser, &endorser_profile);

        // Reputation: endorsee gets teaching/social points; endorser gets teaching credit
        crate::reputation::ReputationManager::add_teaching_points(env, endorsee, 5);
        crate::reputation::ReputationManager::add_teaching_points(env, endorser, 2);

        // Leaderboard update for endorsee (endorsements_received changed)
        let ep = GamificationStorage::get_profile(env, endorsee);
        crate::leaderboard::LeaderboardManager::update_user_score(env, &ep);

        // Achievement check for endorsee
        crate::achievements::AchievementManager::check_and_award_achievements(
            env,
            endorsee,
            &ep,
        );

        GamificationEvents::emit_endorsed(env, endorser, endorsee);
        Ok(())
    }

    // ── Peer Recognition ───────────────────────────────────────────────────

    pub fn recognize(
        env: &Env,
        from: &Address,
        to: &Address,
        recognition_type: RecognitionType,
        message: String,
    ) -> Result<(), Error> {
        if from == to {
            return Err(Error::SelfEndorsement);
        }

        let now = env.ledger().timestamp();
        let _recognition = PeerRecognition {
            from: from.clone(),
            to: to.clone(),
            message,
            recognition_type,
            created_at: now,
        };

        // Recognition gives teaching points to sender and boosts recipient's reputation
        crate::reputation::ReputationManager::add_teaching_points(env, from, 3);
        crate::reputation::ReputationManager::add_teaching_points(env, to, 5);

        GamificationEvents::emit_recognized(env, from, to);
        Ok(())
    }

    // ── Queries ────────────────────────────────────────────────────────────

    pub fn get_endorsements(env: &Env, endorsee: &Address) -> Vec<PeerEndorsement> {
        env.storage()
            .persistent()
            .get(&GamificationKey::UserEndorsements(endorsee.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }
}
