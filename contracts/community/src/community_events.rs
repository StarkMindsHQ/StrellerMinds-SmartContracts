use soroban_sdk::{Address, Env, String, Vec};

use crate::errors::Error;
use crate::events::CommunityEvents;
use crate::storage::CommunityStorage;
use crate::types::*;

pub struct EventManager;

impl EventManager {
    pub fn create_event(
        env: &Env,
        organizer: &Address,
        event_type: EventType,
        title: String,
        description: String,
        start_time: u64,
        end_time: u64,
        max_participants: u32,
        is_public: bool,
        xp_reward: u32,
    ) -> Result<u64, Error> {
        let event_id = CommunityStorage::increment_counter(env, CommunityKey::EventCounter);
        let now = env.ledger().timestamp();

        let event = CommunityEvent {
            id: event_id,
            organizer: organizer.clone(),
            event_type,
            title,
            description,
            start_time,
            end_time,
            max_participants,
            current_participants: 0,
            status: EventStatus::Scheduled,
            is_public,
            xp_reward,
            created_at: now,
        };

        env.storage()
            .persistent()
            .set(&CommunityKey::Event(event_id), &event);

        CommunityEvents::emit_event_created(env, organizer, event_id);
        Ok(event_id)
    }

    pub fn register_for_event(
        env: &Env,
        user: &Address,
        event_id: u64,
    ) -> Result<(), Error> {
        let mut event: CommunityEvent = env
            .storage()
            .persistent()
            .get(&CommunityKey::Event(event_id))
            .ok_or(Error::EventNotFound)?;

        if event.current_participants >= event.max_participants {
            return Err(Error::EventFull);
        }

        // Check if already registered
        let participant_key = CommunityKey::EventParticipant(user.clone(), event_id);
        if env.storage().persistent().has(&participant_key) {
            return Err(Error::AlreadyRegistered);
        }

        let participant = EventParticipant {
            user: user.clone(),
            event_id,
            registered_at: env.ledger().timestamp(),
            attended: false,
            feedback_rating: 0,
        };

        env.storage()
            .persistent()
            .set(&participant_key, &participant);

        // Add to event participants list
        let mut participants: Vec<Address> = env
            .storage()
            .persistent()
            .get(&CommunityKey::EventParticipants(event_id))
            .unwrap_or_else(|| Vec::new(env));
        participants.push_back(user.clone());
        env.storage()
            .persistent()
            .set(&CommunityKey::EventParticipants(event_id), &participants);

        // Add to user events
        let mut user_events: Vec<u64> = env
            .storage()
            .persistent()
            .get(&CommunityKey::UserEvents(user.clone()))
            .unwrap_or_else(|| Vec::new(env));
        user_events.push_back(event_id);
        env.storage()
            .persistent()
            .set(&CommunityKey::UserEvents(user.clone()), &user_events);

        // Update event
        event.current_participants += 1;
        env.storage()
            .persistent()
            .set(&CommunityKey::Event(event_id), &event);

        CommunityEvents::emit_event_registered(env, user, event_id);
        Ok(())
    }

    pub fn mark_attendance(
        env: &Env,
        organizer: &Address,
        event_id: u64,
        user: &Address,
    ) -> Result<(), Error> {
        let event: CommunityEvent = env
            .storage()
            .persistent()
            .get(&CommunityKey::Event(event_id))
            .ok_or(Error::EventNotFound)?;

        if event.organizer != *organizer {
            CommunityStorage::require_moderator(env, organizer)?;
        }

        let participant_key = CommunityKey::EventParticipant(user.clone(), event_id);
        let mut participant: EventParticipant = env
            .storage()
            .persistent()
            .get(&participant_key)
            .ok_or(Error::NotFound)?;

        participant.attended = true;
        env.storage()
            .persistent()
            .set(&participant_key, &participant);

        // Update user stats
        Self::update_user_stats(env, user);

        // Award XP
        Self::award_xp(env, user, event.xp_reward);

        Ok(())
    }

    pub fn complete_event(
        env: &Env,
        organizer: &Address,
        event_id: u64,
    ) -> Result<(), Error> {
        let mut event: CommunityEvent = env
            .storage()
            .persistent()
            .get(&CommunityKey::Event(event_id))
            .ok_or(Error::EventNotFound)?;

        if event.organizer != *organizer {
            return Err(Error::Unauthorized);
        }

        event.status = EventStatus::Completed;
        env.storage()
            .persistent()
            .set(&CommunityKey::Event(event_id), &event);

        CommunityEvents::emit_event_completed(env, event_id);
        Ok(())
    }

    pub fn submit_feedback(
        env: &Env,
        user: &Address,
        event_id: u64,
        rating: u32,
    ) -> Result<(), Error> {
        if rating > 100 {
            return Err(Error::InvalidInput);
        }

        let participant_key = CommunityKey::EventParticipant(user.clone(), event_id);
        let mut participant: EventParticipant = env
            .storage()
            .persistent()
            .get(&participant_key)
            .ok_or(Error::NotFound)?;

        if !participant.attended {
            return Err(Error::Unauthorized);
        }

        participant.feedback_rating = rating;
        env.storage()
            .persistent()
            .set(&participant_key, &participant);

        Ok(())
    }

    pub fn get_event(env: &Env, event_id: u64) -> Option<CommunityEvent> {
        env.storage()
            .persistent()
            .get(&CommunityKey::Event(event_id))
    }

    pub fn get_event_participants(env: &Env, event_id: u64) -> Vec<EventParticipant> {
        let addresses: Vec<Address> = env
            .storage()
            .persistent()
            .get(&CommunityKey::EventParticipants(event_id))
            .unwrap_or_else(|| Vec::new(env));

        let mut participants = Vec::new(env);
        for addr in addresses.iter() {
            if let Some(p) = env
                .storage()
                .persistent()
                .get(&CommunityKey::EventParticipant(addr, event_id))
            {
                participants.push_back(p);
            }
        }
        participants
    }

    // Helper functions
    fn update_user_stats(env: &Env, user: &Address) {
        let mut stats: UserCommunityStats = env
            .storage()
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
            });

        stats.events_attended += 1;
        env.storage()
            .persistent()
            .set(&CommunityKey::UserStats(user.clone()), &stats);
    }

    fn award_xp(_env: &Env, _user: &Address, _xp: u32) {
        // Integration point with gamification contract
    }
}
