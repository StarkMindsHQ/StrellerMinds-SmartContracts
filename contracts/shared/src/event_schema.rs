use soroban_sdk::{contracttype, Address, BytesN, Env, String, Symbol, Vec};

/// Standard event schema version
pub const EVENT_SCHEMA_VERSION: u32 = 1;

/// Standard event wrapper that all contracts should use
#[contracttype]
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
    /// Event sequence number for ordering guarantees
    pub sequence: Option<u32>,
    /// Event-specific data
    pub event_data: EventData,
}

/// Event categories for better organization and filtering
#[contracttype]
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
    Err,
}

/// Standardized event data types
#[contracttype]
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
    Err(ErrorEventData),
}

// Access Control Event Structs
#[contracttype]
#[derive(Clone, Debug)]
pub struct ContractInitializedEvent {
    pub admin: Address,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct RoleGrantedEvent {
    pub granter: Address,
    pub user: Address,
    pub role_level: u32,
    pub granted_at: u64,
    pub expires_at: Option<u64>,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct RoleRevokedEvent {
    pub revoker: Address,
    pub user: Address,
    pub role_level: u32,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct RoleTransferredEvent {
    pub from: Address,
    pub to: Address,
    pub role_level: u32,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct RoleUpdatedEvent {
    pub updater: Address,
    pub user: Address,
    pub role_level: u32,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct PermissionGrantedEvent {
    pub granter: Address,
    pub user: Address,
    pub permission: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct PermissionRevokedEvent {
    pub revoker: Address,
    pub user: Address,
    pub permission: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct DynamicPermissionGrantedEvent {
    pub granter: Address,
    pub user: Address,
    pub permission: Symbol,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct RoleInheritanceUpdatedEvent {
    pub updater: Address,
    pub user: Address,
    pub inherited_roles: Vec<u32>,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct TemplateCreatedEvent {
    pub admin: Address,
    pub template_id: Symbol,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct AdminChangedEvent {
    pub old_admin: Address,
    pub new_admin: Address,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct RoleExpiredEvent {
    pub user: Address,
    pub role_level: u32,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct AccessDeniedEvent {
    pub user: Address,
    pub permission: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct HierarchyViolationEvent {
    pub granter: Address,
    pub target: Address,
    pub target_level: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum AccessControlEventData {
    ContractInitialized(ContractInitializedEvent),
    RoleGranted(RoleGrantedEvent),
    RoleRevoked(RoleRevokedEvent),
    RoleTransferred(RoleTransferredEvent),
    RoleUpdated(RoleUpdatedEvent),
    PermissionGranted(PermissionGrantedEvent),
    PermissionRevoked(PermissionRevokedEvent),
    DynamicPermissionGranted(DynamicPermissionGrantedEvent),
    RoleInheritanceUpdated(RoleInheritanceUpdatedEvent),
    TemplateCreated(TemplateCreatedEvent),
    AdminChanged(AdminChangedEvent),
    RoleExpired(RoleExpiredEvent),
    AccessDenied(AccessDeniedEvent),
    HierarchyViolation(HierarchyViolationEvent),
}

// Certificate Event Structs
#[contracttype]
#[derive(Clone, Debug)]
pub struct CertificateMintedEvent {
    pub certificate_id: BytesN<32>,
    pub student: Address,
    pub issuer: Address,
    pub token_id: BytesN<32>,
    pub metadata_hash: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct CertificateRevokedEvent {
    pub certificate_id: BytesN<32>,
    pub revoker: Address,
    pub reason: Option<String>,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct CertificateTransferredEvent {
    pub certificate_id: BytesN<32>,
    pub from: Address,
    pub to: Address,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct MetadataUpdatedEvent {
    pub certificate_id: BytesN<32>,
    pub updater: Address,
    pub old_uri: String,
    pub new_uri: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct RenewalRequestedEvent {
    pub certificate_id: BytesN<32>,
    pub requester: Address,
    pub requested_extension: u64,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct RenewalApprovedEvent {
    pub certificate_id: BytesN<32>,
    pub approver: Address,
    pub requester: Address,
    pub extension_period: u64,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct RenewalRejectedEvent {
    pub certificate_id: BytesN<32>,
    pub approver: Address,
    pub requester: Address,
    pub reason: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct CertificateExtendedEvent {
    pub certificate_id: BytesN<32>,
    pub admin: Address,
    pub owner: Address,
    pub extension_period: u64,
    pub reason: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct CertificateExpiredEvent {
    pub certificate_id: BytesN<32>,
    pub owner: Address,
    pub expiry_date: u64,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct CertificateAutoRenewedEvent {
    pub certificate_id: BytesN<32>,
    pub owner: Address,
    pub new_expiry_date: u64,
    pub renewal_count: u32,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct ExpiryNotificationEvent {
    pub certificate_id: BytesN<32>,
    pub owner: Address,
    pub notification_type: String,
    pub expiry_date: u64,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct NotificationAcknowledgedEvent {
    pub certificate_id: BytesN<32>,
    pub user: Address,
    pub notification_type: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct BatchMintCompletedEvent {
    pub issuer: Address,
    pub total_count: u32,
    pub success_count: u32,
    pub failure_count: u32,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct IssuerAddedEvent {
    pub admin: Address,
    pub issuer: Address,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct IssuerRemovedEvent {
    pub admin: Address,
    pub issuer: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum CertificateEventData {
    CertificateMinted(CertificateMintedEvent),
    CertificateRevoked(CertificateRevokedEvent),
    CertificateTransferred(CertificateTransferredEvent),
    MetadataUpdated(MetadataUpdatedEvent),
    RenewalRequested(RenewalRequestedEvent),
    RenewalApproved(RenewalApprovedEvent),
    RenewalRejected(RenewalRejectedEvent),
    CertificateExtended(CertificateExtendedEvent),
    CertificateExpired(CertificateExpiredEvent),
    CertificateAutoRenewed(CertificateAutoRenewedEvent),
    ExpiryNotification(ExpiryNotificationEvent),
    NotificationAcknowledged(NotificationAcknowledgedEvent),
    BatchMintCompleted(BatchMintCompletedEvent),
    IssuerAdded(IssuerAddedEvent),
    IssuerRemoved(IssuerRemovedEvent),
}

/// Analytics event data
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AnalyticsEventData {
    SessionRecorded(BytesN<32>, Address, Symbol, Symbol, String, u64, u32), // session_id, student, course_id, module_id, session_type, time_spent, completion_percentage
    SessionCompleted(BytesN<32>, Address, Symbol, Symbol, Option<u32>, u64), // session_id, student, course_id, module_id, final_score, total_time
    ProgressUpdated(Address, Symbol, u32, u64, String), // student, course_id, completion_percentage, total_time_spent, performance_trend
    CourseAnalyticsUpdated(Symbol, u32, u32, Option<u32>), // course_id, total_students, completion_rate, average_score
    ModuleAnalyticsUpdated(Symbol, Symbol, u32, u64, String), // course_id, module_id, completion_rate, average_time, difficulty_rating
    AchievementEarned(Address, Symbol, String, Symbol, u64), // student, achievement_id, achievement_type, course_id, earned_date
    LeaderboardUpdated(Symbol, String, Address, u32, u32), // course_id, metric_type, top_student, top_score, total_entries
    ReportGenerated(Address, Symbol, String, u64, u64, u32), // student, course_id, report_period, start_date, end_date, sessions_count
    BatchProcessed(u32, u64, u32), // batch_size, processing_time, updated_analytics
    ConfigUpdated(Address, String), // admin, config_type
    DataAggregated(Symbol, u64, u32, u32), // course_id, date, active_students, total_sessions
    TrendChange(Address, Symbol, String, String), // student, course_id, old_trend, new_trend
    StreakMilestone(Address, Symbol, u32, String), // student, course_id, streak_days, milestone_type
    InsightRequested(Address, Symbol, String),     // student, course_id, insight_type
    InsightReceived(Address, BytesN<32>, String, String, u64), // student, insight_id, insight_type, content, timestamp
}

// Token Event Structs
#[contracttype]
#[derive(Clone, Debug)]
pub struct TokensTransferredEvent {
    pub from: Address,
    pub to: Address,
    pub amount: i128,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct TokensMintedEvent {
    pub to: Address,
    pub amount: i128,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct TokensBurnedEvent {
    pub from: Address,
    pub amount: i128,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct IncentiveEarnedEvent {
    pub student: Address,
    pub course_id: Symbol,
    pub amount: i128,
    pub reason: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct RewardClaimedEvent {
    pub student: Address,
    pub amount: i128,
    pub reward_type: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct EventCreatedEvent {
    pub event_id: Symbol,
    pub multiplier: u32,
    pub start_date: u64,
    pub end_date: u64,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct EventEndedEvent {
    pub event_id: Symbol,
    pub participants: u32,
    pub total_rewards: i128,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum TokenEventData {
    TokensTransferred(TokensTransferredEvent),
    TokensMinted(TokensMintedEvent),
    TokensBurned(TokensBurnedEvent),
    IncentiveEarned(IncentiveEarnedEvent),
    RewardClaimed(RewardClaimedEvent),
    EventCreated(EventCreatedEvent),
    EventEnded(EventEndedEvent),
}

// Progress Event Structs
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProgressUpdatedEvent {
    pub student: Address,
    pub course_id: Symbol,
    pub module_id: Symbol,
    pub progress_percentage: u32,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct ModuleCompletedEvent {
    pub student: Address,
    pub course_id: Symbol,
    pub module_id: Symbol,
    pub completion_time: u64,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct CourseCompletedEvent {
    pub student: Address,
    pub course_id: Symbol,
    pub completion_time: u64,
    pub final_score: Option<u32>,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProgressResetEvent {
    pub student: Address,
    pub course_id: Symbol,
    pub reset_by: Address,
    pub reason: String,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum ProgressEventData {
    ProgressUpdated(ProgressUpdatedEvent),
    ModuleCompleted(ModuleCompletedEvent),
    CourseCompleted(CourseCompletedEvent),
    ProgressReset(ProgressResetEvent),
}

// System Event Structs
#[contracttype]
#[derive(Clone, Debug)]
pub struct ContractInitializedSystemEvent {
    pub admin: Address,
    pub config: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct ContractUpgradedEvent {
    pub admin: Address,
    pub old_version: String,
    pub new_version: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct ConfigurationChangedEvent {
    pub admin: Address,
    pub setting: String,
    pub old_value: String,
    pub new_value: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct MaintenanceModeEvent {
    pub enabled: bool,
    pub admin: Address,
    pub reason: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProxyInitializedEvent {
    pub admin: Address,
    pub implementation: Address,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProxyUpgradedEvent {
    pub admin: Address,
    pub new_impl: Address,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct ProxyRollbackEvent {
    pub admin: Address,
    pub prev_impl: Address,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum SystemEventData {
    ContractInitialized(ContractInitializedSystemEvent),
    ContractUpgraded(ContractUpgradedEvent),
    ConfigurationChanged(ConfigurationChangedEvent),
    MaintenanceMode(MaintenanceModeEvent),
    ProxyInitialized(ProxyInitializedEvent),
    ProxyUpgraded(ProxyUpgradedEvent),
    ProxyRollback(ProxyRollbackEvent),
}

// Error Event Structs
#[contracttype]
#[derive(Clone, Debug)]
pub struct ValidationErrorEvent {
    pub function: String,
    pub error_code: u32,
    pub error_message: String,
    pub context: Option<String>,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct PermissionDeniedEvent {
    pub user: Address,
    pub required_permission: String,
    pub attempted_action: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct ResourceNotFoundEvent {
    pub resource_type: String,
    pub resource_id: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct InvalidInputEvent {
    pub function: String,
    pub parameter: String,
    pub provided_value: String,
    pub expected_format: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct SystemErrorEvent {
    pub function: String,
    pub error_code: u32,
    pub error_message: String,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum ErrorEventData {
    ValidationError(ValidationErrorEvent),
    PermissionDenied(PermissionDeniedEvent),
    ResourceNotFound(ResourceNotFoundEvent),
    InvalidInput(InvalidInputEvent),
    SystemError(SystemErrorEvent),
}

// Multisig Event Structs
#[contracttype]
#[derive(Clone, Debug)]
pub struct RequestCreatedEvent {
    pub request_id: BytesN<32>,
    pub requester: Address,
    pub course_id: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct ApprovalGrantedEvent {
    pub request_id: BytesN<32>,
    pub approver: Address,
    pub current_approvals: u32,
    pub required_approvals: u32,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct RequestRejectedEvent {
    pub request_id: BytesN<32>,
    pub rejector: Address,
    pub reason: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct RequestApprovedEvent {
    pub request_id: BytesN<32>,
    pub certificate_id: BytesN<32>,
    pub final_approvals: u32,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct RequestExpiredEvent {
    pub request_id: BytesN<32>,
    pub certificate_id: BytesN<32>,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct CertificateIssuedEvent {
    pub request_id: BytesN<32>,
    pub certificate_id: BytesN<32>,
    pub student: Address,
    pub approvers_count: u32,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct ConfigUpdatedEvent {
    pub course_id: String,
    pub admin: Address,
    pub required_approvals: u32,
    pub approvers_count: u32,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum MultisigEventData {
    RequestCreated(RequestCreatedEvent),
    ApprovalGranted(ApprovalGrantedEvent),
    RequestRejected(RequestRejectedEvent),
    RequestApproved(RequestApprovedEvent),
    RequestExpired(RequestExpiredEvent),
    CertificateIssued(CertificateIssuedEvent),
    ConfigUpdated(ConfigUpdatedEvent),
}

// Prerequisite Event Structs
#[contracttype]
#[derive(Clone, Debug)]
pub struct PrerequisiteDefinedEvent {
    pub course_id: String,
    pub admin: Address,
    pub prerequisite_count: u32,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct PrerequisiteCheckedEvent {
    pub student: Address,
    pub course_id: String,
    pub eligible: bool,
    pub missing_count: u32,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct OverrideGrantedEvent {
    pub student: Address,
    pub course_id: String,
    pub admin: Address,
    pub reason: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct OverrideRevokedEvent {
    pub student: Address,
    pub course_id: String,
    pub admin: Address,
    pub reason: String,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct ViolationEvent {
    pub student: Address,
    pub course_id: String,
    pub attempted_by: Address,
    pub missing_count: u32,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct LearningPathGeneratedEvent {
    pub student: Address,
    pub target_course: String,
    pub path_length: u32,
    pub estimated_time: u64,
}
#[contracttype]
#[derive(Clone, Debug)]
pub struct EnrollmentValidatedEvent {
    pub student: Address,
    pub course_id: String,
    pub enrolled_by: Address,
    pub validation_result: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub enum PrerequisiteEventData {
    PrerequisiteDefined(PrerequisiteDefinedEvent),
    PrerequisiteChecked(PrerequisiteCheckedEvent),
    OverrideGranted(OverrideGrantedEvent),
    OverrideRevoked(OverrideRevokedEvent),
    Violation(ViolationEvent),
    LearningPathGenerated(LearningPathGeneratedEvent),
    EnrollmentValidated(EnrollmentValidatedEvent),
}

impl StandardEvent {
    /// Create a new standard event
    pub fn new(env: &Env, contract: Symbol, actor: Address, event_data: EventData) -> Self {
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
            sequence: None, // Will be set by publisher
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
            Symbol::new(env, category),
            Symbol::new(env, event_type),
            self.actor.clone(),
        );

        // Create standardized data
        let data = (
            self.version,
            self.timestamp,
            self.tx_hash.clone(),
            self.sequence.unwrap_or(0),
            self.serialize_event_data(env),
        );

        env.events().publish(topics, data);
    }

    /// Get the event category as a string
    pub fn get_category(&self) -> &'static str {
        match &self.event_data {
            EventData::AccessControl(_) => "access_control",
            EventData::Certificate(_) => "certificate",
            EventData::Analytics(_) => "analytics",
            EventData::Token(_) => "token",
            EventData::Progress(_) => "progress",
            EventData::System(_) => "system",
            EventData::Err(_) => "error",
        }
    }

    /// Get the specific event type as a string
    pub fn get_event_type(&self) -> &'static str {
        match &self.event_data {
            EventData::AccessControl(data) => match data {
                AccessControlEventData::ContractInitialized(_) => "contract_initialized",
                AccessControlEventData::RoleGranted(_) => "role_granted",
                AccessControlEventData::RoleRevoked(_) => "role_revoked",
                AccessControlEventData::RoleTransferred(_) => "role_transferred",
                AccessControlEventData::RoleUpdated(_) => "role_updated",
                AccessControlEventData::PermissionGranted(_) => "permission_granted",
                AccessControlEventData::PermissionRevoked(_) => "permission_revoked",
                AccessControlEventData::AdminChanged(_) => "admin_changed",
                AccessControlEventData::RoleExpired(_) => "role_expired",
                AccessControlEventData::AccessDenied(_) => "access_denied",
                AccessControlEventData::HierarchyViolation(_) => "hierarchy_violation",
                AccessControlEventData::DynamicPermissionGranted(_) => "dynamic_permission_granted",
                AccessControlEventData::RoleInheritanceUpdated(_) => "role_inheritance_updated",
                AccessControlEventData::TemplateCreated(_) => "template_created",
            },
            EventData::Certificate(data) => match data {
                CertificateEventData::CertificateMinted(_) => "certificate_minted",
                CertificateEventData::CertificateRevoked(_) => "certificate_revoked",
                CertificateEventData::CertificateTransferred(_) => "certificate_transferred",
                CertificateEventData::MetadataUpdated(_) => "metadata_updated",
                CertificateEventData::RenewalRequested(_) => "renewal_requested",
                CertificateEventData::RenewalApproved(_) => "renewal_approved",
                CertificateEventData::RenewalRejected(_) => "renewal_rejected",
                CertificateEventData::CertificateExtended(_) => "certificate_extended",
                CertificateEventData::CertificateExpired(_) => "certificate_expired",
                CertificateEventData::CertificateAutoRenewed(_) => "certificate_auto_renewed",
                CertificateEventData::ExpiryNotification(_) => "expiry_notification",
                CertificateEventData::NotificationAcknowledged(_) => "notification_acknowledged",
                CertificateEventData::BatchMintCompleted(_) => "batch_mint_completed",
                CertificateEventData::IssuerAdded(_) => "issuer_added",
                CertificateEventData::IssuerRemoved(_) => "issuer_removed",
            },
            EventData::Analytics(data) => match data {
                AnalyticsEventData::SessionRecorded(..) => "session_recorded",
                AnalyticsEventData::SessionCompleted(..) => "session_completed",
                AnalyticsEventData::ProgressUpdated(..) => "progress_updated",
                AnalyticsEventData::CourseAnalyticsUpdated(..) => "course_analytics_updated",
                AnalyticsEventData::ModuleAnalyticsUpdated(..) => "module_analytics_updated",
                AnalyticsEventData::AchievementEarned(..) => "achievement_earned",
                AnalyticsEventData::LeaderboardUpdated(..) => "leaderboard_updated",
                AnalyticsEventData::ReportGenerated(..) => "report_generated",
                AnalyticsEventData::BatchProcessed(..) => "batch_processed",
                AnalyticsEventData::ConfigUpdated(..) => "config_updated",
                AnalyticsEventData::DataAggregated(..) => "data_aggregated",
                AnalyticsEventData::TrendChange(..) => "trend_change",
                AnalyticsEventData::StreakMilestone(..) => "streak_milestone",
                AnalyticsEventData::InsightRequested(..) => "insight_requested",
                AnalyticsEventData::InsightReceived(..) => "insight_received",
            },
            EventData::Token(data) => match data {
                TokenEventData::TokensTransferred(_) => "tokens_transferred",
                TokenEventData::TokensMinted(_) => "tokens_minted",
                TokenEventData::TokensBurned(_) => "tokens_burned",
                TokenEventData::IncentiveEarned(_) => "incentive_earned",
                TokenEventData::RewardClaimed(_) => "reward_claimed",
                TokenEventData::EventCreated(_) => "event_created",
                TokenEventData::EventEnded(_) => "event_ended",
            },
            EventData::Progress(data) => match data {
                ProgressEventData::ProgressUpdated(_) => "progress_updated",
                ProgressEventData::ModuleCompleted(_) => "module_completed",
                ProgressEventData::CourseCompleted(_) => "course_completed",
                ProgressEventData::ProgressReset(_) => "progress_reset",
            },
            EventData::System(data) => match data {
                SystemEventData::ContractInitialized(_) => "contract_initialized",
                SystemEventData::ContractUpgraded(_) => "contract_upgraded",
                SystemEventData::ConfigurationChanged(_) => "configuration_changed",
                SystemEventData::MaintenanceMode(_) => "maintenance_mode",
                SystemEventData::ProxyInitialized(_) => "proxy_initialized",
                SystemEventData::ProxyUpgraded(_) => "proxy_upgraded",
                SystemEventData::ProxyRollback(_) => "proxy_rollback",
            },
            EventData::Err(data) => match data {
                ErrorEventData::ValidationError(_) => "validation_error",
                ErrorEventData::PermissionDenied(_) => "permission_denied",
                ErrorEventData::ResourceNotFound(_) => "resource_not_found",
                ErrorEventData::InvalidInput(_) => "invalid_input",
                ErrorEventData::SystemError(_) => "system_error",
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
            EventData::Err(_) => String::from_str(env, "error_event"),
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
        StandardEvent::new($env, $contract, $actor, EventData::Err($data)).emit($env)
    };
}
