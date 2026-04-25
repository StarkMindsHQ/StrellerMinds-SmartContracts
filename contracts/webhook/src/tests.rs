//! Comprehensive tests for the Webhook contract.
//!
//! Covers:
//! - Initialization and re-initialization guard
//! - Webhook registration, deregistration, and limits
//! - Event dispatch: CertificateIssued, StudentProgress, AchievementUnlocked
//! - Retry mechanism: backoff enforcement, attempt limits, exhaustion
//! - HMAC signing: compute_signature authorization
//! - Error cases: unauthorized access, missing webhooks, inactive webhooks
//!
//! NOTE: soroban-sdk generated client methods return `T` directly (panic on
//! contract error). Use `try_*` variants to get `Result<T, _>` for error paths.

#![cfg(test)]

use soroban_sdk::{
    testutils::{Address as _, Ledger},
    Address, Bytes, BytesN, Env, Vec,
};

use crate::{
    types::{
        AchievementUnlockedPayload, CertificateIssuedPayload, StudentProgressPayload,
        WebhookEventType, MAX_RETRY_ATTEMPTS, MAX_WEBHOOKS_PER_OWNER, RETRY_BACKOFF_LEDGERS,
    },
    WebhookContract, WebhookContractClient,
};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn setup() -> (Env, WebhookContractClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(WebhookContract, ());
    let client = WebhookContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

fn make_secret(env: &Env) -> BytesN<32> {
    BytesN::from_array(env, &[0x42u8; 32])
}

fn make_url(env: &Env) -> Bytes {
    Bytes::from_slice(env, b"https://example.com/webhook")
}

fn event_types_cert(env: &Env) -> soroban_sdk::Vec<WebhookEventType> {
    let mut v = Vec::new(env);
    v.push_back(WebhookEventType::CertificateIssued);
    v
}

fn event_types_all(env: &Env) -> soroban_sdk::Vec<WebhookEventType> {
    let mut v = Vec::new(env);
    v.push_back(WebhookEventType::CertificateIssued);
    v.push_back(WebhookEventType::StudentProgress);
    v.push_back(WebhookEventType::AchievementUnlocked);
    v
}

fn cert_payload(env: &Env, student: &Address) -> CertificateIssuedPayload {
    CertificateIssuedPayload {
        certificate_id: BytesN::from_array(env, &[0xAAu8; 32]),
        student: student.clone(),
        course_id: soroban_sdk::String::from_str(env, "RUST101"),
        issued_at: 1_000_000,
    }
}

fn progress_payload(env: &Env, student: &Address) -> StudentProgressPayload {
    StudentProgressPayload {
        student: student.clone(),
        course_id: soroban_sdk::String::from_str(env, "RUST101"),
        progress_pct: 75,
        updated_at: 1_000_001,
    }
}

fn achievement_payload(env: &Env, student: &Address) -> AchievementUnlockedPayload {
    AchievementUnlockedPayload {
        student: student.clone(),
        achievement_id: 42,
        unlocked_at: 1_000_002,
    }
}

// ---------------------------------------------------------------------------
// Initialization
// ---------------------------------------------------------------------------

#[test]
fn test_initialize_succeeds() {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(WebhookContract, ());
    let client = WebhookContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin); // panics on failure
}

#[test]
fn test_initialize_twice_fails() {
    let (_, client, admin) = setup();
    assert!(client.try_initialize(&admin).is_err());
}

// ---------------------------------------------------------------------------
// Registration
// ---------------------------------------------------------------------------

#[test]
fn test_register_webhook_succeeds() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let id = client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));
    assert_eq!(id, 0);
}

#[test]
fn test_register_returns_incrementing_ids() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let id0 =
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));
    let id1 =
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));
    assert_eq!(id0, 0);
    assert_eq!(id1, 1);
}

#[test]
fn test_register_empty_url_fails() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    assert!(client
        .try_register(&owner, &Bytes::new(&env), &make_secret(&env), &event_types_cert(&env))
        .is_err());
}

#[test]
fn test_register_empty_event_types_fails() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    assert!(client
        .try_register(&owner, &make_url(&env), &make_secret(&env), &Vec::new(&env))
        .is_err());
}

#[test]
fn test_register_too_many_webhooks_fails() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    for _ in 0..MAX_WEBHOOKS_PER_OWNER {
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));
    }
    assert!(client
        .try_register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env))
        .is_err());
}

#[test]
fn test_get_webhook_after_register() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let id =
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));
    let wh = client.get_webhook(&id);
    assert_eq!(wh.id, id);
    assert_eq!(wh.owner, owner);
    assert!(wh.active);
}

#[test]
fn test_get_owner_webhooks() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let id0 =
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));
    let id1 =
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));
    let ids = client.get_owner_webhooks(&owner);
    assert_eq!(ids.len(), 2);
    assert_eq!(ids.get(0).unwrap(), id0);
    assert_eq!(ids.get(1).unwrap(), id1);
}

