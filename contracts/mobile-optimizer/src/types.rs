#![allow(clippy::enum_variant_names)]

use shared::config::{ContractConfig, DeploymentEnv};
use soroban_sdk::{contracttype, Address, BytesN, Map, String, Vec};

// ============================================================================
// Core Transaction & Batch Types
// ============================================================================

/// A batch of on-chain operations to be executed together for gas efficiency.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransactionBatch {
    /// Unique batch identifier.
    pub batch_id: String,
    /// Address of the user who created the batch.
    pub user: Address,
    /// List of operations in this batch.
    pub operations: Vec<BatchOperation>,
    /// Total estimated gas for all operations.
    pub estimated_gas: u64,
    /// Scheduling priority of the batch.
    pub priority: BatchPriority,
    /// Unix timestamp when the batch was created.
    pub created_at: u64,
    /// Unix timestamp after which the batch should not be executed.
    pub expires_at: u64,
    /// Current execution status.
    pub status: BatchStatus,
    /// Strategy used to execute the operations.
    pub execution_strategy: ExecutionStrategy,
    /// Retry policy applied to failed operations.
    pub retry_config: RetryConfig,
    /// Network quality at the time the batch was created.
    pub network_quality: NetworkQuality,
}

/// A single operation within a transaction batch.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchOperation {
    /// Unique operation identifier.
    pub operation_id: String,
    /// Category of the operation.
    pub operation_type: OperationType,
    /// Target contract address.
    pub contract_address: Address,
    /// Name of the contract function to call.
    pub function_name: String,
    /// Arguments to pass to the function.
    pub parameters: Vec<OperationParameter>,
    /// Estimated gas for this individual operation.
    pub estimated_gas: u64,
    /// Relative priority within the batch.
    pub priority: OperationPriority,
    /// Retry policy for this operation.
    pub retry_config: RetryConfig,
    /// IDs of operations that must complete before this one runs.
    pub dependencies: Vec<String>,
}

/// A named parameter supplied to a batch operation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationParameter {
    /// Parameter name matching the function signature.
    pub param_name: String,
    /// Serialized parameter value.
    pub param_value: ParameterValue,
    /// Type descriptor for the parameter.
    pub param_type: ParameterType,
}

/// Typed value container for operation parameters.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParameterValue {
    /// An `Address` value.
    AddressVal(Address),
    /// A `String` value.
    StringVal(String),
    /// A `u32` value.
    U32Val(u32),
    /// A `u64` value.
    U64Val(u64),
    /// An `i64` value.
    I64Val(i64),
    /// A `bool` value.
    BoolVal(bool),
    /// A 32-byte array value.
    BytesVal(BytesN<32>),
    /// A vector of strings.
    VectorVal(Vec<String>),
    /// A string-to-string map.
    MapVal(Map<String, String>),
}

/// Type descriptor for a parameter value.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParameterType {
    /// Stellar address.
    Address,
    /// UTF-8 string.
    String,
    /// 32-bit unsigned integer.
    U32,
    /// 64-bit unsigned integer.
    U64,
    /// 64-bit signed integer.
    I64,
    /// Boolean.
    Bool,
    /// Raw bytes.
    Bytes,
    /// Ordered list.
    Vector,
    /// Key-value map.
    Map,
}

/// High-level category of a batch operation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationType {
    /// Enrolling a student in a course.
    CourseEnrollment,
    /// Updating learning progress.
    ProgressUpdate,
    /// Requesting a new certificate.
    CertificateRequest,
    /// Renewing an existing certificate.
    CertificateRenewal,
    /// Generating a certificate on-chain.
    CertificateGeneration,
    /// Executing a search query.
    SearchQuery,
    /// Updating user preferences.
    PreferenceUpdate,
    /// Transferring tokens.
    TokenTransfer,
    /// Staking tokens.
    TokenStaking,
    /// Burning tokens.
    TokenBurning,
    /// Distributing a token reward.
    TokenReward,
    /// Caching content for offline access.
    ContentCache,
    /// Syncing learning state.
    LearningSync,
    /// Configuring notifications.
    NotificationConfig,
    /// Applying a security update.
    SecurityUpdate,
    /// Recording an analytics event.
    AnalyticsEvent,
    /// Any other operation type.
    Custom,
}

/// Relative priority of a single operation within a batch.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationPriority {
    /// Must execute immediately regardless of conditions.
    Critical,
    /// Execute before normal operations.
    High,
    /// Standard execution order.
    Medium,
    /// Execute last, when resources allow.
    Low,
}

/// Scheduling priority of an entire transaction batch.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BatchPriority {
    /// Execute immediately, ahead of all other batches.
    Critical,
    /// Execute before normal batches.
    High,
    /// Standard scheduling.
    Normal,
    /// Execute when the queue is idle.
    Low,
    /// Defer to low-activity periods.
    Background,
}

/// Execution status of a transaction batch.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BatchStatus {
    /// Queued and awaiting execution.
    Pending,
    /// Currently being processed.
    Executing,
    /// All operations finished successfully.
    Completed,
    /// Some operations succeeded and some failed.
    PartialSuccess,
    /// All operations failed.
    Failed,
    /// Batch was cancelled before completion.
    Cancelled,
    /// Batch was not executed before its expiry time.
    Expired,
}

/// Execution status of a single operation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationStatus {
    /// Queued and not yet started.
    Pending,
    /// Currently being executed.
    Executing,
    /// Finished successfully.
    Completed,
    /// Encountered a non-recoverable error.
    Failed,
    /// Skipped due to a failed dependency.
    Skipped,
    /// Attempting again after a previous failure.
    Retrying,
}

/// Strategy used to execute the operations in a batch.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionStrategy {
    /// Execute operations one after another in order.
    Sequential,
    /// Execute independent operations concurrently.
    Parallel,
    /// Automatically choose the most efficient order.
    Optimized,
    /// Favour safety over speed; validate between steps.
    Conservative,
}

/// Retry policy for failed operations.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryConfig {
    /// Maximum number of retry attempts.
    pub max_retries: u32,
    /// Initial delay between retries in milliseconds.
    pub retry_delay_ms: u32,
    /// Multiplier applied to delay on each subsequent retry.
    pub backoff_multiplier: u32,
    /// Maximum delay cap in milliseconds.
    pub max_delay_ms: u32,
    /// Whether to retry on network-related errors.
    pub retry_on_network_error: bool,
    /// Whether to retry when the operation runs out of gas.
    pub retry_on_gas_error: bool,
    /// Whether to retry on timeout errors.
    pub retry_on_timeout: bool,
}

// ============================================================================
// Session Types
// ============================================================================

/// An active mobile client session for a user on a specific device.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileSession {
    /// Unique session identifier.
    pub session_id: String,
    /// Address of the authenticated user.
    pub user: Address,
    /// Identifier of the device running the session.
    pub device_id: String,
    /// Unix timestamp when the session was created.
    pub created_at: u64,
    /// Unix timestamp of the most recent user activity.
    pub last_activity: u64,
    /// Unix timestamp after which the session is invalid.
    pub expires_at: u64,
    /// Network quality observed at the last activity checkpoint.
    pub network_quality: NetworkQuality,
    /// Key-value map of locally cached data for this session.
    pub cached_data: Map<String, String>,
    /// IDs of operations queued but not yet submitted.
    pub pending_operations: Vec<String>,
    /// User-specific mobile preferences.
    pub preferences: MobilePreferences,
    /// Current lifecycle state of the session.
    pub session_state: SessionState,
}

