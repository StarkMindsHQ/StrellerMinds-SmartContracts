use crate::types::{
    CircuitBreakerState, SecurityConfig, SecurityDataKey, SecurityMetrics, SecurityRecommendation,
    SecurityThreat, UserRiskScore, ThreatIntelligence, IncidentReport, SecurityTrainingStatus
};    
use soroban_sdk::{Address, BytesN, Env, Symbol, Vec};

/// Storage utilities for the Security Monitor contract
pub struct SecurityStorage;

impl SecurityStorage {
    // ===== Config and Admin =====

    pub fn set_config(env: &Env, config: &SecurityConfig) {
        env.storage()
            .instance()
            .set(&SecurityDataKey::Config, config);
    }

    pub fn get_config(env: &Env) -> Option<SecurityConfig> {
        env.storage().instance().get(&SecurityDataKey::Config)
    }

    pub fn set_admin(env: &Env, admin: &Address) {
        env.storage().instance().set(&SecurityDataKey::Admin, admin);
    }

    pub fn get_admin(env: &Env) -> Option<Address> {
        env.storage().instance().get(&SecurityDataKey::Admin)
    }

    // ===== Threat Management =====

    pub fn set_threat(env: &Env, threat: &SecurityThreat) {
        let key = SecurityDataKey::Threat(threat.threat_id.clone());
        env.storage().persistent().set(&key, threat);

        // Add to contract's threat list
        Self::add_contract_threat(env, &threat.contract, &threat.threat_id);
    }

    pub fn get_threat(env: &Env, threat_id: &BytesN<32>) -> Option<SecurityThreat> {
        let key = SecurityDataKey::Threat(threat_id.clone());
        env.storage().persistent().get(&key)
    }

    pub fn has_threat(env: &Env, threat_id: &BytesN<32>) -> bool {
        let key = SecurityDataKey::Threat(threat_id.clone());
        env.storage().persistent().has(&key)
    }

