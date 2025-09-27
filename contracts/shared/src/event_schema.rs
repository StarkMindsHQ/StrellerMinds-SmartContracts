use soroban_sdk::{Address, BytesN, Env, Symbol, String, Vec};

/// Standard event schema version
pub const EVENT_SCHEMA_VERSION: u32 = 1;

/// Standard event wrapper that all contracts should use
#[derive(Clone, Debug)]
pub struct StandardEvent {
    /// Schema version for future compatibility
    pub version: u32,
    /// Contract identifier that emitted the event
    pub contract: Symbol,
    /// Address of the actor who triggered the event
    pub actor: Address,
    /// Ledger timestamp when the event occurred
    pub timestamp: u64,
    /// Transaction hash (derived from ledger sequence for now)
    pub tx_hash: BytesN<32>,
    /// Event-specific data
    pub event_data: EventData,
}

/// Event categories for better organization and filtering
#[derive(Clone, Debug)]
pub enum EventCategory {
    /// Access control and permission events
    AccessControl,
    /// Certificate lifecycle events
    Certificate,
    /// Analytics and tracking events
    Analytics,
    /// Token and incentive events
    Token,
    /// Progress tracking events
    Progress,
    /// System and configuration events
    System,
    /// Error and audit events
    Error,
}

/// Standardized event data types
#[derive(Clone, Debug)]
pub enum EventData {
    /// Access control events
    AccessControl(AccessControlEventData),
    /// Certificate events
    Certificate(CertificateEventData),
    /// Analytics events
    Analytics(AnalyticsEventData),
    /// Token events
    Token(TokenEventData),
    /// Progress events
    Progress(ProgressEventData),
    /// System events
    System(SystemEventData),
    /// Error events
    Error(ErrorEventData),
}

/// Access control event data
#[derive(Clone, Debug)]
pub enum AccessControlEventData {
    ContractInitialized { admin: Address },
    RoleGranted { granter: Address, user: Address, role_level: u32, granted_at: u64, expires_at: Option<u64> },
    RoleRevoked { revoker: Address, user: Address, role_level: u32 },
    RoleTransferred { from: Address, to: Address, role_level: u32 },
    RoleUpdated { updater: Address, user: Address, role_level: u32 },
    PermissionGranted { granter: Address, user: Address, permission: String },
    PermissionRevoked { revoker: Address, user: Address, permission: String },
    AdminChanged { old_admin: Address, new_admin: Address },
    RoleExpired { user: Address, role_level: u32 },
    AccessDenied { user: Address, permission: String },
    HierarchyViolation { granter: Address, target: Address, target_level: u32 },
}

/// Certificate event data
#[derive(Clone, Debug)]
pub enum CertificateEventData {
    CertificateMinted { certificate_id: BytesN<32>, student: Address, issuer: Address, token_id: BytesN<32>, metadata_hash: String },
    CertificateRevoked { certificate_id: BytesN<32>, revoker: Address, reason: Option<String> },
    CertificateTransferred { certificate_id: BytesN<32>, from: Address, to: Address },
    MetadataUpdated { certificate_id: BytesN<32>, updater: Address, old_uri: String, new_uri: String },
    RenewalRequested { certificate_id: BytesN<32>, requester: Address, requested_extension: u64 },
    RenewalApproved { certificate_id: BytesN<32>, approver: Address, requester: Address, extension_period: u64 },
    RenewalRejected { certificate_id: BytesN<32>, approver: Address, requester: Address, reason: String },
    CertificateExtended { certificate_id: BytesN<32>, admin: Address, owner: Address, extension_period: u64, reason: String },
    CertificateExpired { certificate_id: BytesN<32>, owner: Address, expiry_date: u64 },
    CertificateAutoRenewed { certificate_id: BytesN<32>, owner: Address, new_expiry_date: u64, renewal_count: u32 },
    ExpiryNotification { certificate_id: BytesN<32>, owner: Address, notification_type: String, expiry_date: u64 },
    NotificationAcknowledged { certificate_id: BytesN<32>, user: Address, notification_type: String },
    BatchMintCompleted { issuer: Address, total_count: u32, success_count: u32, failure_count: u32 },
    IssuerAdded { admin: Address, issuer: Address },
    IssuerRemoved { admin: Address, issuer: Address },
}

