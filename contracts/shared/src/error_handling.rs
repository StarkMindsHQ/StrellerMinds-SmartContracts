//! Comprehensive Error Handling and Recovery System
//!
//! This module provides intelligent error recovery mechanisms and enhances
//! contract reliability through:
//! - Error classification and severity levels
//! - Circuit breaker patterns for contract calls
//! - Retry mechanisms with exponential backoff
//! - Automated error recovery workflows
//! - Error monitoring and alerting integration

use soroban_sdk::{contracterror, contracttype, Address, Env, Symbol, Vec};

// ============================================================================
// Error Classification and Severity Levels
// ============================================================================

/// Error severity levels for classification
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ErrorSeverity {
    /// Low severity - informational, operation can continue
    Low = 1,
    /// Medium severity - warning, operation may continue with degraded functionality
    Medium = 2,
    /// High severity - error, operation should be retried or handled
    High = 3,
    /// Critical severity - fatal, requires immediate attention
    Critical = 4,
}

/// Error categories for classification
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum ErrorCategory {
    /// Network-related errors (timeouts, connectivity)
    Network = 1,
    /// Authentication and authorization errors
    Authorization = 2,
    /// Validation errors (invalid input)
    Validation = 3,
    /// Business logic errors
    BusinessLogic = 4,
    /// Storage/state errors
    Storage = 5,
    /// External service errors
    ExternalService = 6,
    /// Resource exhaustion (gas, memory)
    Resource = 7,
    /// Unknown/unclassified errors
    Unknown = 8,
}

/// Comprehensive contract error with classification
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum RecoveryError {
    // Circuit Breaker Errors (100-119)
    CircuitBreakerOpen = 100,
    CircuitBreakerHalfOpen = 101,
    TooManyFailures = 102,
    CooldownNotExpired = 103,

    // Retry Errors (120-139)
    MaxRetriesExceeded = 120,
    RetryDelayNotMet = 121,
    NonRetryableError = 122,
    RetryContextNotFound = 123,

    // Recovery Errors (140-159)
    RecoveryFailed = 140,
    RecoveryNotPossible = 141,
    RecoveryInProgress = 142,
    InvalidRecoveryState = 143,

    // Monitoring Errors (160-179)
    AlertingFailed = 160,
    MonitoringDisabled = 161,
    InvalidAlertConfig = 162,
    AlertThresholdExceeded = 163,

    // General Errors (180-199)
    NotInitialized = 180,
    AlreadyInitialized = 181,
    Unauthorized = 182,
    InvalidConfiguration = 183,
    OperationNotAllowed = 184,
}

/// Error context structure for debugging
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErrorContext {
    /// Unix timestamp of when the error occurred
    pub timestamp: u64,
    /// Error code
    pub error_code: u32,
    /// Error severity
    pub severity: ErrorSeverity,
    /// Error category
    pub category: ErrorCategory,
    /// Whether the error is retryable
    pub retryable: bool,
    /// Number of retry attempts made
    pub retry_count: u32,
    /// Additional context identifier
    pub context_id: Symbol,
}

impl ErrorContext {
    /// Create a new error context
    pub fn new(
        env: &Env,
        error_code: u32,
        severity: ErrorSeverity,
        category: ErrorCategory,
        context_id: Symbol,
    ) -> Self {
        Self {
            timestamp: env.ledger().timestamp(),
            error_code,
            severity,
            category,
            retryable: Self::is_category_retryable(&category),
            retry_count: 0,
            context_id,
        }
    }

    /// Determine if an error category is retryable
    fn is_category_retryable(category: &ErrorCategory) -> bool {
        matches!(
            category,
            ErrorCategory::Network | ErrorCategory::ExternalService | ErrorCategory::Resource
        )
    }

    /// Increment retry count
    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }

    /// Check if error should be escalated based on retry count
    pub fn should_escalate(&self, max_retries: u32) -> bool {
        self.retry_count >= max_retries
    }
}

// ============================================================================
// Circuit Breaker Pattern
// ============================================================================

/// Circuit breaker states
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CircuitState {
    /// Circuit is closed, requests flow normally
    Closed,
    /// Circuit is open, requests are blocked
    Open,
    /// Circuit is half-open, limited requests allowed for testing
    HalfOpen,
}

/// Circuit breaker configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening circuit
    pub failure_threshold: u32,
    /// Duration in seconds to keep circuit open
    pub reset_timeout: u64,
    /// Number of successful calls needed to close circuit from half-open
    pub success_threshold: u32,
    /// Maximum requests allowed in half-open state
    pub half_open_max_requests: u32,
}

impl CircuitBreakerConfig {
    /// Create default configuration
    pub fn default_config() -> Self {
        Self {
            failure_threshold: 5,
            reset_timeout: 60, // 60 seconds
            success_threshold: 3,
            half_open_max_requests: 3,
        }
    }

    /// Create a strict configuration for critical operations
    pub fn strict_config() -> Self {
        Self {
            failure_threshold: 3,
            reset_timeout: 120, // 2 minutes
            success_threshold: 5,
            half_open_max_requests: 1,
        }
    }

    /// Create a lenient configuration for non-critical operations
    pub fn lenient_config() -> Self {
        Self {
            failure_threshold: 10,
            reset_timeout: 30, // 30 seconds
            success_threshold: 2,
            half_open_max_requests: 5,
        }
    }
}

/// Circuit breaker state tracking
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CircuitBreakerState {
    /// Current circuit state
    pub state: CircuitState,
    /// Number of consecutive failures
    pub failure_count: u32,
    /// Number of consecutive successes (in half-open)
    pub success_count: u32,
    /// Timestamp when circuit was opened
    pub opened_at: u64,
    /// Number of requests in half-open state
    pub half_open_requests: u32,
    /// Total failures since creation
    pub total_failures: u64,
    /// Total successes since creation
    pub total_successes: u64,
}

