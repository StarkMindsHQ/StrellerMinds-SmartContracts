//! Security test suite for the SecurityMonitor contract.
//!
//! Covers:
//! - Initialisation and admin controls
//! - Unauthorized access (penetration scenarios)
//! - Threat detection and circuit-breaker logic
//! - User risk scores and threat intelligence
//! - Incident reporting
//! - Input boundary / adversarial edge cases

#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, BytesN, Env, String, Symbol, Vec,
};

use crate::{
    storage::SecurityStorage,
    types::{
        BreakerState, SecurityConfig, SecurityMetrics, SecurityThreat, ThreatIntelligence,
        ThreatLevel, ThreatType, MitigationAction,
    },
    SecurityMonitor, SecurityMonitorClient,
};

// ─────────────────────────────────────────────────────────────
// Test setup helpers
// ─────────────────────────────────────────────────────────────

fn default_config(_env: &Env) -> SecurityConfig {
    SecurityConfig::default_config()
}

/// Stand up a fresh SecurityMonitor contract and return the env, contract_id, client, and admin.
fn setup() -> (Env, Address, SecurityMonitorClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(SecurityMonitor, ());
    let client = SecurityMonitorClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    let config = default_config(&env);

    client.initialize(&admin, &config);
    (env, contract_id, client, admin)
}

/// Build a deterministic 32-byte array from a seed byte.
fn bytes32(env: &Env, seed: u8) -> BytesN<32> {
    BytesN::from_array(env, &[seed; 32])
}

/// Build a minimal threat record suitable for storage tests.
fn make_threat(env: &Env, contract: &Symbol, seed: u8) -> SecurityThreat {
    SecurityThreat {
        threat_id: bytes32(env, seed),
        threat_type: ThreatType::BurstActivity,
        threat_level: ThreatLevel::Medium,
        detected_at: env.ledger().timestamp(),
        contract: contract.clone(),
        actor: None,
        description: String::from_str(env, "unit-test threat"),
        metric_value: 200,
        threshold_value: 100,
        auto_mitigated: false,
        mitigation_action: MitigationAction::NoAction,
    }
}

// ─────────────────────────────────────────────────────────────
// 1. Initialisation tests
// ─────────────────────────────────────────────────────────────

#[test]
fn test_initialize_sets_admin() {
    let (env, contract_id, _client, admin) = setup();
    let stored_admin = env.as_contract(&contract_id, || {
        SecurityStorage::get_admin(&env).expect("admin must be stored")
    });
    assert_eq!(stored_admin, admin);
}

#[test]
fn test_initialize_stores_config() {
    let (env, contract_id, _client, _admin) = setup();
    let config = env.as_contract(&contract_id, || {
        SecurityStorage::get_config(&env).expect("config must be stored")
    });
    assert!(config.burst_detection_threshold > 0);
}

#[test]
fn test_scan_for_threats_returns_empty_without_metrics() {
    let (_, _contract_id, client, _) = setup();
    let contract_sym = Symbol::new(&client.env, "mycontract");
    let threats = client.scan_for_threats(&contract_sym, &60u64);
    assert_eq!(threats.len(), 0);
}

// ─────────────────────────────────────────────────────────────
// 2. Authorization / penetration tests
// ─────────────────────────────────────────────────────────────

#[test]
fn test_update_threat_intel_rejects_non_admin() {
    let (env, _contract_id, client, _admin) = setup();
    let attacker = Address::generate(&env);
    let intel = ThreatIntelligence {
        source: Symbol::new(&env, "evil"),
        indicator_type: Symbol::new(&env, "Address"),
        indicator_value: String::from_str(&env, "GATTACKER"),
        threat_level: ThreatLevel::Critical,
        added_at: 0,
    };
    let result = client.try_update_threat_intelligence(&attacker, &intel);
    assert!(
        result.is_err(),
        "non-admin must not be allowed to inject threat intel"
    );
}

#[test]
fn test_update_user_risk_rejects_non_admin_non_oracle() {
    let (env, _contract_id, client, _admin) = setup();
    let attacker = Address::generate(&env);
    let target = Address::generate(&env);
    let result = client.try_update_user_risk_score(
        &attacker,
        &target,
        &99u32,
        &Symbol::new(&env, "attack"),
    );
    assert!(
        result.is_err(),
        "random address must not be able to escalate a user risk score"
    );
}

