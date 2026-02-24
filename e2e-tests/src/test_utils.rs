use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export types from the analytics contract for testing - using standard types for host-side
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningSession {
    pub session_id: String, // Hex string
    pub student: String,    // Address string
    pub course_id: String,  // Symbol string
    pub module_id: String,  // Symbol string
    pub start_time: u64,
    pub end_time: u64,
    pub completion_percentage: u32,
    pub time_spent: u64,    // in seconds
    pub interactions: u32,  // number of interactions/activities
    pub score: Option<u32>, // assessment score if applicable
    pub session_type: SessionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SessionType {
    Study,
    Assessment,
    Practice,
    Review,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressAnalytics {
    pub student: String,
    pub course_id: String,
    pub total_modules: u32,
    pub completed_modules: u32,
    pub completion_percentage: u32,
    pub total_time_spent: u64, // in seconds
    pub average_session_time: u64,
    pub total_sessions: u32,
    pub last_activity: u64,
    pub first_activity: u64,
    pub average_score: Option<u32>,
    pub streak_days: u32,
    pub performance_trend: PerformanceTrend,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Declining,
    Insufficient, // Not enough data
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CourseAnalytics {
    pub course_id: String,
    pub total_students: u32,
    pub active_students: u32, // students with activity in last 30 days
    pub completion_rate: u32, // percentage of students who completed the course
    pub average_completion_time: u64, // in seconds
    pub average_score: Option<u32>,
    pub dropout_rate: u32,
    pub most_difficult_module: Option<String>,
    pub easiest_module: Option<String>,
    pub total_time_invested: u64, // sum of all student time
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModuleAnalytics {
    pub course_id: String,
    pub module_id: String,
    pub total_attempts: u32,
    pub completion_rate: u32,
    pub average_time_to_complete: u64,
    pub average_score: Option<u32>,
    pub difficulty_rating: DifficultyRating,
    pub student_feedback_score: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DifficultyRating {
    Easy,     // >80% completion rate, <avg time
    Medium,   // 60-80% completion rate
    Hard,     // 40-60% completion rate
    VeryHard, // <40% completion rate
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressReport {
    pub student: String,
    pub course_id: String,
    pub report_period: ReportPeriod,
    pub start_date: u64,
    pub end_date: u64,
    pub sessions_count: u32,
    pub total_time: u64,
    pub modules_completed: u32,
    pub average_daily_time: u64,
    pub consistency_score: u32, // 0-100 based on regular activity
    pub achievements: Vec<Achievement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportPeriod {
    Daily,
    Weekly,
    Monthly,
    Custom,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Achievement {
    pub achievement_id: String,
    pub title: String,
    pub description: String,
    pub earned_date: u64,
    pub achievement_type: AchievementType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AchievementType {
    Completion,  // Module/course completion
    Streak,      // Consecutive days of activity
    Speed,       // Fast completion
    Excellence,  // High scores
    Consistency, // Regular activity
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMetrics {
    pub course_id: String,
    pub date: u64, // Daily aggregation timestamp
    pub active_students: u32,
    pub total_sessions: u32,
    pub total_time: u64,
    pub completions: u32,
    pub average_score: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LeaderboardEntry {
    pub student: String,
    pub score: u32,
    pub rank: u32,
    pub course_id: String,
    pub metric_type: LeaderboardMetric,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum LeaderboardMetric {
    CompletionSpeed,
    TotalScore,
    ConsistencyScore,
    TimeSpent,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSessionUpdate {
    pub sessions: Vec<LearningSession>,
    pub update_analytics: bool,
    pub update_leaderboards: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsFilter {
    pub course_id: Option<String>,
    pub student: Option<String>,
    pub start_date: Option<u64>,
    pub end_date: Option<u64>,
    pub session_type: OptionalSessionType,
    pub min_score: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptionalSessionType {
    None,
    Some(SessionType),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsConfig {
    pub min_session_time: u64, // Minimum time to count as valid session
    pub max_session_time: u64, // Maximum time for a single session
    pub streak_threshold: u64, // Hours between activities to maintain streak
    pub active_threshold: u64, // Days to consider student active
    pub difficulty_thresholds: DifficultyThresholds,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyThresholds {
    pub easy_completion_rate: u32,   // >80%
    pub medium_completion_rate: u32, // 60-80%
    pub hard_completion_rate: u32,   // 40-60%
                                     // <40% is VeryHard
}

/// Test assertion helpers
pub struct TestAssertions;

impl TestAssertions {
    /// Assert that two learning sessions are approximately equal
    pub fn assert_session_eq(
        session1: &LearningSession,
        session2: &LearningSession,
    ) -> Result<(), String> {
        if session1.session_id != session2.session_id {
            return Err("Session IDs don't match".to_string());
        }
        if session1.student != session2.student {
            return Err("Students don't match".to_string());
        }
        if session1.course_id != session2.course_id {
            return Err("Course IDs don't match".to_string());
        }
        if session1.module_id != session2.module_id {
            return Err("Module IDs don't match".to_string());
        }
        Ok(())
    }

    /// Assert that progress analytics are within reasonable bounds
    pub fn assert_progress_analytics_valid(analytics: &ProgressAnalytics) -> Result<(), String> {
        if analytics.completion_percentage > 100 {
            return Err("Completion percentage exceeds 100".to_string());
        }
        if analytics.total_sessions == 0 && analytics.completion_percentage > 0 {
            return Err("Completion percentage > 0 with no sessions".to_string());
        }
        if analytics.total_sessions > 0 && analytics.average_session_time == 0 {
            return Err("Average session time is 0 with sessions present".to_string());
        }
        if let Some(score) = analytics.average_score {
            if score > 100 {
                return Err("Average score exceeds 100".to_string());
            }
        }
        Ok(())
    }

    /// Assert that course analytics are consistent
    pub fn assert_course_analytics_valid(analytics: &CourseAnalytics) -> Result<(), String> {
        if analytics.total_students == 0 {
            return Err("Total students is 0".to_string());
        }
        if analytics.active_students > analytics.total_students {
            return Err("Active students exceed total students".to_string());
        }
        if analytics.completion_rate > 100 {
            return Err("Completion rate exceeds 100".to_string());
        }
        if analytics.dropout_rate > 100 {
            return Err("Dropout rate exceeds 100".to_string());
        }
        if analytics.completion_rate + analytics.dropout_rate > 100 {
            return Err("Completion rate + dropout rate exceeds 100".to_string());
        }
        Ok(())
    }

    /// Assert that leaderboard entries are properly ordered
    pub fn assert_leaderboard_ordered(leaderboard: &[LeaderboardEntry]) -> Result<(), String> {
        for i in 1..leaderboard.len() {
            let current = &leaderboard[i];
            let previous = &leaderboard[i - 1];

            if current.rank != (i + 1) as u32 {
                return Err(format!(
                    "Rank mismatch at position {}: expected {}, got {}",
                    i,
                    i + 1,
                    current.rank
                ));
            }

            if current.score > previous.score {
                return Err("Leaderboard not in descending order".to_string());
            }
        }
        Ok(())
    }
}

/// Performance measurement utilities
pub struct PerformanceTracker {
    start_time: std::time::Instant,
    checkpoints: HashMap<String, std::time::Instant>,
}

impl Default for PerformanceTracker {
    fn default() -> Self {
        Self::new()
    }
}

impl PerformanceTracker {
    pub fn new() -> Self {
        Self {
            start_time: std::time::Instant::now(),
            checkpoints: HashMap::new(),
        }
    }

    pub fn checkpoint(&mut self, name: &str) {
        self.checkpoints
            .insert(name.to_string(), std::time::Instant::now());
    }

    pub fn elapsed_total(&self) -> std::time::Duration {
        self.start_time.elapsed()
    }

    pub fn elapsed_since(&self, checkpoint: &str) -> Option<std::time::Duration> {
        self.checkpoints.get(checkpoint)?.elapsed().into()
    }

    pub fn print_summary(&self) {
        println!("Performance Summary:");
        println!("Total time: {:?}", self.elapsed_total());

        let mut sorted_checkpoints: Vec<_> = self.checkpoints.iter().collect();
        sorted_checkpoints.sort_by_key(|(_, &time)| time);

        let mut prev_time = self.start_time;
        for (name, &time) in &sorted_checkpoints {
            println!(
                "  {}: {:?} (since start: {:?})",
                name,
                time - prev_time,
                time - self.start_time
            );
            prev_time = time;
        }
    }
}

/// Data validation utilities
pub struct DataValidator;

impl DataValidator {
    /// Validate learning session data integrity
    pub fn validate_session(session: &LearningSession) -> Result<(), String> {
        if session.end_time > 0 && session.end_time <= session.start_time {
            return Err("End time must be after start time".to_string());
        }

        if session.completion_percentage > 100 {
            return Err("Completion percentage cannot exceed 100".to_string());
        }

        if let Some(score) = session.score {
            if score > 100 {
                return Err("Score cannot exceed 100".to_string());
            }
        }

        if session.time_spent > 0 && session.end_time == 0 {
            return Err("Time spent > 0 but end time is 0".to_string());
        }

        if session.end_time > 0 && session.time_spent != session.end_time - session.start_time {
            return Err("Time spent doesn't match start/end times".to_string());
        }

        Ok(())
    }

    /// Validate analytics calculation consistency
    pub fn validate_analytics_consistency(
        sessions: &[LearningSession],
        analytics: &ProgressAnalytics,
    ) -> Result<(), String> {
        let actual_sessions = sessions.len() as u32;
        if analytics.total_sessions != actual_sessions {
            return Err(format!(
                "Session count mismatch: expected {}, got {}",
                actual_sessions, analytics.total_sessions
            ));
        }

        let actual_time: u64 = sessions.iter().map(|s| s.time_spent).sum();
        if analytics.total_time_spent != actual_time {
            return Err(format!(
                "Total time mismatch: expected {}, got {}",
                actual_time, analytics.total_time_spent
            ));
        }

        if actual_sessions > 0 {
            let expected_avg = actual_time / actual_sessions as u64;
            if analytics.average_session_time != expected_avg {
                return Err(format!(
                    "Average session time mismatch: expected {}, got {}",
                    expected_avg, analytics.average_session_time
                ));
            }
        }

        Ok(())
    }
}

/// Mock data generators for specific test scenarios
pub struct MockDataGenerator;

impl MockDataGenerator {
    /// Generate sessions with specific performance characteristics
    pub fn generate_performance_scenario(
        student_address: &str,
        course_id: &str,
        target_completion: u32,
        target_score: u32,
        session_count: usize,
    ) -> Vec<LearningSession> {
        let base_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs()
            - (7 * 24 * 60 * 60);

        let mut sessions = Vec::new();

        for i in 0..session_count {
            let session_start = base_time + (i as u64 * 24 * 60 * 60);
            let session_duration = 3600; // 1 hour

            let bytes = (i as u8).to_be_bytes();
            // Just padding to 32 bytes for hex representation
            let mut id_bytes = [0u8; 32];
            id_bytes[0] = bytes[0];
            let session_id = hex::encode(id_bytes);

            let session = LearningSession {
                session_id,
                student: student_address.to_string(),
                course_id: course_id.to_string(),
                module_id: format!("perf_module_{}", i % 5 + 1),
                start_time: session_start,
                end_time: session_start + session_duration,
                completion_percentage: target_completion,
                time_spent: session_duration,
                interactions: 15,
                score: Some(target_score),
                session_type: SessionType::Study,
            };

            sessions.push(session);
        }

        sessions
    }

    /// Generate edge case sessions for boundary testing
    pub fn generate_edge_cases(
        student_id: &str,
        course_id: &str,
        base_time: u64,
    ) -> Vec<LearningSession> {
        vec![
            // Session with minimum valid duration
            LearningSession {
                session_id: format!("{student_id}_min"),
                student: student_id.to_string(),
                course_id: course_id.to_string(),
                module_id: "intro".to_string(),
                time_spent: 1, // 1 second
                start_time: base_time,
                end_time: base_time + 1,
                completion_percentage: 100,
                interactions: 1,
                score: Some(50), // Minimum passing
                session_type: SessionType::Study,
            },
            // Session with maximum reasonable duration
            LearningSession {
                session_id: format!("{student_id}_max"),
                student: student_id.to_string(),
                course_id: course_id.to_string(),
                module_id: "deep_dive".to_string(),
                time_spent: 4 * 60 * 60, // 4 hours
                start_time: base_time + 3600,
                end_time: base_time + 3600 + (4 * 60 * 60),
                completion_percentage: 100,
                interactions: 100,
                score: Some(100),
                session_type: SessionType::Study,
            },
            // Failed session
            LearningSession {
                session_id: format!("{student_id}_fail"),
                student: student_id.to_string(),
                course_id: course_id.to_string(),
                module_id: "quiz".to_string(),
                time_spent: 1800,
                start_time: base_time + 7200,
                end_time: base_time + 7200 + 1800,
                completion_percentage: 30,
                interactions: 10,
                score: Some(30),
                session_type: SessionType::Practice,
            },
            // Perfect session
            LearningSession {
                session_id: format!("{student_id}_perfect"),
                student: student_id.to_string(),
                course_id: course_id.to_string(),
                module_id: "final".to_string(),
                time_spent: 3600,
                start_time: base_time + 10800,
                end_time: base_time + 10800 + 3600,
                completion_percentage: 100,
                interactions: 50,
                score: Some(100),
                session_type: SessionType::Study,
            },
        ]
    }
}

/// Test environment setup utilities
pub struct TestEnvironment;

impl TestEnvironment {
    /// Create a test environment with multiple students and courses
    pub fn setup_multi_student_environment() -> HashMap<String, Vec<LearningSession>> {
        let mut environment = HashMap::new();

        let students = vec!["alice", "bob", "charlie", "dave", "eve"];
        let courses = vec![
            ("intro_to_rust", 8),
            ("advanced_blockchain", 10),
            ("smart_contracts", 6),
            ("cryptography", 12),
        ];

        for student in &students {
            let mut student_sessions = Vec::new();

            for (course_name, module_count) in &courses {
                let course_sessions = MockDataGenerator::generate_performance_scenario(
                    student,
                    course_name,
                    75 + (students.iter().position(|&s| s == *student).unwrap() as u32 * 5),
                    80 + (students.iter().position(|&s| s == *student).unwrap() as u32 * 3),
                    *module_count,
                );
                student_sessions.extend(course_sessions);
            }

            environment.insert(student.to_string(), student_sessions);
        }

        environment
    }

    /// Calculate expected analytics for validation
    pub fn calculate_expected_analytics(sessions: &[LearningSession]) -> ProgressAnalytics {
        let total_sessions = sessions.len() as u32;
        let total_time_spent: u64 = sessions.iter().map(|s| s.time_spent).sum();
        let average_session_time = if total_sessions > 0 {
            total_time_spent / total_sessions as u64
        } else {
            0
        };

        let scores: Vec<u32> = sessions.iter().filter_map(|s| s.score).collect();
        let average_score = if !scores.is_empty() {
            Some(scores.iter().sum::<u32>() / scores.len() as u32)
        } else {
            None
        };

        let completed_modules = sessions
            .iter()
            .filter(|s| s.completion_percentage == 100)
            .map(|s| &s.module_id)
            .collect::<std::collections::HashSet<_>>()
            .len() as u32;

        let completion_percentage = if !sessions.is_empty()
            && sessions
                .iter()
                .any(|s| s.course_id == sessions[0].course_id)
        {
            (completed_modules * 100) / 8 // Assume 8 modules total
        } else {
            0
        };

        let last_activity = sessions.iter().map(|s| s.end_time).max().unwrap_or(0);
        let first_activity = sessions.iter().map(|s| s.start_time).min().unwrap_or(0);
        let student = if sessions.is_empty() {
            "".to_string()
        } else {
            sessions[0].student.clone()
        };
        let course_id = if sessions.is_empty() {
            "".to_string()
        } else {
            sessions[0].course_id.clone()
        };

        ProgressAnalytics {
            student,
            course_id,
            total_modules: 8,
            completed_modules,
            completion_percentage,
            total_time_spent,
            average_session_time,
            total_sessions,
            last_activity,
            first_activity,
            average_score,
            streak_days: 0, // Would need more complex calculation
            performance_trend: PerformanceTrend::Stable,
        }
    }
}