    pub fn add_contract_threat(env: &Env, contract: &Symbol, threat_id: &BytesN<32>) {
        let key = SecurityDataKey::ContractThreats(contract.clone());
        let mut threats: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));

        threats.push_back(threat_id.clone());
        env.storage().persistent().set(&key, &threats);
    }

    pub fn get_contract_threats(env: &Env, contract: &Symbol) -> Vec<BytesN<32>> {
        let key = SecurityDataKey::ContractThreats(contract.clone());
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env))
    }

    // ===== Security Metrics =====

    pub fn set_security_metrics(
        env: &Env,
        contract: &Symbol,
        window_id: u64,
        metrics: &SecurityMetrics,
    ) {
        let key = SecurityDataKey::SecurityMetrics(contract.clone(), window_id);
        env.storage().persistent().set(&key, metrics);
    }

    pub fn get_security_metrics(
        env: &Env,
        contract: &Symbol,
        window_id: u64,
    ) -> Option<SecurityMetrics> {
        let key = SecurityDataKey::SecurityMetrics(contract.clone(), window_id);
        env.storage().persistent().get(&key)
    }

    // ===== Circuit Breaker =====

    pub fn set_circuit_breaker_state(env: &Env, state: &CircuitBreakerState) {
        let key =
            SecurityDataKey::CircuitBreaker(state.contract.clone(), state.function_name.clone());
        env.storage().persistent().set(&key, state);
    }

    pub fn get_circuit_breaker_state(
        env: &Env,
        contract: &Symbol,
        function: &Symbol,
    ) -> Option<CircuitBreakerState> {
        let key = SecurityDataKey::CircuitBreaker(contract.clone(), function.clone());
        env.storage().persistent().get(&key)
    }

    // ===== Actor Event Tracking =====

    pub fn set_actor_event_count(env: &Env, actor: &Address, window_id: u64, count: u32) {
        let key = SecurityDataKey::ActorEventCount(actor.clone(), window_id);
        env.storage().temporary().set(&key, &count);
    }

    pub fn get_actor_event_count(env: &Env, actor: &Address, window_id: u64) -> Option<u32> {
        let key = SecurityDataKey::ActorEventCount(actor.clone(), window_id);
        env.storage().temporary().get(&key)
    }

    pub fn increment_actor_event_count(env: &Env, actor: &Address, window_id: u64) {
        let current = Self::get_actor_event_count(env, actor, window_id).unwrap_or(0);
        Self::set_actor_event_count(env, actor, window_id, current + 1);
    }

    // ===== Contract Event Baseline =====

    pub fn set_contract_baseline(env: &Env, contract: &Symbol, baseline: u32) {
        let key = SecurityDataKey::ContractEventBaseline(contract.clone());
        env.storage().persistent().set(&key, &baseline);
    }

    pub fn get_contract_baseline(env: &Env, contract: &Symbol) -> Option<u32> {
        let key = SecurityDataKey::ContractEventBaseline(contract.clone());
        env.storage().persistent().get(&key)
    }

    // ===== Recommendations =====

    pub fn set_recommendation(env: &Env, recommendation: &SecurityRecommendation) {
        let key = SecurityDataKey::Recommendation(recommendation.recommendation_id.clone());
        env.storage().persistent().set(&key, recommendation);

        // Add to threat's recommendations list
        Self::add_threat_recommendation(
            env,
            &recommendation.threat_id,
            &recommendation.recommendation_id,
        );
    }

    pub fn get_recommendation(
        env: &Env,
        recommendation_id: &BytesN<32>,
    ) -> Option<SecurityRecommendation> {
        let key = SecurityDataKey::Recommendation(recommendation_id.clone());
        env.storage().persistent().get(&key)
    }

    pub fn add_threat_recommendation(
        env: &Env,
        threat_id: &BytesN<32>,
        recommendation_id: &BytesN<32>,
    ) {
        let key = SecurityDataKey::ThreatRecommendations(threat_id.clone());
        let mut recommendations: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));

        recommendations.push_back(recommendation_id.clone());
        env.storage().persistent().set(&key, &recommendations);
    }

    pub fn get_threat_recommendations(env: &Env, threat_id: &BytesN<32>) -> Vec<BytesN<32>> {
        let key = SecurityDataKey::ThreatRecommendations(threat_id.clone());
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env))
    }

    // ===== Advanced Security Features =====

    pub fn set_user_risk_score(env: &Env, user: &Address, score: &UserRiskScore) {
        let key = SecurityDataKey::UserRiskScore(user.clone());
        env.storage().persistent().set(&key, score);
    }

    pub fn get_user_risk_score(env: &Env, user: &Address) -> Option<UserRiskScore> {
        let key = SecurityDataKey::UserRiskScore(user.clone());
        env.storage().persistent().get(&key)
    }

    pub fn set_threat_intelligence(env: &Env, indicator_type: &Symbol, intel: &ThreatIntelligence) {
        let key = SecurityDataKey::ThreatIntelligence(indicator_type.clone());
        env.storage().persistent().set(&key, intel);
    }

    pub fn get_threat_intelligence(env: &Env, indicator_type: &Symbol) -> Option<ThreatIntelligence> {
        let key = SecurityDataKey::ThreatIntelligence(indicator_type.clone());
        env.storage().persistent().get(&key)
    }

    pub fn set_training_status(env: &Env, user: &Address, status: &SecurityTrainingStatus) {
        let key = SecurityDataKey::TrainingStatus(user.clone());
        env.storage().persistent().set(&key, status);
    }

    pub fn get_training_status(env: &Env, user: &Address) -> Option<SecurityTrainingStatus> {
        let key = SecurityDataKey::TrainingStatus(user.clone());
        env.storage().persistent().get(&key)
    }

    pub fn set_incident_report(env: &Env, report: &IncidentReport) {
        let key = SecurityDataKey::IncidentReport(report.incident_id.clone());
        env.storage().persistent().set(&key, report);
    }

    pub fn get_incident_report(env: &Env, incident_id: &BytesN<32>) -> Option<IncidentReport> {
        let key = SecurityDataKey::IncidentReport(incident_id.clone());
        env.storage().persistent().get(&key)
    }

    pub fn set_oracle_authorization(env: &Env, oracle: &Address, authorized: bool) {
        let key = SecurityDataKey::Oracle(oracle.clone());
        if authorized {
            env.storage().persistent().set(&key, &true);
        } else {
            env.storage().persistent().remove(&key);
        }
    }

    pub fn is_oracle_authorized(env: &Env, oracle: &Address) -> bool {
        let key = SecurityDataKey::Oracle(oracle.clone());
        env.storage().persistent().get(&key).unwrap_or(false)
    }
}