/// Lifecycle state of a mobile session.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionState {
    /// Session is in active use.
    Active,
    /// Session is open but the user has been inactive briefly.
    Idle,
    /// App is running in the background.
    Background,
    /// Session is suspended and will resume on next foreground.
    Suspended,
    /// Session has passed its expiry time.
    Expired,
}

// ============================================================================
// Network Types
// ============================================================================

/// Observed quality of the mobile device's network connection.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NetworkQuality {
    /// Very fast and reliable connection.
    Excellent,
    /// Fast connection with minimal packet loss.
    Good,
    /// Acceptable connection with occasional delays.
    Fair,
    /// Slow or highly unreliable connection.
    Poor,
    /// No network connectivity.
    Offline,
}

// ============================================================================
// Preferences & Configuration
// ============================================================================

/// User-defined preferences controlling mobile optimizer behaviour.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobilePreferences {
    /// Whether operations should be automatically batched.
    pub auto_batch_operations: bool,
    /// Maximum number of operations per batch.
    pub max_batch_size: u32,
    /// Whether to favour lower-gas execution paths.
    pub prefer_low_gas: bool,
    /// Whether offline operation queueing is enabled.
    pub enable_offline_mode: bool,
    /// Whether failed operations are automatically retried.
    pub auto_retry_failed: bool,
    /// Per-channel notification opt-ins.
    pub notification_preferences: NotificationPreferences,
    /// Data-usage policy for network calls.
    pub data_usage_mode: DataUsageMode,
    /// Whether battery-saving optimizations are active.
    pub battery_optimization: bool,
}

/// Fine-grained notification opt-in settings for a user.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationPreferences {
    /// Notify when a transaction completes successfully.
    pub transaction_complete: bool,
    /// Notify when a transaction fails.
    pub transaction_failed: bool,
    /// Notify when a batch is ready for submission.
    pub batch_ready: bool,
    /// Notify when network issues are detected.
    pub network_issues: bool,
    /// Notify when gas prices change significantly.
    pub gas_price_alerts: bool,
    /// Notify when an offline sync finishes.
    pub offline_sync_complete: bool,
    /// Notify with study reminders.
    pub learning_reminders: bool,
    /// Notify to maintain learning streaks.
    pub streak_alerts: bool,
    /// Notify about course updates.
    pub course_updates: bool,
}

/// Controls how much mobile data the app is permitted to consume.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataUsageMode {
    /// No restrictions; use as much data as needed.
    Unlimited,
    /// Minimise background data transfers.
    Conservative,
    /// Only perform network calls on Wi-Fi.
    WifiOnly,
    /// Absolute minimum data use; critical operations only.
    Emergency,
}

/// Contract-level configuration for the mobile optimizer.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileOptimizerConfig {
    /// Administrator address with contract governance rights.
    pub admin: Address,
    /// Maximum allowed operations in a single batch.
    pub max_batch_size: u32,
    /// Default gas limit applied to operations without an explicit limit.
    pub default_gas_limit: u64,
    /// Seconds of inactivity before a session expires.
    pub session_timeout_seconds: u64,
    /// Maximum number of operations in the offline queue.
    pub offline_queue_limit: u32,
    /// Milliseconds before a network call times out.
    pub network_timeout_ms: u32,
    /// Default number of retry attempts for failed operations.
    pub retry_attempts: u32,
    /// Default time-to-live for cache entries, in seconds.
    pub cache_ttl_seconds: u64,
    /// Maximum number of registered devices per user.
    pub max_devices_per_user: u32,
    /// Number of days analytics data is retained.
    pub analytics_retention_days: u32,
}

impl MobileOptimizerConfig {
    pub fn for_env(admin: Address, profile: DeploymentEnv) -> Self {
        let defaults = ContractConfig::mobile(profile);
        Self {
            admin,
            max_batch_size: defaults.max_batch_size,
            default_gas_limit: defaults.default_gas_limit,
            session_timeout_seconds: defaults.session_timeout_seconds,
            offline_queue_limit: defaults.offline_queue_limit,
            network_timeout_ms: defaults.network_timeout_ms,
            retry_attempts: defaults.retry_attempts,
            cache_ttl_seconds: defaults.cache_ttl_seconds,
            max_devices_per_user: defaults.max_devices_per_user,
            analytics_retention_days: defaults.analytics_retention_days,
        }
    }

    pub fn validate(&self) -> Result<(), MobileOptimizerError> {
        if self.max_batch_size == 0
            || self.session_timeout_seconds == 0
            || self.offline_queue_limit == 0
            || self.max_devices_per_user == 0
        {
            return Err(MobileOptimizerError::InvalidInput);
        }
        Ok(())
    }
}

// ============================================================================
// Gas Estimation Types
// ============================================================================

/// Gas cost estimate for a single operation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GasEstimate {
    /// Identifier of the operation being estimated.
    pub operation_id: String,
    /// Estimated gas units required.
    pub estimated_gas: u64,
    /// Confidence rating for the estimate.
    pub confidence_level: ConfidenceLevel,
    /// Factors that influenced the estimate.
    pub factors: Vec<GasFactor>,
    /// Actionable suggestions to reduce gas cost.
    pub optimization_suggestions: Vec<OptimizationSuggestion>,
    /// Estimated cost in stroops (1 XLM = 10 000 000 stroops).
    pub estimated_cost_stroops: i64,
    /// Estimated wall-clock execution time in milliseconds.
    pub estimated_time_ms: u32,
}

/// Confidence level of a gas or performance estimate.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfidenceLevel {
    /// Estimate is based on reliable historical data.
    High,
    /// Estimate has moderate uncertainty.
    Medium,
    /// Estimate has high uncertainty; use with caution.
    Low,
    /// Not enough data to produce a meaningful estimate.
    Unknown,
}

/// A contributing factor to the gas cost of an operation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GasFactor {
    /// Network is congested, increasing fees.
    NetworkCongestion,
    /// The operation itself has complex logic.
    OperationComplexity,
    /// Large payloads increase ledger entry costs.
    DataSize,
    /// Number of storage read/write operations.
    StorageOperations,
    /// High compute requirements (loops, crypto, etc.).
    ComputationalLoad,
    /// Cross-contract invocations add overhead.
    ContractInteractions,
}

/// An actionable suggestion to reduce gas or improve performance.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptimizationSuggestion {
    /// Category of the suggestion.
    pub suggestion_type: SuggestionType,
    /// Human-readable explanation of the suggestion.
    pub description: String,
    /// Estimated gas units that could be saved.
    pub potential_savings: u64,
    /// How much work is required to implement the suggestion.
    pub implementation_effort: EffortLevel,
    /// Whether the suggestion applies to the current context.
    pub applicable: bool,
}