impl CircuitBreakerState {
    /// Create a new circuit breaker state
    pub fn new() -> Self {
        Self {
            state: CircuitState::Closed,
            failure_count: 0,
            success_count: 0,
            opened_at: 0,
            half_open_requests: 0,
            total_failures: 0,
            total_successes: 0,
        }
    }
}

impl Default for CircuitBreakerState {
    fn default() -> Self {
        Self::new()
    }
}

/// Storage keys for circuit breaker
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CircuitBreakerKey {
    /// Circuit breaker configuration
    Config(Symbol),
    /// Circuit breaker state
    State(Symbol),
}

/// Circuit breaker implementation
pub struct CircuitBreaker;

impl CircuitBreaker {
    /// Initialize a circuit breaker for a specific operation
    pub fn initialize(env: &Env, operation_id: &Symbol, config: &CircuitBreakerConfig) {
        let config_key = CircuitBreakerKey::Config(operation_id.clone());
        let state_key = CircuitBreakerKey::State(operation_id.clone());

        env.storage().instance().set(&config_key, config);
        env.storage()
            .instance()
            .set(&state_key, &CircuitBreakerState::new());
    }

    /// Check if a request can proceed through the circuit breaker
    pub fn can_proceed(env: &Env, operation_id: &Symbol) -> Result<bool, RecoveryError> {
        let config_key = CircuitBreakerKey::Config(operation_id.clone());
        let state_key = CircuitBreakerKey::State(operation_id.clone());

        let config: CircuitBreakerConfig = env
            .storage()
            .instance()
            .get(&config_key)
            .ok_or(RecoveryError::NotInitialized)?;

        let mut state: CircuitBreakerState = env
            .storage()
            .instance()
            .get(&state_key)
            .ok_or(RecoveryError::NotInitialized)?;

        let current_time = env.ledger().timestamp();

        match state.state {
            CircuitState::Closed => Ok(true),
            CircuitState::Open => {
                // Check if timeout has passed
                if current_time >= state.opened_at + config.reset_timeout {
                    // Transition to half-open
                    state.state = CircuitState::HalfOpen;
                    state.half_open_requests = 0;
                    state.success_count = 0;
                    env.storage().instance().set(&state_key, &state);
                    Ok(true)
                } else {
                    Err(RecoveryError::CircuitBreakerOpen)
                }
            }
            CircuitState::HalfOpen => {
                // Allow limited requests
                if state.half_open_requests < config.half_open_max_requests {
                    state.half_open_requests += 1;
                    env.storage().instance().set(&state_key, &state);
                    Ok(true)
                } else {
                    Err(RecoveryError::CircuitBreakerHalfOpen)
                }
            }
        }
    }

    /// Record a successful operation
    pub fn record_success(env: &Env, operation_id: &Symbol) -> Result<(), RecoveryError> {
        let config_key = CircuitBreakerKey::Config(operation_id.clone());
        let state_key = CircuitBreakerKey::State(operation_id.clone());

        let config: CircuitBreakerConfig = env
            .storage()
            .instance()
            .get(&config_key)
            .ok_or(RecoveryError::NotInitialized)?;

        let mut state: CircuitBreakerState = env
            .storage()
            .instance()
            .get(&state_key)
            .ok_or(RecoveryError::NotInitialized)?;

        state.total_successes += 1;

        match state.state {
            CircuitState::Closed => {
                // Reset failure count on success
                state.failure_count = 0;
            }
            CircuitState::HalfOpen => {
                state.success_count += 1;
                // Close circuit if success threshold reached
                if state.success_count >= config.success_threshold {
                    state.state = CircuitState::Closed;
                    state.failure_count = 0;
                    state.success_count = 0;
                    state.half_open_requests = 0;

                    // Emit recovery event
                    Self::emit_circuit_closed(env, operation_id);
                }
            }
            CircuitState::Open => {
                // Should not happen, but handle gracefully
            }
        }

        env.storage().instance().set(&state_key, &state);
        Ok(())
    }

    /// Record a failed operation
    pub fn record_failure(env: &Env, operation_id: &Symbol) -> Result<(), RecoveryError> {
        let config_key = CircuitBreakerKey::Config(operation_id.clone());
        let state_key = CircuitBreakerKey::State(operation_id.clone());

        let config: CircuitBreakerConfig = env
            .storage()
            .instance()
            .get(&config_key)
            .ok_or(RecoveryError::NotInitialized)?;

        let mut state: CircuitBreakerState = env
            .storage()
            .instance()
            .get(&state_key)
            .ok_or(RecoveryError::NotInitialized)?;

        state.total_failures += 1;
        state.failure_count += 1;

        match state.state {
            CircuitState::Closed => {
                if state.failure_count >= config.failure_threshold {
                    state.state = CircuitState::Open;
                    state.opened_at = env.ledger().timestamp();

                    // Emit circuit opened event
                    Self::emit_circuit_opened(env, operation_id, state.failure_count);
                }
            }
            CircuitState::HalfOpen => {
                // Any failure in half-open returns to open
                state.state = CircuitState::Open;
                state.opened_at = env.ledger().timestamp();
                state.success_count = 0;
                state.half_open_requests = 0;

                Self::emit_circuit_opened(env, operation_id, state.failure_count);
            }
            CircuitState::Open => {
                // Already open, just count
            }
        }

        env.storage().instance().set(&state_key, &state);
        Ok(())
    }

