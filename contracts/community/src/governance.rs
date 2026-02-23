use soroban_sdk::{Address, Env, String, Vec};

use crate::errors::Error;
use crate::events::CommunityEvents;
use crate::storage::CommunityStorage;
use crate::types::*;

pub struct GovernanceManager;

impl GovernanceManager {
    pub fn create_proposal(
        env: &Env,
        proposer: &Address,
        proposal_type: ProposalType,
        title: String,
        description: String,
        voting_duration: u64,
        min_votes_required: u32,
    ) -> Result<u64, Error> {
        // Check if user has sufficient reputation
        let stats: UserCommunityStats = env
            .storage()
            .persistent()
            .get(&CommunityKey::UserStats(proposer.clone()))
            .ok_or(Error::InsufficientReputation)?;

        let config = CommunityStorage::get_config(env);
        if stats.reputation_score < config.vote_weight_threshold {
            return Err(Error::InsufficientVotingPower);
        }

        let proposal_id = CommunityStorage::increment_counter(env, CommunityKey::ProposalCounter);
        let now = env.ledger().timestamp();

        let proposal = CommunityProposal {
            id: proposal_id,
            proposer: proposer.clone(),
            proposal_type,
            title,
            description,
            status: ProposalStatus::Active,
            votes_for: 0,
            votes_against: 0,
            created_at: now,
            voting_ends_at: now + voting_duration,
            min_votes_required,
        };

        env.storage()
            .persistent()
            .set(&CommunityKey::Proposal(proposal_id), &proposal);

        // Add to active proposals
        let mut active: Vec<u64> = env
            .storage()
            .persistent()
            .get(&CommunityKey::ActiveProposals)
            .unwrap_or_else(|| Vec::new(env));
        active.push_back(proposal_id);
        env.storage()
            .persistent()
            .set(&CommunityKey::ActiveProposals, &active);

        CommunityEvents::emit_proposal_created(env, proposer, proposal_id);
        Ok(proposal_id)
    }

    pub fn vote_on_proposal(
        env: &Env,
        voter: &Address,
        proposal_id: u64,
        vote_for: bool,
    ) -> Result<(), Error> {
        let mut proposal: CommunityProposal = env
            .storage()
            .persistent()
            .get(&CommunityKey::Proposal(proposal_id))
            .ok_or(Error::ProposalNotFound)?;

        let now = env.ledger().timestamp();
        if now > proposal.voting_ends_at {
            return Err(Error::VotingClosed);
        }

        if proposal.status != ProposalStatus::Active {
            return Err(Error::VotingClosed);
        }

        // Check if already voted
        let vote_key = CommunityKey::ProposalVote(voter.clone(), proposal_id);
        if env.storage().persistent().has(&vote_key) {
            return Err(Error::AlreadyVotedOnProposal);
        }

        // Calculate vote weight based on reputation
        let stats: UserCommunityStats = env
            .storage()
            .persistent()
            .get(&CommunityKey::UserStats(voter.clone()))
            .unwrap_or(UserCommunityStats {
                user: voter.clone(),
                posts_created: 0,
                replies_given: 0,
                solutions_provided: 0,
                contributions_made: 0,
                events_attended: 0,
                mentorship_sessions: 0,
                helpful_votes_received: 0,
                reputation_score: 0,
                joined_at: now,
            });

        let config = CommunityStorage::get_config(env);
        if stats.reputation_score < config.vote_weight_threshold {
            return Err(Error::InsufficientVotingPower);
        }

        // Weight votes by reputation (simplified: 1 vote per 100 reputation)
        let vote_weight = (stats.reputation_score / 100).max(1);

        if vote_for {
            proposal.votes_for += vote_weight;
        } else {
            proposal.votes_against += vote_weight;
        }

        env.storage()
            .persistent()
            .set(&CommunityKey::Proposal(proposal_id), &proposal);
        env.storage().persistent().set(&vote_key, &vote_for);

        CommunityEvents::emit_vote_cast(env, voter, proposal_id, vote_for);
        Ok(())
    }

    pub fn finalize_proposal(
        env: &Env,
        proposal_id: u64,
    ) -> Result<ProposalStatus, Error> {
        let mut proposal: CommunityProposal = env
            .storage()
            .persistent()
            .get(&CommunityKey::Proposal(proposal_id))
            .ok_or(Error::ProposalNotFound)?;

        let now = env.ledger().timestamp();
        if now <= proposal.voting_ends_at {
            return Err(Error::InvalidInput);
        }

        if proposal.status != ProposalStatus::Active {
            return Err(Error::InvalidInput);
        }

        let total_votes = proposal.votes_for + proposal.votes_against;
        
        if total_votes < proposal.min_votes_required {
            proposal.status = ProposalStatus::Rejected;
        } else if proposal.votes_for > proposal.votes_against {
            proposal.status = ProposalStatus::Passed;
        } else {
            proposal.status = ProposalStatus::Rejected;
        }

        env.storage()
            .persistent()
            .set(&CommunityKey::Proposal(proposal_id), &proposal);

        // Remove from active proposals
        let active: Vec<u64> = env
            .storage()
            .persistent()
            .get(&CommunityKey::ActiveProposals)
            .unwrap_or_else(|| Vec::new(env));
        
        let mut new_active = Vec::new(env);
        for id in active.iter() {
            if id != proposal_id {
                new_active.push_back(id);
            }
        }
        env.storage()
            .persistent()
            .set(&CommunityKey::ActiveProposals, &new_active);

        Ok(proposal.status)
    }

    pub fn get_proposal(env: &Env, proposal_id: u64) -> Option<CommunityProposal> {
        env.storage()
            .persistent()
            .get(&CommunityKey::Proposal(proposal_id))
    }

    pub fn get_active_proposals(env: &Env) -> Vec<CommunityProposal> {
        let proposal_ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&CommunityKey::ActiveProposals)
            .unwrap_or_else(|| Vec::new(env));

        let mut proposals = Vec::new(env);
        for id in proposal_ids.iter() {
            if let Some(proposal) = env.storage().persistent().get(&CommunityKey::Proposal(id)) {
                proposals.push_back(proposal);
            }
        }
        proposals
    }
}
