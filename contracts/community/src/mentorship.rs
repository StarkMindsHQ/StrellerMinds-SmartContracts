use soroban_sdk::{Address, Env, String, Vec};

use crate::errors::Error;
use crate::events::CommunityEvents;
use crate::storage::CommunityStorage;
use crate::types::*;

pub struct MentorshipManager;

impl MentorshipManager {
    pub fn register_mentor(
        env: &Env,
        mentor: &Address,
        expertise_areas: Vec<String>,
        expertise_level: MentorExpertise,
        max_mentees: u32,
        bio: String,
    ) -> Result<(), Error> {
        if env
            .storage()
            .persistent()
            .has(&CommunityKey::MentorProfile(mentor.clone()))
        {
            return Err(Error::AlreadyMentor);
        }

        let profile = MentorProfile {
            mentor: mentor.clone(),
            expertise_areas,
            expertise_level,
            max_mentees,
            current_mentees: 0,
            total_sessions: 0,
            rating: 100, // Start at 100
            is_available: true,
            bio,
            joined_at: env.ledger().timestamp(),
        };

        env.storage()
            .persistent()
            .set(&CommunityKey::MentorProfile(mentor.clone()), &profile);

        CommunityEvents::emit_mentor_registered(env, mentor);
        Ok(())
    }

    pub fn request_mentorship(
        env: &Env,
        mentee: &Address,
        mentor: &Address,
        topic: String,
        message: String,
    ) -> Result<u64, Error> {
        let profile: MentorProfile = env
            .storage()
            .persistent()
            .get(&CommunityKey::MentorProfile(mentor.clone()))
            .ok_or(Error::MentorNotAvailable)?;

        if !profile.is_available {
            return Err(Error::MentorNotAvailable);
        }

        if profile.current_mentees >= profile.max_mentees {
            return Err(Error::MaxMenteesReached);
        }

        let request_id = CommunityStorage::increment_counter(env, CommunityKey::MentorshipCounter);
        let now = env.ledger().timestamp();

        let request = MentorshipRequest {
            id: request_id,
            mentee: mentee.clone(),
            mentor: mentor.clone(),
            topic,
            message,
            status: MentorshipStatus::Pending,
            created_at: now,
            started_at: 0,
            completed_at: 0,
        };

        env.storage()
            .persistent()
            .set(&CommunityKey::MentorshipRequest(request_id), &request);

        // Add to user mentorships
        let mut mentorships: Vec<u64> = env
            .storage()
            .persistent()
            .get(&CommunityKey::UserMentorships(mentee.clone()))
            .unwrap_or_else(|| Vec::new(env));
        mentorships.push_back(request_id);
        env.storage()
            .persistent()
            .set(&CommunityKey::UserMentorships(mentee.clone()), &mentorships);

        CommunityEvents::emit_mentorship_requested(env, mentee, mentor, request_id);
        Ok(request_id)
    }

    pub fn accept_mentorship(
        env: &Env,
        mentor: &Address,
        request_id: u64,
    ) -> Result<(), Error> {
        let mut request: MentorshipRequest = env
            .storage()
            .persistent()
            .get(&CommunityKey::MentorshipRequest(request_id))
            .ok_or(Error::MentorshipNotFound)?;

        if request.mentor != *mentor {
            return Err(Error::Unauthorized);
        }

        if request.status != MentorshipStatus::Pending {
            return Err(Error::InvalidMentorshipStatus);
        }

        request.status = MentorshipStatus::Active;
        request.started_at = env.ledger().timestamp();

        env.storage()
            .persistent()
            .set(&CommunityKey::MentorshipRequest(request_id), &request);

        // Update mentor profile
        let mut profile: MentorProfile = env
            .storage()
            .persistent()
            .get(&CommunityKey::MentorProfile(mentor.clone()))
            .unwrap();
        profile.current_mentees += 1;
        env.storage()
            .persistent()
            .set(&CommunityKey::MentorProfile(mentor.clone()), &profile);

        CommunityEvents::emit_mentorship_started(env, request_id);
        Ok(())
    }

