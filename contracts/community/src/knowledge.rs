use soroban_sdk::{Address, Env, String, Vec};

use crate::errors::Error;
use crate::events::CommunityEvents;
use crate::storage::CommunityStorage;
use crate::types::*;

pub struct KnowledgeManager;

impl KnowledgeManager {
    pub fn submit_contribution(
        env: &Env,
        contributor: &Address,
        contribution_type: ContributionType,
        title: String,
        content: String,
        category: ForumCategory,
        tags: Vec<String>,
    ) -> Result<u64, Error> {
        let contribution_id =
            CommunityStorage::increment_counter(env, CommunityKey::ContributionCounter);
        let now = env.ledger().timestamp();

        let contribution = KnowledgeContribution {
            id: contribution_id,
            contributor: contributor.clone(),
            contribution_type,
            title,
            content,
            status: ContributionStatus::Submitted,
            category: category.clone(),
            tags,
            upvotes: 0,
            views: 0,
            created_at: now,
            published_at: 0,
            xp_reward: 0,
            token_reward: 0,
        };

        env.storage()
            .persistent()
            .set(&CommunityKey::Contribution(contribution_id), &contribution);

        // Add to user contributions
        let mut user_contribs: Vec<u64> = env
            .storage()
            .persistent()
            .get(&CommunityKey::UserContributions(contributor.clone()))
            .unwrap_or_else(|| Vec::new(env));
        user_contribs.push_back(contribution_id);
        env.storage()
            .persistent()
            .set(
                &CommunityKey::UserContributions(contributor.clone()),
                &user_contribs,
            );

        CommunityEvents::emit_contribution_submitted(env, contributor, contribution_id);
        Ok(contribution_id)
    }

    pub fn review_contribution(
        env: &Env,
        moderator: &Address,
        contribution_id: u64,
        approve: bool,
    ) -> Result<(), Error> {
        CommunityStorage::require_moderator(env, moderator)?;

        let mut contribution: KnowledgeContribution = env
            .storage()
            .persistent()
            .get(&CommunityKey::Contribution(contribution_id))
            .ok_or(Error::ContributionNotFound)?;

        if contribution.status != ContributionStatus::Submitted
            && contribution.status != ContributionStatus::UnderReview
        {
            return Err(Error::InvalidContributionStatus);
        }

        let config = CommunityStorage::get_config(env);

        if approve {
            contribution.status = ContributionStatus::Approved;
            contribution.published_at = env.ledger().timestamp();
            
            // Calculate rewards based on contribution type
            let (xp, tokens) = Self::calculate_rewards(&contribution.contribution_type, &config);
            contribution.xp_reward = xp;
            contribution.token_reward = tokens;

            // Add to category index
            let mut category_contribs: Vec<u64> = env
                .storage()
                .persistent()
                .get(&CommunityKey::CategoryContributions(
                    contribution.category.clone(),
                ))
                .unwrap_or_else(|| Vec::new(env));
            category_contribs.push_back(contribution_id);
            env.storage()
                .persistent()
                .set(
                    &CommunityKey::CategoryContributions(contribution.category.clone()),
                    &category_contribs,
                );

            // Update user stats
            Self::update_contributor_stats(env, &contribution.contributor);

            // Award rewards
            Self::award_xp(env, &contribution.contributor, xp);
            Self::award_tokens(env, &contribution.contributor, tokens);

            CommunityEvents::emit_contribution_approved(env, contribution_id);
        } else {
            contribution.status = ContributionStatus::Rejected;
        }

        env.storage()
            .persistent()
            .set(&CommunityKey::Contribution(contribution_id), &contribution);

        Ok(())
    }

