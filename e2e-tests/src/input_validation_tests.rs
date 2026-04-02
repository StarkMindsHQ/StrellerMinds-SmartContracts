//! Comprehensive input validation tests for StrellerMinds contracts
//! 
//! This module tests all validation scenarios including edge cases,
//! boundary conditions, and security considerations.

#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, BytesN as _},
    Address, BytesN, Env, Symbol, String, Vec,
};
use shared::{
    validation::{CoreValidator, ValidationError, ValidationConfig},
    input_validation::ContractValidator,
};

use contracts::assessment::{
    AssessmentContract, AssessmentClient,
    types::{
        AssessmentConfig, QuestionType, QuestionOption, 
        AnswerKey, SubmittedAnswer, SubmittedAnswerValue
    }
};

use contracts::community::{
    CommunityContract, CommunityClient,
    types::ForumCategory
};

use contracts::certificate::{
    CertificateContract, CertificateClient,
    types::CertificateMetadata
};

/// Test helper to create valid assessment config
fn create_valid_assessment_config() -> AssessmentConfig {
    AssessmentConfig {
        time_limit_seconds: 3600, // 1 hour
        max_attempts: 3,
        pass_score: 70,
        allow_review: true,
        is_adaptive: false,
        proctoring_required: false,
    }
}

/// Test helper to create valid question options
fn create_valid_question_options(env: &Env, count: u32) -> Vec<QuestionOption> {
    let mut options = Vec::new(env);
    for i in 0..count {
        options.push_back(QuestionOption {
            id: i,
            label: String::from_str(env, &format!("Option {}", i)),
        });
    }
    options
}

/// Test helper to create valid submitted answers
fn create_valid_submitted_answers(env: &Env, count: u32) -> Vec<SubmittedAnswer> {
    let mut answers = Vec::new(env);
    for i in 0..count {
        answers.push_back(SubmittedAnswer {
            question_id: i as u64 + 1,
            value: SubmittedAnswerValue::SingleChoice(i % 4),
        });
    }
    answers
}

// ==================== CORE VALIDATOR TESTS ====================

#[test]
fn test_validate_address_valid() {
    let env = Env::default();
    let valid_address = Address::generate(&env);
    
    let result = CoreValidator::validate_address_with_env(&env, &valid_address, "test_address");
    assert!(result.is_ok());
}

#[test]
fn test_validate_address_zero_address() {
    let env = Env::default();
    // Create a zero address (all bytes are zero)
    let zero_address_bytes = BytesN::from_array(&env, &[0u8; 32]);
    let zero_address = Address::from_xdr(&env, &zero_address_bytes).unwrap();
    
    let result = CoreValidator::validate_address_with_env(&env, &zero_address, "test_address");
    assert!(matches!(result, Err(ValidationError::InvalidAddress { .. })));
}

#[test]
fn test_validate_numeric_ranges() {
    // Test valid ranges
    assert!(CoreValidator::validate_range(50, "test", 0, 100).is_ok());
    assert!(CoreValidator::validate_u32_range(50, "test", 0, 100).is_ok());
    
    // Test invalid ranges
    assert!(matches!(
        CoreValidator::validate_range(150, "test", 0, 100),
        Err(ValidationError::InvalidRange { .. })
    ));
    assert!(matches!(
        CoreValidator::validate_u32_range(150, "test", 0, 100),
        Err(ValidationError::InvalidRange { .. })
    ));
}

#[test]
fn test_validate_array_sizes() {
    let env = Env::default();
    
    // Test valid array sizes
    let valid_array = Vec::new(&env);
    let result = CoreValidator::validate_array_size(&valid_array, "test", 0, 10);
    assert!(result.is_ok());
    
    // Test invalid array sizes
    let large_array = Vec::new(&env);
    for i in 0..20 {
        large_array.push_back(i);
    }
    let result = CoreValidator::validate_array_size(&large_array, "test", 0, 10);
    assert!(matches!(result, Err(ValidationError::InvalidArraySize { .. })));
}

