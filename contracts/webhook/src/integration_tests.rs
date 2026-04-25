//! Integration tests for the Webhook contract simulating real-world usage
//! alongside the certificate, student-progress-tracker, and gamification contracts.
//!
//! These tests use soroban-sdk testutils to run entirely in-process without
//! a live network. They verify the full dispatch → pending-delivery → retry
//! lifecycle as it would be triggered by other contracts.

#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Bytes, BytesN, Env, Vec,
};

use crate::{
    types::{
        AchievementUnlockedPayload, CertificateIssuedPayload, StudentProgressPayload,
        WebhookEventType, RETRY_BACKOFF_LEDGERS,
    },
    WebhookContract, WebhookContractClient,
};

// ---------------------------------------------------------------------------
// Shared setup
// ---------------------------------------------------------------------------

struct TestEnv {
    env: Env,
    client: WebhookContractClient<'static>,
    admin: Address,
}

impl TestEnv {
    fn new() -> Self {
        let env = Env::default();
        env.mock_all_auths();
        let contract_id = env.register(WebhookContract, ());
        let client = WebhookContractClient::new(&env, &contract_id);
        let admin = Address::generate(&env);
        client.initialize(&admin).unwrap();
        Self { env, client, admin }
    }

    fn register_all_events(&self, owner: &Address) -> u32 {
        let mut events = Vec::new(&self.env);
        events.push_back(WebhookEventType::CertificateIssued);
        events.push_back(WebhookEventType::StudentProgress);
        events.push_back(WebhookEventType::AchievementUnlocked);
        self.client
            .register(
                owner,
                &Bytes::from_slice(&self.env, b"https://integration.example.com/hooks"),
                &BytesN::from_array(&self.env, &[0xBBu8; 32]),
                &events,
            )
            .unwrap()
    }
}

// ---------------------------------------------------------------------------
// Scenario 1: Certificate issuance triggers webhook
// ---------------------------------------------------------------------------

/// Simulates the certificate contract calling dispatch_certificate_issued
/// after successfully issuing a certificate on-chain.
#[test]
fn test_certificate_issued_triggers_webhook() {
    let t = TestEnv::new();
    let integrator = Address::generate(&t.env);
    let webhook_id = t.register_all_events(&integrator);

    // Simulate the certificate contract as the caller
    let cert_contract = Address::generate(&t.env);
    let student = Address::generate(&t.env);

    let payload = CertificateIssuedPayload {
        certificate_id: BytesN::from_array(&t.env, &[0x01u8; 32]),
        student: student.clone(),
        course_id: soroban_sdk::String::from_str(&t.env, "BLOCKCHAIN-101"),
        issued_at: 1_700_000_000,
    };

    let seqs = t.client.dispatch_certificate_issued(&cert_contract, &payload).unwrap();
    assert_eq!(seqs.len(), 1, "Expected one delivery for the registered webhook");

    let delivery = t.client.get_pending_delivery(&webhook_id, &seqs.get(0).unwrap()).unwrap();
    assert_eq!(delivery.webhook_id, webhook_id);
    assert_eq!(delivery.event_type, WebhookEventType::CertificateIssued);
    assert_eq!(delivery.attempts, 1);
}

// ---------------------------------------------------------------------------
// Scenario 2: Student progress update triggers webhook
// ---------------------------------------------------------------------------

/// Simulates the student-progress-tracker contract calling dispatch_student_progress
/// when a student completes a module.
#[test]
fn test_student_progress_triggers_webhook() {
    let t = TestEnv::new();
    let integrator = Address::generate(&t.env);
    let webhook_id = t.register_all_events(&integrator);

    let progress_contract = Address::generate(&t.env);
    let student = Address::generate(&t.env);

    let payload = StudentProgressPayload {
        student: student.clone(),
        course_id: soroban_sdk::String::from_str(&t.env, "DEFI-201"),
        progress_pct: 50,
        updated_at: 1_700_000_100,
    };

    let seqs = t.client.dispatch_student_progress(&progress_contract, &payload).unwrap();
    assert_eq!(seqs.len(), 1);

    let delivery = t.client.get_pending_delivery(&webhook_id, &seqs.get(0).unwrap()).unwrap();
    assert_eq!(delivery.event_type, WebhookEventType::StudentProgress);
}

// ---------------------------------------------------------------------------
// Scenario 3: Achievement unlock triggers webhook
// ---------------------------------------------------------------------------

