# Shared Contract Data Structures

## Overview

This document provides comprehensive documentation for all data structures used in the Shared contract. These structures form the foundation for access control, security, validation, and error handling across the entire StrellerMinds ecosystem.

## Access Control Data Structures

### Role

**Purpose**: Defines user roles within the system with hierarchical permissions.

```rust
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum Role {
    /// System-level operations and maintenance
    System,
    /// Super administrator with full system access
    SuperAdmin,
    /// Regular administrator with most permissions
    Admin,
    /// Content moderator with limited admin rights
    Moderator,
    /// Course instructor with teaching permissions
    Instructor,
    /// Content creator with publishing rights
    ContentCreator,
    /// Regular student with basic access
    Student,
}
```

**Field Descriptions**:
- `System`: Highest privilege level for system operations
- `SuperAdmin`: Can manage all aspects of the system
- `Admin`: Can manage users, content, and most settings
- `Moderator`: Can moderate content and manage users
- `Instructor`: Can create and manage courses, view student data
- `ContentCreator`: Can create and publish educational content
- `Student`: Basic access to learning materials

**Usage Examples**:
```rust
// Grant admin role to user
AccessControl::grant_role(&env, &admin, Role::Admin, &user_address)?;

// Check if user has instructor role
let is_instructor = AccessControl::has_role(&env, &user_address, Role::Instructor);

// Require super admin role for sensitive operations
AccessControl::require_role(&env, &caller, Role::SuperAdmin)?;
```

**Security Considerations**:
- Role hierarchy prevents privilege escalation
- Only higher roles can grant lower roles
- System role is reserved for contract operations
- Role assignments are logged for audit purposes

### Permission

**Purpose**: Defines specific permissions that can be granted to roles.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Permission {
    /// Unique identifier for the permission
    pub id: Symbol,
    /// Human-readable permission name
    pub name: String,
    /// Detailed description of what this permission allows
    pub description: String,
    /// Resource this permission applies to
    pub resource: String,
    /// Action this permission grants
    pub action: String,
    /// Whether this permission is critical for security
    pub is_critical: bool,
}
```

**Field Descriptions**:
- `id`: Unique symbol identifier (e.g., `Symbol::new(&env, "course_create")`)
- `name`: Human-readable name (e.g., `"Create Course"`)
- `description`: Detailed explanation of permission scope
- `resource`: Resource type (e.g., `"course"`, `"user"`, `"content"`)
- `action`: Allowed action (e.g., `"create"`, `"read"`, `"update"`, `"delete"`)
- `is_critical`: Marks permissions that affect system security

**Usage Examples**:
```rust
// Define a new permission
let permission = Permission {
    id: Symbol::new(&env, "course_delete"),
    name: String::from_str(&env, "Delete Course"),
    description: String::from_str(&env, "Allows deletion of courses and associated data"),
    resource: String::from_str(&env, "course"),
    action: String::from_str(&env, "delete"),
    is_critical: true,
};

// Grant permission to role
AccessControl::grant_permission(&env, &admin, Role::Admin, &permission)?;
```

**Common Permissions**:
- `course_create`: Create new courses
- `course_update`: Modify existing courses
- `course_delete`: Remove courses
- `user_manage`: Manage user accounts
- `content_publish`: Publish educational content
- `analytics_view`: View analytics data
- `system_config`: Modify system configuration

## Reentrancy Guard Data Structures

### ReentrancyLock

**Purpose**: Prevents reentrancy attacks by tracking function execution state.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ReentrancyLock {
    /// Unique identifier for the lock
    pub lock_id: BytesN<32>,
    /// Address that owns the lock
    pub owner: Address,
    /// Timestamp when lock was acquired
    pub locked_at: u64,
    /// Maximum duration for lock (in seconds)
    pub max_duration: u64,
    /// Current lock status
    pub is_active: bool,
    /// Function name that acquired the lock
    pub function_name: String,
}
```