#[test]
fn test_validate_symbol() {
    let env = Env::default();
    
    // Test valid symbols
    let valid_symbol = Symbol::from_str(&env, "valid_symbol");
    let result = CoreValidator::validate_symbol(&valid_symbol, "test_symbol");
    assert!(result.is_ok());
    
    // Test invalid symbols
    let invalid_symbol = Symbol::from_str(&env, "invalid-symbol-with-dashes");
    let result = CoreValidator::validate_symbol(&invalid_symbol, "test_symbol");
    assert!(matches!(result, Err(ValidationError::InvalidSymbol { .. })));
}

#[test]
fn test_validate_specific_ranges() {
    // Test score validation
    assert!(CoreValidator::validate_score(50).is_ok());
    assert!(matches!(CoreValidator::validate_score(1500), Err(ValidationError::InvalidRange { .. })));
    
    // Test attempts validation
    assert!(CoreValidator::validate_attempts(3).is_ok());
    assert!(matches!(CoreValidator::validate_attempts(15), Err(ValidationError::InvalidRange { .. })));
    
    // Test time limit validation
    assert!(CoreValidator::validate_time_limit(3600).is_ok());
    assert!(matches!(CoreValidator::validate_time_limit(10), Err(ValidationError::InvalidRange { .. })));
    
    // Test difficulty validation
    assert!(CoreValidator::validate_difficulty(5).is_ok());
    assert!(matches!(CoreValidator::validate_difficulty(15), Err(ValidationError::InvalidRange { .. })));
}

// ==================== ASSESSMENT CONTRACT VALIDATION TESTS ====================

#[test]
fn test_validate_create_assessment_valid() {
    let env = Env::default();
    let instructor = Address::generate(&env);
    let course_id = Symbol::from_str(&env, "CS101");
    let module_id = Symbol::from_str(&env, "module1");
    let config = create_valid_assessment_config();
    
    let result = ContractValidator::validate_create_assessment(
        &env,
        &instructor,
        &course_id,
        &module_id,
        &config,
    );
    assert!(result.is_ok());
}

#[test]
fn test_validate_create_assessment_invalid_config() {
    let env = Env::default();
    let instructor = Address::generate(&env);
    let course_id = Symbol::from_str(&env, "CS101");
    let module_id = Symbol::from_str(&env, "module1");
    
    // Test invalid time limit
    let mut config = create_valid_assessment_config();
    config.time_limit_seconds = 10; // Too short
    
    let result = ContractValidator::validate_create_assessment(
        &env,
        &instructor,
        &course_id,
        &module_id,
        &config,
    );
    assert!(matches!(result, Err(ValidationError::InvalidRange { .. })));
    
    // Test invalid attempts
    config.time_limit_seconds = 3600;
    config.max_attempts = 15; // Too many
    
    let result = ContractValidator::validate_create_assessment(
        &env,
        &instructor,
        &course_id,
        &module_id,
        &config,
    );
    assert!(matches!(result, Err(ValidationError::InvalidRange { .. })));
    
    // Test invalid pass score
    config.max_attempts = 3;
    config.pass_score = 150; // Too high
    
    let result = ContractValidator::validate_create_assessment(
        &env,
        &instructor,
        &course_id,
        &module_id,
        &config,
    );
    assert!(matches!(result, Err(ValidationError::InvalidRange { .. })));
}

#[test]
fn test_validate_create_question_valid() {
    let env = Env::default();
    let content_hash = BytesN::from_array(&env, &[1u8; 32]);
    let options = create_valid_question_options(&env, 4);
    let answer_key = AnswerKey::SingleChoice(0);
    
    let result = ContractValidator::validate_create_question(
        &env,
        1,
        &QuestionType::SingleChoice,
        10,
        3,
        &content_hash,
        &options,
        &answer_key,
    );
    assert!(result.is_ok());
}

