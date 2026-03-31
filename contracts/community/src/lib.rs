#![no_std]
#![allow(clippy::too_many_arguments)]
pub mod analytics;
pub mod community_events;
pub mod errors;
pub mod events;
pub mod forum;
pub mod governance;
pub mod knowledge;
pub mod mentorship;
pub mod moderation;
pub mod storage;
pub mod types;

#[cfg(test)]
mod tests;

use shared::rate_limiter::{enforce_rate_limit, RateLimitConfig};
use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

pub use errors::CommunityError;
pub use errors::Error;
pub use types::*;

use analytics::AnalyticsManager;
use community_events::EventManager;
use forum::ForumManager;
use governance::GovernanceManager;
use knowledge::KnowledgeManager;
use mentorship::MentorshipManager;
use moderation::ModerationManager;
use storage::CommunityStorage;

// Rate limit operation IDs
const RL_OP_POST: u64 = 1;
const RL_OP_REPLY: u64 = 2;
const RL_OP_VOTE: u64 = 3;
const RL_OP_REPORT: u64 = 4;
const RL_OP_PROPOSAL: u64 = 5;
const RL_OP_CONTRIBUTION: u64 = 6;

fn check_rate_limit(
    env: &Env,
    user: &Address,
    operation: u64,
    config: &RateLimitConfig,
) -> Result<(), Error> {
    let is_admin = CommunityStorage::require_admin(env, user).is_ok();
    if is_admin {
        return Ok(());
    }
    enforce_rate_limit(env, &CommunityKey::RateLimit(user.clone(), operation), config)
        .map_err(|_| Error::RateLimitExceeded)
}

#[contract]
pub struct Community;

#[contractimpl]
impl Community {
    // ══════════════════════════════════════════════════════════════════════
    //  Initialization
    // ══════════════════════════════════════════════════════════════════════

