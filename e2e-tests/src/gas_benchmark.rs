//! Gas benchmarking tests for StrellerMinds contracts
//! 
//! This module provides comprehensive benchmarking to compare
//! original vs optimized implementations and validate gas savings.

#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, BytesN, Env, Symbol, Vec,
};
use shared::{
    gas_profiler::{GasProfiler, profile_gas},
    storage_optimizer::{StorageOptimizer, OptimizedKey}
};

use contracts::assessment::{
    AssessmentContract, AssessmentClient,
    types::{
        AssessmentConfig, QuestionType, QuestionOption, 
        AnswerKey, SubmittedAnswer, SubmittedAnswerValue
    }
};

use contracts::assessment::gas_optimized::{
    GasOptimizedAssessment, GasOptimizedAssessmentClient
};

/// Test configuration for benchmarking
struct BenchmarkConfig {
    pub num_assessments: u32,
    pub num_questions_per_assessment: u32,
    pub num_submissions_per_assessment: u32,
    pub question_types: Vec<QuestionType>,
}

impl Default for BenchmarkConfig {
    fn default() -> Self {
        Self {
            num_assessments: 10,
            num_questions_per_assessment: 20,
            num_submissions_per_assessment: 50,
            question_types: vec![
                QuestionType::SingleChoice,
                QuestionType::MultipleChoice,
                QuestionType::Numeric,
                QuestionType::ShortText,
            ],
        }
    }
}

/// Benchmark results structure
#[derive(Debug)]
struct BenchmarkResults {
    pub original_gas: u64,
    pub optimized_gas: u64,
    pub savings_percentage: f64,
    pub operation_count: u32,
}

impl BenchmarkResults {
    fn new(original: u64, optimized: u64, count: u32) -> Self {
        let savings = if original > 0 {
            ((original - optimized) as f64 / original as f64) * 100.0
        } else {
            0.0
        };
        
        Self {
            original_gas: original,
            optimized_gas: optimized,
            savings_percentage: savings,
            operation_count: count,
        }
    }
    
    fn print_summary(&self, operation_name: &str) {
        println!("\n=== {} Benchmark Results ===", operation_name);
        println!("Original Gas: {}", self.original_gas);
        println!("Optimized Gas: {}", self.optimized_gas);
        println!("Gas Savings: {:.2}%", self.savings_percentage);
        println!("Operations: {}", self.operation_count);
        
        if self.savings_percentage > 0.0 {
            println!("✅ Optimization successful!");
        } else {
            println!("❌ No gas savings achieved");
        }
    }
}

/// Gas measurement utility
fn measure_gas<T>(env: &Env, operation: impl FnOnce() -> T) -> (T, u64) {
    let start_gas = env.contract().get_current_gas();
    let result = operation();
    let end_gas = env.contract().get_current_gas();
    (result, end_gas - start_gas)
}

/// Setup test environment with contracts
fn setup_contracts(env: &Env) -> (AssessmentClient, GasOptimizedAssessmentClient, Address) {
    env.mock_all_auths();
    
    // Deploy original assessment contract
    let original_contract_id = env.register(AssessmentContract, ());
    let original_client = AssessmentClient::new(env, &original_contract_id);
    
    // Deploy optimized assessment contract
    let optimized_contract_id = env.register(GasOptimizedAssessment, ());
    let optimized_client = GasOptimizedAssessmentClient::new(env, &optimized_contract_id);
    
    let admin = Address::generate(env);
    
    // Initialize both contracts
    original_client.initialize(&admin);
    optimized_client.initialize(&admin);
    
    (original_client, optimized_client, admin)
}

/// Create test assessment configuration
fn create_test_assessment_config() -> AssessmentConfig {
    AssessmentConfig {
        time_limit_seconds: 3600, // 1 hour
        max_attempts: 3,
        pass_score: 70,
        allow_review: true,
        is_adaptive: false,
        proctoring_required: false,
    }
}