#[test]
fn test_validate_create_question_invalid() {
    let env = Env::default();
    let content_hash = BytesN::from_array(&env, &[1u8; 32]);
    
    // Test invalid assessment ID
    let result = ContractValidator::validate_create_question(
        &env,
        0, // Invalid
        &QuestionType::SingleChoice,
        10,
        3,
        &content_hash,
        &create_valid_question_options(&env, 4),
        &AnswerKey::SingleChoice(0),
    );
    assert!(matches!(result, Err(ValidationError::InvalidRange { .. })));
    
    // Test invalid score
    let result = ContractValidator::validate_create_question(
        &env,
        1,
        &QuestionType::SingleChoice,
        1500, // Too high
        3,
        &content_hash,
        &create_valid_question_options(&env, 4),
        &AnswerKey::SingleChoice(0),
    );
    assert!(matches!(result, Err(ValidationError::InvalidRange { .. })));
    
    // Test invalid difficulty
    let result = ContractValidator::validate_create_question(
        &env,
        1,
        &QuestionType::SingleChoice,
        10,
        15, // Too high
        &content_hash,
        &create_valid_question_options(&env, 4),
        &AnswerKey::SingleChoice(0),
    );
    assert!(matches!(result, Err(ValidationError::InvalidRange { .. })));
    
    // Test zero content hash
    let zero_hash = BytesN::from_array(&env, &[0u8; 32]);
    let result = ContractValidator::validate_create_question(
        &env,
        1,
        &QuestionType::SingleChoice,
        10,
        3,
        &zero_hash, // Invalid
        &create_valid_question_options(&env, 4),
        &AnswerKey::SingleChoice(0),
    );
    assert!(matches!(result, Err(ValidationError::EmptyField { .. })));
}

#[test]
fn test_validate_question_options_by_type() {
    let env = Env::default();
    
    // Test single choice with valid options
    let options = create_valid_question_options(&env, 4);
    let result = ContractValidator::validate_question_options_for_type(
        &QuestionType::SingleChoice,
        &options,
    );
    assert!(result.is_ok());
    
    // Test single choice with too few options
    let few_options = create_valid_question_options(&env, 1);
    let result = ContractValidator::validate_question_options_for_type(
        &QuestionType::SingleChoice,
        &few_options,
    );
    assert!(matches!(result, Err(ValidationError::InvalidArraySize { .. })));
    
    // Test single choice with too many options
    let many_options = create_valid_question_options(&env, 15);
    let result = ContractValidator::validate_question_options_for_type(
        &QuestionType::SingleChoice,
        &many_options,
    );
    assert!(matches!(result, Err(ValidationError::InvalidArraySize { .. })));
    
    // Test numeric question with options (should fail)
    let result = ContractValidator::validate_question_options_for_type(
        &QuestionType::Numeric,
        &options,
    );
    assert!(matches!(result, Err(ValidationError::InvalidArraySize { .. })));
}

#[test]
fn test_validate_answer_key_by_type() {
    let env = Env::default();
    let options = create_valid_question_options(&env, 4);
    
    // Test single choice with valid option
    let result = ContractValidator::validate_answer_key_for_type(
        &QuestionType::SingleChoice,
        &AnswerKey::SingleChoice(0),
        &options,
    );
    assert!(result.is_ok());
    
    // Test single choice with invalid option
    let result = ContractValidator::validate_answer_key_for_type(
        &QuestionType::SingleChoice,
        &AnswerKey::SingleChoice(10), // Doesn't exist
        &options,
    );
    assert!(matches!(result, Err(ValidationError::InvalidRange { .. })));
    
    // Test multiple choice with valid options
    let selected_options = Vec::from_array(&env, &[0u32, 2]);
    let result = ContractValidator::validate_answer_key_for_type(
        &QuestionType::MultipleChoice,
        &AnswerKey::MultipleChoice(selected_options),
        &options,
    );
    assert!(result.is_ok());
    
    // Test numeric range with valid range
    let result = ContractValidator::validate_answer_key_for_type(
        &QuestionType::Numeric,
        &AnswerKey::NumericRange(0, 100),
        &options,
    );
    assert!(result.is_ok());
    
    // Test numeric range with invalid range (min >= max)
    let result = ContractValidator::validate_answer_key_for_type(
        &QuestionType::Numeric,
        &AnswerKey::NumericRange(100, 50),
        &options,
    );
    assert!(matches!(result, Err(ValidationError::InvalidRange { .. })));
    
    // Test short text with valid answers
    let answers = Vec::from_array(&env, &[
        String::from_str(&env, "answer1"),
        String::from_str(&env, "answer2"),
    ]);
    let result = ContractValidator::validate_answer_key_for_type(
        &QuestionType::ShortText,
        &AnswerKey::ShortText(answers),
        &options,
    );
    assert!(result.is_ok());
}

