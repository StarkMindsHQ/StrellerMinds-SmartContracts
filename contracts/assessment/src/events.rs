use shared::event_schema::{
    AccessControlEventData, AssessmentCreatedEvent, AssessmentEventData, AssessmentPublishedEvent,
    ContractInitializedEvent, IntegrityEventData, PlagiarismFlaggedEvent, QuestionAddedEvent,
    ScheduleCreatedEvent, SubmissionGradedEvent, SubmissionReceivedEvent,
};
use shared::{emit_access_control_event, emit_assessment_event};
use soroban_sdk::{symbol_short, Address, BytesN, Env, Symbol};

pub struct AssessmentEvents;

impl AssessmentEvents {
    pub fn emit_initialized(env: &Env, admin: &Address) {
        emit_access_control_event!(
            env,
            symbol_short!("assess"),
            admin.clone(),
            AccessControlEventData::ContractInitialized(ContractInitializedEvent {
                admin: admin.clone(),
            })
        );
    }

    pub fn emit_assessment_created(env: &Env, id: u64, instructor: &Address, course: &Symbol) {
        emit_assessment_event!(
            env,
            symbol_short!("assess"),
            instructor.clone(),
            AssessmentEventData::AssessmentCreated(AssessmentCreatedEvent {
                id,
                instructor: instructor.clone(),
                course: course.clone(),
            })
        );
    }

    pub fn emit_assessment_published(env: &Env, id: u64) {
        let contract_addr = env.current_contract_address();
        emit_assessment_event!(
            env,
            symbol_short!("assess"),
            contract_addr,
            AssessmentEventData::AssessmentPublished(AssessmentPublishedEvent { id })
        );
    }

    pub fn emit_question_added(env: &Env, assessment_id: u64, question_id: u64) {
        let contract_addr = env.current_contract_address();
        emit_assessment_event!(
            env,
            symbol_short!("assess"),
            contract_addr,
            AssessmentEventData::QuestionAdded(QuestionAddedEvent { assessment_id, question_id })
        );
    }

    pub fn emit_submission_received(env: &Env, submission_id: &BytesN<32>, assessment_id: u64) {
        let contract_addr = env.current_contract_address(); // Actor here is tricky without passing it
        emit_assessment_event!(
            env,
            symbol_short!("assess"),
            contract_addr,
            AssessmentEventData::SubmissionReceived(SubmissionReceivedEvent {
                submission_id: submission_id.clone(),
                assessment_id,
            })
        );
    }

    pub fn emit_submission_graded(
        env: &Env,
        submission_id: &BytesN<32>,
        score: u32,
        max_score: u32,
        passed: bool,
    ) {
        let contract_addr = env.current_contract_address();
        emit_assessment_event!(
            env,
            symbol_short!("assess"),
            contract_addr,
            AssessmentEventData::SubmissionGraded(SubmissionGradedEvent {
                submission_id: submission_id.clone(),
                score,
                max_score,
                passed,
            })
        );
    }

    pub fn emit_plagiarism_flagged(
        env: &Env,
        submission_id: &BytesN<32>,
        score: u32,
        flagged: bool,
    ) {
        let contract_addr = env.current_contract_address();
        emit_assessment_event!(
            env,
            symbol_short!("assess"),
            contract_addr,
            AssessmentEventData::PlagiarismFlagged(PlagiarismFlaggedEvent {
                submission_id: submission_id.clone(),
                score,
                flagged,
            })
        );
    }

    pub fn emit_integrity_event(
        env: &Env,
        submission_id: &BytesN<32>,
        flag: &Symbol,
        severity: u32,
    ) {
        let contract_addr = env.current_contract_address();
        emit_assessment_event!(
            env,
            symbol_short!("assess"),
            contract_addr,
            AssessmentEventData::IntegrityEvent(IntegrityEventData {
                submission_id: submission_id.clone(),
                flag: flag.clone(),
                severity,
            })
        );
    }

    pub fn emit_schedule_created(env: &Env, assessment_id: u64) {
        let contract_addr = env.current_contract_address();
        emit_assessment_event!(
            env,
            symbol_short!("assess"),
            contract_addr,
            AssessmentEventData::ScheduleCreated(ScheduleCreatedEvent { assessment_id })
        );
    }
}
