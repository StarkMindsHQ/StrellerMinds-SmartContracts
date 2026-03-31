//! Comprehensive input validation for all StrellerMinds contracts
//! 
//! This module provides contract-specific validation utilities that integrate
//! with the core validation framework to ensure all inputs are properly validated.

#![no_std]

use soroban_sdk::{
    Address, BytesN, Env, Symbol, String, Vec, Map,
    xdr::ToXdr
};

use crate::validation::{
    CoreValidator, ValidationError, ValidationConfig
};

/// Contract-specific validation utilities
pub struct ContractValidator;

impl ContractValidator {
    // ==================== ASSESSMENT CONTRACT VALIDATION ====================
    
    /// Validates assessment creation parameters
    pub fn validate_create_assessment(
        env: &Env,
        instructor: &Address,
        course_id: &Symbol,
        module_id: &Symbol,
        config: &crate::contracts::assessment::types::AssessmentConfig,
    ) -> Result<(), ValidationError> {
        // Validate addresses
        CoreValidator::validate_address_with_env(env, instructor, "instructor")?;
        
        // Validate symbols
        CoreValidator::validate_symbol(course_id, "course_id")?;
        CoreValidator::validate_symbol(module_id, "module_id")?;
        
        // Validate assessment config
        Self::validate_assessment_config(env, config)?;
        
        Ok(())
    }
    
    /// Validates assessment configuration
    pub fn validate_assessment_config(
        env: &Env,
        config: &crate::contracts::assessment::types::AssessmentConfig,
    ) -> Result<(), ValidationError> {
        // Validate time limit
        CoreValidator::validate_time_limit(config.time_limit_seconds)?;
        
        // Validate attempts
        CoreValidator::validate_attempts(config.max_attempts)?;
        
        // Validate pass score
        CoreValidator::validate_score(config.pass_score)?;
        
        // Pass score should not exceed maximum possible score
        if config.pass_score > 100 {
            return Err(ValidationError::InvalidRange {
                field: "pass_score",
                min: 0,
                max: 100,
                actual: config.pass_score as u64,
            });
        }
        
        Ok(())
    }
    
    /// Validates question creation parameters
    pub fn validate_create_question(
        env: &Env,
        assessment_id: u64,
        question_type: &crate::contracts::assessment::types::QuestionType,
        max_score: u32,
        difficulty: u32,
        content_hash: &BytesN<32>,
        options: &Vec<crate::contracts::assessment::types::QuestionOption>,
        answer_key: &crate::contracts::assessment::types::AnswerKey,
    ) -> Result<(), ValidationError> {
        // Validate assessment ID (basic range check)
        if assessment_id == 0 {
            return Err(ValidationError::InvalidRange {
                field: "assessment_id",
                min: 1,
                max: u64::MAX,
                actual: 0,
            });
        }
        
        // Validate score and difficulty
        CoreValidator::validate_score(max_score)?;
        CoreValidator::validate_difficulty(difficulty)?;
        
        // Validate content hash
        CoreValidator::validate_certificate_id(content_hash)?;
        
        // Validate options based on question type
        Self::validate_question_options_for_type(question_type, options)?;
        
        // Validate answer key
        Self::validate_answer_key_for_type(question_type, answer_key, options)?;
        
        Ok(())
    }
    
    /// Validates question options based on question type
    pub fn validate_question_options_for_type(
        question_type: &crate::contracts::assessment::types::QuestionType,
        options: &Vec<crate::contracts::assessment::types::QuestionOption>,
    ) -> Result<(), ValidationError> {
        match question_type {
            crate::contracts::assessment::types::QuestionType::SingleChoice |
            crate::contracts::assessment::types::QuestionType::MultipleChoice => {
                CoreValidator::validate_question_options(options, "question_options")?;
                
                // Validate each option
                for (i, option) in options.iter().enumerate() {
                    // Validate option ID is within reasonable range
                    if option.id > 1000 {
                        return Err(ValidationError::InvalidRange {
                            field: "option_id",
                            min: 0,
                            max: 1000,
                            actual: option.id as u64,
                        });
                    }
                    
                    // Validate option label
                    let label_str = option.label.to_string();
                    CoreValidator::validate_text_field(
                        &label_str,
                        "option_label",
                        1,
                        100,
                    )?;
                }
            },
            crate::contracts::assessment::types::QuestionType::Numeric |
            crate::contracts::assessment::types::QuestionType::ShortText |
            crate::contracts::assessment::types::QuestionType::Essay |
            crate::contracts::assessment::types::QuestionType::Code => {
                // These question types don't require options
                if !options.is_empty() {
                    return Err(ValidationError::InvalidArraySize {
                        field: "question_options",
                        min: 0,
                        max: 0,
                        actual: options.len() as u32,
                    });
                }
            }
        }
        
        Ok(())
    }
    