/// Create test questions
fn create_test_questions(env: &Env, count: u32) -> Vec<(QuestionType, u32, u32, BytesN<32>, Vec<QuestionOption>, AnswerKey)> {
    let mut questions = Vec::new(env);
    
    for i in 0..count {
        let question_type = match i % 4 {
            0 => QuestionType::SingleChoice,
            1 => QuestionType::MultipleChoice,
            2 => QuestionType::Numeric,
            _ => QuestionType::ShortText,
        };
        
        let max_score = 10;
        let difficulty = (i % 5) + 1;
        let content_hash = BytesN::from_array(env, &[(i % 256) as u8; 32]);
        
        let options = match question_type {
            QuestionType::SingleChoice | QuestionType::MultipleChoice => {
                let mut opts = Vec::new(env);
                for j in 0..4 {
                    opts.push_back(QuestionOption {
                        id: j as u32,
                        label: soroban_sdk::String::from_str(env, &format!("Option {}", j)),
                    });
                }
                opts
            },
            _ => Vec::new(env),
        };
        
        let answer_key = match question_type {
            QuestionType::SingleChoice => AnswerKey::SingleChoice(0),
            QuestionType::MultipleChoice => {
                let mut correct = Vec::new(env);
                correct.push_back(0);
                correct.push_back(2);
                AnswerKey::MultipleChoice(correct)
            },
            QuestionType::Numeric => AnswerKey::NumericRange(0, 100),
            QuestionType::ShortText => {
                let mut answers = Vec::new(env);
                answers.push_back(soroban_sdk::String::from_str(env, "answer1"));
                answers.push_back(soroban_sdk::String::from_str(env, "answer2"));
                AnswerKey::ShortText(answers)
            },
            _ => AnswerKey::Manual,
        };
        
        questions.push_back((question_type, max_score, difficulty, content_hash, options, answer_key));
    }
    
    questions
}

/// Create test submission answers
fn create_test_answers(env: &Env, count: u32) -> Vec<SubmittedAnswer> {
    let mut answers = Vec::new(env);
    
    for i in 0..count {
        let value = match i % 4 {
            0 => SubmittedAnswerValue::SingleChoice(0),
            1 => {
                let mut selections = Vec::new(env);
                selections.push_back(0);
                selections.push_back(2);
                SubmittedAnswerValue::MultipleChoice(selections)
            },
            2 => SubmittedAnswerValue::Numeric(42),
            _ => SubmittedAnswerValue::ShortText(soroban_sdk::String::from_str(env, "test answer")),
        };
        
        answers.push_back(SubmittedAnswer {
            question_id: i as u64 + 1,
            value,
        });
    }
    
    answers
}

#[test]
fn benchmark_assessment_creation() {
    let env = Env::default();
    let (original_client, optimized_client, admin) = setup_contracts(&env);
    let config = create_test_assessment_config();
    
    let config = BenchmarkConfig::default();
    
    // Benchmark original implementation
    let (original_results, original_gas) = measure_gas(&env, || {
        let mut assessment_ids = Vec::new(&env);
        for i in 0..config.num_assessments {
            let course_id = Symbol::from_str(&env, &format!("course{}", i));
            let module_id = Symbol::from_str(&env, &format!("module{}", i));
            
            let assessment_id = original_client.create_assessment(
                &admin,
                &course_id,
                &module_id,
                &config,
            ).unwrap();
            assessment_ids.push_back(assessment_id);
        }
        assessment_ids
    });
    
    // Benchmark optimized implementation
    let (optimized_results, optimized_gas) = measure_gas(&env, || {
        let mut assessment_ids = Vec::new(&env);
        for i in 0..config.num_assessments {
            let course_id = Symbol::from_str(&env, &format!("course{}", i));
            let module_id = Symbol::from_str(&env, &format!("module{}", i));
            
            let assessment_id = optimized_client.create_assessment_optimized(
                &admin,
                &course_id,
                &module_id,
                &config,
            ).unwrap();
            assessment_ids.push_back(assessment_id);
        }
        assessment_ids
    });
    
    let results = BenchmarkResults::new(original_gas, optimized_gas, config.num_assessments);
    results.print_summary("Assessment Creation");
    
    // Verify optimization achieved at least 20% savings
    assert!(results.savings_percentage >= 20.0, 
           "Expected at least 20% gas savings, got {:.2}%", results.savings_percentage);
}

