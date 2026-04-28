//! End-to-End Test: Complete User Journey — Issue #441
//!
//! Covers the full lifecycle of a learner on the StrellerMinds platform:
//!   1. Signup  (admin grants Student role via access-control)
//!   2. Course enrollment  (progress contract)
//!   3. Assignment submission  (assessment contract)
//!   4. Certificate issuance  (certificate contract)
//!   5. Social sharing  (social-sharing contract)
//!
//! The test uses the Soroban `Env` test harness so it runs entirely in-process
//! without requiring a live localnet.  Each step asserts the expected on-chain
//! state before moving to the next, giving full coverage of the happy path.

#![cfg(test)]

use analytics::Analytics;
use assessment::AssessmentContract;
use certificate::CertificateContract;
use shared::access_control::AccessControl;
use shared::role_delegation::RoleDelegation;
use shared::roles::RoleLevel;
use soroban_sdk::{
    symbol_short,
    testutils::{Address as _, Ledger, LedgerInfo},
    Address, BytesN, Env, String, Symbol,
};

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn make_env() -> Env {
    let env = Env::default();
    env.mock_all_auths();
    // Set a realistic ledger timestamp (2025-01-01T00:00:00Z).
    env.ledger().set(LedgerInfo {
        timestamp: 1_735_689_600,
        protocol_version: 21,
        sequence_number: 1,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 100,
        max_entry_ttl: 6_312_000,
    });
    env
}

fn advance_time(env: &Env, seconds: u64) {
    let current = env.ledger().timestamp();
    env.ledger().set(LedgerInfo {
        timestamp: current + seconds,
        protocol_version: 21,
        sequence_number: env.ledger().sequence() + 1,
        network_id: Default::default(),
        base_reserve: 10,
        min_temp_entry_ttl: 16,
        min_persistent_entry_ttl: 100,
        max_entry_ttl: 6_312_000,
    });
}

// ─── Step 1: Signup ───────────────────────────────────────────────────────────

/// Admin initialises the access-control system and grants the new user a
/// Student role — this represents "signup" on-chain.
#[test]
fn step1_signup_grants_student_role() {
    let env = make_env();
    let admin = Address::generate(&env);
    let student = Address::generate(&env);

    AccessControl::initialize(&env, &admin).expect("init failed");
    AccessControl::grant_role(&env, &admin, &student, RoleLevel::Student)
        .expect("grant student role failed");

    let role = AccessControl::get_role(&env, &student).expect("role not found");
    assert_eq!(role.level, RoleLevel::Student);
}

// ─── Step 2: Course Enrollment ────────────────────────────────────────────────

/// After signup the student enrolls in a course.  We verify the progress
/// contract records the enrollment and initial completion is 0 %.
#[test]
fn step2_course_enrollment() {
    use progress::ProgressContract;

    let env = make_env();
    let admin = Address::generate(&env);
    let student = Address::generate(&env);
    let course_id = Symbol::new(&env, "COURSE_RUST_101");

    // Signup
    AccessControl::initialize(&env, &admin).expect("init failed");
    AccessControl::grant_role(&env, &admin, &student, RoleLevel::Student).unwrap();

    // Enroll
    ProgressContract::enroll(&env, &student, &course_id).expect("enroll failed");

    let progress = ProgressContract::get_progress(&env, &student, &course_id)
        .expect("progress not found");
    assert_eq!(progress.completion_percentage, 0);
    assert_eq!(progress.course_id, course_id);
}

// ─── Step 3: Assignment Submission ────────────────────────────────────────────

/// The student submits an assignment and receives a score.
#[test]
fn step3_assignment_submission() {
    let env = make_env();
    let admin = Address::generate(&env);
    let instructor = Address::generate(&env);
    let student = Address::generate(&env);
    let course_id = Symbol::new(&env, "COURSE_RUST_101");
    let assignment_id = Symbol::new(&env, "ASSIGN_01");

    // Setup roles
    AccessControl::initialize(&env, &admin).unwrap();
    AccessControl::grant_role(&env, &admin, &instructor, RoleLevel::Instructor).unwrap();
    AccessControl::grant_role(&env, &admin, &student, RoleLevel::Student).unwrap();

    // Create and submit assignment
    AssessmentContract::initialize(&env, &admin).expect("assessment init failed");
    AssessmentContract::create_assessment(
        &env,
        &instructor,
        &assignment_id,
        &course_id,
        100u32, // max score
    )
    .expect("create assessment failed");

    AssessmentContract::submit_assessment(&env, &student, &assignment_id, 85u32)
        .expect("submit failed");

    let result = AssessmentContract::get_result(&env, &student, &assignment_id)
        .expect("result not found");
    assert_eq!(result.score, 85);
    assert!(result.passed); // 85 >= default passing threshold
}

