# Course Progress Analytics Contract

A comprehensive analytics system for tracking detailed learning progress, completion rates, and performance metrics in educational platforms.

## Overview

The Analytics contract provides advanced learning analytics capabilities including:

- **Detailed Progress Tracking**: Session-level learning data with time tracking
- **Performance Metrics**: Comprehensive calculation of student and course performance
- **Completion Rate Analytics**: Course and module completion analysis
- **Time-based Reports**: Daily, weekly, and monthly progress reports
- **Achievement System**: Automated achievement detection and awarding
- **Leaderboards**: Performance-based ranking systems
- **Gas-optimized Storage**: Efficient data storage and retrieval

## Features

### Core Analytics
- Learning session recording and completion tracking
- Real-time progress analytics calculation
- Course-wide performance metrics
- Module difficulty analysis
- Student performance trends

### Reporting System
- Custom time-period reports
- Daily aggregated metrics
- Weekly and monthly summaries
- Completion trend analysis
- Performance comparisons

### Achievement System
- Automatic achievement detection
- Multiple achievement types (Completion, Streak, Excellence, etc.)
- Achievement history tracking
- Milestone notifications

### Administrative Features
- Configurable analytics parameters
- Bulk data operations
- Data cleanup utilities
- Performance optimization tools

## Contract Interface

### Core Functions

```rust
// Initialize the contract
fn initialize(env: Env, admin: Address, config: AnalyticsConfig) -> Result<(), AnalyticsError>

// Record a learning session
fn record_session(env: Env, session: LearningSession) -> Result<(), AnalyticsError>

// Complete a session with final metrics
fn complete_session(
    env: Env,
    session_id: BytesN<32>,
    end_time: u64,
    final_score: Option<u32>,
    completion_percentage: u32,
) -> Result<(), AnalyticsError>

// Get progress analytics for a student
fn get_progress_analytics(
    env: Env,
    student: Address,
    course_id: Symbol,
) -> Result<ProgressAnalytics, AnalyticsError>
```

### Analytics Functions

```rust
// Get course-wide analytics
fn get_course_analytics(env: Env, course_id: Symbol) -> Result<CourseAnalytics, AnalyticsError>

// Get module-specific analytics
fn get_module_analytics(
    env: Env,
    course_id: Symbol,
    module_id: Symbol,
) -> Result<ModuleAnalytics, AnalyticsError>

// Generate leaderboard
fn generate_leaderboard(
    env: Env,
    course_id: Symbol,
    metric: LeaderboardMetric,
    limit: u32,
) -> Result<Vec<LeaderboardEntry>, AnalyticsError>
```

### Reporting Functions

```rust
// Generate progress report
fn generate_progress_report(
    env: Env,
    student: Address,
    course_id: Symbol,
    period: ReportPeriod,
    start_date: u64,
    end_date: u64,
) -> Result<ProgressReport, AnalyticsError>

// Generate daily metrics
fn generate_daily_metrics(
    env: Env,
    course_id: Symbol,
    date: u64,
) -> Result<AggregatedMetrics, AnalyticsError>
```

## Data Types

### Learning Session
```rust
pub struct LearningSession {
    pub session_id: BytesN<32>,
    pub student: Address,
    pub course_id: Symbol,
    pub module_id: Symbol,
    pub start_time: u64,
    pub end_time: u64,
    pub completion_percentage: u32,
    pub time_spent: u64,
    pub interactions: u32,
    pub score: Option<u32>,
    pub session_type: SessionType,
}
```

### Progress Analytics
```rust
pub struct ProgressAnalytics {
    pub student: Address,
    pub course_id: Symbol,
    pub total_modules: u32,
    pub completed_modules: u32,
    pub completion_percentage: u32,
    pub total_time_spent: u64,
    pub average_session_time: u64,
    pub total_sessions: u32,
    pub last_activity: u64,
    pub first_activity: u64,
    pub average_score: Option<u32>,
    pub streak_days: u32,
    pub performance_trend: PerformanceTrend,
}
```

### Course Analytics
```rust
pub struct CourseAnalytics {
    pub course_id: Symbol,
    pub total_students: u32,
    pub active_students: u32,
    pub completion_rate: u32,
    pub average_completion_time: u64,
    pub average_score: Option<u32>,
    pub dropout_rate: u32,
    pub most_difficult_module: Option<Symbol>,
    pub easiest_module: Option<Symbol>,
    pub total_time_invested: u64,
}
```

## Usage Examples

### Recording a Learning Session

