use soroban_sdk::contracterror;

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum AccessControlError {
    // Initialization errors
    AlreadyInitialized = 1,
    NotInitialized = 2,

    // Authorization errors
    Unauthorized = 3,
    RoleNotFound = 4,
    PermissionDenied = 5,

    // Role management errors
    RoleAlreadyExists = 6,
    CannotRevokeOwnRole = 7,
    CannotTransferOwnRole = 8,

    // Permission errors
    InvalidPermission = 9,
    PermissionNotGranted = 10,

    // Role hierarchy errors
    InvalidRoleHierarchy = 11,
    CannotGrantHigherRole = 12,

    // Input validation errors
    InvalidAddress = 13,
    InvalidRole = 14,
}