/// Analytics event data
#[derive(Clone, Debug)]
pub enum AnalyticsEventData {
    SessionRecorded { session_id: BytesN<32>, student: Address, course_id: Symbol, module_id: Symbol, session_type: String, time_spent: u64, completion_percentage: u32 },
    SessionCompleted { session_id: BytesN<32>, student: Address, course_id: Symbol, module_id: Symbol, final_score: Option<u32>, total_time: u64 },
    ProgressUpdated { student: Address, course_id: Symbol, completion_percentage: u32, total_time_spent: u64, performance_trend: String },
    CourseAnalyticsUpdated { course_id: Symbol, total_students: u32, completion_rate: u32, average_score: Option<u32> },
    ModuleAnalyticsUpdated { course_id: Symbol, module_id: Symbol, completion_rate: u32, average_time: u64, difficulty_rating: String },
    AchievementEarned { student: Address, achievement_id: Symbol, achievement_type: String, course_id: Symbol, earned_date: u64 },
    LeaderboardUpdated { course_id: Symbol, metric_type: String, top_student: Address, top_score: u32, total_entries: u32 },
    ReportGenerated { student: Address, course_id: Symbol, report_period: String, start_date: u64, end_date: u64, sessions_count: u32 },
    BatchProcessed { batch_size: u32, processing_time: u64, updated_analytics: u32 },
    ConfigUpdated { admin: Address, config_type: String },
    DataAggregated { course_id: Symbol, date: u64, active_students: u32, total_sessions: u32 },
    TrendChange { student: Address, course_id: Symbol, old_trend: String, new_trend: String },
    StreakMilestone { student: Address, course_id: Symbol, streak_days: u32, milestone_type: String },
}

/// Token event data
#[derive(Clone, Debug)]
pub enum TokenEventData {
    TokensTransferred { from: Address, to: Address, amount: i128 },
    TokensMinted { to: Address, amount: i128 },
    TokensBurned { from: Address, amount: i128 },
    IncentiveEarned { student: Address, course_id: Symbol, amount: i128, reason: String },
    RewardClaimed { student: Address, amount: i128, reward_type: String },
    EventCreated { event_id: Symbol, multiplier: u32, start_date: u64, end_date: u64 },
    EventEnded { event_id: Symbol, participants: u32, total_rewards: i128 },
}

/// Progress event data
#[derive(Clone, Debug)]
pub enum ProgressEventData {
    ProgressUpdated { student: Address, course_id: Symbol, module_id: Symbol, progress_percentage: u32 },
    ModuleCompleted { student: Address, course_id: Symbol, module_id: Symbol, completion_time: u64 },
    CourseCompleted { student: Address, course_id: Symbol, completion_time: u64, final_score: Option<u32> },
    ProgressReset { student: Address, course_id: Symbol, reset_by: Address, reason: String },
}

/// System event data
#[derive(Clone, Debug)]
pub enum SystemEventData {
    ContractInitialized { admin: Address, config: String },
    ContractUpgraded { admin: Address, old_version: String, new_version: String },
    ConfigurationChanged { admin: Address, setting: String, old_value: String, new_value: String },
    MaintenanceMode { enabled: bool, admin: Address, reason: String },
    ProxyInitialized { admin: Address, implementation: Address },
    ProxyUpgraded { admin: Address, new_impl: Address },
    ProxyRollback { admin: Address, prev_impl: Address },
}

/// Error event data
#[derive(Clone, Debug)]
pub enum ErrorEventData {
    ValidationError { function: String, error_code: u32, error_message: String, context: Option<String> },
    PermissionDenied { user: Address, required_permission: String, attempted_action: String },
    ResourceNotFound { resource_type: String, resource_id: String },
    InvalidInput { function: String, parameter: String, provided_value: String, expected_format: String },
    SystemError { function: String, error_code: u32, error_message: String },
}

