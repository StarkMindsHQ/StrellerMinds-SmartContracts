use crate::types::{
    BreakerState, MitigationAction, RecommendationCategory, SecurityThreat, ThreatLevel, ThreatType,
};
use shared::event_schema::{
    AccessControlEventData, AnomalyAnalysisRequestedEvent, BiometricsVerificationRequestedEvent,
    ContractInitializedEvent, FraudVerificationRequestedEvent, IncidentReportGeneratedEvent,
    SecurityEventData, SecurityTrainingRecordedEvent, ThreatIntelligenceAddedEvent,
    UserRiskScoreUpdatedEvent,
};
use shared::{emit_access_control_event, emit_security_event};
use soroban_sdk::{symbol_short, Address, BytesN, Env, String, Symbol};

/// Security event emission utilities
pub struct SecurityEvents;

impl SecurityEvents {
    pub fn emit_initialized(env: &Env, admin: &Address) {
        emit_access_control_event!(
            env,
            symbol_short!("sec_mon"),
            admin.clone(),
            AccessControlEventData::ContractInitialized(ContractInitializedEvent {
                admin: admin.clone(),
            })
        );
    }

    /// Emit threat detected event
    pub fn emit_threat_detected(env: &Env, threat: &SecurityThreat) {
        let topics = (
            Symbol::new(env, "security"),
            Symbol::new(env, "threat_detected"),
            threat.contract.clone(),
        );

        let data = (
            threat.threat_id.clone(),
            Self::threat_type_to_string(env, &threat.threat_type),
            Self::threat_level_to_string(env, &threat.threat_level),
            threat.detected_at,
            threat.metric_value,
            threat.threshold_value,
        );

        env.events().publish(topics, data);
    }

    /// Emit threat mitigated event
    pub fn emit_threat_mitigated(
        env: &Env,
        threat_id: &BytesN<32>,
        action: &MitigationAction,
        mitigated_by: &Address,
    ) {
        let topics = (Symbol::new(env, "security"), Symbol::new(env, "threat_mitigated"));

        let data = (
            threat_id.clone(),
            Self::mitigation_action_to_string(env, action),
            mitigated_by.clone(),
            env.ledger().timestamp(),
        );

        env.events().publish(topics, data);
    }

    /// Emit circuit breaker opened event
    pub fn emit_circuit_breaker_opened(
        env: &Env,
        contract: &Symbol,
        function: &Symbol,
        failure_count: u32,
    ) {
        let topics =
            (Symbol::new(env, "security"), Symbol::new(env, "circuit_opened"), contract.clone());

        let data = (function.clone(), failure_count, env.ledger().timestamp());

        env.events().publish(topics, data);
    }

    /// Emit circuit breaker closed event
    pub fn emit_circuit_breaker_closed(env: &Env, contract: &Symbol, function: &Symbol) {
        let topics =
            (Symbol::new(env, "security"), Symbol::new(env, "circuit_closed"), contract.clone());

        let data = (function.clone(), env.ledger().timestamp());

        env.events().publish(topics, data);
    }

    /// Emit circuit breaker state changed event
    pub fn emit_circuit_breaker_state_changed(
        env: &Env,
        contract: &Symbol,
        function: &Symbol,
        new_state: &BreakerState,
    ) {
        let topics =
            (Symbol::new(env, "security"), Symbol::new(env, "circuit_state"), contract.clone());

        let data = (
            function.clone(),
            Self::breaker_state_to_string(env, new_state),
            env.ledger().timestamp(),
        );

        env.events().publish(topics, data);
    }

    /// Emit rate limit exceeded event
    pub fn emit_rate_limit_exceeded(
        env: &Env,
        actor: &Address,
        contract: &Symbol,
        event_count: u32,
        limit: u32,
    ) {
        let topics =
            (Symbol::new(env, "security"), Symbol::new(env, "rate_limit"), contract.clone());

        let data = (actor.clone(), event_count, limit, env.ledger().timestamp());

        env.events().publish(topics, data);
    }

    /// Emit recommendation generated event
    pub fn emit_recommendation_generated(
        env: &Env,
        recommendation_id: &BytesN<32>,
        threat_id: &BytesN<32>,
        category: &RecommendationCategory,
        severity: &ThreatLevel,
    ) {
        let topics = (Symbol::new(env, "security"), Symbol::new(env, "recommendation"));

        let data = (
            recommendation_id.clone(),
            threat_id.clone(),
            Self::recommendation_category_to_string(env, category),
            Self::threat_level_to_string(env, severity),
            env.ledger().timestamp(),
        );

        env.events().publish(topics, data);
    }

    /// Emit config updated event
    pub fn emit_config_updated(env: &Env, admin: &Address, change_type: &str) {
        env.events().publish(
            (Symbol::new(env, "security"), Symbol::new(env, "config_updated")),
            (admin.clone(), String::from_str(env, change_type)),
        );
    }

    // --- Advanced Feature Events ---

    pub fn emit_anomaly_requested(
        env: &Env,
        actor: &Address,
        contract: &Symbol,
        request_id: &BytesN<32>,
    ) {
        emit_security_event!(
            env,
            symbol_short!("sec_mon"),
            actor.clone(),
            SecurityEventData::AnomalyAnalysisRequested(AnomalyAnalysisRequestedEvent {
                actor: actor.clone(),
                contract: contract.clone(),
                request_id: request_id.clone(),
            })
        );
    }