#[test]
fn benchmark_question_addition() {
    let env = Env::default();
    let (original_client, optimized_client, admin) = setup_contracts(&env);
    let config = create_test_assessment_config();
    
    // Create assessment for both contracts
    let course_id = Symbol::from_str(&env, "benchmark_course");
    let module_id = Symbol::from_str(&env, "benchmark_module");
    
    let original_assessment_id = original_client.create_assessment(
        &admin, &course_id, &module_id, &config
    ).unwrap();
    
    let optimized_assessment_id = optimized_client.create_assessment_optimized(
        &admin, &course_id, &module_id, &config
    ).unwrap();
    
    let num_questions = 20;
    let questions = create_test_questions(&env, num_questions);
    
    // Benchmark original implementation (individual additions)
    let (original_results, original_gas) = measure_gas(&env, || {
        for (question_type, max_score, difficulty, content_hash, options, answer_key) in questions.iter() {
            original_client.add_question(
                &admin,
                original_assessment_id,
                question_type.clone(),
                *max_score,
                *difficulty,
                content_hash.clone(),
                options.clone(),
                answer_key.clone(),
            ).unwrap();
        }
    });
    
    // Benchmark optimized implementation (batch addition)
    let (optimized_results, optimized_gas) = measure_gas(&env, || {
        optimized_client.add_questions_batch(
            &admin,
            optimized_assessment_id,
            questions.clone(),
        ).unwrap();
    });
    
    let results = BenchmarkResults::new(original_gas, optimized_gas, num_questions);
    results.print_summary("Question Addition");
    
    // Batch operations should save significant gas
    assert!(results.savings_percentage >= 30.0, 
           "Expected at least 30% gas savings for batch operations, got {:.2}%", results.savings_percentage);
}

#[test]
fn benchmark_submission_process() {
    let env = Env::default();
    let (original_client, optimized_client, admin) = setup_contracts(&env);
    let config = create_test_assessment_config();
    
    // Create assessments with questions
    let course_id = Symbol::from_str(&env, "submission_course");
    let module_id = Symbol::from_str(&env, "submission_module");
    
    let original_assessment_id = original_client.create_assessment(
        &admin, &course_id, &module_id, &config
    ).unwrap();
    
    let optimized_assessment_id = optimized_client.create_assessment_optimized(
        &admin, &course_id, &module_id, &config
    ).unwrap();
    
    // Add questions
    let questions = create_test_questions(&env, 10);
    for (question_type, max_score, difficulty, content_hash, options, answer_key) in questions.iter() {
        original_client.add_question(
            &admin,
            original_assessment_id,
            question_type.clone(),
            *max_score,
            *difficulty,
            content_hash.clone(),
            options.clone(),
            answer_key.clone(),
        ).unwrap();
    }
    
    optimized_client.add_questions_batch(
        &admin,
        optimized_assessment_id,
        questions.clone(),
    ).unwrap();
    
    // Publish assessments
    original_client.publish_assessment(&admin, original_assessment_id).unwrap();
    optimized_client.publish_assessment(&admin, optimized_assessment_id).unwrap();
    
    let student = Address::generate(&env);
    let answers = create_test_answers(&env, 10);
    
    // Benchmark original submission process
    let (original_result, original_gas) = measure_gas(&env, || {
        let submission_id = original_client.start_submission(&student, original_assessment_id).unwrap();
        original_client.submit_answers(&student, &submission_id, &answers).unwrap()
    });
    
    // Benchmark optimized submission process
    let (optimized_result, optimized_gas) = measure_gas(&env, || {
        let submission_id = optimized_client.start_submission_optimized(&student, optimized_assessment_id).unwrap();
        optimized_client.submit_answers_optimized(&student, &submission_id, &answers).unwrap()
    });
    
    let results = BenchmarkResults::new(original_gas, optimized_gas, 1);
    results.print_summary("Submission Process");
    
    // Submission optimization should save gas
    assert!(results.savings_percentage >= 15.0, 
           "Expected at least 15% gas savings for submissions, got {:.2}%", results.savings_percentage);
}

#[test]
fn benchmark_batch_submissions() {
    let env = Env::default();
    let (_, optimized_client, admin) = setup_contracts(&env);
    let config = create_test_assessment_config();
    
    // Create assessment with questions
    let course_id = Symbol::from_str(&env, "batch_course");
    let module_id = Symbol::from_str(&env, "batch_module");
    
    let assessment_id = optimized_client.create_assessment_optimized(
        &admin, &course_id, &module_id, &config
    ).unwrap();
    
    optimized_client.add_questions_batch(
        &admin,
        assessment_id,
        create_test_questions(&env, 5),
    ).unwrap();
    
    optimized_client.publish_assessment(&admin, assessment_id).unwrap();
    
    let student = Address::generate(&env);
    let answers = create_test_answers(&env, 5);
    
    // Create multiple submissions
    let mut submissions = Vec::new(&env);
    for i in 0..5 {
        let submission_id = optimized_client.start_submission_optimized(&student, assessment_id).unwrap();
        submissions.push_back((submission_id, answers.clone()));
    }
    
    // Benchmark batch submission
    let (batch_result, batch_gas) = measure_gas(&env, || {
        optimized_client.batch_submit_answers(&student, submissions.clone()).unwrap()
    });
    
    // Benchmark individual submissions
    let mut individual_gas_total = 0;
    for (submission_id, answers) in submissions.iter() {
        let (_, gas_used) = measure_gas(&env, || {
            optimized_client.submit_answers_optimized(&student, submission_id, answers.clone()).unwrap()
        });
        individual_gas_total += gas_used;
    }
    
    let results = BenchmarkResults::new(individual_gas_total, batch_gas, 5);
    results.print_summary("Batch Submissions");
    
    // Batch operations should provide significant savings
    assert!(results.savings_percentage >= 25.0, 
           "Expected at least 25% gas savings for batch submissions, got {:.2}%", results.savings_percentage);
}