/// Multisig event data
#[derive(Clone, Debug)]
pub enum MultisigEventData {
    RequestCreated { request_id: BytesN<32>, requester: Address, course_id: String },
    ApprovalGranted { request_id: BytesN<32>, approver: Address, current_approvals: u32, required_approvals: u32 },
    RequestRejected { request_id: BytesN<32>, rejector: Address, reason: String },
    RequestApproved { request_id: BytesN<32>, certificate_id: BytesN<32>, final_approvals: u32 },
    RequestExpired { request_id: BytesN<32>, certificate_id: BytesN<32> },
    CertificateIssued { request_id: BytesN<32>, certificate_id: BytesN<32>, student: Address, approvers_count: u32 },
    ConfigUpdated { course_id: String, admin: Address, required_approvals: u32, approvers_count: u32 },
}

/// Prerequisite event data
#[derive(Clone, Debug)]
pub enum PrerequisiteEventData {
    PrerequisiteDefined { course_id: String, admin: Address, prerequisite_count: u32 },
    PrerequisiteChecked { student: Address, course_id: String, eligible: bool, missing_count: u32 },
    OverrideGranted { student: Address, course_id: String, admin: Address, reason: String },
    OverrideRevoked { student: Address, course_id: String, admin: Address, reason: String },
    Violation { student: Address, course_id: String, attempted_by: Address, missing_count: u32 },
    LearningPathGenerated { student: Address, target_course: String, path_length: u32, estimated_time: u64 },
    EnrollmentValidated { student: Address, course_id: String, enrolled_by: Address, validation_result: bool },
}

impl StandardEvent {
    /// Create a new standard event
    pub fn new(
        env: &Env,
        contract: Symbol,
        actor: Address,
        event_data: EventData,
    ) -> Self {
        // Generate a pseudo tx_hash from ledger sequence and timestamp
        let ledger_seq = env.ledger().sequence();
        let timestamp = env.ledger().timestamp();
        let mut hash_data = [0u8; 32];
        
        // Simple hash generation from ledger sequence and timestamp
        let seq_bytes = ledger_seq.to_be_bytes();
        let time_bytes = timestamp.to_be_bytes();
        
        for i in 0..8 {
            hash_data[i] = seq_bytes[i % 4];
            hash_data[i + 8] = time_bytes[i % 8];
            hash_data[i + 16] = seq_bytes[i % 4] ^ time_bytes[i % 8];
            hash_data[i + 24] = time_bytes[i % 8];
        }
        
        Self {
            version: EVENT_SCHEMA_VERSION,
            contract,
            actor,
            timestamp,
            tx_hash: BytesN::from_array(env, &hash_data),
            event_data,
        }
    }

    /// Emit the event to the Soroban event system
    pub fn emit(&self, env: &Env) {
        let category = self.get_category();
        let event_type = self.get_event_type();
        
        // Create standardized topics
        let topics = (
            Symbol::new(env, "standard_event"),
            self.contract.clone(),
            Symbol::new(env, &category),
            Symbol::new(env, &event_type),
            self.actor.clone(),
        );

        // Create standardized data
        let data = (
            self.version,
            self.timestamp,
            self.tx_hash.clone(),
            self.serialize_event_data(env),
        );

        env.events().publish(topics, data);
    }

    /// Get the event category as a string
    fn get_category(&self) -> String {
        match &self.event_data {
            EventData::AccessControl(_) => "access_control".to_string(),
            EventData::Certificate(_) => "certificate".to_string(),
            EventData::Analytics(_) => "analytics".to_string(),
            EventData::Token(_) => "token".to_string(),
            EventData::Progress(_) => "progress".to_string(),
            EventData::System(_) => "system".to_string(),
            EventData::Error(_) => "error".to_string(),
        }
    }