    pub fn emit_biometrics_requested(env: &Env, user: &Address, request_id: &BytesN<32>) {
        emit_security_event!(
            env,
            symbol_short!("sec_mon"),
            user.clone(),
            SecurityEventData::BiometricsVerificationRequested(
                BiometricsVerificationRequestedEvent {
                    actor: user.clone(),
                    request_id: request_id.clone(),
                }
            )
        );
    }

    pub fn emit_fraud_requested(env: &Env, user: &Address, request_id: &BytesN<32>) {
        emit_security_event!(
            env,
            symbol_short!("sec_mon"),
            user.clone(),
            SecurityEventData::FraudVerificationRequested(FraudVerificationRequestedEvent {
                actor: user.clone(),
                request_id: request_id.clone(),
            })
        );
    }

    pub fn emit_intel_added(
        env: &Env,
        source: &Symbol,
        indicator_type: &Symbol,
        indicator_value: &String,
        threat_level: &ThreatLevel,
    ) {
        let contract_addr = env.current_contract_address();
        emit_security_event!(
            env,
            symbol_short!("sec_mon"),
            contract_addr,
            SecurityEventData::ThreatIntelligenceAdded(ThreatIntelligenceAddedEvent {
                source: source.clone(),
                indicator_type: indicator_type.clone(),
                indicator_value: indicator_value.clone(),
                threat_level: Self::threat_level_to_string(env, threat_level),
            })
        );
    }

    pub fn emit_risk_score_updated(env: &Env, user: &Address, score: u32, risk_factor: &Symbol) {
        let contract_addr = env.current_contract_address();
        emit_security_event!(
            env,
            symbol_short!("sec_mon"),
            contract_addr,
            SecurityEventData::UserRiskScoreUpdated(UserRiskScoreUpdatedEvent {
                user: user.clone(),
                score,
                risk_factor: risk_factor.clone(),
            })
        );
    }

    pub fn emit_training_recorded(env: &Env, user: &Address, module: &Symbol, score: u32) {
        emit_security_event!(
            env,
            symbol_short!("sec_mon"),
            user.clone(),
            SecurityEventData::SecurityTrainingRecorded(SecurityTrainingRecordedEvent {
                user: user.clone(),
                module: module.clone(),
                score,
            })
        );
    }

    pub fn emit_incident_reported(env: &Env, incident_id: &BytesN<32>, admin: &Address) {
        emit_security_event!(
            env,
            symbol_short!("sec_mon"),
            admin.clone(),
            SecurityEventData::IncidentReportGenerated(IncidentReportGeneratedEvent {
                incident_id: incident_id.clone(),
                admin: admin.clone(),
            })
        );
    }

    // Helper functions to convert enums to strings

    fn threat_type_to_string(env: &Env, threat_type: &ThreatType) -> String {
        let s = match threat_type {
            ThreatType::BurstActivity => "burst_activity",
            ThreatType::AnomalousActor => "anomalous_actor",
            ThreatType::ErrorRateSpike => "error_rate_spike",
            ThreatType::SequenceIntegrityIssue => "sequence_integrity",
            ThreatType::AccessViolation => "access_violation",
            ThreatType::ReentrancyAttempt => "reentrancy_attempt",
            ThreatType::ValidationFailure => "validation_failure",
            ThreatType::RateLimitExceeded => "rate_limit_exceeded",
            ThreatType::BehavioralAnomaly => "behavioral_anomaly",
            ThreatType::CredentialFraud => "credential_fraud",
            ThreatType::BiometricFailure => "biometric_failure",
            ThreatType::KnownMaliciousActor => "known_malicious_actor",
        };
        String::from_str(env, s)
    }

    fn threat_level_to_string(env: &Env, level: &ThreatLevel) -> String {
        let s = match level {
            ThreatLevel::Low => "low",
            ThreatLevel::Medium => "medium",
            ThreatLevel::High => "high",
            ThreatLevel::Critical => "critical",
        };
        String::from_str(env, s)
    }

    fn mitigation_action_to_string(env: &Env, action: &MitigationAction) -> String {
        let s = match action {
            MitigationAction::RateLimitApplied => "rate_limit_applied",
            MitigationAction::CircuitBreakerTriggered => "circuit_breaker_triggered",
            MitigationAction::AccessRestricted => "access_restricted",
            MitigationAction::AlertSent => "alert_sent",
            MitigationAction::NoAction => "no_action",
            MitigationAction::RequireReauth => "require_reauth",
            MitigationAction::LockAccount => "lock_account",
        };
        String::from_str(env, s)
    }

    fn breaker_state_to_string(env: &Env, state: &BreakerState) -> String {
        let s = match state {
            BreakerState::Closed => "closed",
            BreakerState::Open => "open",
            BreakerState::HalfOpen => "half_open",
        };
        String::from_str(env, s)
    }

    fn recommendation_category_to_string(env: &Env, category: &RecommendationCategory) -> String {
        let s = match category {
            RecommendationCategory::AccessControl => "access_control",
            RecommendationCategory::InputValidation => "input_validation",
            RecommendationCategory::ReentrancyPrevention => "reentrancy_prevention",
            RecommendationCategory::RateLimiting => "rate_limiting",
            RecommendationCategory::EventIntegrity => "event_integrity",
            RecommendationCategory::Configuration => "configuration",
        };
        String::from_str(env, s)
    }
}