    /// One-time setup: seeds default config and initializes all persistent counters.
    ///
    /// Caller must be the intended admin address.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `admin` - Address that will hold admin privileges for this contract.
    ///
    /// # Errors
    /// Returns [`CommunityError::AlreadyInitialized`] if the contract was already set up.
    ///
    /// # Example
    /// ```ignore
    /// client.initialize(&admin);
    /// ```
    pub fn initialize(env: Env, admin: Address) -> Result<(), CommunityError> {
        admin.require_auth();

        if CommunityStorage::is_initialized(&env) {
            return Err(Error::AlreadyInitialized);
        }

        env.storage().instance().set(&CommunityKey::Admin, &admin);

        let config = CommunityConfig {
            post_xp_reward: 10,
            reply_xp_reward: 5,
            solution_xp_reward: 50,
            contribution_base_xp: 100,
            contribution_base_tokens: 1000,
            mentor_session_xp: 75,
            event_attendance_xp: 25,
            min_reputation_to_moderate: 500,
            max_reports_per_day: 10,
            vote_weight_threshold: 100,
            rate_limit_post: 5,
            rate_limit_reply: 20,
            rate_limit_vote: 50,
            rate_limit_proposal: 2,
            rate_limit_contribution: 5,
            rate_limit_window: 86_400,
        };
        CommunityStorage::set_config(&env, &config);

        // Initialize counters
        for key in [
            CommunityKey::PostCounter,
            CommunityKey::ReplyCounter,
            CommunityKey::ContributionCounter,
            CommunityKey::EventCounter,
            CommunityKey::ReportCounter,
            CommunityKey::ProposalCounter,
            CommunityKey::MentorshipCounter,
            CommunityKey::SessionCounter,
        ] {
            env.storage().persistent().set(&key, &0u64);
        }

        events::CommunityEvents::emit_initialized(&env, &admin);
        Ok(())
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Forum Functions
    // ══════════════════════════════════════════════════════════════════════

    /// Create a new forum post and award XP to the author.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `author` - Address of the user creating the post.
    /// * `category` - Forum category the post belongs to.
    /// * `title` - Title of the post.
    /// * `content` - Body text of the post.
    /// * `tags` - List of searchable tags to attach to the post.
    /// * `course_id` - Identifier of the related course (empty string if none).
    ///
    /// # Errors
    /// Returns [`CommunityError::InvalidInput`] if required fields are empty.
    ///
    /// # Example
    /// ```ignore
    /// client.create_post(&author, &category, &title, &content, &tags, &course_id);
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn create_post(
        env: Env,
        author: Address,
        category: ForumCategory,
        title: String,
        content: String,
        tags: Vec<String>,
        course_id: String,
    ) -> Result<u64, CommunityError> {
        author.require_auth();
        let cfg = CommunityStorage::get_config(&env);
        check_rate_limit(
            &env,
            &author,
            RL_OP_POST,
            &RateLimitConfig {
                max_calls: cfg.rate_limit_post,
                window_seconds: cfg.rate_limit_window,
            },
        )?;
        ForumManager::create_post(&env, &author, category, title, content, tags, course_id)
    }

    /// Add a reply to an existing forum post, supporting nested threads via `parent_reply_id`.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `author` - Address of the user writing the reply.
    /// * `post_id` - ID of the post being replied to.
    /// * `content` - Body text of the reply.
    /// * `parent_reply_id` - ID of the parent reply for nesting (use `0` for top-level replies).
    ///
    /// # Errors
    /// Returns [`CommunityError::PostNotFound`] if `post_id` does not exist.
    /// Returns [`CommunityError::PostClosed`] if the post is no longer accepting replies.
    ///
    /// # Example
    /// ```ignore
    /// client.create_reply(&author, &post_id, &content, &parent_reply_id);
    /// ```
    pub fn create_reply(
        env: Env,
        author: Address,
        post_id: u64,
        content: String,
        parent_reply_id: u64,
    ) -> Result<u64, CommunityError> {
        author.require_auth();
        let cfg = CommunityStorage::get_config(&env);
        check_rate_limit(
            &env,
            &author,
            RL_OP_REPLY,
            &RateLimitConfig {
                max_calls: cfg.rate_limit_reply,
                window_seconds: cfg.rate_limit_window,
            },
        )?;
        ForumManager::create_reply(&env, &author, post_id, content, parent_reply_id)
    }

    /// Mark a reply as the accepted solution for a post, awarding XP to the reply author.
    ///
    /// Only the original post author may call this function.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `post_author` - Address of the post's original author.
    /// * `post_id` - ID of the post to update.
    /// * `reply_id` - ID of the reply to mark as the solution.
    ///
    /// # Errors
    /// Returns [`CommunityError::Unauthorized`] if the caller is not the post author.
    /// Returns [`CommunityError::ReplyNotFound`] if `reply_id` does not exist on the post.
    ///
    /// # Example
    /// ```ignore
    /// client.mark_solution(&post_author, &post_id, &reply_id);
    /// ```
    pub fn mark_solution(
        env: Env,
        post_author: Address,
        post_id: u64,
        reply_id: u64,
    ) -> Result<(), CommunityError> {
        post_author.require_auth();
        ForumManager::mark_solution(&env, &post_author, post_id, reply_id)
    }

    /// Cast an upvote or downvote on a forum post.
    ///
    /// Each user may only vote once per post.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `voter` - Address of the user casting the vote.
    /// * `post_id` - ID of the post to vote on.
    /// * `upvote` - `true` for an upvote, `false` for a downvote.
    ///
    /// # Errors
    /// Returns [`CommunityError::AlreadyVoted`] if the user has already voted on this post.
    /// Returns [`CommunityError::PostNotFound`] if `post_id` does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.vote_post(&voter, &post_id, &upvote);
    /// ```
    pub fn vote_post(
        env: Env,
        voter: Address,
        post_id: u64,
        upvote: bool,
    ) -> Result<(), CommunityError> {
        voter.require_auth();
        let cfg = CommunityStorage::get_config(&env);
        check_rate_limit(
            &env,
            &voter,
            RL_OP_VOTE,
            &RateLimitConfig {
                max_calls: cfg.rate_limit_vote,
                window_seconds: cfg.rate_limit_window,
            },
        )?;
        ForumManager::vote_post(&env, &voter, post_id, upvote)
    }

    /// Retrieve a forum post by ID, returning `None` if it does not exist.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `post_id` - ID of the post to fetch.
    ///
    /// # Example
    /// ```ignore
    /// client.get_post(&post_id);
    /// ```
    pub fn get_post(env: Env, post_id: u64) -> Option<ForumPost> {
        ForumManager::get_post(&env, post_id)
    }

    /// Return all replies for a given post.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `post_id` - ID of the post whose replies to list.
    ///
    /// # Example
    /// ```ignore
    /// client.get_post_replies(&post_id);
    /// ```
    pub fn get_post_replies(env: Env, post_id: u64) -> Vec<ForumReply> {
        ForumManager::get_post_replies(&env, post_id)
    }

    /// Return up to `limit` posts from a specific forum category, sorted by recency.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `category` - The forum category to query.
    /// * `limit` - Maximum number of posts to return.
    ///
    /// # Example
    /// ```ignore
    /// client.get_category_posts(&category, &limit);
    /// ```
    pub fn get_category_posts(env: Env, category: ForumCategory, limit: u32) -> Vec<ForumPost> {
        ForumManager::get_category_posts(&env, category, limit)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Mentorship Functions
    // ══════════════════════════════════════════════════════════════════════

    /// Register the caller as an available mentor with their expertise profile.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `mentor` - Address of the user registering as a mentor.
    /// * `expertise_areas` - List of subject areas the mentor can help with.
    /// * `expertise_level` - Self-assessed expertise tier.
    /// * `max_mentees` - Maximum number of concurrent mentees the mentor will accept.
    /// * `bio` - Short biography displayed to potential mentees.
    ///
    /// # Errors
    /// Returns [`CommunityError::AlreadyMentor`] if the user is already registered as a mentor.
    ///
    /// # Example
    /// ```ignore
    /// client.register_mentor(&mentor, &expertise_areas, &expertise_level, &max_mentees, &bio);
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn register_mentor(
        env: Env,
        mentor: Address,
        expertise_areas: Vec<String>,
        expertise_level: MentorExpertise,
        max_mentees: u32,
        bio: String,
    ) -> Result<(), CommunityError> {
        mentor.require_auth();
        MentorshipManager::register_mentor(
            &env,
            &mentor,
            expertise_areas,
            expertise_level,
            max_mentees,
            bio,
        )
    }

    /// Submit a mentorship request from `mentee` to a specific `mentor`.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `mentee` - Address of the user requesting mentorship.
    /// * `mentor` - Address of the mentor being approached.
    /// * `topic` - Subject or goal of the mentorship.
    /// * `message` - Introductory message to the mentor.
    ///
    /// # Errors
    /// Returns [`CommunityError::MentorNotAvailable`] if the mentor is at capacity or inactive.
    ///
    /// # Example
    /// ```ignore
    /// client.request_mentorship(&mentee, &mentor, &topic, &message);
    /// ```
    pub fn request_mentorship(
        env: Env,
        mentee: Address,
        mentor: Address,
        topic: String,
        message: String,
    ) -> Result<u64, CommunityError> {
        mentee.require_auth();
        MentorshipManager::request_mentorship(&env, &mentee, &mentor, topic, message)
    }

    /// Accept a pending mentorship request, moving it to the active state.
    ///
    /// Only the addressed mentor may call this function.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `mentor` - Address of the mentor accepting the request.
    /// * `request_id` - ID of the mentorship request to accept.
    ///
    /// # Errors
    /// Returns [`CommunityError::MentorshipNotFound`] if `request_id` does not exist.
    /// Returns [`CommunityError::Unauthorized`] if the caller is not the addressed mentor.
    ///
    /// # Example
    /// ```ignore
    /// client.accept_mentorship(&mentor, &request_id);
    /// ```
    pub fn accept_mentorship(
        env: Env,
        mentor: Address,
        request_id: u64,
    ) -> Result<(), CommunityError> {
        mentor.require_auth();
        MentorshipManager::accept_mentorship(&env, &mentor, request_id)
    }

    /// Record a completed mentorship session and award XP to the mentor.
    ///
    /// Only the mentor for the given request may call this function.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `mentor` - Address of the mentor completing the session.
    /// * `request_id` - ID of the active mentorship request.
    /// * `duration` - Length of the session in seconds.
    /// * `notes` - Session summary or notes for the mentee.
    ///
    /// # Errors
    /// Returns [`CommunityError::InvalidMentorshipStatus`] if the request is not active.
    ///
    /// # Example
    /// ```ignore
    /// client.complete_session(&mentor, &request_id, &duration, &notes);
    /// ```
    pub fn complete_session(
        env: Env,
        mentor: Address,
        request_id: u64,
        duration: u64,
        notes: String,
    ) -> Result<u64, CommunityError> {
        mentor.require_auth();
        MentorshipManager::complete_session(&env, &mentor, request_id, duration, notes)
    }

    /// Submit a quality rating for a completed mentorship session.
    ///
    /// Only the mentee from the related request may rate the session.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `mentee` - Address of the mentee rating the session.
    /// * `session_id` - ID of the completed session to rate.
    /// * `rating` - Numeric rating score (valid range is enforced by the manager).
    ///
    /// # Errors
    /// Returns [`CommunityError::Unauthorized`] if the caller is not the session's mentee.
    ///
    /// # Example
    /// ```ignore
    /// client.rate_session(&mentee, &session_id, &rating);
    /// ```
    pub fn rate_session(
        env: Env,
        mentee: Address,
        session_id: u64,
        rating: u32,
    ) -> Result<(), CommunityError> {
        mentee.require_auth();
        MentorshipManager::rate_session(&env, &mentee, session_id, rating)
    }

    /// Retrieve the mentor profile for an address, returning `None` if not registered.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `mentor` - Address of the mentor to look up.
    ///
    /// # Example
    /// ```ignore
    /// client.get_mentor_profile(&mentor);
    /// ```
    pub fn get_mentor_profile(env: Env, mentor: Address) -> Option<MentorProfile> {
        MentorshipManager::get_mentor_profile(&env, &mentor)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Knowledge Base Functions
    // ══════════════════════════════════════════════════════════════════════

    /// Submit a new knowledge-base contribution for moderator review.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `contributor` - Address of the user submitting the contribution.
    /// * `contribution_type` - Type classification of the contribution.
    /// * `title` - Title of the knowledge article or resource.
    /// * `content` - Full body content of the contribution.
    /// * `category` - Forum category the contribution is filed under.
    /// * `tags` - Searchable tags to attach to the contribution.
    ///
    /// # Errors
    /// Returns [`CommunityError::InvalidInput`] if required fields are blank.
    ///
    /// # Example
    /// ```ignore
    /// client.submit_contribution(&contributor, &contribution_type, &title, &content, &category, &tags);
    /// ```
    pub fn submit_contribution(
        env: Env,
        contributor: Address,
        contribution_type: ContributionType,
        title: String,
        content: String,
        category: ForumCategory,
        tags: Vec<String>,
    ) -> Result<u64, CommunityError> {
        contributor.require_auth();
        let cfg = CommunityStorage::get_config(&env);
        check_rate_limit(
            &env,
            &contributor,
            RL_OP_CONTRIBUTION,
            &RateLimitConfig {
                max_calls: cfg.rate_limit_contribution,
                window_seconds: cfg.rate_limit_window,
            },
        )?;
        KnowledgeManager::submit_contribution(
            &env,
            &contributor,
            contribution_type,
            title,
            content,
            category,
            tags,
        )
    }

    /// Approve or reject a pending knowledge-base contribution.
    ///
    /// Caller must hold a moderator role.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `moderator` - Address of the moderator performing the review.
    /// * `contribution_id` - ID of the contribution to review.
    /// * `approve` - `true` to approve and publish, `false` to reject.
    ///
    /// # Errors
    /// Returns [`CommunityError::NotModerator`] if the caller lacks moderator privileges.
    /// Returns [`CommunityError::ContributionNotFound`] if `contribution_id` does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.review_contribution(&moderator, &contribution_id, &approve);
    /// ```
    pub fn review_contribution(
        env: Env,
        moderator: Address,
        contribution_id: u64,
        approve: bool,
    ) -> Result<(), CommunityError> {
        moderator.require_auth();
        KnowledgeManager::review_contribution(&env, &moderator, contribution_id, approve)
    }

    /// Cast an upvote or downvote on a published knowledge contribution.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `voter` - Address of the user voting.
    /// * `contribution_id` - ID of the contribution to vote on.
    /// * `upvote` - `true` for an upvote, `false` for a downvote.
    ///
    /// # Errors
    /// Returns [`CommunityError::ContributionNotFound`] if `contribution_id` does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.vote_contribution(&voter, &contribution_id, &upvote);
    /// ```
    pub fn vote_contribution(
        env: Env,
        voter: Address,
        contribution_id: u64,
        upvote: bool,
    ) -> Result<(), CommunityError> {
        voter.require_auth();
        KnowledgeManager::vote_contribution(&env, &voter, contribution_id, upvote)
    }

    /// Retrieve a knowledge contribution by ID, returning `None` if it does not exist.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `contribution_id` - ID of the contribution to fetch.
    ///
    /// # Example
    /// ```ignore
    /// client.get_contribution(&contribution_id);
    /// ```
    pub fn get_contribution(env: Env, contribution_id: u64) -> Option<KnowledgeContribution> {
        KnowledgeManager::get_contribution(&env, contribution_id)
    }

    /// Return all knowledge contributions submitted by `user`.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `user` - Address of the contributor to query.
    ///
    /// # Example
    /// ```ignore
    /// client.get_user_contributions(&user);
    /// ```
    pub fn get_user_contributions(env: Env, user: Address) -> Vec<KnowledgeContribution> {
        KnowledgeManager::get_user_contributions(&env, &user)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Event Functions
    // ══════════════════════════════════════════════════════════════════════

    /// Create a new community event with an optional XP reward for attendees.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `organizer` - Address of the user organizing the event.
    /// * `event_type` - Classification of the event (workshop, AMA, etc.).
    /// * `title` - Display title of the event.
    /// * `description` - Full description visible to potential attendees.
    /// * `start_time` - Unix timestamp when the event begins.
    /// * `end_time` - Unix timestamp when the event ends.
    /// * `max_participants` - Maximum number of registrants allowed.
    /// * `is_public` - Whether the event is open to all users.
    /// * `xp_reward` - XP awarded to each confirmed attendee on completion.
    ///
    /// # Errors
    /// Returns [`CommunityError::InvalidInput`] if `end_time` is not after `start_time`.
    ///
    /// # Example
    /// ```ignore
    /// client.create_event(&organizer, &event_type, &title, &description, &start_time, &end_time, &max_participants, &is_public, &xp_reward);
    /// ```
    #[allow(clippy::too_many_arguments)]
    pub fn create_event(
        env: Env,
        organizer: Address,
        event_type: EventType,
        title: String,
        description: String,
        start_time: u64,
        end_time: u64,
        max_participants: u32,
        is_public: bool,
        xp_reward: u32,
    ) -> Result<u64, CommunityError> {
        organizer.require_auth();
        EventManager::create_event(
            &env,
            &organizer,
            event_type,
            title,
            description,
            start_time,
            end_time,
            max_participants,
            is_public,
            xp_reward,
        )
    }

    /// Register `user` to attend a community event.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `user` - Address of the user registering.
    /// * `event_id` - ID of the event to register for.
    ///
    /// # Errors
    /// Returns [`CommunityError::EventFull`] if the event has reached max participants.
    /// Returns [`CommunityError::AlreadyRegistered`] if the user is already registered.
    /// Returns [`CommunityError::EventNotActive`] if the event is not accepting registrations.
    ///
    /// # Example
    /// ```ignore
    /// client.register_for_event(&user, &event_id);
    /// ```
    pub fn register_for_event(
        env: Env,
        user: Address,
        event_id: u64,
    ) -> Result<(), CommunityError> {
        user.require_auth();
        EventManager::register_for_event(&env, &user, event_id)
    }

    /// Confirm that `user` attended an event, making them eligible for the XP reward.
    ///
    /// Only the event organizer may mark attendance.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `organizer` - Address of the event organizer calling this function.
    /// * `event_id` - ID of the event.
    /// * `user` - Address of the attendee to mark.
    ///
    /// # Errors
    /// Returns [`CommunityError::Unauthorized`] if the caller is not the event organizer.
    /// Returns [`CommunityError::EventNotFound`] if `event_id` does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.mark_attendance(&organizer, &event_id, &user);
    /// ```
    pub fn mark_attendance(
        env: Env,
        organizer: Address,
        event_id: u64,
        user: Address,
    ) -> Result<(), CommunityError> {
        organizer.require_auth();
        EventManager::mark_attendance(&env, &organizer, event_id, &user)
    }

    /// Mark an event as completed and distribute XP rewards to confirmed attendees.
    ///
    /// Only the event organizer may close an event.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `organizer` - Address of the event organizer.
    /// * `event_id` - ID of the event to close.
    ///
    /// # Errors
    /// Returns [`CommunityError::Unauthorized`] if the caller is not the event organizer.
    /// Returns [`CommunityError::EventNotActive`] if the event is already completed or cancelled.
    ///
    /// # Example
    /// ```ignore
    /// client.complete_event(&organizer, &event_id);
    /// ```
    pub fn complete_event(
        env: Env,
        organizer: Address,
        event_id: u64,
    ) -> Result<(), CommunityError> {
        organizer.require_auth();
        EventManager::complete_event(&env, &organizer, event_id)
    }

    /// Submit a rating for an event the caller attended.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `user` - Address of the attendee submitting feedback.
    /// * `event_id` - ID of the event to rate.
    /// * `rating` - Numeric rating score.
    ///
    /// # Errors
    /// Returns [`CommunityError::EventNotFound`] if `event_id` does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.submit_event_feedback(&user, &event_id, &rating);
    /// ```
    pub fn submit_event_feedback(
        env: Env,
        user: Address,
        event_id: u64,
        rating: u32,
    ) -> Result<(), CommunityError> {
        user.require_auth();
        EventManager::submit_feedback(&env, &user, event_id, rating)
    }

    /// Retrieve a community event by ID, returning `None` if it does not exist.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `event_id` - ID of the event to fetch.
    ///
    /// # Example
    /// ```ignore
    /// client.get_event(&event_id);
    /// ```
    pub fn get_event(env: Env, event_id: u64) -> Option<CommunityEvent> {
        EventManager::get_event(&env, event_id)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Moderation Functions
    // ══════════════════════════════════════════════════════════════════════

    /// Grant a moderation role to `moderator`.
    ///
    /// Only the contract admin may assign moderators.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `admin` - Admin address (must match stored admin).
    /// * `moderator` - Address to be granted the moderator role.
    /// * `role` - The moderation role level to assign.
    ///
    /// # Errors
    /// Returns [`CommunityError::Unauthorized`] if the caller is not the contract admin.
    ///
    /// # Example
    /// ```ignore
    /// client.add_moderator(&admin, &moderator, &role);
    /// ```
    pub fn add_moderator(
        env: Env,
        admin: Address,
        moderator: Address,
        role: ModeratorRole,
    ) -> Result<(), CommunityError> {
        admin.require_auth();
        ModerationManager::add_moderator(&env, &admin, &moderator, role)
    }

    /// File a moderation report against a piece of content.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `reporter` - Address of the user submitting the report.
    /// * `content_type` - String identifier for the content type (e.g. `"post"`, `"reply"`).
    /// * `content_id` - ID of the specific content item being reported.
    /// * `reason` - Enumerated reason for the report.
    /// * `description` - Additional context provided by the reporter.
    ///
    /// # Errors
    /// Returns [`CommunityError::ReportLimitReached`] if the reporter has hit their daily cap.
    /// Returns [`CommunityError::AlreadyReported`] if the reporter already reported this item.
    ///
    /// # Example
    /// ```ignore
    /// client.report_content(&reporter, &content_type, &content_id, &reason, &description);
    /// ```
    pub fn report_content(
        env: Env,
        reporter: Address,
        content_type: String,
        content_id: u64,
        reason: ReportReason,
        description: String,
    ) -> Result<u64, CommunityError> {
        reporter.require_auth();
        let cfg = CommunityStorage::get_config(&env);
        check_rate_limit(
            &env,
            &reporter,
            RL_OP_REPORT,
            &RateLimitConfig {
                max_calls: cfg.max_reports_per_day,
                window_seconds: cfg.rate_limit_window,
            },
        )?;
        ModerationManager::report_content(
            &env,
            &reporter,
            content_type,
            content_id,
            reason,
            description,
        )
    }

    /// Resolve a pending moderation report by taking an action on the reported content.
    ///
    /// Caller must hold a moderator role.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `moderator` - Address of the moderator resolving the report.
    /// * `report_id` - ID of the report to resolve.
    /// * `action` - String describing the action taken (e.g. `"dismissed"`, `"removed"`).
    ///
    /// # Errors
    /// Returns [`CommunityError::NotModerator`] if the caller lacks moderator privileges.
    /// Returns [`CommunityError::ReportNotFound`] if `report_id` does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.resolve_report(&moderator, &report_id, &action);
    /// ```
    pub fn resolve_report(
        env: Env,
        moderator: Address,
        report_id: u64,
        action: String,
    ) -> Result<(), CommunityError> {
        moderator.require_auth();
        ModerationManager::resolve_report(&env, &moderator, report_id, action)
    }

    /// Return all content reports that are awaiting moderator action.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    ///
    /// # Example
    /// ```ignore
    /// client.get_pending_reports();
    /// ```
    pub fn get_pending_reports(env: Env) -> Vec<ContentReport> {
        ModerationManager::get_pending_reports(&env)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Governance Functions
    // ══════════════════════════════════════════════════════════════════════

    /// Submit a community governance proposal for token-weighted voting.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `proposer` - Address of the user submitting the proposal.
    /// * `proposal_type` - Category of the governance proposal.
    /// * `title` - Short title of the proposal.
    /// * `description` - Full description of the proposed change.
    /// * `voting_duration` - How long (in seconds) the voting window remains open.
    /// * `min_votes_required` - Minimum number of votes needed for the proposal to be valid.
    ///
    /// # Errors
    /// Returns [`CommunityError::InsufficientVotingPower`] if the proposer lacks sufficient reputation.
    ///
    /// # Example
    /// ```ignore
    /// client.create_proposal(&proposer, &proposal_type, &title, &description, &voting_duration, &min_votes_required);
    /// ```
    pub fn create_proposal(
        env: Env,
        proposer: Address,
        proposal_type: ProposalType,
        title: String,
        description: String,
        voting_duration: u64,
        min_votes_required: u32,
    ) -> Result<u64, CommunityError> {
        proposer.require_auth();
        let cfg = CommunityStorage::get_config(&env);
        check_rate_limit(
            &env,
            &proposer,
            RL_OP_PROPOSAL,
            &RateLimitConfig {
                max_calls: cfg.rate_limit_proposal,
                window_seconds: cfg.rate_limit_window,
            },
        )?;
        GovernanceManager::create_proposal(
            &env,
            &proposer,
            proposal_type,
            title,
            description,
            voting_duration,
            min_votes_required,
        )
    }

    /// Cast a vote on an active governance proposal.
    ///
    /// Each address may only vote once per proposal.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `voter` - Address of the user casting the vote.
    /// * `proposal_id` - ID of the proposal to vote on.
    /// * `vote_for` - `true` to vote in favor, `false` to vote against.
    ///
    /// # Errors
    /// Returns [`CommunityError::VotingClosed`] if the voting window has expired.
    /// Returns [`CommunityError::AlreadyVotedOnProposal`] if the voter has already voted.
    ///
    /// # Example
    /// ```ignore
    /// client.vote_on_proposal(&voter, &proposal_id, &vote_for);
    /// ```
    pub fn vote_on_proposal(
        env: Env,
        voter: Address,
        proposal_id: u64,
        vote_for: bool,
    ) -> Result<(), CommunityError> {
        voter.require_auth();
        GovernanceManager::vote_on_proposal(&env, &voter, proposal_id, vote_for)
    }

    /// Finalize a proposal after its voting window closes and compute the outcome.
    ///
    /// This function is permissionless and can be called by anyone once the window has passed.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `proposal_id` - ID of the proposal to finalize.
    ///
    /// # Errors
    /// Returns [`CommunityError::ProposalNotFound`] if `proposal_id` does not exist.
    /// Returns [`CommunityError::VotingClosed`] if the voting window has not yet expired.
    ///
    /// # Example
    /// ```ignore
    /// client.finalize_proposal(&proposal_id);
    /// ```
    pub fn finalize_proposal(env: Env, proposal_id: u64) -> Result<ProposalStatus, CommunityError> {
        GovernanceManager::finalize_proposal(&env, proposal_id)
    }

    /// Retrieve a governance proposal by ID, returning `None` if it does not exist.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `proposal_id` - ID of the proposal to fetch.
    ///
    /// # Example
    /// ```ignore
    /// client.get_proposal(&proposal_id);
    /// ```
    pub fn get_proposal(env: Env, proposal_id: u64) -> Option<CommunityProposal> {
        GovernanceManager::get_proposal(&env, proposal_id)
    }

    /// Return all proposals whose voting window is still open.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    ///
    /// # Example
    /// ```ignore
    /// client.get_active_proposals();
    /// ```
    pub fn get_active_proposals(env: Env) -> Vec<CommunityProposal> {
        GovernanceManager::get_active_proposals(&env)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Analytics Functions
    // ══════════════════════════════════════════════════════════════════════

    /// Return aggregate platform-wide community metrics (post counts, active users, etc.).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    ///
    /// # Example
    /// ```ignore
    /// client.get_community_metrics();
    /// ```
    pub fn get_community_metrics(env: Env) -> CommunityMetrics {
        AnalyticsManager::get_community_metrics(&env)
    }

    /// Return the community participation statistics for a specific user.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `user` - Address of the user to query.
    ///
    /// # Example
    /// ```ignore
    /// client.get_user_stats(&user);
    /// ```
    pub fn get_user_stats(env: Env, user: Address) -> UserCommunityStats {
        AnalyticsManager::get_user_stats(&env, &user)
    }

    /// Compute and return the current reputation score for `user`.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `user` - Address of the user whose reputation to calculate.
    ///
    /// # Example
    /// ```ignore
    /// client.calculate_reputation(&user);
    /// ```
    pub fn calculate_reputation(env: Env, user: Address) -> u32 {
        AnalyticsManager::calculate_reputation(&env, &user)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Admin Functions
    // ══════════════════════════════════════════════════════════════════════

    /// Admin: replace the community configuration with new values.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `admin` - Admin address (must match stored admin).
    /// * `config` - New configuration struct to persist.
    ///
    /// # Errors
    /// Returns [`CommunityError::Unauthorized`] if the caller is not the contract admin.
    ///
    /// # Example
    /// ```ignore
    /// client.update_config(&admin, &config);
    /// ```
    pub fn update_config(
        env: Env,
        admin: Address,
        config: CommunityConfig,
    ) -> Result<(), CommunityError> {
        admin.require_auth();
        CommunityStorage::require_admin(&env, &admin)?;
        CommunityStorage::set_config(&env, &config);
        Ok(())
    }

    /// Return the current community configuration.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    ///
    /// # Example
    /// ```ignore
    /// client.get_config();
    /// ```
    pub fn get_config(env: Env) -> CommunityConfig {
        CommunityStorage::get_config(&env)
    }
}
