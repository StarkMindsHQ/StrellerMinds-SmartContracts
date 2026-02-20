use soroban_sdk::{contracttype, Address, BytesN, String, Symbol, Vec};

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
    pub threat_id: BytesN<32>,
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
        Self {
            burst_detection_threshold: 100, // 100 events
            burst_window_seconds: 60,       // in 60 seconds
            error_rate_threshold: 10,       // 10% error rate
            actor_anomaly_threshold: 10,    // 10x normal behavior
            circuit_breaker_threshold: 5,   // 5 failures
            circuit_breaker_timeout: 300,   // 5 minutes
            auto_mitigation_enabled: true,
            rate_limit_per_window: 100, // 100 events
            rate_limit_window: 3600,    // per hour
        }
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
    Threat(BytesN<32>),                // threat_id
    ContractThreats(Symbol),           // contract -> Vec<BytesN<32>>
    SecurityMetrics(Symbol, u64),      // (contract, window_id)
    CircuitBreaker(Symbol, Symbol),    // (contract, function)
    ActorEventCount(Address, u64),     // (actor, window_id)
    ContractEventBaseline(Symbol),     // contract -> baseline metrics
    Recommendation(BytesN<32>),        // recommendation_id
    ThreatRecommendations(BytesN<32>), // threat_id -> Vec<BytesN<32>>
    UserRiskScore(Address),            // user -> risk score data
    ThreatIntelligence(Symbol),        // indicator_type -> intel data
    TrainingStatus(Address),           // user -> training status
    IncidentReport(BytesN<32>),        // incident_id
    Oracle(Address),                   // Authorized oracle
}

/// User Risk Score tracking
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct UserRiskScore {
    pub score: u32,             // 0-100, where 100 is maximum risk
    pub last_updated: u64,
    pub risk_factors: Vec<Symbol>, // e.g., "FailedLogin", "AnomalousBehavior"
}

/// Threat Intelligence data
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ThreatIntelligence {
    pub source: Symbol,           // e.g., "GlobalList", "PartnerAPI"
    pub indicator_type: Symbol,   // e.g., "IP", "Address", "BehaviorPattern"
    pub indicator_value: String,
    pub threat_level: ThreatLevel,
    pub added_at: u64,
}

/// Incident Report for compliance
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct IncidentReport {
    pub incident_id: BytesN<32>,
    pub timestamp: u64,
    pub threat_ids: Vec<BytesN<32>>,
    pub impact_summary: String,
    pub actions_taken: Vec<MitigationAction>,
    pub status: Symbol,          // e.g., "Open", "Mitigated", "Resolved"
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