    /// Get current circuit breaker status
    pub fn get_status(
        env: &Env,
        operation_id: &Symbol,
    ) -> Result<CircuitBreakerState, RecoveryError> {
        let state_key = CircuitBreakerKey::State(operation_id.clone());
        env.storage()
            .instance()
            .get(&state_key)
            .ok_or(RecoveryError::NotInitialized)
    }

    /// Force reset the circuit breaker (admin only)
    pub fn force_reset(env: &Env, operation_id: &Symbol) -> Result<(), RecoveryError> {
        let state_key = CircuitBreakerKey::State(operation_id.clone());

        if !env.storage().instance().has(&state_key) {
            return Err(RecoveryError::NotInitialized);
        }

        env.storage()
            .instance()
            .set(&state_key, &CircuitBreakerState::new());

        Self::emit_circuit_reset(env, operation_id);
        Ok(())
    }

    // Event emission helpers
    fn emit_circuit_opened(env: &Env, operation_id: &Symbol, failure_count: u32) {
        let topics = (
            Symbol::new(env, "circuit_opened"),
            operation_id.clone(),
            failure_count,
        );
        env.events().publish(topics, ());
    }

    fn emit_circuit_closed(env: &Env, operation_id: &Symbol) {
        let topics = (Symbol::new(env, "circuit_closed"), operation_id.clone());
        env.events().publish(topics, ());
    }

    fn emit_circuit_reset(env: &Env, operation_id: &Symbol) {
        let topics = (Symbol::new(env, "circuit_reset"), operation_id.clone());
        env.events().publish(topics, ());
    }
}

// ============================================================================
// Retry Mechanism with Exponential Backoff
// ============================================================================

/// Retry strategy types
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RetryStrategy {
    /// Fixed delay between retries
    Fixed,
    /// Linear increase in delay
    Linear,
    /// Exponential backoff
    Exponential,
    /// Exponential backoff with jitter
    ExponentialWithJitter,
}

/// Retry configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_retries: u32,
    /// Base delay in seconds
    pub base_delay: u64,
    /// Maximum delay in seconds
    pub max_delay: u64,
    /// Retry strategy
    pub strategy: RetryStrategy,
    /// Multiplier for exponential backoff
    pub multiplier: u32,
}

impl RetryConfig {
    /// Create default retry configuration
    pub fn default_config() -> Self {
        Self {
            max_retries: 3,
            base_delay: 1,
            max_delay: 30,
            strategy: RetryStrategy::Exponential,
            multiplier: 2,
        }
    }

    /// Create aggressive retry configuration
    pub fn aggressive_config() -> Self {
        Self {
            max_retries: 5,
            base_delay: 1,
            max_delay: 60,
            strategy: RetryStrategy::ExponentialWithJitter,
            multiplier: 2,
        }
    }

    /// Create conservative retry configuration
    pub fn conservative_config() -> Self {
        Self {
            max_retries: 2,
            base_delay: 5,
            max_delay: 20,
            strategy: RetryStrategy::Fixed,
            multiplier: 1,
        }
    }
}

/// Retry context for tracking retry state
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryContext {
    /// Current retry attempt
    pub attempt: u32,
    /// Next retry timestamp
    pub next_retry_at: u64,
    /// Operation identifier
    pub operation_id: Symbol,
    /// Whether retries are exhausted
    pub exhausted: bool,
    /// Last error code
    pub last_error: u32,
}

/// Storage keys for retry mechanism
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RetryKey {
    Config(Symbol),
    Context(Symbol),
}

/// Retry mechanism implementation
pub struct RetryMechanism;

impl RetryMechanism {
    /// Initialize retry configuration for an operation
    pub fn initialize(env: &Env, operation_id: &Symbol, config: &RetryConfig) {
        let config_key = RetryKey::Config(operation_id.clone());
        env.storage().instance().set(&config_key, config);
    }

    /// Calculate delay for the next retry attempt
    pub fn calculate_delay(config: &RetryConfig, attempt: u32) -> u64 {
        let delay = match config.strategy {
            RetryStrategy::Fixed => config.base_delay,
            RetryStrategy::Linear => config.base_delay * (attempt as u64 + 1),
            RetryStrategy::Exponential => {
                config.base_delay * (config.multiplier as u64).pow(attempt)
            }
            RetryStrategy::ExponentialWithJitter => {
                // Exponential with simple jitter (variation based on attempt number)
                let base = config.base_delay * (config.multiplier as u64).pow(attempt);
                // Add pseudo-jitter: vary by +/- 20% based on attempt
                let jitter_factor = if attempt.is_multiple_of(2) { 80 } else { 120 };
                (base * jitter_factor) / 100
            }
        };

        // Cap at max_delay
        if delay > config.max_delay {
            config.max_delay
        } else {
            delay
        }
    }

    /// Start a new retry sequence
    pub fn start_retry(
        env: &Env,
        operation_id: &Symbol,
        error_code: u32,
    ) -> Result<RetryContext, RecoveryError> {
        let config_key = RetryKey::Config(operation_id.clone());
        let context_key = RetryKey::Context(operation_id.clone());

        let config: RetryConfig = env
            .storage()
            .instance()
            .get(&config_key)
            .ok_or(RecoveryError::NotInitialized)?;

        // Check for existing context
        let mut context: RetryContext =
            env.storage()
                .instance()
                .get(&context_key)
                .unwrap_or(RetryContext {
                    attempt: 0,
                    next_retry_at: 0,
                    operation_id: operation_id.clone(),
                    exhausted: false,
                    last_error: 0,
                });

        if context.exhausted {
            return Err(RecoveryError::MaxRetriesExceeded);
        }

        context.attempt += 1;
        context.last_error = error_code;

        if context.attempt > config.max_retries {
            context.exhausted = true;
            env.storage().instance().set(&context_key, &context);

            Self::emit_retries_exhausted(env, operation_id, context.attempt);
            return Err(RecoveryError::MaxRetriesExceeded);
        }

        let delay = Self::calculate_delay(&config, context.attempt - 1);
        context.next_retry_at = env.ledger().timestamp() + delay;

        env.storage().instance().set(&context_key, &context);

        Self::emit_retry_scheduled(env, operation_id, context.attempt, delay);

        Ok(context)
    }

