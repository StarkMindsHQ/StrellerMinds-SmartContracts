# Certificate Contract

## Overview
A comprehensive certificate management system for educational credentials on the Stellar blockchain. This contract handles the issuance, transfer, revocation, and lifecycle management of educational certificates with advanced features including expiry management, multi-signature approval, and prerequisite validation.

## Interface

### Core Functions
```rust
// Initialize the contract with admin
fn initialize(env: Env, admin: Address) -> Result<(), CertificateError>

// Mint a new certificate
fn mint_certificate(env: Env, issuer: Address, params: MintCertificateParams) -> Result<(), CertificateError>

// Revoke a certificate
fn revoke_certificate(env: Env, revoker: Address, certificate_id: BytesN<32>) -> Result<(), CertificateError>

// Transfer certificate ownership
fn transfer_certificate(env: Env, from: Address, to: Address, certificate_id: BytesN<32>) -> Result<(), CertificateError>

// Update certificate metadata URI
fn update_certificate_uri(env: Env, updater: Address, certificate_id: BytesN<32>, new_uri: String) -> Result<(), CertificateError>

// Get certificate metadata
fn get_certificate(env: Env, certificate_id: BytesN<32>) -> Option<CertificateMetadata>

// Get user's certificates
fn get_user_certificates(env: Env, user: Address) -> Vec<BytesN<32>>

// Check certificate validity
fn is_valid_certificate(env: Env, certificate_id: BytesN<32>) -> bool
```

### Batch Operations
```rust
// Mint multiple certificates in a single transaction
fn mint_certificates_batch(env: Env, issuer: Address, params_list: Vec<MintCertificateParams>) -> Result<(), CertificateError>
```

### Expiry Management
```rust
// Request certificate renewal
fn request_certificate_renewal(env: Env, requester: Address, certificate_id: BytesN<32>, requested_extension: u64, reason: String) -> Result<(), CertificateError>

// Process renewal request (admin only)
fn process_renewal_request(env: Env, approver: Address, certificate_id: BytesN<32>, approved: bool, admin_reason: Option<String>) -> Result<(), CertificateError>

// Extend certificate expiry (admin only)
fn extend_certificate_expiry(env: Env, admin: Address, certificate_id: BytesN<32>, extension_period: u64, reason: String) -> Result<(), CertificateError>

// Bulk extend multiple certificates
fn bulk_extend_certificates(env: Env, admin: Address, certificate_ids: Vec<BytesN<32>>, new_expiry_date: u64, reason: String) -> Result<Vec<BytesN<32>>, CertificateError>
```

### Multi-Signature System
```rust
// Configure multi-signature requirements for a course
fn configure_multisig(env: Env, admin: Address, config: MultiSigConfig) -> Result<(), CertificateError>

// Create a multi-signature certificate request
fn create_multisig_request(env: Env, requester: Address, params: MintCertificateParams, reason: String) -> Result<BytesN<32>, CertificateError>

// Approve or reject a multi-signature request
fn process_multisig_approval(env: Env, approver: Address, request_id: BytesN<32>, approved: bool, comments: String, signature_hash: Option<BytesN<32>>) -> Result<(), CertificateError>

// Execute certificate issuance after multi-signature approval
fn execute_multisig_request(env: Env, executor: Address, request_id: BytesN<32>) -> Result<(), CertificateError>
```

### Prerequisite Management
```rust
// Define prerequisites for a course
fn define_prerequisites(env: Env, admin: Address, course_prerequisite: CoursePrerequisite) -> Result<(), CertificateError>

// Check if student meets prerequisites
fn check_prerequisites(env: Env, student: Address, course_id: String, progress_contract: Address) -> Result<PrerequisiteCheckResult, CertificateError>

// Grant prerequisite override
fn grant_prerequisite_override(env: Env, admin: Address, override_data: PrerequisiteOverride) -> Result<(), CertificateError>

// Generate learning path for student
fn generate_learning_path(env: Env, student: Address, target_course: String, progress_contract: Address) -> Result<LearningPath, CertificateError>
```