/// Category of an optimization suggestion.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SuggestionType {
    /// Group multiple operations into a single batch.
    BatchOperations,
    /// Trim payload sizes to reduce ledger costs.
    ReduceDataSize,
    /// Adjust parameters to use cheaper code paths.
    OptimizeParameters,
    /// Serve data from cache instead of on-chain reads.
    UseCache,
    /// Postpone execution to a lower-congestion period.
    DelayExecution,
    /// Simplify the operation logic to reduce compute.
    SimplifyOperation,
}

/// Relative implementation effort for an optimization.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffortLevel {
    /// No changes required; already optimal.
    None,
    /// Minimal change with quick turnaround.
    Low,
    /// Moderate refactor needed.
    Medium,
    /// Significant engineering effort required.
    High,
}

// ============================================================================
// Offline Queue Types
// ============================================================================

/// Operations queued on a device while it was offline, pending sync.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OfflineQueue {
    /// Address of the user who owns the queue.
    pub user: Address,
    /// Identifier of the device that created the queue.
    pub device_id: String,
    /// List of operations waiting to be synced.
    pub queued_operations: Vec<QueuedOperation>,
    /// Cumulative estimated gas for all queued operations.
    pub total_estimated_gas: u64,
    /// Unix timestamp when the queue was first created.
    pub created_at: u64,
    /// Unix timestamp of the most recent sync attempt.
    pub last_sync_attempt: u64,
    /// Current synchronization status.
    pub sync_status: SyncStatus,
    /// Policy used to resolve conflicts when syncing.
    pub conflict_resolution: ConflictResolution,
}

/// A single operation stored in the offline queue.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueuedOperation {
    /// Unique operation identifier.
    pub operation_id: String,
    /// Category of the operation.
    pub operation_type: OperationType,
    /// Arguments to use when the operation is submitted.
    pub parameters: Vec<OperationParameter>,
    /// Unix timestamp when the operation was queued.
    pub created_at: u64,
    /// Scheduling priority during sync.
    pub priority: BatchPriority,
    /// Hash of local state at the time the operation was queued.
    pub local_state_hash: BytesN<32>,
    /// Number of sync attempts made so far.
    pub retry_count: u32,
    /// Current status of the queued operation.
    pub status: QueuedOperationStatus,
    /// Estimated gas for this operation.
    pub estimated_gas: u64,
}

/// Lifecycle status of a queued offline operation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum QueuedOperationStatus {
    /// Waiting for connectivity to attempt submission.
    Queued,
    /// Submission in progress.
    Syncing,
    /// Successfully submitted and confirmed on-chain.
    Synced,
    /// Conflict detected with server-side state.
    Conflict,
    /// Submission failed after exhausting retries.
    Failed,
    /// Operation was discarded before submission.
    Cancelled,
}

/// Overall sync status of an offline queue.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyncStatus {
    /// Local and remote state are consistent.
    InSync,
    /// Operations are queued and waiting to sync.
    PendingSync,
    /// Sync is currently in progress.
    Syncing,
    /// Conflicts were found that must be resolved.
    Conflicts,
    /// The last sync attempt failed.
    SyncFailed,
    /// Device has no connectivity; sync deferred.
    Offline,
}

/// Strategy used to resolve conflicts during an offline sync.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConflictResolution {
    /// Remote (on-chain) state takes precedence.
    ServerWins,
    /// Local (offline) state takes precedence.
    ClientWins,
    /// Attempt to merge both sets of changes.
    MergeChanges,
    /// Prompt the user to choose.
    UserDecision,
    /// Cancel the sync and leave both states unchanged.
    Abort,
}

// ============================================================================
// Content Cache Types (NEW)
// ============================================================================

/// A single entry stored in the local content cache.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheEntry {
    /// Unique cache lookup key.
    pub cache_key: String,
    /// Hash of the cached content for integrity verification.
    pub content_hash: BytesN<32>,
    /// Type of the cached content.
    pub content_type: ContentType,
    /// Size of the cached content in bytes.
    pub size_bytes: u64,
    /// Unix timestamp when the entry was cached.
    pub created_at: u64,
    /// Unix timestamp after which the entry is stale.
    pub expires_at: u64,
    /// Number of times the entry has been accessed.
    pub access_count: u32,
    /// Unix timestamp of the most recent access.
    pub last_accessed: u64,
    /// Priority that influences eviction ordering.
    pub priority: CachePriority,
}

/// Type of content stored in the local cache.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentType {
    /// Course text and media assets.
    CourseMaterial,
    /// Video lesson content.
    VideoLesson,
    /// Quiz questions and answers.
    QuizData,
    /// Issued certificate data.
    Certificate,
    /// User profile information.
    UserProfile,
    /// Cached search result pages.
    SearchResults,
    /// Learning progress state.
    ProgressData,
    /// Notification payloads.
    NotificationData,
}

/// Cache eviction priority for an entry.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CachePriority {
    /// Must not be evicted; required for offline functionality.
    Essential,
    /// Evict only when cache is critically full.
    High,
    /// Standard eviction ordering.
    Normal,
    /// Evict before normal-priority entries.
    Low,
    /// Evict first whenever space is needed.
    Evictable,
}

/// Configuration for the local content cache.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheConfig {
    /// Maximum total cache size in bytes.
    pub max_cache_size_bytes: u64,
    /// Default TTL applied to new entries, in seconds.
    pub default_ttl_seconds: u64,
    /// Algorithm used to choose which entries to evict.
    pub eviction_policy: EvictionPolicy,
    /// Whether proactive prefetching of upcoming content is enabled.
    pub prefetch_enabled: bool,
    /// Whether cached content is compressed to save storage.
    pub compression_enabled: bool,
}

/// Algorithm used to select cache entries for eviction.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvictionPolicy {
    /// Remove the entry that was accessed least recently.
    LeastRecentlyUsed,
    /// Remove the entry that has been accessed fewest times.
    LeastFrequentlyUsed,
    /// Remove entries whose TTL has elapsed.
    TimeToLive,
    /// Remove lowest-priority entries first.
    PriorityBased,
}

/// A rule that triggers automatic content prefetching.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrefetchRule {
    /// Unique rule identifier.
    pub rule_id: String,
    /// Type of content to prefetch.
    pub content_type: ContentType,
    /// Event that activates this rule.
    pub trigger: PrefetchTrigger,
    /// Minimum network quality required before prefetching begins.
    pub network_requirement: NetworkQuality,
    /// Maximum total bytes to prefetch per trigger.
    pub max_prefetch_size_bytes: u64,
}

/// Event that triggers a prefetch rule.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrefetchTrigger {
    /// Triggered when a user enrols in a course.
    OnCourseEnroll,
    /// Triggered when a module is completed.
    OnModuleComplete,
    /// Triggered when a Wi-Fi connection is established.
    OnWifiConnect,
    /// Triggered on a scheduled interval.
    OnSchedule,
    /// Triggered during periods of low app activity.
    OnLowActivity,
}

/// Aggregated cache performance statistics.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheStats {
    /// Total number of entries currently in the cache.
    pub total_entries: u32,
    /// Total bytes consumed by all cache entries.
    pub total_size_bytes: u64,
    /// Number of successful cache hits.
    pub hit_count: u64,
    /// Number of cache misses requiring a network fetch.
    pub miss_count: u64,
    /// Number of entries that have been evicted.
    pub eviction_count: u32,
    /// Cache hit rate in basis points (10 000 = 100%).
    pub hit_rate_bps: u32,
    /// Average time to serve a cached entry in milliseconds.
    pub avg_access_time_ms: u32,
}

