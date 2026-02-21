#![no_std]
use soroban_sdk::{contract, contractimpl, Address, Env, String, Vec};

mod types;
mod storage;

use types::{ChainId, Credential, CredentialStatus, CrossChainProof, Transcript, VerificationRequest};
use storage::{DataKey, get_admin, is_oracle};

#[contract]
pub struct CrossChainCredentials;

#[contractimpl]
impl CrossChainCredentials {
    pub fn initialize(env: Env, admin: Address) {
        if env.storage().instance().has(&DataKey::Admin) {
            panic!("Already initialized");
        }
        storage::set_admin(&env, &admin);
    }

    pub fn issue_credential(env: Env, student: Address, achievement: String, metadata_hash: String, chain_id: ChainId) -> String {
        let admin = get_admin(&env);
        admin.require_auth();
        
        let credential_id = String::from_str(&env, "CRED");
        let credential = Credential {
            id: credential_id.clone(),
            student: student.clone(),
            issuer: admin,
            achievement,
            issued_at: env.ledger().timestamp(),
            chain_id,
            status: CredentialStatus::Active,
            metadata_hash,
        };
        
        env.storage().persistent().set(&DataKey::Credential(credential_id.clone()), &credential);
        
        let mut student_creds = env.storage().persistent()
            .get::<DataKey, Vec<String>>(&DataKey::StudentCreds(student.clone()))
            .unwrap_or(Vec::new(&env));
        student_creds.push_back(credential_id.clone());
        env.storage().persistent().set(&DataKey::StudentCreds(student), &student_creds);
        
        credential_id
    }

    pub fn revoke_credential(env: Env, credential_id: String) {
        let admin = get_admin(&env);
        admin.require_auth();
        
        let mut credential: Credential = env.storage().persistent()
            .get(&DataKey::Credential(credential_id.clone()))
            .unwrap();
        credential.status = CredentialStatus::Revoked;
        env.storage().persistent().set(&DataKey::Credential(credential_id), &credential);
    }

    pub fn suspend_credential(env: Env, credential_id: String) {
        let admin = get_admin(&env);
        admin.require_auth();
        
        let mut credential: Credential = env.storage().persistent()
            .get(&DataKey::Credential(credential_id.clone()))
            .unwrap();
        credential.status = CredentialStatus::Suspended;
        env.storage().persistent().set(&DataKey::Credential(credential_id), &credential);
    }

    pub fn reactivate_credential(env: Env, credential_id: String) {
        let admin = get_admin(&env);
        admin.require_auth();
        
        let mut credential: Credential = env.storage().persistent()
            .get(&DataKey::Credential(credential_id.clone()))
            .unwrap();
        credential.status = CredentialStatus::Active;
        env.storage().persistent().set(&DataKey::Credential(credential_id), &credential);
    }

    pub fn get_credential(env: Env, credential_id: String) -> Credential {
        env.storage().persistent()
            .get(&DataKey::Credential(credential_id))
            .unwrap()
    }

    pub fn verify_cross_chain(env: Env, credential_id: String, target_chain: ChainId) -> CrossChainProof {
        let credential: Credential = env.storage().persistent()
            .get(&DataKey::Credential(credential_id.clone()))
            .unwrap();
        
        if credential.status != CredentialStatus::Active {
            panic!("Credential not active");
        }
        
        let proof = CrossChainProof {
            credential_id: credential.id.clone(),
            source_chain: credential.chain_id.clone(),
            target_chain,
            proof_hash: String::from_str(&env, "proof_hash"),
            verified_at: env.ledger().timestamp(),
        };
        
        env.storage().persistent().set(&DataKey::Proof(credential_id), &proof);
        proof
    }

    pub fn get_proof(env: Env, credential_id: String) -> CrossChainProof {
        env.storage().persistent()
            .get(&DataKey::Proof(credential_id))
            .unwrap()
    }

    pub fn request_verification(env: Env, credential_id: String, chain_id: ChainId, requester: Address) -> String {
        let request_id = String::from_str(&env, "REQ");
        
        let request = VerificationRequest {
            id: request_id.clone(),
            credential_id,
            requester,
            chain_id,
            created_at: env.ledger().timestamp(),
        };
        
        env.storage().persistent().set(&DataKey::Request(request_id.clone()), &request);
        request_id
    }

    pub fn get_verification_request(env: Env, request_id: String) -> VerificationRequest {
        env.storage().persistent()
            .get(&DataKey::Request(request_id))
            .unwrap()
    }

    pub fn generate_transcript(env: Env, student: Address) -> Transcript {
        let credentials = Self::get_student_credentials(env.clone(), student.clone());
        
        Transcript {
            student,
            credentials: credentials.clone(),
            total_achievements: credentials.len(),
            generated_at: env.ledger().timestamp(),
        }
    }

    pub fn get_student_credentials(env: Env, student: Address) -> Vec<String> {
        env.storage().persistent()
            .get(&DataKey::StudentCreds(student))
            .unwrap_or(Vec::new(&env))
    }

    pub fn add_oracle(env: Env, oracle: Address) {
        let admin = get_admin(&env);
        admin.require_auth();
        storage::add_oracle(&env, &oracle);
    }

    pub fn remove_oracle(env: Env, oracle: Address) {
        let admin = get_admin(&env);
        admin.require_auth();
        env.storage().instance().remove(&DataKey::Oracle(oracle));
    }

    pub fn is_oracle(env: Env, oracle: Address) -> bool {
        is_oracle(&env, &oracle)
    }
}

#[cfg(test)]
mod tests;