    /// Check if retry is allowed (delay has passed)
    pub fn can_retry(env: &Env, operation_id: &Symbol) -> Result<bool, RecoveryError> {
        let context_key = RetryKey::Context(operation_id.clone());

        let context: RetryContext = env
            .storage()
            .instance()
            .get(&context_key)
            .ok_or(RecoveryError::RetryContextNotFound)?;

        if context.exhausted {
            return Err(RecoveryError::MaxRetriesExceeded);
        }

        Ok(env.ledger().timestamp() >= context.next_retry_at)
    }

    /// Mark retry as successful (clear context)
    pub fn mark_success(env: &Env, operation_id: &Symbol) -> Result<(), RecoveryError> {
        let context_key = RetryKey::Context(operation_id.clone());

        if !env.storage().instance().has(&context_key) {
            return Ok(()); // No retry context, operation succeeded on first try
        }

        env.storage().instance().remove(&context_key);
        Self::emit_retry_success(env, operation_id);

        Ok(())
    }

    /// Get current retry context
    pub fn get_context(env: &Env, operation_id: &Symbol) -> Option<RetryContext> {
        let context_key = RetryKey::Context(operation_id.clone());
        env.storage().instance().get(&context_key)
    }

    /// Reset retry context (for manual intervention)
    pub fn reset(env: &Env, operation_id: &Symbol) {
        let context_key = RetryKey::Context(operation_id.clone());
        env.storage().instance().remove(&context_key);
    }

    // Event emission helpers
    fn emit_retry_scheduled(env: &Env, operation_id: &Symbol, attempt: u32, delay: u64) {
        let topics = (
            Symbol::new(env, "retry_scheduled"),
            operation_id.clone(),
            attempt,
        );
        env.events().publish(topics, delay);
    }

    fn emit_retries_exhausted(env: &Env, operation_id: &Symbol, attempts: u32) {
        let topics = (
            Symbol::new(env, "retries_exhausted"),
            operation_id.clone(),
            attempts,
        );
        env.events().publish(topics, ());
    }

    fn emit_retry_success(env: &Env, operation_id: &Symbol) {
        let topics = (Symbol::new(env, "retry_success"), operation_id.clone());
        env.events().publish(topics, ());
    }
}

// ============================================================================
// Automated Error Recovery Workflows
// ============================================================================

/// Recovery action types
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryAction {
    /// Retry the operation
    Retry,
    /// Use fallback/default value
    Fallback,
    /// Skip the operation
    Skip,
    /// Escalate to admin
    Escalate,
    /// Pause the contract/feature
    Pause,
    /// Rollback to previous state
    Rollback,
    /// No action needed
    None,
}

/// Recovery workflow state
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryState {
    /// Recovery not started
    Idle,
    /// Recovery in progress
    InProgress,
    /// Recovery completed successfully
    Completed,
    /// Recovery failed
    Failed,
    /// Recovery requires manual intervention
    PendingIntervention,
}

/// Recovery workflow definition
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryWorkflow {
    /// Workflow identifier
    pub workflow_id: Symbol,
    /// Current state
    pub state: RecoveryState,
    /// Primary recovery action
    pub primary_action: RecoveryAction,
    /// Fallback recovery action
    pub fallback_action: RecoveryAction,
    /// Maximum recovery attempts
    pub max_attempts: u32,
    /// Current attempt
    pub current_attempt: u32,
    /// Timestamp when recovery started
    pub started_at: u64,
    /// Timestamp of last action
    pub last_action_at: u64,
}

impl RecoveryWorkflow {
    /// Create a new recovery workflow
    pub fn new(
        env: &Env,
        workflow_id: Symbol,
        primary_action: RecoveryAction,
        fallback_action: RecoveryAction,
        max_attempts: u32,
    ) -> Self {
        Self {
            workflow_id,
            state: RecoveryState::Idle,
            primary_action,
            fallback_action,
            max_attempts,
            current_attempt: 0,
            started_at: 0,
            last_action_at: env.ledger().timestamp(),
        }
    }
}

/// Storage keys for recovery system
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveryKey {
    Workflow(Symbol),
    ErrorLog(Symbol, u32), // operation_id, sequence
    AlertConfig,
    ErrorCount(Symbol), // operation_id
}

/// Automated recovery system
pub struct RecoverySystem;

