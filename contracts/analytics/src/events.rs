use soroban_sdk::{Address, BytesN, Env, Symbol};
use crate::types::{SessionType, AchievementType, LeaderboardMetric, PerformanceTrend};

/// Analytics contract events for tracking and auditing
pub struct AnalyticsEvents;

impl AnalyticsEvents {
    /// Emit event when a new learning session is recorded
    pub fn emit_session_recorded(
        env: &Env,
        session_id: &BytesN<32>,
        student: &Address,
        course_id: &Symbol,
        module_id: &Symbol,
        session_type: SessionType,
        time_spent: u64,
        completion_percentage: u32,
    ) {
        env.events().publish(
            ("analytics", "session_recorded"),
            (session_id, student, course_id, module_id, session_type, time_spent, completion_percentage),
        );
    }

    /// Emit event when a learning session is completed
    pub fn emit_session_completed(
        env: &Env,
        session_id: &BytesN<32>,
        student: &Address,
        course_id: &Symbol,
        module_id: &Symbol,
        final_score: Option<u32>,
        total_time: u64,
    ) {
        env.events().publish(
            ("analytics", "session_completed"),
            (session_id, student, course_id, module_id, final_score, total_time),
        );
    }

    /// Emit event when progress analytics are updated
    pub fn emit_progress_updated(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        completion_percentage: u32,
        total_time_spent: u64,
        performance_trend: PerformanceTrend,
    ) {
        env.events().publish(
            ("analytics", "progress_updated"),
            (student, course_id, completion_percentage, total_time_spent, performance_trend),
        );
    }

    /// Emit event when course analytics are recalculated
    pub fn emit_course_analytics_updated(
        env: &Env,
        course_id: &Symbol,
        total_students: u32,
        completion_rate: u32,
        average_score: Option<u32>,
    ) {
        env.events().publish(
            ("analytics", "course_analytics_updated"),
            (course_id, total_students, completion_rate, average_score),
        );
    }

    /// Emit event when module analytics are updated
    pub fn emit_module_analytics_updated(
        env: &Env,
        course_id: &Symbol,
        module_id: &Symbol,
        completion_rate: u32,
        average_time: u64,
        difficulty_rating: &str,
    ) {
        env.events().publish(
            ("analytics", "module_analytics_updated"),
            (course_id, module_id, completion_rate, average_time, difficulty_rating),
        );
    }

    /// Emit event when a student earns an achievement
    pub fn emit_achievement_earned(
        env: &Env,
        student: &Address,
        achievement_id: &Symbol,
        achievement_type: AchievementType,
        course_id: &Symbol,
        earned_date: u64,
    ) {
        env.events().publish(
            ("analytics", "achievement_earned"),
            (student, achievement_id, achievement_type, course_id, earned_date),
        );
    }

    /// Emit event when leaderboard is updated
    pub fn emit_leaderboard_updated(
        env: &Env,
        course_id: &Symbol,
        metric_type: LeaderboardMetric,
        top_student: &Address,
        top_score: u32,
        total_entries: u32,
    ) {
        env.events().publish(
            ("analytics", "leaderboard_updated"),
            (course_id, metric_type, top_student, top_score, total_entries),
        );
    }

    /// Emit event when a progress report is generated
    pub fn emit_report_generated(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        report_period: &str,
        start_date: u64,
        end_date: u64,
        sessions_count: u32,
    ) {
        env.events().publish(
            ("analytics", "report_generated"),
            (student, course_id, report_period, start_date, end_date, sessions_count),
        );
    }

    /// Emit event when batch analytics processing is completed
    pub fn emit_batch_processed(
        env: &Env,
        batch_size: u32,
        processing_time: u64,
        updated_analytics: u32,
    ) {
        env.events().publish(
            ("analytics", "batch_processed"),
            (batch_size, processing_time, updated_analytics),
        );
    }

    /// Emit event when analytics configuration is updated
    pub fn emit_config_updated(
        env: &Env,
        admin: &Address,
        config_type: &str,
    ) {
        env.events().publish(
            ("analytics", "config_updated"),
            (admin, config_type),
        );
    }

    /// Emit event when data aggregation is performed
    pub fn emit_data_aggregated(
        env: &Env,
        course_id: &Symbol,
        date: u64,
        active_students: u32,
        total_sessions: u32,
    ) {
        env.events().publish(
            ("analytics", "data_aggregated"),
            (course_id, date, active_students, total_sessions),
        );
    }

    /// Emit event when performance trend changes
    pub fn emit_trend_change(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        old_trend: PerformanceTrend,
        new_trend: PerformanceTrend,
    ) {
        env.events().publish(
            ("analytics", "trend_change"),
            (student, course_id, old_trend, new_trend),
        );
    }

    /// Emit event when streak milestone is reached
    pub fn emit_streak_milestone(
        env: &Env,
        student: &Address,
        course_id: &Symbol,
        streak_days: u32,
        milestone_type: &str,
    ) {
        env.events().publish(
            ("analytics", "streak_milestone"),
            (student, course_id, streak_days, milestone_type),
        );
    }
}
