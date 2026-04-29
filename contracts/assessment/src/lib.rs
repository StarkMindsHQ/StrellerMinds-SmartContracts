#![no_std]
#![allow(clippy::too_many_arguments)]
pub mod errors;
pub mod events;
pub mod grading;
pub mod types;

use errors::AssessmentError;
use events::AssessmentEvents;
use grading::GradingEngine;
use shared::rate_limiter::{enforce_rate_limit, RateLimitConfig};
use types::*;

#[cfg(test)]
mod test;

use shared::monitoring::{ContractHealthReport, Monitor};
use soroban_sdk::xdr::ToXdr;
use soroban_sdk::{
    contract, contractimpl, contracttype, symbol_short, Address, BytesN, Env, Map, String, Symbol, Vec,
};

#[contracttype]
#[derive(Clone)]
pub struct AssessmentExport {
    pub assessment_id: u64,
    pub attempt: u32,
    pub score: u32,
    pub has_score: bool,
    pub max_score: u32,
    pub passed: bool,
    pub submitted_at: u64,
}

const RL_OP_START_SUBMISSION: u64 = 1;
const RL_OP_SUBMIT_ANSWERS: u64 = 2;

fn get_rate_limits(env: &Env) -> AssessmentRateLimits {
    env.storage().instance().get(&DataKey::RateLimitCfg).unwrap_or(AssessmentRateLimits {
        max_submissions_per_day: 3,
        max_answers_per_day: 5,
        window_seconds: 86_400,
    })
}

#[contract]
pub struct Assessment;

fn get_admin(env: &Env) -> Address {
    env.storage().instance().get(&DataKey::Admin).expect("admin not set")
}

fn require_admin(env: &Env, actor: &Address) -> Result<(), AssessmentError> {
    let admin = get_admin(env);
    if admin != *actor {
        return Err(AssessmentError::Unauthorized);
    }
    Ok(())
}

fn get_next_assessment_id(env: &Env) -> u64 {
    let current: u64 = env.storage().instance().get(&DataKey::AssessmentCounter).unwrap_or(0);
    let next = current + 1;
    env.storage().instance().set(&DataKey::AssessmentCounter, &next);
    next
}

fn get_next_question_id(env: &Env) -> u64 {
    let current: u64 = env.storage().instance().get(&DataKey::QuestionCounter).unwrap_or(0);
    let next = current + 1;
    env.storage().instance().set(&DataKey::QuestionCounter, &next);
    next
}

fn get_assessment(env: &Env, assessment_id: u64) -> Result<AssessmentMetadata, AssessmentError> {
    env.storage()
        .persistent()
        .get(&DataKey::Assessment(assessment_id))
        .ok_or(AssessmentError::AssessmentNotFound)
}

fn put_assessment(env: &Env, meta: &AssessmentMetadata) {
    env.storage().persistent().set(&DataKey::Assessment(meta.assessment_id), meta);
}

fn get_questions_for_assessment(env: &Env, assessment_id: u64) -> Vec<Question> {
    let ids: Vec<u64> = env
        .storage()
        .persistent()
        .get(&DataKey::AssessmentQuestions(assessment_id))
        .unwrap_or(Vec::new(env));

    let mut result = Vec::new(env);
    for qid in ids.iter() {
        if let Some(q) = env.storage().persistent().get::<_, Question>(&DataKey::Question(qid)) {
            result.push_back(q);
        }
    }
    result
}

fn within_schedule(env: &Env, assessment_id: u64) -> bool {
    let schedule: Option<ScheduleConfig> =
        env.storage().persistent().get(&DataKey::Schedule(assessment_id));
    if let Some(s) = schedule {
        let now = env.ledger().timestamp();
        now >= s.start_time && now <= s.end_time
    } else {
        true
    }
}

fn get_accommodation(env: &Env, student: &Address) -> Option<AccommodationConfig> {
    env.storage().persistent().get(&DataKey::Accommodation(student.clone()))
}

fn get_effective_time_limit(env: &Env, config: &AssessmentConfig, student: &Address) -> u64 {
    if let Some(ac) = get_accommodation(env, student) {
        let bonus = config.time_limit_seconds.saturating_mul(ac.extra_time_percent as u64) / 100;
        config.time_limit_seconds.saturating_add(bonus)
    } else {
        config.time_limit_seconds
    }
}

