use soroban_sdk::{symbol_short, Address, Env};

pub struct CommunityEvents;

impl CommunityEvents {
    // Forum Events
    pub fn emit_post_created(env: &Env, author: &Address, post_id: u64) {
        env.events()
            .publish((symbol_short!("post_new"),), (author, post_id));
    }

    pub fn emit_reply_created(env: &Env, author: &Address, post_id: u64, reply_id: u64) {
        env.events()
            .publish((symbol_short!("reply_new"),), (author, post_id, reply_id));
    }

    pub fn emit_solution_marked(env: &Env, post_id: u64, reply_id: u64) {
        env.events()
            .publish((symbol_short!("solution"),), (post_id, reply_id));
    }

    // Mentorship Events
    pub fn emit_mentor_registered(env: &Env, mentor: &Address) {
        env.events()
            .publish((symbol_short!("mntr_reg"),), (mentor,));
    }

    pub fn emit_mentorship_requested(env: &Env, mentee: &Address, mentor: &Address, request_id: u64) {
        env.events()
            .publish((symbol_short!("mntr_req"),), (mentee, mentor, request_id));
    }

    pub fn emit_mentorship_started(env: &Env, request_id: u64) {
        env.events()
            .publish((symbol_short!("mntr_strt"),), (request_id,));
    }

    pub fn emit_session_completed(env: &Env, session_id: u64, mentor: &Address, mentee: &Address) {
        env.events()
            .publish((symbol_short!("session"),), (session_id, mentor, mentee));
    }

    // Contribution Events
    pub fn emit_contribution_submitted(env: &Env, contributor: &Address, contribution_id: u64) {
        env.events()
            .publish((symbol_short!("contrib"),), (contributor, contribution_id));
    }

    pub fn emit_contribution_approved(env: &Env, contribution_id: u64) {
        env.events()
            .publish((symbol_short!("approved"),), (contribution_id,));
    }

    // Event Events
    pub fn emit_event_created(env: &Env, organizer: &Address, event_id: u64) {
        env.events()
            .publish((symbol_short!("event_new"),), (organizer, event_id));
    }

    pub fn emit_event_registered(env: &Env, user: &Address, event_id: u64) {
        env.events()
            .publish((symbol_short!("event_reg"),), (user, event_id));
    }

    pub fn emit_event_completed(env: &Env, event_id: u64) {
        env.events()
            .publish((symbol_short!("event_end"),), (event_id,));
    }

    // Moderation Events
    pub fn emit_content_reported(env: &Env, reporter: &Address, report_id: u64) {
        env.events()
            .publish((symbol_short!("report"),), (reporter, report_id));
    }

    pub fn emit_moderator_action(env: &Env, moderator: &Address, action_id: u64, target: &Address) {
        env.events()
            .publish((symbol_short!("mod_act"),), (moderator, action_id, target));
    }

    // Governance Events
    pub fn emit_proposal_created(env: &Env, proposer: &Address, proposal_id: u64) {
        env.events()
            .publish((symbol_short!("proposal"),), (proposer, proposal_id));
    }

    pub fn emit_vote_cast(env: &Env, voter: &Address, proposal_id: u64, vote_for: bool) {
        env.events()
            .publish((symbol_short!("vote"),), (voter, proposal_id, vote_for));
    }

    // System Events
    pub fn emit_initialized(env: &Env, admin: &Address) {
        env.events()
            .publish((symbol_short!("init"),), (admin,));
    }
}
