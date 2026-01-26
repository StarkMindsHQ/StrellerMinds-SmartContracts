#![no_std]

mod circuit_breaker;
mod errors;
mod events;
mod interface;
mod recommendation_engine;
mod storage;
mod threat_detector;
mod types;

#[cfg(test)]
mod tests;

use soroban_sdk::{contract, contractimpl, Address, BytesN, Env, Symbol, Vec};

use shared::{
    access_control::AccessControl,
    reentrancy_guard::ReentrancyLock,
    roles::Permission,
};

use circuit_breaker::CircuitBreaker;
use errors::SecurityError;
use events::SecurityEvents;
use interface::SecurityMonitorTrait;
use recommendation_engine::RecommendationEngine;
use storage::SecurityStorage;
use threat_detector::ThreatDetector;
use types::{
    CircuitBreakerState, MitigationAction, SecurityConfig, SecurityMetrics, SecurityRecommendation,
    SecurityThreat, ThreatLevel,
};

#[contract]
pub struct SecurityMonitor;

#[contractimpl]
impl SecurityMonitorTrait for SecurityMonitor {
    fn initialize(env: Env, admin: Address, config: SecurityConfig) -> Result<(), SecurityError> {
        // Check if already initialized
        if SecurityStorage::get_admin(&env).is_some() {
            return Err(SecurityError::AlreadyInitialized);
        }

        admin.require_auth();

        // Validate configuration
        if config.burst_detection_threshold == 0 || config.burst_window_seconds == 0 {
            return Err(SecurityError::InvalidConfiguration);
        }

        // Initialize shared RBAC
        let _ = AccessControl::initialize(&env, &admin);

        // Store admin and configuration
        SecurityStorage::set_admin(&env, &admin);
        SecurityStorage::set_config(&env, &config);

        // Emit initialization event
        SecurityEvents::emit_initialized(&env, &admin);

        Ok(())
    }

    fn scan_for_threats(
        env: Env,
        contract: Symbol,
        window_seconds: u64,
    ) -> Result<Vec<SecurityThreat>, SecurityError> {
        let mut threats = Vec::new(&env);

        // Run burst detection
        if let Ok(Some(threat)) = ThreatDetector::detect_burst_activity(&env, &contract, window_seconds) {
            SecurityStorage::set_threat(&env, &threat);
            SecurityEvents::emit_threat_detected(&env, &threat);
            threats.push_back(threat);
        }

        // Run error rate detection
        let current_time = env.ledger().timestamp();
        let window_id = current_time / 3600; // Hourly windows
        if let Ok(Some(threat)) = ThreatDetector::detect_error_rate_spike(&env, &contract, window_id) {
            SecurityStorage::set_threat(&env, &threat);
            SecurityEvents::emit_threat_detected(&env, &threat);
            threats.push_back(threat);
        }

        Ok(threats)
    }

    fn get_threat(env: Env, threat_id: BytesN<32>) -> Result<SecurityThreat, SecurityError> {
        SecurityStorage::get_threat(&env, &threat_id).ok_or(SecurityError::ThreatNotFound)
    }

    fn get_contract_threats(env: Env, contract: Symbol) -> Vec<BytesN<32>> {
        SecurityStorage::get_contract_threats(&env, &contract)
    }

    fn apply_mitigation(
        env: Env,
        admin: Address,
        threat_id: BytesN<32>,
        action: MitigationAction,
    ) -> Result<(), SecurityError> {
        let _guard = ReentrancyLock::new(&env);
        admin.require_auth();

        // Verify admin has permission
        if AccessControl::require_permission(&env, &admin, &Permission::UpdateCourse).is_err() {
            return Err(SecurityError::PermissionDenied);
        }

        // Get the threat
        let mut threat = SecurityStorage::get_threat(&env, &threat_id)
            .ok_or(SecurityError::ThreatNotFound)?;

        // Mark as mitigated
        threat.auto_mitigated = true;
        threat.mitigation_action = Some(action.clone());

        // Save updated threat
        SecurityStorage::set_threat(&env, &threat);

        // Emit mitigation event
        SecurityEvents::emit_threat_mitigated(&env, &threat_id, &action, &admin);

        Ok(())
    }

    fn get_security_metrics(
        env: Env,
        contract: Symbol,
        window_id: u64,
    ) -> Result<SecurityMetrics, SecurityError> {
        SecurityStorage::get_security_metrics(&env, &contract, window_id)
            .ok_or(SecurityError::MetricsNotFound)
    }

    fn calculate_security_metrics(
        env: Env,
        contract: Symbol,
        window_seconds: u64,
    ) -> Result<SecurityMetrics, SecurityError> {
        let current_time = env.ledger().timestamp();
        let window_id = current_time / 3600; // Hourly windows
        let start_time = current_time - window_seconds;

        let metrics = ThreatDetector::calculate_metrics(&env, &contract, start_time, current_time, window_id)?;

        // Store metrics
        SecurityStorage::set_security_metrics(&env, &contract, window_id, &metrics);

        Ok(metrics)
    }

