use crate::errors::SecurityError;
use crate::types::{
    CircuitBreakerState, MitigationAction, SecurityConfig, SecurityMetrics, SecurityRecommendation,
    SecurityThreat, UserRiskScore, ThreatIntelligence
};
use soroban_sdk::{Address, BytesN, Env, Symbol, Vec};

/// Security Monitor contract trait
pub trait SecurityMonitorTrait {
    /// Initialize the security monitor contract
    fn initialize(env: Env, admin: Address, config: SecurityConfig) -> Result<(), SecurityError>;

    /// Scan for security threats in a time window
    fn scan_for_threats(
        env: Env,
        contract: Symbol,
        window_seconds: u64,
    ) -> Result<Vec<SecurityThreat>, SecurityError>;

    /// Get a specific threat by ID
    fn get_threat(env: Env, threat_id: BytesN<32>) -> Result<SecurityThreat, SecurityError>;

    /// Get all threats for a contract
    fn get_contract_threats(env: Env, contract: Symbol) -> Vec<BytesN<32>>;

    /// Apply mitigation action to a threat
    fn apply_mitigation(
        env: Env,
        admin: Address,
        threat_id: BytesN<32>,
        action: MitigationAction,
    ) -> Result<(), SecurityError>;

    /// Get security metrics for a contract in a time window
    fn get_security_metrics(
        env: Env,
        contract: Symbol,
        window_id: u64,
    ) -> Result<SecurityMetrics, SecurityError>;

    /// Calculate and store security metrics for a contract
    fn calculate_security_metrics(
        env: Env,
        contract: Symbol,
        window_seconds: u64,
    ) -> Result<SecurityMetrics, SecurityError>;

    /// Check circuit breaker state
    fn check_circuit_breaker(
        env: Env,
        contract: Symbol,
        function: Symbol,
    ) -> Result<CircuitBreakerState, SecurityError>;

    /// Record a circuit breaker event (success/failure)
    fn record_circuit_breaker_event(
        env: Env,
        contract: Symbol,
        function: Symbol,
        success: bool,
    ) -> Result<bool, SecurityError>;

    /// Get recommendations for a threat
    fn get_recommendations(
        env: Env,
        threat_id: BytesN<32>,
    ) -> Result<Vec<SecurityRecommendation>, SecurityError>;

    /// Generate recommendations for a threat
    fn generate_recommendations(
        env: Env,
        threat_id: BytesN<32>,
    ) -> Result<Vec<SecurityRecommendation>, SecurityError>;

    /// Acknowledge a recommendation
    fn acknowledge_recommendation(
        env: Env,
        admin: Address,
        recommendation_id: BytesN<32>,
    ) -> Result<(), SecurityError>;

    /// Update security configuration
    fn update_config(env: Env, admin: Address, config: SecurityConfig)
        -> Result<(), SecurityError>;

    /// Get current security configuration
    fn get_config(env: Env) -> Result<SecurityConfig, SecurityError>;

    /// Check if rate limit is exceeded for an actor
    fn check_rate_limit(env: Env, actor: Address, contract: Symbol) -> Result<bool, SecurityError>;

    // --- Advanced Features ---

    /// Request AI anomaly analysis for an actor
    fn request_anomaly_analysis(env: Env, actor: Address, contract: Symbol) -> Result<BytesN<32>, SecurityError>;

    /// Oracle callback for AI anomaly analysis
    fn callback_anomaly_analysis(env: Env, oracle: Address, request_id: BytesN<32>, is_anomalous: bool, risk_score: u32) -> Result<(), SecurityError>;

    /// Request biometric verification (continuous auth)
    fn verify_biometrics(env: Env, actor: Address, encrypted_payload: soroban_sdk::String) -> Result<BytesN<32>, SecurityError>;

    /// Oracle callback for biometric verification
    fn callback_biometrics_verification(env: Env, oracle: Address, request_id: BytesN<32>, is_valid: bool) -> Result<(), SecurityError>;

    /// Request credential fraud verification
    fn verify_credential_fraud(env: Env, actor: Address, credential_hash: BytesN<32>) -> Result<BytesN<32>, SecurityError>;

    /// Oracle callback for credential fraud verification
    fn callback_credential_fraud(env: Env, oracle: Address, request_id: BytesN<32>, is_fraudulent: bool) -> Result<(), SecurityError>;

    /// Add or update threat intelligence
    fn update_threat_intelligence(env: Env, admin: Address, intel: ThreatIntelligence) -> Result<(), SecurityError>;

    /// Update user risk score
    fn update_user_risk_score(env: Env, admin: Address, user: Address, score: u32, risk_factor: Symbol) -> Result<(), SecurityError>;

    /// Get user risk score
    fn get_user_risk_score(env: Env, user: Address) -> Option<UserRiskScore>;

    /// Record security awareness training
    fn record_security_training(env: Env, admin: Address, user: Address, module: Symbol, score: u32) -> Result<(), SecurityError>;

    /// Generate an incident report
    fn generate_incident_report(env: Env, admin: Address, threat_ids: Vec<BytesN<32>>, impact_summary: soroban_sdk::String) -> Result<BytesN<32>, SecurityError>;
}
