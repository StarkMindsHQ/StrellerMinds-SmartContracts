use crate::errors::SecurityError;
use crate::types::{
    CircuitBreakerState, MitigationAction, SecurityConfig, SecurityMetrics, SecurityRecommendation,
    SecurityThreat,
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
    fn update_config(
        env: Env,
        admin: Address,
        config: SecurityConfig,
    ) -> Result<(), SecurityError>;

    /// Get current security configuration
    fn get_config(env: Env) -> Result<SecurityConfig, SecurityError>;

    /// Check if rate limit is exceeded for an actor
    fn check_rate_limit(
        env: Env,
        actor: Address,
        contract: Symbol,
    ) -> Result<bool, SecurityError>;
}
