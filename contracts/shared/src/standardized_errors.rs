use soroban_sdk::contracterror;

/// Standardized error codes for all StrellerMinds contracts
/// Error codes are organized by category with specific ranges to avoid conflicts
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum StandardError {
    // ===== INITIALIZATION ERRORS (1000-1099) =====
    AlreadyInitialized = 1000,
    NotInitialized = 1001,
    InitializationFailed = 1002,
    InvalidInitializationParams = 1003,
    
    // ===== AUTHORIZATION ERRORS (1100-1199) =====
    Unauthorized = 1100,
    PermissionDenied = 1101,
    InvalidSignature = 1102,
    ExpiredSignature = 1103,
    RoleNotFound = 1104,
    RoleAlreadyExists = 1105,
    CannotRevokeOwnRole = 1106,
    CannotTransferOwnRole = 1107,
    InvalidRole = 1108,
    
    // ===== INPUT VALIDATION ERRORS (1200-1299) =====
    InvalidInput = 1200,
    InvalidAddress = 1201,
    InvalidAmount = 1202,
    InvalidString = 1203,
    InvalidArray = 1204,
    InvalidTimestamp = 1205,
    InvalidEnum = 1206,
    MissingRequiredField = 1207,
    InvalidFormat = 1208,
    InputTooLong = 1209,
    InputTooShort = 1210,
    OutOfBounds = 1211,
    
    // ===== RESOURCE NOT FOUND ERRORS (1300-1399) =====
    NotFound = 1300,
    UserNotFound = 1301,
    CertificateNotFound = 1302,
    AssessmentNotFound = 1303,
    CourseNotFound = 1304,
    TemplateNotFound = 1305,
    ConfigNotFound = 1306,
    SessionNotFound = 1307,
    ReportNotFound = 1308,
    DataNotFound = 1309,
    
    // ===== BUSINESS LOGIC ERRORS (1400-1499) =====
    AlreadyExists = 1400,
    InvalidStatus = 1401,
    OperationNotAllowed = 1402,
    LimitExceeded = 1403,
    QuotaExceeded = 1404,
    RateLimitExceeded = 1405,
    DuplicateEntry = 1406,
    InvalidTransition = 1407,
    DependencyNotMet = 1408,
    PrerequisiteNotMet = 1409,
    
    // ===== CONFIGURATION ERRORS (1500-1599) =====
    InvalidConfig = 1500,
    InvalidConfiguration = 1501,
    ConfigNotFound = 1502,
    ConfigurationLocked = 1503,
    InvalidThreshold = 1504,
    InvalidTimeWindow = 1505,
    InvalidParameter = 1506,
    
    // ===== STORAGE ERRORS (1600-1699) =====
    StorageError = 1600,
    DataCorruption = 1601,
    InsufficientStorage = 1602,
    StorageLimitReached = 1603,
    DataConflict = 1604,
    
    // ===== NETWORK/EXTERNAL ERRORS (1700-1799) =====
    NetworkError = 1700,
    ExternalServiceUnavailable = 1701,
    TimeoutError = 1702,
    ConnectionFailed = 1703,
    InvalidResponse = 1704,
    
    // ===== SECURITY ERRORS (1800-1899) =====
    SecurityViolation = 1800,
    SuspiciousActivity = 1801,
    BlacklistedAddress = 1802,
    MaliciousRequest = 1803,
    AuthenticationFailed = 1804,
    
    // ===== BATCH OPERATION ERRORS (1900-1999) =====
    BatchTooLarge = 1900,
    BatchEmpty = 1901,
    PartialFailure = 1902,
    BatchOperationFailed = 1903,
    InvalidBatchSize = 1904,
    
    // ===== TEMPORAL ERRORS (2000-2099) =====
    Expired = 2000,
    NotYetActive = 2001,
    InvalidTimeRange = 2002,
    TimeWindowExpired = 2003,
    TooEarly = 2004,
    TooLate = 2005,
    
    // ===== SYSTEM ERRORS (2100-2199) =====
    InternalError = 2100,
    SystemOverloaded = 2101,
    MaintenanceMode = 2102,
    FeatureDisabled = 2103,
    NotImplemented = 2104,
    Deprecated = 2105,
    
    // ===== COMPLIANCE ERRORS (2200-2299) =====
    ComplianceCheckFailed = 2200,
    RegulatoryViolation = 2201,
    AuditFailed = 2202,
    UnsupportedStandard = 2203,
    ComplianceRequired = 2204,
    
    // ===== FINANCIAL ERRORS (2300-2399) =====
    InsufficientBalance = 2300,
    InsufficientFunds = 2301,
    InvalidAmount = 2302,
    TransferFailed = 2303,
    PaymentRequired = 2304,
    TransactionFailed = 2305,
    
    // ===== MISCELLANEOUS ERRORS (2400-2499) =====
    UnknownError = 2400,
    UnexpectedError = 2401,
    OperationCancelled = 2402,
    UserCancelled = 2403,
    FeatureNotAvailable = 2404,
}