/// Simulates the gamification contract calling dispatch_achievement_unlocked
/// when a student earns an achievement.
#[test]
fn test_achievement_unlocked_triggers_webhook() {
    let t = TestEnv::new();
    let integrator = Address::generate(&t.env);
    let webhook_id = t.register_all_events(&integrator);

    let gamification_contract = Address::generate(&t.env);
    let student = Address::generate(&t.env);

    let payload = AchievementUnlockedPayload {
        student: student.clone(),
        achievement_id: 7,
        unlocked_at: 1_700_000_200,
    };

    let seqs = t.client.dispatch_achievement_unlocked(&gamification_contract, &payload).unwrap();
    assert_eq!(seqs.len(), 1);

    let delivery = t.client.get_pending_delivery(&webhook_id, &seqs.get(0).unwrap()).unwrap();
    assert_eq!(delivery.event_type, WebhookEventType::AchievementUnlocked);
}

// ---------------------------------------------------------------------------
// Scenario 4: Multiple integrators receive the same event
// ---------------------------------------------------------------------------

/// Two third-party integrators (e.g. an LMS and an analytics platform) both
/// subscribe to CertificateIssued. Both should receive the delivery.
#[test]
fn test_multiple_integrators_receive_certificate_event() {
    let t = TestEnv::new();
    let lms = Address::generate(&t.env);
    let analytics_platform = Address::generate(&t.env);

    let mut cert_events = Vec::new(&t.env);
    cert_events.push_back(WebhookEventType::CertificateIssued);

    let lms_id = t
        .client
        .register(
            &lms,
            &Bytes::from_slice(&t.env, b"https://lms.example.com/hooks"),
            &BytesN::from_array(&t.env, &[0x11u8; 32]),
            &cert_events,
        )
        .unwrap();

    let mut cert_events2 = Vec::new(&t.env);
    cert_events2.push_back(WebhookEventType::CertificateIssued);

    let analytics_id = t
        .client
        .register(
            &analytics_platform,
            &Bytes::from_slice(&t.env, b"https://analytics.example.com/hooks"),
            &BytesN::from_array(&t.env, &[0x22u8; 32]),
            &cert_events2,
        )
        .unwrap();

    let cert_contract = Address::generate(&t.env);
    let student = Address::generate(&t.env);
    let payload = CertificateIssuedPayload {
        certificate_id: BytesN::from_array(&t.env, &[0xFFu8; 32]),
        student,
        course_id: soroban_sdk::String::from_str(&t.env, "RUST-ADVANCED"),
        issued_at: 1_700_000_300,
    };

    let seqs = t.client.dispatch_certificate_issued(&cert_contract, &payload).unwrap();
    assert_eq!(seqs.len(), 2, "Both integrators should receive the event");

    // Verify each integrator got a delivery
    let d0 = t.client.get_pending_delivery(&lms_id, &seqs.get(0).unwrap()).unwrap();
    let d1 = t.client.get_pending_delivery(&analytics_id, &seqs.get(1).unwrap()).unwrap();
    assert_eq!(d0.event_type, WebhookEventType::CertificateIssued);
    assert_eq!(d1.event_type, WebhookEventType::CertificateIssued);
}

// ---------------------------------------------------------------------------
// Scenario 5: Selective subscription — progress-only integrator
// ---------------------------------------------------------------------------

/// An integrator subscribes only to StudentProgress. Certificate events
/// should not create deliveries for them.
#[test]
fn test_selective_subscription_filters_events() {
    let t = TestEnv::new();
    let integrator = Address::generate(&t.env);

    let mut progress_only = Vec::new(&t.env);
    progress_only.push_back(WebhookEventType::StudentProgress);
    t.client
        .register(
            &integrator,
            &Bytes::from_slice(&t.env, b"https://progress-tracker.example.com/hooks"),
            &BytesN::from_array(&t.env, &[0x33u8; 32]),
            &progress_only,
        )
        .unwrap();

    let cert_contract = Address::generate(&t.env);
    let student = Address::generate(&t.env);
    let cert_payload = CertificateIssuedPayload {
        certificate_id: BytesN::from_array(&t.env, &[0xCCu8; 32]),
        student: student.clone(),
        course_id: soroban_sdk::String::from_str(&t.env, "SOLIDITY-101"),
        issued_at: 1_700_000_400,
    };

    let cert_seqs = t.client.dispatch_certificate_issued(&cert_contract, &cert_payload).unwrap();
    assert_eq!(cert_seqs.len(), 0, "Progress-only integrator should not receive cert events");

    // But progress events should be delivered
    let progress_payload = StudentProgressPayload {
        student,
        course_id: soroban_sdk::String::from_str(&t.env, "SOLIDITY-101"),
        progress_pct: 100,
        updated_at: 1_700_000_500,
    };
    let prog_seqs =
        t.client.dispatch_student_progress(&cert_contract, &progress_payload).unwrap();
    assert_eq!(prog_seqs.len(), 1, "Progress-only integrator should receive progress events");
}

