use crate::{
    errors::AnalyticsError,
    types::{
        Achievement, AggregatedMetrics, AnalyticsConfig, AnalyticsFilter, BatchSessionUpdate,
        CourseAnalytics, LeaderboardEntry, LeaderboardMetric, LearningSession, ModuleAnalytics,
        ProgressAnalytics, ProgressReport, ReportPeriod,
    },
};
use soroban_sdk::{contracttype, Address, BytesN, Env, Symbol, Vec};

/// Analytics contract interface
// #[contracttrait]
pub trait AnalyticsTrait {
    /// Initialize the analytics contract
    fn initialize(env: Env, admin: Address, config: AnalyticsConfig) -> Result<(), AnalyticsError>;

    /// Record a new learning session
    fn record_session(env: Env, session: LearningSession) -> Result<(), AnalyticsError>;

    /// Complete a learning session with final metrics
    fn complete_session(
        env: Env,
        session_id: BytesN<32>,
        end_time: u64,
        final_score: Option<u32>,
        completion_percentage: u32,
    ) -> Result<(), AnalyticsError>;

    /// Batch update multiple sessions for efficiency
    fn batch_update_sessions(env: Env, batch: BatchSessionUpdate) -> Result<u32, AnalyticsError>;

    /// Get learning session by ID
    fn get_session(env: Env, session_id: BytesN<32>) -> Option<LearningSession>;

    /// Get all sessions for a student in a course
    fn get_student_sessions(env: Env, student: Address, course_id: Symbol) -> Vec<BytesN<32>>;

    /// Calculate and get progress analytics for a student
    fn get_progress_analytics(
        env: Env,
        student: Address,
        course_id: Symbol,
    ) -> Result<ProgressAnalytics, AnalyticsError>;

    /// Calculate and get course-wide analytics
    fn get_course_analytics(env: Env, course_id: Symbol)
        -> Result<CourseAnalytics, AnalyticsError>;

    /// Calculate and get module-specific analytics
    fn get_module_analytics(
        env: Env,
        course_id: Symbol,
        module_id: Symbol,
    ) -> Result<ModuleAnalytics, AnalyticsError>;

    /// Generate progress report for a time period
    fn generate_progress_report(
        env: Env,
        student: Address,
        course_id: Symbol,
        period: ReportPeriod,
        start_date: u64,
        end_date: u64,
    ) -> Result<ProgressReport, AnalyticsError>;

    /// Get stored progress report
    fn get_progress_report(
        env: Env,
        student: Address,
        course_id: Symbol,
        timestamp: u64,
    ) -> Option<ProgressReport>;

    /// Generate daily aggregated metrics
    fn generate_daily_metrics(
        env: Env,
        course_id: Symbol,
        date: u64,
    ) -> Result<AggregatedMetrics, AnalyticsError>;

    /// Get daily aggregated metrics
    fn get_daily_metrics(env: Env, course_id: Symbol, date: u64) -> Option<AggregatedMetrics>;

    /// Generate leaderboard for a course
    fn generate_leaderboard(
        env: Env,
        course_id: Symbol,
        metric: LeaderboardMetric,
        limit: u32,
    ) -> Result<Vec<LeaderboardEntry>, AnalyticsError>;

    /// Get leaderboard
    fn get_leaderboard(
        env: Env,
        course_id: Symbol,
        metric: LeaderboardMetric,
    ) -> Vec<LeaderboardEntry>;

    /// Get student achievements
    fn get_student_achievements(env: Env, student: Address) -> Vec<Achievement>;

    /// Get filtered analytics data
    fn get_filtered_sessions(
        env: Env,
        filter: AnalyticsFilter,
    ) -> Result<Vec<LearningSession>, AnalyticsError>;

    /// Update analytics configuration (admin only)
    fn update_config(
        env: Env,
        admin: Address,
        config: AnalyticsConfig,
    ) -> Result<(), AnalyticsError>;

    /// Get current configuration
    fn get_config(env: Env) -> Option<AnalyticsConfig>;

    /// Recalculate all analytics for a course (admin only)
    fn recalculate_course_analytics(
        env: Env,
        admin: Address,
        course_id: Symbol,
    ) -> Result<(), AnalyticsError>;

    /// Cleanup old data (admin only)
    fn cleanup_old_data(env: Env, admin: Address, before_date: u64) -> Result<u32, AnalyticsError>;

    /// Get course completion rates over time
    fn get_completion_trends(
        env: Env,
        course_id: Symbol,
        start_date: u64,
        end_date: u64,
    ) -> Vec<AggregatedMetrics>;

    /// Get student performance comparison
    fn compare_student_performance(
        env: Env,
        student1: Address,
        student2: Address,
        course_id: Symbol,
    ) -> Result<(ProgressAnalytics, ProgressAnalytics), AnalyticsError>;

    /// Get top performing students
    fn get_top_performers(
        env: Env,
        course_id: Symbol,
        metric: LeaderboardMetric,
        limit: u32,
    ) -> Vec<LeaderboardEntry>;

    /// Get struggling students (for intervention)
    fn get_struggling_students(env: Env, course_id: Symbol, threshold: u32) -> Vec<Address>;

    /// Get course engagement metrics
    fn get_engagement_metrics(
        env: Env,
        course_id: Symbol,
        date_range: u64,
    ) -> Result<CourseAnalytics, AnalyticsError>;

    /// Generate weekly summary
    fn generate_weekly_summary(
        env: Env,
        course_id: Symbol,
        week_start: u64,
    ) -> Result<Vec<AggregatedMetrics>, AnalyticsError>;

    /// Generate monthly summary
    fn generate_monthly_summary(
        env: Env,
        course_id: Symbol,
        month_start: u64,
        days_in_month: u32,
    ) -> Result<Vec<AggregatedMetrics>, AnalyticsError>;

    /// Get admin address
    fn get_admin(env: Env) -> Option<Address>;

    /// Transfer admin role
    fn transfer_admin(
        env: Env,
        current_admin: Address,
        new_admin: Address,
    ) -> Result<(), AnalyticsError>;
}