#[test]
fn test_record_training_rejects_non_admin() {
    let (env, _contract_id, client, _admin) = setup();
    let attacker = Address::generate(&env);
    let user = Address::generate(&env);
    let result = client.try_record_security_training(
        &attacker,
        &user,
        &Symbol::new(&env, "Module1"),
        &80u32,
    );
    assert!(
        result.is_err(),
        "non-admin must not record security training"
    );
}

#[test]
fn test_generate_incident_report_rejects_non_admin() {
    let (env, _contract_id, client, _admin) = setup();
    let attacker = Address::generate(&env);
    let threat_ids: Vec<BytesN<32>> = Vec::new(&env);
    let result = client.try_generate_incident_report(
        &attacker,
        &threat_ids,
        &String::from_str(&env, "impact"),
    );
    assert!(
        result.is_err(),
        "non-admin must not generate incident reports"
    );
}

#[test]
fn test_oracle_callback_rejects_unauthorized_caller() {
    let (env, _contract_id, client, _admin) = setup();
    let fake_oracle = Address::generate(&env);
    let req_id = bytes32(&env, 0xAB);
    // anomaly callback with an unregistered oracle must fail
    let result =
        client.try_callback_anomaly_analysis(&fake_oracle, &req_id, &true, &95u32);
    assert!(
        result.is_err(),
        "callback from un-authorized oracle must be rejected"
    );
}

#[test]
fn test_biometrics_callback_rejects_unauthorized_caller() {
    let (env, _contract_id, client, _admin) = setup();
    let fake_oracle = Address::generate(&env);
    let req_id = bytes32(&env, 0xCD);
    let result =
        client.try_callback_biometrics_verification(&fake_oracle, &req_id, &true);
    assert!(
        result.is_err(),
        "biometrics callback from unauthorized oracle must fail"
    );
}

#[test]
fn test_credential_fraud_callback_rejects_unauthorized_caller() {
    let (env, _contract_id, client, _admin) = setup();
    let fake_oracle = Address::generate(&env);
    let req_id = bytes32(&env, 0xEF);
    let result =
        client.try_callback_credential_fraud(&fake_oracle, &req_id, &false);
    assert!(
        result.is_err(),
        "credential-fraud callback from unauthorized oracle must fail"
    );
}

// ─────────────────────────────────────────────────────────────
// 3. Threat management tests
// ─────────────────────────────────────────────────────────────

#[test]
fn test_get_nonexistent_threat_returns_error() {
    let (env, _contract_id, client, _) = setup();
    let missing_id = bytes32(&env, 0x00);
    let result = client.try_get_threat(&missing_id);
    assert!(result.is_err(), "fetching a non-existent threat must fail");
}

#[test]
fn test_get_contract_threats_returns_empty_initially() {
    let (_, _contract_id, client, _) = setup();
    let contract_sym = Symbol::new(&client.env, "tokencontract");
    let threats = client.get_contract_threats(&contract_sym);
    assert_eq!(threats.len(), 0);
}

#[test]
fn test_storage_set_and_get_threat_roundtrip() {
    let (env, contract_id, _client, _) = setup();
    let contract_sym = Symbol::new(&env, "testcontract");
    let threat = make_threat(&env, &contract_sym, 0x11);

    env.as_contract(&contract_id, || {
        SecurityStorage::set_threat(&env, &threat);
        let retrieved = SecurityStorage::get_threat(&env, &threat.threat_id)
            .expect("stored threat must be retrievable");
        assert_eq!(retrieved.threat_type, ThreatType::BurstActivity);
        assert_eq!(retrieved.threat_level, ThreatLevel::Medium);
        assert_eq!(retrieved.metric_value, 200);
    });
}

#[test]
fn test_contract_threat_list_grows_with_each_set() {
    let (env, contract_id, _client, _) = setup();
    let contract_sym = Symbol::new(&env, "multicontract");

    env.as_contract(&contract_id, || {
        for seed in 0x20u8..0x25u8 {
            let threat = make_threat(&env, &contract_sym, seed);
            SecurityStorage::set_threat(&env, &threat);
        }
        let threats = SecurityStorage::get_contract_threats(&env, &contract_sym);
        assert_eq!(threats.len(), 5, "five distinct threats must be listed");
    });
}

// ─────────────────────────────────────────────────────────────
// 4. User risk score tests
// ─────────────────────────────────────────────────────────────