// ---------------------------------------------------------------------------
// Scenario 6: Full retry lifecycle
// ---------------------------------------------------------------------------

/// Simulates a delivery that fails initially and is retried by a keeper
/// until it either succeeds or is exhausted.
#[test]
fn test_full_retry_lifecycle() {
    let t = TestEnv::new();
    let integrator = Address::generate(&t.env);
    let webhook_id = t.register_all_events(&integrator);

    let caller = Address::generate(&t.env);
    let student = Address::generate(&t.env);
    let payload = CertificateIssuedPayload {
        certificate_id: BytesN::from_array(&t.env, &[0xDDu8; 32]),
        student,
        course_id: soroban_sdk::String::from_str(&t.env, "KEEPER-TEST"),
        issued_at: 1_700_000_600,
    };

    let seqs = t.client.dispatch_certificate_issued(&caller, &payload).unwrap();
    let seq = seqs.get(0).unwrap();

    // Attempt 1 already happened at dispatch. Retry attempt 2.
    let d1 = t.client.get_pending_delivery(&webhook_id, &seq).unwrap();
    assert_eq!(d1.attempts, 1);

    t.env.ledger().set_sequence_number(d1.next_attempt_ledger + 1);
    t.client.retry_delivery(&webhook_id, &seq).unwrap();

    let d2 = t.client.get_pending_delivery(&webhook_id, &seq).unwrap();
    assert_eq!(d2.attempts, 2);
    assert!(d2.next_attempt_ledger > d1.next_attempt_ledger, "Backoff should increase");

    // Retry attempt 3 (final — MAX_RETRY_ATTEMPTS = 3)
    t.env.ledger().set_sequence_number(d2.next_attempt_ledger + 1);
    t.client.retry_delivery(&webhook_id, &seq).unwrap();

    // Delivery should be removed after exhaustion
    assert!(
        t.client.get_pending_delivery(&webhook_id, &seq).is_none(),
        "Exhausted delivery should be removed"
    );
}

// ---------------------------------------------------------------------------
// Scenario 7: Signing verification across event types
// ---------------------------------------------------------------------------

/// Verifies that each event type produces a unique, deterministic HMAC signature.
#[test]
fn test_signing_across_event_types() {
    let t = TestEnv::new();
    let integrator = Address::generate(&t.env);
    let webhook_id = t.register_all_events(&integrator);

    let msg_cert = Bytes::from_slice(&t.env, b"certificate_payload_hash");
    let msg_prog = Bytes::from_slice(&t.env, b"progress_payload_hash");
    let msg_ach = Bytes::from_slice(&t.env, b"achievement_payload_hash");

    let sig_cert = t.client.compute_signature(&webhook_id, &integrator, &msg_cert).unwrap();
    let sig_prog = t.client.compute_signature(&webhook_id, &integrator, &msg_prog).unwrap();
    let sig_ach = t.client.compute_signature(&webhook_id, &integrator, &msg_ach).unwrap();

    // All signatures are 32 bytes
    assert_eq!(sig_cert.len(), 32);
    assert_eq!(sig_prog.len(), 32);
    assert_eq!(sig_ach.len(), 32);

    // Each event type produces a distinct signature
    assert_ne!(sig_cert, sig_prog);
    assert_ne!(sig_prog, sig_ach);
    assert_ne!(sig_cert, sig_ach);

    // Signatures are deterministic
    let sig_cert2 = t.client.compute_signature(&webhook_id, &integrator, &msg_cert).unwrap();
    assert_eq!(sig_cert, sig_cert2);
}

// ---------------------------------------------------------------------------
// Scenario 8: Deregistered webhook stops receiving events
// ---------------------------------------------------------------------------

/// An integrator deregisters their webhook. Subsequent events should not
/// create new deliveries for them.
#[test]
fn test_deregistered_webhook_stops_receiving_events() {
    let t = TestEnv::new();
    let integrator = Address::generate(&t.env);
    let webhook_id = t.register_all_events(&integrator);

    // Deregister
    t.client.unregister(&integrator, &webhook_id).unwrap();

    let caller = Address::generate(&t.env);
    let student = Address::generate(&t.env);
    let payload = CertificateIssuedPayload {
        certificate_id: BytesN::from_array(&t.env, &[0xEEu8; 32]),
        student,
        course_id: soroban_sdk::String::from_str(&t.env, "DEREGISTER-TEST"),
        issued_at: 1_700_000_700,
    };

    let seqs = t.client.dispatch_certificate_issued(&caller, &payload).unwrap();
    assert_eq!(seqs.len(), 0, "Deregistered webhook should not receive events");
}