#[test]
fn test_validate_submit_answers_valid() {
    let env = Env::default();
    let student = Address::generate(&env);
    let submission_id = BytesN::from_array(&env, &[1u8; 32]);
    let answers = create_valid_submitted_answers(&env, 5);
    
    let result = ContractValidator::validate_submit_answers(
        &env,
        &student,
        &submission_id,
        &answers,
    );
    assert!(result.is_ok());
}

#[test]
fn test_validate_submit_answers_invalid() {
    let env = Env::default();
    let student = Address::generate(&env);
    let submission_id = BytesN::from_array(&env, &[1u8; 32]);
    
    // Test zero submission ID
    let zero_id = BytesN::from_array(&env, &[0u8; 32]);
    let answers = create_valid_submitted_answers(&env, 5);
    let result = ContractValidator::validate_submit_answers(
        &env,
        &student,
        &zero_id,
        &answers,
    );
    assert!(matches!(result, Err(ValidationError::EmptyField { .. })));
    
    // Test too many answers
    let many_answers = create_valid_submitted_answers(&env, 150);
    let result = ContractValidator::validate_submit_answers(
        &env,
        &student,
        &submission_id,
        &many_answers,
    );
    assert!(matches!(result, Err(ValidationError::InvalidArraySize { .. })));
    
    // Test duplicate question IDs
    let mut duplicate_answers = Vec::new(&env);
    duplicate_answers.push_back(SubmittedAnswer {
        question_id: 1,
        value: SubmittedAnswerValue::SingleChoice(0),
    });
    duplicate_answers.push_back(SubmittedAnswer {
        question_id: 1, // Duplicate
        value: SubmittedAnswerValue::SingleChoice(1),
    });
    
    let result = ContractValidator::validate_submit_answers(
        &env,
        &student,
        &submission_id,
        &duplicate_answers,
    );
    assert!(matches!(result, Err(ValidationError::DuplicateValue { .. })));
}

#[test]
fn test_validate_submitted_answer_values() {
    let env = Env::default();
    
    // Test valid single choice
    let answer = SubmittedAnswer {
        question_id: 1,
        value: SubmittedAnswerValue::SingleChoice(0),
    };
    let result = ContractValidator::validate_submitted_answer(&env, &answer);
    assert!(result.is_ok());
    
    // Test invalid single choice (too high)
    let invalid_answer = SubmittedAnswer {
        question_id: 1,
        value: SubmittedAnswerValue::SingleChoice(2000),
    };
    let result = ContractValidator::validate_submitted_answer(&env, &invalid_answer);
    assert!(matches!(result, Err(ValidationError::InvalidRange { .. })));
    
    // Test valid numeric answer
    let numeric_answer = SubmittedAnswer {
        question_id: 1,
        value: SubmittedAnswerValue::Numeric(42),
    };
    let result = ContractValidator::validate_submitted_answer(&env, &numeric_answer);
    assert!(result.is_ok());
    
    // Test invalid numeric answer (out of range)
    let invalid_numeric = SubmittedAnswer {
        question_id: 1,
        value: SubmittedAnswerValue::Numeric(2000000),
    };
    let result = ContractValidator::validate_submitted_answer(&env, &invalid_numeric);
    assert!(matches!(result, Err(ValidationError::InvalidRange { .. })));
    
    // Test valid text answer
    let text_answer = SubmittedAnswer {
        question_id: 1,
        value: SubmittedAnswerValue::ShortText(String::from_str(&env, "Valid answer")),
    };
    let result = ContractValidator::validate_submitted_answer(&env, &text_answer);
    assert!(result.is_ok());
    
    // Test invalid text answer (too long)
    let long_text = "x".repeat(600);
    let long_answer = SubmittedAnswer {
        question_id: 1,
        value: SubmittedAnswerValue::ShortText(String::from_str(&env, &long_text)),
    };
    let result = ContractValidator::validate_submitted_answer(&env, &long_answer);
    assert!(matches!(result, Err(ValidationError::FieldTooLong { .. })));
}