// ============================================================================
// Cross-Device Sync Types (NEW)
// ============================================================================

/// Registration record for a device linked to a user account.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceRegistration {
    /// Unique device identifier.
    pub device_id: String,
    /// Form-factor classification of the device.
    pub device_type: DeviceType,
    /// Unix timestamp when the device was first registered.
    pub registered_at: u64,
    /// Unix timestamp of the most recent activity from this device.
    pub last_seen: u64,
    /// Whether cross-device sync is enabled for this device.
    pub sync_enabled: bool,
    /// Human-readable name for the device.
    pub device_name: String,
    /// Hardware and software capabilities of the device.
    pub capabilities: DeviceCapabilities,
}

/// Form-factor classification of a registered device.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeviceType {
    /// Smartphone.
    MobilePhone,
    /// Tablet or iPad.
    Tablet,
    /// Desktop computer.
    Desktop,
    /// Laptop computer.
    Laptop,
    /// Wearable device (e.g. smartwatch).
    Wearable,
}

/// Hardware and software capabilities reported by a device.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceCapabilities {
    /// Whether the device can receive push notifications.
    pub supports_notifications: bool,
    /// Whether the device has biometric authentication hardware.
    pub supports_biometric: bool,
    /// Whether the device supports offline operation mode.
    pub supports_offline: bool,
    /// Maximum local storage available in bytes.
    pub max_storage_bytes: u64,
    /// Whether the device runs on battery power.
    pub battery_powered: bool,
}

/// Snapshot of a user's learning state for cross-device sync.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LearningState {
    /// Address of the user this state belongs to.
    pub user: Address,
    /// Monotonically increasing version counter.
    pub state_version: u64,
    /// Unix timestamp when the state was last written.
    pub last_updated: u64,
    /// IDs of courses the user is actively enrolled in.
    pub active_courses: Vec<String>,
    /// Map of course ID to completion percentage (0-100).
    pub progress_map: Map<String, u32>,
    /// IDs of bookmarked content items.
    pub bookmarks: Vec<String>,
    /// Hash of the user's current preferences for change detection.
    pub preferences_hash: BytesN<32>,
    /// Opaque token used to detect stale syncs.
    pub sync_token: String,
}

/// An event recording a synchronization action between two devices.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyncEvent {
    /// Unique event identifier.
    pub event_id: String,
    /// Device ID that initiated the sync.
    pub source_device: String,
    /// Device ID that received the synced data.
    pub target_device: String,
    /// Category of data being synchronised.
    pub event_type: SyncEventType,
    /// Unix timestamp when the sync event occurred.
    pub timestamp: u64,
    /// Hash of the data payload for integrity verification.
    pub data_hash: BytesN<32>,
    /// Outcome status of the sync event.
    pub status: SyncEventStatus,
}

/// Category of data involved in a sync event.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyncEventType {
    /// Learning progress data.
    ProgressUpdate,
    /// Bookmarked content items.
    BookmarkSync,
    /// User preference settings.
    PreferenceSync,
    /// Cached content entries.
    CacheSync,
    /// Complete learning state snapshot.
    FullStateSync,
}

/// Outcome status of a sync event.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyncEventStatus {
    /// Sync is queued but not yet started.
    Pending,
    /// Sync is actively transferring data.
    InProgress,
    /// Sync finished successfully.
    Completed,
    /// Sync encountered an unrecoverable error.
    Failed,
    /// Sync completed but conflicts were detected.
    Conflicted,
}

// ============================================================================
// Battery Optimization Types (NEW)
// ============================================================================

/// Current battery state for a user's device.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatteryProfile {
    /// Address of the device owner.
    pub user: Address,
    /// Identifier of the device.
    pub device_id: String,
    /// Current battery charge level as a percentage (0–100).
    pub battery_level: u32,
    /// Whether the device is currently charging.
    pub is_charging: bool,
    /// Active power management mode.
    pub power_mode: PowerMode,
    /// Estimated remaining runtime in minutes at current usage.
    pub estimated_runtime_minutes: u32,
    /// Unix timestamp when this profile was last recorded.
    pub last_updated: u64,
}

/// Power management mode active on the device.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PowerMode {
    /// Standard power usage, no restrictions.
    Normal,
    /// Reduced background activity to extend battery life.
    PowerSaver,
    /// Extreme power saving; only critical functions run.
    UltraSaver,
    /// Maximum performance, battery drains faster.
    Performance,
    /// Automatically adjusts based on battery level.
    Adaptive,
}

/// Configuration thresholds and flags for battery-aware optimizations.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatteryOptimizationConfig {
    /// Battery level at which low-power mode is activated (percentage).
    pub low_battery_threshold: u32,
    /// Battery level at which critical power mode is activated (percentage).
    pub critical_battery_threshold: u32,
    /// Whether to switch to power saver mode automatically on low battery.
    pub auto_power_saver: bool,
    /// Whether to reduce sync frequency on low battery.
    pub reduce_sync_frequency: bool,
    /// Whether to disable content prefetching on low battery.
    pub disable_prefetch_on_low: bool,
    /// Whether to reduce UI animations to save power.
    pub reduce_animation: bool,
    /// Maximum minutes of background operation allowed before suspension.
    pub background_limit_minutes: u32,
}

/// Estimated battery impact report for a mobile session.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatteryImpactReport {
    /// Identifier of the session this report covers.
    pub session_id: String,
    /// Estimated battery drain as a percentage.
    pub estimated_drain_percent: u32,
    /// Total number of operations executed during the session.
    pub operations_count: u32,
    /// Number of sync operations performed.
    pub sync_count: u32,
    /// Number of cache read/write operations performed.
    pub cache_operations: u32,
    /// Number of network calls made.
    pub network_calls: u32,
    /// Actionable recommendations to reduce battery drain.
    pub recommendations: Vec<String>,
}

// ============================================================================
// Notification Types (NEW)
// ============================================================================

/// A scheduled learning reminder for a user.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LearningReminder {
    /// Unique reminder identifier.
    pub reminder_id: String,
    /// Address of the user who owns the reminder.
    pub user: Address,
    /// Category of the reminder.
    pub reminder_type: ReminderType,
    /// Short notification title.
    pub title: String,
    /// Full notification message body.
    pub message: String,
    /// Unix timestamp when the first notification should fire.
    pub scheduled_at: u64,
    /// How often the reminder repeats.
    pub repeat_interval: RepeatInterval,
    /// Whether the reminder is currently active.
    pub is_active: bool,
    /// Unix timestamp when the reminder was last dispatched.
    pub last_sent: u64,
    /// ID of the course this reminder relates to.
    pub course_id: String,
    /// Optional A/B campaign this reminder belongs to.
    pub campaign_id: Option<String>,
    /// Optional A/B variant identifier.
    pub variant_id: Option<String>,
}

