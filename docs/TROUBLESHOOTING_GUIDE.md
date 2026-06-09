# Troubleshooting Guide: StrellerMinds Smart Contracts

This guide provides solutions to common issues encountered when interacting with the StrellerMinds smart contracts on the Soroban platform.

## 1. Transaction Errors

### 1.1 `HostError` (Error Code: 1)
*   **Cause**: The Soroban host environment encountered a fatal error, often due to invalid resource limits or malformed transaction data.
*   **Solution**: 
    *   Verify that your transaction has sufficient resource limits (CPU, RAM, Ledger Read/Write).
    *   Ensure the contract ID and function name are correct.
    *   Check that the arguments passed to the function match the expected types in the contract.

### 1.2 `ContractError` (Custom Error Codes)
*   **Cause**: The contract logic explicitly rejected the transaction (e.g., unauthorized access, invalid input).
*   **Solution**: 
    *   Refer to the `errors.rs` file in the specific contract's source code to map the numeric error code to a human-readable reason.
    *   Common codes include `Unauthorized` (1), `NotInitialized` (2), and `InvalidInput` (3).

### 1.3 `AuthError` (Authorization Failed)
*   **Cause**: One or more required signatures are missing or the caller is not authorized for the operation.
*   **Solution**: 
    *   Ensure that `require_auth()` is being called on the correct address.
    *   If using a multi-sig setup, ensure all required approvers have signed the transaction.
    *   Check that the contract admin has granted the necessary roles via the RBAC system.

## 2. Certificate Issuance Issues

### 2.1 Multi-Sig Request Pending
*   **Issue**: A certificate request has been created but the certificate is not issued.
*   **Diagnostic**: Call `get_multisig_request(request_id)` to check the `current_approvals` vs. `required_approvals`.
*   **Solution**: Remind the authorized approvers to call `process_multisig_approval`.

### 2.2 Template Validation Failed
*   **Issue**: `issue_with_template` returns `MissingRequiredField`.
*   **Solution**: Call `get_template(template_id)` to inspect the required fields and ensure all are provided in the `field_values` vector.

## 3. Compliance & Auditing

### 3.1 Compliance Check "Fail"
*   **Issue**: A certificate is marked as non-compliant.
*   **Diagnostic**: Inspect the certificate's status and expiry date.
*   **Solution**: Ensure the certificate is `Active` and the `expiry_date` is in the future. Check if the certificate has a valid `blockchain_anchor`.

## 4. Performance & Gas

### 4.1 "Gas Limit Exceeded"
*   **Issue**: The transaction exceeds the Soroban gas limit.
*   **Solution**: 
    *   Break large batch operations into smaller chunks (e.g., reduces `batch_issue_certificates` size).
    *   Use the `gas_optimized` versions of functions if available.
    *   Perform complex string template substitution **off-chain** and only pass the finalized result to the contract.

## 5. Data Privacy & Compliance (GDPR/ISO 9001)

### 5.1 On-Chain PII
*   **Warning**: Do NOT store Personal Identifiable Information (PII) like names, emails, or phone numbers in `notes`, `description`, or `metadata_uri` fields.
*   **Best Practice**: Store sensitive user data in an off-chain encrypted database and only store a cryptographic hash (SHA-256) on the blockchain for verification.

### 5.2 Right to Erasure
*   **Issue**: Blockchain records are immutable.
*   **Solution**: Since records cannot be deleted, ensure that any link to PII can be broken by deleting the off-chain decryption keys.

## 6. State Archival & Storage Rent (Soroban)

### 6.1 Certificate or Template "Missing"
*   **Issue**: A previously issued certificate or template is no longer found in storage.
*   **Cause**: On Soroban, `persistent` storage entries have a Time-To-Live (TTL). If the TTL expires and the rent is not paid, the entry is **archived**.
*   **Solution**: 
    *   The contract automatically extends the TTL on every access.
    *   If an entry is archived, it must be **restored** via a special Soroban transaction before it can be used again.

## 7. Support & FAQ

### FAQ
*   **Q: Can I recover a revoked certificate?**
    *   A: No, revocation is permanent. However, if marked "reissuance eligible," a new version can be issued.
*   **Q: How do I change the contract admin?**
    *   A: The current admin must call `change_admin` in the Shared contract.

### Support
For further assistance, please contact the StrellerMinds engineering team at `support@strellerminds.io` or join our Discord developer channel.
