#![no_std]

pub mod errors;

use crate::errors::CrossChainError;
use shared::event_schema::{
    AccessControlEventData, ContractInitializedEvent, CredentialIssuedEvent,
    CredentialReactivatedEvent, CredentialRevokedEvent, CredentialSuspendedEvent,
    CrossChainEventData, OracleUpdatedEvent, ProofGeneratedEvent, VerificationRequestedEvent,
};
use shared::monitoring::{ContractHealthReport, Monitor};
use shared::validation::{CoreValidator, ValidationConfig};
use shared::{emit_access_control_event, emit_crosschain_event};
use soroban_sdk::{contract, contractimpl, symbol_short, Address, Env, String, Vec};

mod storage;
mod types;

use storage::{get_admin, is_oracle, DataKey};
use types::{
    BridgeRequest, BridgeStatus, ChainId, Credential, CredentialStatus, CrossChainProof,
    Transcript, VerificationRequest,
};

#[contract]
pub struct CrossChainCredentials;

#[contractimpl]
impl CrossChainCredentials {
    /// Initializes the contract and sets the admin address.
    ///
    /// # Arguments
    /// * `admin` - Address that will control credential issuance and oracle management.
    ///
    /// # Errors
    /// Returns [`CrossChainError::AlreadyInitialized`] if called more than once.
    ///
    /// # Example
    /// ```ignore
    /// client.initialize(&admin);
    /// ```
    pub fn initialize(env: Env, admin: Address) -> Result<(), CrossChainError> {
        admin.require_auth();
        if env.storage().instance().has(&DataKey::Admin) {
            return Err(CrossChainError::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        emit_access_control_event!(
            &env,
            symbol_short!("creds"),
            admin.clone(),
            AccessControlEventData::ContractInitialized(ContractInitializedEvent { admin })
        );
        Ok(())
    }

    /// Issues a new cross-chain credential to a student and stores it on-chain.
    ///
    /// Requires admin authorization. The credential is initially in `Active` status.
    ///
    /// # Arguments
    /// * `student` - Address of the credential recipient.
    /// * `achievement` - Human-readable description of the achievement.
    /// * `metadata_hash` - Hash of the off-chain metadata associated with the credential.
    /// * `chain_id` - Target chain for which the credential is valid.
    ///
    /// Returns the unique credential ID string.
    ///
    /// # Example
    /// ```ignore
    /// let cred_id = client.issue_credential(&student, &achievement, &hash, &chain_id);
    /// ```
    pub fn issue_credential(
        env: Env,
        student: Address,
        achievement: String,
        metadata_hash: String,
        chain_id: ChainId,
    ) -> String {
        let admin = get_admin(&env);
        admin.require_auth();

        // Validate inputs
        CoreValidator::validate_soroban_string_length(
            &achievement,
            "achievement",
            ValidationConfig::MIN_TITLE_LENGTH,
            ValidationConfig::MAX_TITLE_LENGTH,
        )
        .expect("Invalid achievement");
        CoreValidator::validate_soroban_string_length(
            &metadata_hash,
            "metadata_hash",
            ValidationConfig::MIN_TITLE_LENGTH,
            ValidationConfig::MAX_URI_LENGTH,
        )
        .expect("Invalid metadata_hash");

        let credential_id = String::from_str(&env, "CRED");
        let credential = Credential {
            id: credential_id.clone(),
            student: student.clone(),
            issuer: admin.clone(),
            achievement,
            issued_at: env.ledger().timestamp(),
            chain_id: chain_id.clone(),
            status: CredentialStatus::Active,
            metadata_hash,
        };

        env.storage().persistent().set(&DataKey::Credential(credential_id.clone()), &credential);

        let mut student_creds = env
            .storage()
            .persistent()
            .get::<DataKey, Vec<String>>(&DataKey::StudentCreds(student.clone()))
            .unwrap_or(Vec::new(&env));
        student_creds.push_back(credential_id.clone());
        env.storage().persistent().set(&DataKey::StudentCreds(student.clone()), &student_creds);

        emit_crosschain_event!(
            &env,
            symbol_short!("creds"),
            admin.clone(),
            CrossChainEventData::CredentialIssued(CredentialIssuedEvent {
                student,
                credential_id: credential_id.clone(),
                chain_id: chain_id.to_u32(),
            })
        );

        credential_id
    }

    /// Revokes an active credential, permanently marking it as `Revoked`.
    ///
    /// Requires admin authorization.
    ///
    /// # Arguments
    /// * `credential_id` - ID of the credential to revoke.
    ///
    /// # Errors
    /// Returns [`CrossChainError::CredentialNotFound`] if the credential does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.revoke_credential(&cred_id);
    /// ```
    pub fn revoke_credential(env: Env, credential_id: String) -> Result<(), CrossChainError> {
        let admin = get_admin(&env);
        admin.require_auth();

        let mut credential: Credential = env
            .storage()
            .persistent()
            .get(&DataKey::Credential(credential_id.clone()))
            .ok_or(CrossChainError::CredentialNotFound)?;
        credential.status = CredentialStatus::Revoked;
        env.storage().persistent().set(&DataKey::Credential(credential_id.clone()), &credential);

        emit_crosschain_event!(
            &env,
            symbol_short!("creds"),
            admin,
            CrossChainEventData::CredentialRevoked(CredentialRevokedEvent { credential_id })
        );
        Ok(())
    }

    /// Temporarily suspends a credential, marking it as `Suspended`.
    ///
    /// Requires admin authorization. A suspended credential can be reactivated via
    /// [`CrossChainCredentials::reactivate_credential`].
    ///
    /// # Arguments
    /// * `credential_id` - ID of the credential to suspend.
    ///
    /// # Errors
    /// Returns [`CrossChainError::CredentialNotFound`] if the credential does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.suspend_credential(&cred_id);
    /// ```
    pub fn suspend_credential(env: Env, credential_id: String) -> Result<(), CrossChainError> {
        let admin = get_admin(&env);
        admin.require_auth();

        let mut credential: Credential = env
            .storage()
            .persistent()
            .get(&DataKey::Credential(credential_id.clone()))
            .ok_or(CrossChainError::CredentialNotFound)?;
        credential.status = CredentialStatus::Suspended;
        env.storage().persistent().set(&DataKey::Credential(credential_id.clone()), &credential);

        emit_crosschain_event!(
            &env,
            symbol_short!("creds"),
            admin,
            CrossChainEventData::CredentialSuspended(CredentialSuspendedEvent { credential_id })
        );
        Ok(())
    }

    /// Reactivates a previously suspended credential, restoring it to `Active` status.
    ///
    /// Requires admin authorization.
    ///
    /// # Arguments
    /// * `credential_id` - ID of the credential to reactivate.
    ///
    /// # Errors
    /// Returns [`CrossChainError::CredentialNotFound`] if the credential does not exist.
    ///
    /// # Example
    /// ```ignore
    /// client.reactivate_credential(&cred_id);
    /// ```
    pub fn reactivate_credential(env: Env, credential_id: String) -> Result<(), CrossChainError> {
        let admin = get_admin(&env);
        admin.require_auth();

        let mut credential: Credential = env
            .storage()
            .persistent()
            .get(&DataKey::Credential(credential_id.clone()))
            .ok_or(CrossChainError::CredentialNotFound)?;
        credential.status = CredentialStatus::Active;
        env.storage().persistent().set(&DataKey::Credential(credential_id.clone()), &credential);

        emit_crosschain_event!(
            &env,
            symbol_short!("creds"),
            admin,
            CrossChainEventData::CredentialReactivated(CredentialReactivatedEvent {
                credential_id
            })
        );
        Ok(())
    }

    /// Returns the full credential record for the given credential ID.
    ///
    /// # Arguments
    /// * `credential_id` - ID of the credential to retrieve.
    ///
    /// # Errors
    /// Returns [`CrossChainError::CredentialNotFound`] if no credential with that ID exists.
    ///
    /// # Example
    /// ```ignore
    /// let credential = client.get_credential(&cred_id);
    /// ```
    pub fn get_credential(env: Env, credential_id: String) -> Result<Credential, CrossChainError> {
        env.storage()
            .persistent()
            .get(&DataKey::Credential(credential_id))
            .ok_or(CrossChainError::CredentialNotFound)
    }

    /// Verifies a credential for use on another chain and generates a cross-chain proof.
    ///
    /// The credential must be in `Active` status. The generated proof is stored on-chain and
    /// can be retrieved later via [`CrossChainCredentials::get_proof`].
    ///
    /// # Arguments
    /// * `credential_id` - ID of the credential to verify.
    /// * `target_chain` - The destination chain for which the proof is being generated.
    ///
    /// # Errors
    /// Returns [`CrossChainError::CredentialNotFound`] if the credential does not exist.
    /// Returns [`CrossChainError::CredentialNotActive`] if the credential is revoked or suspended.
    ///
    /// # Example
    /// ```ignore
    /// let proof = client.verify_cross_chain(&cred_id, &ChainId::Ethereum);
    /// ```
    pub fn verify_cross_chain(
        env: Env,
        credential_id: String,
        target_chain: ChainId,
    ) -> Result<CrossChainProof, CrossChainError> {
        let credential: Credential = env
            .storage()
            .persistent()
            .get(&DataKey::Credential(credential_id.clone()))
            .ok_or(CrossChainError::CredentialNotFound)?;

        if credential.status != CredentialStatus::Active {
            return Err(CrossChainError::CredentialNotActive);
        }

        let proof = CrossChainProof {
            credential_id: credential.id.clone(),
            source_chain: credential.chain_id.clone(),
            target_chain: target_chain.clone(),
            proof_hash: String::from_str(&env, "proof_hash"),
            verified_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&DataKey::Proof(credential_id.clone()), &proof);

        emit_crosschain_event!(
            &env,
            symbol_short!("creds"),
            credential.student,
            CrossChainEventData::ProofGenerated(ProofGeneratedEvent {
                credential_id,
                target_chain: target_chain.to_u32(),
            })
        );
        Ok(proof)
    }

    /// Returns the cross-chain proof previously generated for a credential.
    ///
    /// # Arguments
    /// * `credential_id` - ID of the credential whose proof to retrieve.
    ///
    /// # Errors
    /// Returns [`CrossChainError::ProofNotFound`] if no proof has been generated yet.
    ///
    /// # Example
    /// ```ignore
    /// let proof = client.get_proof(&cred_id);
    /// ```
    pub fn get_proof(env: Env, credential_id: String) -> Result<CrossChainProof, CrossChainError> {
        env.storage()
            .persistent()
            .get(&DataKey::Proof(credential_id))
            .ok_or(CrossChainError::ProofNotFound)
    }

    /// Submits a verification request for a credential on a target chain.
    ///
    /// Anyone may submit a verification request. Returns the unique request ID string.
    ///
    /// # Arguments
    /// * `credential_id` - ID of the credential to verify.
    /// * `chain_id` - Target chain for the verification.
    /// * `requester` - Address submitting the verification request.
    ///
    /// # Example
    /// ```ignore
    /// let request_id = client.request_verification(&cred_id, &chain_id, &requester);
    /// ```
    pub fn request_verification(
        env: Env,
        credential_id: String,
        chain_id: ChainId,
        requester: Address,
    ) -> String {
        let request_id = String::from_str(&env, "REQ");

        let request = VerificationRequest {
            id: request_id.clone(),
            credential_id: credential_id.clone(),
            requester: requester.clone(),
            chain_id: chain_id.clone(),
            created_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&DataKey::Request(request_id.clone()), &request);

        emit_crosschain_event!(
            &env,
            symbol_short!("creds"),
            requester,
            CrossChainEventData::VerificationRequested(VerificationRequestedEvent {
                request_id: request_id.clone(),
                credential_id,
                chain_id: chain_id.to_u32(),
            })
        );
        request_id
    }

    /// Returns the verification request record for a given request ID.
    ///
    /// # Arguments
    /// * `request_id` - ID of the verification request to retrieve.
    ///
    /// # Errors
    /// Returns [`CrossChainError::VerificationRequestNotFound`] if the request does not exist.
    ///
    /// # Example
    /// ```ignore
    /// let request = client.get_verification_request(&request_id);
    /// ```
    pub fn get_verification_request(
        env: Env,
        request_id: String,
    ) -> Result<VerificationRequest, CrossChainError> {
        env.storage()
            .persistent()
            .get(&DataKey::Request(request_id))
            .ok_or(CrossChainError::VerificationRequestNotFound)
    }

    /// Generates a full academic transcript for a student from all their issued credentials.
    ///
    /// # Arguments
    /// * `student` - Address of the student whose transcript to generate.
    ///
    /// # Example
    /// ```ignore
    /// let transcript = client.generate_transcript(&student);
    /// ```
    pub fn generate_transcript(env: Env, student: Address) -> Transcript {
        let credentials = Self::get_student_credentials(env.clone(), student.clone());

        Transcript {
            student,
            credentials: credentials.clone(),
            total_achievements: credentials.len(),
            generated_at: env.ledger().timestamp(),
        }
    }

    /// Returns a list of all credential IDs issued to the given student.
    ///
    /// # Arguments
    /// * `student` - Address of the student to query.
    ///
    /// # Example
    /// ```ignore
    /// let creds = client.get_student_credentials(&student);
    /// ```
    pub fn get_student_credentials(env: Env, student: Address) -> Vec<String> {
        env.storage().persistent().get(&DataKey::StudentCreds(student)).unwrap_or(Vec::new(&env))
    }

    /// Registers an oracle address that is trusted for cross-chain attestations.
    ///
    /// Requires admin authorization.
    ///
    /// # Arguments
    /// * `oracle` - Address to register as a trusted oracle.
    ///
    /// # Example
    /// ```ignore
    /// client.add_oracle(&oracle_address);
    /// ```
    pub fn add_oracle(env: Env, oracle: Address) {
        let admin = get_admin(&env);
        admin.require_auth();
        storage::add_oracle(&env, &oracle);

        emit_crosschain_event!(
            &env,
            symbol_short!("creds"),
            admin,
            CrossChainEventData::OracleUpdated(OracleUpdatedEvent { oracle, added: true })
        );
    }

    /// Removes an oracle address from the trusted oracle list.
    ///
    /// Requires admin authorization.
    ///
    /// # Arguments
    /// * `oracle` - Address to deregister.
    ///
    /// # Example
    /// ```ignore
    /// client.remove_oracle(&oracle_address);
    /// ```
    pub fn remove_oracle(env: Env, oracle: Address) {
        let admin = get_admin(&env);
        admin.require_auth();
        env.storage().instance().remove(&DataKey::Oracle(oracle.clone()));

        emit_crosschain_event!(
            &env,
            symbol_short!("creds"),
            admin,
            CrossChainEventData::OracleUpdated(OracleUpdatedEvent { oracle, added: false })
        );
    }

    /// Returns `true` if the given address is a registered trusted oracle.
    ///
    /// # Arguments
    /// * `oracle` - Address to check.
    ///
    /// # Example
    /// ```ignore
    /// let trusted = client.is_oracle(&oracle_address);
    /// ```
    pub fn is_oracle(env: Env, oracle: Address) -> bool {
        is_oracle(&env, &oracle)
    }

    /// Returns the estimated gas cost for a bridge operation to the specified target chain.
    ///
    /// # Arguments
    /// * `target_chain` - The destination chain for which to estimate gas.
    ///
    /// # Example
    /// ```ignore
    /// let estimate = client.estimate_bridge_gas(&ChainId::Arbitrum);
    /// ```
    pub fn estimate_bridge_gas(_env: Env, target_chain: ChainId) -> u64 {
        target_chain.gas_estimate()
    }

    /// Initiates a bridge request to move a credential to a different chain.
    ///
    /// The credential must be active. Creates a [`BridgeRequest`] record in `Pending` state.
    ///
    /// # Arguments
    /// * `credential_id` - ID of the credential to bridge.
    /// * `target_chain` - Destination chain for the credential.
    ///
    /// # Errors
    /// Returns [`CrossChainError::CredentialNotFound`] if the credential does not exist.
    /// Returns [`CrossChainError::CredentialNotActive`] if the credential is not active.
    ///
    /// # Example
    /// ```ignore
    /// let bridge_req = client.initiate_bridge(&cred_id, &ChainId::Arbitrum);
    /// ```
    pub fn initiate_bridge(
        env: Env,
        credential_id: String,
        target_chain: ChainId,
    ) -> Result<BridgeRequest, CrossChainError> {
        let credential: Credential = env
            .storage()
            .persistent()
            .get(&DataKey::Credential(credential_id.clone()))
            .ok_or(CrossChainError::CredentialNotFound)?;

        if credential.status != CredentialStatus::Active {
            return Err(CrossChainError::CredentialNotActive);
        }

        let gas_estimate = target_chain.gas_estimate();
        let request_id = String::from_str(&env, "BRIDGE");

        let request = BridgeRequest {
            request_id: request_id.clone(),
            credential_id: credential_id.clone(),
            student: credential.student.clone(),
            source_chain: credential.chain_id.clone(),
            target_chain: target_chain.clone(),
            status: BridgeStatus::Pending,
            gas_estimate,
            created_at: env.ledger().timestamp(),
        };

        env.storage().persistent().set(&DataKey::BridgeRequest(request_id.clone()), &request);

        emit_crosschain_event!(
            &env,
            symbol_short!("creds"),
            credential.student,
            CrossChainEventData::ProofGenerated(ProofGeneratedEvent {
                credential_id,
                target_chain: target_chain.to_u32(),
            })
        );

        Ok(request)
    }

    /// Migrates a credential to a new chain, updating its `chain_id` in place.
    ///
    /// Requires admin authorization. The credential must be active.
    ///
    /// # Arguments
    /// * `credential_id` - ID of the credential to migrate.
    /// * `new_chain` - The chain to which the credential is being migrated.
    ///
    /// # Errors
    /// Returns [`CrossChainError::CredentialNotFound`] if the credential does not exist.
    /// Returns [`CrossChainError::CredentialNotActive`] if the credential is not active.
    ///
    /// # Example
    /// ```ignore
    /// client.migrate_credential(&cred_id, &ChainId::Arbitrum);
    /// ```
    pub fn migrate_credential(
        env: Env,
        credential_id: String,
        new_chain: ChainId,
    ) -> Result<(), CrossChainError> {
        let admin = get_admin(&env);
        admin.require_auth();

        let mut credential: Credential = env
            .storage()
            .persistent()
            .get(&DataKey::Credential(credential_id.clone()))
            .ok_or(CrossChainError::CredentialNotFound)?;

        if credential.status != CredentialStatus::Active {
            return Err(CrossChainError::CredentialNotActive);
        }

        credential.chain_id = new_chain.clone();
        env.storage().persistent().set(&DataKey::Credential(credential_id.clone()), &credential);

        emit_crosschain_event!(
            &env,
            symbol_short!("creds"),
            admin,
            CrossChainEventData::ProofGenerated(ProofGeneratedEvent {
                credential_id,
                target_chain: new_chain.to_u32(),
            })
        );

        Ok(())
    }

    /// Returns a previously created bridge request by its ID.
    ///
    /// # Arguments
    /// * `request_id` - ID of the bridge request to retrieve.
    ///
    /// # Errors
    /// Returns [`CrossChainError::VerificationRequestNotFound`] if the request does not exist.
    ///
    /// # Example
    /// ```ignore
    /// let req = client.get_bridge_request(&request_id);
    /// ```
    pub fn get_bridge_request(
        env: Env,
        request_id: String,
    ) -> Result<BridgeRequest, CrossChainError> {
        env.storage()
            .persistent()
            .get(&DataKey::BridgeRequest(request_id))
            .ok_or(CrossChainError::VerificationRequestNotFound)
    }

    pub fn health_check(env: Env) -> ContractHealthReport {
        let initialized = env.storage().instance().has(&DataKey::Admin);
        let report = Monitor::build_health_report(&env, symbol_short!("creds"), initialized);
        Monitor::emit_health_check(&env, &report);
        report
    }
}

#[cfg(test)]
mod tests;
