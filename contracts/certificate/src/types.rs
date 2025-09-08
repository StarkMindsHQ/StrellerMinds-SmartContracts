/// Packed struct for efficient storage of certificate data
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PackedCertificateData {
    pub metadata: CertificateMetadata,
    pub owner: Address,
    pub history: Vec<MetadataUpdateEntry>,
}
use soroban_sdk::{contracttype, Address, BytesN, String};

/// Certificate metadata structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateMetadata {
    pub token_id: BytesN<32>,      // Unique NFT identifier
    pub student_id: Address,
    pub instructor_id: Address,
    pub course_id: String,
    pub title: String,             // Certificate title
    pub description: String,       // Certificate description
    pub metadata_uri: String,
    pub status: CertificateStatus, // Certificate status (Active/Revoked)
    pub issue_date: u64,
    pub expiry_date: u64,
    pub original_expiry_date: u64, // Track original expiry for audit
    pub renewal_count: u32,        // Number of times renewed
    pub last_renewed_date: u64,    // Last renewal timestamp
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificateStatus {
    Active,
    Revoked,
    Expired,
    PendingRenewal,
    Renewed,
}

/// Metadata update history entry
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct MetadataUpdateEntry {
    pub updater: Address, // Who made the update
    pub timestamp: u64,   // When the update was made
    pub old_uri: String,  // Previous metadata URI
    pub new_uri: String,  // New metadata URI
}

/// Storage keys for the contract
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DataKey {
    /// Key for the admin address
    Admin,
    /// Flag indicating if contract is initialized
    Initialized,
    /// Key for storing user roles
    Role(Address),
    /// Key for tracking certificates owned by a user
    UserCerts(Address),
    /// Key for tracking certificates issued by an instructor
    Instructors(Address),
    /// Key for storing certificate metadata
    Certificates(BytesN<32>),
    /// Key for storing metadata update history
    MetadataHistory(BytesN<32>),
    /// Key for storing renewal requests
    RenewalRequest(BytesN<32>),
    /// Key for storing expiry notifications
    ExpiryNotifications(Address),
    /// Key for tracking certificates expiring soon
    ExpiringCertificates(u64), // Timestamp bucket
    /// Key for bulk operation tracking
    BulkOperations(BytesN<32>),
}

/// User role definition
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Role {
    pub can_issue: bool,
    pub can_revoke: bool,
}
/// Permission types
#[contracttype]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Permission {
    Issue,
    Revoke,
}

impl Role {
    pub fn has(&self, permission: Permission) -> bool {
        match permission {
            Permission::Issue => self.can_issue,
            Permission::Revoke => self.can_revoke,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
#[contracttype]
pub struct MintCertificateParams {
    pub certificate_id: BytesN<32>,
    pub course_id: String,
    pub student: Address,
    pub title: String,
    pub description: String,
    pub metadata_uri: String,
    pub expiry_date: u64,
}

/// Renewal request structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RenewalRequest {
    pub certificate_id: BytesN<32>,
    pub requester: Address,
    pub requested_extension: u64, // Extension period in seconds
    pub reason: String,
    pub request_date: u64,
    pub status: RenewalStatus,
    pub approver: Option<Address>,
    pub approval_date: Option<u64>,
}

/// Renewal request status
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RenewalStatus {
    Pending,
    Approved,
    Rejected,
    Expired,
}

/// Expiry notification structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExpiryNotification {
    pub certificate_id: BytesN<32>,
    pub owner: Address,
    pub expiry_date: u64,
    pub notification_type: NotificationType,
    pub created_at: u64,
    pub acknowledged: bool,
}

/// Notification types for expiry warnings
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum NotificationType {
    Warning30Days,  // 30 days before expiry
    Warning7Days,   // 7 days before expiry
    Warning1Day,    // 1 day before expiry
    Expired,        // Certificate has expired
}

/// Bulk operation parameters
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct BulkExpiryOperation {
    pub certificate_ids: Vec<BytesN<32>>,
    pub new_expiry_date: u64,
    pub reason: String,
    pub operator: Address,
}

/// Extension parameters
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ExtensionParams {
    pub certificate_id: BytesN<32>,
    pub extension_period: u64, // Extension in seconds
    pub reason: String,
    pub max_renewals: Option<u32>, // Optional limit on renewals
}