    /// Validates answer key based on question type
    pub fn validate_answer_key_for_type(
        question_type: &crate::contracts::assessment::types::QuestionType,
        answer_key: &crate::contracts::assessment::types::AnswerKey,
        options: &Vec<crate::contracts::assessment::types::QuestionOption>,
    ) -> Result<(), ValidationError> {
        match answer_key {
            crate::contracts::assessment::types::AnswerKey::SingleChoice(option_id) => {
                // Validate option exists
                if !options.iter().any(|opt| opt.id == *option_id) {
                    return Err(ValidationError::InvalidRange {
                        field: "answer_option_id",
                        min: 0,
                        max: options.len() as u64,
                        actual: *option_id as u64,
                    });
                }
            },
            crate::contracts::assessment::types::AnswerKey::MultipleChoice(option_ids) => {
                CoreValidator::validate_array_size(option_ids, "correct_options", 1, options.len() as u32)?;
                
                // Validate all options exist
                for option_id in option_ids.iter() {
                    if !options.iter().any(|opt| opt.id == *option_id) {
                        return Err(ValidationError::InvalidRange {
                            field: "answer_option_id",
                            min: 0,
                            max: options.len() as u64,
                            actual: *option_id as u64,
                        });
                    }
                }
            },
            crate::contracts::assessment::types::AnswerKey::NumericRange(min, max) => {
                if min >= max {
                    return Err(ValidationError::InvalidRange {
                        field: "numeric_range",
                        min: 0,
                        max: *max as u64,
                        actual: *min as u64,
                    });
                }
                
                // Validate range is reasonable
                if max - min > 1000000 {
                    return Err(ValidationError::InvalidRange {
                        field: "numeric_range_width",
                        min: 0,
                        max: 1000000,
                        actual: (max - min) as u64,
                    });
                }
            },
            crate::contracts::assessment::types::AnswerKey::ShortText(answers) => {
                CoreValidator::validate_array_size(answers, "accepted_answers", 1, 20)?;
                
                // Validate each answer
                for answer in answers.iter() {
                    let answer_str = answer.to_string();
                    CoreValidator::validate_text_field(
                        &answer_str,
                        "accepted_answer",
                        1,
                        200,
                    )?;
                }
            },
            crate::contracts::assessment::types::AnswerKey::Manual => {
                // Manual grading requires no validation
            }
        }
        
        Ok(())
    }
    
    /// Validates submission parameters
    pub fn validate_start_submission(
        env: &Env,
        student: &Address,
        assessment_id: u64,
    ) -> Result<(), ValidationError> {
        // Validate addresses
        CoreValidator::validate_address_with_env(env, student, "student")?;
        
        // Validate assessment ID
        if assessment_id == 0 {
            return Err(ValidationError::InvalidRange {
                field: "assessment_id",
                min: 1,
                max: u64::MAX,
                actual: 0,
            });
        }
        
        Ok(())
    }
    
