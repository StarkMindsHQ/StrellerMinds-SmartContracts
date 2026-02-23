use soroban_sdk::{Address, Env};

use crate::types::*;

pub struct AnalyticsManager;

impl AnalyticsManager {
    pub fn get_community_metrics(env: &Env) -> CommunityMetrics {
        env.storage()
            .persistent()
            .get(&CommunityKey::CommunityMetrics)
            .unwrap_or(CommunityMetrics {
                total_posts: 0,
                total_replies: 0,
                total_contributions: 0,
                total_events: 0,
                active_mentorships: 0,
                total_members: 0,
                daily_active_users: 0,
                weekly_active_users: 0,
                avg_response_time: 0,
                resolution_rate: 0,
                last_updated: env.ledger().timestamp(),
            })
    }

    pub fn update_metrics(env: &Env) {
        let mut metrics = Self::get_community_metrics(env);

        // Update counters from storage
        metrics.total_posts = env
            .storage()
            .persistent()
            .get(&CommunityKey::PostCounter)
            .unwrap_or(0);

        metrics.total_replies = env
            .storage()
            .persistent()
            .get(&CommunityKey::ReplyCounter)
            .unwrap_or(0);

        metrics.total_contributions = env
            .storage()
            .persistent()
            .get(&CommunityKey::ContributionCounter)
            .unwrap_or(0);

        metrics.total_events = env
            .storage()
            .persistent()
            .get(&CommunityKey::EventCounter)
            .unwrap_or(0);

        metrics.last_updated = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&CommunityKey::CommunityMetrics, &metrics);
    }

    pub fn get_user_stats(env: &Env, user: &Address) -> UserCommunityStats {
        env.storage()
            .persistent()
            .get(&CommunityKey::UserStats(user.clone()))
            .unwrap_or(UserCommunityStats {
                user: user.clone(),
                posts_created: 0,
                replies_given: 0,
                solutions_provided: 0,
                contributions_made: 0,
                events_attended: 0,
                mentorship_sessions: 0,
                helpful_votes_received: 0,
                reputation_score: 0,
                joined_at: env.ledger().timestamp(),
            })
    }

    pub fn calculate_reputation(env: &Env, user: &Address) -> u32 {
        let stats = Self::get_user_stats(env, user);

        // Weighted reputation calculation
        let reputation = stats.posts_created * 10
            + stats.replies_given * 5
            + stats.solutions_provided * 50
            + stats.contributions_made * 100
            + stats.events_attended * 25
            + stats.mentorship_sessions * 75
            + stats.helpful_votes_received * 15;

        // Update stored reputation
        let mut updated_stats = stats;
        updated_stats.reputation_score = reputation;
        env.storage()
            .persistent()
            .set(&CommunityKey::UserStats(user.clone()), &updated_stats);

        reputation
    }
}