### Role-Based Access Control
```rust
// Grant role to user
fn grant_role(env: Env, user: Address, role_level: u32) -> Result<(), CertificateError>

// Revoke user role
fn revoke_role(env: Env, user: Address) -> Result<(), CertificateError>

// Check user permissions
fn has_permission(env: Env, user: Address, permission: u32) -> bool
```

## Events

### Certificate Lifecycle Events
- `certificate_minted`: Emitted when a new certificate is issued
- `certificate_revoked`: Emitted when a certificate is revoked
- `certificate_transferred`: Emitted when certificate ownership changes
- `metadata_updated`: Emitted when certificate metadata is updated

### Expiry Management Events
- `renewal_requested`: Emitted when a renewal is requested
- `renewal_processed`: Emitted when a renewal request is approved/rejected
- `certificate_extended`: Emitted when certificate expiry is extended
- `certificate_expired`: Emitted when a certificate expires

### Multi-Signature Events
- `multisig_request_created`: Emitted when a multi-signature request is created
- `multisig_approval_processed`: Emitted when an approval is processed
- `multisig_request_executed`: Emitted when a request is executed

### Prerequisite Events
- `prerequisites_defined`: Emitted when course prerequisites are set
- `prerequisite_override_granted`: Emitted when an override is granted
- `learning_path_generated`: Emitted when a learning path is created

## Configuration

### Certificate Parameters
- `certificate_id`: Unique 32-byte identifier for each certificate
- `course_id`: String identifying the course (not Symbol)
- `student_id`: Address of the certificate recipient
- `instructor_id`: Address of the certificate issuer
- `metadata_uri`: URI pointing to certificate metadata
- `expiry_date`: Unix timestamp when certificate expires

### Multi-Signature Configuration
- `required_approvals`: Number of approvals needed
- `approvers`: List of authorized approver addresses
- `timeout_seconds`: Request timeout period
- `auto_execute`: Whether to auto-execute when threshold is met

### Prerequisite Configuration
- `required_courses`: List of courses that must be completed
- `minimum_scores`: Minimum scores required for prerequisite courses
- `completion_dates`: Required completion timeframes

## Testing

### Running Tests
```bash
# Run all tests for certificate contract
cargo test --package certificate

# Run specific test modules
cargo test --package certificate metadata_validation_tests
cargo test --package certificate multisig_tests
cargo test --package certificate prerequisite_tests
cargo test --package certificate expiry_tests
```

### Test Coverage
- **Unit Tests**: Individual function testing for core operations
- **Integration Tests**: Complete certificate lifecycle workflows
- **Metadata Validation Tests**: Comprehensive validation testing
- **Multi-Signature Tests**: Multi-sig approval and execution flows
- **Prerequisite Tests**: Prerequisite checking and override scenarios
- **Expiry Tests**: Certificate expiry and renewal scenarios
- **RBAC Tests**: Role-based access control validation

## Deployment

### Prerequisites
- Deploy the `shared` contract for RBAC functionality
- Ensure admin address has proper permissions
- Configure initial multi-signature settings if needed

### Deployment Steps
1. Deploy the certificate contract
2. Initialize with admin address
3. Configure multi-signature settings for courses
4. Set up prerequisite requirements
5. Grant appropriate roles to instructors and moderators
6. Begin certificate issuance

### Environment Setup
- Set admin address for contract initialization
- Configure multi-signature approvers
- Define course prerequisite structures
- Set up metadata validation rules

## Related Docs
- [Multi-Signature Certificate System](../docs/MULTISIG_CERTIFICATE_SYSTEM.md)
- [Prerequisite System](../docs/PREREQUISITE_SYSTEM.md)
- [RBAC Implementation](../docs/RBAC_IMPLEMENTATION.md)
- [Metadata Validation](../docs/METADATA_VALIDATION.md)
- [Reentrancy Protection](../docs/REENTRANCY_PROTECTION.md)