// ==================== COMMUNITY CONTRACT VALIDATION TESTS ====================

#[test]
fn test_validate_create_post_valid() {
    let env = Env::default();
    let author = Address::generate(&env);
    let category = ForumCategory::General;
    let title = String::from_str(&env, "Valid Post Title");
    let content = String::from_str(&env, "This is valid post content with sufficient length.");
    let tags = Vec::from_array(&env, &[
        String::from_str(&env, "tag1"),
        String::from_str(&env, "tag2"),
    ]);
    let course_id = String::from_str(&env, "CS101");
    
    let result = ContractValidator::validate_create_post(
        &env,
        &author,
        &category,
        &title,
        &content,
        &tags,
        &course_id,
    );
    assert!(result.is_ok());
}

#[test]
fn test_validate_create_post_invalid() {
    let env = Env::default();
    let author = Address::generate(&env);
    let category = ForumCategory::General;
    let valid_tags = Vec::from_array(&env, &[
        String::from_str(&env, "tag1"),
        String::from_str(&env, "tag2"),
    ]);
    
    // Test title too short
    let short_title = String::from_str(&env, "Hi");
    let result = ContractValidator::validate_create_post(
        &env,
        &author,
        &category,
        &short_title,
        &String::from_str(&env, "Valid content"),
        &valid_tags,
        &String::from_str(&env, "CS101"),
    );
    assert!(matches!(result, Err(ValidationError::FieldTooShort { .. })));
    
    // Test title too long
    let long_title = "x".repeat(250);
    let result = ContractValidator::validate_create_post(
        &env,
        &author,
        &category,
        &String::from_str(&env, &long_title),
        &String::from_str(&env, "Valid content"),
        &valid_tags,
        &String::from_str(&env, "CS101"),
    );
    assert!(matches!(result, Err(ValidationError::FieldTooLong { .. })));
    
    // Test content too short
    let result = ContractValidator::validate_create_post(
        &env,
        &author,
        &category,
        &String::from_str(&env, "Valid title"),
        &String::from_str(&env, "Short"),
        &valid_tags,
        &String::from_str(&env, "CS101"),
    );
    assert!(matches!(result, Err(ValidationError::FieldTooShort { .. })));
    
    // Test too many tags
    let mut many_tags = Vec::new(&env);
    for i in 0..15 {
        many_tags.push_back(String::from_str(&env, &format!("tag{}", i)));
    }
    let result = ContractValidator::validate_create_post(
        &env,
        &author,
        &category,
        &String::from_str(&env, "Valid title"),
        &String::from_str(&env, "Valid content with sufficient length."),
        &many_tags,
        &String::from_str(&env, "CS101"),
    );
    assert!(matches!(result, Err(ValidationError::InvalidArraySize { .. })));
    
    // Test invalid course ID
    let result = ContractValidator::validate_create_post(
        &env,
        &author,
        &category,
        &String::from_str(&env, "Valid title"),
        &String::from_str(&env, "Valid content with sufficient length."),
        &valid_tags,
        &String::from_str(&env, "invalid@course"),
    );
    assert!(matches!(result, Err(ValidationError::InvalidFormat { .. })));
}

#[test]
fn test_validate_award_reputation_valid() {
    let env = Env::default();
    let from = Address::generate(&env);
    let to = Address::generate(&env);
    let amount = 50;
    let reason = String::from_str(&env, "Helpful contribution to the community");
    
    let result = ContractValidator::validate_award_reputation(
        &env,
        &from,
        &to,
        amount,
        &reason,
    );
    assert!(result.is_ok());
}