/// Category of a learning reminder notification.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReminderType {
    /// Prompt the user to complete their daily study session.
    DailyStudy,
    /// Alert about an upcoming course deadline.
    CourseDeadline,
    /// Encourage the user to maintain their streak.
    StreakMaintenance,
    /// Inform that a new quiz is available.
    QuizAvailable,
    /// Inform that a certificate is ready to claim.
    CertificateReady,
    /// Nudge an inactive user to return.
    InactivityNudge,
    /// Update on progress toward a learning goal.
    GoalProgress,
    /// Alert about activity from a peer or study group.
    PeerActivity,
}

/// How frequently a reminder repeats.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RepeatInterval {
    /// Send once and then disable.
    Once,
    /// Repeat every day.
    Daily,
    /// Repeat every week.
    Weekly,
    /// Repeat on a user-defined schedule.
    Custom,
    /// Fire in response to a specific platform event.
    OnEvent,
}

/// Per-user notification delivery configuration.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationConfig {
    /// Address of the user this configuration belongs to.
    pub user: Address,
    /// Whether notifications are globally enabled.
    pub enabled: bool,
    /// Hour (0–23) at which the quiet period starts.
    pub quiet_hours_start: u32,
    /// Hour (0–23) at which the quiet period ends.
    pub quiet_hours_end: u32,
    /// Maximum notifications the user will receive per day.
    pub max_daily_notifications: u32,
    /// Per-channel opt-in flags (channel name → enabled).
    pub channel_preferences: Map<String, bool>,
    /// Minimum priority a notification must have to be delivered.
    pub priority_threshold: NotificationPriorityLevel,
    /// Preferred language for notification content.
    pub language_preference: String,
    /// Whether the user has consented to marketing messages.
    pub marketing_consent: bool,
}

/// Minimum notification priority the user wishes to receive.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NotificationPriorityLevel {
    /// Receive every notification regardless of priority.
    All,
    /// Receive medium-priority and above.
    Medium,
    /// Receive high-priority and above.
    High,
    /// Receive only critical notifications.
    CriticalOnly,
}

/// Record of a notification that was dispatched to a user.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationRecord {
    /// Unique notification identifier.
    pub notification_id: String,
    /// Address of the notification recipient.
    pub user: Address,
    /// Category of the notification.
    pub notification_type: ReminderType,
    /// Unix timestamp when the notification was sent.
    pub sent_at: u64,
    /// Unix timestamp when the notification was read (0 if unread).
    pub read_at: u64,
    /// Whether the user took action on the notification.
    pub action_taken: bool,
    /// Current delivery status.
    pub delivery_status: DeliveryStatus,
    /// Optional A/B campaign this notification belongs to.
    pub campaign_id: Option<String>,
    /// Optional A/B variant identifier.
    pub variant_id: Option<String>,
    /// Unix timestamp when the notification was clicked (0 if not clicked).
    pub clicked_at: u64,
}

/// Delivery status of a dispatched notification.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeliveryStatus {
    /// Queued but not yet sent.
    Pending,
    /// Sent to the delivery provider.
    Sent,
    /// Confirmed delivered to the device.
    Delivered,
    /// Opened by the user.
    Read,
    /// Delivery failed.
    Failed,
    /// Notification was not delivered before its TTL.
    Expired,
}

/// A reusable notification message template.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationTemplate {
    /// Unique template identifier.
    pub template_id: String,
    /// Category this template is used for.
    pub category: ReminderType,
    /// Default English content when no localization matches.
    pub default_content: String,
    /// Language-code to localized content map.
    pub localized_content: Map<String, String>,
    /// Delivery channels this template supports.
    pub supported_channels: Vec<String>,
    /// Template version number for change tracking.
    pub version: u32,
}

/// An A/B test campaign for notification content.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationCampaign {
    /// Unique campaign identifier.
    pub campaign_id: String,
    /// Human-readable campaign name.
    pub name: String,
    /// Message variants being tested.
    pub variants: Vec<ABTestVariant>,
    /// Unix timestamp when the campaign starts.
    pub start_date: u64,
    /// Unix timestamp when the campaign ends.
    pub end_date: u64,
    /// Whether the campaign is currently running.
    pub is_active: bool,
    /// Total notifications sent across all variants.
    pub total_sent: u32,
    /// Total users who engaged with a notification.
    pub total_engaged: u32,
}

/// A single variant in an A/B notification test.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ABTestVariant {
    /// Unique variant identifier.
    pub variant_id: String,
    /// ID of the notification template used for this variant.
    pub template_id: String,
    /// Relative traffic weight (higher = more traffic).
    pub weight: u32,
}

// ============================================================================
// Security Types (NEW)
// ============================================================================

/// Security configuration and authentication state for a user's account.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityProfile {
    /// Address of the user this profile belongs to.
    pub user: Address,
    /// Whether biometric authentication is active.
    pub biometric_enabled: bool,
    /// Type of biometric authentication configured.
    pub biometric_type: BiometricType,
    /// Seconds of inactivity before the session is locked.
    pub session_lock_timeout: u64,
    /// Number of consecutive failed authentication attempts.
    pub failed_attempts: u32,
    /// Number of failed attempts allowed before lockout.
    pub max_failed_attempts: u32,
    /// Unix timestamp until which the account is locked (0 = not locked).
    pub lockout_until: u64,
    /// Device IDs the user has marked as trusted.
    pub trusted_devices: Vec<String>,
    /// Unix timestamp of the last security check.
    pub last_security_check: u64,
    /// Whether two-factor authentication is active.
    pub two_factor_enabled: bool,
}

/// Biometric authentication method configured on a device.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BiometricType {
    /// No biometric authentication configured.
    None,
    /// Fingerprint sensor.
    Fingerprint,
    /// Facial recognition (Face ID).
    FaceId,
    /// Iris scanner.
    Iris,
    /// Voice recognition.
    VoiceRecognition,
}

/// A recorded authentication attempt by a user.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticationEvent {
    /// Unique event identifier.
    pub event_id: String,
    /// Address of the user who attempted authentication.
    pub user: Address,
    /// Identifier of the device used.
    pub device_id: String,
    /// Authentication method used.
    pub auth_method: AuthMethod,
    /// Unix timestamp of the attempt.
    pub timestamp: u64,
    /// Whether the attempt succeeded.
    pub success: bool,
    /// Hashed IP address of the request origin.
    pub ip_hash: BytesN<32>,
    /// Risk score computed for this attempt (0–100).
    pub risk_score: u32,
}

/// Authentication method used in a login attempt.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthMethod {
    /// Traditional password.
    Password,
    /// Biometric sensor (fingerprint, face, etc.).
    Biometric,
    /// Two-factor authentication code.
    TwoFactor,
    /// Persistent device token.
    DeviceToken,
    /// Resuming an existing valid session.
    SessionResume,
}

/// A security alert raised for a user's account.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityAlert {
    /// Unique alert identifier.
    pub alert_id: String,
    /// Address of the user the alert concerns.
    pub user: Address,
    /// Category of the security event.
    pub alert_type: SecurityAlertType,
    /// Severity level of the alert.
    pub severity: AlertSeverity,
    /// Human-readable description of the alert.
    pub message: String,
    /// Unix timestamp when the alert was raised.
    pub timestamp: u64,
    /// Whether the alert has been acknowledged and resolved.
    pub resolved: bool,
}

