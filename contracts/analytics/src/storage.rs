use crate::types::{
    Achievement, AggregatedMetrics, AnalyticsConfig, CourseAnalytics, DataKey, LeaderboardEntry,
    LearningSession, ModuleAnalytics, ProgressAnalytics, ProgressReport,
};
use soroban_sdk::{Address, BytesN, Env, Symbol, Vec};

/// Storage utilities for analytics contract
pub struct AnalyticsStorage;

impl AnalyticsStorage {
    /// Store a learning session
    pub fn set_session(env: &Env, session: &LearningSession) {
        let key = DataKey::Session(session.session_id.clone());
        env.storage().persistent().set(&key, session);

        // Also add to student's session list
        Self::add_student_session(
            env,
            &session.student,
            &session.course_id,
            &session.session_id,
        );
    }

    /// Get a learning session by ID
    pub fn get_session(env: &Env, session_id: &BytesN<32>) -> Option<LearningSession> {
        let key = DataKey::Session(session_id.clone());
        env.storage().persistent().get(&key)
    }

    /// Add session to student's session list
    pub fn add_student_session(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        session_id: &BytesN<32>,
    ) {
        let key = DataKey::StudentSessions(student.clone(), course_id.clone());
        let mut sessions: Vec<BytesN<32>> = env
            .storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env));

        sessions.push_back(session_id.clone());
        env.storage().persistent().set(&key, &sessions);
    }

    /// Get all sessions for a student in a course
    pub fn get_student_sessions(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
    ) -> Vec<BytesN<32>> {
        let key = DataKey::StudentSessions(student.clone(), course_id.clone());
        env.storage()
            .persistent()
            .get(&key)
            .unwrap_or(Vec::new(env))
    }

    /// Store progress analytics
    pub fn set_progress_analytics(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        analytics: &ProgressAnalytics,
    ) {
        let key = DataKey::ProgressAnalytics(student.clone(), course_id.clone());
        env.storage().persistent().set(&key, analytics);
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
    pub fn has_progress_analytics(env: &Env, student: &Address, course_id: &Symbol) -> bool {
        let key = DataKey::ProgressAnalytics(student.clone(), course_id.clone());
        env.storage().persistent().has(&key)
    }

    /// Remove old sessions (for cleanup)
    pub fn remove_session(env: &Env, session_id: &BytesN<32>) {
        let key = DataKey::Session(session_id.clone());
        env.storage().persistent().remove(&key);
    }

    /// Get default analytics configuration
    pub fn get_default_config(env: &Env) -> AnalyticsConfig {
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
        }
    }
}
