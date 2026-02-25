use soroban_sdk::{contracttype, Address, BytesN, String, Vec};

// ─────────────────────────────────────────────────────────────
// Certificate Priority Levels
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificatePriority {
    Standard,      // 1 approval
    Premium,       // 2 approvals
    Enterprise,    // 3 approvals
    Institutional, // 5 approvals
}

impl CertificatePriority {
    pub fn required_approvals(&self) -> u32 {
        match self {
            CertificatePriority::Standard => 1,
            CertificatePriority::Premium => 2,
            CertificatePriority::Enterprise => 3,
            CertificatePriority::Institutional => 5,
        }
    }
}

// ─────────────────────────────────────────────────────────────
// Multi-Sig Configuration
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiSigConfig {
    pub course_id: String,
    pub required_approvals: u32,
    pub authorized_approvers: Vec<Address>,
    pub timeout_duration: u64,
    pub priority: CertificatePriority,
    pub auto_execute: bool,
}

// ─────────────────────────────────────────────────────────────
// Certificate Request Status
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MultiSigRequestStatus {
    Pending,
    Approved,
    Rejected,
    Executed,
    Expired,
    Cancelled,
}

// ─────────────────────────────────────────────────────────────
// Approval Record
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovalRecord {
    pub approver: Address,
    pub approved: bool,
    pub timestamp: u64,
    pub signature_hash: Option<soroban_sdk::Bytes>,
    pub comments: String,
}

// ─────────────────────────────────────────────────────────────
// Certificate Mint Parameters
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MintCertificateParams {
    pub certificate_id: BytesN<32>,
    pub course_id: String,
    pub student: Address,
    pub title: String,
    pub description: String,
    pub metadata_uri: String,
    pub expiry_date: u64,
}

// ─────────────────────────────────────────────────────────────
// Multi-Sig Certificate Request
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiSigCertificateRequest {
    pub request_id: BytesN<32>,
    pub certificate_params: MintCertificateParams,
    pub requester: Address,
    pub required_approvals: u32,
    pub current_approvals: u32,
    pub approvers: Vec<Address>,
    pub approval_records: Vec<ApprovalRecord>,
    pub status: MultiSigRequestStatus,
    pub created_at: u64,
    pub expires_at: u64,
    pub reason: String,
    pub priority: CertificatePriority,
}

// ─────────────────────────────────────────────────────────────
// Certificate (Issued)
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificateStatus {
    Active,
    Revoked,
    Expired,
    Suspended,
    Reissued,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Certificate {
    pub certificate_id: BytesN<32>,
    pub course_id: String,
    pub student: Address,
    pub title: String,
    pub description: String,
    pub metadata_uri: String,
    pub issued_at: u64,
    pub expiry_date: u64,
    pub status: CertificateStatus,
    pub issuer: Address,
    pub version: u32,
    pub blockchain_anchor: Option<soroban_sdk::Bytes>,
    pub template_id: Option<String>,
    pub share_count: u32,
}

// ─────────────────────────────────────────────────────────────
// Certificate Template
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateTemplate {
    pub template_id: String,
    pub name: String,
    pub description: String,
    pub fields: Vec<TemplateField>,
    pub created_by: Address,
    pub created_at: u64,
    pub is_active: bool,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateField {
    pub field_name: String,
    pub field_type: FieldType,
    pub is_required: bool,
    pub default_value: Option<String>,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FieldType {
    Text,
    Date,
    Number,
    Address,
    Boolean,
}

// ─────────────────────────────────────────────────────────────
// Revocation Record
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevocationRecord {
    pub certificate_id: BytesN<32>,
    pub revoked_by: Address,
    pub revoked_at: u64,
    pub reason: String,
    pub reissuance_eligible: bool,
}

// ─────────────────────────────────────────────────────────────
// Batch Operation
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchResult {
    pub total: u32,
    pub succeeded: u32,
    pub failed: u32,
    pub certificate_ids: Vec<BytesN<32>>,
}

// ─────────────────────────────────────────────────────────────
// Certificate Analytics
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateAnalytics {
    pub total_issued: u32,
    pub total_revoked: u32,
    pub total_expired: u32,
    pub total_reissued: u32,
    pub total_shared: u32,
    pub total_verified: u32,
    pub active_certificates: u32,
    pub pending_requests: u32,
    pub avg_approval_time: u64,
    pub last_updated: u64,
}

// ─────────────────────────────────────────────────────────────
// Compliance Record
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComplianceStandard {
    Iso17024,       // Personnel certification
    Iso27001,       // Information security
    GdprCompliant,  // EU data protection
    FerpaCompliant, // US educational privacy
    Custom,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComplianceRecord {
    pub certificate_id: BytesN<32>,
    pub standard: ComplianceStandard,
    pub verified_at: u64,
    pub verified_by: Address,
    pub is_compliant: bool,
    pub notes: String,
}

// ─────────────────────────────────────────────────────────────
// Share / Social Verification
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShareRecord {
    pub certificate_id: BytesN<32>,
    pub shared_by: Address,
    pub shared_at: u64,
    pub platform: String,
    pub verification_url: String,
}

// ─────────────────────────────────────────────────────────────
// Audit Trail Entry
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditAction {
    Created,
    ApprovalGranted,
    ApprovalRejected,
    Executed,
    Revoked,
    Reissued,
    Shared,
    Verified,
    ComplianceChecked,
    TemplateCreated,
    ConfigUpdated,
    Expired,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiSigAuditEntry {
    pub request_id: BytesN<32>,
    pub action: AuditAction,
    pub actor: Address,
    pub timestamp: u64,
    pub details: String,
}

// ─────────────────────────────────────────────────────────────
// Storage Keys
// ─────────────────────────────────────────────────────────────
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertDataKey {
    Admin,
    Initialized,

    // Multi-sig configs per course
    MultiSigConfig(String),

    // Pending requests
    MultiSigRequest(BytesN<32>),
    PendingRequests,

    // Certificates
    Certificate(BytesN<32>),
    StudentCertificates(Address),
    CourseCertificates(String),

    // Approver tracking
    ApproverPending(Address),

    // Templates
    Template(String),
    TemplateList,

    // Revocations
    RevocationRecord(BytesN<32>),

    // Analytics
    Analytics,

    // Compliance
    ComplianceRecord(BytesN<32>),

    // Share records
    ShareRecords(BytesN<32>),

    // Audit trail
    AuditTrail(BytesN<32>),

    // Counters
    RequestCounter,
    CertificateCounter,
}