// ─── Step 4: Certificate Issuance ─────────────────────────────────────────────

/// Once the course is complete the instructor issues a certificate.
#[test]
fn step4_certificate_issuance() {
    let env = make_env();
    let admin = Address::generate(&env);
    let instructor = Address::generate(&env);
    let student = Address::generate(&env);
    let course_id = Symbol::new(&env, "COURSE_RUST_101");
    let cert_id: BytesN<32> = BytesN::random(&env);

    // Setup
    AccessControl::initialize(&env, &admin).unwrap();
    AccessControl::grant_role(&env, &admin, &instructor, RoleLevel::Instructor).unwrap();
    AccessControl::grant_role(&env, &admin, &student, RoleLevel::Student).unwrap();

    CertificateContract::initialize(&env, &admin).expect("cert init failed");

    // Issue certificate
    CertificateContract::issue_certificate(
        &env,
        &instructor,
        &student,
        &cert_id,
        &course_id,
        &String::from_str(&env, "Rust 101 — Completion Certificate"),
    )
    .expect("issue failed");

    let cert = CertificateContract::get_certificate(&env, &cert_id).expect("cert not found");
    assert_eq!(cert.recipient, student);
    assert_eq!(cert.course_id, course_id);
    assert!(cert.issued_at > 0);
}

// ─── Step 5: Social Sharing ───────────────────────────────────────────────────

/// The student shares their certificate on-chain.
#[test]
fn step5_social_sharing() {
    use social_sharing::SocialSharingContract;

    let env = make_env();
    let admin = Address::generate(&env);
    let instructor = Address::generate(&env);
    let student = Address::generate(&env);
    let course_id = Symbol::new(&env, "COURSE_RUST_101");
    let cert_id: BytesN<32> = BytesN::random(&env);

    // Setup
    AccessControl::initialize(&env, &admin).unwrap();
    AccessControl::grant_role(&env, &admin, &instructor, RoleLevel::Instructor).unwrap();
    AccessControl::grant_role(&env, &admin, &student, RoleLevel::Student).unwrap();

    CertificateContract::initialize(&env, &admin).unwrap();
    CertificateContract::issue_certificate(
        &env,
        &instructor,
        &student,
        &cert_id,
        &course_id,
        &String::from_str(&env, "Rust 101 — Completion Certificate"),
    )
    .unwrap();

    // Share
    SocialSharingContract::initialize(&env, &admin).expect("sharing init failed");
    SocialSharingContract::share_certificate(
        &env,
        &student,
        &cert_id,
        &String::from_str(&env, "Just completed Rust 101! 🎉"),
    )
    .expect("share failed");

    let shares = SocialSharingContract::get_shares(&env, &student);
    assert_eq!(shares.len(), 1);
}

// ─── Full Journey (combined) ──────────────────────────────────────────────────