    /// Get the specific event type as a string
    fn get_event_type(&self) -> String {
        match &self.event_data {
            EventData::AccessControl(data) => match data {
                AccessControlEventData::ContractInitialized { .. } => "contract_initialized".to_string(),
                AccessControlEventData::RoleGranted { .. } => "role_granted".to_string(),
                AccessControlEventData::RoleRevoked { .. } => "role_revoked".to_string(),
                AccessControlEventData::RoleTransferred { .. } => "role_transferred".to_string(),
                AccessControlEventData::RoleUpdated { .. } => "role_updated".to_string(),
                AccessControlEventData::PermissionGranted { .. } => "permission_granted".to_string(),
                AccessControlEventData::PermissionRevoked { .. } => "permission_revoked".to_string(),
                AccessControlEventData::AdminChanged { .. } => "admin_changed".to_string(),
                AccessControlEventData::RoleExpired { .. } => "role_expired".to_string(),
                AccessControlEventData::AccessDenied { .. } => "access_denied".to_string(),
                AccessControlEventData::HierarchyViolation { .. } => "hierarchy_violation".to_string(),
            },
            EventData::Certificate(data) => match data {
                CertificateEventData::CertificateMinted { .. } => "certificate_minted".to_string(),
                CertificateEventData::CertificateRevoked { .. } => "certificate_revoked".to_string(),
                CertificateEventData::CertificateTransferred { .. } => "certificate_transferred".to_string(),
                CertificateEventData::MetadataUpdated { .. } => "metadata_updated".to_string(),
                CertificateEventData::RenewalRequested { .. } => "renewal_requested".to_string(),
                CertificateEventData::RenewalApproved { .. } => "renewal_approved".to_string(),
                CertificateEventData::RenewalRejected { .. } => "renewal_rejected".to_string(),
                CertificateEventData::CertificateExtended { .. } => "certificate_extended".to_string(),
                CertificateEventData::CertificateExpired { .. } => "certificate_expired".to_string(),
                CertificateEventData::CertificateAutoRenewed { .. } => "certificate_auto_renewed".to_string(),
                CertificateEventData::ExpiryNotification { .. } => "expiry_notification".to_string(),
                CertificateEventData::NotificationAcknowledged { .. } => "notification_acknowledged".to_string(),
                CertificateEventData::BatchMintCompleted { .. } => "batch_mint_completed".to_string(),
                CertificateEventData::IssuerAdded { .. } => "issuer_added".to_string(),
                CertificateEventData::IssuerRemoved { .. } => "issuer_removed".to_string(),
            },
            EventData::Analytics(data) => match data {
                AnalyticsEventData::SessionRecorded { .. } => "session_recorded".to_string(),
                AnalyticsEventData::SessionCompleted { .. } => "session_completed".to_string(),
                AnalyticsEventData::ProgressUpdated { .. } => "progress_updated".to_string(),
                AnalyticsEventData::CourseAnalyticsUpdated { .. } => "course_analytics_updated".to_string(),
                AnalyticsEventData::ModuleAnalyticsUpdated { .. } => "module_analytics_updated".to_string(),
                AnalyticsEventData::AchievementEarned { .. } => "achievement_earned".to_string(),
                AnalyticsEventData::LeaderboardUpdated { .. } => "leaderboard_updated".to_string(),
                AnalyticsEventData::ReportGenerated { .. } => "report_generated".to_string(),
                AnalyticsEventData::BatchProcessed { .. } => "batch_processed".to_string(),
                AnalyticsEventData::ConfigUpdated { .. } => "config_updated".to_string(),
                AnalyticsEventData::DataAggregated { .. } => "data_aggregated".to_string(),
                AnalyticsEventData::TrendChange { .. } => "trend_change".to_string(),
                AnalyticsEventData::StreakMilestone { .. } => "streak_milestone".to_string(),
            },
            EventData::Token(data) => match data {
                TokenEventData::TokensTransferred { .. } => "tokens_transferred".to_string(),
                TokenEventData::TokensMinted { .. } => "tokens_minted".to_string(),
                TokenEventData::TokensBurned { .. } => "tokens_burned".to_string(),
                TokenEventData::IncentiveEarned { .. } => "incentive_earned".to_string(),
                TokenEventData::RewardClaimed { .. } => "reward_claimed".to_string(),
                TokenEventData::EventCreated { .. } => "event_created".to_string(),
                TokenEventData::EventEnded { .. } => "event_ended".to_string(),
            },
            EventData::Progress(data) => match data {
                ProgressEventData::ProgressUpdated { .. } => "progress_updated".to_string(),
                ProgressEventData::ModuleCompleted { .. } => "module_completed".to_string(),
                ProgressEventData::CourseCompleted { .. } => "course_completed".to_string(),
                ProgressEventData::ProgressReset { .. } => "progress_reset".to_string(),
            },
            EventData::System(data) => match data {
                SystemEventData::ContractInitialized { .. } => "contract_initialized".to_string(),
                SystemEventData::ContractUpgraded { .. } => "contract_upgraded".to_string(),
                SystemEventData::ConfigurationChanged { .. } => "configuration_changed".to_string(),
                SystemEventData::MaintenanceMode { .. } => "maintenance_mode".to_string(),
                SystemEventData::ProxyInitialized { .. } => "proxy_initialized".to_string(),
                SystemEventData::ProxyUpgraded { .. } => "proxy_upgraded".to_string(),
                SystemEventData::ProxyRollback { .. } => "proxy_rollback".to_string(),
            },
            EventData::Error(data) => match data {
                ErrorEventData::ValidationError { .. } => "validation_error".to_string(),
                ErrorEventData::PermissionDenied { .. } => "permission_denied".to_string(),
                ErrorEventData::ResourceNotFound { .. } => "resource_not_found".to_string(),
                ErrorEventData::InvalidInput { .. } => "invalid_input".to_string(),
                ErrorEventData::SystemError { .. } => "system_error".to_string(),
            },
        }
    }