    pub fn complete_session(
        env: &Env,
        mentor: &Address,
        request_id: u64,
        duration: u64,
        notes: String,
    ) -> Result<u64, Error> {
        let request: MentorshipRequest = env
            .storage()
            .persistent()
            .get(&CommunityKey::MentorshipRequest(request_id))
            .ok_or(Error::MentorshipNotFound)?;

        if request.mentor != *mentor {
            return Err(Error::Unauthorized);
        }

        if request.status != MentorshipStatus::Active {
            return Err(Error::InvalidMentorshipStatus);
        }

        let session_id = CommunityStorage::increment_counter(env, CommunityKey::SessionCounter);
        let now = env.ledger().timestamp();

        let session = MentorshipSession {
            id: session_id,
            request_id,
            mentor: mentor.clone(),
            mentee: request.mentee.clone(),
            topic: request.topic.clone(),
            duration,
            notes,
            rating: 0, // To be rated by mentee
            completed_at: now,
        };

        env.storage()
            .persistent()
            .set(&CommunityKey::MentorshipSession(session_id), &session);

        // Update mentor profile
        let mut profile: MentorProfile = env
            .storage()
            .persistent()
            .get(&CommunityKey::MentorProfile(mentor.clone()))
            .unwrap();
        profile.total_sessions += 1;
        env.storage()
            .persistent()
            .set(&CommunityKey::MentorProfile(mentor.clone()), &profile);

        // Update user stats
        Self::update_mentee_stats(env, &request.mentee);
        
        // Award XP
        let config = CommunityStorage::get_config(env);
        Self::award_xp(env, mentor, config.mentor_session_xp);
        Self::award_xp(env, &request.mentee, config.mentor_session_xp / 2);

        CommunityEvents::emit_session_completed(env, session_id, mentor, &request.mentee);
        Ok(session_id)
    }

    pub fn rate_session(
        env: &Env,
        mentee: &Address,
        session_id: u64,
        rating: u32,
    ) -> Result<(), Error> {
        let mut session: MentorshipSession = env
            .storage()
            .persistent()
            .get(&CommunityKey::MentorshipSession(session_id))
            .ok_or(Error::NotFound)?;

        if session.mentee != *mentee {
            return Err(Error::Unauthorized);
        }

        if rating > 100 {
            return Err(Error::InvalidInput);
        }

        session.rating = rating;
        env.storage()
            .persistent()
            .set(&CommunityKey::MentorshipSession(session_id), &session);

        // Update mentor's average rating
        Self::update_mentor_rating(env, &session.mentor);

        Ok(())
    }

    pub fn get_mentor_profile(env: &Env, mentor: &Address) -> Option<MentorProfile> {
        env.storage()
            .persistent()
            .get(&CommunityKey::MentorProfile(mentor.clone()))
    }

    pub fn get_available_mentors(env: &Env) -> Vec<MentorProfile> {
        // In production, this would use an index
        // For now, returning empty vec as placeholder
        Vec::new(env)
    }

    // Helper functions
    fn update_mentor_rating(env: &Env, mentor: &Address) {
        // Calculate average rating from all sessions
        // Simplified implementation
        let profile: MentorProfile = env
            .storage()
            .persistent()
            .get(&CommunityKey::MentorProfile(mentor.clone()))
            .unwrap();
        
        // In production, iterate through sessions and calculate average
        // For now, keeping current rating
        env.storage()
            .persistent()
            .set(&CommunityKey::MentorProfile(mentor.clone()), &profile);
    }

    fn update_mentee_stats(env: &Env, mentee: &Address) {
        let mut stats: UserCommunityStats = env
            .storage()
            .persistent()
            .get(&CommunityKey::UserStats(mentee.clone()))
            .unwrap_or(UserCommunityStats {
                user: mentee.clone(),
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
        
        stats.mentorship_sessions += 1;
        env.storage()
            .persistent()
            .set(&CommunityKey::UserStats(mentee.clone()), &stats);
    }

    fn award_xp(_env: &Env, _user: &Address, _xp: u32) {
        // Integration point with gamification contract
    }
}
