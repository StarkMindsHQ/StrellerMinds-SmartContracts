# Certificate Metadata Validation Implementation

## Overview

This document describes the comprehensive metadata validation system implemented for the certificate smart contract to prevent malicious data storage, XSS vulnerabilities, and ensure data integrity.

## Features Implemented

### 1. Schema Validation
- **Field Length Constraints**: Enforced minimum and maximum lengths for all text fields
- **Format Validation**: Specific format requirements for course IDs and URIs
- **Type Validation**: Ensures all fields meet their expected data types

### 2. Content Sanitization
- **XSS Prevention**: Blocks dangerous characters that could lead to cross-site scripting
- **Forbidden Characters**: Comprehensive list of blocked characters including control characters
- **Content Quality**: Prevents spam and malformed content through quality checks

### 3. Size Limits
- **Title**: 3-200 characters
- **Description**: 10-1000 characters  
- **Course ID**: 3-100 characters
- **URI**: 10-500 characters

### 4. URI Format Verification
- **Allowed Schemes**: Only `https://`, `ipfs://`, and `ar://` (Arweave) schemes
- **Format Validation**: Proper URI structure validation
- **Domain Validation**: Basic domain format checks for HTTPS URIs
- **Hash Validation**: Format validation for IPFS and Arweave content identifiers

## Implementation Details

### ValidationConfig Constants

```rust
pub struct ValidationConfig;

impl ValidationConfig {
    // Size limits (in bytes)
    pub const MAX_TITLE_LENGTH: u32 = 200;
    pub const MAX_DESCRIPTION_LENGTH: u32 = 1000;
    pub const MAX_COURSE_ID_LENGTH: u32 = 100;
    pub const MAX_URI_LENGTH: u32 = 500;
    
    // Minimum lengths
    pub const MIN_TITLE_LENGTH: u32 = 3;
    pub const MIN_DESCRIPTION_LENGTH: u32 = 10;
    pub const MIN_COURSE_ID_LENGTH: u32 = 3;
    pub const MIN_URI_LENGTH: u32 = 10;
    
    // URI validation patterns
    pub const VALID_URI_SCHEMES: &'static [&'static str] = &["https://", "ipfs://", "ar://"];
    
    // Forbidden characters for XSS prevention
    pub const FORBIDDEN_CHARS: &'static [char] = &['<', '>', '"', '\'', '&', /* control chars */];
    
    // Maximum allowed special characters ratio
    pub const MAX_SPECIAL_CHAR_RATIO: f32 = 0.3;
}
```

### Core Validation Functions

#### `validate_mint_params()`
Comprehensive validation of all certificate minting parameters:
- Certificate ID validation (non-zero bytes)
- String field validation (title, description, course ID)
- URI format and scheme validation
- Address validation
- Expiry date validation (future date, reasonable timeframe)

#### `validate_uri_update()`
Validates URI updates with the same rigor as initial minting.

#### `validate_batch_params()`
Validates batch certificate operations:
- Batch size limits (max 100 certificates)
- Individual certificate validation
- Duplicate ID detection within batch

### Security Features

#### XSS Prevention
Blocks common XSS attack vectors:
- Script tags: `<script>`, `</script>`
- Event handlers: `onload`, `onerror`, etc.
- JavaScript protocols: `javascript:`
- HTML entities and encoded attacks

#### Content Quality Validation
- **Special Character Ratio**: Limits special characters to 30% of content
- **Repetition Prevention**: Max 5 consecutive identical characters
- **Whitespace Validation**: Prevents whitespace-only content
- **Format Enforcement**: Alphanumeric + allowed separators for course IDs

#### URI Security
- **Scheme Restriction**: Only secure protocols allowed
- **Format Validation**: Proper URI structure required
- **Domain Validation**: Basic domain format checks
- **Hash Validation**: Content identifier format validation

### Error Handling

New error types added to `CertificateError`:
```rust
pub enum CertificateError {
    // ... existing errors ...
    
    // Enhanced metadata errors
    InvalidMetadata = 12,
    InvalidUri = 13,
    MetadataTooLarge = 14,
    MetadataTooSmall = 15,
    InvalidCharacters = 16,
    InvalidFormat = 17,
}
```

## Usage Examples

### Valid Certificate Minting
```rust
let params = MintCertificateParams {
    certificate_id: BytesN::from_array(&env, &[1u8; 32]),
    course_id: String::from_str(&env, "CS-101"),
    student: student_address,
    title: String::from_str(&env, "Introduction to Computer Science"),
    description: String::from_str(&env, "This certificate validates completion of CS-101 covering fundamental programming concepts."),
    metadata_uri: String::from_str(&env, "https://university.edu/certificates/metadata/cs101.json"),
    expiry_date: env.ledger().timestamp() + 31536000, // 1 year
};

// This will pass all validation checks
client.mint_certificate(&issuer, &params);
```

### Batch Certificate Minting
```rust
let batch_params = vec![params1, params2, params3];
// Validates all certificates and checks for duplicates
client.mint_certificates_batch(&issuer, &batch_params);
```

## Testing

### Unit Tests
- Individual validation function tests
- Edge case testing
- Boundary condition validation
- XSS attack vector testing

### Integration Tests
- End-to-end certificate minting with validation
- Batch operation testing
- URI update validation
- Error condition testing

### Test Coverage
- ✅ Valid input acceptance
- ✅ Invalid input rejection
- ✅ XSS prevention
- ✅ Size limit enforcement
- ✅ URI scheme validation
- ✅ Content quality checks
- ✅ Batch validation
- ✅ Edge cases and boundary conditions

## Security Considerations

### Prevented Attacks
1. **Cross-Site Scripting (XSS)**: Blocked malicious script injection
2. **Content Injection**: Prevented HTML/JavaScript injection
3. **Spam/Malformed Content**: Quality checks prevent low-quality data
4. **Protocol Attacks**: Only secure URI schemes allowed
5. **Buffer Overflow**: Size limits prevent excessive data storage

### Data Integrity
- All metadata is validated before storage
- Consistent format enforcement
- Proper encoding validation
- Sanitization without data loss for valid content

## Performance Impact

### Validation Overhead
- Minimal computational overhead per validation
- O(n) complexity for string validation
- Batch validation optimized for multiple certificates
- Early termination on first validation failure

### Storage Efficiency
- Size limits prevent blockchain bloat
- Efficient validation reduces failed transactions
- Proper error handling minimizes gas waste

## Future Enhancements

### Potential Improvements
1. **Configurable Limits**: Make validation limits configurable per deployment
2. **Advanced URI Validation**: More sophisticated URI format checking
3. **Content Encoding**: Support for additional character encodings
4. **Metadata Schema**: JSON schema validation for metadata URIs
5. **Rate Limiting**: Prevent spam through rate limiting mechanisms

### Monitoring
- Validation failure metrics
- Common attack pattern detection
- Performance monitoring for validation functions

## Compliance

### Standards Adherence
- Follows Soroban smart contract best practices
- Implements OWASP security guidelines
- Adheres to blockchain data integrity principles
- Compatible with existing certificate standards

### Audit Considerations
- All validation logic is transparent and auditable
- Comprehensive test coverage for security review
- Clear error messages for debugging
- Documented security assumptions and limitations