// ---------------------------------------------------------------------------
// Unregister
// ---------------------------------------------------------------------------

#[test]
fn test_unregister_deactivates_webhook() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let id =
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));
    client.unregister(&owner, &id);
    let wh = client.get_webhook(&id);
    assert!(!wh.active);
}

#[test]
fn test_unregister_wrong_owner_fails() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let other = Address::generate(&env);
    let id =
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));
    assert!(client.try_unregister(&other, &id).is_err());
}

#[test]
fn test_unregister_nonexistent_fails() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    assert!(client.try_unregister(&owner, &999u32).is_err());
}

// ---------------------------------------------------------------------------
// Event dispatch – CertificateIssued
// ---------------------------------------------------------------------------

#[test]
fn test_dispatch_certificate_issued_no_subscribers() {
    let (env, client, _) = setup();
    let caller = Address::generate(&env);
    let student = Address::generate(&env);
    let seqs = client.dispatch_certificate_issued(&caller, &cert_payload(&env, &student));
    assert_eq!(seqs.len(), 0);
}

#[test]
fn test_dispatch_certificate_issued_creates_delivery() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let id =
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));

    let caller = Address::generate(&env);
    let student = Address::generate(&env);
    let seqs = client.dispatch_certificate_issued(&caller, &cert_payload(&env, &student));
    assert_eq!(seqs.len(), 1);

    let delivery = client.get_pending_delivery(&id, &seqs.get(0).unwrap()).unwrap();
    assert_eq!(delivery.webhook_id, id);
    assert_eq!(delivery.attempts, 1);
}

#[test]
fn test_dispatch_certificate_not_sent_to_wrong_event_type() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let mut v = Vec::new(&env);
    v.push_back(WebhookEventType::StudentProgress);
    client.register(&owner, &make_url(&env), &make_secret(&env), &v);

    let caller = Address::generate(&env);
    let student = Address::generate(&env);
    let seqs = client.dispatch_certificate_issued(&caller, &cert_payload(&env, &student));
    assert_eq!(seqs.len(), 0);
}

// ---------------------------------------------------------------------------
// Event dispatch – StudentProgress
// ---------------------------------------------------------------------------

#[test]
fn test_dispatch_student_progress_creates_delivery() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let mut v = Vec::new(&env);
    v.push_back(WebhookEventType::StudentProgress);
    let id = client.register(&owner, &make_url(&env), &make_secret(&env), &v);

    let caller = Address::generate(&env);
    let student = Address::generate(&env);
    let seqs = client.dispatch_student_progress(&caller, &progress_payload(&env, &student));
    assert_eq!(seqs.len(), 1);

    let delivery = client.get_pending_delivery(&id, &seqs.get(0).unwrap()).unwrap();
    assert_eq!(delivery.event_type, WebhookEventType::StudentProgress);
}

// ---------------------------------------------------------------------------
// Event dispatch – AchievementUnlocked
// ---------------------------------------------------------------------------

#[test]
fn test_dispatch_achievement_unlocked_creates_delivery() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let mut v = Vec::new(&env);
    v.push_back(WebhookEventType::AchievementUnlocked);
    let id = client.register(&owner, &make_url(&env), &make_secret(&env), &v);

    let caller = Address::generate(&env);
    let student = Address::generate(&env);
    let seqs = client.dispatch_achievement_unlocked(&caller, &achievement_payload(&env, &student));
    assert_eq!(seqs.len(), 1);

    let delivery = client.get_pending_delivery(&id, &seqs.get(0).unwrap()).unwrap();
    assert_eq!(delivery.event_type, WebhookEventType::AchievementUnlocked);
}

// ---------------------------------------------------------------------------
// Multiple webhooks
// ---------------------------------------------------------------------------

#[test]
fn test_dispatch_to_multiple_subscribers() {
    let (env, client, _) = setup();
    let owner1 = Address::generate(&env);
    let owner2 = Address::generate(&env);
    client.register(&owner1, &make_url(&env), &make_secret(&env), &event_types_all(&env));
    client.register(&owner2, &make_url(&env), &make_secret(&env), &event_types_all(&env));

    let caller = Address::generate(&env);
    let student = Address::generate(&env);
    let seqs = client.dispatch_certificate_issued(&caller, &cert_payload(&env, &student));
    assert_eq!(seqs.len(), 2);
}

#[test]
fn test_inactive_webhook_skipped_on_dispatch() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let id =
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));
    client.unregister(&owner, &id);

    let caller = Address::generate(&env);
    let student = Address::generate(&env);
    let seqs = client.dispatch_certificate_issued(&caller, &cert_payload(&env, &student));
    assert_eq!(seqs.len(), 0);
}

// ---------------------------------------------------------------------------
// Retry mechanism
// ---------------------------------------------------------------------------

