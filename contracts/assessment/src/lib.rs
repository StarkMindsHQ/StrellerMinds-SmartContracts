pub mod errors;
pub mod events;
pub mod grading;
pub mod types;

use errors::AssessmentError;
use events::AssessmentEvents;
use grading::GradingEngine;
use types::*;

#[cfg(test)]
mod test;

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Map, Symbol, Vec};

#[contract]
pub struct Assessment;

fn get_admin(env: &Env) -> Address {
    env.storage()
        .instance()
        .get(&DataKey::Admin)
        .expect("admin not set")
}

fn require_admin(env: &Env, actor: &Address) -> Result<(), AssessmentError> {
    let admin = get_admin(env);
    if admin != *actor {
        return Err(AssessmentError::Unauthorized);
    }
    Ok(())
}

fn get_next_assessment_id(env: &Env) -> u64 {
    let current: u64 = env
        .storage()
        .instance()
        .get(&DataKey::AssessmentCounter)
        .unwrap_or(0);
    let next = current + 1;
    env.storage()
        .instance()
        .set(&DataKey::AssessmentCounter, &next);
    next
}

fn get_next_question_id(env: &Env) -> u64 {
    let current: u64 = env
        .storage()
        .instance()
        .get(&DataKey::QuestionCounter)
        .unwrap_or(0);
    let next = current + 1;
    env.storage()
        .instance()
        .set(&DataKey::QuestionCounter, &next);
    next
}

fn get_assessment(env: &Env, assessment_id: u64) -> Result<AssessmentMetadata, AssessmentError> {
    env.storage()
        .persistent()
        .get(&DataKey::Assessment(assessment_id))
        .ok_or(AssessmentError::AssessmentNotFound)
}

fn put_assessment(env: &Env, meta: &AssessmentMetadata) {
    env.storage()
        .persistent()
        .set(&DataKey::Assessment(meta.assessment_id), meta);
}

fn get_questions_for_assessment(env: &Env, assessment_id: u64) -> Vec<Question> {
    let ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&DataKey::AssessmentQuestions(assessment_id))
        .unwrap_or(Vec::new(env));

    let mut result = Vec::new(env);
    for qid in ids.iter() {
        if let Some(q) = env
            .storage()
            .persistent()
            .get::<_, Question>(&DataKey::Question(qid))
        {
            result.push_back(q);
        }
    }
    result
}

fn within_schedule(env: &Env, assessment_id: u64) -> bool {
    let schedule: Option<ScheduleConfig> = env
        .storage()
        .persistent()
        .get(&DataKey::Schedule(assessment_id));
    if let Some(s) = schedule {
        let now = env.ledger().timestamp();
        now >= s.start_time && now <= s.end_time
    } else {
        true
    }
}

fn get_accommodation(env: &Env, student: &Address) -> Option<AccommodationConfig> {
    env.storage()
        .persistent()
        .get(&DataKey::Accommodation(student.clone()))
}

fn get_effective_time_limit(env: &Env, config: &AssessmentConfig, student: &Address) -> u64 {
    if let Some(ac) = get_accommodation(env, student) {
        let bonus = config
            .time_limit_seconds
            .saturating_mul(ac.extra_time_percent as u64)
            / 100;
        config.time_limit_seconds.saturating_add(bonus)
    } else {
        config.time_limit_seconds
    }
}

fn get_student_attempts(env: &Env, student: &Address, assessment_id: u64) -> u32 {
    let key = DataKey::StudentAssessmentSubmissions(student.clone(), assessment_id);
    let ids: Vec<BytesN<32>> = env.storage().persistent().get(&key).unwrap_or(Vec::new(env));
    ids.len() as u32
}

fn append_student_submission(
    env: &Env,
    student: &Address,
    assessment_id: u64,
    submission_id: &BytesN<32>,
) {
    let key = DataKey::StudentAssessmentSubmissions(student.clone(), assessment_id);
    let mut ids: Vec<BytesN<32>> =
        env.storage().persistent().get(&key).unwrap_or(Vec::new(env));
    ids.push_back(submission_id.clone());
    env.storage().persistent().set(&key, &ids);
}

fn get_submission(env: &Env, submission_id: &BytesN<32>) -> Result<Submission, AssessmentError> {
    env.storage()
        .persistent()
        .get(&DataKey::Submission(submission_id.clone()))
        .ok_or(AssessmentError::SubmissionNotFound)
}

fn put_submission(env: &Env, submission: &Submission) {
    env.storage()
        .persistent()
        .set(&DataKey::Submission(submission.submission_id.clone()), submission);
}