impl RecoverySystem {
    /// Determine appropriate recovery action based on error context
    pub fn determine_recovery_action(context: &ErrorContext) -> RecoveryAction {
        match (&context.severity, &context.category) {
            // Critical errors require immediate escalation
            (ErrorSeverity::Critical, _) => RecoveryAction::Escalate,

            // High severity authorization errors - escalate
            (ErrorSeverity::High, ErrorCategory::Authorization) => RecoveryAction::Escalate,

            // Medium/Low authorization - fallback
            (ErrorSeverity::Medium, ErrorCategory::Authorization) => RecoveryAction::Fallback,
            (ErrorSeverity::Low, ErrorCategory::Authorization) => RecoveryAction::Skip,

            // Network errors are typically retryable
            (_, ErrorCategory::Network) => {
                if context.retry_count < 3 {
                    RecoveryAction::Retry
                } else {
                    RecoveryAction::Fallback
                }
            }

            // External service errors - retry then fallback
            (_, ErrorCategory::ExternalService) => {
                if context.retry_count < 2 {
                    RecoveryAction::Retry
                } else {
                    RecoveryAction::Fallback
                }
            }

            // Validation errors - skip or use fallback
            (_, ErrorCategory::Validation) => RecoveryAction::Skip,

            // Business logic errors - may need escalation
            (ErrorSeverity::High, ErrorCategory::BusinessLogic) => RecoveryAction::Escalate,
            (_, ErrorCategory::BusinessLogic) => RecoveryAction::Fallback,

            // Storage errors - attempt rollback
            (_, ErrorCategory::Storage) => RecoveryAction::Rollback,

            // Resource exhaustion - pause and escalate
            (ErrorSeverity::High, ErrorCategory::Resource) => RecoveryAction::Pause,
            (_, ErrorCategory::Resource) => RecoveryAction::Retry,

            // Unknown errors - escalate
            (_, ErrorCategory::Unknown) => RecoveryAction::Escalate,
        }
    }

    /// Start a recovery workflow
    pub fn start_recovery(
        env: &Env,
        context: &ErrorContext,
    ) -> Result<RecoveryWorkflow, RecoveryError> {
        let workflow_key = RecoveryKey::Workflow(context.context_id.clone());

        // Check if recovery already in progress
        if let Some(existing) = env
            .storage()
            .instance()
            .get::<RecoveryKey, RecoveryWorkflow>(&workflow_key)
        {
            if existing.state == RecoveryState::InProgress {
                return Err(RecoveryError::RecoveryInProgress);
            }
        }

        let primary_action = Self::determine_recovery_action(context);
        let fallback_action = Self::determine_fallback_action(&primary_action);

        let mut workflow = RecoveryWorkflow::new(
            env,
            context.context_id.clone(),
            primary_action,
            fallback_action,
            3, // Default max attempts
        );

        workflow.state = RecoveryState::InProgress;
        workflow.started_at = env.ledger().timestamp();

        env.storage().instance().set(&workflow_key, &workflow);

        Self::emit_recovery_started(env, &context.context_id, primary_action);

        Ok(workflow)
    }

    /// Determine fallback action for primary action
    fn determine_fallback_action(primary: &RecoveryAction) -> RecoveryAction {
        match primary {
            RecoveryAction::Retry => RecoveryAction::Fallback,
            RecoveryAction::Fallback => RecoveryAction::Skip,
            RecoveryAction::Skip => RecoveryAction::Escalate,
            RecoveryAction::Rollback => RecoveryAction::Escalate,
            RecoveryAction::Pause => RecoveryAction::Escalate,
            RecoveryAction::Escalate | RecoveryAction::None => RecoveryAction::None,
        }
    }

    /// Execute recovery action
    pub fn execute_recovery(
        env: &Env,
        workflow_id: &Symbol,
    ) -> Result<RecoveryAction, RecoveryError> {
        let workflow_key = RecoveryKey::Workflow(workflow_id.clone());

        let mut workflow: RecoveryWorkflow = env
            .storage()
            .instance()
            .get(&workflow_key)
            .ok_or(RecoveryError::InvalidRecoveryState)?;

        if workflow.state != RecoveryState::InProgress {
            return Err(RecoveryError::InvalidRecoveryState);
        }

        workflow.current_attempt += 1;
        workflow.last_action_at = env.ledger().timestamp();

        let action = if workflow.current_attempt <= workflow.max_attempts {
            workflow.primary_action
        } else {
            workflow.fallback_action
        };

        // Update workflow state based on action
        if action == RecoveryAction::Escalate || action == RecoveryAction::None {
            workflow.state = RecoveryState::PendingIntervention;
        }

        env.storage().instance().set(&workflow_key, &workflow);

        Ok(action)
    }

    /// Mark recovery as completed
    pub fn complete_recovery(
        env: &Env,
        workflow_id: &Symbol,
        success: bool,
    ) -> Result<(), RecoveryError> {
        let workflow_key = RecoveryKey::Workflow(workflow_id.clone());

        let mut workflow: RecoveryWorkflow = env
            .storage()
            .instance()
            .get(&workflow_key)
            .ok_or(RecoveryError::InvalidRecoveryState)?;

        workflow.state = if success {
            RecoveryState::Completed
        } else {
            RecoveryState::Failed
        };

        env.storage().instance().set(&workflow_key, &workflow);

        Self::emit_recovery_completed(env, workflow_id, success);

        Ok(())
    }

    /// Get current recovery workflow state
    pub fn get_workflow(env: &Env, workflow_id: &Symbol) -> Option<RecoveryWorkflow> {
        let workflow_key = RecoveryKey::Workflow(workflow_id.clone());
        env.storage().instance().get(&workflow_key)
    }

    // Event emission helpers
    fn emit_recovery_started(env: &Env, workflow_id: &Symbol, action: RecoveryAction) {
        let topics = (
            Symbol::new(env, "recovery_started"),
            workflow_id.clone(),
            action as u32,
        );
        env.events().publish(topics, ());
    }

    fn emit_recovery_completed(env: &Env, workflow_id: &Symbol, success: bool) {
        let topics = (
            Symbol::new(env, "recovery_completed"),
            workflow_id.clone(),
            success,
        );
        env.events().publish(topics, ());
    }
}

// ============================================================================
// Error Monitoring and Alerting
// ============================================================================

/// Alert severity levels
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum AlertLevel {
    /// Informational alert
    Info = 1,
    /// Warning alert
    Warning = 2,
    /// High severity alert
    High = 3,
    /// Critical alert
    Critical = 4,
}

