use soroban_sdk::{symbol_short, Address, BytesN, Env, Symbol};

pub struct AssessmentEvents;

impl AssessmentEvents {
    pub fn emit_initialized(env: &Env, admin: &Address) {
        env.events().publish(
            (symbol_short!("assessment"), symbol_short!("initialized")),
            admin,
        );
    }

    pub fn emit_assessment_created(env: &Env, id: u64, instructor: &Address, course: &Symbol) {
        env.events().publish(
            (symbol_short!("assessment"), symbol_short!("created")),
            (id, instructor.clone(), course.clone()),
        );
    }

    pub fn emit_assessment_published(env: &Env, id: u64) {
        env.events().publish(
            (symbol_short!("assessment"), symbol_short!("published")),
            id,
        );
    }

    pub fn emit_question_added(env: &Env, assessment_id: u64, question_id: u64) {
        env.events().publish(
            (symbol_short!("assessment"), symbol_short!("q_added")),
            (assessment_id, question_id),
        );
    }

    pub fn emit_submission_received(env: &Env, submission_id: &BytesN<32>, assessment_id: u64) {
        env.events().publish(
            (symbol_short!("assessment"), symbol_short!("submitted")),
            (submission_id.clone(), assessment_id),
        );
    }

    pub fn emit_submission_graded(
        env: &Env,
        submission_id: &BytesN<32>,
        score: u32,
        max_score: u32,
        passed: bool,
    ) {
        env.events().publish(
            (symbol_short!("assessment"), symbol_short!("graded")),
            (submission_id.clone(), score, max_score, passed),
        );
    }

    pub fn emit_plagiarism_flagged(
        env: &Env,
        submission_id: &BytesN<32>,
        score: u32,
        flagged: bool,
    ) {
        env.events().publish(
            (symbol_short!("assessment"), symbol_short!("plagiarism")),
            (submission_id.clone(), score, flagged),
        );
    }

    pub fn emit_integrity_event(
        env: &Env,
        submission_id: &BytesN<32>,
        flag: &Symbol,
        severity: u32,
    ) {
        env.events().publish(
            (symbol_short!("assessment"), symbol_short!("integrity")),
            (submission_id.clone(), flag.clone(), severity),
        );
    }

    pub fn emit_schedule_created(env: &Env, assessment_id: u64) {
        env.events().publish(
            (symbol_short!("assessment"), symbol_short!("scheduled")),
            assessment_id,
        );
    }
}

