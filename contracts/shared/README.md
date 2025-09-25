# Shared Contract

## Overview
A shared utilities contract providing common functionality used across multiple contracts in the StarkMinds ecosystem. This contract includes role-based access control (RBAC), permissions management, events, storage utilities, error handling, and reentrancy protection.

## Interface

### Access Control Module
```rust
// Initialize RBAC system
fn initialize(env: &Env, admin: &Address) -> Result<(), AccessControlError>

// Grant role to user
fn grant_role(env: &Env, caller: &Address, user: &Address, role_level: RoleLevel) -> Result<(), AccessControlError>

// Revoke user role
fn revoke_role(env: &Env, caller: &Address, user: &Address) -> Result<(), AccessControlError>

// Get user role
fn get_role(env: &Env, user: &Address) -> Option<Role>

// Check user permission
fn has_permission(env: &Env, user: &Address, permission: &Permission) -> bool

// Require permission (panics if not granted)
fn require_permission(env: &Env, user: &Address, permission: &Permission) -> Result<(), AccessControlError>
```

### Roles Module
```rust
// Role levels and permissions
pub enum RoleLevel {
    Student = 1,
    Moderator = 2,
    Instructor = 3,
    Admin = 4,
    SuperAdmin = 5,
}

// Permission types
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
```

### Reentrancy Guard Module
```rust
// Create reentrancy lock
fn new(env: &Env) -> ReentrancyLock

// Check if locked
fn is_locked(env: &Env) -> bool

// Lock for reentrancy protection
fn lock(env: &Env)

// Unlock after operation
fn unlock(env: &Env)
```

### Events Module
```rust
// Emit role granted event
fn emit_role_granted(env: &Env, caller: &Address, user: &Address, role: &Role)

// Emit role revoked event
fn emit_role_revoked(env: &Env, caller: &Address, user: &Address)

// Emit permission checked event
fn emit_permission_checked(env: &Env, user: &Address, permission: &Permission, granted: bool)
```

### Storage Module
```rust
// Storage key management
fn get_role_key(user: &Address) -> DataKey
fn get_permission_key(user: &Address, permission: &Permission) -> DataKey
fn get_admin_key() -> DataKey
```

### Errors Module
```rust
// Access control errors
pub enum AccessControlError {
    AlreadyInitialized,
    NotInitialized,
    Unauthorized,
    InvalidRole,
    PermissionDenied,
    RoleNotFound,
    InvalidPermission,
}
```

## Events

### Role Management Events
- `role_granted`: Emitted when a role is granted to a user
- `role_revoked`: Emitted when a role is revoked from a user
- `permission_checked`: Emitted when permission is checked for a user

### System Events
- `rbac_initialized`: Emitted when RBAC system is initialized
- `reentrancy_locked`: Emitted when reentrancy lock is acquired
- `reentrancy_unlocked`: Emitted when reentrancy lock is released

## Configuration

### Role Hierarchy
1. **Student**: Basic user with limited permissions
2. **Moderator**: Can moderate content and users
3. **Instructor**: Can create courses and issue certificates
4. **Admin**: Can manage users and system settings
5. **SuperAdmin**: Full system access