    /// Validates submitted answers
    pub fn validate_submit_answers(
        env: &Env,
        student: &Address,
        submission_id: &BytesN<32>,
        answers: &Vec<crate::contracts::assessment::types::SubmittedAnswer>,
    ) -> Result<(), ValidationError> {
        // Validate addresses
        CoreValidator::validate_address_with_env(env, student, "student")?;
        
        // Validate submission ID
        CoreValidator::validate_certificate_id(submission_id)?;
        
        // Validate answers array
        CoreValidator::validate_submission_answers(answers, "submitted_answers")?;
        
        // Validate each answer
        for answer in answers.iter() {
            Self::validate_submitted_answer(env, answer)?;
        }
        
        // Check for duplicate question IDs
        let mut question_ids = Vec::new(env);
        for answer in answers.iter() {
            if question_ids.iter().any(|&id| id == answer.question_id) {
                return Err(ValidationError::DuplicateValue {
                    field: "question_id",
                    value: format!("{}", answer.question_id),
                });
            }
            question_ids.push_back(answer.question_id);
        }
        
        Ok(())
    }
    
    /// Validates individual submitted answer
    pub fn validate_submitted_answer(
        env: &Env,
        answer: &crate::contracts::assessment::types::SubmittedAnswer,
    ) -> Result<(), ValidationError> {
        // Validate question ID
        if answer.question_id == 0 {
            return Err(ValidationError::InvalidRange {
                field: "question_id",
                min: 1,
                max: u64::MAX,
                actual: 0,
            });
        }
        
        // Validate answer value
        match &answer.value {
            crate::contracts::assessment::types::SubmittedAnswerValue::SingleChoice(option_id) => {
                if *option_id > 1000 {
                    return Err(ValidationError::InvalidRange {
                        field: "selected_option_id",
                        min: 0,
                        max: 1000,
                        actual: *option_id as u64,
                    });
                }
            },
            crate::contracts::assessment::types::SubmittedAnswerValue::MultipleChoice(option_ids) => {
                CoreValidator::validate_array_size(option_ids, "selected_options", 1, 10)?;
                
                for option_id in option_ids.iter() {
                    if *option_id > 1000 {
                        return Err(ValidationError::InvalidRange {
                            field: "selected_option_id",
                            min: 0,
                            max: 1000,
                            actual: *option_id as u64,
                        });
                    }
                }
            },
            crate::contracts::assessment::types::SubmittedAnswerValue::Numeric(value) => {
                // Validate numeric range is reasonable
                if *value < -1000000 || *value > 1000000 {
                    return Err(ValidationError::InvalidRange {
                        field: "numeric_answer",
                        min: -1000000,
                        max: 1000000,
                        actual: *value as u64,
                    });
                }
            },
            crate::contracts::assessment::types::SubmittedAnswerValue::ShortText(text) => {
                let text_str = text.to_string();
                CoreValidator::validate_text_field(
                    &text_str,
                    "text_answer",
                    1,
                    500,
                )?;
            },
            crate::contracts::assessment::types::SubmittedAnswerValue::Essay(text) => {
                let text_str = text.to_string();
                CoreValidator::validate_text_field(
                    &text_str,
                    "essay_answer",
                    10,
                    5000,
                )?;
            },
            crate::contracts::assessment::types::SubmittedAnswerValue::Code(code) => {
                let code_str = code.to_string();
                CoreValidator::validate_text_field(
                    &code_str,
                    "code_answer",
                    1,
                    10000,
                )?;
            }
        }
        
        Ok(())
    }
    
    // ==================== COMMUNITY CONTRACT VALIDATION ====================
    
    /// Validates post creation parameters
    pub fn validate_create_post(
        env: &Env,
        author: &Address,
        category: &crate::contracts::community::types::ForumCategory,
        title: &String,
        content: &String,
        tags: &Vec<String>,
        course_id: &String,
    ) -> Result<(), ValidationError> {
        // Validate address
        CoreValidator::validate_address_with_env(env, author, "author")?;
        
        // Validate title
        let title_str = title.to_string();
        CoreValidator::validate_text_field(
            &title_str,
            "title",
            ValidationConfig::MIN_TITLE_LENGTH,
            ValidationConfig::MAX_TITLE_LENGTH,
        )?;
        
        // Validate content
        let content_str = content.to_string();
        CoreValidator::validate_text_field(
            &content_str,
            "content",
            ValidationConfig::MIN_DESCRIPTION_LENGTH,
            ValidationConfig::MAX_DESCRIPTION_LENGTH,
        )?;
        
        // Validate tags
        CoreValidator::validate_post_tags(tags, "tags")?;
        
        // Validate each tag
        for tag in tags.iter() {
            let tag_str = tag.to_string();
            CoreValidator::validate_text_field(
                &tag_str,
                "tag",
                1,
                20,
            )?;
        }
        
        // Validate course ID
        let course_str = course_id.to_string();
        CoreValidator::validate_course_id(&course_str)?;
        
        Ok(())
    }
    
