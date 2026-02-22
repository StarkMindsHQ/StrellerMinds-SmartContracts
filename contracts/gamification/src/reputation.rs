use soroban_sdk::Env;

use crate::events::GamificationEvents;
use crate::storage::GamificationStorage;
use crate::types::{
    ActivityRecord, ActivityType, GamificationKey, ReputationScore, ReputationTier,
};

pub struct ReputationManager;

impl ReputationManager {
    // ── Public API ─────────────────────────────────────────────────────────

    pub fn get_reputation(env: &Env, user: &soroban_sdk::Address) -> ReputationScore {
        env.storage()
            .persistent()
            .get(&GamificationKey::UserReputation(user.clone()))
            .unwrap_or_else(|| ReputationScore {
                user: user.clone(),
                total_score: 0,
                teaching_points: 0,
                quality_points: 0,
                consistency_points: 0,
                collaboration_points: 0,
                innovation_points: 0,
                tier: ReputationTier::Novice,
                last_updated: env.ledger().timestamp(),
            })
    }

    /// Called after any learning activity to update consistency/quality points.
    pub fn update_from_activity(env: &Env, user: &soroban_sdk::Address, activity: &ActivityRecord) {
        let mut rep = Self::get_reputation(env, user);
        let now = env.ledger().timestamp();

        match activity.activity_type {
            ActivityType::ModuleCompleted | ActivityType::CourseCompleted => {
                // Consistency: credit for completing content
                rep.consistency_points += 5;
                // Quality: score-weighted bonus
                if activity.score >= 90 {
                    rep.quality_points += 10;
                } else if activity.score >= 75 {
                    rep.quality_points += 5;
                }
            }
            ActivityType::AssessmentPassed => {
                if activity.score >= 90 {
                    rep.quality_points += 15;
                } else if activity.score >= 75 {
                    rep.quality_points += 7;
                } else {
                    rep.quality_points += 3;
                }
            }
            ActivityType::StudySession => {
                // Consistency credit for study time
                let hours = (activity.time_spent / 3600) as u32;
                rep.consistency_points += hours.min(5);
            }
            ActivityType::PeerHelped => {
                rep.teaching_points += 10;
            }
            ActivityType::ChallengeProgress => {
                rep.innovation_points += 2;
            }
        }

        Self::recalculate_and_save(env, user, &mut rep, now);
    }

    /// Award teaching points when a user endorses another (endorsee gets teaching credit).
    pub fn add_teaching_points(env: &Env, user: &soroban_sdk::Address, points: u32) {
        let mut rep = Self::get_reputation(env, user);
        let now = env.ledger().timestamp();
        rep.teaching_points += points;
        Self::recalculate_and_save(env, user, &mut rep, now);
    }

    /// Award collaboration points (e.g. guild activity).
    pub fn add_collaboration_points(env: &Env, user: &soroban_sdk::Address, points: u32) {
        let mut rep = Self::get_reputation(env, user);
        let now = env.ledger().timestamp();
        rep.collaboration_points += points;
        Self::recalculate_and_save(env, user, &mut rep, now);
    }

    /// Award innovation points (e.g. challenge completion).
    pub fn add_innovation_points(env: &Env, user: &soroban_sdk::Address, points: u32) {
        let mut rep = Self::get_reputation(env, user);
        let now = env.ledger().timestamp();
        rep.innovation_points += points;
        Self::recalculate_and_save(env, user, &mut rep, now);
    }

    // ── Internals ──────────────────────────────────────────────────────────

    fn recalculate_and_save(
        env: &Env,
        user: &soroban_sdk::Address,
        rep: &mut ReputationScore,
        now: u64,
    ) {
        rep.total_score = rep.teaching_points
            + rep.quality_points
            + rep.consistency_points
            + rep.collaboration_points
            + rep.innovation_points;

        rep.tier = Self::tier_for_score(rep.total_score);
        rep.last_updated = now;

        // Mirror reputation score back to profile
        let mut profile = GamificationStorage::get_profile(env, user);
        profile.reputation_score = rep.total_score;
        GamificationStorage::set_profile(env, user, &profile);

        env.storage()
            .persistent()
            .set(&GamificationKey::UserReputation(user.clone()), rep);

        GamificationEvents::emit_reputation_updated(env, user, rep.total_score);
    }

    fn tier_for_score(score: u32) -> ReputationTier {
        match score {
            0..=99 => ReputationTier::Novice,
            100..=499 => ReputationTier::Apprentice,
            500..=1999 => ReputationTier::Practitioner,
            2000..=4999 => ReputationTier::Expert,
            5000..=9999 => ReputationTier::Master,
            _ => ReputationTier::Grandmaster,
        }
    }
}