#[test]
fn test_validate_award_reputation_invalid() {
    let env = Env::default();
    let address = Address::generate(&env);
    let valid_reason = String::from_str(&env, "Valid reason for reputation award");
    
    // Test awarding to self
    let result = ContractValidator::validate_award_reputation(
        &env,
        &address,
        &address, // Same address
        50,
        &valid_reason,
    );
    assert!(matches!(result, Err(ValidationError::InvalidAddress { .. })));
    
    // Test invalid amount
    let to = Address::generate(&env);
    let result = ContractValidator::validate_award_reputation(
        &env,
        &address,
        &to,
        2000000, // Too high
        &valid_reason,
    );
    assert!(matches!(result, Err(ValidationError::InvalidRange { .. })));
    
    // Test reason too short
    let result = ContractValidator::validate_award_reputation(
        &env,
        &address,
        &to,
        50,
        &String::from_str(&env, "Bad"),
    );
    assert!(matches!(result, Err(ValidationError::FieldTooShort { .. })));
    
    // Test reason too long
    let long_reason = "x".repeat(250);
    let result = ContractValidator::validate_award_reputation(
        &env,
        &address,
        &to,
        50,
        &String::from_str(&env, &long_reason),
    );
    assert!(matches!(result, Err(ValidationError::FieldTooLong { .. })));
}

// ==================== CERTIFICATE CONTRACT VALIDATION TESTS ====================

#[test]
fn test_validate_mint_certificate_valid() {
    let env = Env::default();
    let recipient = Address::generate(&env);
    let issuer = Address::generate(&env);
    let template_id = BytesN::from_array(&env, &[1u8; 32]);
    
    let metadata = CertificateMetadata {
        title: String::from_str(&env, "Certificate of Completion"),
        description: String::from_str(&env, "This certificate is awarded for successfully completing the course."),
        expiry_date: 0, // No expiry
        skills: Vec::from_array(&env, &[
            String::from_str(&env, "Rust"),
            String::from_str(&env, "Blockchain"),
        ]),
    };
    
    let result = ContractValidator::validate_mint_certificate(
        &env,
        &recipient,
        &issuer,
        &template_id,
        &metadata,
    );
    assert!(result.is_ok());
}

#[test]
fn test_validate_mint_certificate_invalid() {
    let env = Env::default();
    let recipient = Address::generate(&env);
    let issuer = Address::generate(&env);
    
    // Test zero template ID
    let zero_template = BytesN::from_array(&env, &[0u8; 32]);
    let valid_metadata = CertificateMetadata {
        title: String::from_str(&env, "Certificate"),
        description: String::from_str(&env, "Valid description"),
        expiry_date: 0,
        skills: Vec::new(&env),
    };
    
    let result = ContractValidator::validate_mint_certificate(
        &env,
        &recipient,
        &issuer,
        &zero_template,
        &valid_metadata,
    );
    assert!(matches!(result, Err(ValidationError::EmptyField { .. })));
    
    // Test invalid metadata
    let template_id = BytesN::from_array(&env, &[1u8; 32]);
    let invalid_metadata = CertificateMetadata {
        title: String::from_str(&env, "Hi"), // Too short
        description: String::from_str(&env, "Valid description"),
        expiry_date: 0,
        skills: Vec::new(&env),
    };
    
    let result = ContractValidator::validate_mint_certificate(
        &env,
        &recipient,
        &issuer,
        &template_id,
        &invalid_metadata,
    );
    assert!(matches!(result, Err(ValidationError::FieldTooShort { .. })));
}

// ==================== SCHEDULE VALIDATION TESTS ====================

#[test]
fn test_validate_schedule_config_valid() {
    let env = Env::default();
    env.ledger().set_timestamp(1000000); // Set current time
    
    let start_time = 1000000 + 3600; // 1 hour from now
    let end_time = start_time + 86400; // 1 day later
    let time_zone_offset = -300; // EST (UTC-5)
    
    let result = ContractValidator::validate_schedule_config(
        &env,
        start_time,
        end_time,
        time_zone_offset,
    );
    assert!(result.is_ok());
}

