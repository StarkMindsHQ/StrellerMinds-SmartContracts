# Progress Contract

## Overview
A simple and efficient contract for tracking student progress through educational courses. This contract manages course enrollment, module completion tracking, and progress validation with built-in safeguards against invalid progress updates.

## Interface

### Core Functions
```rust
// Initialize the contract with admin
fn initialize(env: Env, admin: Address) -> Result<(), Error>

// Add a new course with total modules
fn add_course(env: Env, course_id: Symbol, total_modules: u32) -> Result<(), Error>

// Update user progress for a course module
fn update_progress(env: Env, user: Address, course_id: Symbol, module: u32, completed: bool) -> Result<(), Error>

// Get user progress for a course
fn get_progress(env: Env, user: Address, course_id: Symbol) -> Result<Vec<bool>, Error>

// Get course total modules
fn get_course_modules(env: Env, course_id: Symbol) -> Result<u32, Error>

// Get completion percentage for a course
fn get_completion_percentage(env: Env, user: Address, course_id: Symbol) -> Result<u32, Error>
```

## Events

### Progress Events
- `progress_update`: Emitted when user progress is successfully updated
- `error`: Emitted when invalid progress attempts are made
  - `invalid_module`: When module number is out of range
  - `already_completed`: When trying to complete an already completed module
  - `non_increasing`: When trying to decrease progress (mark completed as incomplete)

## Configuration

### Course Configuration
- `course_id`: Symbol identifying the course
- `total_modules`: Total number of modules in the course (1-indexed)

### Progress Validation Rules
- **Module Range**: Modules must be between 1 and total_modules
- **Non-Decreasing Progress**: Once a module is marked complete, it cannot be marked incomplete
- **No Duplicate Completion**: Cannot mark an already completed module as complete again

### Storage Structure
- **Instance Storage**: Admin address and course definitions
- **Persistent Storage**: User progress data organized by user and course

## Testing

### Running Tests
```bash
# Run all tests for progress contract
cargo test --package progress

# Run specific test modules
cargo test --package progress test::test_initialize
cargo test --package progress test::test_course_management
cargo test --package progress test::test_user_progress
cargo test --package progress test::test_progress_validation_rules
```

### Test Coverage
- **Initialization Tests**: Contract setup and admin configuration
- **Course Management Tests**: Adding courses and retrieving course information
- **User Progress Tests**: Progress tracking and completion percentage calculation
- **Validation Tests**: Comprehensive validation rule testing
  - Invalid module number testing
  - Duplicate completion prevention
  - Non-decreasing progress enforcement
- **Edge Case Tests**: Boundary condition handling

## Deployment

### Prerequisites
- Admin address for contract initialization
- Course definitions with module counts

### Deployment Steps
1. Deploy the progress contract
2. Initialize with admin address
3. Add courses with their total module counts
4. Begin progress tracking for students

### Environment Setup
- Set admin address for contract initialization
- Define course structures with module counts
- Configure progress validation rules
- Set up event monitoring for progress updates

## Usage Examples

### Adding a Course
```rust
let course_id = Symbol::short("RUST101");
let total_modules = 10;
client.add_course(&course_id, &total_modules)?;
```

### Updating Student Progress
```rust
let user = Address::generate(&env);
let course_id = Symbol::short("RUST101");
let module = 1u32;
let completed = true;

client.update_progress(&user, &course_id, &module, &completed)?;
```

### Getting Progress Information
```rust
// Get detailed progress (boolean array)
let progress = client.get_progress(&user, &course_id)?;

// Get completion percentage
let percentage = client.get_completion_percentage(&user, &course_id)?;
```

### Progress Validation Examples
```rust
// This will succeed - marking module 1 as complete
client.update_progress(&user, &course_id, &1, &true)?;

// This will fail - trying to complete already completed module
let result = client.update_progress(&user, &course_id, &1, &true);
assert_eq!(result, Err(Ok(Error::ModuleAlreadyCompleted)));

// This will fail - trying to decrease progress
let result = client.update_progress(&user, &course_id, &1, &false);
assert_eq!(result, Err(Ok(Error::NonIncreasingProgress)));

// This will fail - invalid module number
let result = client.update_progress(&user, &course_id, &0, &true);
assert_eq!(result, Err(Ok(Error::InvalidProgress)));
```

## Data Structures

### Progress Storage
- **User Progress Map**: Maps course_id to Vec<bool> representing module completion
- **Course Definitions**: Maps course_id to total module count
- **Module Indexing**: Modules are 1-indexed in API but 0-indexed in storage

### Error Types
- `AlreadyInitialized`: Contract already initialized
- `NotInitialized`: Contract not initialized
- `Unauthorized`: User lacks required permissions
- `CourseNotFound`: Course doesn't exist
- `InvalidProgress`: Module number out of range
- `ModuleAlreadyCompleted`: Trying to complete already completed module
- `NonIncreasingProgress`: Trying to decrease progress

## Related Docs
- [Student Progress Tracker Contract](./student-progress-tracker/README.md)
- [Analytics Contract](./analytics/README.md)
- [Development Guide](../docs/development.md)