use soroban_sdk::{contracttype, Address, Vec};

/// Role hierarchy levels (higher number = more permissions)
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum RoleLevel {
    Student = 1,
    Moderator = 2,
    Instructor = 3,
    Admin = 4,
    SuperAdmin = 5,
}

impl RoleLevel {
    pub fn from_u32(value: u32) -> Option<RoleLevel> {
        match value {
            1 => Some(RoleLevel::Student),
            2 => Some(RoleLevel::Moderator),
            3 => Some(RoleLevel::Instructor),
            4 => Some(RoleLevel::Admin),
            5 => Some(RoleLevel::SuperAdmin),
            _ => None,
        }
    }

    pub fn to_u32(&self) -> u32 {
        match self {
            RoleLevel::Student => 1,
            RoleLevel::Moderator => 2,
            RoleLevel::Instructor => 3,
            RoleLevel::Admin => 4,
            RoleLevel::SuperAdmin => 5,
        }
    }

    pub fn can_grant(&self, target_role: &RoleLevel) -> bool {
        self.to_u32() > target_role.to_u32()
    }

    pub fn can_revoke(&self, target_role: &RoleLevel) -> bool {
        self.to_u32() >= target_role.to_u32()
    }
}

/// Role definition with permissions
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Role {
    pub level: RoleLevel,
    pub permissions: Vec<Permission>,
    pub granted_by: Address,
    pub granted_at: u64,
    pub expires_at: Option<u64>, // None means never expires
}

impl Role {
    pub fn new(level: RoleLevel, permissions: Vec<Permission>, granted_by: Address, granted_at: u64) -> Self {
        Self {
            level,
            permissions,
            granted_by,
            granted_at,
            expires_at: None,
        }
    }

    pub fn with_expiry(mut self, expires_at: u64) -> Self {
        self.expires_at = Some(expires_at);
        self
    }

    pub fn has_permission(&self, permission: &Permission) -> bool {
        self.permissions.contains(permission)
    }

    pub fn has_any_permission(&self, permissions: &[Permission]) -> bool {
        permissions.iter().any(|p| self.has_permission(p))
    }

    pub fn has_all_permissions(&self, permissions: &[Permission]) -> bool {
        permissions.iter().all(|p| self.has_permission(p))
    }

    pub fn is_expired(&self, current_time: u64) -> bool {
        if let Some(expires_at) = self.expires_at {
            current_time > expires_at
        } else {
            false
        }
    }

    pub fn is_valid(&self, current_time: u64) -> bool {
        !self.is_expired(current_time)
    }
}

/// Permission types for the RBAC system
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Permission {
    // Certificate permissions
    IssueCertificate,
    RevokeCertificate,
    TransferCertificate,
    UpdateCertificateMetadata,
    
    // Course permissions
    CreateCourse,
    UpdateCourse,
    DeleteCourse,
    EnrollStudent,
    UnenrollStudent,
    
    // Progress permissions
    UpdateProgress,
    ViewProgress,
    MarkCompletion,
    
    // Role management permissions
    GrantRole,
    RevokeRole,
    TransferRole,
    
    // System permissions
    InitializeContract,
    UpgradeContract,
    EmergencyPause,
    EmergencyResume,
    
    // Token permissions
    MintTokens,
    BurnTokens,
    TransferTokens,
    
    // Batch operations
    BatchMintCertificates,
    BatchRevokeCertificates,
    
    // View permissions
    ViewAllCertificates,
    ViewAllCourses,
    ViewAllUsers,
    ViewSystemStats,
}

impl Permission {
    pub fn to_string(&self) -> &'static str {
        match self {
            Permission::IssueCertificate => "IssueCertificate",
            Permission::RevokeCertificate => "RevokeCertificate",
            Permission::TransferCertificate => "TransferCertificate",
            Permission::UpdateCertificateMetadata => "UpdateCertificateMetadata",
            Permission::CreateCourse => "CreateCourse",
            Permission::UpdateCourse => "UpdateCourse",
            Permission::DeleteCourse => "DeleteCourse",
            Permission::EnrollStudent => "EnrollStudent",
            Permission::UnenrollStudent => "UnenrollStudent",
            Permission::UpdateProgress => "UpdateProgress",
            Permission::ViewProgress => "ViewProgress",
            Permission::MarkCompletion => "MarkCompletion",
            Permission::GrantRole => "GrantRole",
            Permission::RevokeRole => "RevokeRole",
            Permission::TransferRole => "TransferRole",
            Permission::InitializeContract => "InitializeContract",
            Permission::UpgradeContract => "UpgradeContract",
            Permission::EmergencyPause => "EmergencyPause",
            Permission::EmergencyResume => "EmergencyResume",
            Permission::MintTokens => "MintTokens",
            Permission::BurnTokens => "BurnTokens",
            Permission::TransferTokens => "TransferTokens",
            Permission::BatchMintCertificates => "BatchMintCertificates",
            Permission::BatchRevokeCertificates => "BatchRevokeCertificates",
            Permission::ViewAllCertificates => "ViewAllCertificates",
            Permission::ViewAllCourses => "ViewAllCourses",
            Permission::ViewAllUsers => "ViewAllUsers",
            Permission::ViewSystemStats => "ViewSystemStats",
        }
    }
} 