#[test]
fn test_update_and_get_user_risk_score() {
    let (env, _contract_id, client, admin) = setup();
    let user = Address::generate(&env);

    client.update_user_risk_score(
        &admin,
        &user,
        &75u32,
        &Symbol::new(&env, "FailedLogin"),
    );

    let score = client
        .get_user_risk_score(&user)
        .expect("score must be present after update");
    assert_eq!(score.score, 75);
    assert_eq!(score.risk_factors.len(), 1);
}

#[test]
fn test_risk_score_absent_before_any_update() {
    let (env, _contract_id, client, _) = setup();
    let unknown_user = Address::generate(&env);
    let score = client.get_user_risk_score(&unknown_user);
    assert!(score.is_none(), "no risk score before any update");
}

#[test]
fn test_security_training_reduces_risk_score() {
    let (env, _contract_id, client, admin) = setup();
    let user = Address::generate(&env);

    // Give the user a high initial risk score
    client.update_user_risk_score(&admin, &user, &60u32, &Symbol::new(&env, "Anomalous"));

    // Record training - should subtract 10 from the score
    client.record_security_training(&admin, &user, &Symbol::new(&env, "Phishing101"), &90u32);

    let score = client
        .get_user_risk_score(&user)
        .expect("risk score must exist");
    assert_eq!(score.score, 50, "training should have reduced score by 10");
}

#[test]
fn test_security_training_does_not_underflow_below_zero() {
    let (env, _contract_id, client, admin) = setup();
    let user = Address::generate(&env);

    // Start at a very low risk score
    client.update_user_risk_score(&admin, &user, &5u32, &Symbol::new(&env, "lowrisk"));
    client.record_security_training(&admin, &user, &Symbol::new(&env, "BasicSec"), &85u32);

    // score was 5, knock-off of 10 is capped because of the >= 10 guard
    let score = client.get_user_risk_score(&user).unwrap();
    // score must stay at 5 (guard prevents subtraction when score < 10)
    assert_eq!(score.score, 5);
}

// ─────────────────────────────────────────────────────────────
// 5. Threat intelligence tests
// ─────────────────────────────────────────────────────────────

#[test]
fn test_update_and_retrieve_threat_intelligence() {
    let (env, contract_id, client, admin) = setup();
    let intel = ThreatIntelligence {
        source: Symbol::new(&env, "GlobalList"),
        indicator_type: Symbol::new(&env, "Address"),
        indicator_value: String::from_str(&env, "GBADACTOR1111"),
        threat_level: ThreatLevel::High,
        added_at: 0,
    };

    client.update_threat_intelligence(&admin, &intel);

    let stored = env.as_contract(&contract_id, || {
        SecurityStorage::get_threat_intelligence(&env, &Symbol::new(&env, "Address"))
            .expect("intel must be stored")
    });
    assert_eq!(stored.threat_level, ThreatLevel::High);
}

// ─────────────────────────────────────────────────────────────
// 6. Anomaly / biometrics / fraud request flows
// ─────────────────────────────────────────────────────────────

#[test]
fn test_request_anomaly_analysis_returns_request_id() {
    let (env, _contract_id, client, _) = setup();
    let actor = Address::generate(&env);
    let contract_sym = Symbol::new(&env, "myapp");
    let req_id = client.request_anomaly_analysis(&actor, &contract_sym);
    // Should be a non-zero 32-byte identifier
    let zero = bytes32(&env, 0x00);
    assert_ne!(req_id, zero, "request id must not be all zeros");
}

#[test]
fn test_verify_biometrics_returns_request_id() {
    let (env, _contract_id, client, _) = setup();
    let actor = Address::generate(&env);
    let payload = String::from_str(&env, "encrypted_biometric_data");
    let req_id = client.verify_biometrics(&actor, &payload);
    let zero = bytes32(&env, 0x00);
    assert_ne!(req_id, zero);
}

#[test]
fn test_verify_credential_fraud_returns_request_id() {
    let (env, _contract_id, client, _) = setup();
    let actor = Address::generate(&env);
    let cred_hash = bytes32(&env, 0xDE);
    let req_id = client.verify_credential_fraud(&actor, &cred_hash);
    let zero = bytes32(&env, 0x00);
    assert_ne!(req_id, zero);
}

// ─────────────────────────────────────────────────────────────
// 7. Incident report tests
// ─────────────────────────────────────────────────────────────