    fn check_circuit_breaker(
        env: Env,
        contract: Symbol,
        function: Symbol,
    ) -> Result<CircuitBreakerState, SecurityError> {
        let config = SecurityStorage::get_config(&env).ok_or(SecurityError::NotInitialized)?;

        let state = SecurityStorage::get_circuit_breaker_state(&env, &contract, &function)
            .unwrap_or_else(|| {
                CircuitBreakerState::new(
                    contract.clone(),
                    function.clone(),
                    config.circuit_breaker_threshold,
                    config.circuit_breaker_timeout,
                )
            });

        Ok(state)
    }

    fn record_circuit_breaker_event(
        env: Env,
        contract: Symbol,
        function: Symbol,
        success: bool,
    ) -> Result<bool, SecurityError> {
        CircuitBreaker::check_and_record(&env, &contract, &function, success)
    }

    fn get_recommendations(
        env: Env,
        threat_id: BytesN<32>,
    ) -> Result<Vec<SecurityRecommendation>, SecurityError> {
        let recommendation_ids = SecurityStorage::get_threat_recommendations(&env, &threat_id);
        let mut recommendations = Vec::new(&env);

        for i in 0..recommendation_ids.len() {
            if let Some(rec) = SecurityStorage::get_recommendation(&env, &recommendation_ids.get(i).unwrap()) {
                recommendations.push_back(rec);
            }
        }

        Ok(recommendations)
    }

    fn generate_recommendations(
        env: Env,
        threat_id: BytesN<32>,
    ) -> Result<Vec<SecurityRecommendation>, SecurityError> {
        // Get the threat
        let threat = SecurityStorage::get_threat(&env, &threat_id)
            .ok_or(SecurityError::ThreatNotFound)?;

        // Generate recommendations
        let recommendations = RecommendationEngine::generate_recommendations(&env, &threat)?;

        // Emit events for each recommendation
        for i in 0..recommendations.len() {
            let rec = recommendations.get(i).unwrap();
            SecurityEvents::emit_recommendation_generated(
                &env,
                &rec.recommendation_id,
                &rec.threat_id,
                &rec.category,
                &rec.severity,
            );
        }

        Ok(recommendations)
    }

    fn acknowledge_recommendation(
        env: Env,
        admin: Address,
        recommendation_id: BytesN<32>,
    ) -> Result<(), SecurityError> {
        admin.require_auth();

        // Verify admin has permission
        if AccessControl::require_permission(&env, &admin, &Permission::UpdateCourse).is_err() {
            return Err(SecurityError::PermissionDenied);
        }

        // Get and update recommendation
        let mut recommendation = SecurityStorage::get_recommendation(&env, &recommendation_id)
            .ok_or(SecurityError::RecommendationNotFound)?;

        recommendation.acknowledged = true;
        SecurityStorage::set_recommendation(&env, &recommendation);

        Ok(())
    }

    fn update_config(
        env: Env,
        admin: Address,
        config: SecurityConfig,
    ) -> Result<(), SecurityError> {
        admin.require_auth();

        // Verify admin
        let stored_admin = SecurityStorage::get_admin(&env).ok_or(SecurityError::NotInitialized)?;
        if admin != stored_admin {
            return Err(SecurityError::Unauthorized);
        }

        // Validate configuration
        if config.burst_detection_threshold == 0 || config.burst_window_seconds == 0 {
            return Err(SecurityError::InvalidConfiguration);
        }

        // Update config
        SecurityStorage::set_config(&env, &config);

        // Emit event
        SecurityEvents::emit_config_updated(&env, &admin, "updated");

        Ok(())
    }

    fn get_config(env: Env) -> Result<SecurityConfig, SecurityError> {
        SecurityStorage::get_config(&env).ok_or(SecurityError::NotInitialized)
    }

    fn check_rate_limit(
        env: Env,
        actor: Address,
        contract: Symbol,
    ) -> Result<bool, SecurityError> {
        let config = SecurityStorage::get_config(&env).ok_or(SecurityError::NotInitialized)?;
        let current_time = env.ledger().timestamp();
        let window_id = current_time / config.rate_limit_window;

        let count = SecurityStorage::get_actor_event_count(&env, &actor, window_id).unwrap_or(0);

        if count >= config.rate_limit_per_window {
            SecurityEvents::emit_rate_limit_exceeded(
                &env,
                &actor,
                &contract,
                count,
                config.rate_limit_per_window,
            );
            Ok(false) // Rate limit exceeded
        } else {
            SecurityStorage::increment_actor_event_count(&env, &actor, window_id);
            Ok(true) // Within limit
        }
    }
}
