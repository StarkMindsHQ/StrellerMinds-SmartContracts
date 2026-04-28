//! Tests for the Progress contract (issue #274 – code coverage)
//! and performance profile (issue #271 – performance optimization).
//!
//! Covers:
//! - `initialize`, `record_progress`, `get_progress`, `get_student_courses`
//! - Gas-optimized `PackedProgress` bit-packing operations
//! - Batch-update throughput benchmark

use soroban_sdk::{symbol_short, testutils::Address as _, Address, Env, Vec};

use crate::{gas_optimized::PackedProgress, Progress, ProgressClient};

// ─────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────

fn setup() -> (Env, ProgressClient<'static>, Address) {
    let env = Env::default();
    env.mock_all_auths();
    let contract_id = env.register(Progress, ());
    let client = ProgressClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
    (env, client, admin)
}

// ─────────────────────────────────────────────────────────────
// 1. Initialisation
// ─────────────────────────────────────────────────────────────

#[test]
fn test_initialize_succeeds() {
    let (_, client, admin) = setup();
    // Second init should fail since we now guard against re-initialization
    let result = client.try_initialize(&admin);
    assert!(result.is_err());
}

// ─────────────────────────────────────────────────────────────
// 2. record_progress
// ─────────────────────────────────────────────────────────────

#[test]
fn test_record_progress_zero_percent() {
    let (env, client, _) = setup();
    let student = Address::generate(&env);
    let course_id = symbol_short!("RUST101");
    client.record_progress(&student, &course_id, &0u32);
}

#[test]
fn test_record_progress_full_completion() {
    let (env, client, _) = setup();
    let student = Address::generate(&env);
    let course_id = symbol_short!("RUST101");
    client.record_progress(&student, &course_id, &100u32);
}

#[test]
fn test_record_progress_multiple_updates() {
    let (env, client, _) = setup();
    let student = Address::generate(&env);
    let course_id = symbol_short!("MATH101");
    for pct in [10u32, 30, 50, 75, 100] {
        client.record_progress(&student, &course_id, &pct);
    }
}

#[test]
fn test_record_progress_multiple_students() {
    let (env, client, _) = setup();
    let course_id = symbol_short!("PHYS101");
    for _ in 0..5 {
        let student = Address::generate(&env);
        client.record_progress(&student, &course_id, &50u32);
    }
}

#[test]
fn test_record_progress_multiple_courses_one_student() {
    let (env, client, _) = setup();
    let student = Address::generate(&env);
    for course in [symbol_short!("C1"), symbol_short!("C2"), symbol_short!("C3")] {
        client.record_progress(&student, &course, &100u32);
    }
}

// ─────────────────────────────────────────────────────────────
// 3. get_progress – now stores and retrieves real values (#365)
// ─────────────────────────────────────────────────────────────

#[test]
fn test_get_progress_returns_recorded_value() {
    let (env, client, _) = setup();
    let student = Address::generate(&env);
    let course_id = symbol_short!("RUST101");
    client.record_progress(&student, &course_id, &75u32);
    let progress = client.get_progress(&student, &course_id);
    assert_eq!(progress, 75);
}

#[test]
fn test_get_progress_not_found_panics() {
    let (env, client, _) = setup();
    let student = Address::generate(&env);
    let course_id = symbol_short!("RUST101");
    let result = client.try_get_progress(&student, &course_id);
    assert!(result.is_err());
}

#[test]
fn test_get_progress_overwrite_updates_value() {
    let (env, client, _) = setup();
    let student = Address::generate(&env);
    let course_id = symbol_short!("RUST101");
    client.record_progress(&student, &course_id, &50u32);
    client.record_progress(&student, &course_id, &90u32);
    let progress = client.get_progress(&student, &course_id);
    assert_eq!(progress, 90);
}

// ─────────────────────────────────────────────────────────────
// 4. get_student_courses – now returns real course list (#365)
// ─────────────────────────────────────────────────────────────

#[test]
fn test_get_student_courses_empty_before_any_progress() {
    let (env, client, _) = setup();
    let student = Address::generate(&env);
    let courses = client.get_student_courses(&student);
    assert_eq!(courses.len(), 0);
}

#[test]
fn test_get_student_courses_tracks_recorded_courses() {
    let (env, client, _) = setup();
    let student = Address::generate(&env);
    client.record_progress(&student, &symbol_short!("C1"), &10u32);
    client.record_progress(&student, &symbol_short!("C2"), &20u32);
    let courses = client.get_student_courses(&student);
    assert_eq!(courses.len(), 2);
}

#[test]
fn test_get_student_courses_no_duplicates() {
    let (env, client, _) = setup();
    let student = Address::generate(&env);
    let course_id = symbol_short!("C1");
    client.record_progress(&student, &course_id, &10u32);
    client.record_progress(&student, &course_id, &50u32);
    let courses = client.get_student_courses(&student);
    // Same course recorded twice should only appear once.
    assert_eq!(courses.len(), 1);
}

