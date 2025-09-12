use soroban_sdk::{contracttype, Address, BytesN, String, Vec, Map};

/// Mobile-optimized transaction batch for efficient operations
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionBatch {
    pub batch_id: String,
    pub user: Address,
    pub operations: Vec<BatchOperation>,
    pub estimated_gas: u64,
    pub priority: BatchPriority,
    pub created_at: u64,
    pub expires_at: u64,
    pub status: BatchStatus,
    pub execution_strategy: ExecutionStrategy,
    pub retry_config: RetryConfig,
}

/// Individual operation within a transaction batch
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchOperation {
    pub operation_id: String,
    pub operation_type: OperationType,
    pub contract_address: Address,
    pub function_name: String,
    pub parameters: Vec<OperationParameter>,
    pub estimated_gas: u64,
    pub dependencies: Vec<String>, // IDs of operations this depends on
    pub optional: bool,            // Can fail without failing entire batch
    pub retry_count: u32,
    pub status: OperationStatus,
}

/// Operation parameter for flexible function calls
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationParameter {
    pub param_name: String,
    pub param_value: ParameterValue,
    pub param_type: ParameterType,
}

/// Parameter value types for mobile optimization
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParameterValue {
    Address(Address),
    String(String),
    U32(u32),
    U64(u64),
    I64(i64),
    Bool(bool),
    Bytes(BytesN<32>),
    Vector(Vec<String>),
    Map(Map<String, String>),
}

/// Parameter type classification
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParameterType {
    Address,
    String,
    U32,
    U64,
    I64,
    Bool,
    Bytes,
    Vector,
    Map,
}

/// Types of operations supported in batches
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationType {
    CourseEnrollment,
    ProgressUpdate,
    CertificateRequest,
    CertificateRenewal,
    SearchQuery,
    PreferenceUpdate,
    TokenTransfer,
    TokenStaking,
    TokenBurning,
    Custom(String),
}

/// Batch execution priority for mobile optimization
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BatchPriority {
    Critical,    // Execute immediately (user-facing actions)
    High,        // Execute within 1 block
    Normal,      // Execute within 5 blocks
    Low,         // Execute when convenient
    Background,  // Execute during low network activity
}

/// Batch execution status
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BatchStatus {
    Pending,     // Waiting for execution
    Executing,   // Currently being processed
    Completed,   // All operations successful
    PartialSuccess, // Some operations failed
    Failed,      // Batch execution failed
    Cancelled,   // User cancelled
    Expired,     // Batch expired before execution
}

/// Individual operation status
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationStatus {
    Pending,
    Executing,
    Completed,
    Failed,
    Skipped,     // Skipped due to dependency failure
    Retrying,    // Being retried
}

/// Execution strategy for mobile optimization
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionStrategy {
    Sequential,   // Execute operations in order
    Parallel,     // Execute independent operations in parallel
    Optimized,    // Smart execution based on dependencies and gas
    Conservative, // Safest execution for mobile networks
}

/// Retry configuration for mobile network reliability
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub retry_delay_ms: u32,
    pub backoff_multiplier: u32,  // Exponential backoff multiplier
    pub max_delay_ms: u32,
    pub retry_on_network_error: bool,
    pub retry_on_gas_error: bool,
    pub retry_on_timeout: bool,
}

/// Mobile session management for persistent interactions
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileSession {
    pub session_id: String,
    pub user: Address,
    pub device_id: String,
    pub created_at: u64,
    pub last_activity: u64,
    pub expires_at: u64,
    pub network_quality: NetworkQuality,
    pub cached_data: Map<String, String>,
    pub pending_operations: Vec<String>, // Batch IDs
    pub preferences: MobilePreferences,
    pub session_state: SessionState,
}

/// Network quality assessment for mobile optimization
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NetworkQuality {
    Excellent,   // Fast, stable connection
    Good,        // Reliable connection with occasional delays
    Fair,        // Unstable connection, frequent retries needed
    Poor,        // Very slow or unreliable connection
    Offline,     // No connection available
}

/// Mobile-specific user preferences
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobilePreferences {
    pub auto_batch_operations: bool,
    pub max_batch_size: u32,
    pub prefer_low_gas: bool,
    pub enable_offline_mode: bool,
    pub auto_retry_failed: bool,
    pub notification_preferences: NotificationPreferences,
    pub data_usage_mode: DataUsageMode,
    pub battery_optimization: bool,
}

/// Notification preferences for mobile users
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationPreferences {
    pub transaction_complete: bool,
    pub transaction_failed: bool,
    pub batch_ready: bool,
    pub network_issues: bool,
    pub gas_price_alerts: bool,
    pub offline_sync_complete: bool,
}

/// Data usage optimization modes
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataUsageMode {
    Unlimited,   // No data restrictions
    Conservative, // Minimize data usage
    WifiOnly,    // Only sync on WiFi
    Emergency,   // Only critical operations
}

/// Session state for mobile applications
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionState {
    Active,
    Idle,
    Background,
    Suspended,
    Expired,
}

/// Gas estimation result for mobile optimization
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GasEstimate {
    pub operation_id: String,
    pub estimated_gas: u64,
    pub confidence_level: ConfidenceLevel,
    pub factors: Vec<GasFactor>,
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    pub estimated_cost_stroops: i64,
    pub estimated_time_ms: u32,
}