/// Enhanced error context for debugging and user experience
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ErrorContext {
    pub error_code: u32,
    pub error_message: String,
    pub operation: String,
    pub contract_name: String,
    pub additional_info: String,
    pub timestamp: u64,
    pub user_address: Option<String>,
}

impl ErrorContext {
    pub fn new(
        error: StandardError,
        operation: &str,
        contract_name: &str,
        additional_info: &str,
    ) -> Self {
        Self {
            error_code: error as u32,
            error_message: format!("{:?}", error),
            operation: operation.to_string(),
            contract_name: contract_name.to_string(),
            additional_info: additional_info.to_string(),
            timestamp: 0, // Will be set when error occurs
            user_address: None,
        }
    }
    
    pub fn with_user_address(mut self, address: &str) -> Self {
        self.user_address = Some(address.to_string());
        self
    }
    
    pub fn with_timestamp(mut self, timestamp: u64) -> Self {
        self.timestamp = timestamp;
        self
    }
    
    pub fn to_user_message(&self) -> String {
        match self.error_code {
            1100..=1199 => format!("Authorization failed: {}", self.error_message),
            1200..=1299 => format!("Invalid input: {}", self.error_message),
            1300..=1399 => format!("Resource not found: {}", self.error_message),
            1400..=1499 => format!("Operation not allowed: {}", self.error_message),
            1500..=1599 => format!("Configuration error: {}", self.error_message),
            1600..=1699 => format!("Storage error: {}", self.error_message),
            1700..=1799 => format!("Network error: {}", self.error_message),
            1800..=1899 => format!("Security error: {}", self.error_message),
            1900..=1999 => format!("Batch operation error: {}", self.error_message),
            2000..=2099 => format!("Timing error: {}", self.error_message),
            2100..=2199 => format!("System error: {}", self.error_message),
            2200..=2299 => format!("Compliance error: {}", self.error_message),
            2300..=2399 => format!("Financial error: {}", self.error_message),
            _ => format!("Error: {}", self.error_message),
        }
    }
}

/// Error severity levels for logging and monitoring
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u8)]
pub enum ErrorSeverity {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

impl StandardError {
    /// Get the severity level for this error
    pub fn severity(&self) -> ErrorSeverity {
        match self {
            // Critical errors
            StandardError::InternalError
            | StandardError::DataCorruption
            | StandardError::SecurityViolation
            | StandardError::SystemOverloaded => ErrorSeverity::Critical,
            
            // High severity errors
            StandardError::Unauthorized
            | StandardError::PermissionDenied
            | StandardError::StorageError
            | StandardError::ComplianceCheckFailed
            | StandardError::InsufficientBalance => ErrorSeverity::High,
            
            // Medium severity errors
            StandardError::InvalidInput
            | StandardError::NotFound
            | StandardError::AlreadyExists
            | StandardError::InvalidConfig
            | StandardError::NetworkError => ErrorSeverity::Medium,
            
            // Low severity errors
            _ => ErrorSeverity::Low,
        }
    }
    