#[test]
fn benchmark_storage_operations() {
    let env = Env::default();
    
    // Test packed vs unpacked storage
    let test_data = vec![
        ("flag1", true),
        ("flag2", false),
        ("flag3", true),
        ("flag4", false),
        ("flag5", true),
    ];
    
    // Benchmark individual boolean storage
    let (individual_result, individual_gas) = measure_gas(&env, || {
        for (name, value) in test_data.iter() {
            let key = Symbol::from_str(&env, name);
            env.storage().instance().set(&key, value);
        }
    });
    
    // Benchmark packed boolean storage
    let (packed_result, packed_gas) = measure_gas(&env, || {
        let packed = StorageOptimizer::pack_flags(&test_data.iter().map(|(_, v)| *v).collect::<Vec<_>>());
        env.storage().instance().set(&Symbol::short("packed_flags"), &packed);
    });
    
    let results = BenchmarkResults::new(individual_gas, packed_gas, test_data.len() as u32);
    results.print_summary("Storage Operations");
    
    // Packed storage should save gas
    assert!(results.savings_percentage >= 40.0, 
           "Expected at least 40% gas savings for packed storage, got {:.2}%", results.savings_percentage);
}

#[test]
fn benchmark_gas_profiling() {
    let env = Env::default();
    let (_, optimized_client, admin) = setup_contracts(&env);
    let config = create_test_assessment_config();
    
    // Enable gas profiling
    GasProfiler::reset_profile(&env);
    
    // Execute operations with profiling
    let course_id = Symbol::from_str(&env, "profile_course");
    let module_id = Symbol::from_str(&env, "profile_module");
    
    profile_gas!(&env, Symbol::short("create_assessment"), {
        optimized_client.create_assessment_optimized(&admin, &course_id, &module_id, &config).unwrap();
    });
    
    profile_gas!(&env, Symbol::short("batch_questions"), {
        optimized_client.add_questions_batch(&admin, 1, create_test_questions(&env, 10)).unwrap();
    });
    
    // Get profiling results
    let profile = optimized_client.get_gas_profile().unwrap();
    let efficiency_report = optimized_client.get_gas_efficiency_report();
    let recommendations = optimized_client.get_optimization_recommendations();
    
    // Verify profiling data
    assert!(profile.total_measurements > 0);
    assert!(profile.average_gas_per_call > 0);
    
    println!("\n=== Gas Profiling Results ===");
    println!("Total Measurements: {}", profile.total_measurements);
    println!("Average Gas per Call: {}", profile.average_gas_per_call);
    println!("Peak Gas Consumption: {}", profile.peak_gas_consumption);
    
    println!("\nEfficiency Report:");
    for (func, avg_gas) in efficiency_report.iter() {
        println!("  {}: {} gas", func, avg_gas);
    }
    
    println!("\nOptimization Recommendations:");
    for recommendation in recommendations.iter() {
        println!("  {}", recommendation);
    }
}