```rust
let session = LearningSession {
    session_id: BytesN::from_array(&env, &[1u8; 32]),
    student: student_address,
    course_id: Symbol::new(&env, "RUST101"),
    module_id: Symbol::new(&env, "module_1"),
    start_time: env.ledger().timestamp(),
    end_time: 0,
    completion_percentage: 0,
    time_spent: 0,
    interactions: 0,
    score: None,
    session_type: SessionType::Study,
};

client.record_session(&session)?;
```

### Completing a Session

```rust
let end_time = env.ledger().timestamp() + 1800; // 30 minutes later
let final_score = Some(85u32);
let completion_percentage = 100u32;

client.complete_session(
    &session_id,
    &end_time,
    &final_score,
    &completion_percentage,
)?;
```

### Getting Analytics

```rust
// Student progress analytics
let progress = client.get_progress_analytics(&student, &course_id)?;

// Course analytics
let course_stats = client.get_course_analytics(&course_id)?;

// Generate leaderboard
let leaderboard = client.generate_leaderboard(
    &course_id,
    &LeaderboardMetric::TotalScore,
    &10,
)?;
```

### Generating Reports

```rust
// Weekly progress report
let report = client.generate_progress_report(
    &student,
    &course_id,
    &ReportPeriod::Weekly,
    &start_date,
    &end_date,
)?;

// Daily metrics
let metrics = client.generate_daily_metrics(&course_id, &date)?;
```

## Configuration

### Analytics Configuration
```rust
pub struct AnalyticsConfig {
    pub min_session_time: u64,        // Minimum valid session duration
    pub max_session_time: u64,        // Maximum valid session duration
    pub streak_threshold: u64,        // Time threshold for maintaining streaks
    pub active_threshold: u64,        // Time threshold for active students
    pub difficulty_thresholds: DifficultyThresholds,
}
```

### Difficulty Thresholds
```rust
pub struct DifficultyThresholds {
    pub easy_completion_rate: u32,    // >80% completion rate
    pub medium_completion_rate: u32,  // 60-80% completion rate
    pub hard_completion_rate: u32,    // 40-60% completion rate
    // <40% is classified as VeryHard
}
```

## Performance Optimization

### Gas Optimization Features
- Batch session processing (up to 50 sessions per transaction)
- Aggregated daily metrics to reduce query complexity
- Efficient storage patterns with persistent and instance storage
- Configurable cleanup for old data

### Storage Strategy
- **Persistent Storage**: Session data, analytics, reports
- **Instance Storage**: Configuration, admin settings
- **Aggregated Data**: Daily metrics for efficient querying
- **Indexed Access**: Student-course mappings for fast retrieval

## Events

The contract emits comprehensive events for all major operations:

- `session_recorded`: New learning session started
- `session_completed`: Learning session finished
- `progress_updated`: Student progress analytics updated
- `achievement_earned`: Student earned new achievement
- `leaderboard_updated`: Course leaderboard recalculated
- `report_generated`: Progress report created
- `batch_processed`: Batch operation completed

## Security Features

- **Authorization**: All operations require proper authentication
- **Input Validation**: Comprehensive validation of all input data
- **Admin Controls**: Protected administrative functions
- **Data Integrity**: Validation of session durations, scores, and percentages

## Testing

The contract includes comprehensive test suites:

- **Unit Tests**: Individual function testing
- **Integration Tests**: Complete workflow testing
- **Performance Tests**: Gas optimization validation
- **Edge Case Tests**: Boundary condition handling

## Frontend Integration

### Key Integration Points

1. **Session Tracking**: Record learning sessions in real-time
2. **Progress Display**: Show student progress analytics
3. **Leaderboards**: Display course rankings
4. **Reports**: Generate and display progress reports
5. **Achievements**: Show earned achievements and milestones

### API Patterns

```javascript
// Record session start
await contract.record_session(sessionData);

// Update session progress
await contract.complete_session(sessionId, endTime, score, completion);

// Get analytics
const progress = await contract.get_progress_analytics(student, courseId);
const courseStats = await contract.get_course_analytics(courseId);

// Generate reports
const report = await contract.generate_progress_report(
    student, courseId, period, startDate, endDate
);
```

## Deployment

1. Deploy the contract with initial configuration
2. Set up admin permissions
3. Configure analytics parameters
4. Initialize course data
5. Begin session recording

## Future Enhancements

- Machine learning integration for predictive analytics
- Advanced visualization data preparation
- Real-time notifications and alerts
- Integration with external learning management systems
- Advanced reporting with custom metrics