    /// Get user-friendly error description
    pub fn description(&self) -> &'static str {
        match self {
            StandardError::AlreadyInitialized => "Contract has already been initialized",
            StandardError::NotInitialized => "Contract has not been initialized",
            StandardError::Unauthorized => "You are not authorized to perform this action",
            StandardError::PermissionDenied => "Permission denied for this operation",
            StandardError::InvalidInput => "The input provided is invalid",
            StandardError::NotFound => "The requested resource was not found",
            StandardError::AlreadyExists => "The resource already exists",
            StandardError::InvalidConfig => "The configuration is invalid",
            StandardError::StorageError => "A storage error occurred",
            StandardError::InternalError => "An internal error occurred",
            StandardError::InsufficientBalance => "Insufficient balance for this operation",
            StandardError::Expired => "The item has expired",
            StandardError::RateLimitExceeded => "Rate limit has been exceeded",
            StandardError::BatchTooLarge => "The batch size is too large",
            StandardError::SecurityViolation => "A security violation was detected",
            StandardError::ComplianceCheckFailed => "Compliance check failed",
            StandardError::NetworkError => "A network error occurred",
            StandardError::FeatureDisabled => "This feature is currently disabled",
            StandardError::OperationNotAllowed => "This operation is not allowed",
            StandardError::InvalidStatus => "The status is invalid for this operation",
            StandardError::LimitExceeded => "A limit has been exceeded",
            StandardError::MissingRequiredField => "A required field is missing",
            StandardError::InvalidAddress => "The address provided is invalid",
            StandardError::InvalidAmount => "The amount provided is invalid",
            StandardError::InvalidTimestamp => "The timestamp provided is invalid",
            StandardError::DataNotFound => "The requested data was not found",
            StandardError::InvalidParameter => "The parameter provided is invalid",
            StandardError::AuthenticationFailed => "Authentication failed",
            StandardError::TransactionFailed => "The transaction failed",
            StandardError::TimeoutError => "The operation timed out",
            StandardError::MaintenanceMode => "System is in maintenance mode",
            StandardError::NotImplemented => "This feature is not implemented",
            StandardError::Deprecated => "This feature is deprecated",
            StandardError::UnknownError => "An unknown error occurred",
            StandardError::UnexpectedError => "An unexpected error occurred",
            StandardError::OperationCancelled => "The operation was cancelled",
            StandardError::UserCancelled => "The user cancelled the operation",
            StandardError::FeatureNotAvailable => "This feature is not available",
            _ => "An error occurred",
        }
    }
    
    /// Get suggested action for the user
    pub fn suggested_action(&self) -> &'static str {
        match self {
            StandardError::Unauthorized => "Please check your permissions and try again",
            StandardError::InvalidInput => "Please check your input and try again",
            StandardError::NotFound => "Please verify the resource exists and try again",
            StandardError::AlreadyExists => "Please use a different identifier or check existing resources",
            StandardError::InsufficientBalance => "Please ensure you have sufficient balance",
            StandardError::Expired => "Please refresh and try again",
            StandardError::RateLimitExceeded => "Please wait and try again later",
            StandardError::BatchTooLarge => "Please reduce the batch size and try again",
            StandardError::NetworkError => "Please check your connection and try again",
            StandardError::MaintenanceMode => "Please try again later when maintenance is complete",
            StandardError::FeatureDisabled => "This feature is temporarily unavailable",
            StandardError::InvalidConfig => "Please check the configuration parameters",
            StandardError::StorageError => "Please try again or contact support if the issue persists",
            StandardError::InternalError => "Please try again or contact support if the issue persists",
            StandardError::SecurityViolation => "This action has been blocked for security reasons",
            StandardError::ComplianceCheckFailed => "Please ensure you meet all compliance requirements",
            StandardError::OperationNotAllowed => "This operation is not permitted in the current state",
            StandardError::InvalidStatus => "Please check the current status and try again",
            StandardError::LimitExceeded => "Please reduce the amount or try again later",
            StandardError::MissingRequiredField => "Please provide all required fields",
            StandardError::InvalidAddress => "Please provide a valid address",
            StandardError::InvalidAmount => "Please provide a valid amount",
            StandardError::InvalidTimestamp => "Please provide a valid timestamp",
            StandardError::DataNotFound => "Please verify the data exists and try again",
            StandardError::InvalidParameter => "Please check the parameter value",
            StandardError::AuthenticationFailed => "Please authenticate and try again",
            StandardError::TransactionFailed => "Please try again or contact support",
            StandardError::TimeoutError => "Please try again with a shorter timeout",
            StandardError::NotImplemented => "This feature is not yet available",
            StandardError::Deprecated => "Please use the updated method or feature",
            StandardError::UnknownError => "Please try again or contact support",
            StandardError::UnexpectedError => "Please try again or contact support",
            StandardError::OperationCancelled => "The operation was cancelled - please try again if needed",
            StandardError::UserCancelled => "You cancelled the operation - please try again if needed",
            StandardError::FeatureNotAvailable => "This feature is not available in your region or plan",
            _ => "Please try again or contact support if the issue persists",
        }
    }
}

/// Macro for creating standardized errors with context
#[macro_export]
macro_rules! create_error {
    ($error:expr, $operation:expr, $contract:expr, $info:expr) => {
        $crate::standardized_errors::ErrorContext::new(
            $error,
            $operation,
            $contract,
            $info,
        )
    };
}

/// Macro for returning standardized errors
#[macro_export]
macro_rules! return_error {
    ($error:expr) => {
        return Err($error)
    };
    ($error:expr, $context:expr) => {
        return Err($error)
    };
}