    /// Serialize event data for emission (simplified for now)
    fn serialize_event_data(&self, env: &Env) -> String {
        // For now, return a simple string representation
        // In a full implementation, this would serialize to a structured format
        match &self.event_data {
            EventData::AccessControl(_) => String::from_str(env, "access_control_event"),
            EventData::Certificate(_) => String::from_str(env, "certificate_event"),
            EventData::Analytics(_) => String::from_str(env, "analytics_event"),
            EventData::Token(_) => String::from_str(env, "token_event"),
            EventData::Progress(_) => String::from_str(env, "progress_event"),
            EventData::System(_) => String::from_str(env, "system_event"),
            EventData::Error(_) => String::from_str(env, "error_event"),
        }
    }
}

/// Helper macros for easy event emission
#[macro_export]
macro_rules! emit_access_control_event {
    ($env:expr, $contract:expr, $actor:expr, $data:expr) => {
        StandardEvent::new($env, $contract, $actor, EventData::AccessControl($data)).emit($env)
    };
}

#[macro_export]
macro_rules! emit_certificate_event {
    ($env:expr, $contract:expr, $actor:expr, $data:expr) => {
        StandardEvent::new($env, $contract, $actor, EventData::Certificate($data)).emit($env)
    };
}

#[macro_export]
macro_rules! emit_analytics_event {
    ($env:expr, $contract:expr, $actor:expr, $data:expr) => {
        StandardEvent::new($env, $contract, $actor, EventData::Analytics($data)).emit($env)
    };
}

#[macro_export]
macro_rules! emit_token_event {
    ($env:expr, $contract:expr, $actor:expr, $data:expr) => {
        StandardEvent::new($env, $contract, $actor, EventData::Token($data)).emit($env)
    };
}

#[macro_export]
macro_rules! emit_progress_event {
    ($env:expr, $contract:expr, $actor:expr, $data:expr) => {
        StandardEvent::new($env, $contract, $actor, EventData::Progress($data)).emit($env)
    };
}

#[macro_export]
macro_rules! emit_system_event {
    ($env:expr, $contract:expr, $actor:expr, $data:expr) => {
        StandardEvent::new($env, $contract, $actor, EventData::System($data)).emit($env)
    };
}

#[macro_export]
macro_rules! emit_error_event {
    ($env:expr, $contract:expr, $actor:expr, $data:expr) => {
        StandardEvent::new($env, $contract, $actor, EventData::Error($data)).emit($env)
    };
}
