use soroban_sdk::contracterror;

use crate::error_codes::{ErrorCodeCategory, ErrorDescriptor, StandardizedError};

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AccessControlError {
    // Initialization errors
    /// The access control module has already been initialized.
    AlreadyInitialized = 1,
    /// The access control module has not been initialized yet.
    NotInitialized = 2,

    // Authorization errors
    /// Caller does not have the required authority to perform this action.
    Unauthorized = 3,
    /// The specified role does not exist in the system.
    RoleNotFound = 4,
    /// The caller's role does not grant the required permission.
    PermissionDenied = 5,

    // Role management errors
    /// A role with this identifier has already been registered.
    RoleAlreadyExists = 6,
    /// An admin cannot revoke their own role.
    CannotRevokeOwnRole = 7,
    /// An admin cannot transfer their own role to another address.
    CannotTransferOwnRole = 8,

    // Permission errors
    /// The provided permission identifier is not recognized.
    InvalidPermission = 9,
    /// The requested permission has not been granted to the caller.
    PermissionNotGranted = 10,

    // Role hierarchy errors
    /// The specified role hierarchy relationship is not valid.
    InvalidRoleHierarchy = 11,
    /// A role cannot grant permissions to a higher-ranked role.
    CannotGrantHigherRole = 12,

    // Input validation errors
    /// The provided address is invalid or zero.
    InvalidAddress = 13,
    /// The provided role identifier is invalid or empty.
    InvalidRole = 14,
    /// The requested role template was not found.
    TemplateNotFound = 15,
}

impl StandardizedError for AccessControlError {
    fn descriptor(&self) -> ErrorDescriptor {
        match self {
            Self::AlreadyInitialized => ErrorDescriptor::new(
                "SHR-001",
                "Contract is already initialized",
                "Reuse the existing access-control state instead of reinitializing it",
                ErrorCodeCategory::Initialization,
            ),
            Self::NotInitialized => ErrorDescriptor::new(
                "SHR-002",
                "Contract is not initialized",
                "Initialize the contract before invoking protected operations",
                ErrorCodeCategory::Initialization,
            ),
            Self::Unauthorized => ErrorDescriptor::new(
                "SHR-003",
                "Caller is not authorized for this operation",
                "Retry with an account that has the required role or permission",
                ErrorCodeCategory::Authorization,
            ),
            Self::RoleNotFound => ErrorDescriptor::new(
                "SHR-004",
                "Requested role does not exist",
                "Create the role first or use a valid existing role identifier",
                ErrorCodeCategory::NotFound,
            ),
            Self::PermissionDenied => ErrorDescriptor::new(
                "SHR-005",
                "Permission was denied for the requested action",
                "Review the caller permissions and grant the missing capability if appropriate",
                ErrorCodeCategory::Authorization,
            ),
            Self::RoleAlreadyExists => ErrorDescriptor::new(
                "SHR-006",
                "Role already exists",
                "Use the existing role or choose a different role identifier",
                ErrorCodeCategory::BusinessLogic,
            ),
            Self::CannotRevokeOwnRole => ErrorDescriptor::new(
                "SHR-007",
                "Caller cannot revoke their own role",
                "Use another authorized administrator to perform the revoke action",
                ErrorCodeCategory::BusinessLogic,
            ),
            Self::CannotTransferOwnRole => ErrorDescriptor::new(
                "SHR-008",
                "Caller cannot transfer their own role",
                "Perform the transfer from a separate authorized account",
                ErrorCodeCategory::BusinessLogic,
            ),
            Self::InvalidPermission => ErrorDescriptor::new(
                "SHR-009",
                "Permission identifier is invalid",
                "Provide a supported permission value and retry",
                ErrorCodeCategory::Validation,
            ),
            Self::PermissionNotGranted => ErrorDescriptor::new(
                "SHR-010",
                "Required permission has not been granted",
                "Grant the permission before retrying the requested action",
                ErrorCodeCategory::Authorization,
            ),
            Self::InvalidRoleHierarchy => ErrorDescriptor::new(
                "SHR-011",
                "Role hierarchy is invalid",
                "Adjust the requested role assignment to fit the configured hierarchy",
                ErrorCodeCategory::Validation,
            ),
            Self::CannotGrantHigherRole => ErrorDescriptor::new(
                "SHR-012",
                "Caller cannot grant a higher role",
                "Use an administrator with sufficient privileges for this assignment",
                ErrorCodeCategory::Authorization,
            ),
            Self::InvalidAddress => ErrorDescriptor::new(
                "SHR-013",
                "Address value is invalid",
                "Provide a valid Stellar address and retry",
                ErrorCodeCategory::Validation,
            ),
            Self::InvalidRole => ErrorDescriptor::new(
                "SHR-014",
                "Role identifier is invalid",
                "Use a supported role identifier and retry",
                ErrorCodeCategory::Validation,
            ),
            Self::TemplateNotFound => ErrorDescriptor::new(
                "SHR-015",
                "Referenced template was not found",
                "Create the template first or update the request to reference an existing template",
                ErrorCodeCategory::NotFound,
            ),
        }
    }
}