**Field Descriptions**:
- `lock_id`: Unique identifier generated for each lock instance
- `owner`: Address that currently holds the lock
- `locked_at`: Unix timestamp when lock was acquired
- `max_duration`: Maximum time lock can be held (prevents deadlocks)
- `is_active`: Current status of the lock
- `function_name`: Name of function that acquired the lock for debugging

**Usage Examples**:
```rust
// Acquire reentrancy lock
ReentrancyLock::enter(&env)?;

// Perform protected operations
self.protected_function(&env)?;

// Release lock
ReentrancyLock::exit(&env)?;
```

**Security Features**:
- Automatic lock expiration prevents deadlocks
- Lock ownership verification prevents unauthorized release
- Function tracking for debugging and audit trails
- Gas-efficient implementation for minimal overhead

### CircuitBreakerState

**Purpose**: Implements circuit breaker pattern for protection against cascading failures.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CircuitBreakerState {
    /// Current state of the circuit breaker
    pub state: CircuitState,
    /// Number of consecutive failures
    pub failure_count: u32,
    /// Threshold for opening circuit
    pub failure_threshold: u32,
    /// Timestamp when circuit was last opened
    pub opened_at: Option<u64>,
    /// Duration to keep circuit open (in seconds)
    pub timeout_duration: u64,
    /// Number of successful operations since last failure
    pub success_count: u32,
    /// Threshold for closing circuit
    pub success_threshold: u32,
}
```

**Field Descriptions**:
- `state`: Current circuit state (Closed, Open, Half-Open)
- `failure_count`: Tracks consecutive failures
- `failure_threshold`: Number of failures before opening circuit
- `opened_at`: Timestamp when circuit was opened
- `timeout_duration`: How long to stay in open state
- `success_count`: Successful operations in half-open state
- `success_threshold`: Successes needed to close circuit

**Circuit States**:
```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CircuitState {
    /// Normal operation, requests pass through
    Closed,
    /// Circuit is open, requests are rejected
    Open,
    /// Testing if service has recovered
    HalfOpen,
}
```

**Usage Examples**:
```rust
// Check circuit breaker before operation
if !circuit_breaker.allow_request(&env)? {
    return Err(Error::from_contract_error(5001)); // Circuit open error
}

// Execute operation
let result = self.execute_operation(&env)?;

// Record success
circuit_breaker.record_success(&env)?;

Ok(result)
```

## Error Handling Data Structures

### ContractError

**Purpose**: Centralized error type for consistent error handling across contracts.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContractError {
    /// Access control related errors
    AccessControl(AccessControlError),
    /// Input validation errors
    Validation(ValidationError),
    /// Reentrancy protection errors
    Reentrancy(ReentrancyError),
    /// System-level errors
    System(SystemError),
    /// Business logic errors
    Business(BusinessError),
}
```

**Error Categories**:

#### AccessControlError
```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AccessControlError {
    /// Caller lacks required role
    Unauthorized,
    /// Role does not exist
    InvalidRole,
    /// Cannot grant higher role
    InsufficientPrivilege,
    /// User already has role
    RoleAlreadyAssigned,
    /// User does not have role
    RoleNotAssigned,
    /// Admin operations not allowed
    AdminOperationRequired,
}
```

#### ValidationError
```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ValidationError {
    /// Address is invalid or zero
    InvalidAddress,
    /// Amount is outside valid range
    InvalidAmount,
    /// String length exceeds limits
    InvalidStringLength,
    /// Invalid symbol format
    InvalidSymbol,
    /// Required field is missing
    MissingField,
    /// Data format is incorrect
    InvalidFormat,
    /// Value outside allowed range
    OutOfRange,
}
```

#### ReentrancyError
```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReentrancyError {
    /// Reentrant call detected
    ReentrantCall,
    /// Lock acquisition failed
    LockFailed,
    /// Lock not found
    LockNotFound,
    /// Lock expired
    LockExpired,
    /// Lock not owned by caller
    NotLockOwner,
}
```