fn get_or_init_adaptive_state(env: &Env, student: &Address, assessment_id: u64) -> AdaptiveState {
    env.storage()
        .persistent()
        .get(&DataKey::Adaptive(student.clone(), assessment_id))
        .unwrap_or(AdaptiveState {
            current_difficulty: 3,
            completed_questions: Vec::new(env),
        })
}

fn put_adaptive_state(
    env: &Env,
    student: &Address,
    assessment_id: u64,
    state: &AdaptiveState,
) {
    env.storage()
        .persistent()
        .set(&DataKey::Adaptive(student.clone(), assessment_id), state);
}

fn get_integration(env: &Env) -> IntegrationConfig {
    env.storage()
        .instance()
        .get(&DataKey::Integration)
        .unwrap_or(IntegrationConfig {
            analytics_contract: None,
            progress_contract: None,
            security_monitor_contract: None,
        })
}

#[contractimpl]
impl Assessment {
    // Initialization & configuration
    pub fn initialize(env: Env, admin: Address) -> Result<(), AssessmentError> {
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(AssessmentError::AlreadyInitialized);
        }
        admin.require_auth();
        env.storage().instance().set(&DataKey::Admin, &admin);
        env.storage().instance().set(
            &DataKey::Integration,
            &IntegrationConfig {
                analytics_contract: None,
                progress_contract: None,
                security_monitor_contract: None,
            },
        );
        AssessmentEvents::emit_initialized(&env, &admin);
        Ok(())
    }

    pub fn set_integration(
        env: Env,
        admin: Address,
        analytics: Option<Address>,
        progress: Option<Address>,
        security_monitor: Option<Address>,
    ) -> Result<(), AssessmentError> {
        require_admin(&env, &admin)?;
        let config = IntegrationConfig {
            analytics_contract: analytics,
            progress_contract: progress,
            security_monitor_contract: security_monitor,
        };
        env.storage().instance().set(&DataKey::Integration, &config);
        Ok(())
    }

    // Assessment & question management

    pub fn create_assessment(
        env: Env,
        instructor: Address,
        course_id: Symbol,
        module_id: Symbol,
        config: AssessmentConfig,
    ) -> Result<u64, AssessmentError> {
        instructor.require_auth();
        if config.max_attempts == 0 || config.pass_score == 0 {
            return Err(AssessmentError::InvalidConfig);
        }

        let id = get_next_assessment_id(&env);
        let meta = AssessmentMetadata {
            assessment_id: id,
            course_id: course_id.clone(),
            module_id,
            instructor: instructor.clone(),
            config,
            published: false,
        };
        put_assessment(&env, &meta);
        env.storage()
            .persistent()
            .set(&DataKey::AssessmentQuestions(id), &Vec::<u64>::new(&env));
        AssessmentEvents::emit_assessment_created(&env, id, &instructor, &course_id);
        Ok(id)
    }

    pub fn publish_assessment(
        env: Env,
        admin: Address,
        assessment_id: u64,
    ) -> Result<(), AssessmentError> {
        require_admin(&env, &admin)?;
        let mut meta = get_assessment(&env, assessment_id)?;
        meta.published = true;
        put_assessment(&env, &meta);
        AssessmentEvents::emit_assessment_published(&env, assessment_id);
        Ok(())
    }

    pub fn add_question(
        env: Env,
        admin: Address,
        assessment_id: u64,
        question_type: QuestionType,
        max_score: u32,
        difficulty: u32,
        content_hash: BytesN<32>,
        options: Vec<QuestionOption>,
        answer_key: AnswerKey,
    ) -> Result<u64, AssessmentError> {
        require_admin(&env, &admin)?;
        let _ = get_assessment(&env, assessment_id)?;

        if max_score == 0 || difficulty == 0 {
            return Err(AssessmentError::InvalidQuestionType);
        }

        let qid = get_next_question_id(&env);
        let q = Question {
            question_id: qid,
            assessment_id,
            question_type,
            max_score,
            difficulty,
            content_hash,
            options,
            answer_key,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Question(qid), &q);

        let mut ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::AssessmentQuestions(assessment_id))
            .unwrap_or(Vec::new(&env));
        ids.push_back(qid);
        env.storage()
            .persistent()
            .set(&DataKey::AssessmentQuestions(assessment_id), &ids);

        AssessmentEvents::emit_question_added(&env, assessment_id, qid);
        Ok(qid)
    }

    pub fn get_assessment_metadata(
        env: Env,
        assessment_id: u64,
    ) -> Option<AssessmentMetadata> {
        env.storage()
            .persistent()
            .get(&DataKey::Assessment(assessment_id))
    }

    // Scheduling & accessibility

    pub fn set_schedule(
        env: Env,
        admin: Address,
        assessment_id: u64,
        start_time: u64,
        end_time: u64,
        time_zone_offset_minutes: i32,
        proctoring_provider: Option<Symbol>,
    ) -> Result<(), AssessmentError> {
        require_admin(&env, &admin)?;
        if end_time <= start_time {
            return Err(AssessmentError::InvalidSchedule);
        }
        let _ = get_assessment(&env, assessment_id)?;
        let schedule = ScheduleConfig {
            assessment_id,
            start_time,
            end_time,
            time_zone_offset_minutes,
            proctoring_provider,
        };
        env.storage()
            .persistent()
            .set(&DataKey::Schedule(assessment_id), &schedule);
        AssessmentEvents::emit_schedule_created(&env, assessment_id);
        Ok(())
    }

    pub fn set_accommodation(
        env: Env,
        admin: Address,
        student: Address,
        config: AccommodationConfig,
    ) -> Result<(), AssessmentError> {
        require_admin(&env, &admin)?;
        env.storage()
            .persistent()
            .set(&DataKey::Accommodation(student), &config);
        Ok(())
    }

    pub fn get_accommodation_for_student(
        env: Env,
        student: Address,
    ) -> Option<AccommodationConfig> {
        get_accommodation(&env, &student)
    }

    // Adaptive testing

    pub fn get_next_question(
        env: Env,
        student: Address,
        assessment_id: u64,
    ) -> Result<Option<Question>, AssessmentError> {
        let meta = get_assessment(&env, assessment_id)?;
        if !meta.config.is_adaptive {
            return Err(AssessmentError::AdaptiveNotEnabled);
        }

        let state = get_or_init_adaptive_state(&env, &student, assessment_id);
        let questions = get_questions_for_assessment(&env, assessment_id);

        let mut chosen: Option<Question> = None;
        for q in questions.iter() {
            let mut already_done = false;
            for done_id in state.completed_questions.iter() {
                if q.question_id == done_id {
                    already_done = true;
                    break;
                }
            }
            if !already_done && q.difficulty == state.current_difficulty {
                chosen = Some(q.clone());
                break;
            }
        }

        Ok(chosen)
    }

    pub fn update_adaptive_state(
        env: Env,
        student: Address,
        assessment_id: u64,
        question_id: u64,
        was_correct: bool,
    ) -> Result<(), AssessmentError> {
        let meta = get_assessment(&env, assessment_id)?;
        if !meta.config.is_adaptive {
            return Err(AssessmentError::AdaptiveNotEnabled);
        }

        let mut state = get_or_init_adaptive_state(&env, &student, assessment_id);
        state.completed_questions.push_back(question_id);

        if was_correct && state.current_difficulty < 5 {
            state.current_difficulty += 1;
        } else if !was_correct && state.current_difficulty > 1 {
            state.current_difficulty -= 1;
        }

        put_adaptive_state(&env, &student, assessment_id, &state);
        Ok(())
    }

    // Submissions, grading, and integrity

    pub fn start_submission(
        env: Env,
        student: Address,
        assessment_id: u64,
    ) -> Result<BytesN<32>, AssessmentError> {
        student.require_auth();
        let meta = get_assessment(&env, assessment_id)?;
        if !meta.published {
            return Err(AssessmentError::AssessmentNotPublished);
        }
        if !within_schedule(&env, assessment_id) {
            return Err(AssessmentError::AssessmentClosed);
        }

        let attempts = get_student_attempts(&env, &student, assessment_id);
        let max_attempts = {
            let base = meta.config.max_attempts;
            if let Some(ac) = get_accommodation(&env, &student) {
                base.saturating_add(ac.extra_attempts)
            } else {
                base
            }
        };
        if attempts >= max_attempts {
            return Err(AssessmentError::MaxAttemptsReached);
        }

        let sid = env.crypto().sha256(&Vec::from_array(&env, &student.clone().to_xdr(&env)));
        let submission = Submission {
            submission_id: sid.clone(),
            assessment_id,
            student: student.clone(),
            attempt: attempts + 1,
            started_at: env.ledger().timestamp(),
            submitted_at: 0,
            score: 0,
            max_score: 0,
            passed: false,
            status: SubmissionStatus::InProgress,
            answers: Vec::new(&env),
            integrity: IntegrityMetadata {
                plagiarism_score: None,
                plagiarism_flag: false,
                integrity_flags: Vec::new(&env),
                proctoring_evidence_hash: None,
            },
        };
        put_submission(&env, &submission);
        append_student_submission(&env, &student, assessment_id, &sid);
        Ok(sid)
    }

    pub fn submit_answers(
        env: Env,
        student: Address,
        submission_id: BytesN<32>,
        answers: Vec<SubmittedAnswer>,
    ) -> Result<Submission, AssessmentError> {
        student.require_auth();
        let mut submission = get_submission(&env, &submission_id)?;
        if submission.student != student {
            return Err(AssessmentError::Unauthorized);
        }
        if let SubmissionStatus::Finalized = submission.status {
            return Err(AssessmentError::SubmissionAlreadyFinalized);
        }

        let meta = get_assessment(&env, submission.assessment_id)?;
        let effective_limit = get_effective_time_limit(&env, &meta.config, &student);
        let now = env.ledger().timestamp();
        if effective_limit > 0 && now > submission.started_at + effective_limit {
            return Err(AssessmentError::AssessmentClosed);
        }

        submission.answers = answers;
        submission.submitted_at = now;

        let questions = get_questions_for_assessment(&env, submission.assessment_id);
        let result = GradingEngine::grade_submission(&env, &questions, &submission);
        submission.score = result.score;
        submission.max_score = result.max_score;
        submission.passed = submission.score >= meta.config.pass_score;
        submission.status = GradingEngine::derive_status(result.requires_manual_review);

        AssessmentEvents::emit_submission_received(
            &env,
            &submission.submission_id,
            submission.assessment_id,
        );
        AssessmentEvents::emit_submission_graded(
            &env,
            &submission.submission_id,
            submission.score,
            submission.max_score,
            submission.passed,
        );

        put_submission(&env, &submission);
        Ok(submission)
    }

    pub fn get_submission_details(
        env: Env,
        submission_id: BytesN<32>,
    ) -> Option<Submission> {
        env.storage()
            .persistent()
            .get(&DataKey::Submission(submission_id))
    }

    pub fn update_integrity_metadata(
        env: Env,
        oracle_or_admin: Address,
        submission_id: BytesN<32>,
        plagiarism_score: Option<u32>,
        plagiarism_flag: bool,
        integrity_flags: Vec<Symbol>,
        proctoring_evidence_hash: Option<BytesN<32>>,
    ) -> Result<(), AssessmentError> {
        let integration = get_integration(&env);
        let mut authorized = false;
        if let Some(sec_addr) = integration.security_monitor_contract {
            if sec_addr == oracle_or_admin {
                authorized = true;
            }
        }
        if oracle_or_admin == get_admin(&env) {
            authorized = true;
        }
        if !authorized {
            return Err(AssessmentError::SecurityIntegrationMissing);
        }

        let mut submission = get_submission(&env, &submission_id)?;
        submission.integrity.plagiarism_score = plagiarism_score;
        submission.integrity.plagiarism_flag = plagiarism_flag;
        submission.integrity.integrity_flags = integrity_flags.clone();
        submission.integrity.proctoring_evidence_hash = proctoring_evidence_hash;

        if plagiarism_flag {
            AssessmentEvents::emit_plagiarism_flagged(
                &env,
                &submission_id,
                plagiarism_score.unwrap_or(0),
                true,
            );
            let flag = Symbol::new(&env, "PLAGIARISM");
            AssessmentEvents::emit_integrity_event(&env, &submission_id, &flag, 100);
        }

        put_submission(&env, &submission);
        Ok(())
    }

    // Progress / analytics helper
    // Returns assessment_id -> (score, max_score, passed) for the latest attempt
    pub fn get_course_assessment_progress(
        env: Env,
        student: Address,
        course_id: Symbol,
    ) -> Map<u64, (u32, u32, bool)> {
        let mut result: Map<u64, (u32, u32, bool)> = Map::new(&env);

        // Scan sequential assessment IDs until a gap is found.
        let mut id: u64 = 1;
        loop {
            let meta_opt: Option<AssessmentMetadata> =
                env.storage().persistent().get(&DataKey::Assessment(id));
            let meta = match meta_opt {
                Some(m) => m,
                None => break,
            };
            if meta.course_id != course_id {
                id += 1;
                continue;
            }

            let key =
                DataKey::StudentAssessmentSubmissions(student.clone(), meta.assessment_id);
            let subs: Vec<BytesN<32>> =
                env.storage().persistent().get(&key).unwrap_or(Vec::new(&env));
            if subs.len() == 0 {
                id += 1;
                continue;
            }
            let last_id = subs.get(subs.len() - 1).unwrap();
            if let Some(sub) = env
                .storage()
                .persistent()
                .get::<_, Submission>(&DataKey::Submission(last_id.clone()))
            {
                result.set(
                    meta.assessment_id,
                    (sub.score, sub.max_score, sub.passed),
                );
            }

            id += 1;
        }

        result
    }
}




