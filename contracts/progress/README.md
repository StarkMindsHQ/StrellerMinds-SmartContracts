# Progress Contract

## Purpose

The Progress contract is the lightweight course-progress registry for the StrellerMinds platform. It records and retrieves a student's completion percentage for any given course, emitting a `ProgressUpdated` event on every write so that downstream systems (analytics, gamification) can react in real time. It is designed as a simple, high-throughput store for aggregate progress values; fine-grained module-level tracking is handled by the `student-progress-tracker` contract.

## Architecture

| Module | Description |
|---|---|
| `src/lib.rs` | Contract entrypoint — exposes `initialize`, `record_progress`, `get_progress`, and `get_student_courses`; emits `ProgressUpdated` events via shared macros |
| `src/errors.rs` | `ProgressError` enum covering initialization state, authorization, validation, and look-up failures |
| `src/gas_optimized.rs` | Gas-optimized variants for batch progress updates |
| `src/tests.rs` | Unit test suite |

## Public API

| Function | Description | Auth Required |
|---|---|---|
| `initialize(admin)` | One-time setup; records the admin address | No (open, call once) |
| `record_progress(student, course_id, progress)` | Stores `progress` (0–100) for `student` in `course_id`; emits `ProgressUpdated` | Yes — admin or authorized caller |
| `get_progress(student, course_id)` | Returns the stored completion percentage for a student/course pair | No |
| `get_student_courses(student)` | Returns all course IDs for which `student` has recorded progress | No |

## Usage Example

```text
# Initialize contract
progress.initialize(admin_address)

# Instructor records that a student is 75% through a course
progress.record_progress(student_address, "rust-101", 75)

# Query progress
pct = progress.get_progress(student_address, "rust-101")  # returns 75

# List all courses the student has started
courses = progress.get_student_courses(student_address)
```

## Errors

For the full error code reference and conventions, see [ERROR_HANDLING.md](../../docs/ERROR_HANDLING.md).

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | `initialize` has already been called |
| 2 | `NotInitialized` | Contract has not been initialized yet |
| 10 | `Unauthorized` | Caller does not have the required permissions |
| 20 | `InvalidProgress` | Progress value is outside the 0–100 range |
| 21 | `InvalidCourseId` | Supplied course ID is empty or malformed |
| 50 | `ProgressNotFound` | No progress record found for this student/course pair |

## Integration

| Contract | Relationship |
|---|---|
| `student-progress-tracker` | Complementary contract for per-module progress; this contract tracks aggregate percentages |
| `analytics` | Consumes `ProgressUpdated` events for learning velocity and completion rate reports |
| `gamification` | May read progress to gate achievement unlocks and XP awards |
| `shared` | Uses event schema macros (`emit_progress_event!`, `emit_access_control_event!`) |