#[test]
fn test_generate_incident_report_by_admin_succeeds() {
    let (env, _contract_id, client, admin) = setup();
    let threat_ids: Vec<BytesN<32>> = Vec::new(&env);
    let summary = String::from_str(&env, "Major disruption from burst-activity.");
    let incident_id = client.generate_incident_report(&admin, &threat_ids, &summary);
    let zero = bytes32(&env, 0x00);
    assert_ne!(incident_id, zero, "incident id must not be all-zeros");
}

#[test]
fn test_incident_report_stored_in_storage() {
    let (env, contract_id, client, admin) = setup();
    let threat_ids: Vec<BytesN<32>> = Vec::new(&env);
    let summary = String::from_str(&env, "Test incident");
    let incident_id = client.generate_incident_report(&admin, &threat_ids, &summary);

    let report = env.as_contract(&contract_id, || {
        SecurityStorage::get_incident_report(&env, &incident_id)
            .expect("incident report must be persisted")
    });
    assert_eq!(report.incident_id, incident_id);
    assert_eq!(report.threat_ids.len(), 0);
}

// ─────────────────────────────────────────────────────────────
// 8. Metric storage and detection boundary tests
// ─────────────────────────────────────────────────────────────

#[test]
fn test_security_metrics_stored_and_retrieved() {
    let (env, contract_id, _client, _admin) = setup();
    let contract_sym = Symbol::new(&env, "metrictest");
    let window_id: u64 = 42;

    let metrics = SecurityMetrics {
        window_id,
        contract: contract_sym.clone(),
        start_time: 1000,
        end_time: 4600,
        total_events: 50,
        error_events: 5,
        error_rate: 10,
        unique_actors: 3,
        access_violations: 0,
        threat_count: 0,
        highest_threat_level: ThreatLevel::Low,
        security_score: 90,
    };

    env.as_contract(&contract_id, || {
        SecurityStorage::set_security_metrics(&env, &contract_sym, window_id, &metrics);
        let retrieved =
            SecurityStorage::get_security_metrics(&env, &contract_sym, window_id)
                .expect("must retrieve stored metrics");
        assert_eq!(retrieved.total_events, 50);
        assert_eq!(retrieved.error_rate, 10);
    });
}

#[test]
fn test_burst_activity_not_detected_under_threshold() {
    use crate::threat_detector::ThreatDetector;

    let (env, contract_id, _client, _admin) = setup();
    let contract_sym = Symbol::new(&env, "quietcontract");
    let window_id: u64 = 1;

    env.as_contract(&contract_id, || {
        let config = SecurityStorage::get_config(&env).unwrap();
        let metrics = SecurityMetrics {
            window_id,
            contract: contract_sym.clone(),
            start_time: 0,
            end_time: 3600,
            total_events: config.burst_detection_threshold - 1,
            error_events: 0,
            error_rate: 0,
            unique_actors: 1,
            access_violations: 0,
            threat_count: 0,
            highest_threat_level: ThreatLevel::Low,
            security_score: 100,
        };
        SecurityStorage::set_security_metrics(&env, &contract_sym, window_id, &metrics);

        let threat = ThreatDetector::detect_burst_activity(&env, &contract_sym, 3600)
            .expect("detection must not error");
        assert!(threat.is_none(), "no threat when under threshold");
    });
}

#[test]
fn test_burst_activity_detected_over_threshold() {
    use crate::threat_detector::ThreatDetector;

    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(3600); // must match window_id calculation

    let contract_id = env.register(SecurityMonitor, ());
    let client = SecurityMonitorClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &SecurityConfig::default_config());

    let contract_sym = Symbol::new(&env, "busycontract");
    let window_id: u64 = 1; // timestamp 3600 / 3600 = 1

    env.as_contract(&contract_id, || {
        let config = SecurityStorage::get_config(&env).unwrap();
        let metrics = SecurityMetrics {
            window_id,
            contract: contract_sym.clone(),
            start_time: 0,
            end_time: 3600,
            total_events: config.burst_detection_threshold + 50,
            error_events: 0,
            error_rate: 0,
            unique_actors: 1,
            access_violations: 0,
            threat_count: 0,
            highest_threat_level: ThreatLevel::Low,
            security_score: 50,
        };
        SecurityStorage::set_security_metrics(&env, &contract_sym, window_id, &metrics);

        let threat = ThreatDetector::detect_burst_activity(&env, &contract_sym, 3600)
            .expect("detection must not error");
        assert!(threat.is_some(), "threat must be detected over threshold");
        let t = threat.unwrap();
        assert_eq!(t.threat_type, ThreatType::BurstActivity);
    });
}