/// Alert configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AlertConfig {
    /// Whether alerting is enabled
    pub enabled: bool,
    /// Minimum alert level to trigger
    pub min_level: AlertLevel,
    /// Error count threshold for alerts
    pub error_threshold: u32,
    /// Time window for error counting (seconds)
    pub time_window: u64,
    /// Admin address for escalation
    pub admin: Address,
}

impl AlertConfig {
    /// Create default alert configuration
    pub fn new(admin: Address) -> Self {
        Self {
            enabled: true,
            min_level: AlertLevel::Warning,
            error_threshold: 5,
            time_window: 300, // 5 minutes
            admin,
        }
    }
}

/// Error log entry
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErrorLogEntry {
    /// Timestamp of the error
    pub timestamp: u64,
    /// Error code
    pub error_code: u32,
    /// Error severity
    pub severity: ErrorSeverity,
    /// Error category
    pub category: ErrorCategory,
    /// Operation that caused the error
    pub operation_id: Symbol,
}

/// Error monitoring system
pub struct ErrorMonitor;

impl ErrorMonitor {
    /// Initialize monitoring configuration
    pub fn initialize(env: &Env, config: &AlertConfig) {
        env.storage()
            .instance()
            .set(&RecoveryKey::AlertConfig, config);
    }

    /// Log an error and check for alerting
    pub fn log_error(
        env: &Env,
        context: &ErrorContext,
    ) -> Result<Option<AlertLevel>, RecoveryError> {
        let config: AlertConfig = env
            .storage()
            .instance()
            .get(&RecoveryKey::AlertConfig)
            .ok_or(RecoveryError::NotInitialized)?;

        if !config.enabled {
            return Err(RecoveryError::MonitoringDisabled);
        }

        // Increment error count
        let count_key = RecoveryKey::ErrorCount(context.context_id.clone());
        let current_count: u32 = env.storage().instance().get(&count_key).unwrap_or(0);
        let new_count = current_count + 1;
        env.storage().instance().set(&count_key, &new_count);

        // Create log entry
        let entry = ErrorLogEntry {
            timestamp: context.timestamp,
            error_code: context.error_code,
            severity: context.severity,
            category: context.category,
            operation_id: context.context_id.clone(),
        };

        // Emit error logged event
        Self::emit_error_logged(env, &entry);

        // Determine alert level
        let alert_level = Self::determine_alert_level(context, new_count, &config);

        if let Some(level) = alert_level {
            if level >= config.min_level {
                Self::emit_alert(env, &context.context_id, level, new_count);
                return Ok(Some(level));
            }
        }

        Ok(None)
    }

    /// Determine alert level based on error context and count
    fn determine_alert_level(
        context: &ErrorContext,
        error_count: u32,
        config: &AlertConfig,
    ) -> Option<AlertLevel> {
        // Critical severity always triggers critical alert
        if context.severity == ErrorSeverity::Critical {
            return Some(AlertLevel::Critical);
        }

        // High error count triggers alerts
        if error_count >= config.error_threshold * 2 {
            return Some(AlertLevel::Critical);
        }

        if error_count >= config.error_threshold {
            return Some(AlertLevel::High);
        }

        // Map severity to alert level
        match context.severity {
            ErrorSeverity::Critical => Some(AlertLevel::Critical),
            ErrorSeverity::High => Some(AlertLevel::High),
            ErrorSeverity::Medium => Some(AlertLevel::Warning),
            ErrorSeverity::Low => Some(AlertLevel::Info),
        }
    }

    /// Get error count for an operation
    pub fn get_error_count(env: &Env, operation_id: &Symbol) -> u32 {
        let count_key = RecoveryKey::ErrorCount(operation_id.clone());
        env.storage().instance().get(&count_key).unwrap_or(0)
    }

    /// Reset error count for an operation
    pub fn reset_error_count(env: &Env, operation_id: &Symbol) {
        let count_key = RecoveryKey::ErrorCount(operation_id.clone());
        env.storage().instance().remove(&count_key);
    }

    /// Get current alert configuration
    pub fn get_config(env: &Env) -> Option<AlertConfig> {
        env.storage().instance().get(&RecoveryKey::AlertConfig)
    }

    /// Update alert configuration
    pub fn update_config(env: &Env, config: &AlertConfig) {
        env.storage()
            .instance()
            .set(&RecoveryKey::AlertConfig, config);
    }

    // Event emission helpers
    fn emit_error_logged(env: &Env, entry: &ErrorLogEntry) {
        let topics = (
            Symbol::new(env, "error_logged"),
            entry.operation_id.clone(),
            entry.error_code,
        );
        env.events().publish(topics, entry.severity as u32);
    }

    fn emit_alert(env: &Env, operation_id: &Symbol, level: AlertLevel, error_count: u32) {
        let topics = (
            Symbol::new(env, "alert_triggered"),
            operation_id.clone(),
            level as u32,
        );
        env.events().publish(topics, error_count);
    }
}

// ============================================================================
// Fault-Tolerant Contract Interaction Patterns
// ============================================================================

/// Result status for fault-tolerant operations
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FaultTolerantStatus {
    /// Operation succeeded
    Success,
    /// Operation failed but has fallback value
    Fallback,
    /// Operation failed, skipped
    Skipped,
    /// Operation failed, requires retry
    PendingRetry,
    /// Operation failed, escalated
    Escalated,
}

/// Graceful degradation configuration
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DegradationConfig {
    /// Whether degradation is enabled
    pub enabled: bool,
    /// Features that can be degraded
    pub degradable_features: Vec<Symbol>,
    /// Currently degraded features
    pub degraded_features: Vec<Symbol>,
}

/// Storage key for degradation
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DegradationKey {
    Config,
}