**Usage Examples**:
```rust
// Convert domain errors to contract errors
pub fn validate_user_input(env: &Env, input: &UserInput) -> Result<(), ContractError> {
    if input.address.is_zero() {
        return Err(ContractError::Validation(ValidationError::InvalidAddress));
    }
    
    if input.amount == 0 || input.amount > MAX_AMOUNT {
        return Err(ContractError::Validation(ValidationError::InvalidAmount));
    }
    
    Ok(())
}
```

**Error Handling Best Practices**:
- Use specific error types for better debugging
- Include error codes for programmatic handling
- Provide human-readable error messages
- Log errors for monitoring and debugging
- Implement error recovery where possible

## Validation Data Structures

### ValidationRule

**Purpose**: Defines validation rules for different data types and contexts.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationRule {
    /// Unique identifier for the rule
    pub id: Symbol,
    /// Type of data this rule applies to
    pub data_type: DataType,
    /// Validation pattern or constraints
    pub pattern: String,
    /// Minimum allowed value (for numeric types)
    pub min_value: Option<u128>,
    /// Maximum allowed value (for numeric types)
    pub max_value: Option<u128>,
    /// Minimum string length
    pub min_length: Option<u32>,
    /// Maximum string length
    pub max_length: Option<u32>,
    /// Whether this rule is required
    pub is_required: bool,
    /// Custom validation function reference
    pub custom_validator: Option<Symbol>,
}
```

**Field Descriptions**:
- `id`: Unique rule identifier
- `data_type`: Type of data (Address, Amount, String, Symbol, etc.)
- `pattern`: Regex pattern or validation rule description
- `min_value`/`max_value`: Numeric constraints
- `min_length`/`max_length`: String length constraints
- `is_required`: Whether the field must be present
- `custom_validator`: Reference to custom validation logic

**Data Types**:
```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataType {
    Address,
    Amount,
    String,
    Symbol,
    Bytes,
    Boolean,
    Integer,
    Float,
    Array,
    Map,
    Custom(Symbol),
}
```

**Usage Examples**:
```rust
// Define validation rule for course names
let course_name_rule = ValidationRule {
    id: Symbol::new(&env, "course_name"),
    data_type: DataType::String,
    pattern: String::from_str(&env, "^[a-zA-Z0-9\\s\\-]{3,100}$"),
    min_length: Some(3),
    max_length: Some(100),
    is_required: true,
    custom_validator: None,
};

// Validate input against rule
Validation::validate_against_rule(&env, &course_name, &course_name_rule)?;
```

### ValidationResult

**Purpose**: Contains the result of validation operations.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ValidationResult {
    /// Whether validation passed
    pub is_valid: bool,
    /// List of validation errors
    pub errors: Vec<ValidationError>,
    /// Validated and potentially transformed data
    pub validated_data: Option<Bytes>,
    /// Validation metadata
    pub metadata: Map<Symbol, String>,
}
```

**Field Descriptions**:
- `is_valid`: Overall validation result
- `errors`: Detailed error information
- `validated_data`: Cleaned/transformed data if validation succeeded
- `metadata`: Additional validation information

**Usage Examples**:
```rust
// Validate complex input
let result = Validation::validate_complex_input(&env, &input_data)?;

if !result.is_valid {
    // Handle validation errors
    for error in result.errors {
        log_error(&env, &error);
    }
    return Err(ContractError::Validation(error));
}

// Use validated data
let clean_data = result.validated_data.unwrap();
```

## Configuration Data Structures

### ContractConfig

