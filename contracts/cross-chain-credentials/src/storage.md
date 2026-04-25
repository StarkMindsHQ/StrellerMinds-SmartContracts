# Storage Module Documentation

## Overview

This document describes the storage architecture for the cross-chain credentials contract, including the modifications made to support the enhanced verification service with CORS support.

## Storage Keys

### Original DataKey Structure

```rust
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Credential(String),
    Proof(String),
    Oracle(Address),
    Request(String),
    StudentCreds(Address),
    ChainBridge(u32),
}
```

### Enhanced DataKey Structure

To support the enhanced verification service, a new storage key was added:

```rust
#[contracttype]
#[derive(Clone)]
pub enum DataKey {
    Admin,
    Credential(String),
    Proof(String),
    Oracle(Address),
    Request(String),
    StudentCreds(Address),
    ChainBridge(u32),
    VerificationResult(String),  // Added for enhanced verification
}
```

## Storage Patterns

### Persistent Storage
- **Credentials**: Stored by credential ID
- **Proofs**: Stored by proof ID for cross-chain verification
- **Verification Results**: Stored by verification result ID
- **Oracle Addresses**: Trusted oracle addresses for cross-chain attestation

### Instance Storage
- **Admin**: Contract administrator address
- **Rate Limiting**: Per-address rate limiting data

## Verification Result Storage

### Purpose
The `VerificationResult` storage key was added to store results from external academic verification services that include CORS handling.

### Data Structure
```rust
#[derive(Clone, Debug, soroban_sdk::contracttype)]
pub struct ExternalVerificationResult {
    pub verification_id: String,
    pub credential_id: String,
    pub status: VerificationStatus,
    pub verified_at: u64,
    pub verifier_address: Address,
    pub verification_score: u32,
    pub confidence_score: f64,
    pub external_metadata: Vec<String>,
}
```

### Storage Key Format
```rust
DataKey::VerificationResult(String)  // Format: "VER-{credential_id}-{verification_id}"
```

## Storage Functions

### Core Functions
```rust
pub fn set_admin(env: &Env, admin: &Address)
pub fn get_admin(env: &Env) -> Address
pub fn is_oracle(env: &Env, oracle: &Address) -> bool
pub fn add_oracle(env: &Env, oracle: &Address)
```

### Verification Result Functions (Planned)
```rust
pub fn store_verification_result(env: &Env, key: &str, result: &ExternalVerificationResult)
pub fn get_verification_result(env: &Env, key: &str) -> Option<ExternalVerificationResult>
pub fn has_verification_result(env: &Env, key: &str) -> bool
```

## Storage Optimization

### TTL Management
- **Verification Results**: Short TTL (24 hours) to reduce storage costs
- **Credentials**: Long TTL based on credential validity period
- **Proofs**: Medium TTL (30 days) for cross-chain verification

### Batch Operations
- Batch storage of multiple verification results
- Batch retrieval of student credentials
- Batch cleanup of expired data

## Security Considerations

### Access Control
- Admin-only functions for critical storage operations
- Oracle validation for cross-chain attestations
- Owner-only access to private credential data

### Data Privacy
- Sensitive credential data encrypted at rest
- Verification results with limited access scope
- Audit trail for all storage modifications

## Performance Optimizations

### Indexing Strategy
- Primary index by credential ID
- Secondary index by student address
- Tertiary index by verification status

### Caching Layer
- Hot credential data in instance storage
- Verification results in persistent storage with caching
- Oracle addresses in instance storage for fast access

## Migration Strategy

### Data Migration
1. Deploy new storage schema
2. Migrate existing credentials to new format
3. Add verification result storage
4. Update access patterns

### Backward Compatibility
- Maintain existing storage keys
- Add new keys without breaking changes
- Gradual migration of data access patterns

## Monitoring and Analytics

### Storage Metrics
- Storage usage by type
- Access patterns and frequency
- TTL expiration rates
- Failed storage operations

### Performance Metrics
- Storage operation latency
- Batch operation throughput
- Cache hit rates
- Storage cost optimization

## Troubleshooting

### Common Issues

#### Storage Capacity Exceeded
**Solution**: Implement TTL cleanup and data archiving

#### Access Permission Denied
**Solution**: Verify admin/oracle authentication

#### Data Corruption
**Solution**: Implement storage validation and backup

#### Performance Degradation
**Solution**: Optimize storage patterns and add caching

## Future Enhancements

### Planned Improvements
1. **Sharded Storage**: Distribute storage across multiple contracts
2. **Compression**: Data compression for large credential metadata
3. **Encryption**: Enhanced encryption for sensitive data
4. **Backup/Recovery**: Automated backup and disaster recovery
5. **Analytics**: Real-time storage analytics and reporting

### Research Areas
1. **IPFS Integration**: Decentralized storage for large files
2. **Zero-Knowledge Proofs**: Privacy-preserving verification storage
3. **Cross-Chain Storage**: Multi-blockchain storage synchronization
4. **Quantum-Resistant Encryption**: Future-proofing storage security

## Conclusion

The storage module provides a robust foundation for the cross-chain credentials contract with enhanced verification capabilities. The modular design allows for easy extension while maintaining security and performance standards.

### Key Features
- ✅ Comprehensive storage key management
- ✅ Verification result storage with CORS support
- ✅ Security and privacy protections
- ✅ Performance optimization strategies
- ✅ Monitoring and analytics capabilities
- ✅ Future-proofing for scalability

---

**Last Updated**: 2025-04-24  
**Version**: 1.0.0  
**Status**: Documentation Complete - Implementation in shared/cors_config.rs
