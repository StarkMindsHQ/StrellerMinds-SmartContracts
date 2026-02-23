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

use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

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

#[contract]
pub struct Community;

#[contractimpl]
impl Community {
    // ══════════════════════════════════════════════════════════════════════
    //  Initialization
    // ══════════════════════════════════════════════════════════════════════

    pub fn initialize(env: Env, admin: Address) -> Result<(), Error> {
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

    pub fn create_post(
        env: Env,
        author: Address,
        category: ForumCategory,
        title: String,
        content: String,
        tags: Vec<String>,
        course_id: String,
    ) -> Result<u64, Error> {
        author.require_auth();
        ForumManager::create_post(&env, &author, category, title, content, tags, course_id)
    }

    pub fn create_reply(
        env: Env,
        author: Address,
        post_id: u64,
        content: String,
        parent_reply_id: u64,
    ) -> Result<u64, Error> {
        author.require_auth();
        ForumManager::create_reply(&env, &author, post_id, content, parent_reply_id)
    }

    pub fn mark_solution(
        env: Env,
        post_author: Address,
        post_id: u64,
        reply_id: u64,
    ) -> Result<(), Error> {
        post_author.require_auth();
        ForumManager::mark_solution(&env, &post_author, post_id, reply_id)
    }

    pub fn vote_post(env: Env, voter: Address, post_id: u64, upvote: bool) -> Result<(), Error> {
        voter.require_auth();
        ForumManager::vote_post(&env, &voter, post_id, upvote)
    }

    pub fn get_post(env: Env, post_id: u64) -> Option<ForumPost> {
        ForumManager::get_post(&env, post_id)
    }

    pub fn get_post_replies(env: Env, post_id: u64) -> Vec<ForumReply> {
        ForumManager::get_post_replies(&env, post_id)
    }

    pub fn get_category_posts(env: Env, category: ForumCategory, limit: u32) -> Vec<ForumPost> {
        ForumManager::get_category_posts(&env, category, limit)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Mentorship Functions
    // ══════════════════════════════════════════════════════════════════════

    pub fn register_mentor(
        env: Env,
        mentor: Address,
        expertise_areas: Vec<String>,
        expertise_level: MentorExpertise,
        max_mentees: u32,
        bio: String,
    ) -> Result<(), Error> {
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

    pub fn request_mentorship(
        env: Env,
        mentee: Address,
        mentor: Address,
        topic: String,
        message: String,
    ) -> Result<u64, Error> {
        mentee.require_auth();
        MentorshipManager::request_mentorship(&env, &mentee, &mentor, topic, message)
    }

    pub fn accept_mentorship(env: Env, mentor: Address, request_id: u64) -> Result<(), Error> {
        mentor.require_auth();
        MentorshipManager::accept_mentorship(&env, &mentor, request_id)
    }

    pub fn complete_session(
        env: Env,
        mentor: Address,
        request_id: u64,
        duration: u64,
        notes: String,
    ) -> Result<u64, Error> {
        mentor.require_auth();
        MentorshipManager::complete_session(&env, &mentor, request_id, duration, notes)
    }

    pub fn rate_session(
        env: Env,
        mentee: Address,
        session_id: u64,
        rating: u32,
    ) -> Result<(), Error> {
        mentee.require_auth();
        MentorshipManager::rate_session(&env, &mentee, session_id, rating)
    }

    pub fn get_mentor_profile(env: Env, mentor: Address) -> Option<MentorProfile> {
        MentorshipManager::get_mentor_profile(&env, &mentor)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Knowledge Base Functions
    // ══════════════════════════════════════════════════════════════════════

    pub fn submit_contribution(
        env: Env,
        contributor: Address,
        contribution_type: ContributionType,
        title: String,
        content: String,
        category: ForumCategory,
        tags: Vec<String>,
    ) -> Result<u64, Error> {
        contributor.require_auth();
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

    pub fn review_contribution(
        env: Env,
        moderator: Address,
        contribution_id: u64,
        approve: bool,
    ) -> Result<(), Error> {
        moderator.require_auth();
        KnowledgeManager::review_contribution(&env, &moderator, contribution_id, approve)
    }

    pub fn vote_contribution(
        env: Env,
        voter: Address,
        contribution_id: u64,
        upvote: bool,
    ) -> Result<(), Error> {
        voter.require_auth();
        KnowledgeManager::vote_contribution(&env, &voter, contribution_id, upvote)
    }

    pub fn get_contribution(env: Env, contribution_id: u64) -> Option<KnowledgeContribution> {
        KnowledgeManager::get_contribution(&env, contribution_id)
    }

    pub fn get_user_contributions(env: Env, user: Address) -> Vec<KnowledgeContribution> {
        KnowledgeManager::get_user_contributions(&env, &user)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Event Functions
    // ══════════════════════════════════════════════════════════════════════

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
    ) -> Result<u64, Error> {
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

    pub fn register_for_event(env: Env, user: Address, event_id: u64) -> Result<(), Error> {
        user.require_auth();
        EventManager::register_for_event(&env, &user, event_id)
    }

    pub fn mark_attendance(
        env: Env,
        organizer: Address,
        event_id: u64,
        user: Address,
    ) -> Result<(), Error> {
        organizer.require_auth();
        EventManager::mark_attendance(&env, &organizer, event_id, &user)
    }

    pub fn complete_event(env: Env, organizer: Address, event_id: u64) -> Result<(), Error> {
        organizer.require_auth();
        EventManager::complete_event(&env, &organizer, event_id)
    }

    pub fn submit_event_feedback(
        env: Env,
        user: Address,
        event_id: u64,
        rating: u32,
    ) -> Result<(), Error> {
        user.require_auth();
        EventManager::submit_feedback(&env, &user, event_id, rating)
    }

    pub fn get_event(env: Env, event_id: u64) -> Option<CommunityEvent> {
        EventManager::get_event(&env, event_id)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Moderation Functions
    // ══════════════════════════════════════════════════════════════════════

    pub fn add_moderator(
        env: Env,
        admin: Address,
        moderator: Address,
        role: ModeratorRole,
    ) -> Result<(), Error> {
        admin.require_auth();
        ModerationManager::add_moderator(&env, &admin, &moderator, role)
    }

    pub fn report_content(
        env: Env,
        reporter: Address,
        content_type: String,
        content_id: u64,
        reason: ReportReason,
        description: String,
    ) -> Result<u64, Error> {
        reporter.require_auth();
        ModerationManager::report_content(
            &env,
            &reporter,
            content_type,
            content_id,
            reason,
            description,
        )
    }

    pub fn resolve_report(
        env: Env,
        moderator: Address,
        report_id: u64,
        action: String,
    ) -> Result<(), Error> {
        moderator.require_auth();
        ModerationManager::resolve_report(&env, &moderator, report_id, action)
    }

    pub fn get_pending_reports(env: Env) -> Vec<ContentReport> {
        ModerationManager::get_pending_reports(&env)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Governance Functions
    // ══════════════════════════════════════════════════════════════════════

    pub fn create_proposal(
        env: Env,
        proposer: Address,
        proposal_type: ProposalType,
        title: String,
        description: String,
        voting_duration: u64,
        min_votes_required: u32,
    ) -> Result<u64, Error> {
        proposer.require_auth();
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

    pub fn vote_on_proposal(
        env: Env,
        voter: Address,
        proposal_id: u64,
        vote_for: bool,
    ) -> Result<(), Error> {
        voter.require_auth();
        GovernanceManager::vote_on_proposal(&env, &voter, proposal_id, vote_for)
    }

    pub fn finalize_proposal(env: Env, proposal_id: u64) -> Result<ProposalStatus, Error> {
        GovernanceManager::finalize_proposal(&env, proposal_id)
    }

    pub fn get_proposal(env: Env, proposal_id: u64) -> Option<CommunityProposal> {
        GovernanceManager::get_proposal(&env, proposal_id)
    }

    pub fn get_active_proposals(env: Env) -> Vec<CommunityProposal> {
        GovernanceManager::get_active_proposals(&env)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Analytics Functions
    // ══════════════════════════════════════════════════════════════════════

    pub fn get_community_metrics(env: Env) -> CommunityMetrics {
        AnalyticsManager::get_community_metrics(&env)
    }

    pub fn get_user_stats(env: Env, user: Address) -> UserCommunityStats {
        AnalyticsManager::get_user_stats(&env, &user)
    }

    pub fn calculate_reputation(env: Env, user: Address) -> u32 {
        AnalyticsManager::calculate_reputation(&env, &user)
    }

    // ══════════════════════════════════════════════════════════════════════
    //  Admin Functions
    // ══════════════════════════════════════════════════════════════════════

    pub fn update_config(env: Env, admin: Address, config: CommunityConfig) -> Result<(), Error> {
        admin.require_auth();
        CommunityStorage::require_admin(&env, &admin)?;
        CommunityStorage::set_config(&env, &config);
        Ok(())
    }

    pub fn get_config(env: Env) -> CommunityConfig {
        CommunityStorage::get_config(&env)
    }
}
