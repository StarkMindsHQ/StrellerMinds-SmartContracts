#![no_std]

pub mod types;
pub mod storage;
pub mod events;
pub mod errors;

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};
use types::{MentorProfile, MentorshipStatus, MentorshipSession, Review};
use errors::MentorshipError;

#[contract]
pub struct MentorshipContract;

#[contractimpl]
impl MentorshipContract {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&types::MentorshipDataKey::Admin) {
            panic!("Already initialized");
        }
        storage::set_admin(&env, &admin);
    }

    pub fn register_mentor(
        env: Env,
        mentor: Address,
        name: String,
        expertise: Vec<String>,
        bio: String,
    ) -> Result<(), MentorshipError> {
        mentor.require_auth();
        if storage::get_mentor_profile(&env, &mentor).is_some() {
            return Err(MentorshipError::AlreadyRegistered);
        }

        let profile = MentorProfile {
            address: mentor.clone(),
            name,
            expertise,
            bio,
            rating_sum: 0,
            rating_count: 0,
            is_active: true,
        };

        storage::set_mentor_profile(&env, &mentor, &profile);
        events::emit_mentor_registered(&env, &mentor);
        Ok(())
    }

    pub fn request_mentorship(
        env: Env,
        mentee: Address,
        mentor: Address,
    ) -> Result<u64, MentorshipError> {
        mentee.require_auth();
        if mentee == mentor {
            return Err(MentorshipError::SelfMentorshipNotAllowed);
        }

        let mentor_profile = storage::get_mentor_profile(&env, &mentor)
            .ok_or(MentorshipError::NotRegistered)?;
        
        if !mentor_profile.is_active {
            return Err(MentorshipError::NotRegistered);
        }

        let id = storage::increment_session_counter(&env);
        let session = MentorshipSession {
            id,
            mentor: mentor.clone(),
            mentee: mentee.clone(),
            status: MentorshipStatus::Pending,
            start_time: 0,
            end_time: 0,
            notes: String::from_str(&env, ""),
        };

        storage::set_session(&env, id, &session);
        storage::add_mentor_request(&env, &mentor, id);
        storage::add_mentee_request(&env, &mentee, id);

        events::emit_session_created(&env, id, &mentor, &mentee);
        Ok(id)
    }

    pub fn update_session_status(
        env: Env,
        user: Address,
        session_id: u64,
        new_status: MentorshipStatus,
    ) -> Result<(), MentorshipError> {
        user.require_auth();
        let mut session = storage::get_session(&env, session_id)
            .ok_or(MentorshipError::SessionNotFound)?;

        if user != session.mentor && user != session.mentee {
            return Err(MentorshipError::Unauthorized);
        }

        // Basic status transition logic
        match (&session.status, &new_status) {
            (MentorshipStatus::Pending, MentorshipStatus::Active) => {
                if user != session.mentor { return Err(MentorshipError::Unauthorized); }
                session.start_time = env.ledger().timestamp();
            }
            (MentorshipStatus::Active, MentorshipStatus::Completed) => {
                session.end_time = env.ledger().timestamp();
            }
            (MentorshipStatus::Pending, MentorshipStatus::Rejected) => {
                if user != session.mentor { return Err(MentorshipError::Unauthorized); }
            }
            (MentorshipStatus::Pending, MentorshipStatus::Cancelled) => {}
            _ => return Err(MentorshipError::InvalidStatusTransition),
        }

        session.status = new_status;
        storage::set_session(&env, session_id, &session);
        events::emit_session_updated(&env, session_id, 1); // Simplified status enum emit
        Ok(())
    }

    pub fn submit_review(
        env: Env,
        reviewer: Address,
        target: Address,
        rating: u32,
        comment: String,
    ) -> Result<(), MentorshipError> {
        reviewer.require_auth();
        if rating < 1 || rating > 5 {
            return Err(MentorshipError::InvalidRating);
        }

        // Logic to check if a mentorship session existed could be added here

        if let Some(mut profile) = storage::get_mentor_profile(&env, &target) {
            profile.rating_sum += rating;
            profile.rating_count += 1;
            storage::set_mentor_profile(&env, &target, &profile);
        }

        events::emit_review_submitted(&env, &reviewer, &target, rating);
        Ok(())
    }

    pub fn get_mentor(env: Env, mentor: Address) -> Option<MentorProfile> {
        storage::get_mentor_profile(&env, &mentor)
    }

    pub fn get_session(env: Env, session_id: u64) -> Option<MentorshipSession> {
        storage::get_session(&env, session_id)
    }
}