#[test]
fn benchmark_comprehensive_workflow() {
    let env = Env::default();
    let (original_client, optimized_client, admin) = setup_contracts(&env);
    let config = create_test_assessment_config();
    
    // Comprehensive workflow: create assessment, add questions, publish, submit
    let workflow_steps = 4;
    
    // Benchmark original workflow
    let (original_result, original_gas) = measure_gas(&env, || {
        // Step 1: Create assessment
        let course_id = Symbol::from_str(&env, "workflow_course");
        let module_id = Symbol::from_str(&env, "workflow_module");
        let assessment_id = original_client.create_assessment(&admin, &course_id, &module_id, &config).unwrap();
        
        // Step 2: Add questions
        let questions = create_test_questions(&env, 5);
        for (question_type, max_score, difficulty, content_hash, options, answer_key) in questions.iter() {
            original_client.add_question(&admin, assessment_id, question_type.clone(), *max_score, *difficulty, content_hash.clone(), options.clone(), answer_key.clone()).unwrap();
        }
        
        // Step 3: Publish
        original_client.publish_assessment(&admin, assessment_id).unwrap();
        
        // Step 4: Submit
        let student = Address::generate(&env);
        let submission_id = original_client.start_submission(&student, assessment_id).unwrap();
        let answers = create_test_answers(&env, 5);
        original_client.submit_answers(&student, &submission_id, &answers).unwrap();
    });
    
    // Benchmark optimized workflow
    let (optimized_result, optimized_gas) = measure_gas(&env, || {
        // Step 1: Create assessment
        let course_id = Symbol::from_str(&env, "workflow_course_opt");
        let module_id = Symbol::from_str(&env, "workflow_module_opt");
        let assessment_id = optimized_client.create_assessment_optimized(&admin, &course_id, &module_id, &config).unwrap();
        
        // Step 2: Add questions (batch)
        let questions = create_test_questions(&env, 5);
        optimized_client.add_questions_batch(&admin, assessment_id, questions).unwrap();
        
        // Step 3: Publish
        optimized_client.publish_assessment(&admin, assessment_id).unwrap();
        
        // Step 4: Submit (optimized)
        let student = Address::generate(&env);
        let submission_id = optimized_client.start_submission_optimized(&student, assessment_id).unwrap();
        let answers = create_test_answers(&env, 5);
        optimized_client.submit_answers_optimized(&student, &submission_id, &answers).unwrap();
    });
    
    let results = BenchmarkResults::new(original_gas, optimized_gas, workflow_steps);
    results.print_summary("Comprehensive Workflow");
    
    // Overall workflow should show significant savings
    assert!(results.savings_percentage >= 25.0, 
           "Expected at least 25% gas savings for comprehensive workflow, got {:.2}%", results.savings_percentage);
}

#[test]
fn stress_test_gas_limits() {
    let env = Env::default();
    let (_, optimized_client, admin) = setup_contracts(&env);
    let config = create_test_assessment_config();
    
    // Test with maximum realistic data sizes
    let max_questions = 50; // Maximum reasonable questions per assessment
    let max_options = 10;    // Maximum options per question
    
    let course_id = Symbol::from_str(&env, "stress_course");
    let module_id = Symbol::from_str(&env, "stress_module");
    
    // Create assessment
    let assessment_id = optimized_client.create_assessment_optimized(&admin, &course_id, &module_id, &config).unwrap();
    
    // Add maximum questions with maximum options
    let mut large_questions = Vec::new(&env);
    for i in 0..max_questions {
        let mut options = Vec::new(&env);
        for j in 0..max_options {
            options.push_back(QuestionOption {
                id: j as u32,
                label: soroban_sdk::String::from_str(&env, &format!("Option {} for Question {}", j, i)),
            });
        }
        
        large_questions.push_back((
            QuestionType::MultipleChoice,
            10,
            3,
            BytesN::from_array(&env, &[(i % 256) as u8; 32]),
            options,
            AnswerKey::MultipleChoice({
                let mut correct = Vec::new(&env);
                correct.push_back(0);
                correct.push_back(1);
                correct
            }),
        ));
    }
    
    // Benchmark large batch operation
    let (result, gas_used) = measure_gas(&env, || {
        optimized_client.add_questions_batch(&admin, assessment_id, large_questions).unwrap()
    });
    
    println!("\n=== Stress Test Results ===");
    println!("Questions: {}", max_questions);
    println!("Options per Question: {}", max_options);
    println!("Total Gas Used: {}", gas_used);
    println!("Gas per Question: {}", gas_used / max_questions as u64);
    
    // Should handle large operations within reasonable gas limits
    assert!(gas_used < 1_000_000, "Gas usage exceeded reasonable limit for large batch operation");
    
    // Test submission with many answers
    optimized_client.publish_assessment(&admin, assessment_id).unwrap();
    let student = Address::generate(&env);
    let many_answers = create_test_answers(&env, max_questions);
    
    let (submission_result, submission_gas) = measure_gas(&env, || {
        let submission_id = optimized_client.start_submission_optimized(&student, assessment_id).unwrap();
        optimized_client.submit_answers_optimized(&student, &submission_id, many_answers).unwrap()
    });
    
    println!("Submission Gas Used: {}", submission_gas);
    println!("Gas per Answer: {}", submission_gas / max_questions as u64);
    
    assert!(submission_gas < 500_000, "Submission gas exceeded reasonable limit");
}