**Purpose**: Centralized configuration for contract behavior and parameters.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContractConfig {
    /// Contract version
    pub version: String,
    /// Maximum number of roles per user
    pub max_roles_per_user: u32,
    /// Default lock duration for reentrancy protection
    pub default_lock_duration: u64,
    /// Maximum lock duration
    pub max_lock_duration: u64,
    /// Circuit breaker failure threshold
    pub circuit_breaker_failure_threshold: u32,
    /// Circuit breaker timeout duration
    pub circuit_breaker_timeout: u64,
    /// Maximum string length for validation
    pub max_string_length: u32,
    /// Maximum array length for validation
    pub max_array_length: u32,
    /// Whether to enable debug logging
    pub enable_debug_logging: bool,
    /// Gas limit for expensive operations
    pub gas_limit: u64,
}
```

**Field Descriptions**:
- `version`: Semantic version of the contract
- `max_roles_per_user`: Prevents role explosion
- `default_lock_duration`: Standard reentrancy lock time
- `max_lock_duration`: Maximum allowed lock time
- `circuit_breaker_*`: Circuit breaker configuration
- `max_*_length`: Validation limits
- `enable_debug_logging`: Debug mode toggle
- `gas_limit`: Safety limit for expensive operations

**Usage Examples**:
```rust
// Get current configuration
let config = ContractConfig::get(&env)?;

// Check against configuration
if user.roles.len() > config.max_roles_per_user {
    return Err(ContractError::Validation(ValidationError::TooManyRoles));
}

// Use configuration values
ReentrancyLock::set_duration(&env, config.default_lock_duration)?;
```

### FeatureFlag

**Purpose**: Controls feature availability and experimental functionality.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FeatureFlag {
    /// Feature identifier
    pub id: Symbol,
    /// Whether feature is enabled
    pub enabled: bool,
    /// Feature description
    pub description: String,
    /// List of allowed users (if restricted)
    pub allowed_users: Vec<Address>,
    /// List of blocked users
    pub blocked_users: Vec<Address>,
    /// Feature activation time
    pub activation_time: Option<u64>,
    /// Feature deactivation time
    pub deactivation_time: Option<u64>,
    /// Feature version
    pub version: u32,
}
```

**Usage Examples**:
```rust
// Check if feature is enabled for user
let feature = FeatureFlag::get(&env, &feature_id)?;
if !feature.is_enabled_for_user(&user_address) {
    return Err(ContractError::Business(BusinessError::FeatureDisabled));
}
```

## Audit and Logging Data Structures

### AuditLog

**Purpose**: Records important events for security and compliance.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuditLog {
    /// Unique log entry identifier
    pub id: BytesN<32>,
    /// Timestamp of the event
    pub timestamp: u64,
    /// Type of event
    pub event_type: Symbol,
    /// User who performed the action
    pub actor: Address,
    /// Target of the action (if applicable)
    pub target: Option<Address>,
    /// Action performed
    pub action: String,
    /// Additional event data
    pub data: Map<Symbol, String>,
    /// IP address or origin
    pub origin: Option<String>,
    /// Whether this was a successful operation
    pub success: bool,
    /// Error message (if operation failed)
    pub error_message: Option<String>,
}
```

**Usage Examples**:
```rust
// Log role assignment
AuditLog::create(&env, AuditLog {
    id: generate_log_id(&env),
    timestamp: env.ledger().timestamp(),
    event_type: Symbol::new(&env, "role_assigned"),
    actor: admin_address,
    target: Some(user_address),
    action: String::from_str(&env, "grant_role"),
    data: map!(&env, (Symbol::new(&env, "role"), role.to_string())),
    origin: None,
    success: true,
    error_message: None,
})?;
```

### SecurityMetrics

**Purpose**: Tracks security-related metrics for monitoring.

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityMetrics {
    /// Number of failed authentication attempts
    pub failed_authentications: u64,
    /// Number of blocked reentrancy attempts
    pub blocked_reentrancy: u64,
    /// Number of circuit breaker activations
    pub circuit_breaker_activations: u64,
    /// Number of validation failures
    pub validation_failures: u64,
    /// Number of unauthorized access attempts
    pub unauthorized_attempts: u64,
    /// Last security incident timestamp
    pub last_incident: Option<u64>,
    /// Current security level
    pub security_level: SecurityLevel,
}
```

**Security Levels**:
```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SecurityLevel {
    Normal,
    Elevated,
    High,
    Critical,
}
```

## Usage Patterns and Best Practices

### 1. Error Handling Pattern

