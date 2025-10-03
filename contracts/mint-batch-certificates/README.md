# Mint Batch Certificates Contract

## Overview
A specialized contract for efficiently minting multiple certificates in batch operations. This contract optimizes gas usage and provides robust error handling for bulk certificate issuance, making it ideal for large-scale educational credential distribution.

## Interface

### Core Functions
```rust
// Initialize the contract with admin and max batch size
fn initialize(env: &Env, admin: Address, max_batch_size: u32) -> Result<(), Error>

// Add an authorized issuer
fn add_issuer(env: &Env, admin: Address, issuer: Address) -> Result<(), Error>

// Remove an authorized issuer
fn remove_issuer(env: &Env, admin: Address, issuer: Address) -> Result<(), Error>

// Mint a single certificate
fn mint_single_certificate(env: &Env, issuer: Address, owner: Address, certificate: CertificateData) -> Result<(), Error>

// Mint multiple certificates in a batch
fn mint_batch_certificates(env: &Env, issuer: Address, owners: Vec<Address>, certificates: Vec<CertificateData>) -> Vec<MintResult>

// Mint certificates with dynamic batch optimization
fn mint_batch_certificates_dynamic(env: &Env, issuer: Address, owners: Vec<Address>, certificates: Vec<CertificateData>, target_gas_limit: u64) -> Vec<MintResult>

// Revoke a certificate
fn revoke_certificate(env: &Env, issuer: Address, certificate_id: u64) -> Result<(), Error>
```

### Utility Functions
```rust
// Get certificate data
fn get_certificate(env: &Env, certificate_id: u64) -> Option<CertificateData>

// Get certificates owned by an address
fn get_owner_certificates(env: &Env, owner: Address) -> Vec<u64>

// Check if an address is an authorized issuer
fn is_issuer(env: &Env, address: Address) -> bool

// Estimate gas usage for batch operations
fn estimate_gas_for_batch(env: &Env, issuer: Address, owners: Vec<Address>, certificates: Vec<CertificateData>, target_gas_limit: u64) -> (u64, u32)

// Split batch into optimal sub-batches
fn split_into_optimal_batches(env: &Env, owners: Vec<Address>, certificates: Vec<CertificateData>, target_gas_limit: u64) -> Vec<(Vec<Address>, Vec<CertificateData>)>
```

## Events

### Contract Events
- `contract_initialized`: Emitted when contract is initialized with admin and batch size
- `issuer_added`: Emitted when a new issuer is authorized
- `issuer_removed`: Emitted when an issuer is removed
- `certificate_minted`: Emitted when a certificate is successfully minted
- `certificate_revoked`: Emitted when a certificate is revoked
- `batch_mint_completed`: Emitted when a batch minting operation completes
- `error_event`: Emitted when errors occur during operations

## Configuration

### Contract Configuration
- `max_batch_size`: Maximum number of certificates that can be minted in a single batch
- `admin`: Contract administrator address
- `authorized_issuers`: List of addresses authorized to mint certificates

### Certificate Data Structure
```rust
pub struct CertificateData {
    pub id: u64,
    pub metadata_hash: BytesN<32>,
    pub valid_from: u64,
    pub valid_until: u64,
    pub revocable: bool,
    pub cert_type: CertificateType,
}
```

### Batch Optimization Parameters
- `target_gas_limit`: Maximum gas to use per batch operation
- `retry_attempts`: Number of retry attempts for failed operations
- `storage_error_retry`: Automatic retry for storage errors

## Testing

### Running Tests
```bash
# Run all tests for mint-batch-certificates contract
cargo test --package mint-batch-certificates

# Run specific test modules
cargo test --package mint-batch-certificates test::test_batch_minting
cargo test --package mint-batch-certificates test::test_gas_optimization
cargo test --package mint-batch-certificates test::test_error_handling
```

### Test Coverage
- **Unit Tests**: Individual function testing
- **Batch Operation Tests**: Multi-certificate minting scenarios
- **Gas Optimization Tests**: Batch size optimization validation
- **Error Handling Tests**: Comprehensive error scenario testing
- **Storage Tests**: Persistent storage validation
- **Authorization Tests**: Issuer permission validation

## Deployment

### Prerequisites
- Admin address for contract initialization
- List of authorized issuer addresses
- Maximum batch size configuration

### Deployment Steps
1. Deploy the mint-batch-certificates contract
2. Initialize with admin address and max batch size
3. Add authorized issuers
4. Configure batch optimization parameters
5. Begin batch certificate minting operations

### Environment Setup
- Set appropriate max batch size based on gas limits
- Configure authorized issuers
- Set up retry and error handling parameters
- Test batch operations with small batches first

## Usage Examples

### Single Certificate Minting
```rust
let certificate = CertificateData {
    id: 1,
    metadata_hash: BytesN::from_array(&env, &[1u8; 32]),
    valid_from: env.ledger().timestamp(),
    valid_until: env.ledger().timestamp() + 31536000, // 1 year
    revocable: true,
    cert_type: CertificateType::Standard,
};

client.mint_single_certificate(&issuer, &owner, &certificate)?;
```

### Batch Certificate Minting
```rust
let mut owners = Vec::new(&env);
let mut certificates = Vec::new(&env);

// Add multiple certificate data
for i in 0..10 {
    owners.push_back(owner_addresses.get(i).unwrap());
    certificates.push_back(certificate_data.get(i).unwrap());
}

let results = client.mint_batch_certificates(&issuer, &owners, &certificates);
```

### Dynamic Batch Optimization
```rust
let target_gas_limit = 1000000; // 1M gas units
let results = client.mint_batch_certificates_dynamic(
    &issuer, 
    &owners, 
    &certificates, 
    target_gas_limit
);
```

## Related Docs
- [Certificate Contract](./certificate/README.md)
- [Gas Optimization Analysis](../docs/gas_optimization_analysis.md)
- [Development Guide](../docs/development.md)