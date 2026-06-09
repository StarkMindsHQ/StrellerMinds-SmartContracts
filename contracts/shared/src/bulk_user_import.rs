//! Bulk user import for the shared/admin contract (fixes #438).
//!
//! Allows administrators to register multiple users with assigned roles in a
//! single transaction, with per-entry validation and duplicate detection.

use crate::access_control::AccessControl;
use crate::errors::AccessControlError;
use crate::roles::RoleLevel;
use crate::storage::AccessControlStorage;
use soroban_sdk::{contracttype, Address, Env, Vec};

/// A single user entry in a bulk import request.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserImportEntry {
    /// Address of the user to register.
    pub user: Address,
    /// Role level to assign to the user.
    pub role: RoleLevel,
}

/// Per-entry result returned after a bulk import.
#[contracttype]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct UserImportResult {
    /// Address that was processed.
    pub user: Address,
    /// `true` if the user was successfully imported.
    pub success: bool,
    /// Error code if `success` is `false`; 0 otherwise.
    pub error_code: u32,
}

/// Maximum number of users that can be imported in a single call (gas guard).
const MAX_IMPORT_BATCH: u32 = 50;

/// Bulk-import users and assign roles.
///
/// # Arguments
/// * `env`     - The contract environment.
/// * `admin`   - Address of the caller; must be an initialized admin.
/// * `entries` - List of `(user, role)` pairs to import.
///
/// # Returns
/// A `Vec<UserImportResult>` with one entry per input, indicating success or
/// the error code for failures (duplicate detection, invalid role, etc.).
///
/// # Errors
/// Returns `AccessControlError::NotInitialized` if the contract has not been
/// set up, or `AccessControlError::PermissionDenied` if `admin` is not an admin.
pub fn bulk_import_users(
    env: &Env,
    admin: &Address,
    entries: Vec<UserImportEntry>,
) -> Result<Vec<UserImportResult>, AccessControlError> {
    // Validate contract is initialized and caller is admin
    if !AccessControlStorage::is_initialized(env) {
        return Err(AccessControlError::NotInitialized);
    }
    admin.require_auth();
    let stored_admin = AccessControlStorage::get_admin(env);
    if *admin != stored_admin {
        return Err(AccessControlError::PermissionDenied);
    }

    // Gas guard
    if entries.len() > MAX_IMPORT_BATCH {
        return Err(AccessControlError::PermissionDenied); // reuse closest error; callers should split batches
    }

    let mut results: Vec<UserImportResult> = Vec::new(env);

    for entry in entries.iter() {
        // Duplicate detection: skip users that already have a role assigned
        if AccessControlStorage::has_role(env, &entry.user) {
            results.push_back(UserImportResult {
                user: entry.user.clone(),
                success: false,
                error_code: AccessControlError::RoleAlreadyExists as u32,
            });
            continue;
        }

        // Grant the role via the shared AccessControl module
        match AccessControl::grant_role(env, admin, &entry.user, entry.role.clone()) {
            Ok(()) => {
                results.push_back(UserImportResult {
                    user: entry.user.clone(),
                    success: true,
                    error_code: 0,
                });
            }
            Err(e) => {
                results.push_back(UserImportResult {
                    user: entry.user.clone(),
                    success: false,
                    error_code: e as u32,
                });
            }
        }
    }

    Ok(results)
}
