# Error Handling Best Practices Guide

## Overview

This guide outlines the best practices for error handling in StrellerMinds smart contracts. Following these practices ensures consistent, maintainable, and user-friendly error handling across the entire platform.

## Table of Contents

1. [Fundamental Principles](#fundamental-principles)
2. [Error Design Patterns](#error-design-patterns)
3. [Implementation Guidelines](#implementation-guidelines)
4. [Testing Error Scenarios](#testing-error-scenarios)
5. [Performance Considerations](#performance-considerations)
6. [Security Considerations](#security-considerations)
7. [Code Examples](#code-examples)
8. [Common Pitfalls](#common-pitfalls)
9. [Review Checklist](#review-checklist)

## Fundamental Principles

### 1. Fail Fast and Explicitly

**Principle**: Detect and report errors as early as possible with clear, explicit messages.

```rust
// ✅ Good: Validate input immediately
pub fn create_certificate(env: &Env, params: CertificateParams) -> Result<CertificateId, StandardError> {
    // Validate input first
    if params.title.is_empty() {
        return Err(StandardError::MissingRequiredField);
    }
    if params.title.len() > MAX_TITLE_LENGTH {
        return Err(StandardError::InputTooLong);
    }
    
    // Continue with operation
    // ...
}

// ❌ Bad: Delay validation
pub fn create_certificate(env: &Env, params: CertificateParams) -> Result<CertificateId, StandardError> {
    // Do some work first
    let certificate_id = generate_id(&env);
    
    // Validate later - wasteful if invalid
    if params.title.is_empty() {
        return Err(StandardError::MissingRequiredField);
    }
    // ...
}
```

### 2. Provide Context and Recovery Information

**Principle**: Always include sufficient context for debugging and provide recovery guidance.

```rust
// ✅ Good: Include context and recovery info
pub fn revoke_certificate(env: &Env, certificate_id: &CertificateId, reason: &str) -> Result<(), StandardError> {
    let certificate = storage::get_certificate(&env, certificate_id)
        .ok_or(StandardError::CertificateNotFound)?;
    
    if certificate.is_revoked() {
        let context = ErrorHandler::create_error_context(
            &env,
            StandardError::InvalidStatus,
            "revoke_certificate",
            "CertificateContract",
            Some(&env.current_contract_address()),
            &format!("Certificate {} already revoked", certificate_id)
        );
        ErrorHandler::log_error(&env, &context);
        return Err(StandardError::InvalidStatus);
    }
    
    // Continue with revocation
    // ...
}

// ❌ Bad: Minimal context
pub fn revoke_certificate(env: &Env, certificate_id: &CertificateId, reason: &str) -> Result<(), StandardError> {
    let certificate = storage::get_certificate(&env, certificate_id);
    if certificate.is_none() {
        return Err(StandardError::CertificateNotFound);
    }
    // ...
}
```

### 3. Use Standardized Error Types

**Principle**: Leverage the standardized error system for consistency.

```rust
// ✅ Good: Use standardized errors
use crate::standardized_errors::StandardError;

pub fn validate_amount(amount: u64) -> Result<(), StandardError> {
    if amount == 0 {
        return Err(StandardError::InvalidAmount);
    }
    if amount > MAX_AMOUNT {
        return Err(StandardError::LimitExceeded);
    }
    Ok(())
}

// ❌ Bad: Custom error types
#[contracterror]
pub enum MyError {
    AmountZero = 1,
    AmountTooBig = 2,
}
```

### 4. Handle All Error Cases

**Principle**: Ensure every possible error condition is handled explicitly.

```rust
// ✅ Good: Handle all cases
pub fn process_payment(env: &Env, from: &Address, to: &Address, amount: u64) -> Result<(), StandardError> {
    // Validate inputs
    ErrorHandler::validate_address(&env, from, "process_payment")?;
    ErrorHandler::validate_address(&env, to, "process_payment")?;
    ErrorHandler::validate_range(&env, amount, 1, u64::MAX, "process_payment", "amount")?;
    
    // Check balance
    let balance = token::balance(&env, from)?;
    if balance < amount {
        return Err(StandardError::InsufficientBalance);
    }
    
    // Check authorization
    from.require_auth();
    
    // Process payment
    token::transfer(&env, from, to, amount)?;
    
    Ok(())
}

// ❌ Bad: Missing error cases
pub fn process_payment(env: &Env, from: &Address, to: &Address, amount: u64) -> Result<(), StandardError> {
    // Missing validation
    token::transfer(&env, from, to, amount)?; // Could fail with insufficient balance
    Ok(())
}
```

## Error Design Patterns

### 1. Validation Pattern

Validate all inputs at the beginning of functions:

```rust
pub fn create_assessment(env: &Env, params: AssessmentParams) -> Result<AssessmentId, StandardError> {
    // Validation phase
    validate_assessment_params(&env, &params)?;
    
    // Authorization phase
    check_instructor_permissions(&env, &params.instructor)?;
    
    // Business logic phase
    let assessment_id = create_assessment_internal(&env, &params)?;
    
    Ok(assessment_id)
}

fn validate_assessment_params(env: &Env, params: &AssessmentParams) -> Result<(), StandardError> {
    ErrorHandler::validate_string(&env, &params.title, "create_assessment", "title", 1, 100)?;
    ErrorHandler::validate_range(&env, params.duration, 60, 86400, "create_assessment", "duration")?;
    ErrorHandler::validate_range(&env, params.max_attempts, 1, 10, "create_assessment", "max_attempts")?;
    
    if params.questions.is_empty() {
        return Err(StandardError::MissingRequiredField);
    }
    
    Ok(())
}
```

### 2. Resource Access Pattern

Always check resource existence before use:

```rust
pub fn update_certificate(env: &Env, certificate_id: &CertificateId, updates: CertificateUpdates) -> Result<(), StandardError> {
    // Check resource exists
    let mut certificate = storage::get_certificate(&env, certificate_id)
        .ok_or(StandardError::CertificateNotFound)?;
    
    // Check resource state
    if certificate.is_revoked() {
        return Err(StandardError::CertificateRevoked);
    }
    
    // Check authorization
    check_update_permissions(&env, &certificate, &updates.updater)?;
    
    // Apply updates
    apply_updates(&mut certificate, updates)?;
    storage::save_certificate(&env, certificate_id, &certificate);
    
    Ok(())
}
```

### 3. Batch Operation Pattern

Handle batch operations with detailed error reporting:

```rust
pub fn batch_create_certificates(env: &Env, params: Vec<CertificateParams>) -> Result<Vec<CertificateId>, StandardError> {
    if params.is_empty() {
        return Err(StandardError::BatchEmpty);
    }
    
    if params.len() > MAX_BATCH_SIZE {
        return Err(StandardError::BatchTooLarge);
    }
    
    let mut results = Vec::new(&env);
    let mut failed_count = 0;
    
    for (index, param) in params.iter().enumerate() {
        match create_certificate_internal(&env, param) {
            Ok(cert_id) => results.push_back(cert_id),
            Err(error) => {
                failed_count += 1;
                log_batch_error(&env, index, error);
                
                // Continue processing other items
                continue;
            }
        }
    }
    
    if failed_count > 0 {
        // Return partial success with error context
        return Err(StandardError::PartialFailure);
    }
    
    Ok(results)
}
```

### 4. State Transition Pattern

Validate state transitions explicitly:

```rust
pub fn update_assessment_status(env: &Env, assessment_id: &AssessmentId, new_status: AssessmentStatus) -> Result<(), StandardError> {
    let mut assessment = storage::get_assessment(&env, assessment_id)
        .ok_or(StandardError::AssessmentNotFound)?;
    
    // Validate state transition
    if !is_valid_transition(assessment.status, new_status) {
        let context = ErrorHandler::create_error_context(
            &env,
            StandardError::InvalidTransition,
            "update_assessment_status",
            "AssessmentContract",
            Some(&env.current_contract_address()),
            &format!("Invalid transition from {:?} to {:?}", assessment.status, new_status)
        );
        ErrorHandler::log_error(&env, &context);
        return Err(StandardError::InvalidTransition);
    }
    
    // Apply transition
    assessment.status = new_status;
    storage::save_assessment(&env, assessment_id, &assessment);
    
    Ok(())
}

fn is_valid_transition(from: AssessmentStatus, to: AssessmentStatus) -> bool {
    match (from, to) {
        (AssessmentStatus::Draft, AssessmentStatus::Published) => true,
        (AssessmentStatus::Published, AssessmentStatus::Active) => true,
        (AssessmentStatus::Active, AssessmentStatus::Completed) => true,
        (AssessmentStatus::Active, AssessmentStatus::Cancelled) => true,
        _ => false,
    }
}
```

## Implementation Guidelines

### 1. Error Message Standards

**Use consistent error message format:**

```rust
// ✅ Good: Structured error messages
pub fn create_user_error_message(error: StandardError, context: &str) -> String {
    format!(
        "Error {}: {}. Context: {}. Action: {}",
        error as u32,
        error.description(),
        context,
        error.suggested_action()
    )
}

// ❌ Bad: Inconsistent messages
return Err(MyError::SomethingWentWrong);
```

### 2. Error Logging Standards

**Log errors with appropriate severity:**

```rust
// ✅ Good: Structured logging
pub fn log_contract_error(env: &Env, error: StandardError, operation: &str, details: &str) {
    let severity = match error.severity() {
        ErrorSeverity::Critical => "CRITICAL",
        ErrorSeverity::High => "HIGH",
        ErrorSeverity::Medium => "MEDIUM",
        ErrorSeverity::Low => "LOW",
    };
    
    let log_entry = format!(
        "[{}] {} in {}: {} - {}",
        severity,
        error.description(),
        operation,
        details,
        error.suggested_action()
    );
    
    env.log_str(&log_entry);
    
    // Emit error event for monitoring
    events::ErrorLogged {
        error_code: error as u32,
        operation: String::from_str(env, operation),
        severity: String::from_str(env, severity),
        timestamp: env.ledger().timestamp(),
    }.publish(&env);
}

// ❌ Bad: Unstructured logging
env.log_str("Error happened");
```

### 3. Error Recovery Patterns

**Provide recovery paths when possible:**

```rust
// ✅ Good: Provide recovery options
pub fn submit_assessment(env: &Env, submission: AssessmentSubmission) -> Result<SubmissionId, StandardError> {
    // Check if assessment exists and is active
    let assessment = storage::get_assessment(&env, &submission.assessment_id)
        .ok_or(StandardError::AssessmentNotFound)?;
    
    match assessment.status {
        AssessmentStatus::Active => {
            // Normal path
            submit_assessment_internal(&env, submission)
        }
        AssessmentStatus::Published => {
            // Assessment not yet active - provide helpful error
            let context = ErrorHandler::create_error_context(
                &env,
                StandardError::InvalidStatus,
                "submit_assessment",
                "AssessmentContract",
                Some(&submission.student),
                &format!("Assessment {} is published but not yet active. Starts at: {}", 
                        submission.assessment_id, assessment.start_time)
            );
            ErrorHandler::log_error(&env, &context);
            Err(StandardError::InvalidStatus)
        }
        AssessmentStatus::Completed => {
            // Assessment completed - suggest checking results
            let context = ErrorHandler::create_error_context(
                &env,
                StandardError::InvalidStatus,
                "submit_assessment",
                "AssessmentContract",
                Some(&submission.student),
                &format!("Assessment {} is completed. Check results instead.", 
                        submission.assessment_id)
            );
            ErrorHandler::log_error(&env, &context);
            Err(StandardError::InvalidStatus)
        }
        _ => {
            // Other statuses
            Err(StandardError::InvalidStatus)
        }
    }
}
```

### 4. Error Aggregation

**Aggregate related errors for batch operations:**

```rust
// ✅ Good: Aggregate errors
pub struct BatchError {
    pub failed_indices: Vec<u32>,
    pub errors: Vec<StandardError>,
    pub success_count: u32,
    pub total_count: u32,
}

pub fn batch_validate_certificates(env: &Env, certificate_ids: Vec<CertificateId>) -> Result<(), BatchError> {
    let mut batch_error = BatchError {
        failed_indices: Vec::new(&env),
        errors: Vec::new(&env),
        success_count: 0,
        total_count: certificate_ids.len() as u32,
    };
    
    for (index, cert_id) in certificate_ids.iter().enumerate() {
        match validate_certificate(&env, cert_id) {
            Ok(_) => batch_error.success_count += 1,
            Err(error) => {
                batch_error.failed_indices.push_back(index as u32);
                batch_error.errors.push_back(error);
            }
        }
    }
    
    if batch_error.failed_indices.is_empty() {
        Ok(())
    } else {
        Err(batch_error)
    }
}
```

## Testing Error Scenarios

### 1. Unit Testing Error Cases

**Test all error paths:**

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_create_certificate_invalid_title() {
        let env = Env::default();
        
        // Test empty title
        let params = CertificateParams {
            title: String::from_str(&env, ""),
            ..Default::default()
        };
        
        let result = create_certificate(&env, params);
        assert_eq!(result, Err(StandardError::MissingRequiredField));
    }
    
    #[test]
    fn test_create_certificate_title_too_long() {
        let env = Env::default();
        
        // Test title too long
        let long_title = "a".repeat(MAX_TITLE_LENGTH + 1);
        let params = CertificateParams {
            title: String::from_str(&env, &long_title),
            ..Default::default()
        };
        
        let result = create_certificate(&env, params);
        assert_eq!(result, Err(StandardError::InputTooLong));
    }
    
    #[test]
    fn test_create_certificate_unauthorized() {
        let env = Env::default();
        let unauthorized_user = Address::generate(&env);
        
        let params = CertificateParams {
            title: String::from_str(&env, "Valid Title"),
            creator: unauthorized_user,
            ..Default::default()
        };
        
        let result = create_certificate(&env, params);
        assert_eq!(result, Err(StandardError::Unauthorized));
    }
}
```

### 2. Integration Testing Error Flows

**Test error propagation across contract boundaries:**

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    
    #[test]
    fn test_cross_contract_error_propagation() {
        let env = Env::default();
        
        // Setup contracts
        let certificate_contract = CertificateContractClient::new(&env, &certificate_id);
        let community_contract = CommunityContractClient::new(&env, &community_id);
        
        // Test error propagation from certificate to community
        let invalid_cert_id = CertificateId::generate(&env);
        
        let result = community_contract.link_certificate(&invalid_cert_id);
        assert_eq!(result, Err(CommunityError::CertificateNotFound));
    }
}
```

### 3. Error Recovery Testing

**Test error recovery scenarios:**

```rust
#[test]
fn test_error_recovery_scenarios() {
    let env = Env::default();
    
    // Test rate limit recovery
    let user = Address::generate(&env);
    
    // Exceed rate limit
    for _ in 0..RATE_LIMIT + 1 {
        let result = create_certificate(&env, create_test_params(&user));
        // First RATE_LIMIT operations should succeed
        if _ < RATE_LIMIT {
            assert!(result.is_ok());
        } else {
            assert_eq!(result, Err(StandardError::RateLimitExceeded));
        }
    }
    
    // Wait for rate limit reset (simulate time passing)
    env.ledger().set_timestamp(env.ledger().timestamp() + RATE_LIMIT_WINDOW);
    
    // Should work again
    let result = create_certificate(&env, create_test_params(&user));
    assert!(result.is_ok());
}
```

## Performance Considerations

### 1. Error Handling Overhead

**Minimize error handling overhead:**

```rust
// ✅ Good: Efficient error checking
pub fn validate_inputs_fast(params: &CertificateParams) -> Result<(), StandardError> {
    // Use bit flags for multiple validations
    let mut validation_flags = 0u8;
    
    if params.title.is_empty() { validation_flags |= 0x01; }
    if params.title.len() > MAX_TITLE_LENGTH { validation_flags |= 0x02; }
    if params.description.len() > MAX_DESC_LENGTH { validation_flags |= 0x04; }
    
    if validation_flags != 0 {
        return Err(map_validation_flags_to_error(validation_flags));
    }
    
    Ok(())
}

// ❌ Bad: Multiple separate checks
pub fn validate_inputs_slow(params: &CertificateParams) -> Result<(), StandardError> {
    if params.title.is_empty() {
        return Err(StandardError::MissingRequiredField);
    }
    if params.title.len() > MAX_TITLE_LENGTH {
        return Err(StandardError::InputTooLong);
    }
    if params.description.len() > MAX_DESC_LENGTH {
        return Err(StandardError::InputTooLong);
    }
    Ok(())
}
```

### 2. Gas Optimization

**Optimize error handling for gas efficiency:**

```rust
// ✅ Good: Gas-efficient validation
pub fn validate_amount_gas_optimized(amount: u64) -> Result<(), StandardError> {
    // Single comparison for multiple checks
    match amount {
        0 => Err(StandardError::InvalidAmount),
        a if a > MAX_AMOUNT => Err(StandardError::LimitExceeded),
        _ => Ok(()),
    }
}

// ❌ Bad: Multiple separate comparisons
pub fn validate_amount_gas_inefficient(amount: u64) -> Result<(), StandardError> {
    if amount == 0 {
        return Err(StandardError::InvalidAmount);
    }
    if amount > MAX_AMOUNT {
        return Err(StandardError::LimitExceeded);
    }
    Ok(())
}
```

### 3. Memory Usage

**Minimize memory allocations in error handling:**

```rust
// ✅ Good: Reuse error context
pub fn process_with_reused_context(env: &Env, operations: Vec<Operation>) -> Result<(), StandardError> {
    let mut context = ErrorContext::new(
        StandardError::InternalError,
        "process_operations",
        "MyContract",
        ""
    );
    
    for op in operations {
        context.operation = op.name.clone();
        if let Err(e) = process_single_operation(&env, &op) {
            context.error_code = e as u32;
            context.error_message = format!("{:?}", e);
            ErrorHandler::log_error(&env, &context);
            return Err(e);
        }
    }
    
    Ok(())
}
```

## Security Considerations

### 1. Information Disclosure

**Avoid exposing sensitive information in errors:**

```rust
// ✅ Good: Sanitized error messages
pub fn check_admin_permissions(env: &Env, caller: &Address) -> Result<(), StandardError> {
    let admin = storage::get_admin(&env);
    
    if caller != admin {
        // Don't reveal admin address
        let context = ErrorHandler::create_error_context(
            &env,
            StandardError::Unauthorized,
            "check_admin_permissions",
            "MyContract",
            Some(caller),
            "Insufficient permissions for operation"
        );
        ErrorHandler::log_error(&env, &context);
        return Err(StandardError::Unauthorized);
    }
    
    Ok(())
}

// ❌ Bad: Exposing sensitive information
pub fn check_admin_permissions_unsafe(env: &Env, caller: &Address) -> Result<(), StandardError> {
    let admin = storage::get_admin(&env);
    
    if caller != admin {
        // Exposes admin address - security risk
        return Err(StandardError::Unauthorized);
    }
    
    Ok(())
}
```

### 2. Error Rate Limiting

**Prevent error-based attacks:**

```rust
// ✅ Good: Rate limit error responses
pub fn validate_with_rate_limit(env: &Env, user: &Address, input: &str) -> Result<(), StandardError> {
    // Check error rate for this user
    let error_count = get_user_error_count(&env, user);
    if error_count > MAX_ERROR_RATE {
        return Err(StandardError::RateLimitExceeded);
    }
    
    // Perform validation
    if !is_valid_input(input) {
        increment_user_error_count(&env, user);
        return Err(StandardError::InvalidInput);
    }
    
    // Reset error count on success
    reset_user_error_count(&env, user);
    Ok(())
}
```

### 3. Audit Trail

**Maintain comprehensive audit trail for security events:**

```rust
// ✅ Good: Comprehensive security logging
pub fn handle_security_error(env: &Env, error: StandardError, user: &Address, operation: &str) {
    // Log with high severity
    let context = ErrorHandler::create_error_context(
        &env,
        error,
        operation,
        "SecurityContract",
        Some(user),
        "Security violation detected"
    );
    
    ErrorHandler::log_error(&env, &context);
    
    // Emit security event
    events::SecurityViolation {
        user: user.clone(),
        operation: String::from_str(env, operation),
        error_code: error as u32,
        timestamp: env.ledger().timestamp(),
    }.publish(env);
    
    // Update security metrics
    update_security_metrics(env, error, user);
}
```

## Code Examples

### 1. Complete Function with Error Handling

```rust
pub fn create_certificate_with_full_error_handling(
    env: &Env,
    params: CertificateParams,
) -> Result<CertificateId, StandardError> {
    // Input validation phase
    validate_certificate_params(&env, &params)?;
    
    // Authorization phase
    check_creation_permissions(&env, &params.creator)?;
    
    // Business logic phase
    let certificate_id = generate_certificate_id(&env);
    
    // Check for duplicates
    if storage::certificate_exists(&env, &certificate_id) {
        return Err(StandardError::AlreadyExists);
    }
    
    // Create certificate
    let certificate = Certificate::new(params);
    storage::save_certificate(&env, &certificate_id, &certificate);
    
    // Emit success event
    events::CertificateCreated {
        certificate_id: certificate_id.clone(),
        creator: certificate.creator,
        timestamp: env.ledger().timestamp(),
    }.publish(env);
    
    Ok(certificate_id)
}

fn validate_certificate_params(env: &Env, params: &CertificateParams) -> Result<(), StandardError> {
    // Validate title
    ErrorHandler::validate_string(&env, &params.title, "create_certificate", "title", 1, 100)?;
    
    // Validate description
    ErrorHandler::validate_string(&env, &params.description, "create_certificate", "description", 0, 1000)?;
    
    // Validate template
    if !storage::template_exists(&env, &params.template_id) {
        return Err(StandardError::TemplateNotFound);
    }
    
    // Validate expiration
    if params.expires_at <= env.ledger().timestamp() {
        return Err(StandardError::Expired);
    }
    
    Ok(())
}

fn check_creation_permissions(env: &Env, creator: &Address) -> Result<(), StandardError> {
    // Check if creator is authorized
    creator.require_auth();
    
    // Check if creator has certificate creation permissions
    if !has_certificate_creation_permission(&env, creator) {
        let context = ErrorHandler::create_error_context(
            &env,
            StandardError::Unauthorized,
            "create_certificate",
            "CertificateContract",
            Some(creator),
            "User lacks certificate creation permission"
        );
        ErrorHandler::log_error(&env, &context);
        return Err(StandardError::Unauthorized);
    }
    
    Ok(())
}
```

### 2. Error Handling in Cross-Contract Calls

```rust
pub fn link_certificate_to_community(
    env: &Env,
    certificate_id: &CertificateId,
    community_id: &CommunityId,
) -> Result<(), StandardError> {
    // Verify certificate exists
    let certificate = storage::get_certificate(&env, certificate_id)
        .ok_or(StandardError::CertificateNotFound)?;
    
    // Call community contract
    let community_client = CommunityContractClient::new(&env, &community_id);
    
    match community_client.add_certificate(certificate_id) {
        Ok(_) => {
            // Success - emit event
            events::CertificateLinked {
                certificate_id: certificate_id.clone(),
                community_id: community_id.clone(),
                timestamp: env.ledger().timestamp(),
            }.publish(env);
            Ok(())
        }
        Err(community_error) => {
            // Map community error to standard error
            let standard_error = map_community_error_to_standard(community_error);
            
            let context = ErrorHandler::create_error_context(
                &env,
                standard_error,
                "link_certificate_to_community",
                "CertificateContract",
                Some(&env.current_contract_address()),
                &format!("Failed to link certificate {} to community {}", certificate_id, community_id)
            );
            ErrorHandler::log_error(&env, &context);
            
            Err(standard_error)
        }
    }
}

fn map_community_error_to_standard(error: CommunityError) -> StandardError {
    match error {
        CommunityError::Unauthorized => StandardError::Unauthorized,
        CommunityError::CertificateNotFound => StandardError::CertificateNotFound,
        CommunityError::AlreadyExists => StandardError::AlreadyExists,
        CommunityError::InvalidStatus => StandardError::InvalidStatus,
        _ => StandardError::InternalError,
    }
}
```

## Common Pitfalls

### 1. Silent Failures

**❌ Pitfall**: Ignoring errors or using `unwrap()` without proper handling

```rust
// Bad: Silent failure
let certificate = storage::get_certificate(&env, &cert_id).unwrap(); // Can panic

// Bad: Ignoring error
let _ = storage::get_certificate(&env, &cert_id); // Error ignored

// ✅ Good: Proper error handling
let certificate = storage::get_certificate(&env, &cert_id)
    .ok_or(StandardError::CertificateNotFound)?;
```

### 2. Generic Error Messages

**❌ Pitfall**: Using generic error messages that don't help users

```rust
// Bad: Generic error
return Err(StandardError::InvalidInput);

// ✅ Good: Specific error with context
let context = ErrorHandler::create_error_context(
    &env,
    StandardError::InvalidInput,
    "create_certificate",
    "CertificateContract",
    Some(&user),
    "Certificate title contains invalid characters: only letters, numbers, and spaces allowed"
);
ErrorHandler::log_error(&env, &context);
return Err(StandardError::InvalidInput);
```

### 3. Inconsistent Error Handling

**❌ Pitfall**: Inconsistent error handling across similar functions

```rust
// Bad: Inconsistent handling
pub fn func1() -> Result<(), StandardError> {
    if condition {
        return Err(StandardError::Unauthorized);
    }
    Ok(())
}

pub fn func2() -> Result<(), StandardError> {
    if condition {
        panic!("Not authorized"); // Different handling
    }
    Ok(())
}

// ✅ Good: Consistent handling
pub fn func1() -> Result<(), StandardError> {
    check_authorization(&env, &user)?;
    Ok(())
}

pub fn func2() -> Result<(), StandardError> {
    check_authorization(&env, &user)?;
    Ok(())
}
```

### 4. Missing Error Context

**❌ Pitfall**: Returning errors without sufficient context

```rust
// Bad: No context
pub fn update_certificate() -> Result<(), StandardError> {
    if !has_permission() {
        return Err(StandardError::Unauthorized);
    }
    // ...
}

// ✅ Good: With context
pub fn update_certificate() -> Result<(), StandardError> {
    if !has_permission() {
        let context = ErrorHandler::create_error_context(
            &env,
            StandardError::Unauthorized,
            "update_certificate",
            "CertificateContract",
            Some(&user),
            "User lacks certificate update permission"
        );
        ErrorHandler::log_error(&env, &context);
        return Err(StandardError::Unauthorized);
    }
    // ...
}
```

## Review Checklist

### Code Review Checklist

- [ ] All functions return appropriate error types
- [ ] Input validation is performed at function entry
- [ ] Error messages are descriptive and actionable
- [ ] Error context includes operation details
- [ ] Standardized error codes are used
- [ ] All error paths are tested
- [ ] Security-sensitive information is not exposed
- [ ] Error handling is consistent across similar functions
- [ ] Performance impact of error handling is considered
- [ ] Error logging is implemented for debugging

### Testing Checklist

- [ ] All error codes are tested
- [ ] Error propagation across contract boundaries is tested
- [ ] Error recovery scenarios are tested
- [ ] Rate limiting in error handling is tested
- [ ] Security error handling is tested
- [ ] Performance impact of error handling is tested
- [ ] Error logging functionality is tested
- [ ] User error messages are tested for clarity

### Documentation Checklist

- [ ] Error codes are documented
- [ ] Error handling patterns are documented
- [ ] Troubleshooting guides are provided
- [ ] Best practices are documented
- [ ] Examples of error handling are provided
- [ ] Migration guides are available

---

*This best practices guide is maintained by the StrellerMinds development team. Last updated: March 2026*
