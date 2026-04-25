use soroban_sdk::{contracttype, Address, BytesN, Vec};

#[contracttype]
#[derive(Clone)]
pub struct GdprProgressExport {
    pub course_id: BytesN<32>,
    pub progress_percentage: u32,
    pub last_updated: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct GdprCertificateExport {
    pub certificate_id: BytesN<32>,
    pub course_id: BytesN<32>,
    pub title: BytesN<32>,
    pub issued_at: u64,
    pub expiry_date: u64,
    pub status: BytesN<32>,
    pub issuer: Address,
}

#[contracttype]
#[derive(Clone)]
pub struct GdprAssessmentExport {
    pub assessment_id: u64,
    pub attempt: u32,
    pub score: u32,
    pub has_score: bool,
    pub max_score: u32,
    pub passed: bool,
    pub submitted_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct GdprAnalyticsExport {
    pub total_sessions: u32,
    pub total_time_spent: u64,
    pub average_session_time: u64,
    pub completed_modules: u32,
    pub total_modules: u32,
    pub completion_percentage: u32,
    pub average_score: u32,
    pub has_average_score: bool,
    pub streak_days: u32,
    pub performance_trend: BytesN<32>,
}

#[contracttype]
#[derive(Clone)]
pub struct GdprCommunityExport {
    pub posts_created: u32,
    pub replies_given: u32,
    pub solutions_provided: u32,
    pub contributions_made: u32,
    pub events_attended: u32,
    pub mentorship_sessions: u32,
    pub helpful_votes_received: u32,
    pub reputation_score: u32,
    pub joined_at: u64,
}

#[contracttype]
#[derive(Clone)]
pub struct GdprGamificationExport {
    pub xp_total: u32,
    pub level: u32,
    pub achievements_count: u32,
    pub guild_id: u64,
    pub has_guild: bool,
    pub current_streak: u32,
}

#[contracttype]
#[derive(Clone)]
pub struct GdprExportData {
    pub exported_at: u64,
    pub progress_list: Vec<GdprProgressExport>,
    pub certificate_list: Vec<GdprCertificateExport>,
    pub assessment_list: Vec<GdprAssessmentExport>,
    pub has_analytics: bool,
    pub analytics: GdprAnalyticsExport,
    pub has_community: bool,
    pub community: GdprCommunityExport,
    pub has_gamification: bool,
    pub gamification: GdprGamificationExport,
}

#[contracttype]
#[derive(Clone)]
pub struct GdprExportRecord {
    pub request_id: u64,
    pub requested_at: u64,
    pub delivered_at: u64,
    pub data_hash: BytesN<32>,
}
