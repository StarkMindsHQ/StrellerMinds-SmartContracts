use soroban_sdk::{Env, Address, String};
use crate::standardized_errors::{StandardError, ErrorContext, ErrorSeverity};

/// Enhanced error handler for comprehensive error management
pub struct ErrorHandler;

impl ErrorHandler {
    /// Creates a new error context with full debugging information
    pub fn create_error_context(
        env: &Env,
        error: StandardError,
        operation: &str,
        contract_name: &str,
        user_address: Option<&Address>,
        additional_info: &str,
    ) -> ErrorContext {
        let timestamp = env.ledger().timestamp();
        
        let mut context = ErrorContext::new(error, operation, contract_name, additional_info)
            .with_timestamp(timestamp);
            
        if let Some(address) = user_address {
            context = context.with_user_address(&address.to_string());
        }
        
        context
    }
    
    /// Logs an error with appropriate severity level
    pub fn log_error(env: &Env, context: &ErrorContext) {
        let severity = match StandardError::from(context.error_code).severity() {
            ErrorSeverity::Critical => "CRITICAL",
            ErrorSeverity::High => "HIGH",
            ErrorSeverity::Medium => "MEDIUM",
            ErrorSeverity::Low => "LOW",
        };
        
        // In a real implementation, this would emit events or write to logs
        // For now, we'll create a structured error message
        let error_message = format!(
            "ERROR [{}] Code: {} Contract: {} Operation: {} Message: {} User: {:?} Info: {} Timestamp: {}",
            severity,
            context.error_code,
            context.contract_name,
            context.operation,
            context.error_message,
            context.user_address,
            context.additional_info,
            context.timestamp
        );
        
        // This would be emitted as an event in a real implementation
        env.log_str(&error_message);
    }
    
    /// Validates input and returns appropriate error if invalid
    pub fn validate_address(env: &Env, address: &Address, operation: &str) -> Result<(), StandardError> {
        if address.is_null() {
            let context = Self::create_error_context(
                env,
                StandardError::InvalidAddress,
                operation,
                "ErrorHandler",
                Some(address),
                "Address cannot be null"
            );
            Self::log_error(env, &context);
            return Err(StandardError::InvalidAddress);
        }
        Ok(())
    }
    
    /// Validates string input and returns appropriate error if invalid
    pub fn validate_string(
        env: &Env,
        input: &String,
        operation: &str,
        field_name: &str,
        min_len: u32,
        max_len: u32,
    ) -> Result<(), StandardError> {
        let len = input.len();
        
        if len == 0 {
            let context = Self::create_error_context(
                env,
                StandardError::MissingRequiredField,
                operation,
                "ErrorHandler",
                None,
                &format!("Field '{}' cannot be empty", field_name)
            );
            Self::log_error(env, &context);
            return Err(StandardError::MissingRequiredField);
        }
        
        if len < min_len as usize {
            let context = Self::create_error_context(
                env,
                StandardError::InputTooShort,
                operation,
                "ErrorHandler",
                None,
                &format!("Field '{}' must be at least {} characters", field_name, min_len)
            );
            Self::log_error(env, &context);
            return Err(StandardError::InputTooShort);
        }
        
        if len > max_len as usize {
            let context = Self::create_error_context(
                env,
                StandardError::InputTooLong,
                operation,
                "ErrorHandler",
                None,
                &format!("Field '{}' cannot exceed {} characters", field_name, max_len)
            );
            Self::log_error(env, &context);
            return Err(StandardError::InputTooLong);
        }
        
        Ok(())
    }
    
    /// Validates numeric range and returns appropriate error if invalid
    pub fn validate_range<T: PartialOrd>(
        env: &Env,
        value: T,
        min: T,
        max: T,
        operation: &str,
        field_name: &str,
    ) -> Result<(), StandardError> {
        if value < min {
            let context = Self::create_error_context(
                env,
                StandardError::OutOfBounds,
                operation,
                "ErrorHandler",
                None,
                &format!("Field '{}' value is below minimum", field_name)
            );
            Self::log_error(env, &context);
            return Err(StandardError::OutOfBounds);
        }
        
        if value > max {
            let context = Self::create_error_context(
                env,
                StandardError::OutOfBounds,
                operation,
                "ErrorHandler",
                None,
                &format!("Field '{}' value exceeds maximum", field_name)
            );
            Self::log_error(env, &context);
            return Err(StandardError::OutOfBounds);
        }
        
        Ok(())
    }
    
    /// Checks authorization and returns appropriate error if unauthorized
    pub fn check_authorization(
        env: &Env,
        caller: &Address,
        admin: &Address,
        operation: &str,
    ) -> Result<(), StandardError> {
        if caller != admin {
            let context = Self::create_error_context(
                env,
                StandardError::Unauthorized,
                operation,
                "ErrorHandler",
                Some(caller),
                "Caller is not authorized to perform this operation"
            );
            Self::log_error(env, &context);
            return Err(StandardError::Unauthorized);
        }
        Ok(())
    }
    