/// Graceful degradation system
pub struct GracefulDegradation;

impl GracefulDegradation {
    /// Initialize degradation configuration
    pub fn initialize(env: &Env, degradable_features: Vec<Symbol>) {
        let config = DegradationConfig {
            enabled: true,
            degradable_features,
            degraded_features: Vec::new(env),
        };
        env.storage()
            .instance()
            .set(&DegradationKey::Config, &config);
    }

    /// Check if a feature is currently degraded
    pub fn is_degraded(env: &Env, feature: &Symbol) -> bool {
        if let Some(config) = Self::get_config(env) {
            config.degraded_features.contains(feature)
        } else {
            false
        }
    }

    /// Degrade a feature
    pub fn degrade_feature(env: &Env, feature: &Symbol) -> Result<(), RecoveryError> {
        let mut config = Self::get_config(env).ok_or(RecoveryError::NotInitialized)?;

        if !config.enabled {
            return Err(RecoveryError::OperationNotAllowed);
        }

        if !config.degradable_features.contains(feature) {
            return Err(RecoveryError::InvalidConfiguration);
        }

        if !config.degraded_features.contains(feature) {
            config.degraded_features.push_back(feature.clone());
            env.storage()
                .instance()
                .set(&DegradationKey::Config, &config);

            Self::emit_feature_degraded(env, feature);
        }

        Ok(())
    }

    /// Restore a degraded feature
    pub fn restore_feature(env: &Env, feature: &Symbol) -> Result<(), RecoveryError> {
        let mut config = Self::get_config(env).ok_or(RecoveryError::NotInitialized)?;

        // Find and remove the feature from degraded list
        let mut index_to_remove: Option<u32> = None;
        for (i, f) in config.degraded_features.iter().enumerate() {
            if &f == feature {
                index_to_remove = Some(i as u32);
                break;
            }
        }

        if let Some(idx) = index_to_remove {
            config.degraded_features.remove(idx);
            env.storage()
                .instance()
                .set(&DegradationKey::Config, &config);

            Self::emit_feature_restored(env, feature);
        }

        Ok(())
    }

    /// Get current degradation configuration
    pub fn get_config(env: &Env) -> Option<DegradationConfig> {
        env.storage().instance().get(&DegradationKey::Config)
    }

    /// Get list of currently degraded features
    pub fn get_degraded_features(env: &Env) -> Vec<Symbol> {
        if let Some(config) = Self::get_config(env) {
            config.degraded_features
        } else {
            Vec::new(env)
        }
    }

    // Event emission helpers
    fn emit_feature_degraded(env: &Env, feature: &Symbol) {
        let topics = (Symbol::new(env, "feature_degraded"), feature.clone());
        env.events().publish(topics, ());
    }

    fn emit_feature_restored(env: &Env, feature: &Symbol) {
        let topics = (Symbol::new(env, "feature_restored"), feature.clone());
        env.events().publish(topics, ());
    }
}

// ============================================================================
// Health Check System
// ============================================================================

/// Health status
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum HealthStatus {
    /// System is healthy
    Healthy,
    /// System is degraded but operational
    Degraded,
    /// System is unhealthy
    Unhealthy,
    /// System status is unknown
    Unknown,
}

/// Health check result
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct HealthCheckResult {
    /// Overall health status
    pub status: HealthStatus,
    /// Timestamp of the check
    pub timestamp: u64,
    /// Number of open circuit breakers
    pub open_circuits: u32,
    /// Number of degraded features
    pub degraded_features: u32,
    /// Recent error count
    pub recent_errors: u32,
}

/// Health check system
pub struct HealthCheck;

impl HealthCheck {
    /// Perform a comprehensive health check
    pub fn check(env: &Env, circuit_breaker_ids: &Vec<Symbol>) -> HealthCheckResult {
        let mut open_circuits: u32 = 0;
        let mut total_errors: u32 = 0;

        // Check circuit breakers
        for id in circuit_breaker_ids.iter() {
            if let Ok(state) = CircuitBreaker::get_status(env, &id) {
                if state.state == CircuitState::Open {
                    open_circuits += 1;
                }
            }
            total_errors += ErrorMonitor::get_error_count(env, &id);
        }

        // Check degraded features
        let degraded_features = GracefulDegradation::get_degraded_features(env).len();

        // Determine overall status
        let status = if open_circuits > 0 || degraded_features > 2 {
            HealthStatus::Unhealthy
        } else if degraded_features > 0 || total_errors > 10 {
            HealthStatus::Degraded
        } else {
            HealthStatus::Healthy
        };

        HealthCheckResult {
            status,
            timestamp: env.ledger().timestamp(),
            open_circuits,
            degraded_features,
            recent_errors: total_errors,
        }
    }

