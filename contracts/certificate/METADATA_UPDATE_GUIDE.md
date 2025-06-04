# Certificate Metadata Updates

## Overview
Update certificate metadata URIs after minting with full audit trail.

## Key Features
- **Access Control**: Only issuer or admin can update
- **History Tracking**: Immutable record of all changes
- **Event Emissions**: Updates trigger events
- **Validation**: Non-empty URI required

## API

### `update_certificate_uri`
```rust
fn update_certificate_uri(
    env: Env,
    updater: Address,
    certificate_id: BytesN<32>,
    new_uri: String,
) -> Result<(), CertificateError>
```

**Authorization**: Issuer or admin only
**Validation**: URI cannot be empty

### `get_metadata_history`
```rust
fn get_metadata_history(env: Env, certificate_id: BytesN<32>) -> Vec<MetadataUpdateEntry>
```

Returns complete update history with timestamps.

## Usage
```rust
// Update metadata
certificate_contract.update_certificate_uri(&env, &issuer, &cert_id, &new_uri)?;

// Get history
let history = certificate_contract.get_metadata_history(&env, &cert_id);
```

