use soroban_sdk::{contracttype, Address, BytesN, String};

/// Certificate metadata structure
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CertificateMetadata {
    pub course_id: String,
    pub student_id: Address,
    pub instructor_id: Address,
    pub issue_date: u64,
    pub metadata_uri: String,
    pub token_id: BytesN<32>,      // Unique NFT identifier
    pub title: String,             // Certificate title
    pub description: String,       // Certificate description
    pub status: CertificateStatus, // Certificate status (Active/Revoked)
    pub expiry_date: u64,
}

#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CertificateStatus {
    Active,
    Revoked,
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
