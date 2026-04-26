use shared::config::{ContractConfig, DeploymentEnv};
use soroban_sdk::{contracttype, Address, BytesN, String, Symbol, Vec};

/// Common aliases used across the security monitor to keep repetitive Soroban
/// collection and identifier types readable.
pub type ThreatId = BytesN<32>;
pub type ThreatIdList = Vec<ThreatId>;
pub type RiskFactorList = Vec<Symbol>;

/// Security threat severity levels
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[contracttype]
pub enum ThreatLevel {
    Low = 1,
    Medium = 2,
    High = 3,
    Critical = 4,
}

/// Types of security threats that can be detected
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ThreatType {
    BurstActivity,          // Spike in events
    AnomalousActor,         // Unusual actor behavior
    ErrorRateSpike,         // High error rate
    SequenceIntegrityIssue, // Event sequence problems
    AccessViolation,        // RBAC violations
    ReentrancyAttempt,      // Potential reentrancy
    ValidationFailure,      // Input validation issues
    RateLimitExceeded,      // Rate limit violations
    BehavioralAnomaly,      // Detected by AI oracle
    CredentialFraud,        // Detected during verification/login
    BiometricFailure,       // Continuous authentication failed
    KnownMaliciousActor,    // Flagged by threat intelligence
}

/// Automated mitigation actions
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MitigationAction {
    RateLimitApplied,
    CircuitBreakerTriggered,
    AccessRestricted,
    AlertSent,
    NoAction,
    RequireReauth,
    LockAccount,
}

/// Security threat detection record
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityThreat {
    pub threat_id: ThreatId,
    pub threat_type: ThreatType,
    pub threat_level: ThreatLevel,
    pub detected_at: u64,
    pub contract: Symbol,
    pub actor: Option<Address>,
    pub description: String,
    pub metric_value: u32,    // The metric that triggered detection
    pub threshold_value: u32, // The threshold that was exceeded
    pub auto_mitigated: bool,
    pub mitigation_action: MitigationAction, // Use NoAction variant instead of None
}

/// Security metrics for a time window
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SecurityMetrics {
    pub window_id: u64,
    pub contract: Symbol,
    pub start_time: u64,
    pub end_time: u64,
    pub total_events: u32,
    pub error_events: u32,
    pub error_rate: u32, // Percentage
    pub unique_actors: u32,
    pub access_violations: u32,
    pub threat_count: u32,
    pub highest_threat_level: ThreatLevel,
    pub security_score: u32, // 0-100
}

/// Circuit breaker states
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum BreakerState {
    Closed,   // Normal operation
    Open,     // Blocking calls
    HalfOpen, // Testing recovery
}

/// Circuit breaker state tracking
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CircuitBreakerState {
    pub contract: Symbol,
    pub function_name: Symbol,
    pub state: BreakerState,
    pub failure_count: u32,
    pub failure_threshold: u32,
    pub last_failure_time: u64,
    pub opened_at: Option<u64>,
    pub last_checked: u64,
    pub timeout_duration: u64, // How long to keep circuit open
}

impl CircuitBreakerState {
    pub fn new(contract: Symbol, function_name: Symbol, threshold: u32, timeout: u64) -> Self {
        Self {
            contract,
            function_name,
            state: BreakerState::Closed,
            failure_count: 0,
            failure_threshold: threshold,
            last_failure_time: 0,
            opened_at: None,
            last_checked: 0,
            timeout_duration: timeout,
        }
    }
}

/// Security configuration
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SecurityConfig {
    pub burst_detection_threshold: u32, // Events per window
    pub burst_window_seconds: u64,      // Time window for burst detection
    pub error_rate_threshold: u32,      // Percentage
    pub actor_anomaly_threshold: u32,   // Multiplier of normal behavior
    pub circuit_breaker_threshold: u32, // Failures before opening
    pub circuit_breaker_timeout: u64,   // Seconds to keep open
    pub auto_mitigation_enabled: bool,
    pub rate_limit_per_window: u32,
    pub rate_limit_window: u64,
}

impl SecurityConfig {
    pub fn default_config() -> Self {
        Self::for_env(DeploymentEnv::Production)
    }

    pub fn for_env(profile: DeploymentEnv) -> Self {
        let defaults = ContractConfig::security(profile);
        Self {
            burst_detection_threshold: defaults.burst_detection_threshold,
            burst_window_seconds: defaults.burst_window_seconds,
            error_rate_threshold: defaults.error_rate_threshold,
            actor_anomaly_threshold: defaults.actor_anomaly_threshold,
            circuit_breaker_threshold: defaults.circuit_breaker_threshold,
            circuit_breaker_timeout: defaults.circuit_breaker_timeout,
            auto_mitigation_enabled: defaults.auto_mitigation_enabled,
            rate_limit_per_window: defaults.rate_limit_per_window,
            rate_limit_window: defaults.rate_limit_window,
        }
    }

