use soroban_sdk::{contracttype, Address, BytesN, String, Symbol, Vec};

/// Learning session data for detailed analytics
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct LearningSession {
    pub session_id: BytesN<32>,
    pub student: Address,
    pub course_id: Symbol,
    pub module_id: Symbol,
    pub start_time: u64,
    pub end_time: u64,
    pub completion_percentage: u32,
    pub time_spent: u64,    // in seconds
    pub interactions: u32,  // number of interactions/activities
    pub score: Option<u32>, // assessment score if applicable
    pub session_type: SessionType,
}

/// Types of learning sessions
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
#[repr(u32)]
pub enum SessionType {
    Study,
    Assessment,
    Practice,
    Review,
}

/// Comprehensive progress analytics for a student-course pair
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ProgressAnalytics {
    pub student: Address,
    pub course_id: Symbol,
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

/// Performance trend indicators
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum PerformanceTrend {
    Improving,
    Stable,
    Declining,
    Insufficient, // Not enough data
}

/// Course-wide analytics
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CourseAnalytics {
    pub course_id: Symbol,
    pub total_students: u32,
    pub active_students: u32, // students with activity in last 30 days
    pub completion_rate: u32, // percentage of students who completed the course
    pub average_completion_time: u64, // in seconds
    pub average_score: Option<u32>,
    pub dropout_rate: u32,
    pub most_difficult_module: Option<Symbol>,
    pub easiest_module: Option<Symbol>,
    pub total_time_invested: u64, // sum of all student time
}

/// Module-specific analytics
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ModuleAnalytics {
    pub course_id: Symbol,
    pub module_id: Symbol,
    pub total_attempts: u32,
    pub completion_rate: u32,
    pub average_time_to_complete: u64,
    pub average_score: Option<u32>,
    pub difficulty_rating: DifficultyRating,
    pub student_feedback_score: Option<u32>,
}

/// Module difficulty rating based on analytics
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DifficultyRating {
    Easy,     // >80% completion rate, <avg time
    Medium,   // 60-80% completion rate
    Hard,     // 40-60% completion rate
    VeryHard, // <40% completion rate
}

/// Time-based progress report
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ProgressReport {
    pub student: Address,
    pub course_id: Symbol,
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

/// Report time periods
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum ReportPeriod {
    Daily,
    Weekly,
    Monthly,
    Custom,
}

/// Student achievements and milestones
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct Achievement {
    pub achievement_id: Symbol,
    pub title: String,
    pub description: String,
    pub earned_date: u64,
    pub achievement_type: AchievementType,
}

/// Types of achievements
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AchievementType {
    Completion,  // Module/course completion
    Streak,      // Consecutive days of activity
    Speed,       // Fast completion
    Excellence,  // High scores
    Consistency, // Regular activity
}

/// Aggregated analytics for efficient querying
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AggregatedMetrics {
    pub course_id: Symbol,
    pub date: u64, // Daily aggregation timestamp
    pub active_students: u32,
    pub total_sessions: u32,
    pub total_time: u64,
    pub completions: u32,
    pub average_score: Option<u32>,
}

/// Leaderboard entry
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct LeaderboardEntry {
    pub student: Address,
    pub score: u32,
    pub rank: u32,
    pub course_id: Symbol,
    pub metric_type: LeaderboardMetric,
}

/// Leaderboard metrics
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum LeaderboardMetric {
    CompletionSpeed,
    TotalScore,
    ConsistencyScore,
    TimeSpent,
}

/// Storage keys for the analytics contract
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum DataKey {
    // Learning sessions
    Session(BytesN<32>),              // session_id
    StudentSessions(Address, Symbol), // (student, course_id) -> Vec<BytesN<32>>

    // Progress analytics
    ProgressAnalytics(Address, Symbol), // (student, course_id)

    // Course analytics
    CourseAnalytics(Symbol), // course_id
    CourseStudents(Symbol),  // course_id -> Vec<Address>

    // Module analytics
    ModuleAnalytics(Symbol, Symbol), // (course_id, module_id)

    // Time-based reports
    ProgressReport(Address, Symbol, u64), // (student, course_id, timestamp)

    // Aggregated metrics
    DailyMetrics(Symbol, u64), // (course_id, date)

    // Achievements
    StudentAchievements(Address), // student -> Vec<Achievement>

    // Leaderboards
    Leaderboard(Symbol, LeaderboardMetric), // (course_id, metric) -> Vec<LeaderboardEntry>

    // Configuration
    Admin,
    AnalyticsConfig,

    // ML Insights
    MLInsight(Address, Symbol, InsightType), // (student, course_id, type)
}

/// Configuration for analytics calculations
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AnalyticsConfig {
    pub min_session_time: u64, // Minimum time to count as valid session
    pub max_session_time: u64, // Maximum time for a single session
    pub streak_threshold: u64, // Hours between activities to maintain streak
    pub active_threshold: u64, // Days to consider student active
    pub difficulty_thresholds: DifficultyThresholds,
    pub oracle_address: Option<Address>, // External ML oracle address
}

/// Thresholds for difficulty calculation
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct DifficultyThresholds {
    pub easy_completion_rate: u32,   // >80%
    pub medium_completion_rate: u32, // 60-80%
    pub hard_completion_rate: u32,   // 40-60%
                                     // <40% is VeryHard
}

/// Batch operation for efficient data processing
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct BatchSessionUpdate {
    pub sessions: Vec<LearningSession>,
    pub update_analytics: bool,
    pub update_leaderboards: bool,
}

/// Query filters for analytics
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AnalyticsFilter {
    pub course_id: Option<Symbol>,
    pub student: Option<Address>,
    pub start_date: Option<u64>,
    pub end_date: Option<u64>,
    pub session_type: OptionalSessionType,
    pub min_score: Option<u32>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum OptionalSessionType {
    None,
    Some(SessionType),
}

/// Advanced ML Insight Types
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum InsightType {
    PatternRecognition,
    CompletionPrediction,
    Recommendation,
    AnomalyDetection,
    EngagementPrediction,
    KnowledgeGapAnalysis,
    CollaborativeInsight,
    LearningPathOptimization,
    PerformanceForecast,
    AdaptiveRecommendation,
    ContentAnalysis,
    EffectivenessMetrics,
}

/// Advanced ML-generated learning insight
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct MLInsight {
    pub insight_id: BytesN<32>,
    pub student: Address,
    pub course_id: Symbol,
    pub insight_type: InsightType,
    pub data: String, // Dynamic insight data
    pub confidence: u32,
    pub timestamp: u64,
    pub model_version: u32,
    pub metadata: Vec<(String, String)>, // Additional metadata
}

/// Enhanced predictive metrics for course completion
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PredictionMetrics {
    pub predicted_completion_date: u64,
    pub probability_of_completion: u32,
    pub risk_score: u32,
    pub estimated_remaining_hours: u32,
    pub confidence_interval: (u32, u32), // Lower and upper bounds
    pub influencing_factors: Vec<String>,
    pub recommended_actions: Vec<String>,
}

/// Advanced personalized learning recommendation
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct LearningRecommendation {
    pub target_module: Symbol,
    pub reason: String,
    pub priority: u32,
    pub estimated_difficulty: u32,
    pub prerequisites: Vec<Symbol>,
    pub learning_resources: Vec<String>,
    pub adaptive_path: bool,
}

/// Enhanced anomaly detection data
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct AnomalyData {
    pub detected_at: u64,
    pub anomaly_score: u32,
    pub description: String,
    pub anomaly_type: AnomalyType,
    pub severity: AnomalySeverity,
    pub suggested_intervention: String,
}

/// Types of learning anomalies
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AnomalyType {
    EngagementDrop,
    PerformanceDecline,
    UnusualPace,
    SkippingContent,
    Overstudying,
    InconsistentPattern,
}

/// Anomaly severity levels
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum AnomalySeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Student engagement metrics
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct EngagementMetrics {
    pub student: Address,
    pub course_id: Symbol,
    pub engagement_score: u32,
    pub activity_frequency: u32,
    pub session_regularity: u32,
    pub interaction_quality: u32,
    pub predicted_engagement: u32,
    pub risk_of_dropout: u32,
    pub trend: EngagementTrend,
}

/// Engagement trend indicators
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub enum EngagementTrend {
    Increasing,
    Stable,
    Decreasing,
    Fluctuating,
}

/// Knowledge gap analysis result
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct KnowledgeGapAnalysis {
    pub student: Address,
    pub course_id: Symbol,
    pub identified_gaps: Vec<KnowledgeGap>,
    pub mastery_level: u32,
    pub recommended_remediation: Vec<LearningRecommendation>,
    pub confidence_score: u32,
}

/// Individual knowledge gap
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct KnowledgeGap {
    pub topic: Symbol,
    pub gap_severity: u32,
    pub impact_on_progress: u32,
    pub remediation_priority: u32,
}

/// Collaborative learning insights
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CollaborativeInsight {
    pub student: Address,
    pub course_id: Symbol,
    pub peer_comparison_metrics: PeerComparison,
    pub collaboration_opportunities: Vec<CollaborationOpportunity>,
    pub social_learning_score: u32,
}

/// Peer comparison data
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct PeerComparison {
    pub percentile_rank: u32,
    pub performance_vs_peers: i32, // Positive means above average
    pub pace_vs_peers: i32,
    pub engagement_vs_peers: i32,
}

/// Collaboration opportunity
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct CollaborationOpportunity {
    pub opportunity_type: String,
    pub recommended_peers: Vec<Address>,
    pub mutual_benefit: String,
    pub confidence: u32,
}

/// Learning path optimization result
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct LearningPathOptimization {
    pub student: Address,
    pub course_id: Symbol,
    pub optimized_path: Vec<Symbol>,
    pub estimated_time_savings: u32,
    pub difficulty_progression: Vec<u32>,
    pub adaptation_reason: String,
    pub confidence: u32,
}

/// Multi-dimensional learning effectiveness metrics
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct EffectivenessMetrics {
    pub student: Address,
    pub course_id: Symbol,
    pub retention_score: u32,
    pub application_score: u32,
    pub critical_thinking_score: u32,
    pub overall_effectiveness: u32,
    pub improvement_areas: Vec<String>,
    pub strengths: Vec<String>,
}

/// Content analysis result
#[derive(Clone, Debug, Eq, PartialEq)]
#[contracttype]
pub struct ContentAnalysis {
    pub content_id: Symbol,
    pub complexity_score: u32,
    pub engagement_prediction: u32,
    pub completion_prediction: u32,
    pub content_tags: Vec<String>,
    pub prerequisites: Vec<Symbol>,
    pub learning_objectives: Vec<String>,
}