    /// Emit health status event
    pub fn emit_health_status(env: &Env, result: &HealthCheckResult) {
        let topics = (Symbol::new(env, "health_check"), result.status as u32);
        env.events().publish(topics, result.timestamp);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use soroban_sdk::testutils::Address as _;
    use soroban_sdk::Env;

    #[test]
    fn test_error_context_creation() {
        let env = Env::default();
        let context_id = Symbol::new(&env, "test_op");

        let context = ErrorContext::new(
            &env,
            100,
            ErrorSeverity::High,
            ErrorCategory::Network,
            context_id,
        );

        assert_eq!(context.error_code, 100);
        assert_eq!(context.severity, ErrorSeverity::High);
        assert_eq!(context.category, ErrorCategory::Network);
        assert!(context.retryable); // Network errors are retryable
        assert_eq!(context.retry_count, 0);
    }

    #[test]
    fn test_circuit_breaker_initialization() {
        let env = Env::default();
        let operation_id = Symbol::new(&env, "test_circuit");
        let config = CircuitBreakerConfig::default_config();

        CircuitBreaker::initialize(&env, &operation_id, &config);

        let status = CircuitBreaker::get_status(&env, &operation_id).expect("status should exist");
        assert_eq!(status.state, CircuitState::Closed);
        assert_eq!(status.failure_count, 0);
    }

    #[test]
    fn test_circuit_breaker_opens_after_failures() {
        let env = Env::default();
        let operation_id = Symbol::new(&env, "test_circuit");
        let config = CircuitBreakerConfig {
            failure_threshold: 3,
            reset_timeout: 60,
            success_threshold: 2,
            half_open_max_requests: 1,
        };

        CircuitBreaker::initialize(&env, &operation_id, &config);

        // Record failures
        for _ in 0..3 {
            CircuitBreaker::record_failure(&env, &operation_id).expect("should record failure");
        }

        let status = CircuitBreaker::get_status(&env, &operation_id).expect("status should exist");
        assert_eq!(status.state, CircuitState::Open);

        // Can't proceed when open
        let result = CircuitBreaker::can_proceed(&env, &operation_id);
        assert!(result.is_err());
    }

    #[test]
    fn test_retry_mechanism() {
        let env = Env::default();
        let operation_id = Symbol::new(&env, "test_retry");
        let config = RetryConfig::default_config();

        RetryMechanism::initialize(&env, &operation_id, &config);

        // Start retry
        let context =
            RetryMechanism::start_retry(&env, &operation_id, 500).expect("should start retry");
        assert_eq!(context.attempt, 1);
        assert!(!context.exhausted);
    }

    #[test]
    fn test_retry_delay_calculation() {
        let config = RetryConfig {
            max_retries: 5,
            base_delay: 1,
            max_delay: 30,
            strategy: RetryStrategy::Exponential,
            multiplier: 2,
        };

        assert_eq!(RetryMechanism::calculate_delay(&config, 0), 1); // 1 * 2^0 = 1
        assert_eq!(RetryMechanism::calculate_delay(&config, 1), 2); // 1 * 2^1 = 2
        assert_eq!(RetryMechanism::calculate_delay(&config, 2), 4); // 1 * 2^2 = 4
        assert_eq!(RetryMechanism::calculate_delay(&config, 3), 8); // 1 * 2^3 = 8
        assert_eq!(RetryMechanism::calculate_delay(&config, 5), 30); // Capped at max_delay
    }

    #[test]
    fn test_recovery_action_determination() {
        let env = Env::default();

        // Critical error should escalate
        let critical_context = ErrorContext::new(
            &env,
            100,
            ErrorSeverity::Critical,
            ErrorCategory::BusinessLogic,
            Symbol::new(&env, "test"),
        );
        assert_eq!(
            RecoverySystem::determine_recovery_action(&critical_context),
            RecoveryAction::Escalate
        );

        // Network error should retry
        let network_context = ErrorContext::new(
            &env,
            101,
            ErrorSeverity::Medium,
            ErrorCategory::Network,
            Symbol::new(&env, "test"),
        );
        assert_eq!(
            RecoverySystem::determine_recovery_action(&network_context),
            RecoveryAction::Retry
        );

        // Validation error should skip
        let validation_context = ErrorContext::new(
            &env,
            102,
            ErrorSeverity::Low,
            ErrorCategory::Validation,
            Symbol::new(&env, "test"),
        );
        assert_eq!(
            RecoverySystem::determine_recovery_action(&validation_context),
            RecoveryAction::Skip
        );
    }

    #[test]
    fn test_error_monitor_initialization() {
        let env = Env::default();
        let admin = Address::generate(&env);
        let config = AlertConfig::new(admin);

        ErrorMonitor::initialize(&env, &config);

        let retrieved = ErrorMonitor::get_config(&env).expect("config should exist");
        assert!(retrieved.enabled);
        assert_eq!(retrieved.min_level, AlertLevel::Warning);
    }

    #[test]
    fn test_graceful_degradation() {
        let env = Env::default();
        let mut features = Vec::new(&env);
        let feature1 = Symbol::new(&env, "analytics");
        let feature2 = Symbol::new(&env, "reporting");
        features.push_back(feature1.clone());
        features.push_back(feature2.clone());

        GracefulDegradation::initialize(&env, features);

        // Feature should not be degraded initially
        assert!(!GracefulDegradation::is_degraded(&env, &feature1));

        // Degrade feature
        GracefulDegradation::degrade_feature(&env, &feature1).expect("should degrade feature");
        assert!(GracefulDegradation::is_degraded(&env, &feature1));

        // Restore feature
        GracefulDegradation::restore_feature(&env, &feature1).expect("should restore feature");
        assert!(!GracefulDegradation::is_degraded(&env, &feature1));
    }

    #[test]
    fn test_health_check() {
        let env = Env::default();
        let operation_id = Symbol::new(&env, "health_test");
        let config = CircuitBreakerConfig::default_config();

        CircuitBreaker::initialize(&env, &operation_id, &config);

        let admin = Address::generate(&env);
        ErrorMonitor::initialize(&env, &AlertConfig::new(admin));

        let mut features = Vec::new(&env);
        features.push_back(Symbol::new(&env, "feature1"));
        GracefulDegradation::initialize(&env, features);

        let mut circuit_ids = Vec::new(&env);
        circuit_ids.push_back(operation_id);

        let result = HealthCheck::check(&env, &circuit_ids);
        assert_eq!(result.status, HealthStatus::Healthy);
        assert_eq!(result.open_circuits, 0);
        assert_eq!(result.degraded_features, 0);
    }
}