    pub fn vote_contribution(
        env: &Env,
        _voter: &Address,
        contribution_id: u64,
        upvote: bool,
    ) -> Result<(), Error> {
        let mut contribution: KnowledgeContribution = env
            .storage()
            .persistent()
            .get(&CommunityKey::Contribution(contribution_id))
            .ok_or(Error::ContributionNotFound)?;

        if contribution.status != ContributionStatus::Approved
            && contribution.status != ContributionStatus::Published
        {
            return Err(Error::InvalidContributionStatus);
        }

        if upvote {
            contribution.upvotes += 1;
            
            // Update contributor's helpful votes
            let contributor_addr = contribution.contributor.clone();
            let mut stats: UserCommunityStats = env
                .storage()
                .persistent()
                .get(&CommunityKey::UserStats(contributor_addr.clone()))
                .unwrap();
            stats.helpful_votes_received += 1;
            env.storage()
                .persistent()
                .set(&CommunityKey::UserStats(contributor_addr), &stats);
        }

        env.storage()
            .persistent()
            .set(&CommunityKey::Contribution(contribution_id), &contribution);

        Ok(())
    }

    pub fn get_contribution(env: &Env, contribution_id: u64) -> Option<KnowledgeContribution> {
        let mut contribution: Option<KnowledgeContribution> = env
            .storage()
            .persistent()
            .get(&CommunityKey::Contribution(contribution_id));

        if let Some(ref mut c) = contribution {
            c.views += 1;
            env.storage()
                .persistent()
                .set(&CommunityKey::Contribution(contribution_id), c);
        }

        contribution
    }

    pub fn get_category_contributions(
        env: &Env,
        category: ForumCategory,
        limit: u32,
    ) -> Vec<KnowledgeContribution> {
        let contrib_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&CommunityKey::CategoryContributions(category))
            .unwrap_or_else(|| Vec::new(env));

        let mut contributions = Vec::new(env);
        let max = limit.min(contrib_ids.len());
        for i in 0..max {
            if let Some(id) = contrib_ids.get(i) {
                if let Some(contrib) = env
                    .storage()
                    .persistent()
                    .get(&CommunityKey::Contribution(id))
                {
                    contributions.push_back(contrib);
                }
            }
        }
        contributions
    }

    pub fn get_user_contributions(env: &Env, user: &Address) -> Vec<KnowledgeContribution> {
        let contrib_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&CommunityKey::UserContributions(user.clone()))
            .unwrap_or_else(|| Vec::new(env));

        let mut contributions = Vec::new(env);
        for id in contrib_ids.iter() {
            if let Some(contrib) = env
                .storage()
                .persistent()
                .get(&CommunityKey::Contribution(id))
            {
                contributions.push_back(contrib);
            }
        }
        contributions
    }

    // Helper functions
    fn calculate_rewards(
        contribution_type: &ContributionType,
        config: &CommunityConfig,
    ) -> (u32, i128) {
        let multiplier = match contribution_type {
            ContributionType::Article => 3,
            ContributionType::Tutorial => 4,
            ContributionType::CodeSnippet => 2,
            ContributionType::Resource => 2,
            ContributionType::FAQ => 1,
        };

        (
            config.contribution_base_xp * multiplier,
            config.contribution_base_tokens * multiplier as i128,
        )
    }

    fn update_contributor_stats(env: &Env, contributor: &Address) {
        let mut stats: UserCommunityStats = env
            .storage()
            .persistent()
            .get(&CommunityKey::UserStats(contributor.clone()))
            .unwrap_or(UserCommunityStats {
                user: contributor.clone(),
                posts_created: 0,
                replies_given: 0,
                solutions_provided: 0,
                contributions_made: 0,
                events_attended: 0,
                mentorship_sessions: 0,
                helpful_votes_received: 0,
                reputation_score: 0,
                joined_at: env.ledger().timestamp(),
            });

        stats.contributions_made += 1;
        env.storage()
            .persistent()
            .set(&CommunityKey::UserStats(contributor.clone()), &stats);
    }

    fn award_xp(_env: &Env, _user: &Address, _xp: u32) {
        // Integration point with gamification contract
    }

    fn award_tokens(_env: &Env, _user: &Address, _tokens: i128) {
        // Integration point with token contract
    }
}