    pub fn validate(&self) -> Result<(), crate::errors::SecurityError> {
        if self.burst_detection_threshold == 0
            || self.burst_window_seconds == 0
            || self.circuit_breaker_threshold == 0
            || self.rate_limit_per_window == 0
            || self.rate_limit_window == 0
        {
            return Err(crate::errors::SecurityError::InvalidConfiguration);
        }

        Ok(())
    }
}

/// Security fix recommendation categories
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum RecommendationCategory {
    AccessControl,
    InputValidation,
    ReentrancyPrevention,
    RateLimiting,
    EventIntegrity,
    Configuration,
}

/// Security fix recommendation
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SecurityRecommendation {
    pub recommendation_id: BytesN<32>,
    pub threat_id: BytesN<32>,
    pub severity: ThreatLevel,
    pub category: RecommendationCategory,
    pub title: String,
    pub description: String,
    pub code_location: Option<String>,
    pub fix_suggestion: String,
    pub created_at: u64,
    pub acknowledged: bool,
}

/// Storage keys for the security monitor contract
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum SecurityDataKey {
    Config,
    Admin,
    Threat(ThreatId),                // threat_id
    ContractThreats(Symbol),         // contract -> ThreatIdList
    SecurityMetrics(Symbol, u64),    // (contract, window_id)
    CircuitBreaker(Symbol, Symbol),  // (contract, function)
    ActorEventCount(Address, u64),   // (actor, window_id)
    ActorRateLimit(Address, Symbol), // (actor, contract)
    ContractEventBaseline(Symbol),   // contract -> baseline metrics
    Recommendation(ThreatId),        // recommendation_id
    ThreatRecommendations(ThreatId), // threat_id -> ThreatIdList
    UserRiskScore(Address),          // user -> risk score data
    ThreatIntelligence(Symbol),      // indicator_type -> intel data
    TrainingStatus(Address),         // user -> training status
    IncidentReport(ThreatId),        // incident_id
    Oracle(Address),                 // Authorized oracle
    // RBAC keys
    RbacRole(Symbol),                 // role_id -> RbacRole
    RbacUserRoles(Address),           // user -> Vec<Symbol>
    RbacAssignment(Address, Symbol),  // (user, role_id) -> RoleAssignment
    RbacDelegations(Address, Symbol), // (delegator, role_id) -> Vec<RoleDelegation>
}

/// A role definition in the RBAC hierarchy.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RbacRole {
    /// Unique identifier for this role.
    pub role_id: Symbol,
    /// Optional parent role for hierarchy inheritance.
    pub parent_role: Option<Symbol>,
    /// Human-readable description of the role.
    pub description: String,
    /// Whether this role is currently active and assignable.
    pub is_active: bool,
    /// Unix timestamp when the role was created.
    pub created_at: u64,
}

/// An assignment of a role to a user.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RoleAssignment {
    /// The user who holds this role.
    pub user: Address,
    /// The role being assigned.
    pub role_id: Symbol,
    /// Address that performed the assignment.
    pub assigned_by: Address,
    /// Unix timestamp when the assignment was made.
    pub assigned_at: u64,
    /// Optional expiry timestamp; `None` means the assignment never expires.
    pub expires_at: Option<u64>,
    /// Whether this assignment is currently active.
    pub is_active: bool,
}

/// A delegation of a role from one user to another.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RoleDelegation {
    /// User who is delegating the role.
    pub delegator: Address,
    /// User receiving the delegated role.
    pub delegate: Address,
    /// The role being delegated.
    pub role_id: Symbol,
    /// Unix timestamp when the delegation was created.
    pub delegated_at: u64,
    /// Optional expiry timestamp for the delegation.
    pub expires_at: Option<u64>,
}

/// Persistent tracking for an actor's current rate-limit bucket.
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct RateLimitState {
    pub window_started_at: u64,
    pub event_count: u32,
    pub last_attempt_at: u64,
}

/// User Risk Score tracking
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UserRiskScore {
    pub score: u32, // 0-100, where 100 is maximum risk
    pub last_updated: u64,
    pub risk_factors: RiskFactorList, // e.g., "FailedLogin", "AnomalousBehavior"
}

/// Threat Intelligence data
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ThreatIntelligence {
    pub source: Symbol,         // e.g., "GlobalList", "PartnerAPI"
    pub indicator_type: Symbol, // e.g., "IP", "Address", "BehaviorPattern"
    pub indicator_value: String,
    pub threat_level: ThreatLevel,
    pub added_at: u64,
}

/// Incident Report for compliance
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct IncidentReport {
    pub incident_id: ThreatId,
    pub timestamp: u64,
    pub threat_ids: ThreatIdList,
    pub impact_summary: String,
    pub actions_taken: Vec<MitigationAction>,
    pub status: Symbol, // e.g., "Open", "Mitigated", "Resolved"
    pub resolved_at: Option<u64>,
}

/// Security Awareness Training tracking
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct SecurityTrainingStatus {
    pub user: Address,
    pub completed_modules: Vec<Symbol>,
    pub last_training_date: u64,
    pub score: u32, // Passed quiz score, etc.
}
