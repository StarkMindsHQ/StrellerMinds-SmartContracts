# Student Progress Tracker Contract

## Purpose

The Student Progress Tracker contract provides fine-grained, per-module progress tracking for students on the StrellerMinds platform. Unlike the `progress` contract which stores a single aggregate percentage per course, this contract stores a map of module completion percentages keyed by `(student, course_id)` — enabling detailed curriculum dashboards, adaptive learning paths, and per-module gating logic. All updates emit `ProgressUpdated` events consumed by analytics and gamification contracts.

## Architecture

| Module | Description |
|---|---|
| `src/lib.rs` | Contract entrypoint — defines the `Progress` and `DataKey` types inline and exposes `initialize`, `update_progress`, `get_progress`, and `get_admin` |
| `src/errors.rs` | `StudentProgressError` enum covering initialization state, authorization, admin state, and validation |
| `src/gas_optimized.rs` | Gas-optimized batch update utilities |
| `src/test.rs` | Unit test suite |

## Public API

| Function | Description | Auth Required |
|---|---|---|
| `initialize(admin)` | One-time setup; sets the admin address in instance storage | Yes — `admin` must sign |
| `update_progress(student, course_id, module_id, percent)` | Stores or updates the completion percentage (0–100) for a specific module within a course | Yes — `student` (or admin if student is admin) |
| `get_progress(student, course_id)` | Returns a `Map<Symbol, u32>` of module IDs to completion percentages; empty map if none recorded | No |
| `get_admin()` | Returns the stored admin address | No |

## Usage Example

```text
# Initialize
tracker.initialize(admin_address)

# Student completes 80% of module-1 in course rust-101
tracker.update_progress(student_address, "rust-101", "module-1", 80)

# Student finishes module-1 and starts module-2
tracker.update_progress(student_address, "rust-101", "module-1", 100)
tracker.update_progress(student_address, "rust-101", "module-2", 30)

# Query all module progress for this student in rust-101
progress_map = tracker.get_progress(student_address, "rust-101")
# returns { "module-1": 100, "module-2": 30 }
```

## Errors

For the full error code reference and conventions, see [ERROR_HANDLING.md](../../docs/ERROR_HANDLING.md).

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | `initialize` has already been called |
| 2 | `NotInitialized` | Contract has not been initialized yet |
| 10 | `Unauthorized` | Caller does not have the required permissions |
| 11 | `AdminNotSet` | Admin address is missing from storage (contract not initialized) |
| 20 | `InvalidPercent` | Supplied completion percentage exceeds 100 |

## Integration

| Contract | Relationship |
|---|---|
| `progress` | Complementary contract storing single aggregate percentages per course; this contract handles per-module granularity |
| `analytics` | Consumes `ProgressUpdated` events for module-level engagement and time-on-task reporting |
| `gamification` | May read module completion maps to unlock module-specific achievements and XP rewards |
| `shared` | Uses event schema macros (`emit_progress_event!`, `emit_access_control_event!`) |