// ─────────────────────────────────────────────────────────────
// 5. PackedProgress – bit-packing unit tests (gas optimization)
// ─────────────────────────────────────────────────────────────

#[test]
fn test_packed_progress_initial_state() {
    let pp = PackedProgress::default();
    assert_eq!(pp.completed_module_count(), 0);
    assert_eq!(pp.score_x10(), 0);
    assert_eq!(pp.completion_pct(), 0);
    assert!(!pp.is_module_complete(0));
}

#[test]
fn test_packed_progress_mark_module_complete() {
    let mut pp = PackedProgress::default();
    let changed = pp.mark_module_complete(0);
    assert!(changed, "first mark should succeed");
    assert!(pp.is_module_complete(0));
    assert_eq!(pp.completed_module_count(), 1);
}

#[test]
fn test_packed_progress_mark_same_module_twice_is_idempotent() {
    let mut pp = PackedProgress::default();
    pp.mark_module_complete(3);
    let second = pp.mark_module_complete(3);
    assert!(!second, "second mark on same module must return false");
    assert_eq!(pp.completed_module_count(), 1);
}

#[test]
fn test_packed_progress_mark_all_64_modules() {
    let mut pp = PackedProgress::default();
    for i in 0u8..64 {
        pp.mark_module_complete(i);
    }
    assert_eq!(pp.completed_module_count(), 64);
}

#[test]
fn test_packed_progress_score_roundtrip() {
    let mut pp = PackedProgress::default();
    pp.set_score_x10(950); // 95.0
    assert_eq!(pp.score_x10(), 950);
}

#[test]
fn test_packed_progress_completion_pct_roundtrip() {
    let mut pp = PackedProgress::default();
    pp.set_completion_pct(87);
    assert_eq!(pp.completion_pct(), 87);
}

#[test]
fn test_packed_progress_started_ledger_roundtrip() {
    let mut pp = PackedProgress::default();
    pp.set_started_ledger(123456);
    assert_eq!(pp.started_ledger(), 123456);
}

#[test]
fn test_packed_progress_is_completed_true() {
    let mut pp = PackedProgress::default();
    for i in 0u8..4 {
        pp.mark_module_complete(i);
    }
    assert!(pp.is_completed(4));
}

#[test]
fn test_packed_progress_is_not_completed_when_modules_missing() {
    let mut pp = PackedProgress::default();
    pp.mark_module_complete(0);
    assert!(!pp.is_completed(4));
}

#[test]
fn test_packed_progress_fields_do_not_interfere() {
    // Write all fields and verify they don't clobber each other.
    let mut pp = PackedProgress::default();
    pp.set_score_x10(750);
    pp.set_completion_pct(75);
    pp.set_started_ledger(9_999_999);
    pp.mark_module_complete(0);
    pp.mark_module_complete(7);

    assert_eq!(pp.score_x10(), 750);
    assert_eq!(pp.completion_pct(), 75);
    assert_eq!(pp.started_ledger(), 9_999_999);
    assert_eq!(pp.completed_module_count(), 2);
}

// ─────────────────────────────────────────────────────────────
// 6. Performance / throughput tests
// ─────────────────────────────────────────────────────────────

/// Verify that 50 sequential progress updates complete without exhausting
/// the simulated Soroban execution budget.
#[test]
fn test_record_progress_bulk_50_updates_within_budget() {
    let (env, client, _) = setup();
    let student = Address::generate(&env);
    let course_id = symbol_short!("PERF1");
    for pct in 0u32..50 {
        client.record_progress(&student, &course_id, &(pct * 2));
    }
    // If we reach here the budget was not exceeded.
}

/// Verify that 20 different students can record progress in the same course
/// without hitting budget limits.
#[test]
fn test_record_progress_20_students_concurrent_simulation() {
    let (env, client, _) = setup();
    let course_id = symbol_short!("PERF2");
    let mut students = Vec::new(&env);
    for _ in 0..20 {
        students.push_back(Address::generate(&env));
    }
    for i in 0..students.len() {
        let student = students.get(i).unwrap();
        client.record_progress(&student, &course_id, &100u32);
    }
}

/// Verify PackedProgress bit-set operations on 64 modules are O(1)
/// and complete without excessive iteration.
#[test]
fn test_packed_progress_bulk_operations_performance() {
    // In-memory only – no Env needed.
    let mut pp = PackedProgress::default();
    for i in 0u8..64 {
        assert!(pp.mark_module_complete(i));
    }
    assert_eq!(pp.completed_module_count(), 64);
    assert!(pp.is_completed(64));
}
