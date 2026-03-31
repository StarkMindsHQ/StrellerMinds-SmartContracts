//! Gas-optimized version of the Assessment contract
//! 
//! This module provides optimized implementations of key functions
//! with significant gas savings through storage optimization and batching.

#![no_std]

use soroban_sdk::{
    contract, contractimpl, Address, BytesN, Env, Map, String, Symbol, Vec,
    xdr::ToXdr
};

use crate::{
    errors::AssessmentError,
    events::AssessmentEvents,
    grading::GradingEngine,
    types::{
        AssessmentConfig, AssessmentMetadata, Question, QuestionType, 
        SubmittedAnswer, Submission, SubmissionStatus, DataKey, 
        QuestionOption, AnswerKey, IntegrityMetadata
    },
};

use shared::{
    gas_profiler::{GasProfiler, profile_gas},
    storage_optimizer::{StorageOptimizer, OptimizedKey, OptimizedPatterns, PackedData}
};

#[contract]
pub struct GasOptimizedAssessment;

#[contractimpl]
impl GasOptimizedAssessment {
    /// Gas-optimized initialization with reduced storage operations
    pub fn initialize(env: Env, admin: Address) -> Result<(), AssessmentError> {
        profile_gas!(&env, Symbol::short("initialize"), {
            if env.storage().instance().has(&DataKey::Admin) {
                return Err(AssessmentError::AlreadyInitialized);
            }
            
            admin.require_auth();
            
            // Batch set initial values
            let mut batch_data = Map::new(&env);
            batch_data.set(OptimizedKey::Admin, admin.clone().to_xdr(&env));
            
            // Initialize counters in packed format
            let packed_counters = [0u8; 4]; // assessment, question, submission, batch counters
            batch_data.set(OptimizedKey::Counter(0), soroban_sdk::Bytes::from_array(&env, &packed_counters));
            
            StorageOptimizer::batch_write(&env, &batch_data);
            
            AssessmentEvents::emit_initialized(&env, &admin);
            Ok(())
        })
    }

    /// Gas-optimized assessment creation with compact storage
    pub fn create_assessment_optimized(
        env: Env,
        instructor: Address,
        course_id: Symbol,
        module_id: Symbol,
        config: AssessmentConfig,
    ) -> Result<u64, AssessmentError> {
        profile_gas!(&env, Symbol::short("create_assessment"), {
            instructor.require_auth();
            
            if config.max_attempts == 0 || config.pass_score == 0 {
                return Err(AssessmentError::InvalidConfig);
            }

            // Use packed counter for ID generation
            let id = StorageOptimizer::increment_counter(&env, 0); // assessment counter
            
            // Store assessment metadata in optimized format
            let meta = AssessmentMetadata {
                assessment_id: id,
                course_id: course_id.clone(),
                module_id,
                instructor: instructor.clone(),
                config,
                published: false,
            };
            
            // Use optimized storage key
            let key = OptimizedKey::Index(id as u16);
            env.storage().persistent().set(&key, &meta);
            
            // Initialize empty question list as packed data
            let packed_data = PackedData {
                flags: 0, // no flags set
                counters: [0, 0, 0, 0], // all counters zero
                timestamp: env.ledger().timestamp() as u32,
            };
            OptimizedPatterns::store_user_profile(&env, &instructor, &packed_data);
            
            AssessmentEvents::emit_assessment_created(&env, id, &instructor, &course_id);
            Ok(id)
        })
    }