fn get_student_attempts(env: &Env, student: &Address, assessment_id: u64) -> u32 {
    let key = DataKey::StudentAssessmentSubmissions(student.clone(), assessment_id);
    let ids: Vec<BytesN<32>> = env.storage().persistent().get(&key).unwrap_or(Vec::new(env));
    ids.len()
}

fn append_student_submission(
    env: &Env,
    student: &Address,
    assessment_id: u64,
    submission_id: &BytesN<32>,
) {
    let key = DataKey::StudentAssessmentSubmissions(student.clone(), assessment_id);
    let mut ids: Vec<BytesN<32>> = env.storage().persistent().get(&key).unwrap_or(Vec::new(env));
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

    let mut all_ids = env
        .storage()
        .persistent()
        .get::<_, Vec<BytesN<32>>>(&DataKey::StudentAllSubmissions(submission.student.clone()))
        .unwrap_or_else(|| Vec::new(env));

    let mut found = false;
    for i in 0..all_ids.len() {
        if all_ids.get(i).unwrap() == submission.submission_id {
            found = true;
            break;
        }
    }
    if !found {
        all_ids.push_back(submission.submission_id.clone());
    }
    env.storage()
        .persistent()
        .set(&DataKey::StudentAllSubmissions(submission.student.clone()), &all_ids);
}

fn get_or_init_adaptive_state(env: &Env, student: &Address, assessment_id: u64) -> AdaptiveState {
    env.storage()
        .persistent()
        .get(&DataKey::Adaptive(student.clone(), assessment_id))
        .unwrap_or(AdaptiveState { current_difficulty: 3, completed_questions: Vec::new(env) })
}

fn put_adaptive_state(env: &Env, student: &Address, assessment_id: u64, state: &AdaptiveState) {
    env.storage().persistent().set(&DataKey::Adaptive(student.clone(), assessment_id), state);
}

fn get_integration(env: &Env) -> IntegrationConfig {
    env.storage().instance().get(&DataKey::Integration).unwrap_or(IntegrationConfig {
        analytics_contract: None,
        progress_contract: None,
        security_monitor_contract: None,
    })
}

