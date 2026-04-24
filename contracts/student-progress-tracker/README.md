# Student Progress Tracker Contract

## Overview
A specialized contract for tracking individual student progress through course modules. This contract provides granular progress tracking with percentage-based completion for each module, enabling detailed learning analytics and personalized educational experiences.

## Interface

### Core Functions
```rust
// Initialize the contract with admin
fn initialize(env: Env, admin: Address)

// Update student progress for a specific module
fn update_progress(env: Env, student: Address, course_id: Symbol, module_id: Symbol, percent: u32)

// Get student progress for a course
fn get_progress(env: Env, student: Address, course_id: Symbol) -> Map<Symbol, u32>

// Get admin address
fn get_admin(env: Env) -> Address
```

## Events

### Progress Events
- `progress_updated`: Emitted when student progress is updated
  - Contains: student address, course_id, module_id, completion percentage

## Configuration

### Progress Tracking Configuration
- **Module Progress**: Percentage-based completion (0-100%)
- **Course Identification**: Symbol-based course IDs
- **Student Identification**: Address-based student identification
- **Admin Management**: Single admin address for contract management

### Storage Structure
- **Instance Storage**: Admin address
- **Persistent Storage**: Student progress maps organized by (student, course_id)

## Testing

### Running Tests
```bash
# Run all tests for student-progress-tracker contract
cargo test --package student-progress-tracker

# Run specific test modules
cargo test --package student-progress-tracker test::test_initialization
cargo test --package student-progress-tracker test::test_progress_updates
cargo test --package student-progress-tracker test::test_progress_retrieval
```

### Test Coverage
- **Initialization Tests**: Contract setup and admin configuration
- **Progress Update Tests**: Module progress tracking functionality
- **Progress Retrieval Tests**: Getting student progress data
- **Authorization Tests**: Admin and student permission validation
- **Edge Case Tests**: Boundary condition handling (0%, 100%, invalid percentages)

## Deployment

### Prerequisites
- Admin address for contract initialization
- Course structure definitions

### Deployment Steps
1. Deploy the student-progress-tracker contract
2. Initialize with admin address
3. Begin tracking student progress for courses
4. Set up progress monitoring and analytics

### Environment Setup
- Set admin address for contract initialization
- Define course and module structures
- Configure progress tracking parameters
- Set up event monitoring for progress updates

## Usage Examples

### Initializing the Contract
```rust
let admin = Address::generate(&env);
client.initialize(&admin);
```

### Updating Student Progress
```rust
let student = Address::generate(&env);
let course_id = Symbol::short("BLOCKCHAIN101");
let module_id = Symbol::short("MODULE1");
let progress_percent = 75u32; // 75% complete

client.update_progress(&student, &course_id, &module_id, &progress_percent);
```

### Getting Student Progress
```rust
let progress_map = client.get_progress(&student, &course_id);
// progress_map contains module_id -> percentage mappings
```

### Admin Operations
```rust
let admin_addr = client.get_admin();
// Admin can update progress for any student
client.update_progress(&admin, &course_id, &module_id, &100u32);
```

## Data Structures

### Progress Storage
- **Key**: `(student_address, course_id)`
- **Value**: `Map<Symbol, u32>` (module_id -> completion_percentage)
- **Range**: 0-100% completion per module

### Event Data
```rust
// Progress update event structure
(
    symbol_short!("progress"),
    (
        symbol_short!("updated"),
        student: Address,
        course_id: Symbol,
        module_id: Symbol,
        percent: u32,
    ),
)
```

## Key Features

### Granular Progress Tracking
- **Module-Level Tracking**: Individual progress for each course module
- **Percentage-Based**: Precise completion tracking (0-100%)
- **Real-Time Updates**: Immediate progress updates with event emission

### Flexible Authorization
- **Student Self-Update**: Students can update their own progress
- **Admin Override**: Admin can update progress for any student
- **Authentication Required**: All operations require proper authorization

### Efficient Storage
- **Persistent Storage**: Progress data persists across contract calls
- **Map-Based Organization**: Efficient lookup by student and course
- **Symbol Optimization**: Uses Soroban symbols for efficient storage

## Integration Points

### With Progress Contract
- **Complementary Functionality**: This contract provides detailed module-level tracking
- **Course-Level Aggregation**: Can aggregate module progress to course-level completion
- **Analytics Integration**: Provides data for learning analytics

### With Analytics Contract
- **Detailed Data Source**: Provides granular progress data for analytics
- **Performance Metrics**: Enables detailed performance tracking
- **Learning Path Optimization**: Supports personalized learning recommendations

### With Certificate Contract
- **Completion Verification**: Provides progress data for certificate eligibility
- **Prerequisite Checking**: Supports prerequisite validation based on progress

## Error Handling

### Input Validation
- **Percentage Range**: Automatically validates 0-100% range
- **Panic on Invalid**: Panics if percentage exceeds 100%
- **Symbol Validation**: Ensures valid course and module symbols

### Authorization Checks
- **Student Authentication**: Students must authenticate to update their progress
- **Admin Override**: Admin can update any student's progress
- **Address Validation**: Ensures valid student and admin addresses

## Related Docs
- [Progress Contract](./progress/README.md)
- [Analytics Contract](./analytics/README.md)
- [Development Guide](../docs/development.md)