/// Category of security alert raised for a user.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SecurityAlertType {
    /// Login attempt from an unrecognised device.
    UnknownDevice,
    /// Threshold of failed login attempts exceeded.
    MultipleFailedAttempts,
    /// Login from an unexpected geographic location.
    LocationAnomaly,
    /// Possible session token theft.
    SessionHijack,
    /// Possible credential or data exposure.
    DataBreach,
    /// Behaviour that does not match the user's normal pattern.
    SuspiciousActivity,
}

/// Severity level for a security alert.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlertSeverity {
    /// Informational; no immediate action required.
    Info,
    /// Requires attention but is not an emergency.
    Warning,
    /// Requires immediate action.
    Critical,
}

// ============================================================================
// PWA Types (NEW)
// ============================================================================

/// Progressive Web App configuration for a user's browser installation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PwaConfig {
    /// Address of the user this PWA config belongs to.
    pub user: Address,
    /// Current installation status of the PWA.
    pub install_status: PwaInstallStatus,
    /// Version string of the active service worker.
    pub service_worker_version: String,
    /// URL routes that have been cached for offline access.
    pub cached_routes: Vec<String>,
    /// Pages that can be served offline.
    pub offline_pages: Vec<String>,
    /// Whether background sync is enabled.
    pub background_sync_enabled: bool,
    /// Whether a Web Push subscription is active.
    pub push_subscription_active: bool,
    /// Total storage quota allocated to the PWA in bytes.
    pub storage_quota_bytes: u64,
    /// Storage currently consumed by the PWA in bytes.
    pub storage_used_bytes: u64,
}

/// Installation status of a Progressive Web App.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PwaInstallStatus {
    /// PWA has not been installed.
    NotInstalled,
    /// The install prompt has been shown to the user.
    PromptShown,
    /// PWA is installed but running in a browser tab.
    Installed,
    /// PWA is installed and running in standalone window mode.
    Standalone,
}

/// Web app manifest metadata for the PWA.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PwaManifest {
    /// Full application name shown in install prompts.
    pub app_name: String,
    /// Short name used on home screens and task switchers.
    pub short_name: String,
    /// Application version string.
    pub version: String,
    /// CSS colour for the browser UI chrome (e.g. "#3367D6").
    pub theme_color: String,
    /// Background colour shown during app launch (e.g. "#FFFFFF").
    pub background_color: String,
    /// Display mode for the installed PWA.
    pub display_mode: DisplayMode,
    /// Screen orientation preference (e.g. "portrait", "landscape").
    pub orientation: String,
    /// URL loaded when the PWA is launched.
    pub start_url: String,
}

/// Display mode used when the PWA is launched as a standalone app.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisplayMode {
    /// Runs in its own window without any browser UI.
    Standalone,
    /// Runs full-screen with no browser chrome.
    Fullscreen,
    /// Shows minimal browser UI (back/forward buttons only).
    MinimalUi,
    /// Runs inside a normal browser tab.
    Browser,
}

/// Runtime status of the service worker for a PWA.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceWorkerStatus {
    /// Version string of the service worker script.
    pub version: String,
    /// Current lifecycle state of the service worker.
    pub state: SwState,
    /// Unix timestamp when the service worker was last updated.
    pub last_updated: u64,
    /// Number of static assets currently cached.
    pub cached_assets_count: u32,
    /// Number of API responses currently cached.
    pub cached_api_responses: u32,
    /// Number of background sync tasks waiting to fire.
    pub pending_sync_count: u32,
}

/// Lifecycle state of a service worker.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SwState {
    /// Service worker script is being downloaded and parsed.
    Installing,
    /// Script is installed but waiting to become active.
    Installed,
    /// Service worker is taking over from the previous version.
    Activating,
    /// Service worker is fully active and controlling pages.
    Activated,
    /// Service worker has been superseded and will be removed.
    Redundant,
}

// ============================================================================
// Analytics & Monitoring Types (NEW)
// ============================================================================

/// Aggregated mobile analytics for a user and device over a time period.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileAnalytics {
    /// Address of the user.
    pub user: Address,
    /// Identifier of the device.
    pub device_id: String,
    /// Total number of sessions in the period.
    pub session_count: u32,
    /// Total operations submitted.
    pub total_operations: u32,
    /// Number of operations that completed successfully.
    pub successful_operations: u32,
    /// Number of operations that failed.
    pub failed_operations: u32,
    /// Average gas consumed per operation.
    pub average_gas_used: u64,
    /// Distribution of sessions by network quality label.
    pub network_quality_distribution: Map<String, u32>,
    /// Breakdown of operations by type.
    pub common_operation_types: Vec<OperationTypeStats>,
    /// Measured impact of optimizer features.
    pub optimization_impact: OptimizationImpact,
    /// Unix timestamp for the start of the analytics period.
    pub period_start: u64,
    /// Unix timestamp for the end of the analytics period.
    pub period_end: u64,
}

/// Per-type performance statistics for operations.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationTypeStats {
    /// Category of operation these stats describe.
    pub operation_type: OperationType,
    /// Total number of operations of this type.
    pub count: u32,
    /// Percentage of operations that succeeded.
    pub success_rate: u32,
    /// Average gas consumed per operation of this type.
    pub average_gas: u64,
    /// Average execution time in milliseconds.
    pub average_duration_ms: u32,
}

/// Measured improvement achieved by the mobile optimizer.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptimizationImpact {
    /// Gas savings as a percentage of unoptimised cost.
    pub gas_savings_pct: u32,
    /// Improvement in operation success rate (percentage points).
    pub op_success_rate_improvement: u32,
    /// Reduction in average response time in milliseconds.
    pub avg_response_improve_ms: u32,
    /// Reduction in battery drain as a percentage.
    pub battery_reduction_pct: u32,
    /// Reduction in mobile data usage as a percentage.
    pub data_reduction_pct: u32,
}

/// Client-side performance measurements for a single session.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerformanceMetrics {
    /// Session this measurement belongs to.
    pub session_id: String,
    /// Unix timestamp when the measurement was recorded.
    pub timestamp: u64,
    /// Time taken to load a page in milliseconds.
    pub page_load_time_ms: u32,
    /// Median API response time in milliseconds.
    pub api_response_time_ms: u32,
    /// Time taken to render the UI in milliseconds.
    pub render_time_ms: u32,
    /// Memory consumed by the app in bytes.
    pub memory_usage_bytes: u64,
    /// Network round-trip latency in milliseconds.
    pub network_latency_ms: u32,
    /// Rendered frames per second.
    pub frame_rate: u32,
    /// Number of JavaScript or Rust errors encountered.
    pub error_count: u32,
    /// Number of app crashes recorded.
    pub crash_count: u32,
}

/// Engagement metrics for a user over a day.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserEngagement {
    /// Address of the user.
    pub user: Address,
    /// Total seconds the user was active today.
    pub daily_active_time_seconds: u64,
    /// Number of sessions started today.
    pub sessions_today: u32,
    /// Number of distinct courses accessed.
    pub courses_accessed: u32,
    /// Number of modules completed today.
    pub modules_completed: u32,
    /// Current consecutive-day learning streak.
    pub streak_days: u32,
    /// Unix timestamp of the most recent activity.
    pub last_active: u64,
    /// Composite engagement score (0–100).
    pub engagement_score: u32,
}

