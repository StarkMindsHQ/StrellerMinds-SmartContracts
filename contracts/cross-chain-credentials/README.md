# Cross-Chain Credentials Contract

## Purpose

The Cross-Chain Credentials contract enables StrellerMinds learning achievements to be recognized across multiple blockchains. It issues, manages, and verifies credentials on Stellar while generating cryptographic proofs that external chains (Ethereum, Polygon, BSC) can consume. An oracle system bridges the trust gap between chains, and a full transcript API lets any party generate a verifiable record of all credentials earned by a student.

## Architecture

| Module | Description |
|---|---|
| `src/lib.rs` | Contract entrypoint — full credential lifecycle (issue, revoke, suspend, reactivate, verify) plus oracle management, transcript generation, and verification request handling |
| `src/errors.rs` | `CrossChainError` enum covering initialization state, authorization, credential look-up, and status-based business logic |
| `src/types.rs` | Core data types: `Credential`, `CredentialStatus`, `ChainId`, `CrossChainProof`, `OracleAttestation`, `VerificationRequest`, `Transcript` |
| `src/storage.rs` | `DataKey` enum and storage accessor helpers (`get_admin`, `is_oracle`, `add_oracle`, `set_admin`) |

## Public API

| Function | Description | Auth Required |
|---|---|---|
| `initialize(admin)` | One-time setup; stores the admin address | No (open, call once) |
| `issue_credential(student, achievement, metadata_hash, chain_id)` | Issues a new `Active` credential to `student`; returns the credential ID | Yes — admin |
| `revoke_credential(credential_id)` | Permanently marks a credential as `Revoked` | Yes — admin |
| `suspend_credential(credential_id)` | Temporarily marks a credential as `Suspended` | Yes — admin |
| `reactivate_credential(credential_id)` | Restores a `Suspended` credential to `Active` | Yes — admin |
| `get_credential(credential_id)` | Returns the full `Credential` struct | No |
| `verify_cross_chain(credential_id, target_chain)` | Generates and stores a `CrossChainProof` for an active credential | No |
| `get_proof(credential_id)` | Returns the stored `CrossChainProof` for a credential | No |
| `request_verification(credential_id, chain_id, requester)` | Submits a `VerificationRequest`; returns the request ID | No |
| `get_verification_request(request_id)` | Returns the `VerificationRequest` record | No |
| `generate_transcript(student)` | Builds a `Transcript` from all credentials issued to `student` | No |
| `get_student_credentials(student)` | Returns all credential IDs issued to `student` | No |
| `add_oracle(oracle)` | Registers a trusted oracle address | Yes — admin |
| `remove_oracle(oracle)` | Removes an oracle from the trusted list | Yes — admin |
| `is_oracle(oracle)` | Returns `true` if `oracle` is a registered trusted oracle | No |

## Usage Example

```text
# Initialize
cross_chain.initialize(admin_address)

# Admin issues a credential to a student for completing a Rust course
cred_id = cross_chain.issue_credential(
    student_address,
    "Rust Fundamentals — Certificate of Completion",
    metadata_hash,
    ChainId::Ethereum
)

# Anyone requests cross-chain verification targeting Polygon
proof = cross_chain.verify_cross_chain(cred_id, ChainId::Polygon)

# Retrieve the generated proof later
proof = cross_chain.get_proof(cred_id)

# Generate the student's full academic transcript
transcript = cross_chain.generate_transcript(student_address)

# Admin temporarily suspends a credential pending review
cross_chain.suspend_credential(cred_id)

# After review, reactivate the credential
cross_chain.reactivate_credential(cred_id)
```

## Errors

For the full error code reference and conventions, see [ERROR_HANDLING.md](../../docs/ERROR_HANDLING.md).

| Code | Variant | Meaning |
|---|---|---|
| 1 | `AlreadyInitialized` | `initialize` has already been called |
| 2 | `NotInitialized` | Contract has not been initialized yet |
| 10 | `Unauthorized` | Caller is not the admin |
| 50 | `CredentialNotFound` | No credential exists with the supplied ID |
| 51 | `ProofNotFound` | No cross-chain proof has been generated for this credential |
| 52 | `VerificationRequestNotFound` | No verification request found for the supplied request ID |
| 80 | `CredentialNotActive` | Credential must be in `Active` status to perform this operation |
| 81 | `CredentialRevoked` | Credential has been permanently revoked |
| 82 | `CredentialSuspended` | Credential is temporarily suspended |

## Integration

| Contract | Relationship |
|---|---|
| `certificate` | Stellar-native certificate issuance; cross-chain-credentials extends portability to other chains |
| `analytics` | Consumes `CredentialIssued` and `ProofGenerated` events for credentialing metrics |
| `shared` | Uses RBAC helpers, event schema macros (`emit_crosschain_event!`, `emit_access_control_event!`) |
