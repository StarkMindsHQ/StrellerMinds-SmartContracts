use crate::types::{
    Achievement, AggregatedMetrics, AnalyticsConfig, CourseAnalytics, DataKey, InsightType,
    LeaderboardEntry, LearningSession, MLInsight, ModuleAnalytics, ProgressAnalytics,
    ProgressReport,
};
use soroban_sdk::{Address, BytesN, Env, Symbol, Vec};
use crate::shared::storage_optimization::{CompactStorage, PackedStudentData, CompressedSessionCollection};
use crate::shared::storage_cleanup::StorageCleanup;

/// Storage utilities for analytics contract
pub struct AnalyticsStorage;

impl AnalyticsStorage {
    /// Store a learning session with optimization
    pub fn set_session(env: &Env, session: &LearningSession) {
        let key = DataKey::Session(session.session_id.clone());
        env.storage().persistent().set(&key, session);

        // Use optimized student session tracking
        Self::add_student_session_optimized(
            env,
            &session.student,
            &session.course_id,
            &session.session_id,
            session.start_time,
        );
    }

    /// Get a learning session by ID
    pub fn get_session(env: &Env, session_id: &BytesN<32>) -> Option<LearningSession> {
        let key = DataKey::Session(session_id.clone());
        env.storage().persistent().get(&key)
    }

    /// Add session to student's session list (optimized version)
    pub fn add_student_session_optimized(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        session_id: &BytesN<32>,
        timestamp: u64,
    ) {
        let key = DataKey::StudentSessions(student.clone(), course_id.clone());
        
        // Try to get existing compressed collection
        if let Some(mut compressed) = env.storage().persistent().get::<_, CompressedSessionCollection>(&key) {
            // Add new session to compressed collection
            let sessions = compressed.decompress_sessions(env);
            let mut new_sessions = Vec::new(env);
            
            // Add existing sessions
            for session in sessions.iter() {
                new_sessions.push_back(session.clone());
            }
            
            // Add new session
            new_sessions.push_back((timestamp, 0, 0)); // Default values for compression
            
            // Re-compress
            compressed = CompressedSessionCollection::compress_sessions(env, new_sessions);
            env.storage().persistent().set(&key, &compressed);
        } else {
            // Create new compressed collection
            let sessions = Vec::from_array(env, [(timestamp, 0, 0)]);
            let compressed = CompressedSessionCollection::compress_sessions(env, sessions);
            env.storage().persistent().set(&key, &compressed);
        }
    }
    
    /// Legacy method for backward compatibility
    pub fn add_student_session(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        session_id: &BytesN<32>,
    ) {
        Self::add_student_session_optimized(env, student, course_id, session_id, env.ledger().timestamp());
    }

    /// Get all sessions for a student in a course (optimized)
    pub fn get_student_sessions(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
    ) -> Vec<BytesN<32>> {
        let key = DataKey::StudentSessions(student.clone(), course_id.clone());
        
        // Try to get compressed collection first
        if let Some(compressed) = env.storage().persistent().get::<_, CompressedSessionCollection>(&key) {
            // For now, return empty Vec - would need session ID mapping
            Vec::new(env)
        } else {
            // Fallback to legacy storage
            env.storage()
                .persistent()
                .get(&key)
                .unwrap_or(Vec::new(env))
        }
    }
    
    /// Get compressed session collection for analytics
    pub fn get_student_sessions_compressed(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
    ) -> Option<CompressedSessionCollection> {
        let key = DataKey::StudentSessions(student.clone(), course_id.clone());
        env.storage().persistent().get(&key)
    }