#[test]
fn test_validate_schedule_config_invalid() {
    let env = Env::default();
    env.ledger().set_timestamp(1000000);
    
    // Test start time >= end time
    let result = ContractValidator::validate_schedule_config(
        &env,
        2000000,
        1000000, // End before start
        0,
    );
    assert!(matches!(result, Err(ValidationError::InvalidRange { .. })));
    
    // Test start time too far in past
    let past_time = 1000000 - (86400 * 31); // 31 days ago
    let result = ContractValidator::validate_schedule_config(
        &env,
        past_time,
        past_time + 86400,
        0,
    );
    assert!(matches!(result, Err(ValidationError::InvalidDate { .. })));
    
    // Test end time too far in future
    let future_time = 1000000 + (ValidationConfig::MAX_FUTURE_EXPIRY + 86400);
    let result = ContractValidator::validate_schedule_config(
        &env,
        1000000 + 3600,
        future_time,
        0,
    );
    assert!(matches!(result, Err(ValidationError::InvalidDate { .. })));
    
    // Test invalid timezone offset
    let result = ContractValidator::validate_schedule_config(
        &env,
        1000000 + 3600,
        1000000 + 86400,
        -1000, // Too far negative
    );
    assert!(matches!(result, Err(ValidationError::InvalidRange { .. })));
    
    let result = ContractValidator::validate_schedule_config(
        &env,
        1000000 + 3600,
        1000000 + 86400,
        1000, // Too far positive
    );
    assert!(matches!(result, Err(ValidationError::InvalidRange { .. })));
}

// ==================== BATCH OPERATION VALIDATION TESTS ====================

#[test]
fn test_validate_batch_operation_valid() {
    let env = Env::default();
    let batch_items = Vec::from_array(&env, &[1u32, 2, 3, 4, 5]);
    
    let result = ContractValidator::validate_batch_operation(
        &env,
        &batch_items,
        "test_batch",
        10,
    );
    assert!(result.is_ok());
}

#[test]
fn test_validate_batch_operation_invalid() {
    let env = Env::default();
    
    // Test empty batch
    let empty_batch = Vec::new(&env);
    let result = ContractValidator::validate_batch_operation(
        &env,
        &empty_batch,
        "test_batch",
        10,
    );
    assert!(matches!(result, Err(ValidationError::InvalidArraySize { .. })));
    
    // Test batch too large
    let mut large_batch = Vec::new(&env);
    for i in 0..100 {
        large_batch.push_back(i);
    }
    let result = ContractValidator::validate_batch_operation(
        &env,
        &large_batch,
        "test_batch",
        50,
    );
    assert!(matches!(result, Err(ValidationError::InvalidBatchSize { .. })));
}

// ==================== ACCOMMODATION VALIDATION TESTS ====================

#[test]
fn test_validate_accommodation_config_valid() {
    let result = ContractValidator::validate_accommodation_config(
        25,  // 25% extra time
        2,   // 2 extra attempts
    );
    assert!(result.is_ok());
}

#[test]
fn test_validate_accommodation_config_invalid() {
    // Test extra time percent too high
    let result = ContractValidator::validate_accommodation_config(
        250, // Too high
        2,
    );
    assert!(matches!(result, Err(ValidationError::InvalidRange { .. })));
    
    // Test extra attempts too high
    let result = ContractValidator::validate_accommodation_config(
        25,
        15, // Too high
    );
    assert!(matches!(result, Err(ValidationError::InvalidRange { .. })));
}

// ==================== INTEGRATION TESTS ====================

#[test]
fn test_contract_integration_with_validation() {
    let env = Env::default();
    env.mock_all_auths();
    
    // Deploy assessment contract
    let contract_id = env.register(AssessmentContract, ());
    let client = AssessmentClient::new(&env, &contract_id);
    
    let admin = Address::generate(&env);
    let instructor = Address::generate(&env);
    
    // Initialize contract
    client.initialize(&admin).unwrap();
    
    // Test creating assessment with invalid config (should fail validation)
    let invalid_config = AssessmentConfig {
        time_limit_seconds: 10, // Too short
        max_attempts: 3,
        pass_score: 70,
        allow_review: true,
        is_adaptive: false,
        proctoring_required: false,
    };
    
    let course_id = Symbol::from_str(&env, "CS101");
    let module_id = Symbol::from_str(&env, "module1");
    
    // This should fail due to validation
    let result = client.try_create_assessment(
        &instructor,
        &course_id,
        &module_id,
        &invalid_config,
    );
    
    // The exact error will depend on how the contract integrates validation
    // For now, we just verify that some error occurs
    assert!(result.is_err());
}