    /// Validates reply creation parameters
    pub fn validate_create_reply(
        env: &Env,
        author: &Address,
        post_id: u64,
        content: &String,
    ) -> Result<(), ValidationError> {
        // Validate address
        CoreValidator::validate_address_with_env(env, author, "author")?;
        
        // Validate post ID
        if post_id == 0 {
            return Err(ValidationError::InvalidRange {
                field: "post_id",
                min: 1,
                max: u64::MAX,
                actual: 0,
            });
        }
        
        // Validate content
        let content_str = content.to_string();
        CoreValidator::validate_text_field(
            &content_str,
            "reply_content",
            ValidationConfig::MIN_DESCRIPTION_LENGTH,
            ValidationConfig::MAX_DESCRIPTION_LENGTH,
        )?;
        
        Ok(())
    }
    
    /// Validates reputation award parameters
    pub fn validate_award_reputation(
        env: &Env,
        from: &Address,
        to: &Address,
        amount: u32,
        reason: &String,
    ) -> Result<(), ValidationError> {
        // Validate addresses
        CoreValidator::validate_address_with_env(env, from, "from")?;
        CoreValidator::validate_address_with_env(env, to, "to")?;
        
        // Cannot award reputation to self
        if from == to {
            return Err(ValidationError::InvalidAddress {
                reason: "Cannot award reputation to yourself",
            });
        }
        
        // Validate amount
        CoreValidator::validate_reputation(amount)?;
        
        // Validate reason
        let reason_str = reason.to_string();
        CoreValidator::validate_text_field(
            &reason_str,
            "reputation_reason",
            5,
            200,
        )?;
        
        Ok(())
    }
    
    // ==================== CERTIFICATE CONTRACT VALIDATION ====================
    
    /// Validates certificate minting parameters
    pub fn validate_mint_certificate(
        env: &Env,
        recipient: &Address,
        issuer: &Address,
        template_id: &BytesN<32>,
        metadata: &crate::contracts::certificate::types::CertificateMetadata,
    ) -> Result<(), ValidationError> {
        // Validate addresses
        CoreValidator::validate_address_with_env(env, recipient, "recipient")?;
        CoreValidator::validate_address_with_env(env, issuer, "issuer")?;
        
        // Validate template ID
        CoreValidator::validate_certificate_id(template_id)?;
        
        // Validate metadata
        Self::validate_certificate_metadata(env, metadata)?;
        
        Ok(())
    }
    
    /// Validates certificate metadata
    pub fn validate_certificate_metadata(
        env: &Env,
        metadata: &crate::contracts::certificate::types::CertificateMetadata,
    ) -> Result<(), ValidationError> {
        // Validate title
        let title_str = metadata.title.to_string();
        CoreValidator::validate_text_field(
            &title_str,
            "certificate_title",
            ValidationConfig::MIN_TITLE_LENGTH,
            ValidationConfig::MAX_TITLE_LENGTH,
        )?;
        
        // Validate description
        let desc_str = metadata.description.to_string();
        CoreValidator::validate_text_field(
            &desc_str,
            "certificate_description",
            ValidationConfig::MIN_DESCRIPTION_LENGTH,
            ValidationConfig::MAX_DESCRIPTION_LENGTH,
        )?;
        
        // Validate expiry date
        CoreValidator::validate_expiry_date(env, metadata.expiry_date)?;
        
        // Validate skills
        CoreValidator::validate_array_size(
            &metadata.skills,
            "skills",
            0,
            20,
        )?;
        
        // Validate each skill
        for skill in metadata.skills.iter() {
            let skill_str = skill.to_string();
            CoreValidator::validate_text_field(
                &skill_str,
                "skill",
                2,
                50,
            )?;
        }
        
        Ok(())
    }
    
