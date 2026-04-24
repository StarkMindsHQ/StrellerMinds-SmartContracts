# Analytics Integration Test Suite

This directory contains a comprehensive integration test suite for the analytics contract that validates real-world usage scenarios and ensures data consistency across contract operations.

## Overview

The integration test suite covers all major functionality of the analytics contract:

- **Learning Session Tracking**: End-to-end workflow for recording and completing learning sessions
- **Progress Analytics Calculations**: Validation of student and course-wide analytics
- **Leaderboard Generation**: Testing ranking systems and competitive metrics
- **Performance Metrics Aggregation**: Time-based analytics and reporting
- **Data Consistency Validation**: Ensuring data integrity across different views
- **Edge Cases and Error Conditions**: Boundary testing and error handling
- **CI/CD Pipeline Integration**: Automated testing and deployment validation

## Test Structure

### Main Test Files

- `analytics_integration.rs` - Primary integration test suite with all test scenarios
- `test_data.rs` - Realistic test data generators for various learning scenarios
- `test_utils.rs` - Helper utilities, assertions, and validation functions

### Test Categories

#### 1. Learning Session Tracking (`test_learning_session_tracking_e2e`)
- Records individual learning sessions with realistic data
- Validates session storage and retrieval
- Tests session completion workflows
- Validates batch session updates
- Ensures student session history accuracy

#### 2. Progress Analytics Calculations (`test_progress_analytics_calculations`)
- Creates diverse learning scenarios (excellent, good, average, poor students)
- Validates progress analytics calculations
- Tests course-wide analytics aggregation
- Verifies performance trend calculations
- Ensures metric accuracy and consistency

#### 3. Leaderboard Generation (`test_leaderboard_generation`)
- Creates competitive learning scenarios
- Tests different leaderboard metrics (TotalScore, CompletionSpeed, TimeSpent, ConsistencyScore)
- Validates ranking order and consistency
- Tests top performers retrieval
- Validates struggling student identification

#### 4. Performance Metrics Aggregation (`test_performance_metrics_aggregation`)
- Generates time-based learning activity over a week
- Tests daily metrics aggregation
- Validates weekly and monthly summaries
- Tests completion trends analysis
- Validates individual progress reports

#### 5. Data Consistency Validation (`test_data_consistency_validation`)
- Ensures session data matches analytics calculations
- Validates course analytics aggregate student data correctly
- Tests filtered session queries
- Verifies leaderboard consistency
- Tests data integrity after recalculation

#### 6. Edge Cases and Error Conditions (`test_edge_cases_and_error_conditions`)
- Tests duplicate session recording prevention
- Validates invalid session data rejection
- Tests non-existent data handling
- Validates batch operations with mixed valid/invalid data
- Tests unauthorized operation prevention
- Validates edge case analytics calculations

#### 7. CI/CD Pipeline Integration (`test_cicd_pipeline_integration`)
- Validates contract deployment success
- Tests basic functionality smoke tests
- Ensures all major functions are callable
- Validates health check functionality

## Test Data Generation

### Realistic Learning Scenarios

The test suite generates realistic learning data including:

- **Excellent Students**: High completion rates, consistent study patterns, 2-3 sessions per day
- **Good Students**: Steady progress, occasional breaks, 1-2 sessions per day  
- **Average Students**: Inconsistent progress, irregular study patterns
- **Struggling Students**: Low completion rates, long gaps between sessions

### Time-Based Patterns

- Sessions distributed over realistic time periods (weeks/months)
- Natural breaks and study patterns
- Varying session durations (30 minutes to 4 hours)
- Different session types (Study, Practice, Assessment, Review)

### Performance Variations

- Score ranges from 55-98 depending on student performance level
- Completion percentages reflecting actual learning progress
- Interaction counts varying by session type and engagement
- Realistic time spent per session

## Running the Tests

### Prerequisites

1. **Soroban CLI**: Install the latest Soroban CLI
2. **Rust Toolchain**: Stable Rust with required components
3. **Local Network**: Soroban standalone network running

### Quick Start

```bash
# Run the complete test suite
./scripts/run_analytics_tests.sh

# Run specific test categories
./scripts/run_analytics_tests.sh unit      # Unit tests only
./scripts/run_analytics_tests.sh e2e       # E2E tests only
./scripts/run_analytics_tests.sh performance # Performance tests only
./scripts/run_analytics_tests.sh security   # Security checks only
```

### Manual Test Execution

```bash
# Build contracts
make build

# Run specific integration test
cd e2e-tests
cargo test --test analytics_integration test_learning_session_tracking_e2e

# Run all integration tests
cargo test --test analytics_integration -- --nocapture
```

## CI/CD Integration

### GitHub Actions Workflow

The test suite is integrated with GitHub Actions through `.github/workflows/analytics-integration-tests.yml`:

- **Triggers**: Push to main/develop branches, pull requests
- **Test Matrix**: Unit tests, integration tests, security checks, documentation tests
- **Coverage**: Automatic coverage reporting to Codecov
- **Performance**: Benchmark testing and validation
- **Security**: Dependency audit and security checks

### Pipeline Stages

1. **Setup**: Install Rust, Soroban CLI, cache dependencies
2. **Build**: Compile all contracts and test dependencies
3. **Unit Tests**: Contract-level unit and integration tests
4. **E2E Tests**: Full integration test suite
5. **Linting**: Code formatting and clippy checks
6. **Coverage**: Generate and upload coverage reports
7. **Security**: Dependency audit and security checks
8. **Documentation**: Test documentation examples and generate docs

## Test Utilities

### Assertion Helpers

```rust
use test_utils::TestAssertions;

// Validate session data
TestAssertions::assert_session_eq(&session1, &session2)?;

// Validate analytics calculations
TestAssertions::assert_progress_analytics_valid(&analytics)?;

// Validate course analytics
TestAssertions::assert_course_analytics_valid(&course_analytics)?;

// Validate leaderboard ordering
TestAssertions::assert_leaderboard_ordered(&leaderboard)?;
```

### Performance Tracking

```rust
use test_utils::PerformanceTracker;

let mut tracker = PerformanceTracker::new();
tracker.checkpoint("session_recording");
// ... perform operations
tracker.print_summary();
```

### Data Validation

```rust
use test_utils::DataValidator;

// Validate session integrity
DataValidator::validate_session(&session)?;

// Validate analytics consistency
DataValidator::validate_analytics_consistency(&sessions, &analytics)?;
```

## Mock Data Generation

### Performance Scenarios

```rust
use test_data::MockDataGenerator;

// Generate specific performance scenario
let sessions = MockDataGenerator::generate_performance_scenario(
    "student_address",
    course_id,
    target_completion,
    target_score,
    session_count,
);
```

### Boundary Testing

```rust
// Generate edge case sessions
let boundary_sessions = MockDataGenerator::generate_boundary_sessions("student_address");
```

## Configuration

### Test Configuration

The test suite uses a standardized analytics configuration:

```rust
AnalyticsConfig {
    min_session_time: 300,      // 5 minutes
    max_session_time: 14400,    // 4 hours
    streak_threshold: 48,       // 48 hours
    active_threshold: 30,       // 30 days
    difficulty_thresholds: DifficultyThresholds {
        easy_completion_rate: 80,
        medium_completion_rate: 60,
        hard_completion_rate: 40,
    },
}
```

### Environment Setup

The test suite automatically:

- Starts a local Soroban network if not running
- Deploys all required contracts
- Initializes contracts with test configuration
- Creates test accounts and funding
- Cleans up resources after tests complete

## Reports and Output

### Test Reports

After running tests, comprehensive reports are generated in `reports/analytics/`:

- **Coverage Report**: `tarpaulin-report.html` - Code coverage visualization
- **Documentation**: `docs/index.html` - Generated API documentation
- **Test Logs**: Detailed test execution logs with performance metrics

### Performance Metrics

The test suite tracks and reports:

- Contract deployment times
- Session recording performance
- Analytics calculation speed
- Batch operation efficiency
- Memory usage patterns

## Troubleshooting

### Common Issues

1. **Soroban Network Not Running**
   ```bash
   soroban network standalone &
   soroban config network standalone --global --rpc-url http://localhost:8000
   ```

2. **Contract Build Failures**
   ```bash
   cd contracts/analytics
   cargo clean
   cargo build --release --target wasm32-unknown-unknown
   ```

3. **Test Account Issues**
   ```bash
   # Reset local network
   pkill -f soroban
   soroban network standalone &
   ```

### Debug Mode

Run tests with additional logging:

```bash
RUST_LOG=debug cargo test --test analytics_integration -- --nocapture
```

### Performance Analysis

For detailed performance analysis:

```bash
# Run with performance tracking
./scripts/run_analytics_tests.sh performance

# Generate flame graph (requires cargo-flamegraph)
cargo install flamegraph
cargo flamegraph --test analytics_integration
```

## Contributing

When adding new tests:

1. Follow the existing test structure and naming conventions
2. Use the provided test utilities and assertion helpers
3. Generate realistic test data using the mock data generators
4. Include both positive and negative test cases
5. Add performance tracking for expensive operations
6. Update this documentation with new test descriptions

## Best Practices

- **Test Isolation**: Each test should be independent and not rely on other tests
- **Data Cleanup**: Tests should clean up after themselves or use isolated data
- **Realistic Data**: Use the provided generators for realistic test scenarios
- **Error Testing**: Include both success and failure scenarios
- **Performance**: Track performance for expensive operations
- **Documentation**: Document complex test scenarios and expected outcomes