/// A single analytics event recorded on the mobile client.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalyticsEvent {
    /// Unique event identifier.
    pub event_id: String,
    /// Address of the user who triggered the event.
    pub user: Address,
    /// Category of the event.
    pub event_type: AnalyticsEventType,
    /// Unix timestamp when the event occurred.
    pub timestamp: u64,
    /// Additional key-value properties attached to the event.
    pub properties: Map<String, String>,
    /// Session in which the event occurred.
    pub session_id: String,
    /// Type of device that generated the event.
    pub device_type: DeviceType,
}

/// Category of an analytics event.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnalyticsEventType {
    /// User started a new session.
    SessionStart,
    /// User ended a session.
    SessionEnd,
    /// User navigated to a page.
    PageView,
    /// User tapped or clicked a button.
    ButtonClick,
    /// User began a course.
    CourseStart,
    /// User completed a module.
    ModuleComplete,
    /// User attempted a quiz.
    QuizAttempt,
    /// User claimed a certificate.
    CertificateClaim,
    /// User switched offline mode on or off.
    OfflineToggle,
    /// Background sync completed.
    SyncComplete,
    /// An error occurred in the app.
    ErrorOccurred,
    /// A performance warning threshold was breached.
    PerformanceWarning,
}

/// High-level analytics dashboard for the mobile optimizer contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalyticsDashboard {
    /// Total registered users.
    pub total_users: u32,
    /// Users who were active in the last 24 hours.
    pub active_users_24h: u32,
    /// Users who were active in the last 7 days.
    pub active_users_7d: u32,
    /// Total number of sessions recorded.
    pub total_sessions: u64,
    /// Average session duration in seconds.
    pub avg_session_duration_seconds: u64,
    /// Percentage of sessions that used offline mode.
    pub offline_usage_percentage: u32,
    /// Cache hit rate in basis points (10 000 = 100%).
    pub cache_hit_rate_bps: u32,
    /// Average time to complete a background sync in milliseconds.
    pub avg_sync_time_ms: u32,
    /// Error rate in basis points.
    pub error_rate_bps: u32,
    /// Usage statistics broken down by device type.
    pub top_devices: Vec<DeviceUsageStats>,
}

/// Usage statistics for a specific device type.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceUsageStats {
    /// Device form-factor being described.
    pub device_type: DeviceType,
    /// Number of users on this device type.
    pub user_count: u32,
    /// Average session duration in seconds.
    pub avg_session_duration: u64,
    /// Average battery drain per session as a percentage.
    pub avg_battery_impact: u32,
}

// ============================================================================
// Mobile Error Types
// ============================================================================

/// A structured error produced by the mobile optimizer.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileError {
    /// Machine-readable error code.
    pub error_code: String,
    /// Category of the error.
    pub error_type: MobileErrorType,
    /// Short message suitable for display to end users.
    pub user_friendly_message: String,
    /// Full technical description for debugging.
    pub technical_details: String,
    /// Actionable steps the user or app can take.
    pub suggested_actions: Vec<String>,
    /// Whether retrying the operation is likely to succeed.
    pub retry_recommended: bool,
    /// Unix timestamp when the error occurred.
    pub timestamp: u64,
}

/// Category of a mobile optimizer error.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MobileErrorType {
    /// Network request timed out.
    NetworkTimeout,
    /// Operation ran out of gas.
    InsufficientGas,
    /// On-chain transaction was rejected.
    TransactionFailed,
    /// Smart contract returned an error.
    ContractError,
    /// Input failed validation before submission.
    ValidationError,
    /// Authentication was required but failed.
    AuthenticationError,
    /// Too many requests were made in a short window.
    RateLimitExceeded,
    /// The backend service is unavailable.
    ServiceUnavailable,
    /// Cached or synced data is corrupted.
    DataCorruption,
    /// Offline sync produced a merge conflict.
    SyncConflict,
    /// The local cache has reached its size limit.
    CacheFull,
    /// A security policy was violated.
    SecurityViolation,
    /// The device has not been registered for this account.
    DeviceNotRegistered,
    /// Biometric authentication failed.
    BiometricFailed,
}

// ============================================================================
// Storage Keys
// ============================================================================

/// Storage keys for the mobile optimizer contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Contract administrator address.
    Admin,
    /// Global mobile optimizer configuration.
    Config,
    /// Contract initialization flag.
    Initialized,
    /// A transaction batch by its ID.
    TransactionBatch(String),
    /// List of batch IDs belonging to a user.
    UserBatches(Address),
    /// A mobile session by its ID.
    MobileSession(String),
    /// List of session IDs belonging to a user.
    UserSessions(Address),
    /// Offline operation queue for a user.
    OfflineQueue(Address),
    /// A cache entry by its key.
    ContentCache(String),
    /// Cache configuration for a user.
    UserCacheConfig(Address),
    /// Cache performance statistics for a user.
    CacheStats(Address),
    /// Prefetch rules configured for a user.
    PrefetchRules(Address),
    /// Battery profile for a device by device ID.
    BatteryProfile(String),
    /// Notification configuration for a user.
    NotifConfig(Address),
    /// Active reminders for a user.
    Reminders(Address),
    /// Security profile for a user.
    SecurityProfile(Address),
    /// PWA configuration for a user.
    PwaConfig(Address),
    /// Global PWA manifest.
    PwaManifest,
    /// Global analytics dashboard snapshot.
    AnalyticsDashboard,
    /// Running total of sessions.
    TotalSessions,
    /// Running total of batches submitted.
    TotalBatches,
    /// Running total of offline operations queued.
    TotalOfflineOps,
    /// A notification template by its ID.
    NotificationTemplate(String),
    /// A notification campaign by its ID.
    NotificationCampaign(String),
    /// A content metadata item by its ID.
    ContentItem(String),
    /// Version history for a content item by content ID.
    ContentVersionHistory(String),
    /// A study group by its ID.
    StudyGroup(String),
    /// A forum post by its ID.
    ForumPost(String),
    /// A peer review by its ID.
    PeerReview(String),
    /// A mentorship session by its ID.
    MentorshipSession(String),
    /// Collaboration profile for a user.
    CollabProfile(Address),
    /// UI preferences for a user.
    UiPreferences(Address),
    /// Onboarding state for a user.
    OnboardingState(Address),
    /// Feedback history for a user.
    UserFeedbackHistory(Address),
    /// Analytics events for a user.
    AnalyticsEvents(Address),
    /// Performance log for a session by session ID.
    PerformanceLog(String),
    /// Battery optimization configuration for a user.
    BatteryConfig(Address),
    /// Notification history for a user.
    NotifHistory(Address),
    /// Authentication events for a user.
    AuthEvents(Address),
    /// Security alerts for a user.
    SecurityAlerts(Address),
    /// Service worker status for a user.
    SwStatus(Address),
    /// Engagement metrics for a user.
    UserEngagement(Address),
}

// ============================================================================
// User Experience Types (NEW)
// ============================================================================

