use soroban_sdk::{contracttype, Address, BytesN, Symbol, Vec};

#[contracttype]
#[derive(Clone)]
pub struct GdprExportData {
    pub exported_at: u64,
    pub certificates: Vec<CertificateExport>,
    pub progress: Vec<ProgressExport>,
    pub assessments: Vec<AssessmentExport>,
    pub has_analytics: bool,
    pub analytics: AnalyticsExport,
    pub has_community: bool,
    pub community: CommunityExport,
    pub has_gamification: bool,
    pub gamification: GamificationExport,
}

#[contracttype]
#[derive(Clone)]
pub struct CertificateExport {
    pub certificate_id: BytesN<32>,
    pub course_id: Symbol,
    pub title: Symbol,
    pub issued_at: u64,
    pub expiry_date: u64,
    pub status: Symbol,
    pub issuer: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct ProgressExport {
    pub course_id: Symbol,
    pub progress_percentage: u32,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct AnalyticsExport {
    pub total_sessions: u32,
    pub total_time_spent: u64,
    pub average_session_time: u64,
    pub completed_modules: u32,
    pub total_modules: u32,
    pub completion_percentage: u32,
    pub average_score: u32,
    pub has_average_score: bool,
    pub streak_days: u32,
    pub performance_trend: Symbol,
}

#[contracttype]
#[derive(Clone)]
pub struct AssessmentExport {
    pub assessment_id: u64,
    pub title: Symbol,
    pub course_id: Symbol,
    pub attempt: u32,
    pub score: u32,
    pub has_score: bool,
    pub max_score: u32,
    pub passed: bool,
    pub submitted_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct CommunityExport {
    pub posts_count: u32,
    pub replies_count: u32,
    pub solutions_count: u32,
    pub reputation_score: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct GamificationExport {
    pub xp_total: u64,
    pub level: u32,
    pub achievements_count: u32,
    pub guild_id: u64,
    pub has_guild: bool,
    pub endorsements_count: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct ExportRequest {
    pub request_id: u64,
    pub user: Address,
    pub requested_at: u64,
    pub status: ExportStatus,
    pub data: GdprExportData,
    pub has_data: bool,
    pub data_hash: BytesN<32>,
    pub has_hash: bool,
}

#[contracttype]
#[derive(Clone, PartialEq)]
pub enum ExportStatus {
    Pending,
    Processing,
    Ready,
    Delivered,
    Expired,
}

#[contracttype]
#[derive(Clone)]
pub struct ExportRecord {
    pub request_id: u64,
    pub requested_at: u64,
    pub delivered_at: u64,
    pub data_hash: BytesN<32>,
}