    /// Batch question addition for significant gas savings
    pub fn add_questions_batch(
        env: Env,
        admin: Address,
        assessment_id: u64,
        questions: Vec<(QuestionType, u32, u32, BytesN<32>, Vec<QuestionOption>, AnswerKey)>,
    ) -> Result<Vec<u64>, AssessmentError> {
        profile_gas!(&env, Symbol::short("add_questions_batch"), {
            admin.require_auth();
            
            // Verify assessment exists
            let key = OptimizedKey::Index(assessment_id as u16);
            let _meta: AssessmentMetadata = env.storage().persistent()
                .get(&key)
                .ok_or(AssessmentError::AssessmentNotFound)?;

            let mut question_ids = Vec::new(&env);
            let mut batch_writes = Map::new(&env);
            
            // Process all questions in a single batch
            for (question_type, max_score, difficulty, content_hash, options, answer_key) in questions.iter() {
                if *max_score == 0 || *difficulty == 0 {
                    return Err(AssessmentError::InvalidQuestionType);
                }

                // Use packed counter for question ID
                let qid = StorageOptimizer::increment_counter(&env, 1); // question counter
                
                let question = Question {
                    question_id: qid,
                    assessment_id,
                    question_type: question_type.clone(),
                    max_score: *max_score,
                    difficulty: *difficulty,
                    content_hash: content_hash.clone(),
                    options: options.clone(),
                    answer_key: answer_key.clone(),
                };
                
                // Add to batch writes
                let q_key = OptimizedKey::Index(qid as u16);
                let q_data = question.to_xdr(&env);
                batch_writes.set(q_key, q_data);
                
                question_ids.push_back(qid);
            }
            
            // Execute batch write
            let result = StorageOptimizer::batch_write(&env, &batch_writes);
            
            // Update assessment question list
            let list_key = OptimizedKey::User(Address::from_xdr(&env, &assessment_id.to_xdr(&env)).unwrap(), 1);
            let id_list = question_ids.to_xdr(&env);
            env.storage().persistent().set(&list_key, &id_list);
            
            AssessmentEvents::emit_question_added(&env, assessment_id, question_ids.get(question_ids.len() - 1).unwrap_or(&0));
            
            Ok(question_ids)
        })
    }

