# Error Troubleshooting Guide

## Overview

This comprehensive troubleshooting guide helps developers and users diagnose, understand, and resolve errors encountered in the StrellerMinds smart contract platform.

## Table of Contents

1. [Quick Reference](#quick-reference)
2. [Error Diagnosis Workflow](#error-diagnosis-workflow)
3. [Common Error Scenarios](#common-error-scenarios)
4. [Contract-Specific Troubleshooting](#contract-specific-troubleshooting)
5. [Debugging Tools and Techniques](#debugging-tools-and-techniques)
6. [Error Recovery Procedures](#error-recovery-procedures)
7. [Escalation Procedures](#escalation-procedures)

## Quick Reference

### Error Code Ranges

| Range | Category | Common Issues | Quick Fix |
|-------|----------|---------------|-----------|
| 1000-1099 | Initialization | Contract setup issues | Check initialization status |
| 1100-1199 | Authorization | Permission problems | Verify user roles |
| 1200-1299 | Input Validation | Data format issues | Validate input format |
| 1300-1399 | Not Found | Missing resources | Check resource IDs |
| 1400-1499 | Business Logic | Operational errors | Check business rules |
| 1500-1599 | Configuration | Setup problems | Verify configuration |
| 1600-1699 | Storage | Data issues | Check storage state |
| 1700-1799 | Network | External services | Check connectivity |
| 1800-1899 | Security | Security issues | Contact security team |
| 1900-1999 | Batch | Batch operations | Reduce batch size |
| 2000-2099 | Temporal | Time-related | Check timestamps |
| 2100-2199 | System | System issues | Contact support |
| 2200-2299 | Compliance | Regulatory issues | Check compliance |
| 2300-2399 | Financial | Payment issues | Check balances |
| 2400-2499 | Miscellaneous | General issues | Check logs |

### Severity Levels

| Severity | Response Time | Action |
|----------|----------------|--------|
| Critical | Immediate | Contact support immediately |
| High | Within 1 hour | Address promptly or escalate |
| Medium | Within 4 hours | Fix when convenient |
| Low | Within 24 hours | Fix when convenient |

## Error Diagnosis Workflow

### Step 1: Identify the Error

1. **Get the Error Code**: Note the numeric error code (e.g., 1100, 1200, 1300)
2. **Read the Error Message**: Understand the human-readable description
3. **Check the Context**: Note the operation, contract, and additional info
4. **Determine Severity**: Assess if immediate action is required

### Step 2: Analyze the Context

```rust
// Example error context
{
    "error_code": 1100,
    "error_message": "Unauthorized",
    "operation": "create_certificate",
    "contract_name": "CertificateContract",
    "additional_info": "User not authorized to create certificates",
    "timestamp": 1234567890,
    "user_address": "GABC123..."
}
```

**Analysis Questions:**
- Who is the user?
- What operation were they trying to perform?
- Which contract generated the error?
- When did the error occur?
- What additional context is provided?

### Step 3: Check Common Causes

Based on the error category, check the most common causes:

#### Authorization Errors (1100-1199)
- User not authenticated
- Insufficient role permissions
- Admin privileges required
- Expired authentication

#### Input Validation Errors (1200-1299)
- Invalid data format
- Missing required fields
- Input length constraints
- Invalid addresses or amounts

#### Resource Not Found (1300-1399)
- Incorrect resource ID
- Resource not created yet
- Resource deleted
- Resource access permissions

#### Business Logic Errors (1400-1499)
- Invalid state transitions
- Duplicate resources
- Rate limits exceeded
- Quota limits reached

### Step 4: Apply the Solution

Use the suggested action from the error description and follow the specific troubleshooting steps for that error type.

### Step 5: Verify the Fix

1. Retry the operation
2. Check for success
3. Monitor for related errors
4. Document the resolution

## Common Error Scenarios

### Scenario 1: Unauthorized Access (Error 1100)

**Symptoms:**
```
Error 1100: Unauthorized. Operation: create_certificate. 
User: GABC123... Contract: CertificateContract.
```

**Common Causes:**
- User lacks certificate creation permission
- User account not properly initialized
- Role assignment missing
- Admin privileges required

**Troubleshooting Steps:**
1. **Check User Permissions:**
   ```rust
   // Verify user has certificate creation permission
   let has_permission = access_control::has_permission(&env, &user, "create_certificate");
   if !has_permission {
       // Grant permission or use authorized user
   }
   ```

2. **Verify User Initialization:**
   ```rust
   // Check if user is properly initialized
   if !user_storage::is_initialized(&env, &user) {
       return Err(StandardError::NotInitialized);
   }
   ```

3. **Check Role Assignment:**
   ```rust
   // Verify user has appropriate role
   let user_role = role_manager::get_user_role(&env, &user);
   if !role_can_create_certificates(user_role) {
       return Err(StandardError::Unauthorized);
   }
   ```

**Solutions:**
- Grant appropriate permissions to the user
- Use an admin account for admin operations
- Initialize the user account properly
- Assign correct role to the user

### Scenario 2: Invalid Input (Error 1200)

**Symptoms:**
```
Error 1200: Invalid input. Operation: create_certificate.
Contract: CertificateContract. Info: Title contains invalid characters.
```

**Common Causes:**
- Invalid characters in string fields
- Incorrect data format
- Missing required fields
- Input length constraints violated

**Troubleshooting Steps:**
1. **Validate Input Format:**
   ```rust
   // Check string format
   fn validate_certificate_title(title: &str) -> Result<(), StandardError> {
       if title.is_empty() {
           return Err(StandardError::MissingRequiredField);
       }
       
       if title.len() > MAX_TITLE_LENGTH {
           return Err(StandardError::InputTooLong);
       }
       
       if !title.chars().all(|c| c.is_alphanumeric() || c.is_whitespace() || "-_".contains(c)) {
           return Err(StandardError::InvalidInput);
       }
       
       Ok(())
   }
   ```

2. **Check Required Fields:**
   ```rust
   // Verify all required fields are present
   if params.title.is_empty() {
       return Err(StandardError::MissingRequiredField);
   }
   if params.template_id.is_none() {
       return Err(StandardError::MissingRequiredField);
   }
   ```

3. **Validate Data Types:**
   ```rust
   // Check numeric ranges
   if params.expiration_date <= env.ledger().timestamp() {
       return Err(StandardError::InvalidTimestamp);
   }
   ```

**Solutions:**
- Correct the input format
- Provide all required fields
- Ensure input meets length constraints
- Use valid characters only

### Scenario 3: Resource Not Found (Error 1300)

**Symptoms:**
```
Error 1300: Not found. Operation: get_certificate.
Contract: CertificateContract. Info: Certificate ID not found.
```

**Common Causes:**
- Incorrect certificate ID
- Certificate not created yet
- Certificate deleted
- Access permissions issue

**Troubleshooting Steps:**
1. **Verify Resource ID:**
   ```rust
   // Check if certificate exists
   let certificate = storage::get_certificate(&env, &certificate_id);
   if certificate.is_none() {
       return Err(StandardError::CertificateNotFound);
   }
   ```

2. **Check Resource Status:**
   ```rust
   // Verify certificate is not deleted
   let certificate = storage::get_certificate(&env, &certificate_id)
       .ok_or(StandardError::CertificateNotFound)?;
   
   if certificate.is_deleted() {
       return Err(StandardError::NotFound);
   }
   ```

3. **Verify Access Permissions:**
   ```rust
   // Check if user can access this certificate
   if !certificate.is_accessible_by(&user) {
       return Err(StandardError::Unauthorized);
   }
   ```

**Solutions:**
- Use correct certificate ID
- Create the certificate first
- Check if certificate was deleted
- Verify access permissions

### Scenario 4: Rate Limit Exceeded (Error 1405)

**Symptoms:**
```
Error 1405: Rate limit exceeded. Operation: create_certificate.
Contract: CertificateContract. Info: Too many requests in time window.
```

**Common Causes:**
- Too many requests in short time
- Bot activity detected
- API limits exceeded
- System protection triggered

**Troubleshooting Steps:**
1. **Check Rate Limit Status:**
   ```rust
   // Get current rate limit status
   let rate_limit_info = rate_limiter::get_user_status(&env, &user);
   if rate_limit_info.requests_remaining == 0 {
       return Err(StandardError::RateLimitExceeded);
   }
   ```

2. **Calculate Reset Time:**
   ```rust
   // Calculate when rate limit resets
   let reset_time = rate_limit_info.window_start + RATE_LIMIT_WINDOW;
   let current_time = env.ledger().timestamp();
   
   if current_time < reset_time {
       let wait_time = reset_time - current_time;
       // Inform user to wait
   }
   ```

3. **Implement Exponential Backoff:**
   ```rust
   // Implement backoff strategy
   let backoff_time = calculate_backoff(attempt_count);
   // Wait before retrying
   ```

**Solutions:**
- Wait for rate limit reset
- Implement exponential backoff
- Reduce request frequency
- Use batch operations when possible

### Scenario 5: Internal Error (Error 2100)

**Symptoms:**
```
Error 2100: Internal error. Operation: complex_calculation.
Contract: AnalyticsContract. Info: Unexpected error in calculation.
```

**Common Causes:**
- Unexpected system state
- Data corruption
- Logic errors
- External service failures

**Troubleshooting Steps:**
1. **Check System State:**
   ```rust
   // Verify system is in expected state
   if !is_system_healthy(&env) {
       return Err(StandardError::InternalError);
   }
   ```

2. **Validate Data Integrity:**
   ```rust
   // Check for data corruption
   if !validate_data_integrity(&env) {
       return Err(StandardError::DataCorruption);
   }
   ```

3. **Check External Dependencies:**
   ```rust
   // Verify external services are available
   if !external_service::is_available() {
       return Err(StandardError::ExternalServiceUnavailable);
   }
   ```

**Solutions:**
- Contact support immediately
- Check system logs
- Verify data integrity
- Restart if necessary

## Contract-Specific Troubleshooting

### Certificate Contract

#### Common Certificate Errors

| Error | Code | Common Cause | Solution |
|-------|------|--------------|----------|
| CertificateNotFound | 1302 | Invalid certificate ID | Verify certificate exists |
| CertificateRevoked | 3100 | Certificate already revoked | Check certificate status |
| MultiSigRequestNotFound | 3000 | Invalid multi-sig request | Verify request ID |
| TemplateNotFound | 1305 | Invalid template ID | Check template exists |

#### Troubleshooting Certificate Creation

```rust
// Debug certificate creation issues
pub fn debug_certificate_creation(env: &Env, params: CertificateParams) -> Result<(), String> {
    // Step 1: Check initialization
    if !certificate::is_initialized(&env) {
        return Err("Contract not initialized".to_string());
    }
    
    // Step 2: Check permissions
    if !certificate::has_creation_permission(&env, &params.creator) {
        return Err("Insufficient permissions".to_string());
    }
    
    // Step 3: Validate template
    if !certificate::template_exists(&env, &params.template_id) {
        return Err("Template not found".to_string());
    }
    
    // Step 4: Validate parameters
    if let Err(e) = certificate::validate_params(&env, &params) {
        return Err(format!("Validation failed: {:?}", e));
    }
    
    Ok(())
}
```

### Community Contract

#### Common Community Errors

| Error | Code | Common Cause | Solution |
|-------|------|--------------|----------|
| PostNotFound | 4000 | Invalid post ID | Verify post exists |
| MentorNotAvailable | 4100 | Mentor busy/inactive | Check mentor status |
| EventFull | 4301 | Event capacity reached | Wait for opening |
| InsufficientReputation | 4202 | Low reputation score | Increase reputation |

#### Troubleshooting Community Operations

```rust
// Debug community operations
pub fn debug_community_operation(env: &Env, operation: &str, user: &Address) -> Result<(), String> {
    // Step 1: Check user status
    let user_profile = community::get_user_profile(&env, user);
    if user_profile.is_none() {
        return Err("User profile not found".to_string());
    }
    
    // Step 2: Check reputation
    let reputation = user_profile.unwrap().reputation;
    if reputation < MIN_REPUTATION_FOR_POSTING {
        return Err("Insufficient reputation".to_string());
    }
    
    // Step 3: Check rate limits
    if community::is_rate_limited(&env, user, operation) {
        return Err("Rate limited".to_string());
    }
    
    Ok(())
}
```

### Assessment Contract

#### Common Assessment Errors

| Error | Code | Common Cause | Solution |
|-------|------|--------------|----------|
| AssessmentNotFound | 1303 | Invalid assessment ID | Verify assessment exists |
| QuestionNotFound | 5100 | Invalid question ID | Check question exists |
| MaxAttemptsReached | 5103 | Too many attempts | Wait for reset |
| AssessmentClosed | 5104 | Assessment not active | Check assessment status |

#### Troubleshooting Assessment Operations

```rust
// Debug assessment operations
pub fn debug_assessment_operation(env: &Env, assessment_id: &AssessmentId, user: &Address) -> Result<(), String> {
    // Step 1: Check assessment exists
    let assessment = assessment::get_assessment(&env, assessment_id);
    if assessment.is_none() {
        return Err("Assessment not found".to_string());
    }
    
    // Step 2: Check assessment status
    let assessment = assessment.unwrap();
    if assessment.status != AssessmentStatus::Active {
        return Err(format!("Assessment not active: {:?}", assessment.status));
    }
    
    // Step 3: Check user attempts
    let attempts = assessment::get_user_attempts(&env, assessment_id, user);
    if attempts >= assessment.max_attempts {
        return Err("Max attempts reached".to_string());
    }
    
    Ok(())
}
```

## Debugging Tools and Techniques

### 1. Error Logging

Enable comprehensive error logging:

```rust
// Enable detailed error logging
pub fn enable_debug_logging(env: &Env) {
    env.log_str("=== Debug Mode Enabled ===");
    
    // Log system state
    let timestamp = env.ledger().timestamp();
    env.log_str(&format!("Current timestamp: {}", timestamp));
    
    // Log contract state
    let contract_address = env.current_contract_address();
    env.log_str(&format!("Contract address: {}", contract_address));
    
    // Log storage usage
    let storage_usage = env.storage().usage();
    env.log_str(&format!("Storage usage: {:?}", storage_usage));
}
```

### 2. State Inspection

Inspect contract state for debugging:

```rust
// Inspect contract state
pub fn inspect_contract_state(env: &Env) {
    env.log_str("=== Contract State Inspection ===");
    
    // Check initialization
    let is_initialized = storage::is_initialized(&env);
    env.log_str(&format!("Initialized: {}", is_initialized));
    
    // Check admin
    let admin = storage::get_admin(&env);
    env.log_str(&format!("Admin: {}", admin));
    
    // Check configuration
    let config = storage::get_config(&env);
    env.log_str(&format!("Config: {:?}", config));
    
    // Check statistics
    let stats = storage::get_statistics(&env);
    env.log_str(&format!("Statistics: {:?}", stats));
}
```

### 3. Error Reproduction

Create test cases to reproduce errors:

```rust
// Reproduce specific error scenarios
pub fn reproduce_error_scenario(env: &Env, scenario: &str) {
    match scenario {
        "unauthorized_access" => {
            let unauthorized_user = Address::generate(&env);
            let result = certificate::create_certificate(&env, create_test_params(&unauthorized_user));
            assert_eq!(result, Err(StandardError::Unauthorized));
        }
        "invalid_input" => {
            let invalid_params = CertificateParams {
                title: String::from_str(&env, ""), // Empty title
                ..Default::default()
            };
            let result = certificate::create_certificate(&env, invalid_params);
            assert_eq!(result, Err(StandardError::MissingRequiredField));
        }
        _ => env.log_str("Unknown scenario"),
    }
}
```

### 4. Performance Monitoring

Monitor performance to identify bottlenecks:

```rust
// Monitor operation performance
pub fn monitor_performance(env: &Env, operation: &str) {
    let start_time = env.ledger().timestamp();
    
    // Perform operation
    match operation {
        "create_certificate" => {
            let params = create_test_params(&create_test_user(&env));
            let _result = certificate::create_certificate(&env, params);
        }
        _ => {}
    }
    
    let end_time = env.ledger().timestamp();
    let duration = end_time - start_time;
    
    env.log_str(&format!("Operation '{}' took {} units", operation, duration));
    
    // Alert if operation is too slow
    if duration > PERFORMANCE_THRESHOLD {
        env.log_str(&format!("WARNING: Operation '{}' is slow", operation));
    }
}
```

## Error Recovery Procedures

### 1. Automatic Recovery

Implement automatic recovery where possible:

```rust
// Automatic recovery for transient errors
pub fn auto_recover_from_error(env: &Env, error: StandardError, operation: &str) -> Result<(), StandardError> {
    match error {
        StandardError::RateLimitExceeded => {
            // Wait for rate limit reset
            let reset_time = calculate_rate_limit_reset(&env);
            if env.ledger().timestamp() >= reset_time {
                // Retry the operation
                return retry_operation(env, operation);
            }
        }
        StandardError::SystemOverloaded => {
            // Implement exponential backoff
            let backoff_time = calculate_backoff(get_retry_count());
            if backoff_time == 0 {
                return retry_operation(env, operation);
            }
        }
        _ => {} // No automatic recovery available
    }
    
    Err(error)
}
```

### 2. Manual Recovery

Provide manual recovery options:

```rust
// Manual recovery procedures
pub fn manual_recovery_guide(env: &Env, error: StandardError) -> String {
    match error {
        StandardError::Unauthorized => {
            "1. Check user permissions\n2. Verify user role\n3. Contact admin for access".to_string()
        }
        StandardError::RateLimitExceeded => {
            "1. Wait for rate limit reset\n2. Reduce request frequency\n3. Use batch operations".to_string()
        }
        StandardError::InsufficientBalance => {
            "1. Add funds to account\n2. Check transaction fees\n3. Verify balance calculation".to_string()
        }
        _ => {
            "1. Check error documentation\n2. Contact support\n3. Review system logs".to_string()
        }
    }
}
```

### 3. State Recovery

Recover from corrupted state:

```rust
// State recovery procedures
pub fn recover_contract_state(env: &Env) -> Result<(), StandardError> {
    // Backup current state
    let backup = create_state_backup(&env);
    
    // Validate state integrity
    if !validate_state_integrity(&env) {
        // Restore from backup
        restore_state_from_backup(&env, backup)?;
        return Err(StandardError::DataCorruption);
    }
    
    // Repair common issues
    repair_common_state_issues(&env)?;
    
    // Verify recovery
    if validate_state_integrity(&env) {
        Ok(())
    } else {
        Err(StandardError::InternalError)
    }
}
```

## Escalation Procedures

### When to Escalate

Escalate immediately for:
- Critical errors (severity: Critical)
- Data corruption issues
- Security violations
- System-wide failures
- Unresolved errors after standard troubleshooting

### Escalation Levels

| Level | Contact | Response Time | When to Use |
|-------|---------|---------------|-------------|
| 1 | Technical Support | 1 hour | Standard issues |
| 2 | Senior Engineer | 30 minutes | Complex issues |
| 3 | Engineering Lead | 15 minutes | Critical issues |
| 4 | Security Team | Immediate | Security issues |

### Escalation Information

When escalating, provide:

1. **Error Details**
   - Error code and message
   - Operation and contract
   - Timestamp and user address
   - Full error context

2. **System Information**
   - Contract version
   - System state
   - Recent changes
   - Performance metrics

3. **Troubleshooting Steps Taken**
   - Steps already attempted
   - Results of each step
   - Additional observations

4. **Impact Assessment**
   - Number of users affected
   - Business impact
   - Urgency level

### Escalation Template

```
URGENCY: [HIGH/MEDIUM/LOW]
ERROR CODE: [Numeric code]
ERROR MESSAGE: [Error description]
TIMESTAMP: [When error occurred]
USER: [User address if applicable]
OPERATION: [Operation being performed]
CONTRACT: [Contract name]

STEPS TAKEN:
1. [First troubleshooting step]
2. [Second troubleshooting step]
3. [Additional steps]

SYSTEM STATE:
- Contract version: [version]
- Last deployment: [date]
- Recent changes: [description]

IMPACT:
- Users affected: [number]
- Business impact: [description]
- Urgency: [assessment]

ADDITIONAL INFO:
[Any other relevant information]
```

## Monitoring and Alerting

### Error Rate Monitoring

Monitor error rates to detect issues:

```rust
// Monitor error rates
pub fn monitor_error_rates(env: &Env) {
    let metrics = get_error_metrics(&env);
    let current_time = env.ledger().timestamp();
    let time_window = 3600; // 1 hour
    
    let error_rate = metrics.get_error_rate(time_window, current_time);
    
    // Alert if error rate is too high
    if error_rate > ERROR_RATE_THRESHOLD {
        emit_high_error_rate_alert(&env, error_rate);
    }
    
    // Alert for critical errors
    if metrics.errors_by_severity.get(&ErrorSeverity::Critical).unwrap_or(&0) > 0 {
        emit_critical_error_alert(&env);
    }
}
```

### Health Checks

Implement regular health checks:

```rust
// Health check implementation
pub fn health_check(env: &Env) -> HealthStatus {
    let mut status = HealthStatus::Healthy;
    
    // Check system health
    if !is_system_healthy(&env) {
        status = HealthStatus::Unhealthy;
    }
    
    // Check error rates
    let error_rate = get_current_error_rate(&env);
    if error_rate > ERROR_RATE_THRESHOLD {
        status = HealthStatus::Degraded;
    }
    
    // Check storage
    if storage_usage_percentage(&env) > STORAGE_WARNING_THRESHOLD {
        status = HealthStatus::Degraded;
    }
    
    status
}
```

---

*This troubleshooting guide is maintained by the StrellerMinds development team. Last updated: March 2026*