    // ==================== ANALYTICS CONTRACT VALIDATION ====================
    
    /// Validates session recording parameters
    pub fn validate_record_session(
        env: &Env,
        user: &Address,
        session_id: &BytesN<32>,
    ) -> Result<(), ValidationError> {
        // Validate address
        CoreValidator::validate_address_with_env(env, user, "user")?;
        
        // Validate session ID
        CoreValidator::validate_certificate_id(session_id)?;
        
        Ok(())
    }
    
    // ==================== BATCH OPERATION VALIDATION ====================
    
    /// Validates batch operation parameters
    pub fn validate_batch_operation<T>(
        env: &Env,
        batch_items: &Vec<T>,
        operation_name: &'static str,
        max_batch_size: u32,
    ) -> Result<(), ValidationError> {
        // Validate batch size
        CoreValidator::validate_batch_size(
            batch_items.len() as u32,
            operation_name,
            max_batch_size,
        )?;
        
        // Validate minimum batch size
        if batch_items.is_empty() {
            return Err(ValidationError::InvalidArraySize {
                field: operation_name,
                min: 1,
                max: max_batch_size,
                actual: 0,
            });
        }
        
        Ok(())
    }
    
    /// Validates schedule configuration
    pub fn validate_schedule_config(
        env: &Env,
        start_time: u64,
        end_time: u64,
        time_zone_offset: i32,
    ) -> Result<(), ValidationError> {
        // Validate time range
        if start_time >= end_time {
            return Err(ValidationError::InvalidRange {
                field: "schedule_time",
                min: 0,
                max: end_time,
                actual: start_time,
            });
        }
        
        // Validate start time is not too far in the past
        let current_time = env.ledger().timestamp();
        if start_time < current_time.saturating_sub(86400 * 30) { // 30 days ago
            return Err(ValidationError::InvalidDate {
                reason: "Start time cannot be more than 30 days in the past",
            });
        }
        
        // Validate end time is not too far in the future
        let max_future = current_time + ValidationConfig::MAX_FUTURE_EXPIRY;
        if end_time > max_future {
            return Err(ValidationError::InvalidDate {
                reason: "End time cannot be more than 100 years in the future",
            });
        }
        
        // Validate timezone offset (reasonable range: -12 to +14 hours)
        let offset_minutes = time_zone_offset;
        if offset_minutes < -720 || offset_minutes > 840 {
            return Err(ValidationError::InvalidRange {
                field: "timezone_offset",
                min: -720,
                max: 840,
                actual: offset_minutes as u64,
            });
        }
        
        Ok(())
    }
    
    /// Validates accommodation configuration
    pub fn validate_accommodation_config(
        extra_time_percent: u32,
        extra_attempts: u32,
    ) -> Result<(), ValidationError> {
        // Validate extra time percent (0-200%)
        if extra_time_percent > 200 {
            return Err(ValidationError::InvalidRange {
                field: "extra_time_percent",
                min: 0,
                max: 200,
                actual: extra_time_percent as u64,
            });
        }
        
        // Validate extra attempts (0-10)
        if extra_attempts > 10 {
            return Err(ValidationError::InvalidRange {
                field: "extra_attempts",
                min: 0,
                max: 10,
                actual: extra_attempts as u64,
            });
        }
        
        Ok(())
    }
}

/// Macro for easy validation integration in contracts
#[macro_export]
macro_rules! validate_input {
    ($env:expr, $validator:ident, $($param:expr),*) => {
        $crate::contracts::shared::input_validation::ContractValidator::$validator($env, $($param),*)?;
    };
}

/// Macro for validating multiple inputs
#[macro_export]
macro_rules! validate_inputs {
    ($env:expr, $($validator:ident($($param:expr),*)),*) => {
        $(
            $crate::contracts::shared::input_validation::ContractValidator::$validator($env, $($param),*)?;
        )*
    };
}