### Permission Matrix
| Permission | Student | Moderator | Instructor | Admin | SuperAdmin |
|------------|---------|-----------|------------|-------|------------|
| IssueCertificate | ❌ | ❌ | ✅ | ✅ | ✅ |
| RevokeCertificate | ❌ | ❌ | ✅ | ✅ | ✅ |
| TransferCertificate | ✅ | ✅ | ✅ | ✅ | ✅ |
| UpdateCertificateMetadata | ❌ | ❌ | ✅ | ✅ | ✅ |
| CreateCourse | ❌ | ❌ | ✅ | ✅ | ✅ |
| UpdateCourse | ❌ | ❌ | ✅ | ✅ | ✅ |
| DeleteCourse | ❌ | ❌ | ❌ | ✅ | ✅ |
| EnrollStudent | ❌ | ✅ | ✅ | ✅ | ✅ |
| UnenrollStudent | ❌ | ✅ | ✅ | ✅ | ✅ |
| UpdateProgress | ✅ | ✅ | ✅ | ✅ | ✅ |
| ViewProgress | ✅ | ✅ | ✅ | ✅ | ✅ |
| MarkCompletion | ✅ | ✅ | ✅ | ✅ | ✅ |
| GrantRole | ❌ | ❌ | ❌ | ✅ | ✅ |
| RevokeRole | ❌ | ❌ | ❌ | ✅ | ✅ |
| TransferRole | ❌ | ❌ | ❌ | ✅ | ✅ |
| InitializeContract | ❌ | ❌ | ❌ | ❌ | ✅ |
| UpgradeContract | ❌ | ❌ | ❌ | ❌ | ✅ |
| EmergencyPause | ❌ | ❌ | ❌ | ❌ | ✅ |
| EmergencyResume | ❌ | ❌ | ❌ | ❌ | ✅ |
| MintTokens | ❌ | ❌ | ❌ | ❌ | ✅ |
| BurnTokens | ❌ | ❌ | ❌ | ❌ | ✅ |
| TransferTokens | ✅ | ✅ | ✅ | ✅ | ✅ |
| BatchMintCertificates | ❌ | ❌ | ❌ | ✅ | ✅ |
| BatchRevokeCertificates | ❌ | ❌ | ❌ | ✅ | ✅ |
| ViewAllCertificates | ❌ | ✅ | ✅ | ✅ | ✅ |
| ViewAllCourses | ❌ | ✅ | ✅ | ✅ | ✅ |
| ViewAllUsers | ❌ | ❌ | ❌ | ✅ | ✅ |
| ViewSystemStats | ❌ | ❌ | ❌ | ✅ | ✅ |

## Testing

### Running Tests
```bash
# Run all tests for shared contract
cargo test --package shared

# Run specific test modules
cargo test --package shared access_control::tests
cargo test --package shared roles::tests
cargo test --package shared reentrancy_guard::tests
```

### Test Coverage
- **Access Control Tests**: RBAC system functionality
- **Role Management Tests**: Role granting and revocation
- **Permission Tests**: Permission checking and enforcement
- **Reentrancy Tests**: Reentrancy protection mechanisms
- **Storage Tests**: Persistent storage operations
- **Event Tests**: Event emission and handling

## Deployment

### Prerequisites
- Admin address for RBAC initialization
- Contract dependencies resolved

### Deployment Steps
1. Deploy the shared contract
2. Initialize RBAC system with admin address
3. Grant initial roles to system users
4. Configure permission matrix
5. Begin using shared functionality in other contracts

### Environment Setup
- Set admin address for RBAC initialization
- Configure role hierarchy
- Set up permission matrix
- Enable reentrancy protection
- Configure event emission

## Usage Examples

### Initializing RBAC
```rust
let admin = Address::generate(&env);
AccessControl::initialize(&env, &admin)?;
```

### Granting Roles
```rust
let user = Address::generate(&env);
let role_level = RoleLevel::Instructor;
AccessControl::grant_role(&env, &admin, &user, role_level)?;
```

### Checking Permissions
```rust
let permission = Permission::IssueCertificate;
if AccessControl::has_permission(&env, &user, &permission) {
    // User can issue certificates
}
```

### Using Reentrancy Protection
```rust
let _guard = ReentrancyLock::new(&env);
// Protected operation here
// Lock is automatically released when guard goes out of scope
```

## Data Structures

### Role Structure
```rust
pub struct Role {
    pub level: RoleLevel,
    pub permissions: Vec<Permission>,
    pub granted_by: Address,
    pub granted_at: u64,
}
```

### Permission Structure
```rust
pub enum Permission {
    IssueCertificate,
    RevokeCertificate,
    TransferCertificate,
    UpdateCertificateMetadata,
    GrantRole,
    RevokeRole,
    UpdateProgress,
    CreateCourse,
    ManageUsers,
}
```

### Storage Keys
```rust
pub enum DataKey {
    Role(Address),
    Permission(Address, Permission),
    Admin,
    ReentrancyLock,
}
```

## Integration with Other Contracts

### Certificate Contract
- Uses RBAC for certificate issuance permissions
- Implements reentrancy protection for critical operations
- Emits events for certificate lifecycle

### Progress Contract
- Uses RBAC for progress update permissions
- Implements reentrancy protection for progress updates

### Token Contract
- Uses RBAC for token management permissions
- Implements reentrancy protection for token operations

## Related Docs
- [RBAC Implementation](../docs/RBAC_IMPLEMENTATION.md)
- [Reentrancy Protection](../docs/REENTRANCY_PROTECTION.md)
- [Security Documentation](../docs/security.md)
- [Development Guide](../docs/development.md)