/// Confidence level for gas estimates
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfidenceLevel {
    High,     // 95%+ accuracy
    Medium,   // 80-95% accuracy
    Low,      // 60-80% accuracy
    Unknown,  // Unable to estimate accurately
}

/// Factors affecting gas consumption
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GasFactor {
    NetworkCongestion,
    OperationComplexity,
    DataSize,
    StorageOperations,
    ComputationalLoad,
    ContractInteractions,
}

/// Optimization suggestions for mobile users
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptimizationSuggestion {
    pub suggestion_type: SuggestionType,
    pub description: String,
    pub potential_savings: u64, // Gas savings
    pub implementation_effort: EffortLevel,
    pub applicable: bool,
}

/// Types of optimization suggestions
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SuggestionType {
    BatchOperations,
    ReduceDataSize,
    OptimizeParameters,
    UseCache,
    DelayExecution,
    SimplifyOperation,
}

/// Implementation effort required
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffortLevel {
    None,     // Automatic optimization
    Low,      // Simple parameter change
    Medium,   // Moderate code changes
    High,     // Significant restructuring
}

/// Offline operation queue for mobile devices
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OfflineQueue {
    pub user: Address,
    pub device_id: String,
    pub queued_operations: Vec<QueuedOperation>,
    pub total_estimated_gas: u64,
    pub created_at: u64,
    pub last_sync_attempt: u64,
    pub sync_status: SyncStatus,
    pub conflict_resolution: ConflictResolution,
}

/// Individual queued operation for offline execution
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueuedOperation {
    pub operation_id: String,
    pub operation_type: OperationType,
    pub parameters: Vec<OperationParameter>,
    pub created_at: u64,
    pub priority: BatchPriority,
    pub local_state_hash: BytesN<32>, // For conflict detection
    pub retry_count: u32,
    pub status: QueuedOperationStatus,
}

/// Status of queued operations
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueuedOperationStatus {
    Queued,
    Syncing,
    Synced,
    Conflict,
    Failed,
    Cancelled,
}

/// Synchronization status for offline operations
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyncStatus {
    InSync,
    PendingSync,
    Syncing,
    Conflicts,
    SyncFailed,
    Offline,
}

/// Conflict resolution strategies
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConflictResolution {
    ServerWins,    // Server state takes precedence
    ClientWins,    // Client state takes precedence
    MergeChanges,  // Attempt to merge changes
    UserDecision,  // Prompt user to resolve
    Abort,         // Cancel conflicting operations
}

/// Mobile analytics for optimization insights
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileAnalytics {
    pub user: Address,
    pub device_id: String,
    pub session_count: u32,
    pub total_operations: u32,
    pub successful_operations: u32,
    pub failed_operations: u32,
    pub average_gas_used: u64,
    pub network_quality_distribution: Map<String, u32>, // NetworkQuality -> count
    pub common_operation_types: Vec<OperationTypeStats>,
    pub optimization_impact: OptimizationImpact,
    pub period_start: u64,
    pub period_end: u64,
}

/// Statistics for operation types
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationTypeStats {
    pub operation_type: OperationType,
    pub count: u32,
    pub success_rate: u32,        // Percentage
    pub average_gas: u64,
    pub average_duration_ms: u32,
}

/// Impact of mobile optimizations
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptimizationImpact {
    pub gas_savings_percentage: u32,
    pub operation_success_rate_improvement: u32,
    pub average_response_time_improvement_ms: i32,
    pub user_satisfaction_score: u32, // 1-100
    pub battery_usage_reduction_percentage: u32,
    pub data_usage_reduction_percentage: u32,
}

/// Mobile-friendly error information
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileError {
    pub error_code: String,
    pub error_type: MobileErrorType,
    pub user_friendly_message: String,
    pub technical_details: String,
    pub suggested_actions: Vec<String>,
    pub retry_recommended: bool,
    pub contact_support: bool,
    pub timestamp: u64,
}

/// Types of mobile-specific errors
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MobileErrorType {
    NetworkTimeout,
    InsufficientGas,
    TransactionFailed,
    ContractError,
    ValidationError,
    AuthenticationError,
    RateLimitExceeded,
    ServiceUnavailable,
    DataCorruption,
    SyncConflict,
}

/// Storage keys for mobile optimizer contract
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Contract admin
    Admin,
    /// Contract initialization flag
    Initialized,
    /// Transaction batches by batch ID
    TransactionBatch(String),
    /// User's pending batches
    UserBatches(Address),
    /// Mobile sessions by session ID
    MobileSession(String),
    /// User's active sessions
    UserSessions(Address),
    /// Gas estimates cache
    GasEstimates(String), // Operation hash
    /// Offline operation queues by user
    OfflineQueue(Address),
    /// Mobile analytics by user
    MobileAnalytics(Address),
    /// Global mobile preferences
    GlobalMobileConfig,
    /// Network quality metrics
    NetworkMetrics(u64), // Timestamp bucket
    /// Optimization suggestions cache
    OptimizationCache(String),
    /// Error logs for debugging
    ErrorLogs(Address),
    /// Performance metrics
    PerformanceMetrics(u64),
    /// Batch execution history
    BatchHistory(String),
    /// Session cleanup queue
    SessionCleanup(u64),
}