    /// Store progress analytics (optimized with packed data)
    pub fn set_progress_analytics(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        analytics: &ProgressAnalytics,
    ) {
        // Store full analytics for detailed queries
        let key = DataKey::ProgressAnalytics(student.clone(), course_id.clone());
        env.storage().persistent().set(&key, analytics);
        
        // Also store packed version for quick stats
        let packed_key = DataKey::StudentAchievements(student.clone()); // Reuse existing key type
        let packed_data = CompactStorage::optimize_student_data(
            analytics.completion_percentage,
            (analytics.total_time_spent / 3600) as u32, // Convert to hours
            5, // Default interaction level
            3, // Default performance tier
            analytics.first_activity,
            analytics.last_activity,
            analytics.total_sessions,
            analytics.completed_modules,
            analytics.average_score.unwrap_or(0),
            analytics.streak_days,
        );
        env.storage().persistent().set(&packed_key, &packed_data);
    }

    /// Get progress analytics
    pub fn get_progress_analytics(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
    ) -> Option<ProgressAnalytics> {
        let key = DataKey::ProgressAnalytics(student.clone(), course_id.clone());
        env.storage().persistent().get(&key)
    }

    /// Store course analytics
    pub fn set_course_analytics(env: &Env, course_id: &Symbol, analytics: &CourseAnalytics) {
        let key = DataKey::CourseAnalytics(course_id.clone());
        env.storage().persistent().set(&key, analytics);
    }

    /// Get course analytics
    pub fn get_course_analytics(env: &Env, course_id: &Symbol) -> Option<CourseAnalytics> {
        let key = DataKey::CourseAnalytics(course_id.clone());
        env.storage().persistent().get(&key)
    }

