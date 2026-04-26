use soroban_sdk::{contracttype, Address, Bytes, BytesN, String, Vec};

// ─────────────────────────────────────────────────────────────
// Certificate Priority Levels
// ─────────────────────────────────────────────────────────────
/// Priority level assigned to a certificate, determining how many approvals are required.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificatePriority {
    /// Requires 1 approval.
    Standard,
    /// Requires 2 approvals.
    Premium,
    /// Requires 3 approvals.
    Enterprise,
    /// Requires 5 approvals.
    Institutional,
}

/// Supported blockchain networks for certificate export operations.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ChainId {
    /// The Stellar network.
    Stellar,
    /// The Ethereum mainnet.
    Ethereum,
    /// The Polygon (MATIC) network.
    Polygon,
    /// The Binance Smart Chain network.
    Bsc,
    /// The Arbitrum Layer-2 network.
    Arbitrum,
    /// A custom or private blockchain network.
    Custom,
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
/// Configuration for multi-signature certificate issuance on a per-course basis.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiSigConfig {
    /// Identifier of the course this configuration applies to.
    pub course_id: String,
    /// Number of approvals required before the certificate can be executed.
    pub required_approvals: u32,
    /// List of addresses permitted to approve certificate requests.
    pub authorized_approvers: Vec<Address>,
    /// Duration in seconds before a pending request expires.
    pub timeout_duration: u64,
    /// Priority level that controls the approval threshold.
    pub priority: CertificatePriority,
    /// Whether the certificate is automatically executed once the approval threshold is met.
    pub auto_execute: bool,
}

// ─────────────────────────────────────────────────────────────
// Certificate Request Status
// ─────────────────────────────────────────────────────────────
/// Lifecycle state of a multi-signature certificate request.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum MultiSigRequestStatus {
    /// Awaiting approvals.
    Pending,
    /// Approval threshold has been reached.
    Approved,
    /// Request was explicitly rejected.
    Rejected,
    /// Certificate has been minted and the request is fulfilled.
    Executed,
    /// Request timed out before the required approvals were collected.
    Expired,
    /// Request was cancelled by the requester or an admin.
    Cancelled,
}

// ─────────────────────────────────────────────────────────────
// Approval Record
// ─────────────────────────────────────────────────────────────
/// Individual approval or rejection cast by an authorized approver.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ApprovalRecord {
    /// Address of the approver who cast this record.
    pub approver: Address,
    /// Whether the approver approved (`true`) or rejected (`false`) the request.
    pub approved: bool,
    /// Unix timestamp (seconds) when the approval was recorded.
    pub timestamp: u64,
    /// Optional cryptographic signature hash provided by the approver.
    pub signature_hash: Option<Bytes>,
    /// Free-form comments left by the approver.
    pub comments: String,
}

// ─────────────────────────────────────────────────────────────
// Certificate Mint Parameters
// ─────────────────────────────────────────────────────────────
/// Parameters required to mint a new certificate on-chain.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MintCertificateParams {
    /// Unique 32-byte identifier for the certificate.
    pub certificate_id: BytesN<32>,
    /// Identifier of the course associated with this certificate.
    pub course_id: String,
    /// Address of the student receiving the certificate.
    pub student: Address,
    /// Human-readable title of the certificate.
    pub title: String,
    /// Description of what the certificate represents.
    pub description: String,
    /// URI pointing to off-chain certificate metadata.
    pub metadata_uri: String,
    /// Unix timestamp (seconds) after which the certificate expires; 0 means no expiry.
    pub expiry_date: u64,
}

// ─────────────────────────────────────────────────────────────
// Multi-Sig Certificate Request
// ─────────────────────────────────────────────────────────────
/// A pending or completed multi-signature certificate issuance request.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiSigCertificateRequest {
    /// Unique 32-byte identifier for this request.
    pub request_id: BytesN<32>,
    /// Parameters that will be used to mint the certificate once approved.
    pub certificate_params: MintCertificateParams,
    /// Address that submitted the request.
    pub requester: Address,
    /// Number of approvals needed to fulfil this request.
    pub required_approvals: u32,
    /// Number of approvals collected so far.
    pub current_approvals: u32,
    /// List of addresses that have been asked to approve.
    pub approvers: Vec<Address>,
    /// Detailed approval or rejection records from each approver.
    pub approval_records: Vec<ApprovalRecord>,
    /// Current lifecycle status of the request.
    pub status: MultiSigRequestStatus,
    /// Unix timestamp (seconds) when the request was created.
    pub created_at: u64,
    /// Unix timestamp (seconds) after which the request is considered expired.
    pub expires_at: u64,
    /// Reason or justification provided by the requester.
    pub reason: String,
    /// Priority level of this request, which may affect approval thresholds.
    pub priority: CertificatePriority,
}