    /// Checks if a resource exists and returns appropriate error if not found
    pub fn check_resource_exists<T>(
        env: &Env,
        resource: Option<&T>,
        resource_type: &str,
        operation: &str,
    ) -> Result<&T, StandardError> {
        match resource {
            Some(resource) => Ok(resource),
            None => {
                let context = Self::create_error_context(
                    env,
                    StandardError::NotFound,
                    operation,
                    "ErrorHandler",
                    None,
                    &format!("{} not found", resource_type)
                );
                Self::log_error(env, &context);
                Err(StandardError::NotFound)
            }
        }
    }
    
    /// Handles batch operation errors with detailed context
    pub fn handle_batch_error(
        env: &Env,
        operation: &str,
        batch_size: u32,
        max_size: u32,
        error_type: BatchErrorType,
    ) -> StandardError {
        let error = match error_type {
            BatchErrorType::TooLarge => StandardError::BatchTooLarge,
            BatchErrorType::Empty => StandardError::BatchEmpty,
            BatchErrorType::PartialFailure => StandardError::PartialFailure,
        };
        
        let context = Self::create_error_context(
            env,
            error,
            operation,
            "ErrorHandler",
            None,
            &format!("Batch operation failed. Size: {}, Max: {}, Type: {:?}", batch_size, max_size, error_type)
        );
        Self::log_error(env, &context);
        
        error
    }
    
    /// Creates a user-friendly error response
    pub fn create_user_response(env: &Env, error: StandardError, context: &ErrorContext) -> String {
        let base_message = error.description();
        let suggested_action = error.suggested_action();
        
        let response = format!(
            "Error {}: {}. Suggested action: {}",
            context.error_code,
            base_message,
            suggested_action
        );
        
        String::from_str(env, &response)
    }
}

/// Types of batch operation errors
#[derive(Debug, Clone, PartialEq)]
pub enum BatchErrorType {
    TooLarge,
    Empty,
    PartialFailure,
}

/// Error metrics for monitoring and analytics
#[derive(Debug, Clone)]
pub struct ErrorMetrics {
    pub total_errors: u64,
    pub errors_by_type: std::collections::HashMap<u32, u64>,
    pub errors_by_severity: std::collections::HashMap<ErrorSeverity, u64>,
    pub errors_by_contract: std::collections::HashMap<String, u64>,
    pub last_error_timestamp: u64,
}

impl ErrorMetrics {
    pub fn new() -> Self {
        Self {
            total_errors: 0,
            errors_by_type: std::collections::HashMap::new(),
            errors_by_severity: std::collections::HashMap::new(),
            errors_by_contract: std::collections::HashMap::new(),
            last_error_timestamp: 0,
        }
    }
    
    pub fn record_error(&mut self, context: &ErrorContext) {
        self.total_errors += 1;
        self.last_error_timestamp = context.timestamp;
        
        // Record by error type
        *self.errors_by_type.entry(context.error_code).or_insert(0) += 1;
        
        // Record by severity
        let severity = StandardError::from(context.error_code).severity();
        *self.errors_by_severity.entry(severity).or_insert(0) += 1;
        
        // Record by contract
        *self.errors_by_contract.entry(context.contract_name.clone()).or_insert(0) += 1;
    }
    
    pub fn get_error_rate(&self, time_window: u64, current_timestamp: u64) -> f64 {
        if self.last_error_timestamp == 0 || time_window == 0 {
            return 0.0;
        }
        
        let time_diff = current_timestamp.saturating_sub(self.last_error_timestamp);
        if time_diff > time_window {
            return 0.0;
        }
        
        self.total_errors as f64 / time_window as f64
    }
}

/// Global error metrics instance (would be stored in contract state in real implementation)
pub static mut ERROR_METRICS: ErrorMetrics = ErrorMetrics::new();

/// Macro for easy error handling with context
#[macro_export]
macro_rules! handle_error {
    ($env:expr, $error:expr, $operation:expr, $contract:expr, $info:expr) => {{
        let context = $crate::error_handler::ErrorHandler::create_error_context(
            $env,
            $error,
            $operation,
            $contract,
            None,
            $info,
        );
        $crate::error_handler::ErrorHandler::log_error($env, &context);
        Err($error)
    }};
    
    ($env:expr, $error:expr, $operation:expr, $contract:expr, $info:expr, $user:expr) => {{
        let context = $crate::error_handler::ErrorHandler::create_error_context(
            $env,
            $error,
            $operation,
            $contract,
            Some($user),
            $info,
        );
        $crate::error_handler::ErrorHandler::log_error($env, &context);
        Err($error)
    }};
}

/// Macro for validation with error handling
#[macro_export]
macro_rules! validate_or_error {
    ($env:expr, $condition:expr, $error:expr, $operation:expr, $info:expr) => {
        if !$condition {
            handle_error!($env, $error, $operation, "Validation", $info);
        }
    };
}
