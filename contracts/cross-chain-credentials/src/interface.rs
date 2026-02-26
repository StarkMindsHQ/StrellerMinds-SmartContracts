use soroban_sdk::{Address, Env, String, Vec};
use crate::types::{ChainId, Credential, CrossChainProof, Transcript, VerificationRequest};

pub trait CrossChainCredentialsTrait {
    fn initialize(env: Env, admin: Address);
    
    // Credential Management
    fn issue_credential(env: Env, student: Address, achievement: String, metadata_hash: String, chain_id: ChainId) -> String;
    fn revoke_credential(env: Env, credential_id: String);
    fn suspend_credential(env: Env, credential_id: String);
    fn reactivate_credential(env: Env, credential_id: String);
    fn get_credential(env: Env, credential_id: String) -> Credential;
    
    // Cross-Chain Verification
    fn verify_cross_chain(env: Env, credential_id: String, target_chain: ChainId) -> CrossChainProof;
    fn submit_oracle_attestation(env: Env, credential_id: String, chain_id: ChainId, is_valid: bool);
    fn get_proof(env: Env, credential_id: String) -> CrossChainProof;
    
    // Verification Requests
    fn request_verification(env: Env, credential_id: String, chain_id: ChainId, requester: Address) -> String;
    fn get_verification_request(env: Env, request_id: String) -> VerificationRequest;
    
    // Transcript & Aggregation
    fn generate_transcript(env: Env, student: Address) -> Transcript;
    fn get_student_credentials(env: Env, student: Address) -> Vec<String>;
    
    // Oracle Management
    fn add_oracle(env: Env, oracle: Address);
    fn remove_oracle(env: Env, oracle: Address);
    fn is_oracle(env: Env, oracle: Address) -> bool;
}