// ─────────────────────────────────────────────────────────────
// Certificate (Issued)
// ─────────────────────────────────────────────────────────────
/// Current lifecycle status of an issued certificate.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificateStatus {
    /// Certificate is valid and in use.
    Active,
    /// Certificate has been permanently revoked.
    Revoked,
    /// Certificate has passed its expiry date.
    Expired,
    /// Certificate is temporarily suspended.
    Suspended,
    /// Certificate was revoked and then reissued as a new version.
    Reissued,
}

/// An on-chain record of an issued certificate.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Certificate {
    /// Unique 32-byte identifier for this certificate.
    pub certificate_id: BytesN<32>,
    /// Identifier of the course this certificate was awarded for.
    pub course_id: String,
    /// Address of the student who holds this certificate.
    pub student: Address,
    /// Human-readable title of the certificate.
    pub title: String,
    /// Description of the achievement the certificate represents.
    pub description: String,
    /// URI pointing to off-chain metadata for the certificate.
    pub metadata_uri: String,
    /// Unix timestamp (seconds) when the certificate was issued.
    pub issued_at: u64,
    /// Unix timestamp (seconds) when the certificate expires; 0 means no expiry.
    pub expiry_date: u64,
    /// Current status of the certificate.
    pub status: CertificateStatus,
    /// Address of the entity that issued this certificate.
    pub issuer: Address,
    /// Incremental version number, updated on reissuance.
    pub version: u32,
    /// Optional on-chain anchor (e.g., block hash) for additional provenance.
    pub blockchain_anchor: Option<Bytes>,
    /// Optional identifier of the template used to generate this certificate.
    pub template_id: Option<String>,
    /// Number of times this certificate has been shared externally.
    pub share_count: u32,
}

// ─────────────────────────────────────────────────────────────
// Certificate Template
// ─────────────────────────────────────────────────────────────
/// A reusable template that defines the structure and fields of a certificate.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateTemplate {
    /// Unique identifier for this template.
    pub template_id: String,
    /// Display name of the template.
    pub name: String,
    /// Description of the template's purpose.
    pub description: String,
    /// Ordered list of fields that certificates based on this template must include.
    pub fields: Vec<TemplateField>,
    /// Address of the user who created the template.
    pub created_by: Address,
    /// Unix timestamp (seconds) when the template was created.
    pub created_at: u64,
    /// Whether the template is available for use.
    pub is_active: bool,
}

/// A single field definition within a certificate template.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TemplateField {
    /// Name of the field as it appears on the certificate.
    pub field_name: String,
    /// Data type expected for this field.
    pub field_type: FieldType,
    /// Whether the field must be provided when minting a certificate.
    pub is_required: bool,
    /// Optional default value to use when none is supplied.
    pub default_value: Option<String>,
}

/// Supported data types for template fields.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FieldType {
    /// Free-form text value.
    Text,
    /// A date value.
    Date,
    /// A numeric value.
    Number,
    /// A Stellar address.
    Address,
    /// A true/false flag.
    Boolean,
}

// ─────────────────────────────────────────────────────────────
// Revocation Record
// ─────────────────────────────────────────────────────────────
/// Record of a certificate revocation event.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RevocationRecord {
    /// Identifier of the revoked certificate.
    pub certificate_id: BytesN<32>,
    /// Address that performed the revocation.
    pub revoked_by: Address,
    /// Unix timestamp (seconds) when the revocation occurred.
    pub revoked_at: u64,
    /// Human-readable reason for revocation.
    pub reason: String,
    /// Whether a new certificate may be issued to replace this one.
    pub reissuance_eligible: bool,
}

// ─────────────────────────────────────────────────────────────
// Batch Operation
// ─────────────────────────────────────────────────────────────
/// Summary result of a batch certificate operation.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BatchResult {
    /// Total number of certificates processed in the batch.
    pub total: u32,
    /// Number of certificates that were processed successfully.
    pub succeeded: u32,
    /// Number of certificates that failed to process.
    pub failed: u32,
    /// Identifiers of certificates that were successfully issued.
    pub certificate_ids: Vec<BytesN<32>>,
}