// ─────────────────────────────────────────────────────────────
// 9. Circuit-breaker state tests
// ─────────────────────────────────────────────────────────────

#[test]
fn test_circuit_breaker_starts_closed() {
    use crate::circuit_breaker::CircuitBreaker;

    let (env, contract_id, _client, _admin) = setup();
    let contract_sym = Symbol::new(&env, "cbcontract");
    let fn_sym = Symbol::new(&env, "transfer");

    let allowed = env.as_contract(&contract_id, || {
        CircuitBreaker::check_and_record(&env, &contract_sym, &fn_sym, true)
            .expect("check_and_record should not error")
    });
    assert!(allowed, "closed circuit must allow the operation");
}

#[test]
fn test_circuit_breaker_opens_after_threshold_failures() {
    use crate::circuit_breaker::CircuitBreaker;

    let (env, contract_id, _client, _admin) = setup();
    let contract_sym = Symbol::new(&env, "fragilecontract");
    let fn_sym = Symbol::new(&env, "execute");

    env.as_contract(&contract_id, || {
        let config = SecurityStorage::get_config(&env).unwrap();
        for _ in 0..config.circuit_breaker_threshold {
            CircuitBreaker::check_and_record(&env, &contract_sym, &fn_sym, false)
                .expect("recording failure must not error");
        }
        let state = SecurityStorage::get_circuit_breaker_state(&env, &contract_sym, &fn_sym)
            .expect("breaker state must be persisted");
        assert_eq!(
            state.state,
            BreakerState::Open,
            "circuit breaker must open after repeated failures"
        );
    });
}

#[test]
fn test_open_circuit_blocks_new_calls() {
    use crate::circuit_breaker::CircuitBreaker;

    let (env, contract_id, _client, _admin) = setup();
    let contract_sym = Symbol::new(&env, "blockedcontract");
    let fn_sym = Symbol::new(&env, "send");

    let allowed = env.as_contract(&contract_id, || {
        let config = SecurityStorage::get_config(&env).unwrap();
        for _ in 0..config.circuit_breaker_threshold {
            CircuitBreaker::check_and_record(&env, &contract_sym, &fn_sym, false).ok();
        }
        // Circuit is now Open – must block
        CircuitBreaker::check_and_record(&env, &contract_sym, &fn_sym, true)
            .expect("should not error")
    });
    assert!(!allowed, "open circuit must block new calls");
}

#[test]
fn test_circuit_breaker_transitions_to_half_open_after_timeout() {
    use crate::circuit_breaker::CircuitBreaker;

    let env = Env::default();
    env.mock_all_auths();
    env.ledger().set_timestamp(100);

    let contract_id = env.register(SecurityMonitor, ());
    let client = SecurityMonitorClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin, &SecurityConfig::default_config());

    let contract_sym = Symbol::new(&env, "recovercontract");
    let fn_sym = Symbol::new(&env, "recover");

    // Trip the breaker at time 100
    env.as_contract(&contract_id, || {
        let config = SecurityStorage::get_config(&env).unwrap();
        for _ in 0..config.circuit_breaker_threshold {
            CircuitBreaker::check_and_record(&env, &contract_sym, &fn_sym, false).ok();
        }
    });

    // Advance time past timeout
    let timeout = SecurityConfig::default_config().circuit_breaker_timeout;
    env.ledger().set_timestamp(100 + timeout + 1);

    // Next check should let one call through (transition to HalfOpen)
    let allowed = env.as_contract(&contract_id, || {
        CircuitBreaker::check_and_record(&env, &contract_sym, &fn_sym, true)
            .expect("should succeed")
    });
    assert!(allowed, "after timeout, circuit must move to HalfOpen and allow one call");
}

// ─────────────────────────────────────────────────────────────
// 10. Scan result consistency test
// ─────────────────────────────────────────────────────────────

#[test]
fn test_scan_for_threats_always_returns_vec() {
    let (_, _contract_id, client, _) = setup();
    let contract_sym = Symbol::new(&client.env, "scanme");
    // Should never panic – just return an empty or populated Vec
    let _ = client.scan_for_threats(&contract_sym, &3600u64);
}
