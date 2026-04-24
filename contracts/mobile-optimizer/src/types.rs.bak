#![allow(clippy::enum_variant_names)]

use soroban_sdk::{contracterror, contracttype, Address, BytesN, Map, String, Vec};

// ============================================================================
// Core Transaction & Batch Types
// ============================================================================

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
    pub network_quality: NetworkQuality,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchOperation {
    pub operation_id: String,
    pub operation_type: OperationType,
    pub contract_address: Address,
    pub function_name: String,
    pub parameters: Vec<OperationParameter>,
    pub estimated_gas: u64,
    pub priority: OperationPriority,
    pub retry_config: RetryConfig,
    pub dependencies: Vec<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationParameter {
    pub param_name: String,
    pub param_value: ParameterValue,
    pub param_type: ParameterType,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ParameterValue {
    AddressVal(Address),
    StringVal(String),
    U32Val(u32),
    U64Val(u64),
    I64Val(i64),
    BoolVal(bool),
    BytesVal(BytesN<32>),
    VectorVal(Vec<String>),
    MapVal(Map<String, String>),
}

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationType {
    CourseEnrollment,
    ProgressUpdate,
    CertificateRequest,
    CertificateRenewal,
    CertificateGeneration,
    SearchQuery,
    PreferenceUpdate,
    TokenTransfer,
    TokenStaking,
    TokenBurning,
    TokenReward,
    ContentCache,
    LearningSync,
    NotificationConfig,
    SecurityUpdate,
    AnalyticsEvent,
    Custom,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationPriority {
    Critical,
    High,
    Medium,
    Low,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BatchPriority {
    Critical,
    High,
    Normal,
    Low,
    Background,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BatchStatus {
    Pending,
    Executing,
    Completed,
    PartialSuccess,
    Failed,
    Cancelled,
    Expired,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum OperationStatus {
    Pending,
    Executing,
    Completed,
    Failed,
    Skipped,
    Retrying,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ExecutionStrategy {
    Sequential,
    Parallel,
    Optimized,
    Conservative,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RetryConfig {
    pub max_retries: u32,
    pub retry_delay_ms: u32,
    pub backoff_multiplier: u32,
    pub max_delay_ms: u32,
    pub retry_on_network_error: bool,
    pub retry_on_gas_error: bool,
    pub retry_on_timeout: bool,
}

// ============================================================================
// Session Types
// ============================================================================

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
    pub pending_operations: Vec<String>,
    pub preferences: MobilePreferences,
    pub session_state: SessionState,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SessionState {
    Active,
    Idle,
    Background,
    Suspended,
    Expired,
}

// ============================================================================
// Network Types
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NetworkQuality {
    Excellent,
    Good,
    Fair,
    Poor,
    Offline,
}

// ============================================================================
// Preferences & Configuration
// ============================================================================

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationPreferences {
    pub transaction_complete: bool,
    pub transaction_failed: bool,
    pub batch_ready: bool,
    pub network_issues: bool,
    pub gas_price_alerts: bool,
    pub offline_sync_complete: bool,
    pub learning_reminders: bool,
    pub streak_alerts: bool,
    pub course_updates: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataUsageMode {
    Unlimited,
    Conservative,
    WifiOnly,
    Emergency,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileOptimizerConfig {
    pub admin: Address,
    pub max_batch_size: u32,
    pub default_gas_limit: u64,
    pub session_timeout_seconds: u64,
    pub offline_queue_limit: u32,
    pub network_timeout_ms: u32,
    pub retry_attempts: u32,
    pub cache_ttl_seconds: u64,
    pub max_devices_per_user: u32,
    pub analytics_retention_days: u32,
}

// ============================================================================
// Gas Estimation Types
// ============================================================================

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConfidenceLevel {
    High,
    Medium,
    Low,
    Unknown,
}

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptimizationSuggestion {
    pub suggestion_type: SuggestionType,
    pub description: String,
    pub potential_savings: u64,
    pub implementation_effort: EffortLevel,
    pub applicable: bool,
}

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EffortLevel {
    None,
    Low,
    Medium,
    High,
}

// ============================================================================
// Offline Queue Types
// ============================================================================

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct QueuedOperation {
    pub operation_id: String,
    pub operation_type: OperationType,
    pub parameters: Vec<OperationParameter>,
    pub created_at: u64,
    pub priority: BatchPriority,
    pub local_state_hash: BytesN<32>,
    pub retry_count: u32,
    pub status: QueuedOperationStatus,
    pub estimated_gas: u64,
}

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

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ConflictResolution {
    ServerWins,
    ClientWins,
    MergeChanges,
    UserDecision,
    Abort,
}

// ============================================================================
// Content Cache Types (NEW)
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheEntry {
    pub cache_key: String,
    pub content_hash: BytesN<32>,
    pub content_type: ContentType,
    pub size_bytes: u64,
    pub created_at: u64,
    pub expires_at: u64,
    pub access_count: u32,
    pub last_accessed: u64,
    pub priority: CachePriority,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentType {
    CourseMaterial,
    VideoLesson,
    QuizData,
    Certificate,
    UserProfile,
    SearchResults,
    ProgressData,
    NotificationData,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CachePriority {
    Essential,
    High,
    Normal,
    Low,
    Evictable,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheConfig {
    pub max_cache_size_bytes: u64,
    pub default_ttl_seconds: u64,
    pub eviction_policy: EvictionPolicy,
    pub prefetch_enabled: bool,
    pub compression_enabled: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum EvictionPolicy {
    LeastRecentlyUsed,
    LeastFrequentlyUsed,
    TimeToLive,
    PriorityBased,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PrefetchRule {
    pub rule_id: String,
    pub content_type: ContentType,
    pub trigger: PrefetchTrigger,
    pub network_requirement: NetworkQuality,
    pub max_prefetch_size_bytes: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PrefetchTrigger {
    OnCourseEnroll,
    OnModuleComplete,
    OnWifiConnect,
    OnSchedule,
    OnLowActivity,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CacheStats {
    pub total_entries: u32,
    pub total_size_bytes: u64,
    pub hit_count: u64,
    pub miss_count: u64,
    pub eviction_count: u32,
    pub hit_rate_bps: u32,
    pub avg_access_time_ms: u32,
}

// ============================================================================
// Cross-Device Sync Types (NEW)
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceRegistration {
    pub device_id: String,
    pub device_type: DeviceType,
    pub registered_at: u64,
    pub last_seen: u64,
    pub sync_enabled: bool,
    pub device_name: String,
    pub capabilities: DeviceCapabilities,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeviceType {
    MobilePhone,
    Tablet,
    Desktop,
    Laptop,
    Wearable,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceCapabilities {
    pub supports_notifications: bool,
    pub supports_biometric: bool,
    pub supports_offline: bool,
    pub max_storage_bytes: u64,
    pub battery_powered: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LearningState {
    pub user: Address,
    pub state_version: u64,
    pub last_updated: u64,
    pub active_courses: Vec<String>,
    pub progress_map: Map<String, u32>,
    pub bookmarks: Vec<String>,
    pub preferences_hash: BytesN<32>,
    pub sync_token: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SyncEvent {
    pub event_id: String,
    pub source_device: String,
    pub target_device: String,
    pub event_type: SyncEventType,
    pub timestamp: u64,
    pub data_hash: BytesN<32>,
    pub status: SyncEventStatus,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyncEventType {
    ProgressUpdate,
    BookmarkSync,
    PreferenceSync,
    CacheSync,
    FullStateSync,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SyncEventStatus {
    Pending,
    InProgress,
    Completed,
    Failed,
    Conflicted,
}

// ============================================================================
// Battery Optimization Types (NEW)
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatteryProfile {
    pub user: Address,
    pub device_id: String,
    pub battery_level: u32,
    pub is_charging: bool,
    pub power_mode: PowerMode,
    pub estimated_runtime_minutes: u32,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PowerMode {
    Normal,
    PowerSaver,
    UltraSaver,
    Performance,
    Adaptive,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatteryOptimizationConfig {
    pub low_battery_threshold: u32,
    pub critical_battery_threshold: u32,
    pub auto_power_saver: bool,
    pub reduce_sync_frequency: bool,
    pub disable_prefetch_on_low: bool,
    pub reduce_animation: bool,
    pub background_limit_minutes: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatteryImpactReport {
    pub session_id: String,
    pub estimated_drain_percent: u32,
    pub operations_count: u32,
    pub sync_count: u32,
    pub cache_operations: u32,
    pub network_calls: u32,
    pub recommendations: Vec<String>,
}

// ============================================================================
// Notification Types (NEW)
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LearningReminder {
    pub reminder_id: String,
    pub user: Address,
    pub reminder_type: ReminderType,
    pub title: String,
    pub message: String,
    pub scheduled_at: u64,
    pub repeat_interval: RepeatInterval,
    pub is_active: bool,
    pub last_sent: u64,
    pub course_id: String,
    pub campaign_id: Option<String>,
    pub variant_id: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReminderType {
    DailyStudy,
    CourseDeadline,
    StreakMaintenance,
    QuizAvailable,
    CertificateReady,
    InactivityNudge,
    GoalProgress,
    PeerActivity,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RepeatInterval {
    Once,
    Daily,
    Weekly,
    Custom,
    OnEvent,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationConfig {
    pub user: Address,
    pub enabled: bool,
    pub quiet_hours_start: u32,
    pub quiet_hours_end: u32,
    pub max_daily_notifications: u32,
    pub channel_preferences: Map<String, bool>,
    pub priority_threshold: NotificationPriorityLevel,
    pub language_preference: String,
    pub marketing_consent: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NotificationPriorityLevel {
    All,
    Medium,
    High,
    CriticalOnly,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationRecord {
    pub notification_id: String,
    pub user: Address,
    pub notification_type: ReminderType,
    pub sent_at: u64,
    pub read_at: u64,
    pub action_taken: bool,
    pub delivery_status: DeliveryStatus,
    pub campaign_id: Option<String>,
    pub variant_id: Option<String>,
    pub clicked_at: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DeliveryStatus {
    Pending,
    Sent,
    Delivered,
    Read,
    Failed,
    Expired,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationTemplate {
    pub template_id: String,
    pub category: ReminderType,
    pub default_content: String,
    pub localized_content: Map<String, String>,
    pub supported_channels: Vec<String>,
    pub version: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct NotificationCampaign {
    pub campaign_id: String,
    pub name: String,
    pub variants: Vec<ABTestVariant>,
    pub start_date: u64,
    pub end_date: u64,
    pub is_active: bool,
    pub total_sent: u32,
    pub total_engaged: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ABTestVariant {
    pub variant_id: String,
    pub template_id: String,
    pub weight: u32,
}

// ============================================================================
// Security Types (NEW)
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityProfile {
    pub user: Address,
    pub biometric_enabled: bool,
    pub biometric_type: BiometricType,
    pub session_lock_timeout: u64,
    pub failed_attempts: u32,
    pub max_failed_attempts: u32,
    pub lockout_until: u64,
    pub trusted_devices: Vec<String>,
    pub last_security_check: u64,
    pub two_factor_enabled: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BiometricType {
    None,
    Fingerprint,
    FaceId,
    Iris,
    VoiceRecognition,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthenticationEvent {
    pub event_id: String,
    pub user: Address,
    pub device_id: String,
    pub auth_method: AuthMethod,
    pub timestamp: u64,
    pub success: bool,
    pub ip_hash: BytesN<32>,
    pub risk_score: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuthMethod {
    Password,
    Biometric,
    TwoFactor,
    DeviceToken,
    SessionResume,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityAlert {
    pub alert_id: String,
    pub user: Address,
    pub alert_type: SecurityAlertType,
    pub severity: AlertSeverity,
    pub message: String,
    pub timestamp: u64,
    pub resolved: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SecurityAlertType {
    UnknownDevice,
    MultipleFailedAttempts,
    LocationAnomaly,
    SessionHijack,
    DataBreach,
    SuspiciousActivity,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

// ============================================================================
// PWA Types (NEW)
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PwaConfig {
    pub user: Address,
    pub install_status: PwaInstallStatus,
    pub service_worker_version: String,
    pub cached_routes: Vec<String>,
    pub offline_pages: Vec<String>,
    pub background_sync_enabled: bool,
    pub push_subscription_active: bool,
    pub storage_quota_bytes: u64,
    pub storage_used_bytes: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum PwaInstallStatus {
    NotInstalled,
    PromptShown,
    Installed,
    Standalone,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PwaManifest {
    pub app_name: String,
    pub short_name: String,
    pub version: String,
    pub theme_color: String,
    pub background_color: String,
    pub display_mode: DisplayMode,
    pub orientation: String,
    pub start_url: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DisplayMode {
    Standalone,
    Fullscreen,
    MinimalUi,
    Browser,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ServiceWorkerStatus {
    pub version: String,
    pub state: SwState,
    pub last_updated: u64,
    pub cached_assets_count: u32,
    pub cached_api_responses: u32,
    pub pending_sync_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SwState {
    Installing,
    Installed,
    Activating,
    Activated,
    Redundant,
}

// ============================================================================
// Analytics & Monitoring Types (NEW)
// ============================================================================

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
    pub network_quality_distribution: Map<String, u32>,
    pub common_operation_types: Vec<OperationTypeStats>,
    pub optimization_impact: OptimizationImpact,
    pub period_start: u64,
    pub period_end: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OperationTypeStats {
    pub operation_type: OperationType,
    pub count: u32,
    pub success_rate: u32,
    pub average_gas: u64,
    pub average_duration_ms: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OptimizationImpact {
    pub gas_savings_pct: u32,
    pub op_success_rate_improvement: u32,
    pub avg_response_improve_ms: u32,
    pub battery_reduction_pct: u32,
    pub data_reduction_pct: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PerformanceMetrics {
    pub session_id: String,
    pub timestamp: u64,
    pub page_load_time_ms: u32,
    pub api_response_time_ms: u32,
    pub render_time_ms: u32,
    pub memory_usage_bytes: u64,
    pub network_latency_ms: u32,
    pub frame_rate: u32,
    pub error_count: u32,
    pub crash_count: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserEngagement {
    pub user: Address,
    pub daily_active_time_seconds: u64,
    pub sessions_today: u32,
    pub courses_accessed: u32,
    pub modules_completed: u32,
    pub streak_days: u32,
    pub last_active: u64,
    pub engagement_score: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalyticsEvent {
    pub event_id: String,
    pub user: Address,
    pub event_type: AnalyticsEventType,
    pub timestamp: u64,
    pub properties: Map<String, String>,
    pub session_id: String,
    pub device_type: DeviceType,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnalyticsEventType {
    SessionStart,
    SessionEnd,
    PageView,
    ButtonClick,
    CourseStart,
    ModuleComplete,
    QuizAttempt,
    CertificateClaim,
    OfflineToggle,
    SyncComplete,
    ErrorOccurred,
    PerformanceWarning,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalyticsDashboard {
    pub total_users: u32,
    pub active_users_24h: u32,
    pub active_users_7d: u32,
    pub total_sessions: u64,
    pub avg_session_duration_seconds: u64,
    pub offline_usage_percentage: u32,
    pub cache_hit_rate_bps: u32,
    pub avg_sync_time_ms: u32,
    pub error_rate_bps: u32,
    pub top_devices: Vec<DeviceUsageStats>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeviceUsageStats {
    pub device_type: DeviceType,
    pub user_count: u32,
    pub avg_session_duration: u64,
    pub avg_battery_impact: u32,
}

// ============================================================================
// Mobile Error Types
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MobileError {
    pub error_code: String,
    pub error_type: MobileErrorType,
    pub user_friendly_message: String,
    pub technical_details: String,
    pub suggested_actions: Vec<String>,
    pub retry_recommended: bool,
    pub timestamp: u64,
}

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
    CacheFull,
    SecurityViolation,
    DeviceNotRegistered,
    BiometricFailed,
}

// ============================================================================
// Storage Keys
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    Admin,
    Config,
    Initialized,
    TransactionBatch(String),
    UserBatches(Address),
    MobileSession(String),
    UserSessions(Address),
    GasEstimates(String),
    OfflineQueue(Address),
    MobileAnalytics(Address),
    GlobalMobileConfig,
    NetworkMetrics(u64),
    OptimizationCache(String),
    ErrorLogs(Address),
    PerformanceMetrics(u64),
    BatchHistory(String),
    SessionCleanup(u64),
    // New storage keys for enhanced features
    ContentCache(String),
    UserCacheConfig(Address),
    CacheStats(Address),
    PrefetchRules(Address),
    DeviceRegistry(Address),
    LearningState(Address),
    SyncEvents(Address),
    BatteryProfile(String),
    BatteryConfig(Address),
    NotifConfig(Address),
    Reminders(Address),
    NotifHistory(Address),
    SecurityProfile(Address),
    AuthEvents(Address),
    SecurityAlerts(Address),
    PwaConfig(Address),
    PwaManifest,
    SwStatus(Address),
    AnalyticsDashboard,
    UserEngagement(Address),
    AnalyticsEvents(Address),
    PerformanceLog(String),
    TotalSessions,
    TotalBatches,
    TotalOfflineOps,
    NotificationTemplate(String),
    NotificationCampaign(String),
    ContentItem(String),
    ContentVersionHistory(String),
    StudyGroup(String),
    ForumPost(String),
    PeerReview(String),
    MentorshipSession(String),
    CollabProfile(Address),
    UiPreferences(Address),
    OnboardingState(Address),
    UserFeedback(String),
    UserFeedbackHistory(Address),
}

// ============================================================================
// User Experience Types (NEW)
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UiPreferences {
    pub user: Address,
    pub theme_id: String,
    pub language: String,
    pub font_scale: u32,
    pub high_contrast: bool,
    pub reduce_motion: bool,
    pub layout_mode: LayoutMode,
    pub accessibility_settings: Map<String, bool>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum LayoutMode {
    Standard,
    Compact,
    Comfortable,
    MobileOptimized,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OnboardingState {
    pub user: Address,
    pub is_completed: bool,
    pub current_step: u32,
    pub completed_steps: Vec<String>,
    pub skipped_steps: Vec<String>,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserFeedback {
    pub feedback_id: String,
    pub user: Address,
    pub category: String,
    pub rating: u32,
    pub comment: String,
    pub context_data: Map<String, String>,
    pub timestamp: u64,
}

// ============================================================================
// Content Management Types (NEW)
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentMetadata {
    pub content_id: String,
    pub content_type: ContentType,
    pub title: String,
    pub uri: String,
    pub current_version: u32,
    pub author: Address,
    pub access_rule: ContentAccessRule,
    pub delivery_config: ContentDeliveryConfig,
    pub total_views: u32,
    pub average_rating: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentVersion {
    pub content_id: String,
    pub version: u32,
    pub content_hash: BytesN<32>,
    pub uri: String,
    pub created_at: u64,
    pub changelog: String,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ContentAccessRule {
    Public,
    RegisteredUser,
    CourseEnrolled(String),
    PremiumOnly,
    CreatorOnly,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ContentDeliveryConfig {
    pub cdn_enabled: bool,
    pub region_restrictions: Vec<String>,
    pub optimization_level: u32,
    pub drm_enabled: bool,
}

// ============================================================================
// Collaboration Types (NEW)
// ============================================================================

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StudyGroup {
    pub group_id: String,
    pub name: String,
    pub creator: Address,
    pub members: Vec<Address>,
    pub topic: String,
    pub created_at: u64,
    pub is_active: bool,
    pub max_members: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ForumPost {
    pub post_id: String,
    pub group_id: String,
    pub author: Address,
    pub content: String,
    pub timestamp: u64,
    pub upvotes: u32,
    pub parent_id: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PeerReview {
    pub review_id: String,
    pub reviewer: Address,
    pub target_user: Address,
    pub context_id: String,
    pub score: u32,
    pub comments: String,
    pub timestamp: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MentorshipSession {
    pub session_id: String,
    pub mentor: Address,
    pub mentee: Address,
    pub topic: String,
    pub status: MentorshipStatus,
    pub scheduled_at: u64,
    pub duration_minutes: u32,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MentorshipStatus {
    Pending,
    Accepted,
    Completed,
    Cancelled,
    Rejected,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CollaborationProfile {
    pub user: Address,
    pub reputation_score: u32,
    pub groups_joined: u32,
    pub reviews_given: u32,
    pub mentorships_completed: u32,
    pub badges: Vec<String>,
}

// ============================================================================
// Contract-Level Error Enum
// ============================================================================

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum MobileOptimizerError {
    NotInitialized = 1,
    AlreadyInitialized = 2,
    SessionCreationFailed = 3,
    SessionUpdateFailed = 4,
    SessionNotFound = 5,
    SessionExpired = 6,
    BatchExecutionFailed = 7,
    BatchNotFound = 8,
    BatchExpired = 9,
    GasEstimationFailed = 10,
    OptimizationFailed = 11,
    InteractionFailed = 12,
    OfflineOperationFailed = 13,
    OfflineSyncFailed = 14,
    OfflineQueueFull = 15,
    ConflictResolutionFailed = 16,
    PreferenceUpdateFailed = 17,
    AnalyticsNotAvailable = 18,
    ConfigNotFound = 19,
    AdminNotSet = 20,
    UnauthorizedAdmin = 21,
    Unauthorized = 22,
    CacheError = 23,
    CacheFull = 24,
    DeviceNotRegistered = 25,
    MaxDevicesReached = 26,
    SyncFailed = 27,
    SecurityViolation = 28,
    BiometricAuthFailed = 29,
    AccountLocked = 30,
    NotificationError = 31,
    PwaError = 32,
    InvalidInput = 33,
    InternalError = 34,
    ContentError = 35,
    CollaborationError = 36,
    UserExperienceError = 37,
}