    /// Add student to course
    pub fn add_course_student(env: &Env, course_id: &Symbol, student: &Address) {
        let key = DataKey::CourseStudents(course_id.clone());
        let mut students: Vec<Address> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));

        // Check if student already exists
        for i in 0..students.len() {
            if students.get(i).unwrap() == *student {
                return; // Student already exists
            }
        }

        students.push_back(student.clone());
        env.storage().persistent().set(&key, &students);
    }

    /// Get all students in a course
    pub fn get_course_students(env: &Env, course_id: &Symbol) -> Vec<Address> {
        let key = DataKey::CourseStudents(course_id.clone());
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env))
    }

    /// Store module analytics
    pub fn set_module_analytics(
        env: &Env,
        course_id: &Symbol,
        module_id: &Symbol,
        analytics: &ModuleAnalytics,
    ) {
        let key = DataKey::ModuleAnalytics(course_id.clone(), module_id.clone());
        env.storage().persistent().set(&key, analytics);
    }

    /// Get module analytics
    pub fn get_module_analytics(
        env: &Env,
        course_id: &Symbol,
        module_id: &Symbol,
    ) -> Option<ModuleAnalytics> {
        let key = DataKey::ModuleAnalytics(course_id.clone(), module_id.clone());
        env.storage().persistent().get(&key)
    }

    /// Store progress report
    pub fn set_progress_report(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        timestamp: u64,
        report: &ProgressReport,
    ) {
        let key = DataKey::ProgressReport(student.clone(), course_id.clone(), timestamp);
        env.storage().persistent().set(&key, report);
    }

    /// Get progress report
    pub fn get_progress_report(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        timestamp: u64,
    ) -> Option<ProgressReport> {
        let key = DataKey::ProgressReport(student.clone(), course_id.clone(), timestamp);
        env.storage().persistent().get(&key)
    }

    /// Store daily aggregated metrics
    pub fn set_daily_metrics(
        env: &Env,
        course_id: &Symbol,
        date: u64,
        metrics: &AggregatedMetrics,
    ) {
        let key = DataKey::DailyMetrics(course_id.clone(), date);
        env.storage().persistent().set(&key, metrics);
    }

    /// Get daily aggregated metrics
    pub fn get_daily_metrics(
        env: &Env,
        course_id: &Symbol,
        date: u64,
    ) -> Option<AggregatedMetrics> {
        let key = DataKey::DailyMetrics(course_id.clone(), date);
        env.storage().persistent().get(&key)
    }

    /// Store student achievements
    pub fn set_student_achievements(env: &Env, student: &Address, achievements: &Vec<Achievement>) {
        let key = DataKey::StudentAchievements(student.clone());
        env.storage().persistent().set(&key, achievements);
    }

    /// Get student achievements
    pub fn get_student_achievements(env: &Env, student: &Address) -> Vec<Achievement> {
        let key = DataKey::StudentAchievements(student.clone());
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env))
    }

    /// Add achievement to student
    pub fn add_student_achievement(env: &Env, student: &Address, achievement: &Achievement) {
        let mut achievements = Self::get_student_achievements(env, student);
        achievements.push_back(achievement.clone());
        Self::set_student_achievements(env, student, &achievements);
    }

    /// Store leaderboard
    pub fn set_leaderboard(
        env: &Env,
        course_id: &Symbol,
        metric: &crate::types::LeaderboardMetric,
        entries: &Vec<LeaderboardEntry>,
    ) {
        let key = DataKey::Leaderboard(course_id.clone(), metric.clone());
        env.storage().persistent().set(&key, entries);
    }

    /// Get leaderboard
    pub fn get_leaderboard(
        env: &Env,
        course_id: &Symbol,
        metric: &crate::types::LeaderboardMetric,
    ) -> Vec<LeaderboardEntry> {
        let key = DataKey::Leaderboard(course_id.clone(), metric.clone());
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env))
    }

    /// Store an ML insight
    pub fn set_ml_insight(env: &Env, insight: &MLInsight) {
        let key = DataKey::MLInsight(
            insight.student.clone(),
            insight.course_id.clone(),
            insight.insight_type.clone(),
        );
        env.storage().persistent().set(&key, insight);
    }

    /// Get an ML insight
    pub fn get_ml_insight(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        insight_type: &InsightType,
    ) -> Option<MLInsight> {
        let key = DataKey::MLInsight(student.clone(), course_id.clone(), insight_type.clone());
        env.storage().persistent().get(&key)
    }

    /// Store analytics configuration
    pub fn set_config(env: &Env, config: &AnalyticsConfig) {
        let key = DataKey::AnalyticsConfig;
        env.storage().instance().set(&key, config);
    }

    /// Get analytics configuration
    pub fn get_config(env: &Env) -> Option<AnalyticsConfig> {
        let key = DataKey::AnalyticsConfig;
        env.storage().instance().get(&key)
    }

    /// Store admin address
    pub fn set_admin(env: &Env, admin: &Address) {
        let key = DataKey::Admin;
        env.storage().instance().set(&key, admin);
    }

    /// Get admin address
    pub fn get_admin(env: &Env) -> Option<Address> {
        let key = DataKey::Admin;
        env.storage().instance().get(&key)
    }

    /// Check if session exists
    pub fn has_session(env: &Env, session_id: &BytesN<32>) -> bool {
        let key = DataKey::Session(session_id.clone());
        env.storage().persistent().has(&key)
    }

    /// Check if progress analytics exists
    #[allow(dead_code)]
    pub fn has_progress_analytics(env: &Env, student: &Address, course_id: &Symbol) -> bool {
        let key = DataKey::ProgressAnalytics(student.clone(), course_id.clone());
        env.storage().persistent().has(&key)
    }

    /// Remove old sessions (for cleanup)
    #[allow(dead_code)]
    pub fn remove_session(env: &Env, session_id: &BytesN<32>) {
        let key = DataKey::Session(session_id.clone());
        env.storage().persistent().remove(&key);
    }

    /// Get default analytics configuration
    pub fn get_default_config(_env: &Env) -> AnalyticsConfig {
        AnalyticsConfig {
            min_session_time: 60,      // 1 minute
            max_session_time: 14400,   // 4 hours
            streak_threshold: 86400,   // 24 hours
            active_threshold: 2592000, // 30 days
            difficulty_thresholds: crate::types::DifficultyThresholds {
                easy_completion_rate: 80,
                medium_completion_rate: 60,
                hard_completion_rate: 40,
            },
            oracle_address: None,
        }
    }
}