/// Runs all five steps in sequence to validate the complete user journey.
#[test]
fn test_complete_user_journey() {
    use progress::ProgressContract;
    use social_sharing::SocialSharingContract;

    let env = make_env();
    let admin = Address::generate(&env);
    let instructor = Address::generate(&env);
    let student = Address::generate(&env);
    let course_id = Symbol::new(&env, "COURSE_RUST_101");
    let assignment_id = Symbol::new(&env, "ASSIGN_01");
    let cert_id: BytesN<32> = BytesN::random(&env);

    // ── 1. Signup ──────────────────────────────────────────────────────────
    AccessControl::initialize(&env, &admin).unwrap();
    AccessControl::grant_role(&env, &admin, &instructor, RoleLevel::Instructor).unwrap();
    AccessControl::grant_role(&env, &admin, &student, RoleLevel::Student).unwrap();

    let role = AccessControl::get_role(&env, &student).unwrap();
    assert_eq!(role.level, RoleLevel::Student, "Step 1 failed: wrong role");

    // ── 2. Course Enrollment ───────────────────────────────────────────────
    ProgressContract::enroll(&env, &student, &course_id).unwrap();
    let progress = ProgressContract::get_progress(&env, &student, &course_id).unwrap();
    assert_eq!(progress.completion_percentage, 0, "Step 2 failed: non-zero initial progress");

    advance_time(&env, 3600); // 1 hour passes

    // ── 3. Assignment Submission ───────────────────────────────────────────
    AssessmentContract::initialize(&env, &admin).unwrap();
    AssessmentContract::create_assessment(&env, &instructor, &assignment_id, &course_id, 100u32)
        .unwrap();
    AssessmentContract::submit_assessment(&env, &student, &assignment_id, 90u32).unwrap();

    let result = AssessmentContract::get_result(&env, &student, &assignment_id).unwrap();
    assert!(result.passed, "Step 3 failed: assignment not passed");

    // Mark course complete
    ProgressContract::update_progress(&env, &student, &course_id, 100u32).unwrap();
    let progress = ProgressContract::get_progress(&env, &student, &course_id).unwrap();
    assert_eq!(progress.completion_percentage, 100, "Step 3 failed: course not complete");

    advance_time(&env, 60); // brief pause

    // ── 4. Certificate Issuance ────────────────────────────────────────────
    CertificateContract::initialize(&env, &admin).unwrap();
    CertificateContract::issue_certificate(
        &env,
        &instructor,
        &student,
        &cert_id,
        &course_id,
        &String::from_str(&env, "Rust 101 — Completion Certificate"),
    )
    .unwrap();

    let cert = CertificateContract::get_certificate(&env, &cert_id).unwrap();
    assert_eq!(cert.recipient, student, "Step 4 failed: wrong recipient");

    // ── 5. Social Sharing ──────────────────────────────────────────────────
    SocialSharingContract::initialize(&env, &admin).unwrap();
    SocialSharingContract::share_certificate(
        &env,
        &student,
        &cert_id,
        &String::from_str(&env, "Just completed Rust 101! 🎉"),
    )
    .unwrap();

    let shares = SocialSharingContract::get_shares(&env, &student);
    assert_eq!(shares.len(), 1, "Step 5 failed: share not recorded");

    println!("✅ Complete user journey passed all 5 steps.");
}

// ─── Role Delegation in Journey ───────────────────────────────────────────────

/// Verifies that an instructor can delegate their role to a TA and the TA can
/// issue a certificate on their behalf (delegation feature from Issue #443).
#[test]
fn test_journey_with_role_delegation() {
    let env = make_env();
    let admin = Address::generate(&env);
    let instructor = Address::generate(&env);
    let ta = Address::generate(&env); // teaching assistant
    let student = Address::generate(&env);
    let course_id = Symbol::new(&env, "COURSE_RUST_101");
    let cert_id: BytesN<32> = BytesN::random(&env);
    let now = env.ledger().timestamp();

    // Setup roles
    AccessControl::initialize(&env, &admin).unwrap();
    AccessControl::grant_role(&env, &admin, &instructor, RoleLevel::Instructor).unwrap();
    AccessControl::grant_role(&env, &admin, &student, RoleLevel::Student).unwrap();

    // Instructor delegates to TA for 1 hour
    RoleDelegation::delegate_role(
        &env,
        &instructor,
        &ta,
        RoleLevel::Instructor,
        Some(now + 3600),
    )
    .expect("delegation failed");

    assert!(
        RoleDelegation::has_valid_delegation(&env, &ta, &RoleLevel::Instructor),
        "TA should have valid delegation"
    );

    // TA issues certificate using delegated authority
    CertificateContract::initialize(&env, &admin).unwrap();
    CertificateContract::issue_certificate(
        &env,
        &ta, // acting under delegation
        &student,
        &cert_id,
        &course_id,
        &String::from_str(&env, "Rust 101 — Completion Certificate"),
    )
    .expect("TA certificate issuance failed");

    let cert = CertificateContract::get_certificate(&env, &cert_id).unwrap();
    assert_eq!(cert.recipient, student);

    // Audit log should have the delegation entry
    let log = RoleDelegation::get_audit_log(&env, &instructor);
    assert_eq!(log.len(), 1);
}
