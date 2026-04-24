use shared::event_schema::{
    AccessControlEventData, CommunityEventData, ContentReportedEvent, ContractInitializedEvent,
    ContributionApprovedEvent, ContributionSubmittedEvent, EventCompletedEvent,
    EventRegisteredEvent, GovernanceEventData, InternalEventCreatedEvent, MentorRegisteredEvent,
    MentorshipEventData, MentorshipRequestedEvent, MentorshipSessionCompletedEvent,
    MentorshipStartedEvent, ModeratorActionEvent, PostCreatedEvent, ProposalCreatedEvent,
    ReplyCreatedEvent, SolutionMarkedEvent, VoteCastEvent,
};
use shared::{
    emit_access_control_event, emit_community_event, emit_governance_event, emit_mentorship_event,
};
use soroban_sdk::{symbol_short, Address, Env};

pub struct CommunityEvents;

impl CommunityEvents {
    // Forum Events
    pub fn emit_post_created(env: &Env, author: &Address, post_id: u64) {
        emit_community_event!(
            env,
            symbol_short!("comm"),
            author.clone(),
            CommunityEventData::PostCreated(PostCreatedEvent { author: author.clone(), post_id })
        );
    }

    pub fn emit_reply_created(env: &Env, author: &Address, post_id: u64, reply_id: u64) {
        emit_community_event!(
            env,
            symbol_short!("comm"),
            author.clone(),
            CommunityEventData::ReplyCreated(ReplyCreatedEvent {
                author: author.clone(),
                post_id,
                reply_id,
            })
        );
    }

    pub fn emit_solution_marked(env: &Env, post_id: u64, reply_id: u64) {
        // Use a generic actor if not provided, or better, pass one.
        // For now, using the contract itself as actor placeholder if not available.
        let contract_addr = env.current_contract_address();
        emit_community_event!(
            env,
            symbol_short!("comm"),
            contract_addr,
            CommunityEventData::SolutionMarked(SolutionMarkedEvent { post_id, reply_id })
        );
    }

    // Mentorship Events
    pub fn emit_mentor_registered(env: &Env, mentor: &Address) {
        emit_mentorship_event!(
            env,
            symbol_short!("comm"),
            mentor.clone(),
            MentorshipEventData::MentorRegistered(MentorRegisteredEvent { mentor: mentor.clone() })
        );
    }

    pub fn emit_mentorship_requested(
        env: &Env,
        mentee: &Address,
        mentor: &Address,
        request_id: u64,
    ) {
        emit_mentorship_event!(
            env,
            symbol_short!("comm"),
            mentee.clone(),
            MentorshipEventData::MentorshipRequested(MentorshipRequestedEvent {
                mentee: mentee.clone(),
                mentor: mentor.clone(),
                request_id,
            })
        );
    }

    pub fn emit_mentorship_started(env: &Env, request_id: u64) {
        let contract_addr = env.current_contract_address();
        emit_mentorship_event!(
            env,
            symbol_short!("comm"),
            contract_addr,
            MentorshipEventData::MentorshipStarted(MentorshipStartedEvent { request_id })
        );
    }

    pub fn emit_session_completed(env: &Env, session_id: u64, mentor: &Address, mentee: &Address) {
        emit_mentorship_event!(
            env,
            symbol_short!("comm"),
            mentor.clone(),
            MentorshipEventData::SessionCompleted(MentorshipSessionCompletedEvent {
                session_id,
                mentor: mentor.clone(),
                mentee: mentee.clone(),
            })
        );
    }

    // Contribution Events
    pub fn emit_contribution_submitted(env: &Env, contributor: &Address, contribution_id: u64) {
        emit_community_event!(
            env,
            symbol_short!("comm"),
            contributor.clone(),
            CommunityEventData::ContributionSubmitted(ContributionSubmittedEvent {
                contributor: contributor.clone(),
                contribution_id,
            })
        );
    }

    pub fn emit_contribution_approved(env: &Env, contribution_id: u64) {
        let contract_addr = env.current_contract_address();
        emit_community_event!(
            env,
            symbol_short!("comm"),
            contract_addr,
            CommunityEventData::ContributionApproved(ContributionApprovedEvent { contribution_id })
        );
    }

    // Event Events
    pub fn emit_event_created(env: &Env, organizer: &Address, event_id: u64) {
        emit_community_event!(
            env,
            symbol_short!("comm"),
            organizer.clone(),
            CommunityEventData::EventCreated(InternalEventCreatedEvent {
                organizer: organizer.clone(),
                event_id,
            })
        );
    }

    pub fn emit_event_registered(env: &Env, user: &Address, event_id: u64) {
        emit_community_event!(
            env,
            symbol_short!("comm"),
            user.clone(),
            CommunityEventData::EventRegistered(EventRegisteredEvent {
                user: user.clone(),
                event_id,
            })
        );
    }

    pub fn emit_event_completed(env: &Env, event_id: u64) {
        let contract_addr = env.current_contract_address();
        emit_community_event!(
            env,
            symbol_short!("comm"),
            contract_addr,
            CommunityEventData::EventCompleted(EventCompletedEvent { event_id })
        );
    }

    // Moderation Events
    pub fn emit_content_reported(env: &Env, reporter: &Address, report_id: u64) {
        emit_community_event!(
            env,
            symbol_short!("comm"),
            reporter.clone(),
            CommunityEventData::ContentReported(ContentReportedEvent {
                reporter: reporter.clone(),
                report_id,
            })
        );
    }

    pub fn emit_moderator_action(env: &Env, moderator: &Address, action_id: u64, target: &Address) {
        emit_community_event!(
            env,
            symbol_short!("comm"),
            moderator.clone(),
            CommunityEventData::ModeratorAction(ModeratorActionEvent {
                moderator: moderator.clone(),
                action_id,
                target: target.clone(),
            })
        );
    }

    // Governance Events
    pub fn emit_proposal_created(env: &Env, proposer: &Address, proposal_id: u64) {
        emit_governance_event!(
            env,
            symbol_short!("comm"),
            proposer.clone(),
            GovernanceEventData::ProposalCreated(ProposalCreatedEvent {
                proposer: proposer.clone(),
                proposal_id,
            })
        );
    }

    pub fn emit_vote_cast(env: &Env, voter: &Address, proposal_id: u64, vote_for: bool) {
        emit_governance_event!(
            env,
            symbol_short!("comm"),
            voter.clone(),
            GovernanceEventData::VoteCast(VoteCastEvent {
                voter: voter.clone(),
                proposal_id,
                vote_for,
            })
        );
    }

    // System Events
    pub fn emit_initialized(env: &Env, admin: &Address) {
        emit_access_control_event!(
            env,
            symbol_short!("comm"),
            admin.clone(),
            AccessControlEventData::ContractInitialized(ContractInitializedEvent {
                admin: admin.clone(),
            })
        );
    }
}