// ─────────────────────────────────────────────────────────────
// Certificate Analytics
// ─────────────────────────────────────────────────────────────
/// Aggregate analytics counters for the certificate contract.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateAnalytics {
    /// Total number of certificates ever issued.
    pub total_issued: u32,
    /// Total number of certificates that have been revoked.
    pub total_revoked: u32,
    /// Total number of certificates that have expired.
    pub total_expired: u32,
    /// Total number of certificates that have been reissued.
    pub total_reissued: u32,
    /// Total number of times certificates have been shared.
    pub total_shared: u32,
    /// Total number of certificate verifications performed.
    pub total_verified: u32,
    /// Current count of certificates in the Active status.
    pub active_certificates: u32,
    /// Current count of multi-sig requests awaiting approval.
    pub pending_requests: u32,
    /// Rolling average time (seconds) from request creation to execution.
    pub avg_approval_time: u64,
    /// Unix timestamp (seconds) when these analytics were last updated.
    pub last_updated: u64,
}

// ─────────────────────────────────────────────────────────────
// Compliance Record
// ─────────────────────────────────────────────────────────────
/// Recognised compliance or regulatory standards that a certificate may be verified against.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComplianceStandard {
    /// ISO 9001 quality management systems standard.
    Iso9001,
    /// ISO 17024 personnel certification standard.
    Iso17024,
    /// ISO 27001 information security management standard.
    Iso27001,
    /// General Data Protection Regulation (EU) compliance.
    GdprCompliant,
    /// Family Educational Rights and Privacy Act (US) compliance.
    FerpaCompliant,
    /// A bespoke or platform-specific compliance standard.
    Custom,
}

/// Record of a compliance verification performed on a certificate.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ComplianceRecord {
    /// Identifier of the certificate that was verified.
    pub certificate_id: BytesN<32>,
    /// The compliance standard against which the certificate was checked.
    pub standard: ComplianceStandard,
    /// Unix timestamp (seconds) when the verification was performed.
    pub verified_at: u64,
    /// Address of the entity that performed the verification.
    pub verified_by: Address,
    /// Whether the certificate passed the compliance check.
    pub is_compliant: bool,
    /// Additional notes from the verifier.
    pub notes: String,
}

// ─────────────────────────────────────────────────────────────
// Share / Social Verification
// ─────────────────────────────────────────────────────────────
/// Record of a certificate being shared to an external platform.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShareRecord {
    /// Identifier of the shared certificate.
    pub certificate_id: BytesN<32>,
    /// Address of the certificate holder who performed the share.
    pub shared_by: Address,
    /// Unix timestamp (seconds) when the share occurred.
    pub shared_at: u64,
    /// Name of the external platform to which the certificate was shared.
    pub platform: String,
    /// URL where third parties can verify the shared certificate.
    pub verification_url: String,
}

// ─────────────────────────────────────────────────────────────
// Blockchain Export Record
// ─────────────────────────────────────────────────────────────
/// Record of a certificate being exported to an external blockchain ledger.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExportRecord {
    /// Identifier of the exported certificate.
    pub certificate_id: BytesN<32>,
    /// Address of the user who initiated the export.
    pub exported_by: Address,
    /// Unix timestamp (seconds) when the export occurred.
    pub exported_at: u64,
    /// The target blockchain ledger.
    pub target_chain: ChainId,
    /// Optional name of the custom chain if target_chain is Custom.
    pub custom_chain_name: Option<String>,
    /// Transaction hash or reference on the target chain.
    pub target_tx_hash: String,
}

// ─────────────────────────────────────────────────────────────
// Audit Trail Entry
// ─────────────────────────────────────────────────────────────
/// Action types that may appear in a certificate audit trail.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AuditAction {
    /// A new multi-sig request was created.
    Created,
    /// An approver granted approval.
    ApprovalGranted,
    /// An approver rejected the request.
    ApprovalRejected,
    /// The certificate was minted after reaching the approval threshold.
    Executed,
    /// The certificate was revoked.
    Revoked,
    /// The certificate was reissued as a new version.
    Reissued,
    /// The certificate was shared to an external platform.
    Shared,
    /// The certificate was verified by a third party.
    Verified,
    /// A compliance check was performed on the certificate.
    ComplianceChecked,
    /// A new certificate template was created.
    TemplateCreated,
    /// The multi-sig configuration for a course was updated.
    ConfigUpdated,
    /// The certificate passed its expiry date.
    Expired,
    /// The certificate was exported to an external blockchain ledger.
    Exported,
}

/// A single entry in the audit trail for a multi-sig certificate request.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MultiSigAuditEntry {
    /// Identifier of the request this entry belongs to.
    pub request_id: BytesN<32>,
    /// The action that was taken.
    pub action: AuditAction,
    /// Address of the user or contract that performed the action.
    pub actor: Address,
    /// Unix timestamp (seconds) when the action occurred.
    pub timestamp: u64,
    /// Human-readable details about the action.
    pub details: String,
}