/// User interface display preferences for the mobile app.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiPreferences {
    /// Address of the user these preferences belong to.
    pub user: Address,
    /// Identifier of the selected colour theme.
    pub theme_id: String,
    /// Language code used for the UI (e.g. "en", "es").
    pub language: String,
    /// Font size multiplier as a percentage (100 = default size).
    pub font_scale: u32,
    /// Whether high-contrast mode is enabled for accessibility.
    pub high_contrast: bool,
    /// Whether UI animations should be reduced for accessibility.
    pub reduce_motion: bool,
    /// Chosen layout density mode.
    pub layout_mode: LayoutMode,
    /// Additional accessibility feature toggles.
    pub accessibility_settings: Map<String, bool>,
}

/// UI layout density mode.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LayoutMode {
    /// Default layout with standard spacing.
    Standard,
    /// Densely packed layout showing more items on screen.
    Compact,
    /// Spacious layout with extra padding for readability.
    Comfortable,
    /// Layout optimised for small mobile screens.
    MobileOptimized,
}

/// Tracks where a user is in the app onboarding flow.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnboardingState {
    /// Address of the user being onboarded.
    pub user: Address,
    /// Whether the user has completed the full onboarding flow.
    pub is_completed: bool,
    /// Index of the step the user is currently on.
    pub current_step: u32,
    /// IDs of steps the user has already finished.
    pub completed_steps: Vec<String>,
    /// IDs of steps the user has chosen to skip.
    pub skipped_steps: Vec<String>,
    /// Unix timestamp when this record was last modified.
    pub last_updated: u64,
}

/// Feedback submitted by a user about the application.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserFeedback {
    /// Unique feedback identifier.
    pub feedback_id: String,
    /// Address of the user who submitted the feedback.
    pub user: Address,
    /// High-level category of the feedback (e.g. "bug", "suggestion").
    pub category: String,
    /// Rating given by the user (1–5).
    pub rating: u32,
    /// Free-text comment.
    pub comment: String,
    /// Additional contextual data attached to the feedback.
    pub context_data: Map<String, String>,
    /// Unix timestamp when the feedback was submitted.
    pub timestamp: u64,
}

// ============================================================================
// Content Management Types (NEW)
// ============================================================================

/// Metadata describing a piece of managed content.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentMetadata {
    /// Unique content identifier.
    pub content_id: String,
    /// Type classification of the content.
    pub content_type: ContentType,
    /// Human-readable content title.
    pub title: String,
    /// URI where the content can be retrieved.
    pub uri: String,
    /// Currently active version number.
    pub current_version: u32,
    /// Address of the content author.
    pub author: Address,
    /// Access control rule governing who may view the content.
    pub access_rule: ContentAccessRule,
    /// Delivery configuration for CDN and DRM.
    pub delivery_config: ContentDeliveryConfig,
    /// Total number of views across all versions.
    pub total_views: u32,
    /// Average user rating (1–100 scale).
    pub average_rating: u32,
}

/// A specific version of a managed content item.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentVersion {
    /// Identifier of the parent content item.
    pub content_id: String,
    /// Version number for this entry.
    pub version: u32,
    /// Hash of the content at this version.
    pub content_hash: BytesN<32>,
    /// URI for this specific version's assets.
    pub uri: String,
    /// Unix timestamp when this version was created.
    pub created_at: u64,
    /// Description of changes introduced in this version.
    pub changelog: String,
}

/// Access control rule for a content item.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentAccessRule {
    /// Available to anyone without authentication.
    Public,
    /// Requires a registered account.
    RegisteredUser,
    /// Requires enrolment in the specified course.
    CourseEnrolled(String),
    /// Only available to premium subscribers.
    PremiumOnly,
    /// Only accessible by the content creator.
    CreatorOnly,
}

/// CDN and DRM delivery settings for a content item.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentDeliveryConfig {
    /// Whether CDN distribution is enabled.
    pub cdn_enabled: bool,
    /// Country or region codes where access is restricted.
    pub region_restrictions: Vec<String>,
    /// Compression and encoding optimization level (0–10).
    pub optimization_level: u32,
    /// Whether digital rights management is enforced.
    pub drm_enabled: bool,
}

// ============================================================================
// Collaboration Types (NEW)
// ============================================================================

/// A collaborative study group for learners.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StudyGroup {
    /// Unique group identifier.
    pub group_id: String,
    /// Human-readable group name.
    pub name: String,
    /// Address of the user who created the group.
    pub creator: Address,
    /// Addresses of all group members.
    pub members: Vec<Address>,
    /// Subject or topic the group focuses on.
    pub topic: String,
    /// Unix timestamp when the group was created.
    pub created_at: u64,
    /// Whether the group is currently accepting new members.
    pub is_active: bool,
    /// Maximum number of members allowed.
    pub max_members: u32,
}

/// A post in a study group forum thread.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForumPost {
    /// Unique post identifier.
    pub post_id: String,
    /// Identifier of the group this post belongs to.
    pub group_id: String,
    /// Address of the post author.
    pub author: Address,
    /// Text content of the post.
    pub content: String,
    /// Unix timestamp when the post was created.
    pub timestamp: u64,
    /// Number of upvotes the post has received.
    pub upvotes: u32,
    /// Optional ID of the parent post (for threaded replies).
    pub parent_id: Option<String>,
}

/// A peer review submitted by one user about another's work.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PeerReview {
    /// Unique review identifier.
    pub review_id: String,
    /// Address of the user writing the review.
    pub reviewer: Address,
    /// Address of the user being reviewed.
    pub target_user: Address,
    /// Identifier of the submission or artefact being reviewed.
    pub context_id: String,
    /// Numerical score assigned by the reviewer (0–100).
    pub score: u32,
    /// Written feedback from the reviewer.
    pub comments: String,
    /// Unix timestamp when the review was submitted.
    pub timestamp: u64,
}

/// A scheduled one-on-one mentorship session.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MentorshipSession {
    /// Unique session identifier.
    pub session_id: String,
    /// Address of the mentor.
    pub mentor: Address,
    /// Address of the mentee.
    pub mentee: Address,
    /// Topic or focus area for the session.
    pub topic: String,
    /// Current lifecycle status of the session.
    pub status: MentorshipStatus,
    /// Unix timestamp when the session is scheduled to begin.
    pub scheduled_at: u64,
    /// Planned duration of the session in minutes.
    pub duration_minutes: u32,
}

/// Lifecycle status of a mentorship session.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MentorshipStatus {
    /// Session request has been sent but not yet accepted.
    Pending,
    /// Session has been confirmed by the mentor.
    Accepted,
    /// Session took place and is now closed.
    Completed,
    /// Session was cancelled before it occurred.
    Cancelled,
    /// Mentor declined the session request.
    Rejected,
}

/// Collaboration reputation and activity profile for a user.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollaborationProfile {
    /// Address of the user this profile belongs to.
    pub user: Address,
    /// Overall reputation score based on contributions (0–100).
    pub reputation_score: u32,
    /// Total number of study groups the user has joined.
    pub groups_joined: u32,
    /// Total number of peer reviews the user has written.
    pub reviews_given: u32,
    /// Total number of mentorship sessions the user has completed.
    pub mentorships_completed: u32,
    /// Earned collaboration badges.
    pub badges: Vec<String>,
}

// ============================================================================
// Contract-Level Error Enum
// ============================================================================

pub use crate::errors::MobileOptimizerError;
