use crate::storage::Storage;
use crate::types::*;
use soroban_sdk::{Address, Env, String};

pub struct ContributionManager;

impl ContributionManager {
    pub fn submit_contribution(
        env: &Env,
        contribution_id: String,
        contributor: &Address,
        doc_id: String,
        contribution_type: ContributionType,
        content: String,
    ) -> Result<Contribution, Error> {
        if env.storage().persistent().has(&DataKey::Contribution(contribution_id.clone())) {
            return Err(Error::AlreadyExists);
        }

        let contribution = Contribution {
            contribution_id: contribution_id.clone(),
            contributor: contributor.clone(),
            doc_id,
            contribution_type,
            content,
            status: ContributionStatus::Pending,
            created_at: env.ledger().timestamp(),
            reviewed_by: None,
            review_notes: None,
        };

        env.storage()
            .persistent()
            .set(&DataKey::Contribution(contribution_id.clone()), &contribution);
        Storage::add_to_user_contributions(env, contributor, &contribution_id);
        Storage::increment_counter(env, &DataKey::TotalContributions);

        Ok(contribution)
    }

    pub fn review_contribution(
        env: &Env,
        contribution_id: String,
        reviewer: &Address,
        status: ContributionStatus,
        notes: Option<String>,
    ) -> Result<(), Error> {
        let mut contribution: Contribution = env
            .storage()
            .persistent()
            .get(&DataKey::Contribution(contribution_id.clone()))
            .ok_or(Error::ContributionNotFound)?;

        contribution.status = status;
        contribution.reviewed_by = Some(reviewer.clone());
        contribution.review_notes = notes;

        env.storage()
            .persistent()
            .set(&DataKey::Contribution(contribution_id), &contribution);

        Ok(())
    }

    pub fn get_contribution(env: &Env, contribution_id: &String) -> Option<Contribution> {
        env.storage()
            .persistent()
            .get(&DataKey::Contribution(contribution_id.clone()))
    }
}