#[allow(clippy::too_many_arguments)]
#[contractimpl]
impl Assessment {
    // Initialization & configuration
    /// Initializes the assessment contract, setting the admin address and default integration config.
    ///
    /// Must be called once before any other function. The caller must authorize the transaction.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `admin` - The address that will be granted admin privileges.
    ///
    /// # Errors
    /// Returns [`AssessmentError::AlreadyInitialized`] if the contract has already been initialized.
    ///
    /// # Example
    /// ```ignore
    /// client.initialize(&admin);
    /// ```
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
        env.storage().instance().set(
            &DataKey::RateLimitCfg,
            &AssessmentRateLimits {
                max_submissions_per_day: 3,
                max_answers_per_day: 5,
                window_seconds: 86_400,
            },
        );
        AssessmentEvents::emit_initialized(&env, &admin);
        Ok(())
    }

    pub fn update_rate_limits(
        env: Env,
        admin: Address,
        rate_limits: AssessmentRateLimits,
    ) -> Result<(), AssessmentError> {
        require_admin(&env, &admin)?;
        admin.require_auth();
        env.storage().instance().set(&DataKey::RateLimitCfg, &rate_limits);
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

    /// Creates a new assessment for the given course and module, returning the generated assessment ID.
    ///
    /// The instructor must authorize the call. The assessment starts in an unpublished state.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `instructor` - The instructor's address creating the assessment.
    /// * `course_id` - The symbol identifying the course this assessment belongs to.
    /// * `module_id` - The symbol identifying the module within the course.
    /// * `config` - The [`AssessmentConfig`] specifying pass score, max attempts, and time limits.
    ///
    /// # Errors
    /// Returns [`AssessmentError::InvalidConfig`] if `max_attempts` or `pass_score` is zero.
    ///
    /// # Example
    /// ```ignore
    /// let assessment_id = client.create_assessment(&instructor, &course_id, &module_id, &config);
    /// ```
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
        env.storage().persistent().set(&DataKey::AssessmentQuestions(id), &Vec::<u64>::new(&env));
        AssessmentEvents::emit_assessment_created(&env, id, &instructor, &course_id);
        Ok(id)
    }

    /// Marks an assessment as published, making it available for students to attempt.
    ///
    /// Requires admin authorization.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `admin` - The admin address authorizing the publication.
    /// * `assessment_id` - The ID of the assessment to publish.
    ///
    /// # Errors
    /// Returns [`AssessmentError::Unauthorized`] if the caller is not the admin.
    /// Returns [`AssessmentError::AssessmentNotFound`] if no assessment exists with the given ID.
    ///
    /// # Example
    /// ```ignore
    /// client.publish_assessment(&admin, assessment_id);
    /// ```
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

    /// Internal helper used by the specialized add_question_* functions.
    #[allow(clippy::too_many_arguments)]
    fn add_question_internal(
        env: &Env,
        admin: &Address,
        assessment_id: u64,
        question_type: QuestionType,
        max_score: u32,
        difficulty: u32,
        content_hash: BytesN<32>,
        options: Vec<QuestionOption>,
        answer_key: AnswerKey,
    ) -> Result<u64, AssessmentError> {
        require_admin(env, admin)?;
        let _ = get_assessment(env, assessment_id)?;

        if max_score == 0 || difficulty == 0 {
            return Err(AssessmentError::InvalidQuestionType);
        }

        let qid = get_next_question_id(env);
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
        env.storage().persistent().set(&DataKey::Question(qid), &q);

        let mut ids: Vec<u64> = env
            .storage()
            .persistent()
            .get(&DataKey::AssessmentQuestions(assessment_id))
            .unwrap_or(Vec::new(env));
        ids.push_back(qid);
        env.storage().persistent().set(&DataKey::AssessmentQuestions(assessment_id), &ids);

        AssessmentEvents::emit_question_added(env, assessment_id, qid);
        Ok(qid)
    }

    /// Add a single-choice question.
    pub fn add_question_single_choice(
        env: Env,
        admin: Address,
        assessment_id: u64,
        max_score: u32,
        difficulty: u32,
        content_hash: BytesN<32>,
        options: Vec<QuestionOption>,
        correct_option_id: u32,
    ) -> Result<u64, AssessmentError> {
        let answer_key = AnswerKey::SingleChoice(correct_option_id);
        Self::add_question_internal(
            &env,
            &admin,
            assessment_id,
            QuestionType::SingleChoice,
            max_score,
            difficulty,
            content_hash,
            options,
            answer_key,
        )
    }

    /// Add a multiple-choice question.
    pub fn add_question_multiple_choice(
        env: Env,
        admin: Address,
        assessment_id: u64,
        max_score: u32,
        difficulty: u32,
        content_hash: BytesN<32>,
        options: Vec<QuestionOption>,
        correct_option_ids: Vec<u32>,
    ) -> Result<u64, AssessmentError> {
        let answer_key = AnswerKey::MultipleChoice(correct_option_ids);
        Self::add_question_internal(
            &env,
            &admin,
            assessment_id,
            QuestionType::MultipleChoice,
            max_score,
            difficulty,
            content_hash,
            options,
            answer_key,
        )
    }

    /// Add a numeric-range question.
    pub fn add_question_numeric_range(
        env: Env,
        admin: Address,
        assessment_id: u64,
        max_score: u32,
        difficulty: u32,
        content_hash: BytesN<32>,
        options: Vec<QuestionOption>,
        min: i64,
        max: i64,
    ) -> Result<u64, AssessmentError> {
        let answer_key = AnswerKey::NumericRange(min, max);
        Self::add_question_internal(
            &env,
            &admin,
            assessment_id,
            QuestionType::Numeric,
            max_score,
            difficulty,
            content_hash,
            options,
            answer_key,
        )
    }

    /// Add a short-text question.
    pub fn add_question_short_text(
        env: Env,
        admin: Address,
        assessment_id: u64,
        max_score: u32,
        difficulty: u32,
        content_hash: BytesN<32>,
        options: Vec<QuestionOption>,
        accepted_answers: Vec<String>,
    ) -> Result<u64, AssessmentError> {
        let answer_key = AnswerKey::ShortText(accepted_answers);
        Self::add_question_internal(
            &env,
            &admin,
            assessment_id,
            QuestionType::ShortText,
            max_score,
            difficulty,
            content_hash,
            options,
            answer_key,
        )
    }

    /// Generic add_question for specialized needs (used in tests).
    #[allow(clippy::too_many_arguments)]
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
        Self::add_question_internal(
            &env,
            &admin,
            assessment_id,
            question_type,
            max_score,
            difficulty,
            content_hash,
            options,
            answer_key,
        )
    }

    /// Returns the metadata for the given assessment, or `None` if it does not exist.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `assessment_id` - The ID of the assessment to retrieve.
    ///
    /// # Example
    /// ```ignore
    /// let meta = client.get_assessment_metadata(assessment_id);
    /// ```
    pub fn get_assessment_metadata(env: Env, assessment_id: u64) -> Option<AssessmentMetadata> {
        env.storage().persistent().get(&DataKey::Assessment(assessment_id))
    }

    // Scheduling & accessibility

    /// Sets or replaces the availability schedule for the given assessment.
    ///
    /// Requires admin authorization. Students can only start submissions within the scheduled window.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `admin` - The admin address authorizing the schedule change.
    /// * `assessment_id` - The ID of the assessment to schedule.
    /// * `start_time` - Unix timestamp (seconds) when the assessment opens.
    /// * `end_time` - Unix timestamp (seconds) when the assessment closes.
    /// * `time_zone_offset_minutes` - Timezone offset in minutes for display purposes.
    /// * `proctoring_provider` - Optional symbol identifying an external proctoring service.
    ///
    /// # Errors
    /// Returns [`AssessmentError::Unauthorized`] if the caller is not the admin.
    /// Returns [`AssessmentError::InvalidSchedule`] if `end_time` is not after `start_time`.
    /// Returns [`AssessmentError::AssessmentNotFound`] if no assessment exists with the given ID.
    ///
    /// # Example
    /// ```ignore
    /// client.set_schedule(&admin, assessment_id, start, end, 0, None);
    /// ```
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
        env.storage().persistent().set(&DataKey::Schedule(assessment_id), &schedule);
        AssessmentEvents::emit_schedule_created(&env, assessment_id);
        Ok(())
    }

    /// Sets accessibility accommodations for a student, such as extra time or additional attempts.
    ///
    /// Requires admin authorization. Overwrites any previously stored accommodation for the student.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `admin` - The admin address authorizing the accommodation.
    /// * `student` - The student's address to apply accommodations to.
    /// * `config` - The [`AccommodationConfig`] specifying extra time percentage and extra attempts.
    ///
    /// # Errors
    /// Returns [`AssessmentError::Unauthorized`] if the caller is not the admin.
    ///
    /// # Example
    /// ```ignore
    /// client.set_accommodation(&admin, &student, &config);
    /// ```
    pub fn set_accommodation(
        env: Env,
        admin: Address,
        student: Address,
        config: AccommodationConfig,
    ) -> Result<(), AssessmentError> {
        require_admin(&env, &admin)?;
        env.storage().persistent().set(&DataKey::Accommodation(student), &config);
        Ok(())
    }

    /// Returns the accessibility accommodation configuration for the given student, or `None` if none is set.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `student` - The student's address.
    ///
    /// # Example
    /// ```ignore
    /// let accommodation = client.get_accommodation_for_student(&student);
    /// ```
    pub fn get_accommodation_for_student(
        env: Env,
        student: Address,
    ) -> Option<AccommodationConfig> {
        get_accommodation(&env, &student)
    }

    // Adaptive testing

    /// Returns the next unanswered question for the student based on their current adaptive difficulty level.
    ///
    /// Requires the assessment to have adaptive mode enabled. Returns `None` if no matching question is available.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `student` - The student's address.
    /// * `assessment_id` - The ID of the adaptive assessment.
    ///
    /// # Errors
    /// Returns [`AssessmentError::AssessmentNotFound`] if the assessment does not exist.
    /// Returns [`AssessmentError::AdaptiveNotEnabled`] if the assessment is not configured for adaptive testing.
    ///
    /// # Example
    /// ```ignore
    /// let question = client.get_next_question(&student, assessment_id);
    /// ```
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

    /// Updates the student's adaptive difficulty state after answering a question.
    ///
    /// Difficulty increases by one on a correct answer and decreases by one on an incorrect answer, clamped between 1 and 5.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `student` - The student's address.
    /// * `assessment_id` - The ID of the adaptive assessment.
    /// * `question_id` - The ID of the question just answered.
    /// * `was_correct` - `true` if the student answered the question correctly.
    ///
    /// # Errors
    /// Returns [`AssessmentError::AssessmentNotFound`] if the assessment does not exist.
    /// Returns [`AssessmentError::AdaptiveNotEnabled`] if the assessment is not configured for adaptive testing.
    ///
    /// # Example
    /// ```ignore
    /// client.update_adaptive_state(&student, assessment_id, question_id, true);
    /// ```
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

    /// Opens a new in-progress submission for the student, returning a unique submission ID.
    ///
    /// The student must authorize the call. Checks that the assessment is published, within schedule, and that the student has remaining attempts (including any accommodation bonuses).
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `student` - The student's address starting the attempt.
    /// * `assessment_id` - The ID of the assessment to attempt.
    ///
    /// # Errors
    /// Returns [`AssessmentError::AssessmentNotFound`] if the assessment does not exist.
    /// Returns [`AssessmentError::AssessmentNotPublished`] if the assessment is not yet published.
    /// Returns [`AssessmentError::AssessmentClosed`] if the current time is outside the scheduled window.
    /// Returns [`AssessmentError::MaxAttemptsReached`] if the student has used all allowed attempts.
    ///
    /// # Example
    /// ```ignore
    /// let submission_id = client.start_submission(&student, assessment_id);
    /// ```
    pub fn start_submission(
        env: Env,
        student: Address,
        assessment_id: u64,
    ) -> Result<BytesN<32>, AssessmentError> {
        student.require_auth();
        let rl = get_rate_limits(&env);
        enforce_rate_limit(
            &env,
            &DataKey::RateLimit(student.clone(), RL_OP_START_SUBMISSION),
            &RateLimitConfig {
                max_calls: rl.max_submissions_per_day,
                window_seconds: rl.window_seconds,
            },
        )
        .map_err(|_| AssessmentError::RateLimitExceeded)?;
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

        let addr_bytes = student.clone().to_xdr(&env);
        let sid_hash = env.crypto().sha256(&addr_bytes);
        let sid: BytesN<32> = sid_hash.into();
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
                plagiarism_score: 0,
                plagiarism_flag: false,
                integrity_flags: Vec::new(&env),
                has_proctoring_evidence: false,
                proctoring_evidence_hash: BytesN::from_array(&env, &[0u8; 32]),
            },
        };
        put_submission(&env, &submission);
        append_student_submission(&env, &student, assessment_id, &sid);
        Ok(sid)
    }

    /// Submits answers for an in-progress submission, triggers auto-grading, and returns the completed submission.
    ///
    /// The student must authorize the call. The submission is finalized after this call; time-limit violations cause an error.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `student` - The student's address submitting their answers.
    /// * `submission_id` - The ID of the in-progress submission to finalize.
    /// * `answers` - A list of [`SubmittedAnswer`] entries, one per answered question.
    ///
    /// # Errors
    /// Returns [`AssessmentError::SubmissionNotFound`] if the submission does not exist.
    /// Returns [`AssessmentError::Unauthorized`] if the caller does not own the submission.
    /// Returns [`AssessmentError::SubmissionAlreadyFinalized`] if the submission has already been graded.
    /// Returns [`AssessmentError::AssessmentClosed`] if the student's time limit has been exceeded.
    ///
    /// # Example
    /// ```ignore
    /// let result = client.submit_answers(&student, &submission_id, &answers);
    /// ```
    pub fn submit_answers(
        env: Env,
        student: Address,
        submission_id: BytesN<32>,
        answers: Vec<SubmittedAnswer>,
    ) -> Result<Submission, AssessmentError> {
        student.require_auth();
        let rl = get_rate_limits(&env);
        enforce_rate_limit(
            &env,
            &DataKey::RateLimit(student.clone(), RL_OP_SUBMIT_ANSWERS),
            &RateLimitConfig {
                max_calls: rl.max_answers_per_day,
                window_seconds: rl.window_seconds,
            },
        )
        .map_err(|_| AssessmentError::RateLimitExceeded)?;
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

    /// Returns the full submission record for the given submission ID, or `None` if it does not exist.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `submission_id` - The unique identifier of the submission to retrieve.
    ///
    /// # Example
    /// ```ignore
    /// let submission = client.get_submission_details(&submission_id);
    /// ```
    pub fn get_submission_details(env: Env, submission_id: BytesN<32>) -> Option<Submission> {
        env.storage().persistent().get(&DataKey::Submission(submission_id))
    }

    /// Attaches integrity metadata to a submission, such as plagiarism scores and proctoring evidence.
    ///
    /// Only the registered security monitor contract or the admin may call this function. Emits integrity events if a plagiarism flag is set.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `oracle_or_admin` - The address of the security monitor contract or the admin.
    /// * `submission_id` - The ID of the submission to update.
    /// * `plagiarism_score` - Numeric plagiarism confidence score (0–100).
    /// * `plagiarism_flag` - `true` if the submission is flagged for plagiarism.
    /// * `integrity_flags` - A list of symbolic integrity violation codes.
    /// * `has_proctoring_evidence` - Whether proctoring evidence is available for this submission.
    /// * `proctoring_evidence_hash` - Hash of the proctoring evidence artifact.
    ///
    /// # Errors
    /// Returns [`AssessmentError::SecurityIntegrationMissing`] if the caller is neither the admin nor the registered security monitor.
    /// Returns [`AssessmentError::SubmissionNotFound`] if the submission does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.update_integrity_metadata(&oracle, &submission_id, 0, false, &flags, false, &hash);
    /// ```
    pub fn update_integrity_metadata(
        env: Env,
        oracle_or_admin: Address,
        submission_id: BytesN<32>,
        plagiarism_score: u32,
        plagiarism_flag: bool,
        integrity_flags: Vec<Symbol>,
        has_proctoring_evidence: bool,
        proctoring_evidence_hash: BytesN<32>,
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
        submission.integrity.has_proctoring_evidence = has_proctoring_evidence;
        submission.integrity.proctoring_evidence_hash = proctoring_evidence_hash;

        if plagiarism_flag {
            AssessmentEvents::emit_plagiarism_flagged(&env, &submission_id, plagiarism_score, true);
            let flag = Symbol::new(&env, "PLAGIARISM");
            AssessmentEvents::emit_integrity_event(&env, &submission_id, &flag, 100);
        }

        put_submission(&env, &submission);
        Ok(())
    }

    /// Returns the latest assessment results for all assessments in a course taken by the student.
    ///
    /// The returned map keys are assessment IDs and values are `(score, max_score, passed)` tuples from the most recent attempt.
    ///
    /// # Arguments
    /// * `env` - The Soroban environment.
    /// * `student` - The student's address.
    /// * `course_id` - The symbol identifying the course to aggregate progress for.
    ///
    /// # Example
    /// ```ignore
    /// let progress = client.get_course_assessment_progress(&student, &course_id);
    /// ```
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

            let key = DataKey::StudentAssessmentSubmissions(student.clone(), meta.assessment_id);
            let subs: Vec<BytesN<32>> =
                env.storage().persistent().get(&key).unwrap_or(Vec::new(&env));
            if subs.is_empty() {
                id += 1;
                continue;
            }
            let last_id = subs.get(subs.len() - 1).unwrap();
            if let Some(sub) = env
                .storage()
                .persistent()
                .get::<_, Submission>(&DataKey::Submission(last_id.clone()))
            {
                result.set(meta.assessment_id, (sub.score, sub.max_score, sub.passed));
            }

            id += 1;
        }

        result
    }

    fn get_student_all_submissions(env: &Env, student: &Address) -> Vec<BytesN<32>> {
        env.storage()
            .persistent()
            .get(&DataKey::StudentAllSubmissions(student.clone()))
            .unwrap_or_else(|| Vec::new(env))
    }

    pub fn export_user_data(env: Env, user: Address) -> Vec<AssessmentExport> {
        let submission_ids = Self::get_student_all_submissions(&env, &user);
        let mut exports: Vec<AssessmentExport> = Vec::new(&env);

        for i in 0..submission_ids.len() {
            if let Some(submission_id) = submission_ids.get(i) {
                if let Some(submission) = env
                    .storage()
                    .persistent()
                    .get::<_, Submission>(&DataKey::Submission(submission_id))
                {
                    exports.push_back(AssessmentExport {
                        assessment_id: submission.assessment_id,
                        attempt: submission.attempt,
                        score: submission.score,
                        has_score: true,
                        max_score: submission.max_score,
                        passed: submission.passed,
                        submitted_at: submission.submitted_at,
                    });
                }
            }
        }

        exports
    }

    pub fn health_check(env: Env) -> ContractHealthReport {
        let initialized = env.storage().instance().has(&DataKey::Admin);
        let report = Monitor::build_health_report(&env, symbol_short!("assess"), initialized);
        Monitor::emit_health_check(&env, &report);
        report
    }
}