// ─────────────────────────────────────────────────────────────
// Certificate Recovery
// ─────────────────────────────────────────────────────────────
/// Status of a certificate recovery request.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveryStatus {
    /// Recovery request is pending verification.
    Pending,
    /// Recovery request has been approved.
    Approved,
    /// Recovery request was rejected.
    Rejected,
    /// Recovery process completed successfully.
    Recovered,
}

/// Backup record for certificate recovery purposes.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateBackup {
    /// Unique identifier for this backup.
    pub backup_id: BytesN<32>,
    /// ID of the certificate being backed up.
    pub certificate_id: BytesN<32>,
    /// Student's address.
    pub student: Address,
    /// Encrypted backup data hash for verification.
    pub data_hash: BytesN<32>,
    /// Unix timestamp (seconds) when backup was created.
    pub created_at: u64,
    /// Unix timestamp (seconds) after which backup expires.
    pub expires_at: u64,
    /// Current status of the backup.
    pub status: RecoveryStatus,
}

/// Certificate recovery request.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryRequest {
    /// Unique identifier for recovery request.
    pub request_id: BytesN<32>,
    /// ID of certificate to recover.
    pub certificate_id: BytesN<32>,
    /// Student requesting recovery.
    pub requester: Address,
    /// Backup ID being used for recovery.
    pub backup_id: BytesN<32>,
    /// Current status of recovery request.
    pub status: RecoveryStatus,
    /// Unix timestamp (seconds) when request was created.
    pub created_at: u64,
    /// Unix timestamp (seconds) after which request expires.
    pub expires_at: u64,
    /// Verification data provided by requester.
    pub verification_data: Bytes,
}

// ─────────────────────────────────────────────────────────────
// Storage Keys
// ─────────────────────────────────────────────────────────────
/// Storage key enum used to namespace all certificate contract state in the ledger.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertDataKey {
    /// Address of the contract administrator.
    Admin,
    /// Flag indicating whether the contract has been initialised.
    Initialized,

    // Multi-sig configs per course
    /// Multi-sig configuration keyed by course identifier.
    MultiSigConfig(String),

    // Pending requests
    /// A specific multi-sig certificate request keyed by its ID.
    MultiSigRequest(BytesN<32>),
    /// List of all currently pending request IDs.
    PendingRequests,

    // Certificates
    /// A specific issued certificate keyed by its ID.
    Certificate(BytesN<32>),
    /// List of certificate IDs belonging to a student.
    StudentCertificates(Address),
    /// List of certificate IDs issued for a particular course.
    CourseCertificates(String),
    /// Mapping from course and student to their certificate ID to prevent duplicates.
    CourseStudentCertificate(String, Address),

    // Approver tracking
    /// List of pending request IDs assigned to an approver.
    ApproverPending(Address),

    // Templates
    /// A specific certificate template keyed by its ID.
    Template(String),
    /// List of all registered template IDs.
    TemplateList,

    // Revocations
    /// Revocation record for a specific certificate.
    RevocationRecord(BytesN<32>),

    // Analytics
    /// Global certificate analytics counters.
    Analytics,

    // Compliance
    /// Compliance record for a specific certificate.
    ComplianceRecord(BytesN<32>),

    // Share records
    /// List of share records for a specific certificate.
    ShareRecords(BytesN<32>),

    // Audit trail
    /// Audit trail entries for a specific multi-sig request.
    AuditTrail(BytesN<32>),

    // Counters
    /// Monotonically increasing counter for multi-sig request IDs.
    RequestCounter,
    /// Monotonically increasing counter for certificate IDs.
    CertificateCounter,

    // Rate Limiting
    RateLimit(Address, u64), // (user, operation_id) -> RateLimitState
    RateLimitCfg,            // CertRateLimitConfig

    // Certificate Recovery
    /// Backup record for a certificate.
    CertificateBackup(BytesN<32>),
    /// List of backup IDs for a student.
    StudentBackups(Address),
    /// Recovery request keyed by request ID.
    RecoveryRequest(BytesN<32>),
    /// List of pending recovery request IDs.
    PendingRecoveryRequests,
    /// List of export records for a specific certificate.
    ExportRecords(BytesN<32>),
}

/// Configurable rate limits for certificate operations.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertRateLimitConfig {
    pub max_requests_per_day: u32,
    pub window_seconds: u64,
}