#[test]
fn test_edge_cases_and_boundary_conditions() {
    let env = Env::default();
    
    // Test boundary values for ranges
    assert!(CoreValidator::validate_score(0).is_ok()); // Minimum
    assert!(CoreValidator::validate_score(1000).is_ok()); // Maximum
    assert!(CoreValidator::validate_score(1001).is_err()); // Above maximum
    
    assert!(CoreValidator::validate_attempts(1).is_ok()); // Minimum
    assert!(CoreValidator::validate_attempts(10).is_ok()); // Maximum
    assert!(CoreValidator::validate_attempts(11).is_err()); // Above maximum
    
    assert!(CoreValidator::validate_time_limit(60).is_ok()); // Minimum
    assert!(CoreValidator::validate_time_limit(604800).is_ok()); // Maximum (7 days)
    assert!(CoreValidator::validate_time_limit(59).is_err()); // Below minimum
    assert!(CoreValidator::validate_time_limit(604801).is_err()); // Above maximum
    
    assert!(CoreValidator::validate_difficulty(1).is_ok()); // Minimum
    assert!(CoreValidator::validate_difficulty(10).is_ok()); // Maximum
    assert!(CoreValidator::validate_difficulty(0).is_err()); // Below minimum
    assert!(CoreValidator::validate_difficulty(11).is_err()); // Above maximum
}

#[test]
fn test_security_validation_scenarios() {
    let env = Env::default();
    
    // Test XSS prevention in text fields
    let xss_text = "<script>alert('xss')</script>";
    let result = CoreValidator::validate_no_forbidden_chars(xss_text, "test_field");
    assert!(matches!(result, Err(ValidationError::InvalidCharacters { .. })));
    
    // Test SQL injection patterns
    let sql_text = "'; DROP TABLE users; --";
    let result = CoreValidator::validate_no_forbidden_chars(sql_text, "test_field");
    assert!(matches!(result, Err(ValidationError::InvalidCharacters { .. })));
    
    // Test excessive special characters
    let spam_text = "!@#$%^&*()!@#$%^&*()!@#$%^&*()";
    let result = CoreValidator::validate_text_quality(spam_text, "test_field");
    assert!(matches!(result, Err(ValidationError::ContentQuality { .. })));
    
    // Test excessive repetition
    let repeat_text = "aaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let result = CoreValidator::validate_text_quality(repeat_text, "test_field");
    assert!(matches!(result, Err(ValidationError::ContentQuality { .. })));
}

#[test]
fn test_performance_with_large_inputs() {
    let env = Env::default();
    
    // Test validation performance with large arrays
    let mut large_array = Vec::new(&env);
    for i in 0..1000 {
        large_array.push_back(i);
    }
    
    let start = std::time::Instant::now();
    let result = CoreValidator::validate_array_size(&large_array, "large_array", 0, 1000);
    let duration = start.elapsed();
    
    assert!(result.is_ok());
    assert!(duration.as_millis() < 100, "Validation should be fast even with large inputs");
}

#[test]
fn test_error_message_quality() {
    // Test that error messages are informative
    let result = CoreValidator::validate_string_length("ab", "test_field", 3, 10);
    if let Err(ValidationError::FieldTooShort { field, min_length, actual_length }) = result {
        assert_eq!(field, "test_field");
        assert_eq!(min_length, 3);
        assert_eq!(actual_length, 2);
    } else {
        panic!("Expected FieldTooShort error");
    }
    
    let result = CoreValidator::validate_range(150, "test_field", 0, 100);
    if let Err(ValidationError::InvalidRange { field, min, max, actual }) = result {
        assert_eq!(field, "test_field");
        assert_eq!(min, 0);
        assert_eq!(max, 100);
        assert_eq!(actual, 150);
    } else {
        panic!("Expected InvalidRange error");
    }
}