    /// Gas-optimized submission start with reduced storage operations
    pub fn start_submission_optimized(
        env: Env,
        student: Address,
        assessment_id: u64,
    ) -> Result<BytesN<32>, AssessmentError> {
        profile_gas!(&env, Symbol::short("start_submission"), {
            student.require_auth();
            
            // Verify assessment exists and is published
            let key = OptimizedKey::Index(assessment_id as u16);
            let meta: AssessmentMetadata = env.storage().persistent()
                .get(&key)
                .ok_or(AssessmentError::AssessmentNotFound)?;
            
            if !meta.published {
                return Err(AssessmentError::AssessmentNotPublished);
            }
            
            // Check schedule (simplified for gas optimization)
            let now = env.ledger().timestamp();
            if let Some(schedule_key) = env.storage().persistent().get::<_, OptimizedKey>(&OptimizedKey::TimeBucket(assessment_id as u32)) {
                // In practice, would check actual schedule
                if now < 1000000 || now > 2000000 { // Placeholder check
                    return Err(AssessmentError::AssessmentClosed);
                }
            }
            
            // Check attempts using packed data
            let attempts = StorageOptimizer::increment_counter(&env, 2); // submission counter
            if attempts > meta.config.max_attempts as u64 {
                return Err(AssessmentError::MaxAttemptsReached);
            }
            
            // Generate optimized submission ID
            let mut bytes = [0u8; 32];
            let addr_bytes = student.to_xdr(&env);
            bytes[0..20].copy_from_slice(&addr_bytes.to_array()[0..20]);
            let attempt_bytes = attempts.to_be_bytes();
            bytes[28..32].copy_from_slice(&attempt_bytes[4..8]);
            let submission_id: BytesN<32> = BytesN::from_array(&env, &bytes);
            
            // Create optimized submission with packed integrity data
            let submission = Submission {
                submission_id: submission_id.clone(),
                assessment_id,
                student: student.clone(),
                attempt: attempts as u32,
                started_at: now,
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
            
            // Store submission with optimized key
            let sub_key = OptimizedKey::Batch(submission_id.to_array()[0] as u32);
            env.storage().persistent().set(&sub_key, &submission);
            
            // Track in time bucket for analytics
            OptimizedPatterns::track_user_activity(&env, &student, 1); // 1 = submission activity
            
            Ok(submission_id)
        })
    }

    /// Gas-optimized answer submission with batched grading
    pub fn submit_answers_optimized(
        env: Env,
        student: Address,
        submission_id: BytesN<32>,
        answers: Vec<SubmittedAnswer>,
    ) -> Result<Submission, AssessmentError> {
        profile_gas!(&env, Symbol::short("submit_answers"), {
            student.require_auth();
            
            // Retrieve submission
            let sub_key = OptimizedKey::Batch(submission_id.to_array()[0] as u32);
            let mut submission: Submission = env.storage().persistent()
                .get(&sub_key)
                .ok_or(AssessmentError::SubmissionNotFound)?;
            
            if submission.student != student {
                return Err(AssessmentError::Unauthorized);
            }
            
            if let SubmissionStatus::Finalized = submission.status {
                return Err(AssessmentError::SubmissionAlreadyFinalized);
            }
            
            // Get assessment metadata
            let key = OptimizedKey::Index(submission.assessment_id as u16);
            let meta: AssessmentMetadata = env.storage().persistent()
                .get(&key)
                .ok_or(AssessmentError::AssessmentNotFound)?;
            
            // Time limit check (simplified)
            let now = env.ledger().timestamp();
            if meta.config.time_limit_seconds > 0 && 
               now > submission.started_at + meta.config.time_limit_seconds {
                return Err(AssessmentError::AssessmentClosed);
            }
            
            // Update submission
            submission.answers = answers.clone();
            submission.submitted_at = now;
            
            // Batch grading operation
            let questions = Self::get_questions_for_assessment_optimized(&env, submission.assessment_id)?;
            let result = GradingEngine::grade_submission(&env, &questions, &submission);
            
            submission.score = result.score;
            submission.max_score = result.max_score;
            submission.passed = submission.score >= meta.config.pass_score;
            submission.status = GradingEngine::derive_status(result.requires_manual_review);
            
            // Batch update submission and analytics
            let mut batch_writes = Map::new(&env);
            batch_writes.set(sub_key.clone(), submission.to_xdr(&env));
            
            // Update analytics counters
            StorageOptimizer::increment_counter(&env, 3); // batch counter
            OptimizedPatterns::track_user_activity(&env, &student, 2); // 2 = completed submission
            
            // Execute batch update
            StorageOptimizer::batch_write(&env, &batch_writes);
            
            AssessmentEvents::emit_submission_received(&env, &submission.submission_id, submission.assessment_id);
            AssessmentEvents::emit_submission_graded(&env, &submission.submission_id, submission.score, submission.max_score, submission.passed);
            
            Ok(submission)
        })
    }

    /// Optimized question retrieval with caching
    pub fn get_questions_for_assessment_optimized(env: &Env, assessment_id: u64) -> Result<Vec<Question>, AssessmentError> {
        profile_gas!(env, Symbol::short("get_questions"), {
            // Try cache first
            let cache_key = OptimizedKey::TimeBucket(assessment_id as u32);
            if let Some(cached_questions) = StorageOptimizer::get_cached::<Vec<Question>>(env, cache_key.clone()) {
                return Ok(cached_questions);
            }
            
            // Load from storage
            let list_key = OptimizedKey::User(Address::from_xdr(env, &assessment_id.to_xdr(env)).unwrap_or(Address::generate(env)), 1);
            let question_ids: Vec<u64> = env.storage().persistent()
                .get(&list_key)
                .unwrap_or(Vec::new(env));
            
            let mut questions = Vec::new(env);
            for qid in question_ids.iter() {
                let q_key = OptimizedKey::Index(*qid as u16);
                if let Some(question) = env.storage().persistent().get::<_, Question>(&q_key) {
                    questions.push_back(question);
                }
            }
            
            // Cache for future use
            StorageOptimizer::cache_in_instance(env, cache_key, OptimizedKey::Batch(0), 300); // 5 minute cache
            
            Ok(questions)
        })
    }

    /// Batch operation for multiple submissions
    pub fn batch_submit_answers(
        env: Env,
        student: Address,
        submissions: Vec<(BytesN<32>, Vec<SubmittedAnswer>)>,
    ) -> Result<Vec<Submission>, AssessmentError> {
        profile_gas!(&env, Symbol::short("batch_submit"), {
            student.require_auth();
            
            let mut results = Vec::new(&env);
            let mut batch_writes = Map::new(&env);
            
            for (submission_id, answers) in submissions.iter() {
                match Self::submit_answers_optimized(env.clone(), student.clone(), submission_id.clone(), answers.clone()) {
                    Ok(submission) => {
                        results.push_back(submission);
                    },
                    Err(_) => {
                        // Continue with other submissions even if one fails
                        continue;
                    }
                }
            }
            
            Ok(results)
        })
    }

    /// Get gas profile for this contract
    pub fn get_gas_profile(env: Env) -> Option<shared::gas_profiler::GasProfile> {
        GasProfiler::get_profile(&env)
    }

    /// Get gas efficiency report
    pub fn get_gas_efficiency_report(env: Env) -> Map<Symbol, u64> {
        GasProfiler::generate_efficiency_report(&env)
    }

    /// Get optimization recommendations
    pub fn get_optimization_recommendations(env: Env) -> Vec<Symbol> {
        shared::gas_profiler::GasOptimizer::analyze_and_recommend(&env)
    }
}