#[test]
fn test_retry_too_early_fails() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let id =
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));

    let caller = Address::generate(&env);
    let student = Address::generate(&env);
    let seqs = client.dispatch_certificate_issued(&caller, &cert_payload(&env, &student));
    let seq = seqs.get(0).unwrap();

    assert!(client.try_retry_delivery(&id, &seq).is_err());
}

#[test]
fn test_retry_after_backoff_succeeds() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let id =
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));

    let caller = Address::generate(&env);
    let student = Address::generate(&env);
    let seqs = client.dispatch_certificate_issued(&caller, &cert_payload(&env, &student));
    let seq = seqs.get(0).unwrap();

    env.ledger().set_sequence_number(RETRY_BACKOFF_LEDGERS + 10);
    client.retry_delivery(&id, &seq);
}

#[test]
fn test_retry_increments_attempts() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let id =
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));

    let caller = Address::generate(&env);
    let student = Address::generate(&env);
    let seqs = client.dispatch_certificate_issued(&caller, &cert_payload(&env, &student));
    let seq = seqs.get(0).unwrap();

    env.ledger().set_sequence_number(RETRY_BACKOFF_LEDGERS + 10);
    client.retry_delivery(&id, &seq);

    let delivery = client.get_pending_delivery(&id, &seq).unwrap();
    assert_eq!(delivery.attempts, 2);
}

#[test]
fn test_retry_exhaustion_removes_delivery() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let id =
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));

    let caller = Address::generate(&env);
    let student = Address::generate(&env);
    let seqs = client.dispatch_certificate_issued(&caller, &cert_payload(&env, &student));
    let seq = seqs.get(0).unwrap();

    // Exhaust remaining retries (starts at attempts=1, max=3)
    for _ in 1..MAX_RETRY_ATTEMPTS {
        let delivery = client.get_pending_delivery(&id, &seq).unwrap();
        env.ledger().set_sequence_number(delivery.next_attempt_ledger + 1);
        client.retry_delivery(&id, &seq);
    }

    assert!(client.get_pending_delivery(&id, &seq).is_none());
}

#[test]
fn test_retry_nonexistent_delivery_fails() {
    let (_, client, _) = setup();
    assert!(client.try_retry_delivery(&0u32, &999u32).is_err());
}

#[test]
fn test_retry_inactive_webhook_fails() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let id =
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));

    let caller = Address::generate(&env);
    let student = Address::generate(&env);
    let seqs = client.dispatch_certificate_issued(&caller, &cert_payload(&env, &student));
    let seq = seqs.get(0).unwrap();

    client.unregister(&owner, &id);
    env.ledger().set_sequence_number(RETRY_BACKOFF_LEDGERS + 10);
    assert!(client.try_retry_delivery(&id, &seq).is_err());
}

// ---------------------------------------------------------------------------
// Signing
// ---------------------------------------------------------------------------

#[test]
fn test_compute_signature_returns_bytes() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let id =
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));

    let message = Bytes::from_slice(&env, b"hello webhook");
    let sig = client.compute_signature(&id, &owner, &message);
    assert_eq!(sig.len(), 32);
}

#[test]
fn test_compute_signature_deterministic() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let id =
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));

    let message = Bytes::from_slice(&env, b"deterministic");
    let sig1 = client.compute_signature(&id, &owner, &message);
    let sig2 = client.compute_signature(&id, &owner, &message);
    assert_eq!(sig1, sig2);
}

#[test]
fn test_compute_signature_different_messages_differ() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let id =
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));

    let msg1 = Bytes::from_slice(&env, b"message one");
    let msg2 = Bytes::from_slice(&env, b"message two");
    let sig1 = client.compute_signature(&id, &owner, &msg1);
    let sig2 = client.compute_signature(&id, &owner, &msg2);
    assert_ne!(sig1, sig2);
}

#[test]
fn test_compute_signature_wrong_owner_fails() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let other = Address::generate(&env);
    let id =
        client.register(&owner, &make_url(&env), &make_secret(&env), &event_types_cert(&env));

    let message = Bytes::from_slice(&env, b"test");
    assert!(client.try_compute_signature(&id, &other, &message).is_err());
}

// ---------------------------------------------------------------------------
// Queries on missing data
// ---------------------------------------------------------------------------

#[test]
fn test_get_nonexistent_webhook_fails() {
    let (_, client, _) = setup();
    assert!(client.try_get_webhook(&999u32).is_err());
}

#[test]
fn test_get_pending_delivery_none_when_missing() {
    let (_, client, _) = setup();
    assert!(client.get_pending_delivery(&0u32, &0u32).is_none());
}

#[test]
fn test_get_owner_webhooks_empty_for_new_owner() {
    let (env, client, _) = setup();
    let owner = Address::generate(&env);
    let ids = client.get_owner_webhooks(&owner);
    assert_eq!(ids.len(), 0);
}