```rust
// Use Result type for all functions that can fail
pub fn protected_operation(env: &Env, caller: Address, data: Data) -> Result<ReturnValue, ContractError> {
    // 1. Input validation
    Validation::validate_data(&env, &data)
        .map_err(ContractError::Validation)?;
    
    // 2. Access control
    AccessControl::require_role(&env, &caller, Role::Admin)
        .map_err(ContractError::AccessControl)?;
    
    // 3. Reentrancy protection
    ReentrancyLock::enter(&env)
        .map_err(ContractError::Reentrancy)?;
    
    // 4. Execute operation with cleanup
    let result = match self.execute_business_logic(&env, &data) {
        Ok(value) => value,
        Err(error) => {
            ReentrancyLock::exit(&env)?;
            return Err(error);
        }
    };
    
    // 5. Cleanup
    ReentrancyLock::exit(&env)
        .map_err(ContractError::Reentrancy)?;
    
    Ok(result)
}
```

### 2. Validation Pattern

```rust
// Create reusable validation functions
pub fn validate_course_input(env: &Env, input: &CourseInput) -> Result<(), ValidationError> {
    // Validate course name
    if input.name.len() < 3 || input.name.len() > 100 {
        return Err(ValidationError::InvalidStringLength);
    }
    
    // Validate course description
    if input.description.len() > 1000 {
        return Err(ValidationError::InvalidStringLength);
    }
    
    // Validate instructor address
    if input.instructor.is_zero() {
        return Err(ValidationError::InvalidAddress);
    }
    
    // Validate course fee
    if input.fee > MAX_COURSE_FEE {
        return Err(ValidationError::InvalidAmount);
    }
    
    Ok(())
}
```

### 3. Access Control Pattern

```rust
// Implement role-based access control
pub fn admin_only_function(env: &Env, caller: Address) -> Result<(), ContractError> {
    // Check if caller has admin role
    if !AccessControl::has_role(&env, &caller, Role::Admin) {
        // Log unauthorized attempt
        AuditLog::log_security_event(&env, &caller, "unauthorized_admin_access")?;
        
        return Err(ContractError::AccessControl(
            AccessControlError::Unauthorized
        ));
    }
    
    // Proceed with admin operation
    Ok(())
}
```

### 4. Configuration Pattern

```rust
// Use configuration for flexible behavior
pub fn get_validation_limits(env: &Env) -> ValidationLimits {
    let config = ContractConfig::get(&env);
    
    ValidationLimits {
        max_string_length: config.max_string_length,
        max_array_length: config.max_array_length,
        max_amount: config.max_transaction_amount,
    }
}
```

## Migration and Versioning

### Data Structure Versioning

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DataVersion {
    /// Version number
    pub version: u32,
    /// Migration timestamp
    pub migrated_at: u64,
    /// Previous version (if applicable)
    pub previous_version: Option<u32>,
    /// Migration function used
    pub migration_function: Option<Symbol>,
}
```

### Migration Pattern

```rust
// Handle data structure migrations
pub fn migrate_data_structure(env: &Env, from_version: u32, to_version: u32) -> Result<(), ContractError> {
    match (from_version, to_version) {
        (1, 2) => self.migrate_v1_to_v2(&env)?,
        (2, 3) => self.migrate_v2_to_v3(&env)?,
        _ => return Err(ContractError::System(SystemError::UnsupportedMigration)),
    }
    
    // Update version
    self.set_data_version(&env, to_version)?;
    
    Ok(())
}
```

## Conclusion

The Shared contract data structures provide a comprehensive foundation for secure, efficient, and maintainable smart contract development. By using these standardized structures, developers can ensure consistency across the entire StrellerMinds ecosystem while maintaining high security and performance standards.

Key benefits of these data structures:
- **Consistency**: Standardized patterns across all contracts
- **Security**: Built-in security features and validations
- **Maintainability**: Clear structure and documentation
- **Flexibility**: Configurable and extensible design
- **Performance**: Optimized for gas efficiency
- **Auditability**: Comprehensive logging and tracking

These structures serve as building blocks for the entire platform, enabling robust and scalable educational blockchain applications.
